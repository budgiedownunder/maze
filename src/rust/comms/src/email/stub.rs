use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use chrono::Utc;

use crate::email::{EmailMessage, EmailProvider};
use crate::error::CommsError;
use crate::provider::{DeliveryReceipt, Provider};
use crate::retry::RetryPolicy;

/// In-memory `EmailProvider` that captures every dispatched `EmailMessage`
/// instead of sending it. Construct one in a test, hand a clone to the system
/// under test, and inspect captures via `last()` / `len()` / `into_iter()`.
///
/// Cloning the stub is an `Arc` clone — every clone shares the same capture
/// buffer and retry policy.
pub struct StubEmailProvider {
    inner: Arc<Inner>,
}

struct Inner {
    captures: Mutex<Vec<EmailMessage>>,
    retry_policy: RetryPolicy,
}

impl StubEmailProvider {
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
    pub fn last(&self) -> Option<EmailMessage> {
        self.lock_captures().last().cloned()
    }

    /// Drain the capture buffer and return the messages as an iterator.
    /// After this call the buffer is empty.
    pub fn into_iter(&self) -> std::vec::IntoIter<EmailMessage> {
        let drained: Vec<EmailMessage> = std::mem::take(&mut *self.lock_captures());
        drained.into_iter()
    }

    /// Reset the capture buffer.
    pub fn clear(&self) {
        self.lock_captures().clear();
    }

    fn lock_captures(&self) -> std::sync::MutexGuard<'_, Vec<EmailMessage>> {
        self.inner
            .captures
            .lock()
            .expect("StubEmailProvider mutex poisoned")
    }
}

impl Default for StubEmailProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for StubEmailProvider {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Provider for StubEmailProvider {
    fn name(&self) -> &'static str {
        "stub_email"
    }

    fn retry_policy(&self) -> &RetryPolicy {
        &self.inner.retry_policy
    }
}

#[async_trait]
impl EmailProvider for StubEmailProvider {
    async fn send_email(&self, msg: &EmailMessage) -> Result<DeliveryReceipt, CommsError> {
        self.lock_captures().push(msg.clone());
        Ok(DeliveryReceipt {
            provider: "stub_email".into(),
            provider_message_id: None,
            accepted_at: Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::email::EmailAddress;
    use pretty_assertions::assert_eq;

    fn message(to: &str, subject: &str) -> EmailMessage {
        EmailMessage {
            from: EmailAddress::new("noreply@example.com"),
            to: vec![EmailAddress::new(to)],
            cc: vec![],
            bcc: vec![],
            reply_to: None,
            subject: subject.into(),
            body_text: "body".into(),
            body_html: None,
            headers: vec![],
            idempotency_key: None,
        }
    }

    #[tokio::test]
    async fn captures_dispatched_messages_in_order() {
        let stub = StubEmailProvider::new();
        assert!(stub.is_empty());
        assert_eq!(stub.len(), 0);
        assert_eq!(stub.last(), None);

        let first = message("alice@example.com", "first");
        let second = message("bob@example.com", "second");

        stub.send_email(&first).await.expect("send first");
        assert_eq!(stub.len(), 1);
        assert_eq!(stub.last(), Some(first.clone()));

        stub.send_email(&second).await.expect("send second");
        assert_eq!(stub.len(), 2);
        assert_eq!(stub.last(), Some(second.clone()));
    }

    #[tokio::test]
    async fn clear_resets_the_capture_buffer() {
        let stub = StubEmailProvider::new();
        stub.send_email(&message("alice@example.com", "x"))
            .await
            .expect("send");
        assert_eq!(stub.len(), 1);

        stub.clear();
        assert_eq!(stub.len(), 0);
        assert_eq!(stub.last(), None);
    }

    #[tokio::test]
    async fn into_iter_drains_the_capture_buffer() {
        let stub = StubEmailProvider::new();
        let first = message("alice@example.com", "first");
        let second = message("bob@example.com", "second");
        stub.send_email(&first).await.expect("send first");
        stub.send_email(&second).await.expect("send second");

        let drained: Vec<EmailMessage> = stub.into_iter().collect();
        assert_eq!(drained, vec![first, second]);
        assert_eq!(stub.len(), 0);
    }

    #[tokio::test]
    async fn clones_share_the_same_capture_buffer() {
        let stub = StubEmailProvider::new();
        let clone = stub.clone();
        clone
            .send_email(&message("alice@example.com", "x"))
            .await
            .expect("send via clone");
        assert_eq!(stub.len(), 1);
        assert_eq!(clone.len(), 1);
    }

    #[tokio::test]
    async fn provider_name_and_receipt_identify_the_stub() {
        let stub = StubEmailProvider::new();
        assert_eq!(stub.name(), "stub_email");

        let receipt = stub
            .send_email(&message("alice@example.com", "x"))
            .await
            .expect("send");
        assert_eq!(receipt.provider, "stub_email");
        assert_eq!(receipt.provider_message_id, None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn concurrent_sends_do_not_lose_messages() {
        let stub = Arc::new(StubEmailProvider::new());

        let one = stub.clone();
        let two = stub.clone();
        let h1 = tokio::spawn(async move {
            one.send_email(&message("alice@example.com", "from-one"))
                .await
                .expect("send from task one");
        });
        let h2 = tokio::spawn(async move {
            two.send_email(&message("bob@example.com", "from-two"))
                .await
                .expect("send from task two");
        });
        h1.await.expect("task one");
        h2.await.expect("task two");

        assert_eq!(stub.len(), 2);
        let subjects: Vec<String> = stub.into_iter().map(|m| m.subject).collect();
        assert!(subjects.contains(&"from-one".to_string()), "{subjects:?}");
        assert!(subjects.contains(&"from-two".to_string()), "{subjects:?}");
    }
}
