use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::email::{EmailAddress, EmailMessage, EmailProvider};
use crate::error::CommsError;
use crate::provider::DeliveryReceipt;
use crate::recipient::Recipient;
use crate::retry::RetryPolicy;
use crate::sms::{PhoneNumber, SmsMessage, SmsProvider};
use crate::template::{Channel, TemplateContext, TemplateRenderer};

/// Top-level dispatcher. Holds typed slots for each medium (`email`, `sms`),
/// the shared `TemplateRenderer`, and the default sender identities used when
/// `send_template` synthesises a message.
///
/// Provider slots are `Option`: a deployment can omit any medium it doesn't
/// use, and a mismatched send returns `EmailNotConfigured` / `SmsNotConfigured`
/// rather than panicking. The retry policy in use for a given send is read
/// from the dispatched provider's `Provider::retry_policy()`, so different
/// providers can have different policies side-by-side.
pub struct Comms {
    email: Option<Arc<dyn EmailProvider>>,
    sms: Option<Arc<dyn SmsProvider>>,
    renderer: TemplateRenderer,
    default_from_email: Option<EmailAddress>,
    default_sms_from: Option<PhoneNumber>,
}

impl Comms {
    pub fn new(
        renderer: TemplateRenderer,
        email: Option<Arc<dyn EmailProvider>>,
        sms: Option<Arc<dyn SmsProvider>>,
        default_from_email: Option<EmailAddress>,
        default_sms_from: Option<PhoneNumber>,
    ) -> Self {
        Self {
            renderer,
            email,
            sms,
            default_from_email,
            default_sms_from,
        }
    }

    pub fn renderer(&self) -> &TemplateRenderer {
        &self.renderer
    }

    /// Render `template_id` against `context` and dispatch the result to the
    /// medium matching `recipient`. Returns `ChannelMismatch` if the rendered
    /// template's channel does not match the recipient's variant.
    pub async fn send_template<C: Serialize>(
        &self,
        template_id: &str,
        recipient: Recipient,
        context: &C,
    ) -> Result<DeliveryReceipt, CommsError> {
        let value = serde_json::to_value(context)
            .map_err(|e| CommsError::TemplateRender(format!("context serialise: {e}")))?;
        let vars = match value {
            Value::Object(m) => m,
            _ => {
                return Err(CommsError::TemplateRender(
                    "template context must be a map / struct".into(),
                ));
            }
        };
        let ctx = TemplateContext { vars };
        let rendered = self.renderer.render(template_id, &ctx)?;

        match (rendered.channel, recipient) {
            (Channel::Email, Recipient::Email(to)) => {
                let from = self.default_from_email.clone().ok_or_else(|| {
                    CommsError::Config(
                        "send_template: no default_from_email configured".into(),
                    )
                })?;
                let msg = EmailMessage {
                    from,
                    to: vec![to],
                    cc: vec![],
                    bcc: vec![],
                    reply_to: None,
                    subject: rendered.subject.unwrap_or_default(),
                    body_text: rendered.text,
                    body_html: rendered.html,
                    headers: vec![],
                    idempotency_key: None,
                };
                self.send_email(msg).await
            }
            (Channel::Sms, Recipient::Sms(to)) => {
                let from = self.default_sms_from.clone().ok_or_else(|| {
                    CommsError::Config(
                        "send_template: no default_sms_from configured".into(),
                    )
                })?;
                let msg = SmsMessage {
                    from,
                    to,
                    body: rendered.text,
                    idempotency_key: None,
                };
                self.send_sms(msg).await
            }
            (template_ch, recip) => Err(CommsError::ChannelMismatch {
                template_channel: channel_label(template_ch).into(),
                recipient_channel: recipient_label(&recip).into(),
            }),
        }
    }

    pub async fn send_email(
        &self,
        msg: EmailMessage,
    ) -> Result<DeliveryReceipt, CommsError> {
        let provider = self
            .email
            .as_ref()
            .ok_or(CommsError::EmailNotConfigured)?;
        dispatch_with_retry(provider.retry_policy(), || provider.send_email(&msg)).await
    }

    pub async fn send_sms(
        &self,
        msg: SmsMessage,
    ) -> Result<DeliveryReceipt, CommsError> {
        let provider = self.sms.as_ref().ok_or(CommsError::SmsNotConfigured)?;
        dispatch_with_retry(provider.retry_policy(), || provider.send_sms(&msg)).await
    }
}

fn channel_label(ch: Channel) -> &'static str {
    match ch {
        Channel::Email => "email",
        Channel::Sms => "sms",
    }
}

fn recipient_label(r: &Recipient) -> &'static str {
    match r {
        Recipient::Email(_) => "email",
        Recipient::Sms(_) => "sms",
    }
}

/// Run `op` up to `policy.max_attempts` times. Transient errors trigger a
/// backoff sleep and another attempt; permanent errors return immediately.
/// `max_attempts = 1` disables retry.
async fn dispatch_with_retry<F, Fut>(
    policy: &RetryPolicy,
    mut op: F,
) -> Result<DeliveryReceipt, CommsError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<DeliveryReceipt, CommsError>>,
{
    let mut last_err: Option<CommsError> = None;
    for attempt in 1..=policy.max_attempts {
        if attempt > 1 {
            let backoff = policy.backoff_before_attempt(attempt);
            if !backoff.is_zero() {
                tokio::time::sleep(backoff).await;
            }
        }
        match op().await {
            Ok(receipt) => return Ok(receipt),
            Err(e) if e.is_transient() && attempt < policy.max_attempts => {
                last_err = Some(e);
            }
            Err(e) => return Err(e),
        }
    }
    Err(last_err
        .unwrap_or_else(|| CommsError::Provider("retry loop exhausted with no captured error".into())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    use async_trait::async_trait;
    use chrono::{DateTime, TimeZone, Utc};
    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use std::time::Duration;

    use crate::email::{EmailAddress, EmailMessage, EmailProvider};
    use crate::error::CommsError;
    use crate::provider::{DeliveryReceipt, Provider};
    use crate::recipient::Recipient;
    use crate::retry::RetryPolicy;
    use crate::sms::{PhoneNumber, SmsMessage, SmsProvider};
    use crate::template::{
        AppContext, BrandingContext, EmbeddedTemplateLoader, TemplateLoader, TemplateRenderer,
    };
    use crate::template::renderer::BrandingPartialSources;

    /// Test-only provider that records every received message and pops a
    /// pre-programmed response off a queue. When the queue is empty it falls
    /// back to a default `Ok(DeliveryReceipt)` carrying the provider name.
    struct RecordingEmailProvider {
        retry_policy: RetryPolicy,
        received: Mutex<Vec<EmailMessage>>,
        responses: Mutex<VecDeque<Result<DeliveryReceipt, CommsError>>>,
    }

    impl RecordingEmailProvider {
        fn with_retry(retry_policy: RetryPolicy) -> Self {
            Self {
                retry_policy,
                received: Mutex::new(Vec::new()),
                responses: Mutex::new(VecDeque::new()),
            }
        }

        fn enqueue(&self, response: Result<DeliveryReceipt, CommsError>) {
            self.responses
                .lock()
                .expect("responses poisoned")
                .push_back(response);
        }

        fn received(&self) -> Vec<EmailMessage> {
            self.received.lock().expect("received poisoned").clone()
        }

        fn attempts(&self) -> usize {
            self.received.lock().expect("received poisoned").len()
        }
    }

    impl Provider for RecordingEmailProvider {
        fn name(&self) -> &'static str {
            "recording_email"
        }
        fn retry_policy(&self) -> &RetryPolicy {
            &self.retry_policy
        }
    }

    #[async_trait]
    impl EmailProvider for RecordingEmailProvider {
        async fn send_email(&self, msg: &EmailMessage) -> Result<DeliveryReceipt, CommsError> {
            self.received
                .lock()
                .expect("received poisoned")
                .push(msg.clone());
            self.responses
                .lock()
                .expect("responses poisoned")
                .pop_front()
                .unwrap_or_else(|| {
                    Ok(DeliveryReceipt {
                        provider: "recording_email".into(),
                        provider_message_id: Some("default-message-id".into()),
                        accepted_at: Utc::now(),
                    })
                })
        }
    }

    struct RecordingSmsProvider {
        retry_policy: RetryPolicy,
        received: Mutex<Vec<SmsMessage>>,
        responses: Mutex<VecDeque<Result<DeliveryReceipt, CommsError>>>,
    }

    impl RecordingSmsProvider {
        fn new() -> Self {
            Self {
                retry_policy: RetryPolicy::no_retry(),
                received: Mutex::new(Vec::new()),
                responses: Mutex::new(VecDeque::new()),
            }
        }

        fn received(&self) -> Vec<SmsMessage> {
            self.received.lock().expect("received poisoned").clone()
        }
    }

    impl Provider for RecordingSmsProvider {
        fn name(&self) -> &'static str {
            "recording_sms"
        }
        fn retry_policy(&self) -> &RetryPolicy {
            &self.retry_policy
        }
    }

    #[async_trait]
    impl SmsProvider for RecordingSmsProvider {
        async fn send_sms(&self, msg: &SmsMessage) -> Result<DeliveryReceipt, CommsError> {
            self.received
                .lock()
                .expect("received poisoned")
                .push(msg.clone());
            self.responses
                .lock()
                .expect("responses poisoned")
                .pop_front()
                .unwrap_or_else(|| {
                    Ok(DeliveryReceipt {
                        provider: "recording_sms".into(),
                        provider_message_id: Some("default-sms-id".into()),
                        accepted_at: Utc::now(),
                    })
                })
        }
    }

    fn sample_app_context() -> AppContext {
        AppContext {
            app_name: "Maze".into(),
            server_url: "https://example.com".into(),
            branding: BrandingContext {
                company_name: "Maze, Inc.".into(),
                company_address: "123 Example St".into(),
                logo_url: "https://example.com/logo.png".into(),
            },
        }
    }

    fn sample_partials() -> BrandingPartialSources {
        BrandingPartialSources {
            logo_html: "<img alt=\"{{ company_name }}\">".into(),
            logo_text: "{{ company_name }}".into(),
            header_html: "<h1>{{ company_name }}</h1>".into(),
            header_text: "== {{ company_name }} ==".into(),
            footer_html: "<p>{{ company_name }}</p>".into(),
            footer_text: "{{ company_name }}".into(),
        }
    }

    fn build_renderer() -> TemplateRenderer {
        let loader: Arc<dyn TemplateLoader> = Arc::new(EmbeddedTemplateLoader::from_pairs(&[
            (
                "welcome",
                "channel = \"email\"\nsubject = \"Hello {{ name }}\"\ntext = \"Hi {{ name }}, welcome.\"\n",
            ),
            (
                "ping",
                "channel = \"sms\"\ntext = \"Maze: hi {{ name }}\"\n",
            ),
        ]));
        TemplateRenderer::new(sample_app_context(), loader, sample_partials())
            .expect("renderer construction")
    }

    fn from_email() -> EmailAddress {
        EmailAddress::with_name("noreply@example.com", "Maze")
    }

    fn from_sms() -> PhoneNumber {
        PhoneNumber::new("+15550001111")
    }

    fn email_msg() -> EmailMessage {
        EmailMessage {
            from: from_email(),
            to: vec![EmailAddress::new("alice@example.com")],
            cc: vec![],
            bcc: vec![],
            reply_to: None,
            subject: "x".into(),
            body_text: "x".into(),
            body_html: None,
            headers: vec![],
            idempotency_key: None,
        }
    }

    fn sms_msg() -> SmsMessage {
        SmsMessage {
            from: from_sms(),
            to: PhoneNumber::new("+15555550100"),
            body: "x".into(),
            idempotency_key: None,
        }
    }

    fn fixed_accepted_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 4, 12, 0, 0).unwrap()
    }

    #[derive(Serialize)]
    struct NameCtx<'a> {
        name: &'a str,
    }

    #[tokio::test]
    async fn email_recipient_routes_to_email_slot() {
        let provider = Arc::new(RecordingEmailProvider::with_retry(RetryPolicy::no_retry()));
        let comms = Comms::new(
            build_renderer(),
            Some(provider.clone()),
            None,
            Some(from_email()),
            None,
        );
        comms
            .send_template(
                "welcome",
                Recipient::Email(EmailAddress::new("alice@example.com")),
                &NameCtx { name: "Alice" },
            )
            .await
            .expect("send");

        let received = provider.received();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].subject, "Hello Alice");
        assert_eq!(received[0].body_text, "Hi Alice, welcome.");
        assert_eq!(received[0].to[0].address, "alice@example.com");
        assert_eq!(received[0].from.address, "noreply@example.com");
    }

    #[tokio::test]
    async fn sms_recipient_routes_to_sms_slot() {
        let provider = Arc::new(RecordingSmsProvider::new());
        let comms = Comms::new(
            build_renderer(),
            None,
            Some(provider.clone()),
            None,
            Some(from_sms()),
        );
        comms
            .send_template(
                "ping",
                Recipient::Sms(PhoneNumber::new("+15555550100")),
                &NameCtx { name: "Alice" },
            )
            .await
            .expect("send");

        let received = provider.received();
        assert_eq!(received.len(), 1);
        assert_eq!(received[0].body, "Maze: hi Alice");
        assert_eq!(received[0].to.as_str(), "+15555550100");
        assert_eq!(received[0].from.as_str(), "+15550001111");
    }

    #[tokio::test]
    async fn missing_email_slot_returns_email_not_configured() {
        let comms = Comms::new(build_renderer(), None, None, Some(from_email()), None);
        let err = comms.send_email(email_msg()).await.expect_err("must fail");
        assert!(matches!(err, CommsError::EmailNotConfigured));
    }

    #[tokio::test]
    async fn missing_sms_slot_returns_sms_not_configured() {
        let comms = Comms::new(build_renderer(), None, None, None, Some(from_sms()));
        let err = comms.send_sms(sms_msg()).await.expect_err("must fail");
        assert!(matches!(err, CommsError::SmsNotConfigured));
    }

    #[tokio::test]
    async fn channel_mismatch_returns_channel_mismatch_error() {
        let email_provider = Arc::new(RecordingEmailProvider::with_retry(RetryPolicy::no_retry()));
        let sms_provider = Arc::new(RecordingSmsProvider::new());
        let comms = Comms::new(
            build_renderer(),
            Some(email_provider.clone()),
            Some(sms_provider.clone()),
            Some(from_email()),
            Some(from_sms()),
        );
        // Email-channel template + SMS recipient.
        let err = comms
            .send_template(
                "welcome",
                Recipient::Sms(PhoneNumber::new("+15555550100")),
                &NameCtx { name: "Alice" },
            )
            .await
            .expect_err("must fail");
        match err {
            CommsError::ChannelMismatch {
                template_channel,
                recipient_channel,
            } => {
                assert_eq!(template_channel, "email");
                assert_eq!(recipient_channel, "sms");
            }
            other => panic!("unexpected variant: {other:?}"),
        }
        // Neither provider was dispatched.
        assert_eq!(email_provider.attempts(), 0);
        assert_eq!(sms_provider.received().len(), 0);
    }

    #[tokio::test(start_paused = true)]
    async fn retries_on_transient_then_succeeds() {
        let provider = Arc::new(RecordingEmailProvider::with_retry(RetryPolicy {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            backoff_multiplier: 2,
        }));
        provider.enqueue(Err(CommsError::Transient("blip 1".into())));
        provider.enqueue(Err(CommsError::Transient("blip 2".into())));
        provider.enqueue(Ok(DeliveryReceipt {
            provider: "recording_email".into(),
            provider_message_id: Some("succeeded".into()),
            accepted_at: fixed_accepted_at(),
        }));

        let comms = Comms::new(
            build_renderer(),
            Some(provider.clone()),
            None,
            Some(from_email()),
            None,
        );
        let receipt = comms.send_email(email_msg()).await.expect("eventually ok");
        assert_eq!(receipt.provider, "recording_email");
        assert_eq!(receipt.provider_message_id.as_deref(), Some("succeeded"));
        assert_eq!(provider.attempts(), 3);
    }

    #[tokio::test(start_paused = true)]
    async fn retry_loop_honours_max_attempts() {
        let provider = Arc::new(RecordingEmailProvider::with_retry(RetryPolicy {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            backoff_multiplier: 2,
        }));
        provider.enqueue(Err(CommsError::Transient("blip 1".into())));
        provider.enqueue(Err(CommsError::Transient("blip 2".into())));
        provider.enqueue(Err(CommsError::Transient("blip 3".into())));

        let comms = Comms::new(
            build_renderer(),
            Some(provider.clone()),
            None,
            Some(from_email()),
            None,
        );
        let err = comms.send_email(email_msg()).await.expect_err("exhausted");
        assert!(err.is_transient());
        assert_eq!(provider.attempts(), 3);
    }

    #[tokio::test]
    async fn does_not_retry_on_permanent_error() {
        let provider = Arc::new(RecordingEmailProvider::with_retry(RetryPolicy::default()));
        provider.enqueue(Err(CommsError::Provider("permanent failure".into())));
        // If the orchestrator wrongly retried, this second response would be
        // consumed too and the test would still pass attempts == 2 — so we
        // only enqueue one to make the assertion definite.

        let comms = Comms::new(
            build_renderer(),
            Some(provider.clone()),
            None,
            Some(from_email()),
            None,
        );
        let err = comms.send_email(email_msg()).await.expect_err("permanent");
        assert!(!err.is_transient(), "{err:?}");
        assert_eq!(provider.attempts(), 1);
    }

    #[tokio::test]
    async fn delivery_receipt_carries_provider_metadata() {
        let provider = Arc::new(RecordingEmailProvider::with_retry(RetryPolicy::no_retry()));
        provider.enqueue(Ok(DeliveryReceipt {
            provider: "recording_email".into(),
            provider_message_id: Some("msg-42".into()),
            accepted_at: fixed_accepted_at(),
        }));

        let comms = Comms::new(
            build_renderer(),
            Some(provider.clone()),
            None,
            Some(from_email()),
            None,
        );
        let receipt = comms.send_email(email_msg()).await.expect("send");
        assert_eq!(receipt.provider, "recording_email");
        assert_eq!(receipt.provider_message_id.as_deref(), Some("msg-42"));
        assert_eq!(receipt.accepted_at, fixed_accepted_at());
    }
}
