//! End-to-end smoke test for the `Comms` pipeline. Exercises the embedded
//! default templates and branding partials through a `StubEmailProvider`
//! and asserts the captured `EmailMessage` matches the rendered template
//! plus the supplied per-message context.

use std::sync::Arc;

use comms::{Comms, EmailAddress, EmailProvider, StubEmailProvider};
use maze_web_server::config::comms::{
    CommsAppConfig, CommsBrandingConfig, CommsEmailAuditConfig, CommsEmailConfig,
    CommsEmailProvider, MailgunAppConfig, SmtpOauth2AppConfig,
};
use maze_web_server::service::notifications::{
    build_comms, build_default_from, build_renderer,
};
use serde_json::json;

/// A populated config that exercises every branding token used by the
/// embedded templates and partials (so missing-field renders surface as
/// failures here, not at first deployment).
fn populated_config() -> CommsAppConfig {
    CommsAppConfig {
        enabled: false, // stub provider regardless of email.provider
        public_base_url: "https://maze.example.com".into(),
        branding: CommsBrandingConfig {
            company_name: "Maze, Inc.".into(),
            company_address: "123 Example St, City, Country".into(),
            company_url: "https://maze.example.com".into(),
            logo_url: "https://maze.example.com/static/logo.png".into(),
            app_name: String::new(),
        },
        email: CommsEmailConfig {
            provider: CommsEmailProvider::Stub,
            from: "noreply@maze.example.com".into(),
            from_name: "The Maze Team".into(),
            templates_dir: "config/email_templates".into(),
            mailgun: MailgunAppConfig::default(),
            smtp_oauth2: SmtpOauth2AppConfig::default(),
            audit: CommsEmailAuditConfig::default(),
        },
    }
}

#[tokio::test]
async fn password_reset_template_renders_subject_text_and_html_with_branding_partials() {
    let cfg = populated_config();

    // Build the renderer the same way production does, but inject our own
    // `StubEmailProvider` so we can inspect captured messages directly.
    let renderer = build_renderer(&cfg).expect("renderer must build");
    let stub: Arc<StubEmailProvider> = Arc::new(StubEmailProvider::new());
    let stub_dyn: Arc<dyn EmailProvider> = stub.clone();
    let comms = Comms::new(renderer, Some(stub_dyn), build_default_from(&cfg));

    // Render the embedded `password_reset` template against a synthesised
    // user + reset-link context.
    let ctx = json!({
        "first_name": "Alice",
        "reset_link": "https://maze.example.com/reset?token=test-token-123",
    });
    let receipt = comms
        .send_template(
            "password_reset",
            EmailAddress::new("alice@example.com"),
            &ctx,
        )
        .await
        .expect("send_template must succeed");

    // The stub identifies itself in the receipt.
    assert_eq!(receipt.provider, "stub_email");

    let captured = stub.last().expect("stub must have captured the message");

    // Recipient + sender plumbed through correctly.
    assert_eq!(captured.to.len(), 1);
    assert_eq!(captured.to[0].address, "alice@example.com");
    assert_eq!(captured.from.address, "noreply@maze.example.com");
    assert_eq!(captured.from.display_name.as_deref(), Some("The Maze Team"));

    // Subject substitutes `{{ app_name }}` (resolves to
    // `branding.app_name` if set, else `email.from_name`, else
    // `branding.company_name` — here from_name is "The Maze Team" and
    // app_name/company_name aren't set, so subject reads "The Maze Team".)
    assert_eq!(captured.subject, "Reset your The Maze Team password");

    // Plain-text body substitutes `{{ first_name }}` and `{{ reset_link }}`,
    // and contains the rendered `{{ header }}` / `{{ footer }}` partials.
    assert!(
        captured.body_text.contains("Hi Alice,"),
        "body_text missing first_name substitution: {}",
        captured.body_text
    );
    assert!(
        captured
            .body_text
            .contains("https://maze.example.com/reset?token=test-token-123"),
        "body_text missing reset_link substitution: {}",
        captured.body_text
    );
    // header.text partial is `== {{ company_name }} ==` → substitutes the
    // company name; footer.text contains "(c) <year> <company_name>".
    assert!(
        captured.body_text.contains("== Maze, Inc. =="),
        "body_text missing rendered header partial: {}",
        captured.body_text
    );
    assert!(
        captured.body_text.contains("Maze, Inc."),
        "body_text missing rendered footer partial: {}",
        captured.body_text
    );

    // HTML body present and contains the rendered partials + per-message vars.
    let html = captured
        .body_html
        .as_ref()
        .expect("password_reset template carries an html section");
    assert!(
        html.contains("Hi Alice,"),
        "body_html missing first_name substitution: {html}"
    );
    // header.html contains `<img ... alt="{{ company_name }}" ...>`.
    assert!(
        html.contains("alt=\"Maze, Inc.\""),
        "body_html missing rendered header partial: {html}"
    );
    // footer.html contains `&copy; <year> <company_name>`.
    assert!(
        html.contains("&copy;"),
        "body_html missing rendered footer copyright: {html}"
    );
    assert!(
        html.contains("Maze, Inc."),
        "body_html missing company_name in footer: {html}"
    );
}

/// `build_comms` with `enabled = false` short-circuits the provider
/// selection to the in-memory stub regardless of `email.provider` — even
/// when `mailgun` settings are absent. The smoke test verifies the round
/// trip via the resulting `DeliveryReceipt` rather than inspecting the
/// hidden internal stub: `provider == "stub_email"` is unique to the stub.
#[tokio::test]
async fn build_comms_disabled_falls_back_to_stub_provider_end_to_end() {
    let mut cfg = populated_config();
    cfg.enabled = false;
    // Operator declared mailgun but didn't supply any credentials. With
    // enabled = false this is fine — stub is used.
    cfg.email.provider = CommsEmailProvider::Mailgun;
    cfg.email.mailgun = MailgunAppConfig::default(); // empty domain + api_key

    let comms = build_comms(&cfg).expect("disabled comms must build");
    let ctx = json!({
        "first_name": "Bob",
        "reset_link": "https://maze.example.com/reset?token=x",
    });
    let receipt = comms
        .send_template(
            "password_reset",
            EmailAddress::new("bob@example.com"),
            &ctx,
        )
        .await
        .expect("send must succeed when stub is wired");
    assert_eq!(receipt.provider, "stub_email");
}

/// Optional sandbox test: actually delivers a message through the Mailgun
/// sandbox API when the relevant env vars are set. Skipped by default
/// (`#[ignore]`) so the suite stays hermetic; run with
/// `cargo test -p maze_web_server -- --ignored mailgun_sandbox`.
///
/// Required env vars:
/// - `MAILGUN_SANDBOX_DOMAIN` — the sandbox domain (e.g.
///   `sandboxXXXXXXXXX.mailgun.org`).
/// - `MAILGUN_SANDBOX_API_KEY` — the private API key.
/// - `MAILGUN_SANDBOX_TO` — a verified recipient on the sandbox.
#[tokio::test]
#[ignore]
async fn mailgun_sandbox_delivers_when_env_vars_set() {
    let domain = match std::env::var("MAILGUN_SANDBOX_DOMAIN") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!("skipping mailgun_sandbox: MAILGUN_SANDBOX_DOMAIN unset/empty");
            return;
        }
    };
    let api_key = match std::env::var("MAILGUN_SANDBOX_API_KEY") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!("skipping mailgun_sandbox: MAILGUN_SANDBOX_API_KEY unset/empty");
            return;
        }
    };
    let to = match std::env::var("MAILGUN_SANDBOX_TO") {
        Ok(v) if !v.is_empty() => v,
        _ => {
            eprintln!("skipping mailgun_sandbox: MAILGUN_SANDBOX_TO unset/empty");
            return;
        }
    };

    let mut cfg = populated_config();
    cfg.enabled = true;
    cfg.email.provider = CommsEmailProvider::Mailgun;
    cfg.email.mailgun = MailgunAppConfig {
        domain,
        region: "us".into(),
        api_key,
    };
    // Align From-header with the sandbox domain so the message is
    // SPF/DKIM/DMARC-aligned and EOP / Gmail don't silently drop it as
    // high-confidence phish. The default in `populated_config()` is
    // `noreply@maze.example.com` — RFC 2606 reserved, no records, fails
    // every alignment check. Mirror the production env-var override
    // (`MAZE_WEB_SERVER_COMMS_EMAIL_DEFAULT_FROM`) when it's set,
    // otherwise derive `noreply@<sandbox-domain>` so the test produces a
    // sendable message even without an explicit env var.
    cfg.email.from = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_FROM")
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| format!("noreply@{}", cfg.email.mailgun.domain));

    let comms = build_comms(&cfg).expect("mailgun comms must build with credentials");
    let ctx = json!({
        "first_name": "Sandbox",
        "reset_link": "https://maze.example.com/reset?token=sandbox-test",
    });
    let receipt = comms
        .send_template("password_reset", EmailAddress::new(&to), &ctx)
        .await
        .expect("Mailgun sandbox must accept the send");
    assert_eq!(receipt.provider, "mailgun");
    assert!(
        receipt.provider_message_id.is_some(),
        "Mailgun should return a provider message id"
    );
}
