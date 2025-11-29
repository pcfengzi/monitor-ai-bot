// inbox.rs
use tokio::time::{sleep, Duration};

pub async fn send(_user_id: &str, _content: &str) -> (bool, Option<String>, f64) {
    let start = std::time::Instant::now();
    sleep(Duration::from_millis(10)).await;
    (true, None, start.elapsed().as_millis() as f64)
}
