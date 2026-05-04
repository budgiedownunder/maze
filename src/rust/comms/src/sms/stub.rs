use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;

use crate::error::CommsError;
use crate::provider::{DeliveryReceipt, Provider};
use crate::retry::RetryPolicy;
use crate::sms::{SmsMessage, SmsProvider};

/// In-memory `SmsProvider` that captures every dispatched `SmsMessage`
/// instead of sending it. Construct one in a test, hand a clone to the system
/// under test, and inspect captures via `last()` / `len()` / `into_iter()`.
///
/// Cloning the stub is an `Arc` clone — every clone shares the same capture
/// buffer and retry policy.
pub struct StubSmsProvider {
    inner: Arc<Inner>,
}

struct Inner {
    captures: Mutex<Vec<SmsMessage>>,
    retry_policy: RetryPolicy,
}

impl StubSmsProvider {
    pub fn new() -> Self {
        Self::with_retry_policy(RetryPolicy::no_retry())
    }

    pub fn with_retry_policy(retry_policy: RetryPolicy) -> Self {
        Self {
            inner: Arc::new(Inner {
                captures: Mutex::new(Vec::new()),
                retry_policy,
            }),
        }
    }

    /// Number of messages currently in the capture buffer.
    pub fn len(&self) -> usize {
        self.lock_captures().len()
    }

    /// True when the capture buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.lock_captures().is_empty()
    }

    /// Most recent captured message, cloned. `None` if nothing has been sent.
    pub fn last(&self) -> Option<SmsMessage> {
        self.lock_captures().last().cloned()
    }

    /// Drain the capture buffer and return the messages as an iterator.
    /// After this call the buffer is empty.
    pub fn into_iter(&self) -> std::vec::IntoIter<SmsMessage> {
        let drained: Vec<SmsMessage> = std::mem::take(&mut *self.lock_captures());
        drained.into_iter()
    }

    /// Reset the capture buffer.
    pub fn clear(&self) {
        self.lock_captures().clear();
    }

    fn lock_captures(&self) -> std::sync::MutexGuard<'_, Vec<SmsMessage>> {
        self.inner
            .captures
            .lock()
            .expect("StubSmsProvider mutex poisoned")
    }
}

impl Default for StubSmsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StubSmsProvider {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Provider for StubSmsProvider {
    fn name(&self) -> &'static str {
        "stub_sms"
    }

    fn retry_policy(&self) -> &RetryPolicy {
        &self.inner.retry_policy
    }
}

#[async_trait]
impl SmsProvider for StubSmsProvider {
    async fn send_sms(&self, msg: &SmsMessage) -> Result<DeliveryReceipt, CommsError> {
        self.lock_captures().push(msg.clone());
        Ok(DeliveryReceipt {
            provider: "stub_sms".into(),
            provider_message_id: None,
            accepted_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sms::PhoneNumber;
    use pretty_assertions::assert_eq;

    fn message(to: &str, body: &str) -> SmsMessage {
        SmsMessage {
            from: PhoneNumber::new("+15550001111"),
            to: PhoneNumber::new(to),
            body: body.into(),
            idempotency_key: None,
        }
    }

    #[tokio::test]
    async fn captures_dispatched_messages_in_order() {
        let stub = StubSmsProvider::new();
        assert!(stub.is_empty());
        assert_eq!(stub.len(), 0);
        assert_eq!(stub.last(), None);

        let first = message("+15555550100", "first");
        let second = message("+15555550101", "second");

        stub.send_sms(&first).await.expect("send first");
        assert_eq!(stub.len(), 1);
        assert_eq!(stub.last(), Some(first.clone()));

        stub.send_sms(&second).await.expect("send second");
        assert_eq!(stub.len(), 2);
        assert_eq!(stub.last(), Some(second.clone()));
    }

    #[tokio::test]
    async fn clear_resets_the_capture_buffer() {
        let stub = StubSmsProvider::new();
        stub.send_sms(&message("+15555550100", "x"))
            .await
            .expect("send");
        assert_eq!(stub.len(), 1);

        stub.clear();
        assert_eq!(stub.len(), 0);
        assert_eq!(stub.last(), None);
    }

    #[tokio::test]
    async fn into_iter_drains_the_capture_buffer() {
        let stub = StubSmsProvider::new();
        let first = message("+15555550100", "first");
        let second = message("+15555550101", "second");
        stub.send_sms(&first).await.expect("send first");
        stub.send_sms(&second).await.expect("send second");

        let drained: Vec<SmsMessage> = stub.into_iter().collect();
        assert_eq!(drained, vec![first, second]);
        assert_eq!(stub.len(), 0);
    }

    #[tokio::test]
    async fn clones_share_the_same_capture_buffer() {
        let stub = StubSmsProvider::new();
        let clone = stub.clone();
        clone
            .send_sms(&message("+15555550100", "x"))
            .await
            .expect("send via clone");
        assert_eq!(stub.len(), 1);
        assert_eq!(clone.len(), 1);
    }

    #[tokio::test]
    async fn provider_name_and_receipt_identify_the_stub() {
        let stub = StubSmsProvider::new();
        assert_eq!(stub.name(), "stub_sms");

        let receipt = stub
            .send_sms(&message("+15555550100", "x"))
            .await
            .expect("send");
        assert_eq!(receipt.provider, "stub_sms");
        assert_eq!(receipt.provider_message_id, None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_sends_do_not_lose_messages() {
        let stub = Arc::new(StubSmsProvider::new());

        let one = stub.clone();
        let two = stub.clone();
        let h1 = tokio::spawn(async move {
            one.send_sms(&message("+15555550100", "from-one"))
                .await
                .expect("send from task one");
        });
        let h2 = tokio::spawn(async move {
            two.send_sms(&message("+15555550101", "from-two"))
                .await
                .expect("send from task two");
        });
        h1.await.expect("task one");
        h2.await.expect("task two");

        assert_eq!(stub.len(), 2);
        let bodies: Vec<String> = stub.into_iter().map(|m| m.body).collect();
        assert!(bodies.contains(&"from-one".to_string()), "{bodies:?}");
        assert!(bodies.contains(&"from-two".to_string()), "{bodies:?}");
    }
}
