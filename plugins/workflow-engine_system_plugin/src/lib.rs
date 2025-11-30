use std::{
    ffi::{CStr, CString},
    fs,
    os::raw::c_char,
    path::Path,
    sync::OnceLock,
};

use axum::{
    extract::{Path as AxumPath, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use plugin_api::{
    LogLevel, MetricSample, PluginApiInfo, PluginContext, PluginMeta,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{error, info};
use workflow_core::{EngineKind, WorkflowDefinition, WorkflowEngineRunner};
use tokio::net::TcpListener;


/// 插件元信息常量
static NAME: &[u8] = b"workflow-engine_system_plugin\0";
static VERSION: &[u8] = b"0.1.0\0";
static KIND: &[u8] = b"workflow\0";
static API_PREFIX: &[u8] = b"/\0"; // 由 api-server 负责加 /plugin-api/workflow-engine_system_plugin 前缀

/// 只启动一次 HTTP server
static SERVER_STARTED: OnceLock<()> = OnceLock::new();

/// 插件内部共享状态
#[derive(Clone)]
struct AppState {
    /// 所有加载到内存的工作流定义
    workflows: std::sync::Arc<RwLock<Vec<WorkflowDefinition>>>,
    /// 当前选用的执行引擎（LocalJson / Flowable / Zeebe）
    engine_kind: EngineKind,
}

/// 列出工作流时的精简信息
#[derive(Serialize)]
struct WorkflowSummary {
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub engine: String,
}

/// POST /workflows/:key/run 的请求体
#[derive(Deserialize)]
struct RunWorkflowRequest {
    /// 传入给工作流的输入变量（例如 { "USER": "...", "PASS": "..." }）
    pub input: Option<Value>,
}

/// 执行结果的返回体
#[derive(Serialize)]
struct RunWorkflowResponse {
    pub success: bool,
    pub duration_ms: i64,
    pub output: Value,
}

/// ====== 插件必需导出的 ABI ======

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: NAME.as_ptr() as *const c_char,
        version: VERSION.as_ptr() as *const c_char,
        kind: KIND.as_ptr() as *const c_char,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_api_info() -> PluginApiInfo {
    PluginApiInfo {
        // 约定 workflow-engine_system_plugin 插件监听 5601 端口，如有需要可以改成从 env 读取
        port: 5601,
        prefix: API_PREFIX.as_ptr() as *const c_char,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };

    // 简单日志函数：通过 host 的 log_fn 上报
    log_info(ctx, "[workflow-engine_system_plugin] run_with_ctx called");

    // 确保 HTTP server 只启动一次
    if SERVER_STARTED.set(()).is_err() {
        log_info(ctx, "[workflow-engine_system_plugin] server already started, skip");
        return;
    }

    // 1. 加载 workflows 目录下的 LogicFlow JSON
    let defs = load_workflows_from_dir("workflows");
    log_info(
        ctx,
        &format!(
            "[workflow-engine_system_plugin] loaded {} workflow definition(s) from ./workflows",
            defs.len()
        ),
    );

    // 2. 根据环境变量选择引擎类型
    let engine_kind = match std::env::var("WORKFLOW_ENGINE")
        .unwrap_or_else(|_| "local_json".to_string())
        .as_str()
    {
        "flowable" => EngineKind::Flowable,
        "zeebe" => EngineKind::Zeebe,
        _ => EngineKind::LocalJson,
    };

    log_info(
        ctx,
        &format!("[workflow-engine_system_plugin] engine kind = {:?}", engine_kind),
    );

    let state = AppState {
        workflows: std::sync::Arc::new(RwLock::new(defs)),
        engine_kind,
    };

    // 3. 启动 HTTP 服务（Axum）
    //    不要把 ctx 指针 move 进异步任务，只在这里用 log 即可
    tokio::spawn(async move {
        let app = Router::new()
            .route("/health", get(health_check))
            .route("/workflows", get(list_workflows))
            .route("/workflows/:key/run", post(run_workflow_once))
            .with_state(state);

        let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 5601));
        info!("[workflow-engine_system_plugin] listening on http://{}", addr);

        // ✅ axum 0.7 写法：先用 TcpListener 绑定端口
        let listener = TcpListener::bind(addr)
            .await
            .expect("[workflow-engine_system_plugin] failed to bind TCP listener");

        // ✅ 然后用 axum::serve(listener, app)
        if let Err(e) = axum::serve(listener, app).await {
            error!("[workflow-engine_system_plugin] HTTP server error: {e}");
        }
    });
    // 4. 上报一条“启动成功”的 Metric（可选）
    emit_heartbeat_metric(ctx);
}

/// ====== HTTP Handlers ======

async fn health_check() -> &'static str {
    "OK"
}

async fn list_workflows(
    State(state): State<AppState>,
) -> Json<Vec<WorkflowSummary>> {
    let defs = state.workflows.read().await;

    let engine_str = match state.engine_kind {
        EngineKind::LocalJson => "local_json",
        EngineKind::Flowable => "flowable",
        EngineKind::Zeebe => "zeebe",
    }
    .to_string();

    let list = defs
        .iter()
        .map(|def| WorkflowSummary {
            key: def.key.clone(),
            name: def.name.clone(),
            description: Some(def.description.clone()),
            engine: engine_str.clone(),
        })
        .collect();

    Json(list)
}

async fn run_workflow_once(
    State(state): State<AppState>,
    AxumPath(key): AxumPath<String>,
    Json(payload): Json<RunWorkflowRequest>,
) -> Result<Json<RunWorkflowResponse>, StatusCode> {
    // 1. 找到对应的 WorkflowDefinition
    let defs = state.workflows.read().await;
    let def = defs
        .iter()
        .find(|d| d.key == key)
        .ok_or(StatusCode::NOT_FOUND)?;

    // 2. 构造执行器
    let runner = WorkflowEngineRunner::new(state.engine_kind);

    let input = payload.input.unwrap_or_else(|| Value::Object(serde_json::Map::new()));

    // 3. 执行一次
    let result = runner
        .run_once(def, input)
        .await
        .map_err(|e| {
            error!("[workflow-engine_system_plugin] run workflow `{}` error: {e}", key);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let resp = RunWorkflowResponse {
        success: result.success,
        duration_ms: result.duration_ms,
        output: result.output,
    };

    Ok(Json(resp))
}

/// ====== 工具函数：加载工作流定义 ======

fn load_workflows_from_dir(dir: &str) -> Vec<WorkflowDefinition> {
    let mut result = Vec::new();
    let path = Path::new(dir);

    let entries = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(e) => {
            eprintln!(
                "[workflow-engine_system_plugin] cannot read workflows dir {}: {e}",
                path.display()
            );
            return result;
        }
    };

    for entry in entries {
        let Ok(entry) = entry else {
            continue;
        };
        let file_path = entry.path();
        if !file_path.is_file() {
            continue;
        }

        // 只处理 .json
        let is_json = file_path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("json"))
            .unwrap_or(false);
        if !is_json {
            continue;
        }

        let file_name = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        match fs::read_to_string(&file_path) {
            Ok(content) => match serde_json::from_str::<Value>(&content) {
                Ok(json) => {
                    // 约定：workflow-core 提供 from_logicflow_json 构造器
                    let def = WorkflowDefinition::from_logicflow_json(&file_name, json);
                    result.push(def);
                }
                Err(e) => {
                    eprintln!(
                        "[workflow-engine_system_plugin] invalid JSON in {}: {e}",
                        file_path.display()
                    );
                }
            },
            Err(e) => {
                eprintln!(
                    "[workflow-engine_system_plugin] read file {} error: {e}",
                    file_path.display()
                );
            }
        }
    }

    result
}

/// ====== Host 日志 & Metric 工具 ======

fn log_info(ctx: &PluginContext, msg: &str) {
    let c_msg = CString::new(msg).unwrap_or_else(|_| CString::new("log error").unwrap());
    (ctx.log_fn)(LogLevel::Info, c_msg.as_ptr());
}

fn emit_heartbeat_metric(ctx: &PluginContext) {
    let name = CString::new("workflow_engine_heartbeat")
        .unwrap_or_else(|_| CString::new("workflow_engine_heartbeat_fallback").unwrap());

    let timestamp_ms = current_timestamp_ms();

    let sample = MetricSample {
        name: name.as_ptr(),
        value: 1.0,
        timestamp_ms,
    };

    unsafe {
        (ctx.emit_metric_fn)(sample);
    }
}

fn current_timestamp_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_millis() as i64
}

/// 如果你需要从 C 字符串转 Rust String（暂时没用到，预留）
#[allow(dead_code)]
fn c_str_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(ptr).to_str().ok().map(|s| s.to_string()) }
}
