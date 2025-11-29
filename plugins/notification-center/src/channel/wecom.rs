use tracing::{error, info};

/// 发送企业微信通知（当前为 mock 实现，返回 true 代表“模拟发送成功”）
///
/// - `user_id`：可以是企业微信的 userId、手机号映射、或你自己的内部 id
/// - `content`：已经模板渲染后的最终文案
pub async fn send(user_id: &str, content: &str) -> (bool, Option<String>, f64) {
    if user_id.trim().is_empty() {
        let msg = "[notification-center][wecom] user_id 为空，放弃发送";
        error!("{msg}");
        return (false, Some(msg.to_string()), 0.0);
    }

    if content.trim().is_empty() {
        let msg = "[notification-center][wecom] content 为空，放弃发送";
        error!("{msg}");
        return (false, Some(msg.to_string()), 0.0);
    }

    // TODO: 在这里接入真实的企业微信机器人 / 应用消息 API
    //
    // 典型做法：
    // 1. 从 DB / 配置中读取 corp_id、agent_id、secret 或 robot webhook
    // 2. 拼装消息体（text / markdown / news 等）
    // 3. 通过 reqwest 调用企业微信 HTTP 接口
    // 4. 根据 errcode/errmsg 判断是否成功

    info!(
        "[notification-center][wecom] mock 发送成功 -> user_id={}, content={}",
        user_id, content
    );

    (true, None, 0.0)
}
