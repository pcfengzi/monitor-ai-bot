use std::{
    env,
    ffi::CStr,
    fs,
    os::raw::c_char,
    path::{Path, PathBuf},
    sync::OnceLock,
    time::Duration,
};

use chrono::{DateTime, TimeZone, Utc};
use core_types::{LogEvent, LogLevel as HostLogLevel, Metric};
use dotenv::dotenv;
use libloading::{Library, Symbol};
use plugin_api::{
    LogLevel as PluginLogLevel, MetricSample, PluginContext, PluginMeta, PluginMetaFunc,
    PluginRunFunc, PluginRunWithContextFunc,
};
use serde::Deserialize;
use tokio::sync::mpsc;
use tokio::task;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use plugin_api::{PluginApiInfoFunc, PluginApiInfo};
use std::collections::HashSet;

use storage::Db;

// ⭐ 新增：线程本地存储当前正在执行的插件名
use std::cell::RefCell;
use std::thread_local;

// ============ 全局异步写入通道 ============

enum StorageMsg {
    Log(LogEvent),
    Metric(Metric),
}

static GLOBAL_SENDER: OnceLock<mpsc::UnboundedSender<StorageMsg>> = OnceLock::new();


// ⭐ 新增：当前正在执行的插件名称
thread_local! {
    static CURRENT_PLUGIN_NAME: RefCell<Option<String>> = RefCell::new(None);
}

// ============ 配置结构 ============

#[derive(Debug, Deserialize, Default, Clone)]
struct PluginConfig {
    mode: Option<String>,
    dev_dir: Option<String>,
    prod_dir: Option<String>,
    name_pattern: Option<String>,
    default_interval: Option<u64>,
}

#[derive(Debug, Deserialize, Default)]
struct AppConfig {
    plugin: Option<PluginConfig>,
}

// ============ 入口 ============

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_tracing();

    info!("=== 监控AI机器人 bot-host 启动 ===");

    let config = load_config();
    let plugin_cfg = config.plugin.clone().unwrap_or_default();
    let plugin_api_registered = std::sync::Arc::new(tokio::sync::Mutex::new(HashSet::<String>::new()));


    let mode = env::var("MONITOR_AI_PLUGIN_MODE")
        .ok()
        .or(plugin_cfg.mode.clone())
        .unwrap_or_else(|| "dev".to_string());

    let plugin_dir = resolve_plugin_dir(&mode, &plugin_cfg);
    let plugin_ext = plugin_ext();
    let name_pattern = plugin_cfg
        .name_pattern
        .clone()
        .unwrap_or_else(|| "_monitor".to_string());
    let default_interval = plugin_cfg.default_interval.unwrap_or(5);

    info!(
        "运行模式: {mode}, 插件目录: {}, 扩展名: {}, 名称包含: \"{}\"",
        plugin_dir.display(),
        plugin_ext,
        name_pattern
    );

    // 初始化数据库
    let db_url = std::env::var("MONITOR_AI_DB_URL")
        .unwrap_or_else(|_| "sqlite://database/monitor_ai.db".to_string());
    let db = Db::connect(&db_url).await.expect("连接数据库失败");
    info!("已连接 SQLite 数据库: {db_url}");

    // 初始化全局 sender + 异步存储任务
    let (tx, mut rx) = mpsc::unbounded_channel::<StorageMsg>();
    GLOBAL_SENDER.set(tx).expect("GLOBAL_SENDER 已初始化");

    let db_clone = db.clone();
    task::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                StorageMsg::Log(e) => {
                    if let Err(e) = db_clone.insert_log(&e).await {
                        error!("写入日志失败: {e}");
                    }
                }
                StorageMsg::Metric(m) => {
                    if let Err(e) = db_clone.insert_metric(&m).await {
                        error!("写入指标失败: {e}");
                    }
                }
            }
        }
    });

    // 扫描插件
    let plugins = discover_plugins(&plugin_dir, plugin_ext);
    if plugins.is_empty() {
        info!("未发现任何插件动态库，确认已构建插件。");
    } else {
        info!("发现 {} 个插件：", plugins.len());
        for p in &plugins {
            info!("  - {}", p.display());
        }
    }

    // 简单调度循环：每轮执行所有插件
    loop {
        run_plugins_once(&plugins, &db, plugin_api_registered.clone()).await;
        tokio::time::sleep(Duration::from_secs(default_interval)).await;
    }
}

// ============ tracing 初始化 ============

fn init_tracing() {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}

// ============ 配置加载 ============

fn load_config() -> AppConfig {
    let path = PathBuf::from("config.toml");
    match fs::read_to_string(&path) {
        Ok(content) => match toml::from_str::<AppConfig>(&content) {
            Ok(cfg) => {
                info!("已加载配置文件: {}", path.display());
                cfg
            }
            Err(e) => {
                error!("解析配置文件 {} 失败: {e}，使用默认配置", path.display());
                AppConfig::default()
            }
        },
        Err(_) => {
            info!("未找到配置文件 {}，使用默认配置", path.display());
            AppConfig::default()
        }
    }
}

fn resolve_plugin_dir(mode: &str, cfg: &PluginConfig) -> PathBuf {
    match mode {
        "prod" | "release" => PathBuf::from(
            cfg.prod_dir
                .clone()
                .unwrap_or_else(|| "plugins-bin".to_string()),
        ),
        _ => PathBuf::from(
            cfg.dev_dir
                .clone()
                .unwrap_or_else(|| "target/debug".to_string()),
        ),
    }
}

fn plugin_ext() -> &'static str {
    if cfg!(target_os = "windows") {
        "dll"
    } else if cfg!(target_os = "macos") {
        "dylib"
    } else {
        "so"
    }
}

// ============ 扫描插件 ============

fn discover_plugins(dir: &Path, ext: &str) -> Vec<PathBuf> {
    let mut result = Vec::new();

    let read_dir = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(e) => {
            error!("无法读取插件目录 {}: {e}", dir.display());
            return result;
        }
    };

    for entry in read_dir {
        if let Ok(entry) = entry {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // 只检查扩展名
            let is_ext_ok = path
                .extension()
                .and_then(|s| s.to_str())
                .map(|s| s.eq_ignore_ascii_case(ext))
                .unwrap_or(false);

            if !is_ext_ok {
                continue;
            }

            result.push(path);
        }
    }

    result
}

// ============ 插件调度 ============

async fn run_plugins_once(
    plugins: &[PathBuf],
    db: &Db,
    plugin_api_registered: std::sync::Arc<tokio::sync::Mutex<HashSet<String>>>,
) {
    for path in plugins {
        info!("执行插件: {}", path.display());

        let lib = match unsafe { Library::new(path) } {
            Ok(lib) => lib,
            Err(e) => {
                error!("加载插件 {} 失败: {e}", path.display());
                continue;
            }
        };

        unsafe {
            let meta_func: Symbol<PluginMetaFunc> = match lib.get(b"meta") {
                Ok(sym) => sym,
                Err(e) => {
                    error!("插件 {} 缺少 meta 函数: {e}", path.display());
                    continue;
                }
            };
            let meta: PluginMeta = meta_func();
            let plugin_name =
                c_str_to_string(meta.name).unwrap_or_else(|| "<unknown>".to_string());
            let plugin_version =
                c_str_to_string(meta.version).unwrap_or_else(|| "<unknown>".to_string());
            let plugin_kind =
                c_str_to_string(meta.kind).unwrap_or_else(|| "<unknown>".to_string());

            info!(
                "插件信息: name={}, version={}, kind={}",
                plugin_name, plugin_version, plugin_kind
            );

            // ⭐ 在当前线程标记“当前插件名”，给日志和指标桥接使用
            CURRENT_PLUGIN_NAME.with(|slot| {
                *slot.borrow_mut() = Some(plugin_name.clone());
            });

            // ⭐ 新增：尝试读取插件的 API 信息，并写入 DB
            if let Ok(api_info_fn) = lib.get::<PluginApiInfoFunc>(b"plugin_api_info") {
                let info: PluginApiInfo = api_info_fn();

                let prefix = c_str_to_string(info.prefix).unwrap_or_else(|| "/".to_string());
                let base_url = format!("http://127.0.0.1:{}{}", info.port, prefix);

                {
                    let mut guard = plugin_api_registered.lock().await;
                    if !guard.contains(&plugin_name) {
                        // 第一次注册，写入 DB
                        if let Err(e) = db.upsert_plugin_api(&plugin_name, &base_url).await {
                            error!(
                                "注册插件 API 失败: plugin={}, base_url={}, err={e}",
                                plugin_name, base_url
                            );
                        } else {
                            info!(
                                "已注册插件 API: plugin={}, base_url={}",
                                plugin_name, base_url
                            );
                            guard.insert(plugin_name.clone());
                        }
                    }
                }
            } else {
                // 没有 plugin_api_info，说明该插件不暴露 HTTP API，直接略过
            }



            let run_with_ctx: Result<Symbol<PluginRunWithContextFunc>, _> =
                lib.get(b"run_with_ctx");

            if let Ok(run_with_ctx) = run_with_ctx {
                let mut ctx = PluginContext {
                    host_version: 1,
                    log_fn: host_log_bridge,
                    emit_metric_fn: host_emit_metric_bridge,
                };

                info!("调用 run_with_ctx()...");
                run_with_ctx(&mut ctx as *mut PluginContext);
            } else {
                let run_func: Result<Symbol<PluginRunFunc>, _> = lib.get(b"run");
                match run_func {
                    Ok(run) => {
                        info!("调用旧版 run()...");
                        run();
                    }
                    Err(e) => {
                        error!(
                            "插件 {} 既没有 run_with_ctx 也没有 run: {e}",
                            path.display()
                        );
                    }
                }
            }

            // ⭐ 执行完清空当前插件名，避免污染后续调用
            CURRENT_PLUGIN_NAME.with(|slot| {
                *slot.borrow_mut() = None;
            });
        }
        // ⭐ 关键：不要在这里让 lib 被 drop（否则起后台 HTTP 线程的插件会崩）
        //    对于已经成功执行过 run_with_ctx / run 的插件，我们让它常驻内存
        std::mem::forget(lib);
    }
}

// ============ FFI 桥接：Log & Metric ============

extern "C" fn host_log_bridge(level: PluginLogLevel, msg: *const c_char) {
    if msg.is_null() {
        return;
    }

    let message = match unsafe { CStr::from_ptr(msg).to_str() } {
        Ok(s) => s.to_string(),
        Err(_) => "<invalid utf-8>".to_string(),
    };

    // ⭐ 读取当前插件名
    let plugin_name_opt = CURRENT_PLUGIN_NAME.with(|slot| slot.borrow().clone());
    let plugin_label = plugin_name_opt
        .as_deref()
        .unwrap_or("<unknown-plugin>");

    let host_level = match level {
        PluginLogLevel::Debug => HostLogLevel::Debug,
        PluginLogLevel::Info => HostLogLevel::Info,
        PluginLogLevel::Warn => HostLogLevel::Warn,
        PluginLogLevel::Error => HostLogLevel::Error,
    };

    // 写入 DB 的事件，现在带上 plugin 字段
    let event = LogEvent {
        time: Utc::now(),
        level: host_level,
        plugin: plugin_name_opt.clone(),
        message: message.clone(),
        fields: Default::default(),
    };

    if let Some(sender) = GLOBAL_SENDER.get() {
        let _ = sender.send(StorageMsg::Log(event));
    }

    // 控制台日志也加上插件名前缀
    let decorated = format!("[{plugin_label}] {message}");

    match host_level {
        HostLogLevel::Debug => tracing::debug!("{decorated}"),
        HostLogLevel::Info => tracing::info!("{decorated}"),
        HostLogLevel::Warn => tracing::warn!("{decorated}"),
        HostLogLevel::Error => tracing::error!("{decorated}"),
    }
}

extern "C" fn host_emit_metric_bridge(sample: MetricSample) {
    let name = if sample.name.is_null() {
        "<unnamed>".to_string()
    } else {
        c_str_to_string(sample.name).unwrap_or_else(|| "<invalid metric name>".to_string())
    };

    let time = timestamp_ms_to_datetime(sample.timestamp_ms);

    // ⭐ 从线程本地拿当前插件名，默认 unknown
    let plugin_name = CURRENT_PLUGIN_NAME.with(|slot| {
        slot.borrow()
            .clone()
            .unwrap_or_else(|| "unknown".to_string())
    });

    let metric = Metric {
        time,
        plugin: plugin_name,
        name,
        value: sample.value,
        labels: Default::default(),
    };

    if let Some(sender) = GLOBAL_SENDER.get() {
        let _ = sender.send(StorageMsg::Metric(metric));
    }
}

// ============ 小工具函数 ============

fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string()) }
}

fn timestamp_ms_to_datetime(ms: i64) -> DateTime<Utc> {
    match Utc.timestamp_millis_opt(ms) {
        chrono::LocalResult::Single(dt) => dt,
        _ => Utc::now(),
    }
}
