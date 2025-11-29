// plugins/notification-center/src/channel/mod.rs
pub mod sms;
pub mod email;
pub mod push;
pub mod inbox;

use rand::Rng;

use crate::db;
use crate::metrics_bridge::HostBridge;
use crate::types::{Channel, MessageStatus};
use crate::host_bridge;

/// 场景分类：这里可以以后从 DB 配置，这里先写死
pub enum SceneCategory {
    StrongVerify,
    StrongRemind,
    Marketing,
    UserCare,
    SystemNotify,
}

pub fn classify_scene(scene: &str) -> SceneCategory {
    // 简单规则：你可以以后改成配置化
    if scene.contains("verify") || scene.contains("otp") {
        SceneCategory::StrongVerify
    } else if scene.contains("order") || scene.contains("payment") {
        SceneCategory::StrongRemind
    } else if scene.contains("marketing") || scene.contains("campaign") {
        SceneCategory::Marketing
    } else if scene.contains("birthday") || scene.contains("care") {
        SceneCategory::UserCare
    } else {
        SceneCategory::SystemNotify
    }
}

/// 根据场景，给出渠道优先级列表
fn priority_channels(scene: &str, hint: Option<&str>) -> Vec<Channel> {
    use SceneCategory::*;
    let cat = classify_scene(scene);

    let mut base = match cat {
        StrongVerify => vec![Channel::Sms, Channel::Inbox],
        StrongRemind => vec![Channel::Push, Channel::Sms, Channel::Inbox],
        Marketing => vec![Channel::Push, Channel::Email, Channel::Inbox],
        UserCare => vec![Channel::Inbox, Channel::Email, Channel::Push],
        SystemNotify => vec![Channel::Inbox, Channel::Email],
    };

    // 如果有 hint，就把 hint 挪到最前面
    if let Some(h) = hint {
        let ch = match h {
            "sms" => Some(Channel::Sms),
            "email" => Some(Channel::Email),
            "push" => Some(Channel::Push),
            "inbox" => Some(Channel::Inbox),
            _ => None,
        };

        if let Some(c) = ch {
            if let Some(pos) = base.iter().position(|x| x.as_str() == c.as_str()) {
                base.remove(pos);
            }
            base.insert(0, c);
        }
    }

    base
}

/// 多渠道发送 + 自动降级
pub async fn send_with_fallback(
    user_id: &str,
    scene: &str,
    content: &str,
    channel_hint: Option<&str>,
) -> (Channel, MessageStatus, Option<String>, f64) {
    let db = db::db();
    let pri = priority_channels(scene, channel_hint);
    let mut rng = rand::thread_rng();

    for ch in pri {
        // 用户偏好：允许这个渠道吗？
        match db.is_channel_enabled(user_id, &ch).await {
            Ok(false) => {
                continue; // 用户关掉了这个渠道
            }
            Err(e) => {
                HostBridge::log_static(
                    plugin_api::LogLevel::Warn,
                    &format!("[notification-center] check channel pref error: {e}"),
                );
            }
            _ => {}
        }

        // 模拟渠道不可用：这里先简单用随机（现实中应该是 SDK / API 返回）
        let channel_up: bool = rng.gen_range(0..100) > 5; // 5% 概率认定渠道 down
        if !channel_up {
            HostBridge::log_static(
                plugin_api::LogLevel::Warn,
                &format!(
                    "[notification-center] channel {} unavailable, try fallback",
                    ch.as_str()
                ),
            );
            continue;
        }

        // 真正发送
        let (ok, err_msg, latency_ms) = match ch {
            Channel::Sms => sms::send(user_id, content).await,
            Channel::Email => email::send(user_id, content).await,
            Channel::Push => push::send(user_id, content).await,
            Channel::Inbox => inbox::send(user_id, content).await,
        };

        HostBridge::metric_static(
            &format!("notification_channel_latency_ms_{}", ch.as_str()),
            latency_ms,
        );

        if ok {
            return (ch, MessageStatus::Delivered, None, latency_ms);
        } else {
            HostBridge::log_static(
                plugin_api::LogLevel::Warn,
                &format!(
                    "[notification-center] send via {} failed: {:?}",
                    ch.as_str(),
                    err_msg
                ),
            );
            // 失败，尝试下一个渠道
        }
    }

    // 所有渠道都失败
    (
        Channel::Inbox,
        MessageStatus::Failed,
        Some("all channels failed".into()),
        0.0,
    )
}
