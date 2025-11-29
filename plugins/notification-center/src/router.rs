use axum::{
    Router,
    routing::{post, get},
    Json,
};
use serde_json::{json, Value};

use super::template::{render_template};
use super::risk_guard::check_risk;
use super::channel::send_by_best_channel;

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
