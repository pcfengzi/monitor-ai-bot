use std::ffi::CString;
use std::fs;
use std::os::raw::c_char;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use dotenv::dotenv;
use plugin_api::{LogLevel, MetricSample, PluginApiInfo, PluginContext, PluginMeta};
use serde_json::{json, Value};
use workflow_core::{EngineKind, StartResult, WorkflowDefinition, WorkflowEngineRunner};

// --------- 常量 & 静态 ---------

const PLUGIN_NAME_STR: &str = "api-monitor";
const PLUGIN_VERSION_STR: &str = "0.2.0";
const PLUGIN_KIND_STR: &str = "workflow";

static PLUGIN_NAME: &[u8] = b"api-monitor\0";
static PLUGIN_VERSION: &[u8] = b"0.2.0\0";
static PLUGIN_KIND: &[u8] = b"workflow\0";

const API_PORT: u16 = 5501;
const API_PREFIX: &str = "/"; // 或 "/api"

// 只启动一次 HTTP API server
static START_API_ONCE: OnceLock<()> = OnceLock::new();

// --------- C 字符串工具 ---------

fn c_string(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

// --------- 插件 ABI：meta / run / run_with_ctx / plugin_api_info ---------

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: PLUGIN_NAME.as_ptr() as *const c_char,
        version: PLUGIN_VERSION.as_ptr() as *const c_char,
        kind: PLUGIN_KIND.as_ptr() as *const c_char,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    println!("[api-monitor] run() 无上下文版本，仅调试用");
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_api_info() -> PluginApiInfo {
    PluginApiInfo {
        port: API_PORT,
        prefix: c_string(API_PREFIX),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) {
    dotenv().ok();

    if ctx.is_null() {
        println!("[api-monitor] ctx 为空，无法运行");
        return;
    }
    let ctx = unsafe { &*ctx };

    let log = |level: LogLevel, msg: &str| {
        let c = CString::new(msg).unwrap_or_else(|_| CString::new("log error").unwrap());
        (ctx.log_fn)(level, c.as_ptr());
    };

    log(LogLevel::Info, "[api-monitor] run_with_ctx 被调用");

    // 启动插件自己的 HTTP API server（只会启动一次）
    START_API_ONCE.get_or_init(|| {
        // 不能直接阻塞当前线程，开一个新线程+runtime
        std::thread::spawn(|| {
            use axum::{routing::get, Router};
            use tokio::runtime::Runtime;

            let rt = Runtime::new().expect("create tokio runtime for api-monitor");
            rt.block_on(async move {
                let app = Router::new()
                    .route("/health", get(api_health))
                    .route("/status", get(api_status));

                let addr = format!("127.0.0.1:{}", API_PORT)
                    .parse()
                    .unwrap();
                println!("[api-monitor] HTTP API 监听在 http://{}", addr);
                if let Err(e) = axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .await
                {
                    eprintln!("[api-monitor] HTTP server error: {e}");
                }
            });
        });
    });

    log(LogLevel::Info, "[api-monitor] 开始执行 LogicFlow JSON 工作流监控");

    // 1) 从目录加载 LogicFlow JSON 定义
    // 建议使用新的环境变量：API_MONITOR_WORKFLOW_DIR
    // 默认为：workflows/api-monitor
    let wf_dir = std::env::var("API_MONITOR_WORKFLOW_DIR")
        .unwrap_or_else(|_| "workflows/api-monitor".to_string());

    let defs = match load_workflows_from_dir(&wf_dir) {
        Ok(list) => {
            if list.is_empty() {
                log(
                    LogLevel::Warn,
                    &format!(
                        "[api-monitor] 工作流目录 {} 中未发现任何 .json 定义",
                        wf_dir
                    ),
                );
            } else {
                log(
                    LogLevel::Info,
                    &format!(
                        "[api-monitor] 从目录 {} 加载 {} 个工作流定义",
                        wf_dir,
                        list.len()
                    ),
                );
            }
            list
        }
        Err(e) => {
            log(
                LogLevel::Error,
                &format!(
                    "[api-monitor] 读取工作流目录 {} 失败: {e}",
                    wf_dir
                ),
            );
            return;
        }
    };

    if defs.is_empty() {
        log(
            LogLevel::Info,
            "[api-monitor] 没有工作流可执行，本轮结束",
        );
        return;
    }

    // 2) 选择引擎类型（现在主要用 local_json，预留 flowable / zeebe）
    let engine_kind = match std::env::var("WORKFLOW_ENGINE")
        .unwrap_or_else(|_| "local_json".to_string())
        .as_str()
    {
        "local_json" => EngineKind::LocalJson,
        "flowable" => {
            log(
                LogLevel::Warn,
                "[api-monitor] WORKFLOW_ENGINE=flowable，但 FlowableEngine 目前未实现，将回退到 local_json",
            );
            EngineKind::LocalJson
        }
        "zeebe" => {
            log(
                LogLevel::Warn,
                "[api-monitor] WORKFLOW_ENGINE=zeebe，但 ZeebeEngine 目前未实现，将回退到 local_json",
            );
            EngineKind::LocalJson
        }
        other => {
            log(
                LogLevel::Warn,
                &format!(
                    "[api-monitor] 未知 WORKFLOW_ENGINE={}，将使用 local_json",
                    other
                ),
            );
            EngineKind::LocalJson
        }
    };

    let runner = WorkflowEngineRunner::new(engine_kind);

    // 3) 初始变量（USER / PASS / EXPECTED_USER_ID 等）
    let mut input_vars = serde_json::Map::new();
    if let Ok(v) = std::env::var("USER") {
        input_vars.insert("USER".to_string(), Value::String(v));
    }
    if let Ok(v) = std::env::var("PASS") {
        input_vars.insert("PASS".to_string(), Value::String(v));
    }
    if let Ok(v) = std::env::var("EXPECTED_USER_ID") {
        input_vars.insert("EXPECTED_USER_ID".to_string(), Value::String(v));
    }
    let input_vars = Value::Object(input_vars);

    // 4) 逐个执行工作流
    for def in defs {
        let key = def.key.clone();
        log(
            LogLevel::Info,
            &format!("[api-monitor] 开始执行工作流: {}", key),
        );

        let start = SystemTime::now();
        let result = run_workflow_once_blocking(&runner, &def, input_vars.clone());
        let duration_ms =
            start.elapsed().map(|d| d.as_millis() as f64).unwrap_or(0.0);

        match result {
            Ok(StartResult {
                success,
                error_message,
                ..
            }) if success => {
                log(
                    LogLevel::Info,
                    &format!(
                        "[api-monitor] 工作流 '{}' 执行成功, duration_ms={}",
                        key, duration_ms
                    ),
                );
                emit_metric(ctx, "api_flow_success", 1.0);
                emit_metric(ctx, "api_flow_duration_ms", duration_ms);
            }
            Ok(StartResult {
                success: _,
                error_message,
                ..
            }) => {
                log(
                    LogLevel::Warn,
                    &format!(
                        "[api-monitor] 工作流 '{}' 执行失败: {:?}, duration_ms={}",
                        key, error_message, duration_ms
                    ),
                );
                emit_metric(ctx, "api_flow_success", 0.0);
                emit_metric(ctx, "api_flow_duration_ms", duration_ms);
            }
            Err(e) => {
                log(
                    LogLevel::Error,
                    &format!(
                        "[api-monitor] 工作流 '{}' 执行出错: {e}, duration_ms={}",
                        key, duration_ms
                    ),
                );
                emit_metric(ctx, "api_flow_success", 0.0);
                emit_metric(ctx, "api_flow_duration_ms", duration_ms);
            }
        }
    }

    log(LogLevel::Info, "[api-monitor] 本轮执行结束");
}

// --------- 同步封装：在有/无 tokio runtime 时都能安全执行 async 引擎 ---------

fn run_workflow_once_blocking(
    runner: &WorkflowEngineRunner,
    def: &WorkflowDefinition,
    input: Value,
) -> Result<StartResult> {
    // 如果当前线程已经在 tokio runtime 里（host 就是），用 Handle::current() 来 block_on
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.block_on(runner.run_once(def, input))
    } else {
        // 否则就自己建一个 runtime
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(runner.run_once(def, input))
    }
}

// --------- 从目录加载 LogicFlow JSON 工作流定义 ---------

fn load_workflows_from_dir(dir: &str) -> Result<Vec<WorkflowDefinition>> {
    let mut defs = Vec::new();

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) => {
            return Err(anyhow::anyhow!(
                "读取目录 {} 失败: {}",
                dir,
                e
            ));
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[api-monitor] 读取 JSON 文件 {} 失败: {}",
                    path.display(),
                    e
                );
                continue;
            }
        };

        match serde_json::from_str::<WorkflowDefinition>(&content) {
            Ok(mut def) => {
                // 若 JSON 中 engine 未设置或设置为不支持的类型，这里可以兜底
                // 但通常 WorkflowDefinition 里已经有 engine 字段了
                defs.push(def);
            }
            Err(e) => {
                eprintln!(
                    "[api-monitor] 解析 JSON 工作流 {} 失败: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    Ok(defs)
}

// --------- Metric 上报工具 ---------

fn emit_metric(ctx: &PluginContext, name: &str, value: f64) {
    let cname =
        CString::new(name).unwrap_or_else(|_| CString::new("metric").unwrap());
    let sample = MetricSample {
        name: cname.as_ptr(),
        value,
        timestamp_ms: current_timestamp_ms(),
    };

    (ctx.emit_metric_fn)(sample);
}

fn current_timestamp_ms() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_millis() as i64
}

// --------- HTTP API handlers（插件自己的 API，通过插件网关暴露） ---------

use axum::{response::IntoResponse, Json};

async fn api_health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "plugin": PLUGIN_NAME_STR,
        "version": PLUGIN_VERSION_STR,
    }))
}

async fn api_status() -> impl IntoResponse {
    // TODO: 这里可以从 DB / metrics 中读取最近工作流执行情况
    Json(json!({
        "plugin": PLUGIN_NAME_STR,
        "engine": "local_json",
        "last_run": "n/a",
        "remark": "这里可以扩展为真实的工作流状态统计"
    }))
}
