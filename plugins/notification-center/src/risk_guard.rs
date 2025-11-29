use chrono::{Utc, Duration};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref LAST_SENT: Mutex<HashMap<String, i64>> = Mutex::new(HashMap::new());
    static ref BLACKLIST: Vec<String> = vec!["bad_user".into()];
}

pub fn check_risk(user_id: &str, scene: &str) -> Result<(), String> {
    if BLACKLIST.contains(&user_id.to_string()) {
        return Err("blacklisted".into());
    }

    let mut lock = LAST_SENT.lock().unwrap();
    let key = format!("{user_id}:{scene}");
    let now = Utc::now().timestamp();

    if let Some(last) = lock.get(&key) {
        if now - last < 5 {
            return Err("too frequent".into());
        }
    }

    lock.insert(key, now);
    Ok(())
}
