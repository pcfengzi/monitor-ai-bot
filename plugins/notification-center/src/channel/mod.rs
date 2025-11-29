mod sms;
mod email;
mod push;
mod inbox;

pub async fn send_by_best_channel(
    user_id: &str,
    scene: &str,
    content: &str,
) -> String {
    match scene {
        "order_payed" => {
            if sms::send(user_id, content).await {
                return "sms".into();
            }
            if push::send(user_id, content).await {
                return "push".into();
            }
            "inbox".into()
        }

        _ => {
            if inbox::send(user_id, content).await {
                return "inbox".into();
            }
            "none".into()
        }
    }
}
