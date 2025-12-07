// plugins/workflow-engine_system_plugin/src/lib.rs

use std::{
    ffi::CString,
    os::raw::c_char,
    sync::{Arc, Once},
    thread,
};

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use dotenvy::dotenv;
use plugin_api::{PluginApiInfo, PluginContext, PluginMeta};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tokio::runtime::Runtime;
use tracing::{error, info};
use std::net::SocketAddr;

const PLUGIN_NAME: &str = "workflow-engine";
const PLUGIN_VERSION: &str = "0.1.0";
const PLUGIN_KIND: &str = "workflow";
const API_PORT: u16 = 5601;
// 注意：这里仍然是 /workflow，host 会用它拼 base_url=http://127.0.0.1:5601/workflow
const API_PREFIX: &str = "/workflow";

fn cstr(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

// 只启动一个 HTTP server
static SERVER_ONCE: Once = Once::new();

// ====== Plugin ABI ======

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: cstr(PLUGIN_NAME),
        version: cstr(PLUGIN_VERSION),
        kind: cstr(PLUGIN_KIND),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_api_info() -> PluginApiInfo {
    PluginApiInfo {
        port: API_PORT,
        prefix: cstr(API_PREFIX),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(_ctx: *mut PluginContext) {
    // host 会周期性调用 run_with_ctx，这里用 Once 确保 HTTP server 只起一次
    SERVER_ONCE.call_once(|| {
        thread::spawn(|| {
            dotenv().ok();
            let rt = Runtime::new().expect("创建 tokio runtime 失败");
            rt.block_on(async {
                if let Err(e) = start_server().await {
                    eprintln!("[workflow-engine] server error: {e:?}");
                }
            });
        });
    });

    // 注意：这里不要阻塞，host 会周期性调用插件（目前我们也没在这里做额外逻辑）
}

// ====== 内部状态 & DB 结构 ======

#[derive(Clone)]
struct AppState {
    pool: Pool<Sqlite>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowDefinitionRow {
    id: i64,
    name: String,
    description: Option<String>,
    lf_json: serde_json::Value,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowInstanceRow {
    id: i64,
    workflow_id: i64,
    status: String,
    steps: serde_json::Value,
    started_at: DateTime<Utc>,
    finished_at: Option<DateTime<Utc>>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct SaveDefinitionRequest {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub lf_json: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct SaveDefinitionResponse {
    pub id: i64,
}

#[derive(Debug, Deserialize)]
struct RunRequest {
    pub workflow_id: i64,
}

#[derive(Debug, Serialize)]
struct RunResponse {
    pub instance_id: i64,
    pub status: String,
}

#[derive(Debug, Deserialize)]
struct AiGenerateRequest {
    pub prompt: String,
}

#[derive(Debug, Serialize)]
struct AiGenerateResponse {
    pub lf_json: serde_json::Value,
}

// ====== 启动 HTTP 服务 ======

async fn start_server() -> Result<(), sqlx::Error> {
    init_tracing();

    let db_url = std::env::var("MONITOR_AI_DB_URL")
        .unwrap_or_else(|_| "sqlite://database/monitor_ai.db".to_string());

    info!("[workflow-engine] 使用数据库: {db_url}");

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    init_schema(&pool).await?;

    let state = AppState { pool };

    // 这里的路由全部带上 `/workflow` 前缀，和 API_PREFIX 对齐
    let app = Router::new()
        // 健康检查，可以同时保留根和带前缀两个
        .route("/health", get(health))
        .route("/workflow/health", get(health))
        // 工作流定义相关接口：/workflow/definitions...
        .route(
            "/workflow/definitions",
            get(list_definitions).post(save_definition),
        )
        .route("/workflow/definitions/:id", get(get_definition))
        // 运行实例相关
        .route("/workflow/run", post(run_workflow))
        .route("/workflow/instances/:id", get(get_instance))
        // AI 生成 LogicFlow JSON
        .route("/workflow/ai-generate", post(ai_generate))
        .with_state(Arc::new(state));

    let addr: SocketAddr = format!("127.0.0.1:{API_PORT}")
        .parse()
        .expect("invalid API_PORT or bind address");
    info!("[workflow-engine] HTTP server 启动于 http://{addr}{API_PREFIX}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .map_err(|e| {
            error!("[workflow-engine] bind error: {e}");
            sqlx::Error::Protocol(e.to_string().into())
        })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| {
            error!("[workflow-engine] server error: {e}");
            sqlx::Error::Protocol(e.to_string().into())
        })
}

// ====== tracing ======

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());
    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .try_init();
}

// ====== DB schema ======

async fn init_schema(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    // 存储 LogicFlow JSON
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS workflow_definitions (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            name        TEXT NOT NULL,
            description TEXT,
            lf_json     TEXT NOT NULL,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL
        );
    "#,
    )
    .execute(pool)
    .await?;

    // 存执行实例 + 步骤状态
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS workflow_instances (
            id          INTEGER PRIMARY KEY AUTOINCREMENT,
            workflow_id INTEGER NOT NULL,
            status      TEXT NOT NULL,
            steps       TEXT NOT NULL,
            started_at  TEXT NOT NULL,
            finished_at TEXT,
            error       TEXT
        );
    "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

// ====== Handlers ======

async fn health() -> &'static str {
    "OK"
}

// GET /workflow/definitions
async fn list_definitions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WorkflowDefinitionRow>>, String> {
    let rows = sqlx::query_as::<_, (i64, String, Option<String>, String, String, String)>(
        r#"
        SELECT id, name, description, lf_json, created_at, updated_at
        FROM workflow_definitions
        ORDER BY id DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let defs = rows
        .into_iter()
        .filter_map(|(id, name, description, lf_json, created_at, updated_at)| {
            let lf_json_val: serde_json::Value = serde_json::from_str(&lf_json).ok()?;
            let created_at_dt = created_at.parse().ok()?;
            let updated_at_dt = updated_at.parse().ok()?;
            Some(WorkflowDefinitionRow {
                id,
                name,
                description,
                lf_json: lf_json_val,
                created_at: created_at_dt,
                updated_at: updated_at_dt,
            })
        })
        .collect();

    Ok(Json(defs))
}

// GET /workflow/definitions/:id
async fn get_definition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<WorkflowDefinitionRow>, String> {
    let row = sqlx::query_as::<_, (i64, String, Option<String>, String, String, String)>(
        r#"
        SELECT id, name, description, lf_json, created_at, updated_at
        FROM workflow_definitions
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let lf_json_val: serde_json::Value =
        serde_json::from_str(&row.3).map_err(|e| e.to_string())?;

    let created_at_dt: DateTime<Utc> = row
        .4
        .parse::<DateTime<Utc>>() // 显式告诉 parse 要转成 DateTime<Utc>
        .map_err(|e: chrono::ParseError| e.to_string())?;
    let updated_at_dt: DateTime<Utc> = row
        .5
        .parse::<DateTime<Utc>>()
        .map_err(|e| e.to_string())?;

    Ok(Json(WorkflowDefinitionRow {
        id: row.0,
        name: row.1,
        description: row.2,
        lf_json: lf_json_val,
        created_at: created_at_dt,
        updated_at: updated_at_dt,
    }))
}

// POST /workflow/definitions
async fn save_definition(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SaveDefinitionRequest>,
) -> Result<Json<SaveDefinitionResponse>, String> {
    let now = Utc::now().to_rfc3339();
    let lf_str =
        serde_json::to_string(&req.lf_json).map_err(|e| e.to_string())?;

    let id = if let Some(id) = req.id {
        sqlx::query(
            r#"
            UPDATE workflow_definitions
            SET name = ?, description = ?, lf_json = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(&lf_str)
        .bind(&now)
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;
        id
    } else {
        let result = sqlx::query(
            r#"
            INSERT INTO workflow_definitions (name, description, lf_json, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&req.name)
        .bind(&req.description)
        .bind(&lf_str)
        .bind(&now)
        .bind(&now)
        .execute(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

        result.last_insert_rowid()
    };

    Ok(Json(SaveDefinitionResponse { id }))
}

// POST /workflow/run
async fn run_workflow(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RunRequest>,
) -> Result<Json<RunResponse>, String> {
    // 1. 取出 workflow definition
    let row = sqlx::query_as::<_, (i64, String, Option<String>, String, String, String)>(
        r#"
        SELECT id, name, description, lf_json, created_at, updated_at
        FROM workflow_definitions
        WHERE id = ?
        "#,
    )
    .bind(req.workflow_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let lf_json_val: serde_json::Value =
        serde_json::from_str(&row.3).map_err(|e| e.to_string())?;

    // 2. 简单解析节点列表，模拟执行
    let mut steps_vec = Vec::<serde_json::Value>::new();
    let nodes = lf_json_val
        .get("nodes")
        .and_then(|n| n.as_array())
        .cloned()
        .unwrap_or_default();

    let started_at = Utc::now();
    let last_error: Option<String> = None;

    for node in nodes {
        let node_id = node.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let node_type = node.get("type").and_then(|v| v.as_str()).unwrap_or("");
        // 简单模拟：全部当成功
        let step_obj = serde_json::json!({
            "node_id": node_id,
            "node_type": node_type,
            "status": "success",
            "message": "executed"
        });
        steps_vec.push(step_obj);
    }

    let steps_json = serde_json::Value::Array(steps_vec);
    let steps_str =
        serde_json::to_string(&steps_json).map_err(|e| e.to_string())?;

    let status = if last_error.is_some() {
        "failed"
    } else {
        "success"
    };

    // 3. 写入 workflow_instances
    let started_at_str = started_at.to_rfc3339();
    let finished_at_str = Utc::now().to_rfc3339();

    let result = sqlx::query(
        r#"
        INSERT INTO workflow_instances
            (workflow_id, status, steps, started_at, finished_at, error)
        VALUES (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(req.workflow_id)
    .bind(status)
    .bind(&steps_str)
    .bind(&started_at_str)
    .bind(&finished_at_str)
    .bind(&last_error)
    .execute(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let instance_id = result.last_insert_rowid();

    Ok(Json(RunResponse {
        instance_id,
        status: status.to_string(),
    }))
}

// GET /workflow/instances/:id
async fn get_instance(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<WorkflowInstanceRow>, String> {
    let row = sqlx::query_as::<_, (i64, i64, String, String, String, Option<String>, Option<String>)>(
        r#"
        SELECT id, workflow_id, status, steps, started_at, finished_at, error
        FROM workflow_instances
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| e.to_string())?;

    let steps_val: serde_json::Value =
        serde_json::from_str(&row.3).map_err(|e| e.to_string())?;
    let started_at = row
        .4
        .parse::<DateTime<Utc>>()
        .map_err(|e| e.to_string())?;
    let finished_at: Option<DateTime<Utc>> = match row.5 {
        Some(s) => {
            let dt = s
                .parse::<DateTime<Utc>>() // 显式解析类型
                .map_err(|e| e.to_string())?;
            Some(dt)
        }
        None => None,
    };

    Ok(Json(WorkflowInstanceRow {
        id: row.0,
        workflow_id: row.1,
        status: row.2,
        steps: steps_val,
        started_at,
        finished_at,
        error: row.6,
    }))
}

// POST /workflow/ai-generate
async fn ai_generate(
    Json(req): Json<AiGenerateRequest>,
) -> Result<Json<AiGenerateResponse>, String> {
    // 先做一个非常简单的“规则引擎版”，以后再换成真 AI
    // 根据 prompt 拼一个 Start -> API -> End 三个节点的 LogicFlow JSON

    let title = if req.prompt.trim().is_empty() {
        "新工作流"
    } else {
        req.prompt.trim()
    };

    let lf_json = serde_json::json!({
        "nodes": [
            {
                "id": "start_1",
                "type": "start-node",
                "x": 80,
                "y": 150,
                "text": { "value": "开始" }
            },
            {
                "id": "api_1",
                "type": "api-node",
                "x": 260,
                "y": 150,
                "text": { "value": format!("API 调用: {}", title) },
                "properties": {
                    "method": "GET",
                    "url": "https://api.example.com/health",
                    "timeout_ms": 3000
                }
            },
            {
                "id": "end_1",
                "type": "end-node",
                "x": 440,
                "y": 150,
                "text": { "value": "结束" }
            }
        ],
        "edges": [
            {
                "id": "e1",
                "type": "polyline",
                "sourceNodeId": "start_1",
                "targetNodeId": "api_1"
            },
            {
                "id": "e2",
                "type": "polyline",
                "sourceNodeId": "api_1",
                "targetNodeId": "end_1"
            }
        ]
    });

    Ok(Json(AiGenerateResponse { lf_json }))
}
