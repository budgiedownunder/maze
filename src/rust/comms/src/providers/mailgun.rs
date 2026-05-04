use async_trait::async_trait;
use chrono::Utc;
use serde::Deserialize;

use crate::email::{EmailAddress, EmailMessage, EmailProvider};
use crate::error::CommsError;
use crate::provider::{DeliveryReceipt, Provider};
use crate::retry::RetryPolicy;

/// Mailgun regional API endpoint. The two production hosts live on different
/// data planes; the choice is driven by where the sending domain is
/// provisioned in the Mailgun control panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MailgunRegion {
    #[default]
    Us,
    Eu,
}

impl MailgunRegion {
    fn host(self) -> &'static str {
        match self {
            MailgunRegion::Us => "api.mailgun.net",
            MailgunRegion::Eu => "api.eu.mailgun.net",
        }
    }
}

/// Configuration for `MailgunProvider`.
///
/// `domain` is the sending domain registered with Mailgun (for example
/// `mg.example.com`). `api_key` is the private API key — handled as a
/// secret; not logged. `region` selects the Mailgun data plane.
///
/// `base_url_override` replaces `https://<region>` for tests — leave it
/// `None` in production.
pub struct MailgunConfig {
    pub domain: String,
    pub api_key: String,
    pub region: MailgunRegion,
    pub base_url_override: Option<String>,
}

impl MailgunConfig {
    pub fn new(domain: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            api_key: api_key.into(),
            region: MailgunRegion::default(),
            base_url_override: None,
        }
    }

    pub fn with_region(mut self, region: MailgunRegion) -> Self {
        self.region = region;
        self
    }
}

#[derive(Debug, Deserialize)]
struct MailgunSendResponse {
    id: String,
}

/// `EmailProvider` backed by Mailgun's HTTP messages API. Cloning is not
/// supported — wrap an instance in `Arc` if multiple owners need to share
/// the underlying `reqwest::Client`.
pub struct MailgunProvider {
    config: MailgunConfig,
    http: reqwest::Client,
    retry_policy: RetryPolicy,
}

impl MailgunProvider {
    pub fn new(config: MailgunConfig) -> Result<Self, CommsError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| CommsError::Config(format!("reqwest client: {e}")))?;
        Ok(Self::with_http(config, http))
    }

    pub fn with_http(config: MailgunConfig, http: reqwest::Client) -> Self {
        Self {
            config,
            http,
            retry_policy: RetryPolicy::default(),
        }
    }

    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    fn endpoint_url(&self) -> String {
        match &self.config.base_url_override {
            Some(base) => format!("{}/v3/{}/messages", base.trim_end_matches('/'), self.config.domain),
            None => format!(
                "https://{}/v3/{}/messages",
                self.config.region.host(),
                self.config.domain
            ),
        }
    }
}

impl Provider for MailgunProvider {
    fn name(&self) -> &'static str {
        "mailgun"
    }

    fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }
}

#[async_trait]
impl EmailProvider for MailgunProvider {
    async fn send_email(&self, msg: &EmailMessage) -> Result<DeliveryReceipt, CommsError> {
        let url = self.endpoint_url();
        let mut form: Vec<(String, String)> = Vec::new();
        form.push(("from".into(), format_address(&msg.from)));
        for to in &msg.to {
            form.push(("to".into(), format_address(to)));
        }
        for cc in &msg.cc {
            form.push(("cc".into(), format_address(cc)));
        }
        for bcc in &msg.bcc {
            form.push(("bcc".into(), format_address(bcc)));
        }
        if let Some(reply_to) = &msg.reply_to {
            form.push(("h:Reply-To".into(), format_address(reply_to)));
        }
        form.push(("subject".into(), msg.subject.clone()));
        form.push(("text".into(), msg.body_text.clone()));
        if let Some(html) = &msg.body_html {
            form.push(("html".into(), html.clone()));
        }
        // Mailgun encodes each custom header as a separate form field
        // keyed `h:<Header-Name>`.
        for (name, value) in &msg.headers {
            form.push((format!("h:{name}"), value.clone()));
        }

        let response = self
            .http
            .post(&url)
            .basic_auth("api", Some(&self.config.api_key))
            .form(&form)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() || e.is_connect() {
                    CommsError::Transient(format!("mailgun: {e}"))
                } else {
                    CommsError::Provider(format!("mailgun: {e}"))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(CommsError::ProviderHttp {
                status: status.as_u16(),
                body,
            });
        }

        let body: MailgunSendResponse = response
            .json()
            .await
            .map_err(|e| CommsError::Provider(format!("mailgun response decode: {e}")))?;

        Ok(DeliveryReceipt {
            provider: "mailgun".into(),
            provider_message_id: Some(body.id),
            accepted_at: Utc::now(),
        })
    }
}

/// Format an `EmailAddress` in RFC 5322 mailbox form. Display names are
/// passed through unquoted — callers are responsible for not putting RFC
/// 5322 specials in their display names. (The common cases — "Maze",
/// "Acme Inc." — are well-formed without quoting.)
fn format_address(addr: &EmailAddress) -> String {
    match &addr.display_name {
        Some(name) if !name.is_empty() => format!("{name} <{}>", addr.address),
        _ => addr.address.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use wiremock::matchers::{body_string_contains, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn sample_message() -> EmailMessage {
        EmailMessage {
            from: EmailAddress::with_name("noreply@example.com", "Maze"),
            to: vec![EmailAddress::new("alice@example.com")],
            cc: vec![],
            bcc: vec![],
            reply_to: None,
            subject: "Hello".into(),
            body_text: "Body".into(),
            body_html: Some("<p>Body</p>".into()),
            headers: vec![],
            idempotency_key: None,
        }
    }

    fn provider_against(server_uri: &str) -> MailgunProvider {
        let cfg = MailgunConfig {
            domain: "mg.example.com".into(),
            api_key: "test-key".into(),
            region: MailgunRegion::Us,
            base_url_override: Some(server_uri.to_owned()),
        };
        MailgunProvider::with_http(cfg, reqwest::Client::new())
    }

    #[test]
    fn endpoint_url_uses_us_host_by_default() {
        let provider = MailgunProvider::with_http(
            MailgunConfig::new("mg.example.com", "key"),
            reqwest::Client::new(),
        );
        assert_eq!(
            provider.endpoint_url(),
            "https://api.mailgun.net/v3/mg.example.com/messages"
        );
    }

    #[test]
    fn endpoint_url_uses_eu_host_for_eu_region() {
        let provider = MailgunProvider::with_http(
            MailgunConfig::new("mg.example.com", "key").with_region(MailgunRegion::Eu),
            reqwest::Client::new(),
        );
        assert_eq!(
            provider.endpoint_url(),
            "https://api.eu.mailgun.net/v3/mg.example.com/messages"
        );
    }

    #[test]
    fn endpoint_url_respects_base_url_override() {
        let cfg = MailgunConfig {
            domain: "mg.example.com".into(),
            api_key: "key".into(),
            region: MailgunRegion::Us,
            base_url_override: Some("http://localhost:8080".into()),
        };
        let provider = MailgunProvider::with_http(cfg, reqwest::Client::new());
        assert_eq!(
            provider.endpoint_url(),
            "http://localhost:8080/v3/mg.example.com/messages"
        );
    }

    #[test]
    fn provider_name_is_mailgun() {
        let provider = MailgunProvider::with_http(
            MailgunConfig::new("mg.example.com", "key"),
            reqwest::Client::new(),
        );
        assert_eq!(provider.name(), "mailgun");
    }

    #[tokio::test]
    async fn send_email_happy_path() {
        let server = MockServer::start().await;
        // base64("api:test-key") == "YXBpOnRlc3Qta2V5"
        Mock::given(method("POST"))
            .and(path("/v3/mg.example.com/messages"))
            .and(header("authorization", "Basic YXBpOnRlc3Qta2V5"))
            .and(body_string_contains("from=Maze+%3Cnoreply%40example.com%3E"))
            .and(body_string_contains("to=alice%40example.com"))
            .and(body_string_contains("subject=Hello"))
            .and(body_string_contains("text=Body"))
            .and(body_string_contains("html=%3Cp%3EBody%3C%2Fp%3E"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "<20260504.abc@mg.example.com>",
                "message": "Queued. Thank you."
            })))
            .mount(&server)
            .await;

        let provider = provider_against(&server.uri());
        let receipt = provider.send_email(&sample_message()).await.expect("send");
        assert_eq!(receipt.provider, "mailgun");
        assert_eq!(
            receipt.provider_message_id.as_deref(),
            Some("<20260504.abc@mg.example.com>")
        );
    }

    #[tokio::test]
    async fn send_email_serialises_multiple_to_recipients() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/mg.example.com/messages"))
            .and(body_string_contains("to=alice%40example.com"))
            .and(body_string_contains("to=bob%40example.com"))
            .and(body_string_contains("to=carol%40example.com"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "<id>",
                "message": "Queued."
            })))
            .mount(&server)
            .await;

        let mut msg = sample_message();
        msg.to = vec![
            EmailAddress::new("alice@example.com"),
            EmailAddress::new("bob@example.com"),
            EmailAddress::new("carol@example.com"),
        ];

        let provider = provider_against(&server.uri());
        provider.send_email(&msg).await.expect("send");
    }

    #[tokio::test]
    async fn send_email_includes_cc_bcc_and_custom_headers() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/mg.example.com/messages"))
            .and(body_string_contains("cc=audit%40example.com"))
            .and(body_string_contains("bcc=archive%40example.com"))
            .and(body_string_contains("h%3AReply-To=support%40example.com"))
            .and(body_string_contains("h%3AX-Maze-Template=password_reset"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "<id>",
                "message": "Queued."
            })))
            .mount(&server)
            .await;

        let mut msg = sample_message();
        msg.cc = vec![EmailAddress::new("audit@example.com")];
        msg.bcc = vec![EmailAddress::new("archive@example.com")];
        msg.reply_to = Some(EmailAddress::new("support@example.com"));
        msg.headers = vec![("X-Maze-Template".into(), "password_reset".into())];

        let provider = provider_against(&server.uri());
        provider.send_email(&msg).await.expect("send");
    }

    #[tokio::test]
    async fn maps_4xx_to_permanent_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/mg.example.com/messages"))
            .respond_with(ResponseTemplate::new(400).set_body_string("invalid sender"))
            .mount(&server)
            .await;

        let provider = provider_against(&server.uri());
        let err = provider
            .send_email(&sample_message())
            .await
            .expect_err("must reject");
        assert!(!err.is_transient(), "expected permanent: {err:?}");
        match err {
            CommsError::ProviderHttp { status, body } => {
                assert_eq!(status, 400);
                assert!(body.contains("invalid sender"), "{body}");
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn maps_5xx_to_transient_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/mg.example.com/messages"))
            .respond_with(ResponseTemplate::new(502).set_body_string("Bad Gateway"))
            .mount(&server)
            .await;

        let provider = provider_against(&server.uri());
        let err = provider
            .send_email(&sample_message())
            .await
            .expect_err("must reject");
        assert!(err.is_transient(), "expected transient: {err:?}");
        match err {
            CommsError::ProviderHttp { status, .. } => assert_eq!(status, 502),
            other => panic!("unexpected variant: {other:?}"),
        }
    }
}
