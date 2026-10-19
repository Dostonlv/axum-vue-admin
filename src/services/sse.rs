// Global SSE push service

use crate::common::sse::redis_offline_key;
use crate::config::state::{AppState, SSESenders};
use crate::schemas::auth::AuthUser;
use redis::AsyncCommands;
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, Stream, StreamExt};

// Removes the sender from the online users when the stream is dropped
struct SseStreamCleanup {
    user_uuid: String,
    sse_senders: SSESenders,
}

impl Drop for SseStreamCleanup {
    fn drop(&mut self) {
        let user_uuid = self.user_uuid.clone();
        let sse_senders = self.sse_senders.clone();

        // Can't .await inside drop, so spawn a task for the async cleanup
        tokio::spawn(async move {
            sse_senders.lock().await.remove(&user_uuid);
            tracing::debug!(
                "SSE connection for user {} closed, sender cleaned up.",
                user_uuid
            );
        });
    }
}

// Global message push
#[derive(Clone)]
pub struct SSEService {
    app_state: AppState,
}

impl SSEService {
    pub fn new(app_state: AppState) -> Self {
        Self { app_state }
    }

    async fn send_offline_messages(
        state_clone: AppState,
        user_uuid: String,
        tx: mpsc::Sender<String>,
    ) {
        // 1. Get a Redis connection
        let mut conn = match state_clone.redis.get_multiplexed_async_connection().await {
            Ok(conn) => conn,
            Err(e) => {
                tracing::error!("[user {}] Cannot get a Redis connection for offline messages: {}", user_uuid, e);
                return;
            }
        };

        let key = redis_offline_key(user_uuid.clone());

        // 2. Read all messages with LRANGE
        // LRANGE key 0 -1 returns the whole list
        let messages: Vec<String> = match conn.lrange(&key, 0, -1).await {
            Ok(messages) => messages,
            Err(e) => {
                tracing::error!(
                    "[user {}] LRANGE on Redis key '{}' failed: {}",
                    user_uuid,
                    key,
                    e
                );
                return;
            }
        };

        // Nothing stored, return early
        if messages.is_empty() {
            return;
        }

        tracing::debug!(
            "[user {}] Found {} offline messages in Redis. Sending...",
            user_uuid,
            messages.len()
        );

        // 3. Send in reverse to keep the original order
        // LPUSH pushes to the left, so the newest message sits at index 0.
        // Walking from the back sends them oldest to newest.
        for msg in messages.iter().rev() {
            if tx.send(msg.clone()).await.is_err() {
                // If the client disconnects mid-way we stop sending
                // and keep the messages in Redis until the next connection.
                tracing::error!(
                    "[user {}] Client disconnected while sending offline messages. Aborting.",
                    user_uuid.clone()
                );
                return; // exit the task
            }
        }

        // 4. Everything sent, delete the Redis key
        tracing::debug!(
            "[user {}] All offline messages sent. Deleting Redis key '{}'.",
            user_uuid,
            key
        );

        let _: () = conn.del(&key).await.unwrap_or_else(|e| {
            tracing::error!(
                "[user {}] Cannot delete Redis key '{}': {}. Messages may be sent twice.",
                user_uuid,
                key,
                e
            );
        });
    }

    pub async fn global_message(&self, current_user: AuthUser) -> impl Stream<Item = String> + use<> {
        // New MPSC channel for this user's SSE events
        let user_uuid = current_user.sub;
        let (tx, rx) = mpsc::channel(100); // buffer of 100
        let state_clone = self.app_state.clone();
        let tx_clone = tx.clone();

        // ----------------- send offline messages: start -----------------
        tokio::spawn(Self::send_offline_messages(
            state_clone,
            user_uuid.clone(),
            tx_clone,
        ));

        // ----------------- send offline messages: end -----------------

        // Store the sender (tx) in the shared state, keyed by user_id
        // The HashMap needs a lock to be modified safely
        self.app_state
            .sse_senders
            .lock()
            .await
            .insert(user_uuid.clone(), tx.clone());
        tracing::debug!("SSE connection established for user {}", user_uuid.clone());

        let cleanup_guard = SseStreamCleanup {
            user_uuid,
            sse_senders: self.app_state.sse_senders.clone(),
        };

        ReceiverStream::new(rx)
            // Use `map` or `inspect` to tie cleanup_guard's lifetime to the stream
            .map(move |res| {
                // The closure borrows cleanup_guard and keeps it alive as long as the stream is
                // When the stream is dropped, the closure and its cleanup_guard go with it
                let _ = &cleanup_guard;
                res
            })
    }
}
