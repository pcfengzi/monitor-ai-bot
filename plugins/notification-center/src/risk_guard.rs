// plugins/notification-center/src/risk_guard.rs
use chrono::{Local, Timelike};
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

lazy_static::lazy_static! {
    static ref BLACKLIST: Mutex<Vec<String>> = Mutex::new(Vec::new());
    static ref RATE_LIMIT: Mutex<HashMap<(String, String), Vec<i64>>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Error)]
pub enum RiskError {
    #[error("user is in blacklist")]
    Blacklist,
    #[error("too many requests")]
    TooManyRequests,
    #[error("quiet time")]
    QuietTime,
}

/// 简单风控：黑名单 + 频控 + 夜间免打扰
pub fn check_risk(user_id: &str, scene: &str) -> Result<(), RiskError> {
    // 黑名单
    if BLACKLIST
        .lock()
        .unwrap()
        .iter()
        .any(|u| u == user_id)
    {
        return Err(RiskError::Blacklist);
    }

    // 夜间免打扰（22:00-8:00）
    let now = Local::now();
    let hour = now.hour();
    if hour >= 22 || hour < 8 {
        // 这里只对营销类/非强制场景可以限制；简化直接限制全部
        return Err(RiskError::QuietTime);
    }

    // 简单频控：1 分钟内同 user+scene 不超过 5 次
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;

        let mut rl = RATE_LIMIT.lock().unwrap();
        let key = (user_id.to_string(), scene.to_string());
        let list = rl.entry(key).or_default();

        // 过滤掉 60 秒前的数据
        list.retain(|ts| now_ms - *ts <= 60_000);

        if list.len() as i64 >= 5 {
            return Err(RiskError::TooManyRequests);
        }

        list.push(now_ms);
    }

    Ok(())
}
