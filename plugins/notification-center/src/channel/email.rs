// plugins/notification-center/src/channel/email.rs

/// 模拟邮件发送渠道
///
/// 真实情况你可能会：
/// - 调用 SMTP
/// - 或第三方邮件服务（如 SendGrid、Mailgun、阿里云邮件推送等）
///
/// 返回值：true 表示“发送请求成功提交”（不等于用户一定收到），
/// 后续可结合回执做更精细的状态管理。
pub async fn send(user_id: &str, content: &str) -> bool {
    // 这里简单打印，代表发送了一封邮件
    println!(
        "[Notification][Email] to user_id={} | content={}",
        user_id, content
    );

    // TODO：将来可以在这里接入真正的邮件服务，例如：
    // - 读取用户 email 地址（从用户中心或本地映射）
    // - 调用外部 HTTP API / SMTP 客户端发送邮件
    // - 根据对方返回结果决定返回 true/false

    true
}
