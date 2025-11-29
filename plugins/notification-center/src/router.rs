// plugins/notification-center/src/router.rs
use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    extract::Path,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use crate::channel;
use crate::db;
use crate::metrics_bridge::HostBridge;
use crate::risk_guard;
use crate::types::{InternalMessage, MessageStatus, SendRequest};

type Tx = mpsc::Sender<InternalMessage>;
type Rx = mpsc::Receiver<InternalMessage>;

pub async fn start_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 创建队列 & worker
    let (tx, rx) = mpsc::channel::<InternalMessage>(1000);
    tokio::spawn(worker(rx));

    let tx_filter = Arc::new(tx);

    let app = Router::new()
        .route("/send", post({
            let tx = tx_filter.clone();
            move |payload| api_send(tx.clone(), payload)
        }))
        .route("/message/:msg_id", get(api_get_message));

    let port: u16 = std::env::var("NC_PLUGIN_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5601);

    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    println!("[notification-center] listen at http://{addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

// ============ HTTP handler ============

async fn api_send(
    tx: Arc<Tx>,
    Json(req): Json<SendRequest>,
) -> Json<Value> {
    let trace_id = uuid::Uuid::new_v4().to_string();
    let msg_id = format!("ntf_{}", uuid::Uuid::new_v4());
    let created_at = Utc::now();

    // 先写入 waiting 状态
    if let Err(e) = db::db()
        .insert_waiting(&msg_id, &req.user_id, &req.scene, &trace_id, created_at)
        .await
    {
        HostBridge::log_static(
            plugin_api::LogLevel::Error,
            &format!("[notification-center] insert_waiting error: {e}"),
        );
        return Json(json!({
            "msg_id": msg_id,
            "trace_id": trace_id,
            "status": "db_error"
        }));
    }

    // 入队（异步 worker 去做后续步骤）
    let msg = InternalMessage {
        msg_id: msg_id.clone(),
        trace_id: trace_id.clone(),
        req,
        created_at,
        delay_secs: 0,
        retries: 0,
    };

    if let Err(e) = tx.send(msg).await {
        HostBridge::log_static(
            plugin_api::LogLevel::Error,
            &format!("[notification-center] enqueue error: {e}"),
        );
        return Json(json!({
            "msg_id": msg_id,
            "trace_id": trace_id,
            "status": "queue_error"
        }));
    }

    HostBridge::metric_static("notification_enqueue", 1.0);

    Json(json!({
        "msg_id": msg_id,
        "trace_id": trace_id,
        "status": "queued"
    }))
}

async fn api_get_message(Path(msg_id): Path<String>) -> Json<Value> {
    match db::db().get_history(&msg_id).await {
        Ok(Some(h)) => Json(json!({
            "msg_id": h.msg_id,
            "user_id": h.user_id,
            "scene": h.scene,
            "channel": h.channel,
            "content": h.content,
            "status": h.status.as_str(),
            "trace_id": h.trace_id,
            "error": h.error,
            "retries": h.retries,
            "created_at": h.created_at,
        })),
        Ok(None) => Json(json!({ "error": "not_found" })),
        Err(e) => {
            HostBridge::log_static(
                plugin_api::LogLevel::Error,
                &format!("[notification-center] get_history error: {e}"),
            );
            Json(json!({ "error": "db_error" }))
        }
    }
}

// ============ worker：处理队列里的消息 ============

async fn worker(mut rx: Rx) {
    while let Some(mut msg) = rx.recv().await {
        if msg.delay_secs > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(
                msg.delay_secs as u64,
            ))
            .await;
        }

        let start = std::time::Instant::now();
        process_message(&mut msg).await;
        let elapsed = start.elapsed().as_millis() as f64;

        HostBridge::metric_static("notification_process_duration_ms", elapsed);
    }
}

async fn process_message(msg: &mut InternalMessage) {
    let db = db::db();

    if let Err(e) = db.update_status_processing(&msg.msg_id).await {
        HostBridge::log_static(
            plugin_api::LogLevel::Error,
            &format!("[notification-center] update_status_processing error: {e}"),
        );
        return;
    }

    // 1. 风控
    if let Err(e) = risk_guard::check_risk(&msg.req.user_id, &msg.req.scene) {
        let reason = e.to_string();
        let _ = db
            .update_status_final(
                &msg.msg_id,
                MessageStatus::Blocked,
                &channel::Channel::Inbox,
                "",
                Some(reason.clone()),
                msg.retries,
            )
            .await;
        HostBridge::metric_static("notification_blocked", 1.0);
        HostBridge::log_static(
            plugin_api::LogLevel::Warn,
            &format!(
                "[notification-center] msg {} blocked by risk: {reason}",
                msg.msg_id
            ),
        );
        return;
    }

    // 2. 模板获取 + 渲染
    let primary_channel = channel::priority_channels(
        &msg.req.scene,
        msg.req.channel_hint.as_deref(),
    )
    .first()
    .cloned()
    .unwrap_or(channel::Channel::Inbox);

    let tpl = db
        .get_template(&msg.req.scene, &primary_channel)
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "【默认模板】{{scene}} 通知".to_string());

    let mut vars = msg.req.vars.clone();
    if vars.is_null() {
        vars = serde_json::json!({});
    }
    // 注入一些基础变量
    if let Some(map) = vars.as_object_mut() {
        map.insert("scene".into(), serde_json::Value::String(msg.req.scene.clone()));
        map.insert(
            "user_id".into(),
            serde_json::Value::String(msg.req.user_id.clone()),
        );
    }

    let content = crate::template::render(&tpl, &vars);

    // 3. 调用渠道发送（多渠道 + 降级）
    let (channel_used, status, err, _) = channel::send_with_fallback(
        &msg.req.user_id,
        &msg.req.scene,
        &content,
        msg.req.channel_hint.as_deref(),
    )
    .await;

    // 4. 更新最终状态
    let _ = db
        .update_status_final(
            &msg.msg_id,
            status.clone(),
            &channel_used,
            &content,
            err.clone(),
            msg.retries,
        )
        .await;

    // 5. 上报 metric
    let metric_name = match status {
        MessageStatus::Delivered => "notification_success",
        MessageStatus::Failed => "notification_failed",
        MessageStatus::Blocked => "notification_blocked",
        _ => "notification_other",
    };
    HostBridge::metric_static(metric_name, 1.0);

    if let Some(e) = err {
        HostBridge::log_static(
            plugin_api::LogLevel::Warn,
            &format!(
                "[notification-center] msg {} final status {:?}, error: {}",
                msg.msg_id, status, e
            ),
        );
    }
}
