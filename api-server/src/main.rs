use std::net::SocketAddr;
use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, any},
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
use http::{Method, header, Request};
use tower_http::cors::{CorsLayer, Any};


#[derive(Clone)]
struct AppState {
    db: Arc<Db>,
    plugin_apis: Arc<std::sync::RwLock<HashMap<String, String>>>,
    http_client: reqwest::Client,
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

        // 读取插件 API 基础映射
    let apis = db
        .get_all_plugin_apis()
        .await
        .expect("加载 plugin_apis 失败");
    let mut map = HashMap::new();
    for (plugin, base_url) in apis {
        info!("插件 API: plugin={} => {}", plugin, base_url);
        map.insert(plugin, base_url);
    }

    let state = AppState {
        db: Arc::new(db),
        plugin_apis: Arc::new(std::sync::RwLock::new(map)),
        http_client: reqwest::Client::new(),
    };

    // 开发环境 CORS（允许前端和常用方法）
    let cors = CorsLayer::new()
        // 开发图省事就允许所有域名；如果你想严格一点见下面注释
        .allow_origin(Any)
        // 或者更严格一点：
        // .allow_origin("http://127.0.0.1:5173".parse::<http::HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE]);


    let app = Router::new()
        .route("/logs", get(get_logs))
        .route("/metrics", get(get_metrics))
        .route("/alerts", get(get_alerts).post(create_alert))
        .route(
            "/plugin-api/:plugin/*rest",
            any(proxy_plugin_api),
        )
        .with_state(state)
        .layer(cors);  // 挂上 CORS 层;

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

async fn proxy_plugin_api(
    State(state): State<AppState>,
    Path((plugin, rest)): Path<(String, String)>,
    req: Request<Body>,  // 不用 mut 了
) -> impl IntoResponse {
    // 查找 base_url
    let base_url_opt = {
        let guard = state.plugin_apis.read().unwrap();
        guard.get(&plugin).cloned()
    };

    let base_url = match base_url_opt {
        Some(u) => u,
        None => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            let body = Bytes::from(
                format!("未知插件或未注册 API: {}", plugin).into_bytes()
            );
            return (StatusCode::NOT_FOUND, headers, body);
        }
    };

    // 拼接目标 URL
    let path = if rest.is_empty() {
        "".to_string()
    } else {
        format!("/{}", rest.trim_start_matches('/'))
    };
    let target = format!("{}{}", base_url.trim_end_matches('/'), path);

    // 先取出 method / headers
    let method = req.method().clone();
    let headers = req.headers().clone();

    // 拆 request，拿到底层 body
    let (_parts, body) = req.into_parts();

    // 读取 body，限制 1MB（可以按需调大/调小）
    let body_bytes = axum::body::to_bytes(body, 1024 * 1024)
        .await
        .unwrap_or_default();

    let mut builder = state.http_client.request(method, &target);

    // 转发部分头（可根据需要筛选）
    for (k, v) in headers.iter() {
        if k.as_str().eq_ignore_ascii_case("host") {
            continue;
        }
        builder = builder.header(k, v);
    }

    let resp = match builder.body(body_bytes.clone()).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("转发到插件 API 失败: {e}");

            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                header::HeaderValue::from_static("text/plain; charset=utf-8"),
            );
            let body = Bytes::from("调用插件 API 失败".as_bytes().to_vec());
            return (StatusCode::BAD_GATEWAY, headers, body);
        }
    };

    let status = StatusCode::from_u16(resp.status().as_u16())
        .unwrap_or(StatusCode::BAD_GATEWAY);

    let mut out_headers = HeaderMap::new();
    for (name, value) in resp.headers().iter() {
        out_headers.insert(name.clone(), value.clone());
    }

    let bytes = resp.bytes().await.unwrap_or_default();

    // 三元组 (StatusCode, HeaderMap, Bytes) -> impl IntoResponse
    (status, out_headers, Bytes::from(bytes))
}
