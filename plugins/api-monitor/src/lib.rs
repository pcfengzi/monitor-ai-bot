use std::collections::HashMap;
use std::ffi::CString;
use std::fs;
use std::os::raw::c_char;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use chrono::Utc;
use dotenv::dotenv;
use plugin_api::{LogLevel, MetricSample, PluginContext, PluginMeta, PluginApiInfo};
use reqwest::blocking::Client;
use workflow_core::{
    apply_vars,
    handle_step_response,
    ExecutionContext,
    HttpMethod,
    StepResult,
    Workflow,
    WorkflowConfig,
};
use std::sync::OnceLock;


static PLUGIN_NAME: &[u8] = b"api-monitor\0";
static PLUGIN_VERSION: &[u8] = b"0.1.0\0";
static PLUGIN_KIND: &[u8] = b"workflow\0";

const API_PORT: u16 = 5501;
const API_PREFIX: &str = "/"; // 或 "/api"

fn c_string(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

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

    ctx.log_info("[api-monitor] run_with_ctx 被调用");

    // 启动 HTTP API server（只会启动一次）
    START_API_ONCE.get_or_init(|| {
        // 不能直接阻塞当前线程，开一个新线程+runtime
        std::thread::spawn(|| {
            let rt = Runtime::new().expect("create runtime");
            rt.block_on(async move {
                let app = Router::new()
                    .route("/health", get(api_health))
                    .route("/status", get(api_status));

                let addr = format!("127.0.0.1:{}", API_PORT)
                    .parse()
                    .unwrap();
                println!("[api-monitor] HTTP API 监听在 http://{}", addr);
                axum::Server::bind(&addr)
                    .serve(app.into_make_service())
                    .await
                    .unwrap();
            });
        });
    });


    log(LogLevel::Info, "[api-monitor] 开始执行工作流监控");

    // 1. 读取配置
    let cfg_path = std::env::var("API_MONITOR_CONFIG")
        .unwrap_or_else(|_| "workflows/api-monitor.toml".to_string());

    let cfg_str = match fs::read_to_string(&cfg_path) {
        Ok(s) => s,
        Err(e) => {
            log(
                LogLevel::Error,
                &format!("[api-monitor] 读取配置失败 {}: {e}", cfg_path),
            );
            return;
        }
    };

    let cfg: WorkflowConfig = match toml::from_str(&cfg_str) {
        Ok(c) => c,
        Err(e) => {
            log(
                LogLevel::Error,
                &format!("[api-monitor] 解析配置失败: {e}"),
            );
            return;
        }
    };

    let client = Client::new();

    // 可以支持多个工作流，这里依次执行
    for wf in cfg.workflows.into_iter().filter(|w| w.enabled) {
        if let Err(e) = run_single_workflow(&wf, &client, ctx, &log) {
            log(
                LogLevel::Error,
                &format!(
                    "[api-monitor] 工作流 '{}' 执行出错: {e}",
                    wf.name
                ),
            );
        }
    }

    log(LogLevel::Info, "[api-monitor] 本轮执行结束");
}

fn run_single_workflow<F>(
    wf: &Workflow,
    client: &Client,
    ctx: &PluginContext,
    log: &F,
) -> Result<()>
where
    F: Fn(LogLevel, &str),
{
    log(
        LogLevel::Info,
        &format!("[api-monitor] 开始工作流: {}", wf.name),
    );

    let start = Utc::now();

    // 初始变量：从环境变量导入（USER / PASS / EXPECTED_USER_ID 等）
    let mut exec_ctx = ExecutionContext::default();
    inject_env_vars(&mut exec_ctx.vars, &["USER", "PASS", "EXPECTED_USER_ID"]);

    let mut step_results: Vec<StepResult> = Vec::new();
    let mut all_ok = true;

    let base_url = wf.base_url.clone().unwrap_or_default();

    for step in &wf.steps {
        let step_start = Utc::now();

        // 构建 URL / headers / body，做变量替换
        let path = apply_vars(&step.path, &exec_ctx.vars);
        let url = format!("{}{}", base_url, path);

        let mut headers = HashMap::new();
        for (k, v) in &step.headers {
            headers.insert(k.clone(), apply_vars(v, &exec_ctx.vars));
        }

        let body = step
            .body
            .as_ref()
            .map(|b| apply_vars(b, &exec_ctx.vars))
            .unwrap_or_default();

        log(
            LogLevel::Info,
            &format!("[api-monitor] 步骤 {} -> {}", step.id, url),
        );

        let resp = match step.method {
            HttpMethod::GET => client.get(&url).headers(map_to_headers(&headers)).send(),
            HttpMethod::POST => client
                .post(&url)
                .headers(map_to_headers(&headers))
                .body(body)
                .send(),
            HttpMethod::PUT => client
                .put(&url)
                .headers(map_to_headers(&headers))
                .body(body)
                .send(),
            HttpMethod::DELETE => client
                .delete(&url)
                .headers(map_to_headers(&headers))
                .send(),
            HttpMethod::PATCH => client
                .patch(&url)
                .headers(map_to_headers(&headers))
                .body(body)
                .send(),
        };

        let success;
        let mut error_msg = None;
        let status;
        let text_body;

        match resp {
            Ok(r) => {
                status = r.status().as_u16();
                text_body = r.text().unwrap_or_default();

                match handle_step_response(step, status, &text_body, &mut exec_ctx) {
                    Ok(ok) => {
                        success = ok;
                        if !ok {
                            error_msg = Some(format!(
                                "断言失败，status={} body={}",
                                status, text_body
                            ));
                        }
                    }
                    Err(e) => {
                        success = false;
                        error_msg = Some(format!(
                            "处理响应失败: {e}, body={}",
                            text_body
                        ));
                    }
                }
            }
            Err(e) => {
                status = 0;
                // text_body = String::new();
                success = false;
                error_msg = Some(format!("请求失败: {e}"));
            }
        }

        let step_end = Utc::now();
        all_ok &= success;

        if success {
            log(
                LogLevel::Info,
                &format!(
                    "[api-monitor] 步骤 {} 成功, status={}",
                    step.id, status
                ),
            );
        } else {
            log(
                LogLevel::Warn,
                &format!(
                    "[api-monitor] 步骤 {} 失败, status={}, error={:?}",
                    step.id, status, error_msg
                ),
            );
        }

        step_results.push(StepResult {
            id: step.id.clone(),
            start_time: step_start,
            end_time: step_end,
            status,
            success,
            error: error_msg,
        });
    }

    let end = Utc::now();
    let duration_ms = (end - start).num_milliseconds() as f64;

    // 上报整个工作流的成功率 & 耗时
    emit_metric(ctx, "api_flow_success", if all_ok { 1.0 } else { 0.0 });
    emit_metric(ctx, "api_flow_duration_ms", duration_ms);

    log(
        if all_ok {
            LogLevel::Info
        } else {
            LogLevel::Warn
        },
        &format!(
            "[api-monitor] 工作流 '{}' 执行结束, success={}, duration_ms={}",
            wf.name, all_ok, duration_ms
        ),
    );

    Ok(())
}

fn inject_env_vars(vars: &mut HashMap<String, String>, keys: &[&str]) {
    for &k in keys {
        if let Ok(v) = std::env::var(k) {
            vars.insert(k.to_string(), v);
        }
    }
}

fn map_to_headers(map: &HashMap<String, String>) -> reqwest::header::HeaderMap {
    use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

    let mut headers = HeaderMap::new();
    for (k, v) in map {
        if let (Ok(name), Ok(value)) = (
            HeaderName::from_bytes(k.as_bytes()),
            HeaderValue::from_str(v),
        ) {
            headers.insert(name, value);
        }
    }
    headers
}

fn emit_metric(ctx: &PluginContext, name: &str, value: f64) {
    let cname = CString::new(name).unwrap_or_else(|_| CString::new("metric").unwrap());
    let sample = MetricSample {
        name: cname.as_ptr(),
        value,
        timestamp_ms: current_timestamp_ms(),
    };

    // 无需 unsafe block
    (ctx.emit_metric_fn)(sample);
}



fn current_timestamp_ms() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_millis() as i64
}

// 声明自己的 HTTP API 信息
#[unsafe(no_mangle)]
pub extern "C" fn plugin_api_info() -> PluginApiInfo {
    PluginApiInfo {
        port: API_PORT,
        prefix: c_string(API_PREFIX),
    }
}

async fn api_health() -> impl IntoResponse {
    axum::Json(serde_json::json!({
        "status": "ok",
        "plugin": NAME,
        "version": VERSION,
    }))
}

async fn api_status() -> impl IntoResponse {
    // 这里可以将 DB/metrics 里的 workflow 执行情况暴露出来
    axum::Json(serde_json::json!({
        "workflow": "demo-flow",
        "last_run": "2025-11-27T00:00:00Z",
        "success_rate": 0.98
    }))
}