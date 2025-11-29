use axum::{
    Router,
    routing::{post, get},
    Json,
    extract::Path,
};
use serde_json::{json, Value};
use chrono::Utc;
use uuid::Uuid;

use super::template::{render_template};
use super::risk_guard::check_risk;
use super::channel::send_by_best_channel;
use crate::template::render_template;
use crate::risk_guard::check_risk;
use crate::channel::send_by_best_channel;
use crate::db::{get_db, NotificationHistory};

pub async fn start_server(port: u16) {
    let app = Router::new()
        .route("/send", post(api_send))
        .route("/stats", get(api_stats))
        .route("/template_render_preview", post(api_template_preview));

    let addr = format!("127.0.0.1:{port}").parse().unwrap();
    println!("[notification-center] HTTP listening at http://{addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(serde::Deserialize)]
struct SendReq {
    user_id: String,
    scene: String,
    vars: serde_json::Value,
}

async fn api_send(Json(req): Json<SendReq>) -> Json<Value> {
    if let Err(reason) = check_risk(&req.user_id, &req.scene) {
        return Json(json!({ "error": "blocked", "reason": reason }));
    }

    let content = render_template(&req.scene, &req.vars);

    let channel = send_by_best_channel(&req.user_id, &req.scene, &content).await;

    let msg_id = format!("ntf_{}", uuid::Uuid::new_v4());

    Json(json!({
        "msg_id": msg_id,
        "status": "queued",
        "channel": channel,
    }))
}

async fn api_stats() -> Json<Value> {
    Json(json!({
        "total_sent": 123,
        "success_rate": 0.98,
        "top_channels": ["sms","email"]
    }))
}

#[derive(serde::Deserialize)]
struct PreviewReq {
    scene: String,
    vars: serde_json::Value,
}

async fn api_template_preview(Json(req): Json<PreviewReq>) -> Json<Value> {
    let content = render_template(&req.scene, &req.vars);
    Json(json!({ "preview": content }))
}

pub async fn start_server(port: u16) {
    let app = Router::new()
        .route("/send", post(api_send))
        .route("/message/:msg_id", get(api_get_message))
        .route("/templates", get(api_list_templates))
        .route("/template_render_preview", post(api_template_preview))
        .route("/stats", get(api_stats));

    let addr = format!("127.0.0.1:{port}").parse().unwrap();
    println!("[notification-center] HTTP listening at http://{addr}");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(serde::Deserialize)]
struct SendReq {
    user_id: String,
    scene: String,
    #[serde(default)]
    channel_hint: Option<String>,
    vars: serde_json::Value,
}

async fn api_send(Json(req): Json<SendReq>) -> Json<Value> {
    let trace_id = Uuid::new_v4().to_string();
    let msg_id = format!("ntf_{}", Uuid::new_v4());

    if let Err(reason) = check_risk(&req.user_id, &req.scene) {
        return Json(json!({
            "msg_id": msg_id,
            "trace_id": trace_id,
            "status": "blocked",
            "reason": reason
        }));
    }

    let content = render_template(&req.scene, &req.vars);
    let channel = send_by_best_channel(&req.user_id, &req.scene, &content).await;

    let db = get_db();
    let now = Utc::now();
    let history = NotificationHistory {
        msg_id: msg_id.clone(),
        user_id: req.user_id.clone(),
        scene: req.scene.clone(),
        channel: channel.clone(),
        content: content.clone(),
        status: "queued".into(),
        trace_id: trace_id.clone(),
        error: None,
        created_at: now,
        sent_at: Some(now),
        delivered_at: None,
    };

    if let Err(e) = db.insert_history(&history).await {
        eprintln!("[notification-center] insert_history error: {e}");
    }

    Json(json!({
        "msg_id": msg_id,
        "trace_id": trace_id,
        "status": "queued",
        "channel": channel,
    }))
}

async fn api_get_message(Path(msg_id): Path<String>) -> Json<Value> {
    let db = get_db();
    match db.get_history(&msg_id).await {
        Ok(Some(h)) => Json(json!({
            "msg_id": h.msg_id,
            "user_id": h.user_id,
            "scene": h.scene,
            "channel": h.channel,
            "content": h.content,
            "status": h.status,
            "trace_id": h.trace_id,
            "error": h.error,
            "created_at": h.created_at,
            "sent_at": h.sent_at,
            "delivered_at": h.delivered_at,
        })),
        Ok(None) => Json(json!({ "error": "not_found" })),
        Err(e) => {
            eprintln!("[notification-center] get_history error: {e}");
            Json(json!({ "error": "db_error" }))
        }
    }
}

async fn api_list_templates() -> Json<Value> {
    let db = get_db();
    match db.list_templates().await {
        Ok(list) => {
            let items: Vec<Value> = list
                .into_iter()
                .map(|t| {
                    json!({
                        "id": t.id,
                        "scene": t.scene,
                        "channel": t.channel,
                        "lang": t.lang,
                        "version": t.version,
                        "content": t.content,
                        "is_active": t.is_active,
                        "created_at": t.created_at,
                        "updated_at": t.updated_at,
                    })
                })
                .collect();
            Json(json!({ "items": items }))
        }
        Err(e) => {
            eprintln!("[notification-center] list_templates error: {e}");
            Json(json!({ "error": "db_error" }))
        }
    }
}
