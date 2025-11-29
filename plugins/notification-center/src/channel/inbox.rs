// plugins/notification-center/src/channel/inbox.rs

/// 模拟“站内信”渠道
///
/// 在真实系统里，站内信通常是：
/// - 写入一张 `user_inbox_messages` 表
/// - 前端轮询 / WebSocket 拉取
/// - 或通过单独的消息中心接口查询
pub async fn send(user_id: &str, content: &str) -> bool {
    println!(
        "[Notification][Inbox] to user_id={} | content={}",
        user_id, content
    );

    // TODO：后续可以在这里：
/// - 调用 api-server 提供的 `/inbox/messages` 写入接口
/// - 或者直接写 SQLite / 业务库中的站内信表
///   例如插入字段：user_id, title, content, status, created_at

    true
}
