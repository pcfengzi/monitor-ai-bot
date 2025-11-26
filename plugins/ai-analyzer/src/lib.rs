use std::ffi::CString;
use std::os::raw::c_char;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use core_types::{AnomalyResult, Metric};
use dotenv::dotenv;
use plugin_api::{LogLevel, MetricSample, PluginContext, PluginMeta};

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

// ============= 插件元信息 =============

static PLUGIN_NAME: &[u8] = b"ai-analyzer\0";
static PLUGIN_VERSION: &[u8] = b"0.1.0\0";
static PLUGIN_KIND: &[u8] = b"ai\0";

// 让 host 还能调用旧 run()，但主要用 run_with_ctx
#[unsafe(no_mangle)]
pub extern "C" fn run() {
    println!("[ai-analyzer] run() 被调用（无上下文版本，仅调试用）");
}

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: PLUGIN_NAME.as_ptr() as *const c_char,
        version: PLUGIN_VERSION.as_ptr() as *const c_char,
        kind: PLUGIN_KIND.as_ptr() as *const c_char,
    }
}

// ============= run_with_ctx：AI 分析入口 =============

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) {
    dotenv().ok(); // 支持 .env

    if ctx.is_null() {
        println!("[ai-analyzer] ctx 为空，无法运行");
        return;
    }
    let ctx = unsafe { &*ctx };

    let log = |level: LogLevel, msg: &str| {
        let c = CString::new(msg).unwrap_or_else(|_| CString::new("log error").unwrap());
        (ctx.log_fn)(level, c.as_ptr());
    };

    log(LogLevel::Info, "[ai-analyzer] 开始执行 AI 分析");

    // 1. 读取配置（简单版：环境变量）
    let backend = std::env::var("AI_BACKEND").unwrap_or_else(|_| "python".to_string());
    let api_server_base =
        std::env::var("API_SERVER_BASE").unwrap_or_else(|_| "http://127.0.0.1:3001".to_string());

    log(
        LogLevel::Info,
        &format!(
            "[ai-analyzer] 使用 AI_BACKEND = {}, API_SERVER_BASE = {}",
            backend, api_server_base
        ),
    );

    // 2. 拉取 metrics
    let client = Client::new();
    let metrics_url = format!("{}/metrics", api_server_base.trim_end_matches('/'));

    let metrics: Vec<Metric> = match client.get(&metrics_url).send() {
        Ok(resp) => match resp.json() {
            Ok(m) => m,
            Err(e) => {
                log(
                    LogLevel::Error,
                    &format!("[ai-analyzer] 解析 /metrics 响应失败: {e}"),
                );
                return;
            }
        },
        Err(e) => {
            log(
                LogLevel::Error,
                &format!("[ai-analyzer] 请求 /metrics 失败: {e}"),
            );
            return;
        }
    };

    // 3. 过滤需要分析的序列，例如 cpu-monitor / cpu_usage
    let mut series: Vec<Metric> = metrics
        .into_iter()
        .filter(|m| m.plugin == "cpu-monitor" && m.name == "cpu_usage")
        .collect();

    if series.len() < 5 {
        log(
            LogLevel::Warn,
            "[ai-analyzer] cpu_usage 数据不足，跳过分析（<5 条）",
        );
        return;
    }

    // 按时间正序
    series.sort_by_key(|m| m.time);

    // 4. 根据 backend 调不同 AI
    let result = match backend.as_str() {
        "python" => call_python_ai_engine(&client, &series, &log),
        "openai" => call_openai_backend(&client, &series, &log),
        "deepseek" => call_deepseek_backend(&client, &series, &log),
        other => {
            log(
                LogLevel::Warn,
                &format!("[ai-analyzer] 未知 AI_BACKEND = {other}，默认使用 python"),
            );
            call_python_ai_engine(&client, &series, &log)
        }
    };

    let result = match result {
        Ok(r) => r,
        Err(e) => {
            log(
                LogLevel::Error,
                &format!("[ai-analyzer] 调用 AI 后端失败: {e}"),
            );
            return;
        }
    };

    // 5. 结果写日志 + 写一条 anomaly_score 指标
    log(
        if result.is_anomaly {
            LogLevel::Warn
        } else {
            LogLevel::Info
        },
        &format!(
            "[ai-analyzer] AI 分析结果: is_anomaly={}, score={:.2}, reason={:?}",
            result.is_anomaly, result.score, result.reason
        ),
    );

    // 把异常评分写成一个 Metric
    let metric_name = CString::new("anomaly_score_cpu_usage").unwrap();
    let sample = MetricSample {
        name: metric_name.as_ptr(),
        value: result.score,
        timestamp_ms: current_timestamp_ms(),
    };
    unsafe {
        (ctx.emit_metric_fn)(sample);
    }

    // 6. 如果是异常，通过 HTTP 调用 /alerts 写入告警（由 api-server 统一落库）
    if result.is_anomaly {
        if let Err(e) = report_alert_to_api(&client, &api_server_base, &result, &log) {
            log(
                LogLevel::Error,
                &format!("[ai-analyzer] 上报告警到 /alerts 失败: {e}"),
            );
        }
    }

    log(LogLevel::Info, "[ai-analyzer] 执行结束");
}

// ============= /alerts 上报 =============

#[derive(Serialize)]
struct CreateAlertReq {
    plugin: String,
    metric_name: String,
    severity: String, // "Info" | "Warning" | "Critical"
    title: String,
    message: String,
}

fn report_alert_to_api<F>(
    client: &Client,
    api_server_base: &str,
    result: &AnomalyResult,
    log: &F,
) -> anyhow::Result<()>
where
    F: Fn(LogLevel, &str),
{
    let url = format!("{}/alerts", api_server_base.trim_end_matches('/'));

    // 简单逻辑：score 越大，级别越高，你可以以后自己调规则
    let severity = if result.score > 5.0 {
        "Critical"
    } else if result.score > 3.0 {
        "Warning"
    } else {
        "Info"
    }
    .to_string();

    let title = "CPU 使用率异常".to_string();
    let message = result
        .reason
        .clone()
        .unwrap_or_else(|| "AI 检测到异常".to_string());

    let req = CreateAlertReq {
        plugin: "ai-analyzer".to_string(),
        metric_name: "cpu_usage".to_string(),
        severity,
        title,
        message,
    };

    log(
        LogLevel::Info,
        &format!("[ai-analyzer] 向 {} 上报告警...", url),
    );

    let resp = client.post(&url).json(&req).send()?;
    if !resp.status().is_success() {
        log(
            LogLevel::Error,
            &format!(
                "[ai-analyzer] /alerts 响应状态异常: {}",
                resp.status()
            ),
        );
    } else {
        log(LogLevel::Info, "[ai-analyzer] 成功上报告警到 /alerts");
    }

    Ok(())
}

// ============= AI 后端调用 =============

fn call_python_ai_engine<F>(
    client: &Client,
    series: &[Metric],
    log: &F,
) -> anyhow::Result<AnomalyResult>
where
    F: Fn(LogLevel, &str),
{
    let base = std::env::var("AI_ENGINE_BASE")
        .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
    let url = format!("{}/infer/anomaly", base.trim_end_matches('/'));

    log(
        LogLevel::Info,
        &format!("[ai-analyzer] 调用 Python AI 引擎: {}", url),
    );

    #[derive(Serialize)]
    struct MetricPointDto {
        time: DateTime<Utc>,
        value: f64,
    }

    #[derive(Serialize)]
    struct MetricSeriesDto {
        plugin: String,
        name: String,
        points: Vec<MetricPointDto>,
    }

    #[derive(Serialize)]
    struct AnomalyRequestDto {
        series: MetricSeriesDto,
    }

    #[derive(Deserialize)]
    struct AnomalyResponseDto {
        is_anomaly: bool,
        score: f64,
        reason: Option<String>,
    }

    let plugin = series[0].plugin.clone();
    let name = series[0].name.clone();
    let points: Vec<MetricPointDto> = series
        .iter()
        .map(|m| MetricPointDto {
            time: m.time,
            value: m.value,
        })
        .collect();

    let body = AnomalyRequestDto {
        series: MetricSeriesDto {
            plugin,
            name,
            points,
        },
    };

    let resp = client.post(&url).json(&body).send()?;
    let dto: AnomalyResponseDto = resp.json()?;

    Ok(AnomalyResult {
        is_anomaly: dto.is_anomaly,
        score: dto.score,
        reason: dto.reason,
    })
}

/// 下面两个是占位实现，你可以按照各家文档去填真实 HTTP 调用逻辑
fn call_openai_backend<F>(
    _client: &Client,
    _series: &[Metric],
    log: &F,
) -> anyhow::Result<AnomalyResult>
where
    F: Fn(LogLevel, &str),
{
    log(
        LogLevel::Info,
        "[ai-analyzer] 使用 OpenAI/GPT 风格模型（占位实现）",
    );

    // 示例：你可以把 series 压缩成文本 prompt，再调 Chat Completion
    // 这里只先返回一个 mock 结果
    Ok(AnomalyResult {
        is_anomaly: false,
        score: 1.5,
        reason: Some("mock_openai_backend".to_string()),
    })
}

fn call_deepseek_backend<F>(
    _client: &Client,
    _series: &[Metric],
    log: &F,
) -> anyhow::Result<AnomalyResult>
where
    F: Fn(LogLevel, &str),
{
    log(
        LogLevel::Info,
        "[ai-analyzer] 使用 DeepSeek 风格模型（占位实现）",
    );

    Ok(AnomalyResult {
        is_anomaly: true,
        score: 4.2,
        reason: Some("mock_deepseek_backend".to_string()),
    })
}

// ============= 小工具 =============

fn current_timestamp_ms() -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    now.as_millis() as i64
}
