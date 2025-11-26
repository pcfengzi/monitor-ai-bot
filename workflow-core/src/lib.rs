use std::collections::HashMap;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 一个完整的工作流配置文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub workflows: Vec<Workflow>,
}

/// 单个工作流
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub enabled: bool,
    /// 可选的基础 URL（如 https://api.example.com）
    pub base_url: Option<String>,
    pub steps: Vec<WorkflowStep>,
}

/// HTTP 方法
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

/// 断言（简单版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepAssert {
    /// 期望的 HTTP 状态码（可选）
    pub status: Option<u16>,
    /// JSON 路径（用 . 分割，如 data.user.id）
    pub json_path: Option<String>,
    /// 期望值（字符串对比）
    pub equals: Option<String>,
}

/// 工作流中的一步
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub method: HttpMethod,
    /// 相对路径，如 /auth/login
    pub path: String,
    /// 可选的请求体（会做变量替换）
    #[serde(default)]
    pub body: Option<String>,
    /// 请求头（值会做变量替换）
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// 从响应 JSON 中提取变量： var_name -> json_path
    #[serde(default)]
    pub extract: HashMap<String, String>,
    /// 简单断言列表
    #[serde(default)]
    pub asserts: Vec<StepAssert>,
}

/// 执行时的上下文：变量表 + 历史结果
#[derive(Debug, Default)]
pub struct ExecutionContext {
    /// 变量表，比如 {"token": "...", "user_id": "123"}
    pub vars: HashMap<String, String>,
    /// 历史步骤执行结果（可选，用于调试）
    pub results: HashMap<String, StepResult>,
}

/// 单个步骤执行结果
#[derive(Debug, Clone)]
pub struct StepResult {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub status: u16,
    pub success: bool,
    pub error: Option<String>,
}

/// 工作流执行汇总
#[derive(Debug, Clone)]
pub struct WorkflowRunSummary {
    pub name: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub success: bool,
    pub step_results: Vec<StepResult>,
}

/// 工具：将字符串中的 {{var}} 替换为变量值
pub fn apply_vars(input: &str, vars: &HashMap<String, String>) -> String {
    let mut out = input.to_string();
    for (k, v) in vars {
        let pat = format!("{{{{{}}}}}", k); // {{var}}
        out = out.replace(&pat, v);
    }
    out
}

/// 工具：从 JSON 中按 "a.b.c" 路径取值，返回字符串
pub fn get_json_path_str(value: &Value, path: &str) -> Option<String> {
    let mut cur = value;
    for seg in path.split('.') {
        cur = cur.get(seg)?;
    }
    match cur {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

/// 对某一步的响应执行 extract 和 asserts
pub fn handle_step_response(
    step: &WorkflowStep,
    resp_status: u16,
    resp_body: &str,
    ctx: &mut ExecutionContext,
) -> Result<bool> {
    // 尝试解析 JSON
    let json: Value = serde_json::from_str(resp_body).unwrap_or(Value::Null);

    // 提取变量
    for (var_name, path) in &step.extract {
        if let Some(val) = get_json_path_str(&json, path) {
            ctx.vars.insert(var_name.clone(), val);
        }
    }

    // 执行断言
    let mut all_ok = true;

    for asrt in &step.asserts {
        // 状态码断言
        if let Some(exp_status) = asrt.status {
            if resp_status != exp_status {
                all_ok = false;
            }
        }

        // JSON 值断言
        if let (Some(path), Some(exp)) = (&asrt.json_path, &asrt.equals) {
            let actual = get_json_path_str(&json, path);
            if actual.as_deref() != Some(exp.as_str()) {
                all_ok = false;
            }
        }
    }

    Ok(all_ok)
}
