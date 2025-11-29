// sms.rs
use tokio::time::{sleep, Duration};

pub async fn send(_user_id: &str, _content: &str) -> (bool, Option<String>, f64) {
    let start = std::time::Instant::now();
    // 这里模拟一下网络延迟
    sleep(Duration::from_millis(80)).await;
    // 你可以在这里调用真实短信供应商
    (true, None, start.elapsed().as_millis() as f64)
}
