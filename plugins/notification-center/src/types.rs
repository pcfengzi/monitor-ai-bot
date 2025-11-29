// plugins/notification-center/src/types.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Channel {
    Sms,
    Email,
    Push,
    Inbox,
}

impl Channel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Channel::Sms => "sms",
            Channel::Email => "email",
            Channel::Push => "push",
            Channel::Inbox => "inbox",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    Waiting,
    Processing,
    Sent,
    Delivered,
    Failed,
    Blocked,
}

impl MessageStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageStatus::Waiting => "waiting",
            MessageStatus::Processing => "processing",
            MessageStatus::Sent => "sent",
            MessageStatus::Delivered => "delivered",
            MessageStatus::Failed => "failed",
            MessageStatus::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendRequest {
    pub user_id: String,
    pub scene: String,
    pub channel_hint: Option<String>,
    #[serde(default)]
    pub vars: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct InternalMessage {
    pub msg_id: String,
    pub trace_id: String,
    pub req: SendRequest,
    pub created_at: DateTime<Utc>,
    pub delay_secs: i64, // 支持延迟发送
    pub retries: i32,
}
