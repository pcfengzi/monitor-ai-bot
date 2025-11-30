好，这个“系统级定时器插件”我们可以做成一个 **timer-scheduler 插件**：

* 自己起一个后台调度循环
* 自己连 DB，维护 `timer_jobs` / `timer_runs` 两张表
* 暴露 HTTP API：`/plugin-api/timer-scheduler/...`
* 其它插件 & 客户端只要按约定调用它的 API，就能注册 / 管理定时任务

下面我直接给你**完整可落地的实现骨架**（一个 crate），你把它放进 `plugins/timer-scheduler/` 里即可。

---

## 1. `plugins/timer-scheduler/Cargo.toml`

```toml
[package]
name = "timer-scheduler"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
plugin-api = { path = "../../plugin-api" }

tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
axum = "0.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sqlx = { version = "0.7", default-features = false, features = ["runtime-tokio-rustls", "sqlite", "chrono"] }
chrono = { version = "0.4", features = ["serde"] }
once_cell = "1.19"
tracing = "0.1"
tracing-subscriber = "0.3"
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
dotenvy = "0.15"
thiserror = "1.0"
```

> 注意：这里只有 `sqlx::query` / `query_as`，**不使用 `query!` 宏**，所以不会再遇到 “set DATABASE_URL ...” 那个错误。

---

## 2. `plugins/timer-scheduler/src/lib.rs`

这是一个单文件版，内部用小模块拆干净了，方便你后续再拆成多个文件。

```rust
use std::{ffi::CString, os::raw::c_char, time::Duration};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use plugin_api::{LogLevel, MetricSample, PluginApiInfo, PluginContext, PluginMeta};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tokio::time::sleep;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

// ==== 全局：只启动一次 ====

static STARTED: OnceCell<()> = OnceCell::new();

// ====== 主导出：meta / plugin_api_info / run_with_ctx ======

const NAME: &str = "timer-scheduler";
const VERSION: &str = "0.1.0";
const KIND: &str = "timer";

// 定时器插件内部 HTTP 服务端口（通过 plugin_api_info 告诉 api-server）
const API_PORT: u16 = 5601;
const API_PREFIX: &str = "/";

fn c_string(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: c_string(NAME),
        version: c_string(VERSION),
        kind: c_string(KIND),
    }
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
    if ctx.is_null() {
        return;
    }

    // 确保只启动一次（bot-host 每轮都会调用 run_with_ctx）
    if STARTED.set(()).is_err() {
        // 已经启动过了，略过
        return;
    }

    // 拷贝出需要的 host 回调，不能持有 ctx 指针本身
    let ctx_ref = unsafe { &*ctx };
    let bridge = HostBridge {
        log_fn: ctx_ref.log_fn,
        emit_metric_fn: ctx_ref.emit_metric_fn,
    };

    // 初始化插件内部 tracing（方便在 console 看日志）
    init_tracing();

    tokio::spawn(async move {
        if let Err(e) = run_main(bridge).await {
            error!("[timer-scheduler] 后台任务失败: {e}");
        }
    });
}

// ====== HostBridge：把 host 提供的 log/metric 回调包装起来 ======

#[derive(Clone)]
struct HostBridge {
    log_fn: extern "C" fn(LogLevel, *const c_char),
    emit_metric_fn: extern "C" fn(MetricSample),
}

impl HostBridge {
    fn log(&self, level: LogLevel, msg: &str) {
        let c = match CString::new(msg) {
            Ok(c) => c,
            Err(_) => return,
        };
        (self.log_fn)(level, c.as_ptr());
    }

    fn metric(&self, name: &str, value: f64) {
        let c = match CString::new(name) {
            Ok(c) => c,
            Err(_) => return,
        };
        let ts_ms = chrono::Utc::now()
            .timestamp_millis();

        let sample = MetricSample {
            name: c.as_ptr(),
            value,
            timestamp_ms: ts_ms,
        };
        (self.emit_metric_fn)(sample);
    }
}

// ====== 插件主逻辑入口 ======

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
    host: HostBridge,
    http: reqwest::Client,
}

async fn run_main(host: HostBridge) -> anyhow::Result<()> {
    host.log(LogLevel::Info, "[timer-scheduler] 启动中...");

    let db_url = std::env::var("MONITOR_AI_DB_URL")
        .unwrap_or_else(|_| "sqlite://database/monitor_ai.db".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    init_db(&pool).await?;

    let state = AppState {
        pool,
        host: host.clone(),
        http: reqwest::Client::new(),
    };

    // 启动 HTTP API server
    let app = build_router(state.clone());
    let addr = format!("127.0.0.1:{API_PORT}");
    host.log(
        LogLevel::Info,
        &format!("[timer-scheduler] HTTP API 监听在 http://{}", addr),
    );

    // 调度循环（后台）
    tokio::spawn(run_scheduler_loop(state.clone()));

    // 挂 HTTP 服务（阻塞当前任务）
    axum::Server::bind(&addr.parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .try_init();
}

// ====== DB Schema & 访问逻辑 ======

#[derive(Debug, thiserror::Error)]
enum DbError {
    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerJob {
    pub id: i64,
    pub name: String,
    pub target_url: String,
    pub method: String,
    pub interval_secs: Option<i64>,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// HTTP 创建/更新时用的 DTO
#[derive(Debug, Serialize, Deserialize)]
pub struct TimerJobInput {
    pub name: String,
    pub target_url: String,
    #[serde(default = "default_method")]
    pub method: String,
    pub interval_secs: Option<i64>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_method() -> String {
    "POST".to_string()
}

fn default_enabled() -> bool {
    true
}

async fn init_db(pool: &SqlitePool) -> Result<(), DbError> {
    // 只建自己的表，不碰系统 storage
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS timer_jobs (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            name          TEXT NOT NULL,
            target_url    TEXT NOT NULL,
            method        TEXT NOT NULL,
            interval_secs INTEGER,
            enabled       INTEGER NOT NULL,
            last_run_at   TEXT,
            next_run_at   TEXT,
            created_at    TEXT NOT NULL,
            updated_at    TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS timer_runs (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            job_id       INTEGER NOT NULL,
            run_at       TEXT NOT NULL,
            success      INTEGER NOT NULL,
            status_code  INTEGER,
            error        TEXT,
            created_at   TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_job(pool: &SqlitePool, input: TimerJobInput) -> Result<i64, DbError> {
    let now = Utc::now();
    let next = input
        .interval_secs
        .map(|_| now);

    let enabled = if input.enabled { 1 } else { 0 };

    let res = sqlx::query(
        r#"
        INSERT INTO timer_jobs
            (name, target_url, method, interval_secs, enabled,
             last_run_at, next_run_at, created_at, updated_at)
        VALUES
            (?, ?, ?, ?, ?, NULL, ?, ?, ?)
        "#,
    )
    .bind(&input.name)
    .bind(&input.target_url)
    .bind(&input.method)
    .bind(input.interval_secs)
    .bind(enabled)
    .bind(next.map(|dt| dt.to_rfc3339()))
    .bind(now.to_rfc3339())
    .bind(now.to_rfc3339())
    .execute(pool)
    .await?;

    Ok(res.last_insert_rowid())
}

async fn list_jobs(pool: &SqlitePool) -> Result<Vec<TimerJob>, DbError> {
    let rows = sqlx::query_as::<_, (i64, String, String, String, Option<i64>, i64, Option<String>, Option<String>, String, String)>(
        r#"
        SELECT id, name, target_url, method, interval_secs, enabled,
               last_run_at, next_run_at, created_at, updated_at
        FROM timer_jobs
        ORDER BY id DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let jobs = rows
        .into_iter()
        .map(|(id, name, target_url, method, interval_secs, enabled, last_run_at, next_run_at, created_at, updated_at)| {
            TimerJob {
                id,
                name,
                target_url,
                method,
                interval_secs,
                enabled: enabled != 0,
                last_run_at: last_run_at.and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                next_run_at: next_run_at.and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
            }
        })
        .collect();

    Ok(jobs)
}

async fn get_due_jobs(pool: &SqlitePool, now: DateTime<Utc>) -> Result<Vec<TimerJob>, DbError> {
    let now_str = now.to_rfc3339();

    let rows = sqlx::query_as::<_, (i64, String, String, String, Option<i64>, i64, Option<String>, Option<String>, String, String)>(
        r#"
        SELECT id, name, target_url, method, interval_secs, enabled,
               last_run_at, next_run_at, created_at, updated_at
        FROM timer_jobs
        WHERE enabled = 1
          AND (next_run_at IS NULL OR next_run_at <= ?)
        "#,
    )
    .bind(&now_str)
    .fetch_all(pool)
    .await?;

    let jobs = rows
        .into_iter()
        .map(|(id, name, target_url, method, interval_secs, enabled, last_run_at, next_run_at, created_at, updated_at)| {
            TimerJob {
                id,
                name,
                target_url,
                method,
                interval_secs,
                enabled: enabled != 0,
                last_run_at: last_run_at.and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                next_run_at: next_run_at.and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
                updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
            }
        })
        .collect();

    Ok(jobs)
}

async fn update_job_after_run(
    pool: &SqlitePool,
    job_id: i64,
    now: DateTime<Utc>,
    interval_secs: Option<i64>,
) -> Result<(), DbError> {
    let next = interval_secs.map(|sec| now + chrono::Duration::seconds(sec));
    let next_str = next.map(|dt| dt.to_rfc3339());

    sqlx::query(
        r#"
        UPDATE timer_jobs
        SET last_run_at = ?, next_run_at = ?, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(now.to_rfc3339())
    .bind(next_str)
    .bind(now.to_rfc3339())
    .bind(job_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn insert_run_history(
    pool: &SqlitePool,
    job_id: i64,
    now: DateTime<Utc>,
    success: bool,
    status_code: Option<u16>,
    error_msg: Option<String>,
) -> Result<(), DbError> {
    sqlx::query(
        r#"
        INSERT INTO timer_runs
            (job_id, run_at, success, status_code, error, created_at)
        VALUES
            (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(job_id)
    .bind(now.to_rfc3339())
    .bind(if success { 1 } else { 0 })
    .bind(status_code.map(|s| s as i64))
    .bind(error_msg)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}

// ====== 调度循环 ======

async fn run_scheduler_loop(state: AppState) {
    state
        .host
        .log(LogLevel::Info, "[timer-scheduler] 调度循环启动");

    loop {
        let now = Utc::now();

        match get_due_jobs(&state.pool, now).await {
            Ok(jobs) => {
                for job in jobs {
                    let state_clone = state.clone();
                    tokio::spawn(async move {
                        run_one_job(state_clone, job).await;
                    });
                }
            }
            Err(e) => {
                state
                    .host
                    .log(LogLevel::Error, &format!("[timer-scheduler] 查询待执行任务失败: {e}"));
            }
        }

        sleep(Duration::from_secs(1)).await;
    }
}

async fn run_one_job(state: AppState, job: TimerJob) {
    let now = Utc::now();
    let name = job.name.clone();

    state.host.log(
        LogLevel::Info,
        &format!("[timer-scheduler] 执行定时任务: id={}, name={}", job.id, job.name),
    );

    let result = do_call_target(&state, &job).await;

    match result {
        Ok(status_code) => {
            state.host.metric("timer_job_success", 1.0);
            let _ = insert_run_history(
                &state.pool,
                job.id,
                now,
                true,
                Some(status_code.as_u16()),
                None,
            )
            .await;
        }
        Err(err_msg) => {
            state.host.metric("timer_job_failed", 1.0);
            state.host.log(
                LogLevel::Error,
                &format!(
                    "[timer-scheduler] 任务执行失败: id={}, name={}, err={}",
                    job.id, name, err_msg
                ),
            );
            let _ = insert_run_history(&state.pool, job.id, now, false, None, Some(err_msg)).await;
        }
    }

    let _ = update_job_after_run(&state.pool, job.id, now, job.interval_secs).await;
}

async fn do_call_target(state: &AppState, job: &TimerJob) -> Result<reqwest::StatusCode, String> {
    let method = job.method.to_uppercase();
    let url = &job.target_url;

    let resp = match method.as_str() {
        "GET" => state
            .http
            .get(url)
            .send()
            .await
            .map_err(|e| e.to_string())?,
        "POST" => state
            .http
            .post(url)
            .send()
            .await
            .map_err(|e| e.to_string())?,
        _ => {
            return Err(format!("不支持的 HTTP 方法: {}", method));
        }
    };

    Ok(resp.status())
}

// ====== HTTP API（通过 api-server 的 /plugin-api/timer-scheduler/... 暴露） ======

fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/jobs", get(list_jobs_handler).post(create_job_handler))
        .route("/jobs/:id/trigger", post(trigger_job_once_handler))
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

/// GET /plugin-api/timer-scheduler/jobs
async fn list_jobs_handler(
    State(state): State<AppState>,
) -> Result<Json<Vec<TimerJob>>, StatusCode> {
    list_jobs(&state.pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// POST /plugin-api/timer-scheduler/jobs
/// body: { "name": "...", "target_url": "http://...", "method": "POST", "interval_secs": 60 }
async fn create_job_handler(
    State(state): State<AppState>,
    Json(input): Json<TimerJobInput>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if input.interval_secs.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let id = insert_job(&state.pool, input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({
        "id": id
    })))
}

/// POST /plugin-api/timer-scheduler/jobs/{id}/trigger
/// 手动执行一次
async fn trigger_job_once_handler(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    // 简单实现：读一次 job，立即执行
    let rows = sqlx::query_as::<_, (i64, String, String, String, Option<i64>, i64, Option<String>, Option<String>, String, String)>(
        r#"
        SELECT id, name, target_url, method, interval_secs, enabled,
               last_run_at, next_run_at, created_at, updated_at
        FROM timer_jobs
        WHERE id = ?
        "#,
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if rows.is_empty() {
        return Err(StatusCode::NOT_FOUND);
    }

    let (id, name, target_url, method, interval_secs, enabled, last_run_at, next_run_at, created_at, updated_at) = &rows[0];

    let job = TimerJob {
        id: *id,
        name: name.clone(),
        target_url: target_url.clone(),
        method: method.clone(),
        interval_secs: *interval_secs,
        enabled: *enabled != 0,
        last_run_at: last_run_at
            .as_ref()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok()),
        next_run_at: next_run_at
            .as_ref()
            .and_then(|s| s.parse::<DateTime<Utc>>().ok()),
        created_at: created_at.parse().unwrap_or_else(|_| Utc::now()),
        updated_at: updated_at.parse().unwrap_or_else(|_| Utc::now()),
    };

    let state_clone = state.clone();
    tokio::spawn(async move {
        run_one_job(state_clone, job).await;
    });

    Ok(StatusCode::OK)
}
```

---

## 3. “系统约定”：其他插件/客户端如何用这个定时器？

因为你现在已经有了 **插件 API 网关**，所以“约定”其实非常简单：

* **统一入口：**

  ```text
  POST http://API_SERVER/plugin-api/timer-scheduler/jobs
  ```

* 任何插件 / 客户端 / Agent 想注册一个周期任务，只要发这样的请求：

  ```json
  POST /plugin-api/timer-scheduler/jobs
  Content-Type: application/json

  {
    "name": "api-monitor-login-flow",
    "target_url": "http://127.0.0.1:5501/run_workflow/login_and_get_profile",
    "method": "POST",
    "interval_secs": 60,
    "enabled": true
  }
  ```

  > 这里的 `target_url` 就是指向“另一个插件”的内部 HTTP API，
  > 例如：`api-monitor` 插件监听 5501 端口的 `/run_workflow/...`。

* **手动触发一次：**

  ```http
  POST /plugin-api/timer-scheduler/jobs/{id}/trigger
  ```

* **查询已有任务：**

  ```http
  GET /plugin-api/timer-scheduler/jobs
  ```

这样，这个 `timer-scheduler` 插件就成了整个系统的 **统一定时调度中心**，而且：

* 不需要改 `bot-host` / `api-server` / `storage` 任何代码
* 任何插件只要 **能发 HTTP 请求**，就能注册/管理自己的定时任务
* Web / 桌面 / 手机端也可以直接调用这个插件 API 做管理界面

---

如果你愿意，下一步我可以：

* 帮你加一段 `clients/web-client` 或 `dashboard-frontend` 的页面：
  **“定时任务管理”**（列表 + 创建表单），
  对接上面这几个 API，做到真正点一点就能新增/暂停定时调度。
