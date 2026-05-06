use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use lettre::address::{Address, Envelope};
use lettre::message::header::ContentType;
use lettre::message::{Mailbox, Message, MultiPart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{AsyncSmtpTransport, AsyncTransport, Tokio1Executor};

use crate::email::{EmailAddress, EmailMessage, EmailProvider};
use crate::error::CommsError;
use crate::oauth::OAuthTokenSource;
use crate::provider::{DeliveryReceipt, Provider};
use crate::retry::RetryPolicy;

/// Transport-security mode for `SmtpOAuth2Provider`. The choice typically
/// follows the well-known port for the upstream SMTP service:
///
/// - `Implicit` — TLS from `connect()` onwards. Common on port 465.
/// - `StartTls` — plaintext connect, STARTTLS upgrade after EHLO. Common on
///   port 587 (M365, Workspace, most SMTP-AUTH submission endpoints).
/// - `Plain`    — no TLS, plaintext on the wire. Suitable only for local
///   tests against an in-process listener; never for production.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmtpTls {
    Implicit,
    StartTls,
    Plain,
}

/// Configuration for `SmtpOAuth2Provider`.
///
/// `host` and `port` identify the upstream SMTP service. `tls` selects the
/// transport-security mode (see [`SmtpTls`]). `username` is the SASL
/// identity presented during AUTH XOAUTH2 — for company-mailbox flows it is
/// the mailbox address being sent from. `token_source` mints fresh bearer
/// access tokens on demand; the provider re-asks on every send and trusts
/// the source's own caching to amortise OAuth round-trips.
pub struct SmtpOAuth2Config {
    pub host: String,
    pub port: u16,
    pub tls: SmtpTls,
    pub username: String,
    pub token_source: Arc<dyn OAuthTokenSource>,
}

/// `EmailProvider` that ships messages over SMTP, authenticating with
/// XOAUTH2 against any [`OAuthTokenSource`] (Microsoft 365 client-credentials,
/// Google Workspace service-account, etc.).
///
/// Cloning is intentionally not supported — wrap in `Arc` if multiple owners
/// need to share the provider.
pub struct SmtpOAuth2Provider {
    config: SmtpOAuth2Config,
    retry_policy: RetryPolicy,
}

impl SmtpOAuth2Provider {
    /// Build a provider from configuration. No network or token-mint calls
    /// happen here; both are deferred to the first `send_email`.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "provider-smtp-oauth2")] {
    /// use std::sync::Arc;
    /// use async_trait::async_trait;
    /// use comms::{CommsError, OAuthTokenSource, SmtpOAuth2Config, SmtpOAuth2Provider, SmtpTls};
    ///
    /// struct StubToken;
    /// #[async_trait]
    /// impl OAuthTokenSource for StubToken {
    ///     async fn access_token(&self) -> Result<String, CommsError> { Ok("stub".into()) }
    /// }
    ///
    /// let provider = SmtpOAuth2Provider::new(SmtpOAuth2Config {
    ///     host: "smtp.office365.com".into(),
    ///     port: 587,
    ///     tls: SmtpTls::StartTls,
    ///     username: "noreply@contoso.com".into(),
    ///     token_source: Arc::new(StubToken),
    /// })
    /// .expect("build provider");
    /// drop(provider);
    /// # }
    /// ```
    pub fn new(config: SmtpOAuth2Config) -> Result<Self, CommsError> {
        Ok(Self {
            config,
            retry_policy: RetryPolicy::default(),
        })
    }

    /// Builder-style setter for the retry policy applied to transient
    /// `send_email` failures (SMTP 4xx, connect timeouts).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "provider-smtp-oauth2")] {
    /// use std::sync::Arc;
    /// use async_trait::async_trait;
    /// use comms::{CommsError, OAuthTokenSource, RetryPolicy, SmtpOAuth2Config, SmtpOAuth2Provider, SmtpTls};
    ///
    /// struct StubToken;
    /// #[async_trait]
    /// impl OAuthTokenSource for StubToken {
    ///     async fn access_token(&self) -> Result<String, CommsError> { Ok("stub".into()) }
    /// }
    ///
    /// let provider = SmtpOAuth2Provider::new(SmtpOAuth2Config {
    ///     host: "smtp.example.com".into(),
    ///     port: 587,
    ///     tls: SmtpTls::StartTls,
    ///     username: "noreply@example.com".into(),
    ///     token_source: Arc::new(StubToken),
    /// })
    /// .expect("build provider")
    /// .with_retry_policy(RetryPolicy::no_retry());
    /// drop(provider);
    /// # }
    /// ```
    pub fn with_retry_policy(mut self, retry_policy: RetryPolicy) -> Self {
        self.retry_policy = retry_policy;
        self
    }

    fn build_transport(
        &self,
        access_token: &str,
    ) -> Result<AsyncSmtpTransport<Tokio1Executor>, CommsError> {
        let creds = Credentials::new(self.config.username.clone(), access_token.to_owned());
        let builder = match self.config.tls {
            SmtpTls::Implicit => AsyncSmtpTransport::<Tokio1Executor>::relay(&self.config.host)
                .map_err(|e| CommsError::Config(format!("smtp_oauth2: {e}")))?,
            SmtpTls::StartTls => {
                AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.config.host)
                    .map_err(|e| CommsError::Config(format!("smtp_oauth2: {e}")))?
            }
            SmtpTls::Plain => {
                AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&self.config.host)
            }
        };
        Ok(builder
            .port(self.config.port)
            .credentials(creds)
            .authentication(vec![Mechanism::Xoauth2])
            .build())
    }
}

impl Provider for SmtpOAuth2Provider {
    fn name(&self) -> &'static str {
        "smtp_oauth2"
    }

    fn retry_policy(&self) -> &RetryPolicy {
        &self.retry_policy
    }
}

#[async_trait]
impl EmailProvider for SmtpOAuth2Provider {
    async fn send_email(&self, msg: &EmailMessage) -> Result<DeliveryReceipt, CommsError> {
        let access_token = self.config.token_source.access_token().await?;
        let transport = self.build_transport(&access_token)?;
        let (envelope, body) = build_envelope_and_body(msg)?;
        transport
            .send_raw(&envelope, &body)
            .await
            .map_err(map_smtp_error)?;
        Ok(DeliveryReceipt {
            provider: "smtp_oauth2".into(),
            provider_message_id: None,
            accepted_at: Utc::now(),
        })
    }
}

fn email_to_address(addr: &EmailAddress) -> Result<Address, CommsError> {
    addr.address.parse::<Address>().map_err(|e| {
        CommsError::Provider(format!("smtp_oauth2: invalid address {}: {e}", addr.address))
    })
}

fn email_to_mailbox(addr: &EmailAddress) -> Result<Mailbox, CommsError> {
    let address = email_to_address(addr)?;
    Ok(Mailbox::new(addr.display_name.clone(), address))
}

fn build_envelope_and_body(msg: &EmailMessage) -> Result<(Envelope, Vec<u8>), CommsError> {
    // Envelope: MAIL FROM + every RCPT TO including bcc.
    let from_addr = email_to_address(&msg.from)?;
    let mut recipients: Vec<Address> = Vec::new();
    for r in &msg.to {
        recipients.push(email_to_address(r)?);
    }
    for r in &msg.cc {
        recipients.push(email_to_address(r)?);
    }
    for r in &msg.bcc {
        recipients.push(email_to_address(r)?);
    }
    let envelope = Envelope::new(Some(from_addr), recipients)
        .map_err(|e| CommsError::Provider(format!("smtp_oauth2: envelope: {e}")))?;

    // Standard headers via lettre's typed builder.
    let mut builder = Message::builder().from(email_to_mailbox(&msg.from)?);
    for r in &msg.to {
        builder = builder.to(email_to_mailbox(r)?);
    }
    for r in &msg.cc {
        builder = builder.cc(email_to_mailbox(r)?);
    }
    for r in &msg.bcc {
        builder = builder.bcc(email_to_mailbox(r)?);
    }
    if let Some(reply_to) = &msg.reply_to {
        builder = builder.reply_to(email_to_mailbox(reply_to)?);
    }
    builder = builder.subject(msg.subject.clone());

    let message = match &msg.body_html {
        Some(html) => builder
            .multipart(MultiPart::alternative_plain_html(
                msg.body_text.clone(),
                html.clone(),
            ))
            .map_err(|e| CommsError::Provider(format!("smtp_oauth2: build message: {e}")))?,
        None => builder
            .header(ContentType::TEXT_PLAIN)
            .body(msg.body_text.clone())
            .map_err(|e| CommsError::Provider(format!("smtp_oauth2: build message: {e}")))?,
    };

    let bytes = if msg.headers.is_empty() {
        message.formatted()
    } else {
        inject_headers(message, &msg.headers)
    };
    Ok((envelope, bytes))
}

/// Splice extra `Name: Value` headers into a formatted message before the
/// header/body boundary. Lettre's typed builder rejects header names not
/// known at compile time, so dynamic `(String, String)` pairs from
/// `EmailMessage::headers` are inserted here.
fn inject_headers(message: Message, headers: &[(String, String)]) -> Vec<u8> {
    let formatted = message.formatted();
    let needle = b"\r\n\r\n";
    let boundary = formatted
        .windows(needle.len())
        .position(|w| w == needle)
        .unwrap_or(formatted.len());
    let mut out = Vec::with_capacity(formatted.len() + 64 * headers.len());
    out.extend_from_slice(&formatted[..boundary]);
    for (name, value) in headers {
        out.extend_from_slice(b"\r\n");
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(b": ");
        out.extend_from_slice(value.as_bytes());
    }
    out.extend_from_slice(&formatted[boundary..]);
    out
}

fn map_smtp_error(e: lettre::transport::smtp::Error) -> CommsError {
    if e.is_permanent() {
        // SMTP 5yz — Permanent Negative Completion (incl. auth 535).
        CommsError::Provider(format!("smtp_oauth2: {e}"))
    } else {
        // SMTP 4yz, plus pre-response failures (TCP connect, TLS handshake,
        // IO, timeout) — all worth retrying under the configured policy.
        CommsError::Transient(format!("smtp_oauth2: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::OAuthTokenSource;
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
    use tokio::net::{TcpListener, TcpStream};
    use tokio::sync::Mutex;

    /// SASL XOAUTH2 wire format per the Google / Microsoft documentation:
    /// `base64( "user=" <user> 0x01 "auth=Bearer " <token> 0x01 0x01 )`.
    /// Used in tests to verify lettre's `Mechanism::Xoauth2` produces the
    /// bytes the resource servers expect.
    fn xoauth2_sasl_token(username: &str, access_token: &str) -> String {
        let mut blob = Vec::with_capacity(username.len() + access_token.len() + 21);
        blob.extend_from_slice(b"user=");
        blob.extend_from_slice(username.as_bytes());
        blob.push(0x01);
        blob.extend_from_slice(b"auth=Bearer ");
        blob.extend_from_slice(access_token.as_bytes());
        blob.push(0x01);
        blob.push(0x01);
        STANDARD.encode(blob)
    }

    struct StubToken(String);

    #[async_trait]
    impl OAuthTokenSource for StubToken {
        async fn access_token(&self) -> Result<String, CommsError> {
            Ok(self.0.clone())
        }
    }

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

    fn provider_against(host: &str, port: u16, token: &str) -> SmtpOAuth2Provider {
        SmtpOAuth2Provider::new(SmtpOAuth2Config {
            host: host.into(),
            port,
            tls: SmtpTls::Plain,
            username: "noreply@example.com".into(),
            token_source: Arc::new(StubToken(token.into())),
        })
        .expect("build provider")
        .with_retry_policy(RetryPolicy::no_retry())
    }

    // ───────────────────────────────────── helper unit tests

    #[test]
    fn xoauth2_sasl_token_matches_documented_wire_format() {
        let encoded = xoauth2_sasl_token("alice@example.com", "test-token");
        // Decode and compare bytes against the SASL spec.
        let decoded = STANDARD.decode(&encoded).expect("decode");
        let mut expected = Vec::new();
        expected.extend_from_slice(b"user=alice@example.com");
        expected.push(0x01);
        expected.extend_from_slice(b"auth=Bearer test-token");
        expected.push(0x01);
        expected.push(0x01);
        assert_eq!(decoded, expected);
        // And the literal base64 form for full belt-and-braces.
        assert_eq!(
            encoded,
            "dXNlcj1hbGljZUBleGFtcGxlLmNvbQFhdXRoPUJlYXJlciB0ZXN0LXRva2VuAQE="
        );
    }

    #[test]
    fn provider_name_is_smtp_oauth2() {
        let provider = provider_against("127.0.0.1", 25, "tok");
        assert_eq!(provider.name(), "smtp_oauth2");
    }

    // ───────────────────────────────────── MIME-shape tests on the formatted bytes

    #[test]
    fn build_envelope_and_body_serialises_standard_headers() {
        let (envelope, bytes) = build_envelope_and_body(&sample_message()).expect("build");
        let body = String::from_utf8(bytes).expect("utf8");
        assert_eq!(envelope.from().map(|a| a.to_string()), Some("noreply@example.com".into()));
        assert_eq!(envelope.to().len(), 1);
        assert!(body.contains("From: Maze <noreply@example.com>"), "got:\n{body}");
        assert!(body.contains("To: alice@example.com"), "got:\n{body}");
        assert!(body.contains("Subject: Hello"), "got:\n{body}");
        assert!(body.contains("Body"), "got:\n{body}");
        assert!(body.contains("<p>Body</p>"), "got:\n{body}");
    }

    #[test]
    fn build_envelope_and_body_includes_multiple_to_recipients_in_envelope() {
        let mut msg = sample_message();
        msg.to = vec![
            EmailAddress::new("alice@example.com"),
            EmailAddress::new("bob@example.com"),
            EmailAddress::new("carol@example.com"),
        ];
        let (envelope, bytes) = build_envelope_and_body(&msg).expect("build");
        let body = String::from_utf8(bytes).expect("utf8");
        let envelope_to: Vec<String> = envelope.to().iter().map(|a| a.to_string()).collect();
        assert_eq!(
            envelope_to,
            vec![
                "alice@example.com".to_string(),
                "bob@example.com".to_string(),
                "carol@example.com".to_string()
            ]
        );
        assert!(body.contains("alice@example.com"));
        assert!(body.contains("bob@example.com"));
        assert!(body.contains("carol@example.com"));
    }

    #[test]
    fn build_envelope_and_body_includes_cc_bcc_reply_to_and_custom_headers() {
        let mut msg = sample_message();
        msg.cc = vec![EmailAddress::new("audit@example.com")];
        msg.bcc = vec![EmailAddress::new("archive@example.com")];
        msg.reply_to = Some(EmailAddress::new("support@example.com"));
        msg.headers = vec![("X-Maze-Template".into(), "password_reset".into())];

        let (envelope, bytes) = build_envelope_and_body(&msg).expect("build");
        let body = String::from_utf8(bytes).expect("utf8");

        // Bcc must NOT appear in the rendered headers (privacy), but MUST appear in the SMTP envelope.
        assert!(
            !body.contains("Bcc:") && !body.to_lowercase().contains("archive@example.com"),
            "Bcc leaked into body:\n{body}"
        );
        let envelope_to: Vec<String> = envelope.to().iter().map(|a| a.to_string()).collect();
        assert!(envelope_to.contains(&"archive@example.com".to_string()));
        assert!(envelope_to.contains(&"audit@example.com".to_string()));

        assert!(body.contains("Cc: audit@example.com"), "got:\n{body}");
        assert!(body.contains("Reply-To: support@example.com"), "got:\n{body}");
        assert!(
            body.contains("X-Maze-Template: password_reset"),
            "got:\n{body}"
        );
    }

    // ───────────────────────────────────── In-process SMTP listener for wire-level tests

    /// Per-test SMTP scenario. Each entry is the response the listener will
    /// emit for the next inbound line that begins with `match_prefix`.
    /// The `auth_capture` slot stores the raw base64 blob from `AUTH XOAUTH2 …`
    /// so the test can decode it after the conversation finishes.
    #[derive(Default)]
    struct AuthCapture {
        bytes: Mutex<Option<Vec<u8>>>,
    }

    /// Behaviour knob for the scripted listener.
    #[derive(Clone, Copy)]
    enum Scenario {
        Happy,
        AuthFailure,
        TransientAfterMailFrom,
        PermanentAfterMailFrom,
    }

    async fn run_listener(
        listener: TcpListener,
        scenario: Scenario,
        capture: Arc<AuthCapture>,
    ) -> Result<(), std::io::Error> {
        let (socket, _) = listener.accept().await?;
        handle_smtp(socket, scenario, capture).await
    }

    async fn handle_smtp(
        socket: TcpStream,
        scenario: Scenario,
        capture: Arc<AuthCapture>,
    ) -> Result<(), std::io::Error> {
        let (read_half, mut write_half) = socket.into_split();
        let mut reader = BufReader::new(read_half);

        write_half.write_all(b"220 test ESMTP\r\n").await?;

        // Read EHLO
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        write_half
            .write_all(b"250-test\r\n250 AUTH PLAIN LOGIN XOAUTH2\r\n")
            .await?;

        // Read AUTH XOAUTH2 <base64>
        line.clear();
        reader.read_line(&mut line).await?;
        let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');
        if let Some(rest) = trimmed.strip_prefix("AUTH XOAUTH2 ") {
            let decoded = STANDARD.decode(rest.as_bytes()).unwrap_or_default();
            *capture.bytes.lock().await = Some(decoded);
        }
        match scenario {
            Scenario::AuthFailure => {
                write_half
                    .write_all(b"535 5.7.8 Authentication credentials invalid\r\n")
                    .await?;
                // Drain any further commands so lettre can close cleanly.
                let mut buf = [0u8; 256];
                let _ = reader.read(&mut buf).await;
                return Ok(());
            }
            _ => {
                write_half
                    .write_all(b"235 2.7.0 Authentication successful\r\n")
                    .await?;
            }
        }

        // Read MAIL FROM
        line.clear();
        reader.read_line(&mut line).await?;
        match scenario {
            Scenario::TransientAfterMailFrom => {
                write_half
                    .write_all(b"421 4.7.0 Service not available, try again\r\n")
                    .await?;
                let mut buf = [0u8; 256];
                let _ = reader.read(&mut buf).await;
                return Ok(());
            }
            Scenario::PermanentAfterMailFrom => {
                write_half
                    .write_all(b"550 5.7.1 Mailbox refused\r\n")
                    .await?;
                let mut buf = [0u8; 256];
                let _ = reader.read(&mut buf).await;
                return Ok(());
            }
            _ => {
                write_half.write_all(b"250 OK\r\n").await?;
            }
        }

        // RCPT TO (one per recipient — keep replying 250 OK until DATA appears).
        loop {
            line.clear();
            reader.read_line(&mut line).await?;
            let upper = line.trim().to_ascii_uppercase();
            if upper.starts_with("RCPT TO") {
                write_half.write_all(b"250 OK\r\n").await?;
            } else if upper.starts_with("DATA") {
                write_half.write_all(b"354 Go ahead\r\n").await?;
                break;
            } else {
                // Unexpected — close.
                return Ok(());
            }
        }

        // Read message bytes until line containing only "."
        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                return Ok(());
            }
            if line == ".\r\n" || line == ".\n" {
                break;
            }
        }
        write_half.write_all(b"250 OK\r\n").await?;

        // QUIT
        line.clear();
        reader.read_line(&mut line).await?;
        write_half.write_all(b"221 Bye\r\n").await?;

        Ok(())
    }

    async fn bound_listener() -> (TcpListener, u16) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let port = listener.local_addr().expect("addr").port();
        (listener, port)
    }

    #[tokio::test]
    async fn send_email_uses_xoauth2_with_documented_sasl_format() {
        let (listener, port) = bound_listener().await;
        let capture = Arc::new(AuthCapture::default());
        let server_capture = capture.clone();
        let server = tokio::spawn(async move {
            let _ = run_listener(listener, Scenario::Happy, server_capture).await;
        });

        let provider = provider_against("127.0.0.1", port, "test-token");
        let receipt = provider.send_email(&sample_message()).await.expect("send");
        assert_eq!(receipt.provider, "smtp_oauth2");
        assert!(receipt.provider_message_id.is_none());

        server.await.expect("server task");
        let captured = capture.bytes.lock().await.clone().expect("auth captured");
        let mut expected = Vec::new();
        expected.extend_from_slice(b"user=noreply@example.com");
        expected.push(0x01);
        expected.extend_from_slice(b"auth=Bearer test-token");
        expected.push(0x01);
        expected.push(0x01);
        assert_eq!(captured, expected);
    }

    #[tokio::test]
    async fn maps_smtp_5xx_auth_failure_to_permanent_error() {
        let (listener, port) = bound_listener().await;
        let capture = Arc::new(AuthCapture::default());
        let server_capture = capture.clone();
        let server = tokio::spawn(async move {
            let _ = run_listener(listener, Scenario::AuthFailure, server_capture).await;
        });

        let provider = provider_against("127.0.0.1", port, "test-token");
        let err = provider
            .send_email(&sample_message())
            .await
            .expect_err("must reject");
        assert!(!err.is_transient(), "expected permanent: {err:?}");
        match err {
            CommsError::Provider(s) => assert!(s.contains("535"), "{s}"),
            other => panic!("unexpected variant: {other:?}"),
        }
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn maps_smtp_4xx_to_transient_error() {
        let (listener, port) = bound_listener().await;
        let capture = Arc::new(AuthCapture::default());
        let server_capture = capture.clone();
        let server = tokio::spawn(async move {
            let _ = run_listener(listener, Scenario::TransientAfterMailFrom, server_capture).await;
        });

        let provider = provider_against("127.0.0.1", port, "test-token");
        let err = provider
            .send_email(&sample_message())
            .await
            .expect_err("must reject");
        assert!(err.is_transient(), "expected transient: {err:?}");
        match err {
            CommsError::Transient(s) => assert!(s.contains("421"), "{s}"),
            other => panic!("unexpected variant: {other:?}"),
        }
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn maps_smtp_5xx_after_mail_from_to_permanent_error() {
        let (listener, port) = bound_listener().await;
        let capture = Arc::new(AuthCapture::default());
        let server_capture = capture.clone();
        let server = tokio::spawn(async move {
            let _ = run_listener(listener, Scenario::PermanentAfterMailFrom, server_capture).await;
        });

        let provider = provider_against("127.0.0.1", port, "test-token");
        let err = provider
            .send_email(&sample_message())
            .await
            .expect_err("must reject");
        assert!(!err.is_transient(), "expected permanent: {err:?}");
        match err {
            CommsError::Provider(s) => assert!(s.contains("550"), "{s}"),
            other => panic!("unexpected variant: {other:?}"),
        }
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn connect_failure_maps_to_transient_error() {
        // Bind a port and immediately drop the listener so the slot is free
        // but nothing accepts. Whichever port we obtain is virtually
        // guaranteed to be unbound for the next millisecond.
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let port = listener.local_addr().expect("addr").port();
        drop(listener);

        let provider = provider_against("127.0.0.1", port, "test-token");
        let err = provider
            .send_email(&sample_message())
            .await
            .expect_err("must reject");
        assert!(err.is_transient(), "expected transient: {err:?}");
    }
}
