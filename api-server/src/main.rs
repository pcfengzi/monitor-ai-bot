use std::net::SocketAddr;
use std::collections::HashMap;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use chrono::Utc;
use core_types::{AlertEvent, AlertSeverity, LogEvent, Metric};
use dotenv::dotenv;
use serde::Deserialize;
use storage::Db;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;
use sqlx::sqlite::Sqlite;
use sqlx::migrate::MigrateDatabase;

#[derive(Clone)]
struct AppState {
    db: Db,
}

#[derive(Deserialize)]
struct CreateAlertReq {
    plugin: String,
    metric_name: String,
    severity: String, // "Info" | "Warning" | "Critical"
    title: String,
    message: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    init_tracing();

    // let db_url = "sqlite:monitor_ai.db";
    let db_url = std::env::var("MONITOR_AI_DB_URL")
        .unwrap_or_else(|_| "sqlite://database/monitor_ai.db".to_string());

    // 不存在就创建
    if !Sqlite::database_exists(&db_url).await.unwrap_or(false) {
        Sqlite::create_database(&db_url)
            .await
            .expect("创建 SQLite 数据库失败");
    }

    info!("准备连接数据库: {db_url}");
    let db = Db::connect(&db_url)
        .await
        .expect("连接数据库失败");

    info!("api-server 已连接数据库: {db_url}");

    let state = AppState { db };

    let app = Router::new()
        .route("/logs", get(get_logs))
        .route("/metrics", get(get_metrics))
        .route("/alerts", get(get_alerts).post(create_alert))
        .with_state(state);

    let addr: SocketAddr = "127.0.0.1:3001".parse().unwrap();
    info!("api-server 启动：http://{addr}/logs /metrics /alerts");

    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();
}

async fn get_logs(State(state): State<AppState>) -> Json<Vec<LogEvent>> {
    let list = state
        .db
        .latest_logs(100)
        .await
        .unwrap_or_else(|_| vec![]);
    Json(list)
}

async fn get_metrics(State(state): State<AppState>) -> Json<Vec<Metric>> {
    let list = state
        .db
        .latest_metrics(200)
        .await
        .unwrap_or_else(|_| vec![]);
    Json(list)
}

async fn get_alerts(State(state): State<AppState>) -> Json<Vec<AlertEvent>> {
    let list = state
        .db
        .latest_alerts(200)
        .await
        .unwrap_or_else(|_| vec![]);
    Json(list)
}

async fn create_alert(
    State(state): State<AppState>,
    Json(req): Json<CreateAlertReq>,
) -> Result<Json<AlertEvent>, (StatusCode, String)> {
    let severity = match req.severity.as_str() {
        "Info" => AlertSeverity::Info,
        "Warning" => AlertSeverity::Warning,
        "Critical" => AlertSeverity::Critical,
        _ => AlertSeverity::Info,
    };

    let alert = AlertEvent {
        time: Utc::now(),
        plugin: req.plugin,
        metric_name: req.metric_name,
        severity,
        title: req.title,
        message: req.message,
        tags: HashMap::new(), 
    };

    if let Err(e) = state.db.insert_alert(&alert).await {
        tracing::error!("插入告警失败: {e}");
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "insert alert failed".into()));
    }

    Ok(Json(alert))
}
