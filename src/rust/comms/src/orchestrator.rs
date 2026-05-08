use std::sync::Arc;

use serde::Serialize;
use serde_json::Value;

use crate::email::{EmailAddress, EmailMessage, EmailProvider};
use crate::error::CommsError;
use crate::provider::DeliveryReceipt;
use crate::retry::RetryPolicy;
use crate::template::{TemplateContext, TemplateRenderer};

/// Top-level dispatcher. Holds the email provider slot, the shared
/// `TemplateRenderer`, and the default sender identity used when
/// `send_template` synthesises a message.
///
/// The provider slot is `Option`: a deployment can omit it (e.g. when
/// notifications are disabled) and `send_email` returns `EmailNotConfigured`
/// rather than panicking. The retry policy applied to a send is read from
/// the dispatched provider's `Provider::retry_policy()`, so different
/// providers can carry different policies side-by-side.
pub struct Comms {
    email: Option<Arc<dyn EmailProvider>>,
    renderer: TemplateRenderer,
    default_from_email: Option<EmailAddress>,
}

impl Comms {
    /// Build a `Comms` from a renderer, an optional email provider, and
    /// an optional default-from address. Pass `None` for `email` to leave
    /// the email slot unconfigured (subsequent `send_email` /
    /// `send_template` calls will return [`CommsError::EmailNotConfigured`]).
    ///
    /// # Examples
    ///
    /// ```
    /// # use comms::{AppContext, BrandingContext, BrandingPartialSources, Comms,
    /// #             EmbeddedTemplateLoader, TemplateLoader, TemplateRenderer};
    /// # use std::sync::Arc;
    /// # let renderer = TemplateRenderer::new(
    /// #     AppContext { app_name: "App".into(), from_name: "T".into(), server_url: "https://x".into(),
    /// #         branding: BrandingContext { company_name: "X".into(), company_address: "A".into(),
    /// #             company_url: "https://x".into(), logo_url: "https://x".into() } },
    /// #     Arc::new(EmbeddedTemplateLoader::new()) as Arc<dyn TemplateLoader>,
    /// #     BrandingPartialSources { logo_html: String::new(), logo_text: String::new(),
    /// #         header_html: String::new(), header_text: String::new(),
    /// #         footer_html: String::new(), footer_text: String::new() },
    /// # ).expect("renderer");
    /// let comms = Comms::new(renderer, None, None);
    /// ```
    pub fn new(
        renderer: TemplateRenderer,
        email: Option<Arc<dyn EmailProvider>>,
        default_from_email: Option<EmailAddress>,
    ) -> Self {
        Self {
            renderer,
            email,
            default_from_email,
        }
    }

    /// Returns the configured email provider's name (e.g. `"mailgun"`,
    /// `"stub_email"`), or `None` when the email slot is unconfigured.
    /// Used by the audit-log path so each row records which provider
    /// actually carried the send.
    ///
    /// # Examples
    ///
    /// ```
    /// # use comms::{AppContext, BrandingContext, BrandingPartialSources, Comms,
    /// #             EmbeddedTemplateLoader, TemplateLoader, TemplateRenderer};
    /// # use std::sync::Arc;
    /// # let renderer = TemplateRenderer::new(
    /// #     AppContext { app_name: "App".into(), from_name: "T".into(), server_url: "https://x".into(),
    /// #         branding: BrandingContext { company_name: "X".into(), company_address: "A".into(),
    /// #             company_url: "https://x".into(), logo_url: "https://x".into() } },
    /// #     Arc::new(EmbeddedTemplateLoader::new()) as Arc<dyn TemplateLoader>,
    /// #     BrandingPartialSources { logo_html: String::new(), logo_text: String::new(),
    /// #         header_html: String::new(), header_text: String::new(),
    /// #         footer_html: String::new(), footer_text: String::new() },
    /// # ).expect("renderer");
    /// let comms = Comms::new(renderer, None, None);
    /// assert_eq!(comms.email_provider_name(), None);
    /// ```
    pub fn email_provider_name(&self) -> Option<&'static str> {
        self.email.as_ref().map(|p| p.name())
    }

    /// Returns a borrow of the underlying `TemplateRenderer` so callers
    /// can probe template metadata without dispatching anything.
    ///
    /// # Examples
    ///
    /// ```
    /// # use comms::{AppContext, BrandingContext, BrandingPartialSources, Comms,
    /// #             EmbeddedTemplateLoader, TemplateLoader, TemplateRenderer};
    /// # use std::sync::Arc;
    /// # let renderer = TemplateRenderer::new(
    /// #     AppContext { app_name: "App".into(), from_name: "T".into(), server_url: "https://x".into(),
    /// #         branding: BrandingContext { company_name: "X".into(), company_address: "A".into(),
    /// #             company_url: "https://x".into(), logo_url: "https://x".into() } },
    /// #     Arc::new(EmbeddedTemplateLoader::new()) as Arc<dyn TemplateLoader>,
    /// #     BrandingPartialSources { logo_html: String::new(), logo_text: String::new(),
    /// #         header_html: String::new(), header_text: String::new(),
    /// #         footer_html: String::new(), footer_text: String::new() },
    /// # ).expect("renderer");
    /// let comms = Comms::new(renderer, None, None);
    /// let _renderer = comms.renderer();
    /// ```
    pub fn renderer(&self) -> &TemplateRenderer {
        &self.renderer
    }

    /// Render `template_id` against `context` and dispatch the result as an
    /// email to `to`. The `from` address is taken from the configured
    /// `default_from_email`; if unset, returns `Config`.
    pub async fn send_template<C: Serialize>(
        &self,
        template_id: &str,
        to: EmailAddress,
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

        let from = self.default_from_email.clone().ok_or_else(|| {
            CommsError::Config("send_template: no default_from_email configured".into())
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

    /// Dispatch a fully-formed [`EmailMessage`] through the configured
    /// email provider, applying the provider's `RetryPolicy` to transient
    /// failures. Returns [`CommsError::EmailNotConfigured`] if no email
    /// provider is wired.
    ///
    /// # Examples
    ///
    /// Dispatching against a `Comms` with no email provider returns a
    /// clean `EmailNotConfigured` error
    /// ```
    /// # tokio_test::block_on(async {
    /// # use comms::{AppContext, BrandingContext, BrandingPartialSources, Comms,
    /// #             CommsError, EmailAddress, EmailMessage, EmbeddedTemplateLoader,
    /// #             TemplateLoader, TemplateRenderer};
    /// # use std::sync::Arc;
    /// # let renderer = TemplateRenderer::new(
    /// #     AppContext { app_name: "App".into(), from_name: "T".into(), server_url: "https://x".into(),
    /// #         branding: BrandingContext { company_name: "X".into(), company_address: "A".into(),
    /// #             company_url: "https://x".into(), logo_url: "https://x".into() } },
    /// #     Arc::new(EmbeddedTemplateLoader::new()) as Arc<dyn TemplateLoader>,
    /// #     BrandingPartialSources { logo_html: String::new(), logo_text: String::new(),
    /// #         header_html: String::new(), header_text: String::new(),
    /// #         footer_html: String::new(), footer_text: String::new() },
    /// # ).expect("renderer");
    /// let comms = Comms::new(renderer, None, None);
    /// let msg = EmailMessage {
    ///     from: EmailAddress::new("noreply@example.com"),
    ///     to: vec![EmailAddress::new("alice@example.com")],
    ///     cc: vec![], bcc: vec![], reply_to: None,
    ///     subject: "Hi".into(), body_text: "Hi".into(), body_html: None,
    ///     headers: vec![], idempotency_key: None,
    /// };
    /// let err = comms.send_email(msg).await.expect_err("no provider");
    /// assert!(matches!(err, CommsError::EmailNotConfigured));
    /// # });
    /// ```
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
    use crate::retry::RetryPolicy;
    use crate::template::renderer::BrandingPartialSources;
    use crate::template::{
        AppContext, BrandingContext, EmbeddedTemplateLoader, TemplateLoader, TemplateRenderer,
    };

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

    fn sample_app_context() -> AppContext {
        AppContext {
            app_name: "Maze".into(),
            from_name: "The Maze Team".into(),
            server_url: "https://example.com".into(),
            branding: BrandingContext {
                company_name: "Maze, Inc.".into(),
                company_address: "123 Example St".into(),
                company_url: "https://example.com".into(),
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
        let loader: Arc<dyn TemplateLoader> = Arc::new(EmbeddedTemplateLoader::from_pairs(&[(
            "welcome",
            "subject = \"Hello {{ name }}\"\ntext = \"Hi {{ name }}, welcome.\"\n",
        )]));
        TemplateRenderer::new(sample_app_context(), loader, sample_partials())
            .expect("renderer construction")
    }

    fn from_email() -> EmailAddress {
        EmailAddress::with_name("noreply@example.com", "Maze")
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

    fn fixed_accepted_at() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 4, 12, 0, 0).unwrap()
    }

    #[derive(Serialize)]
    struct NameCtx<'a> {
        name: &'a str,
    }

    #[tokio::test]
    async fn send_template_dispatches_to_email_provider() {
        let provider = Arc::new(RecordingEmailProvider::with_retry(RetryPolicy::no_retry()));
        let comms = Comms::new(
            build_renderer(),
            Some(provider.clone()),
            Some(from_email()),
        );
        comms
            .send_template(
                "welcome",
                EmailAddress::new("alice@example.com"),
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
    async fn missing_email_slot_returns_email_not_configured() {
        let comms = Comms::new(build_renderer(), None, Some(from_email()));
        let err = comms.send_email(email_msg()).await.expect_err("must fail");
        assert!(matches!(err, CommsError::EmailNotConfigured));
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
            Some(from_email()),
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
            Some(from_email()),
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
            Some(from_email()),
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
            Some(from_email()),
        );
        let receipt = comms.send_email(email_msg()).await.expect("send");
        assert_eq!(receipt.provider, "recording_email");
        assert_eq!(receipt.provider_message_id.as_deref(), Some("msg-42"));
        assert_eq!(receipt.accepted_at, fixed_accepted_at());
    }
}
