// SSE push messages

use serde::{Deserialize, Serialize};
use crate::config::state::AppState;
use crate::errors::app_error::AppError;

#[derive(Deserialize, Serialize, Debug)]
pub struct SSEPushPayload {
    pub message_source: String, // notification type
    pub message_level: String, // message level
    pub message: String, // message body
}

pub async fn sse_push_message(
    state: &AppState,
    user_uuid: String,
    payload: SSEPushPayload
) -> Result<(), AppError> {
    let senders = state.sse_senders.lock().await;
    let message = serde_json::to_value(&payload).unwrap().to_string();

    if let Some(sender) = senders.get(&user_uuid) {
        // Try to send the message
        // User is online, try to send
        if sender.send(message.clone()).await.is_ok() {
            // Sent
            tracing::debug!("Message [{:?}] sent to online user {}", &message, user_uuid);
            return Ok(());
        } else {
            // Send failed: the user just disconnected, so keep it as an offline message
            tracing::info!("User {} just disconnected. Storing message.", user_uuid);
            // Nothing else to do but store it.
        }
    } else {
        // User is offline
        tracing::info!("User {} is offline. Storing message.", user_uuid);
    }

    drop(senders);
    // Store the message if the user is offline
    let cache_key = redis_offline_key(user_uuid);

    let redis_conn = &mut state.redis.get_multiplexed_async_connection().await?;
    let _: () = redis::cmd("LPUSH")
        .arg(cache_key)
        .arg(message)
        .exec_async(redis_conn).await?;

    Ok(())

}


pub fn redis_offline_key(user_uuid: String) -> String {
    format!("offline:messages:{}", user_uuid)
}