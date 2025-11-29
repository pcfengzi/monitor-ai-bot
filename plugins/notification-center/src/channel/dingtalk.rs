use tracing::{error, info};

/// 发送钉钉通知（当前为 mock 实现，返回 true 代表“模拟发送成功”）
///
/// - `user_id`：可以是内部用户 id、钉钉 userId、或你自己的映射 id
/// - `content`：已经模板渲染后的最终文案
pub async fn send(user_id: &str, content: &str) -> (bool, Option<String>, f64) {
    if user_id.trim().is_empty() {
        let msg = "[notification-center][dingtalk] user_id 为空，放弃发送";
        error!("{msg}");
        return (false, Some(msg.to_string()), 0.0);
    }

    if content.trim().is_empty() {
        let msg = "[notification-center][dingtalk] content 为空，放弃发送";
        error!("{msg}");
        return (false, Some(msg.to_string()), 0.0);
    }

    // TODO: 在这里接入真实的钉钉机器人 / 工作通知等 API
    //
    // 典型做法：
    // 1. 从 DB 或配置中读取 webhook / appKey / appSecret 等
    // 2. 拼装请求体（markdown / text 等）
    // 3. 通过 reqwest 之类发 HTTP 请求
    // 4. 根据返回结果判断是否发送成功

    info!(
        "[notification-center][dingtalk] mock 发送成功 -> user_id={}, content={}",
        user_id, content
    );

    // 这里先简单返回 0ms 延迟的 mock 值
    (true, None, 0.0)
}
