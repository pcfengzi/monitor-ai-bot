//! workflow-core
//!
//! 目标：
//! 1. 支持 LogicFlow JSON 工作流执行（LocalJsonEngine）
//! 2. 预留 Flowable / Zeebe 集成的 Engine 接口，将来可以无缝切换。

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

//
// ================== LogicFlow Graph 模型 ==================
//
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicFlowNode {
    pub id: String,

    /// 节点类型（例如：start / http / extract / assert / gateway 等）
    #[serde(rename = "type")]
    pub node_type: String,

    /// 节点的自定义属性（前端可以随便挂）
    #[serde(default)]
    pub properties: Value,
}

/// LogicFlow 的边（连线）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicFlowEdge {
    pub id: String,
    pub source: String,
    pub target: String,

    #[serde(default)]
    pub properties: Value,
}

/// 完整的 LogicFlow 图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicFlowGraph {
    pub nodes: Vec<LogicFlowNode>,
    pub edges: Vec<LogicFlowEdge>,
}

//
// ================== 工作流定义 & 引擎类型 ==================
///

/// 工作流引擎类型：
/// - LocalJson  : 本地 JSON 执行器（当前真正实现）
/// - Flowable   : 预留，将来对接 Flowable 引擎
/// - Zeebe      : 预留，将来对接 Zeebe / Camunda 8
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineKind {
    LocalJson,
    Flowable,
    Zeebe,
}

/// 一个完整的工作流定义：
/// - key   : 流程 key（逻辑名称，唯一标识）
/// - engine: 使用哪个引擎
/// - graph : LogicFlow 导出的图
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// 流程 key（逻辑唯一标识）
    pub key: String,

    /// 流程名称（一般用于展示，可等于 key）
    #[serde(default)]
    pub name: String,

    /// 流程描述
    #[serde(default)]
    pub description: String,

    /// 使用的引擎
    pub engine: EngineKind,

    /// LogicFlow 导出的图
    pub graph: LogicFlowGraph,
}

// ⚠️ 注意：从这里开始是 impl，不要写进 struct 里面
impl WorkflowDefinition {
    /// 从 LogicFlow JSON 构建工作流定义
    ///
    /// - `key` 一般可以传文件名（不含扩展名）或你想要的流程 key
    /// - `graph_json` 是 LogicFlow 导出的完整 JSON（包含 nodes / edges）
    pub fn from_logicflow_json(key: &str, graph_json: Value) -> WorkflowDefinition {
        let graph: LogicFlowGraph = serde_json::from_value(graph_json)
            .expect("invalid LogicFlow graph json");

        WorkflowDefinition {
            key: key.to_string(),
            name: key.to_string(),
            description: String::new(),
            engine: EngineKind::LocalJson,
            graph,
        }
    }
}


/// 部署结果（对 LocalJson 来说没啥特别意义；对 Flowable/Zeebe 就是部署 ID）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployResult {
    pub key: String,
    pub engine: EngineKind,
    pub raw: Value,
}

/// 执行结果（统一抽象）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartResult {
    pub instance_id: String,
    pub engine: EngineKind,
    pub raw: Value,
    pub success: bool,
    pub error_message: Option<String>,

    /// 工作流总耗时（毫秒）
    pub duration_ms: i64,    // 这里用 i64

    /// 统一输出（给插件简化使用），这里我们约定为 { summary, vars }
    pub output: Value,
}


//
// ================== 执行上下文（本地引擎用） ==================
///

/// 单步执行结果（保留你原来 WorkflowRunSummary 的风格，方便以后接 Metric）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub success: bool,
    pub status: Option<u16>,
    pub error: Option<String>,
}

/// 工作流执行汇总
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRunSummary {
    pub key: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub success: bool,
    pub steps: Vec<StepResult>,
}

/// 执行时上下文（本地 JSON 执行器用）
/// - vars         : 变量表（{{var}}）
#[derive(Debug, Default)]
pub struct ExecutionContext {
    pub vars: HashMap<String, String>,
    pub last_response: Option<Value>,
    pub logs: Vec<String>,
    pub step_results: HashMap<String, StepResult>,
}

//
// ================== 工具函数（保留你原来的 flavor） ==================
///

/// 将字符串中的 {{var}} 替换为变量值
pub fn apply_vars(input: &str, vars: &HashMap<String, String>) -> String {
    let mut out = input.to_string();
    for (k, v) in vars {
        let pat = format!("{{{{{}}}}}", k); // {{var}}
        out = out.replace(&pat, v);
    }
    out
}

/// 从 JSON 按 "a.b.c" 路径取值，返回字符串
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

//
// ================== 引擎通用 trait ==================
///

#[async_trait]
pub trait WorkflowEngine: Send + Sync {
    /// 部署流程（LocalJson 可以只是回显）
    async fn deploy(&self, def: &WorkflowDefinition) -> Result<DeployResult>;

    /// 执行一次流程
    async fn run_once(&self, def: &WorkflowDefinition, input: Value) -> Result<StartResult>;
}

//
// ================== 统一 Runner（对外只用这个） ==================
///

pub struct WorkflowEngineRunner {
    kind: EngineKind,
    local: LocalJsonEngine,
    flowable: Option<FlowableEngine>,
    zeebe: Option<ZeebeEngine>,
}

impl WorkflowEngineRunner {
    /// 只传一个 EngineKind，当前只有 LocalJson 真正可用。
    pub fn new(kind: EngineKind) -> Self {
        Self {
            kind,
            local: LocalJsonEngine::new(),
            flowable: None,
            zeebe: None,
        }
    }

    /// 预留：注入 Flowable 客户端配置
    pub fn with_flowable(mut self, engine: FlowableEngine) -> Self {
        self.flowable = Some(engine);
        self
    }

    /// 预留：注入 Zeebe 客户端配置
    pub fn with_zeebe(mut self, engine: ZeebeEngine) -> Self {
        self.zeebe = Some(engine);
        self
    }

    pub async fn deploy(&self, def: &WorkflowDefinition) -> Result<DeployResult> {
        match self.kind {
            EngineKind::LocalJson => self.local.deploy(def).await,
            EngineKind::Flowable => {
                let engine = self.flowable.as_ref().ok_or_else(|| {
                    anyhow!("FlowableEngine 未初始化，但 EngineKind=Flowable")
                })?;
                engine.deploy(def).await
            }
            EngineKind::Zeebe => {
                let engine = self.zeebe.as_ref().ok_or_else(|| {
                    anyhow!("ZeebeEngine 未初始化，但 EngineKind=Zeebe")
                })?;
                engine.deploy(def).await
            }
        }
    }

    pub async fn run_once(&self, def: &WorkflowDefinition, input: Value) -> Result<StartResult> {
        match self.kind {
            EngineKind::LocalJson => self.local.run_once(def, input).await,
            EngineKind::Flowable => {
                let engine = self.flowable.as_ref().ok_or_else(|| {
                    anyhow!("FlowableEngine 未初始化，但 EngineKind=Flowable")
                })?;
                engine.run_once(def, input).await
            }
            EngineKind::Zeebe => {
                let engine = self.zeebe.as_ref().ok_or_else(|| {
                    anyhow!("ZeebeEngine 未初始化，但 EngineKind=Zeebe")
                })?;
                engine.run_once(def, input).await
            }
        }
    }
}

//
// ================== LocalJsonEngine 实现 ==================
///

/// 本地执行 LogicFlow JSON 的引擎
pub struct LocalJsonEngine {
    http: reqwest::Client,
}

impl LocalJsonEngine {
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }
}

#[async_trait]
impl WorkflowEngine for LocalJsonEngine {
    async fn deploy(&self, def: &WorkflowDefinition) -> Result<DeployResult> {
        // 本地执行器不需要真正“部署”，返回一下 graph 即可
        Ok(DeployResult {
            key: def.key.clone(),
            engine: EngineKind::LocalJson,
            raw: serde_json::json!({
                "graph": def.graph,
            }),
        })
    }

    async fn run_once(&self, def: &WorkflowDefinition, input: Value) -> Result<StartResult> {
        let wf_start = Utc::now();
        let mut ctx = ExecutionContext::default();

        // 把 input 里的 key/value 灌入 vars，方便 {{USER}} 这种占位符
        if let Value::Object(map) = input {
            for (k, v) in map {
                if let Some(s) = v.as_str() {
                    ctx.vars.insert(k, s.to_string());
                } else {
                    ctx.vars.insert(k, v.to_string());
                }
            }
        }

        let ordered_nodes = topological_order(&def.graph)?;

        for node in ordered_nodes {
            let step_start = Utc::now();
            let mut step_result = StepResult {
                id: node.id.clone(),
                start_time: step_start,
                end_time: step_start,
                success: true,
                status: None,
                error: None,
            };

            let r = match node.node_type.as_str() {
                "start" => {
                    ctx.logs.push(format!("start node: {}", node.id));
                    Ok(())
                }
                "http" => execute_http_node(&self.http, &node, &mut ctx).await,
                "extract" => execute_extract_node(&node, &mut ctx),
                "assert" => execute_assert_node(&node, &mut ctx),
                other => {
                    ctx.logs.push(format!(
                        "unknown node type: {} for node {}, skip",
                        other, node.id
                    ));
                    Ok(())
                }
            };

            step_result.end_time = Utc::now();

            if let Err(e) = r {
                step_result.success = false;
                step_result.error = Some(e.to_string());
            }

            // 从 vars 中读取该 node 的 http status（如果是 http 节点）
            if node.node_type == "http" {
                if let Some(status_str) = ctx
                    .vars
                    .get(&format!("{}_status", node.id))
                    .cloned()
                {
                    if let Ok(code) = status_str.parse::<u16>() {
                        step_result.status = Some(code);
                    }
                }
            }

            ctx.step_results.insert(node.id.clone(), step_result);
        }

        // ✅ 在这里定义 all_ok，后面统一复用
        let all_ok = ctx.step_results.values().all(|s| s.success);

        let wf_end = Utc::now();
        let summary = WorkflowRunSummary {
            key: def.key.clone(),
            start_time: wf_start,
            end_time: wf_end,
            success: all_ok,
            steps: ctx.step_results.values().cloned().collect(),
        };

        // 计算总耗时（毫秒，i64）
        let duration_ms = (wf_end - wf_start).num_milliseconds();

        // 给插件使用的统一输出（精简版）
        let output = serde_json::json!({
            "summary": summary,
            "vars": ctx.vars,
        });

        Ok(StartResult {
            instance_id: Uuid::new_v4().to_string(),
            engine: EngineKind::LocalJson,
            raw: serde_json::json!({
                "summary": summary,
                "logs": ctx.logs,
                "vars": ctx.vars,
            }),
            success: all_ok,
            error_message: if all_ok {
                None
            } else {
                Some("one or more steps failed".to_string())
            },
            duration_ms,
            output,
        })
    }
}


//
// ================== LocalJson 辅助逻辑 ==================
///

/// 简单根据 edges 做一个拓扑排序，只支持 DAG/线性流程
/// 这里你可以根据 LogicFlow 的节点类型做更复杂的分支、网关等。
fn topological_order(graph: &LogicFlowGraph) -> Result<Vec<LogicFlowNode>> {
    let mut indeg: HashMap<&str, usize> = HashMap::new();
    for n in &graph.nodes {
        indeg.insert(&n.id, 0);
    }
    for e in &graph.edges {
        if let Some(v) = indeg.get_mut(e.target.as_str()) {
            *v += 1;
        }
    }

    let mut queue: Vec<&LogicFlowNode> = graph
        .nodes
        .iter()
        .filter(|n| indeg.get(n.id.as_str()).copied().unwrap_or(0) == 0)
        .collect();

    let mut order = Vec::new();
    let mut visited = HashSet::new();

    while let Some(node) = queue.pop() {
        if !visited.insert(node.id.as_str()) {
            continue;
        }
        order.push(node.clone());

        for e in graph.edges.iter().filter(|e| e.source == node.id) {
            if let Some(v) = indeg.get_mut(e.target.as_str()) {
                if *v > 0 {
                    *v -= 1;
                    if *v == 0 {
                        if let Some(n2) = graph.nodes.iter().find(|n| n.id == e.target) {
                            queue.push(n2);
                        }
                    }
                }
            }
        }
    }

    if order.is_empty() {
        return Err(anyhow!("graph is empty or invalid"));
    }

    Ok(order)
}

fn prop_str<'a>(props: &'a Value, key: &str, default: &'a str) -> &'a str {
    props
        .get(key)
        .and_then(|v| v.as_str())
        .unwrap_or(default)
}

async fn execute_http_node(
    http: &reqwest::Client,
    node: &LogicFlowNode,
    ctx: &mut ExecutionContext,
) -> Result<()> {
    let method = prop_str(&node.properties, "method", "GET");
    let url_tpl = prop_str(&node.properties, "url", "");
    let url = apply_vars(url_tpl, &ctx.vars);

    let mut req = match method {
        "POST" => http.post(&url),
        "PUT" => http.put(&url),
        "DELETE" => http.delete(&url),
        _ => http.get(&url),
    };

    // headers（可选）
    if let Some(h) = node.properties.get("headers").and_then(|v| v.as_object()) {
        let mut headers_map = HeaderMap::new();
        for (k, v) in h {
            if let Some(s) = v.as_str() {
                let val = apply_vars(s, &ctx.vars);

                let header_name = HeaderName::from_bytes(k.as_bytes())
                    .map_err(|e| anyhow!("invalid header name {}: {}", k, e))?;
                let header_value = HeaderValue::from_str(&val)
                    .map_err(|e| anyhow!("invalid header value for {}: {}", k, e))?;

                headers_map.insert(header_name, header_value);
            }
        }
        req = req.headers(headers_map);
    }

    // body（可选）
    if let Some(body) = node.properties.get("body") {
        if let Some(s) = body.as_str() {
            let body_rendered = apply_vars(s, &ctx.vars);
            req = req.body(body_rendered);
        } else {
            req = req.json(body);
        }
    }

    let resp = req.send().await?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();

    ctx.logs.push(format!(
        "[http] node={} {} {} -> {}",
        node.id, method, url, status
    ));

    // 尝试解析 JSON
    let json = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
    ctx.last_response = Some(json.clone());

    ctx.vars
        .insert(format!("{}_status", node.id), status.as_u16().to_string());
    ctx.vars.insert(format!("{}_body", node.id), text);

    Ok(())
}

fn execute_extract_node(node: &LogicFlowNode, ctx: &mut ExecutionContext) -> Result<()> {
    let Some(ref resp) = ctx.last_response else {
        return Err(anyhow!(
            "extract node {} but last_response is None",
            node.id
        ));
    };

    let path = prop_str(&node.properties, "path", "");
    let var_name = prop_str(&node.properties, "var", "");

    if path.is_empty() || var_name.is_empty() {
        return Err(anyhow!(
            "extract node {} missing path or var",
            node.id
        ));
    }

    if let Some(val) = get_json_path_str(resp, path) {
        ctx.vars.insert(var_name.to_string(), val.clone());
        ctx.logs.push(format!(
            "[extract] node={} path={} -> var={}={}",
            node.id, path, var_name, val
        ));
    } else {
        return Err(anyhow!(
            "extract node {} path {} not found or not scalar",
            node.id,
            path
        ));
    }

    Ok(())
}

fn execute_assert_node(node: &LogicFlowNode, ctx: &mut ExecutionContext) -> Result<()> {
    let Some(ref resp) = ctx.last_response else {
        return Err(anyhow!(
            "assert node {} but last_response is None",
            node.id
        ));
    };

    let path = prop_str(&node.properties, "path", "");
    if path.is_empty() {
        return Err(anyhow!("assert node {} missing path", node.id));
    }

    let expected = node.properties.get("equals").unwrap_or(&Value::Null);
    let expected_str = if let Some(s) = expected.as_str() {
        apply_vars(s, &ctx.vars)
    } else {
        expected.to_string()
    };

    let actual = get_json_path_str(resp, path).unwrap_or_default();

    if actual != expected_str {
        return Err(anyhow!(
            "assert failed at node {} path {}: actual={}, expected={}",
            node.id,
            path,
            actual,
            expected_str
        ));
    }

    ctx.logs.push(format!(
        "[assert] node={} path={} equals={} ok",
        node.id, path, expected_str
    ));

    Ok(())
}

//
// ================== Flowable / Zeebe 引擎预留 ==================
///

/// Flowable 引擎预留壳，用于将来对接 Flowable REST API
#[derive(Clone)]
pub struct FlowableEngine {
    /// 比如 Flowable REST API 的 base_url
    pub base_url: String,
    http: reqwest::Client,
}

impl FlowableEngine {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            http: reqwest::Client::new(),
        }
    }

    // 将来可以在这里增加更多 helper 方法
}

#[async_trait]
impl WorkflowEngine for FlowableEngine {
    async fn deploy(&self, _def: &WorkflowDefinition) -> Result<DeployResult> {
        // TODO: 将 LogicFlow Graph 转 BPMN，POST 到 Flowable
        Err(anyhow!("FlowableEngine::deploy 未实现"))
    }

    async fn run_once(&self, _def: &WorkflowDefinition, _input: Value) -> Result<StartResult> {
        // TODO: 调用 Flowable 启动流程
        Err(anyhow!("FlowableEngine::run_once 未实现"))
    }
}

/// Zeebe 引擎预留壳，用于将来对接 Zeebe / Camunda 8
#[derive(Clone)]
pub struct ZeebeEngine {
    /// 比如 Zeebe Gateway 的地址
    pub gateway_addr: String,
    // 将来可以加 gRPC 客户端等
}

impl ZeebeEngine {
    pub fn new(gateway_addr: impl Into<String>) -> Self {
        Self {
            gateway_addr: gateway_addr.into(),
        }
    }
}

#[async_trait]
impl WorkflowEngine for ZeebeEngine {
    async fn deploy(&self, _def: &WorkflowDefinition) -> Result<DeployResult> {
        // TODO: 通过 Zeebe 部署流程定义
        Err(anyhow!("ZeebeEngine::deploy 未实现"))
    }

    async fn run_once(&self, _def: &WorkflowDefinition, _input: Value) -> Result<StartResult> {
        // TODO: 通过 Zeebe 创建流程实例
        Err(anyhow!("ZeebeEngine::run_once 未实现"))
    }
}
