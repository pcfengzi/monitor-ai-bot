pub async fn send(user_id: &str, content: &str) -> bool {
    println!("[SMS] to={}, {}", user_id, content);
    true
}
