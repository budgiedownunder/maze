//! `Comms` factory. Builds a fully wired `Comms` instance from the
//! `[comms]` config section: embedded default templates and branding
//! partials are compiled in via `include_str!`, and the email provider is
//! selected per `enabled` and `email.provider`. When `enabled = false`
//! the factory always returns a `StubEmailProvider`, so a deployment can
//! omit provider credentials and still run cleanly.

use std::sync::Arc;

use comms::template::renderer::BrandingPartialSources;
use comms::{
    AppContext, BrandingContext, Comms, EmailAddress, EmailProvider, EmbeddedTemplateLoader,
    MailgunConfig, MailgunProvider, MailgunRegion, StubEmailProvider, TemplateLoader,
    TemplateRenderer,
};

use crate::config::comms::{CommsAppConfig, CommsEmailProvider};

const PASSWORD_RESET_TOML: &str =
    include_str!("../../templates/email/password_reset.toml");
const EMAIL_VERIFICATION_TOML: &str =
    include_str!("../../templates/email/email_verification.toml");

const LOGO_HTML: &str = include_str!("../../templates/email/partials/logo.html");
const LOGO_TEXT: &str = include_str!("../../templates/email/partials/logo.text");
const HEADER_HTML: &str = include_str!("../../templates/email/partials/header.html");
const HEADER_TEXT: &str = include_str!("../../templates/email/partials/header.text");
const FOOTER_HTML: &str = include_str!("../../templates/email/partials/footer.html");
const FOOTER_TEXT: &str = include_str!("../../templates/email/partials/footer.text");

/// Build a `Comms` instance from the supplied configuration.
///
/// Provider selection:
/// - `enabled = false` always returns a `StubEmailProvider` (in-memory
///   capture). Operators can run the server without provider credentials.
/// - `enabled = true` honours `email.provider`: `Stub` for the in-memory
///   provider, `Mailgun` for the HTTP API.
///
/// Errors:
/// - empty `mailgun.api_key` when `provider = Mailgun` and `enabled = true`
///   (the api key is environment-only — see
///   `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY`).
/// - empty `mailgun.domain` when `provider = Mailgun` and `enabled = true`.
/// - unrecognised `mailgun.region` value (must be `"us"` or `"eu"`).
/// - template-renderer construction failures (parse errors in the embedded
///   templates or partial-rendering errors against the supplied branding).
pub fn build_comms(cfg: &CommsAppConfig) -> Result<Comms, String> {
    let renderer = build_renderer(cfg)?;
    let email = build_email_provider(cfg)?;
    let default_from = build_default_from(cfg);
    Ok(Comms::new(renderer, email, default_from))
}

/// Build the templated-message renderer from the supplied config: embedded
/// default templates compiled in via `include_str!`, plus the branding
/// partials pre-rendered against `cfg.branding`.
///
/// Exposed publicly so integration tests can construct a `Comms` with their
/// own `StubEmailProvider` (held by the test for capture inspection) without
/// having to duplicate the embedded-templates wiring.
pub fn build_renderer(cfg: &CommsAppConfig) -> Result<TemplateRenderer, String> {
    let app_name = if cfg.email.default_from_name.is_empty() {
        cfg.branding.company_name.clone()
    } else {
        cfg.email.default_from_name.clone()
    };
    let app = AppContext {
        app_name,
        server_url: cfg.public_base_url.clone(),
        branding: BrandingContext {
            company_name: cfg.branding.company_name.clone(),
            company_address: cfg.branding.company_address.clone(),
            company_url: cfg.branding.company_url.clone(),
            logo_url: cfg.branding.logo_url.clone(),
        },
    };

    let templates: Arc<dyn TemplateLoader> = Arc::new(EmbeddedTemplateLoader::from_pairs(&[
        ("password_reset", PASSWORD_RESET_TOML),
        ("email_verification", EMAIL_VERIFICATION_TOML),
    ]));

    let partials = BrandingPartialSources {
        logo_html: LOGO_HTML.to_string(),
        logo_text: LOGO_TEXT.to_string(),
        header_html: HEADER_HTML.to_string(),
        header_text: HEADER_TEXT.to_string(),
        footer_html: FOOTER_HTML.to_string(),
        footer_text: FOOTER_TEXT.to_string(),
    };

    TemplateRenderer::new(app, templates, partials)
        .map_err(|e| format!("comms template renderer: {e}"))
}

/// Build the default `from` `EmailAddress` from `cfg.email.default_from`
/// and `cfg.email.default_from_name`. Returns `None` if `default_from` is
/// empty (operator hasn't configured a sender), in which case
/// `Comms::send_template` will surface a `Config` error at first call.
pub fn build_default_from(cfg: &CommsAppConfig) -> Option<EmailAddress> {
    if cfg.email.default_from.is_empty() {
        None
    } else if cfg.email.default_from_name.is_empty() {
        Some(EmailAddress::new(cfg.email.default_from.clone()))
    } else {
        Some(EmailAddress::with_name(
            cfg.email.default_from.clone(),
            cfg.email.default_from_name.clone(),
        ))
    }
}

fn build_email_provider(
    cfg: &CommsAppConfig,
) -> Result<Option<Arc<dyn EmailProvider>>, String> {
    if !cfg.enabled {
        return Ok(Some(Arc::new(StubEmailProvider::new())));
    }
    match cfg.email.provider {
        CommsEmailProvider::Stub => Ok(Some(Arc::new(StubEmailProvider::new()))),
        CommsEmailProvider::Mailgun => {
            if cfg.email.mailgun.api_key.is_empty() {
                return Err(
                    "[comms.email.mailgun] api_key is empty (set MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY)"
                        .to_string(),
                );
            }
            if cfg.email.mailgun.domain.is_empty() {
                return Err("[comms.email.mailgun] domain is empty".to_string());
            }
            let region = match cfg.email.mailgun.region.as_str() {
                "us" => MailgunRegion::Us,
                "eu" => MailgunRegion::Eu,
                other => {
                    return Err(format!(
                        "[comms.email.mailgun] region '{other}' is invalid (use 'us' or 'eu')"
                    ));
                }
            };
            let mailgun_cfg = MailgunConfig {
                domain: cfg.email.mailgun.domain.clone(),
                api_key: cfg.email.mailgun.api_key.clone(),
                region,
                base_url_override: None,
            };
            let provider =
                MailgunProvider::new(mailgun_cfg).map_err(|e| format!("mailgun: {e}"))?;
            Ok(Some(Arc::new(provider)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::comms::{
        CommsAppConfig, CommsBrandingConfig, CommsEmailConfig, CommsEmailProvider,
        MailgunAppConfig,
    };

    fn disabled_config() -> CommsAppConfig {
        CommsAppConfig {
            enabled: false,
            ..CommsAppConfig::default()
        }
    }

    fn enabled_mailgun_config_with_api_key() -> CommsAppConfig {
        CommsAppConfig {
            enabled: true,
            public_base_url: "https://maze.example.com".into(),
            branding: CommsBrandingConfig {
                company_name: "Maze, Inc.".into(),
                company_address: "123 Example St".into(),
                company_url: "https://maze.example.com".into(),
                logo_url: "https://maze.example.com/static/logo.png".into(),
            },
            email: CommsEmailConfig {
                provider: CommsEmailProvider::Mailgun,
                default_from: "noreply@example.com".into(),
                default_from_name: "The Maze Team".into(),
                templates_dir: "config/email_templates".into(),
                mailgun: MailgunAppConfig {
                    domain: "mg.example.com".into(),
                    region: "us".into(),
                    api_key: "test-resolved-api-key".into(),
                },
            },
        }
    }

    #[test]
    fn build_comms_with_enabled_false_succeeds() {
        let cfg = disabled_config();
        let _ = build_comms(&cfg).expect("build with stub provider");
    }

    #[test]
    fn build_comms_with_mailgun_and_api_key_succeeds() {
        let cfg = enabled_mailgun_config_with_api_key();
        let _ = build_comms(&cfg).expect("build with mailgun provider");
    }

    #[test]
    fn build_comms_with_enabled_true_and_missing_api_key_returns_error() {
        let mut cfg = enabled_mailgun_config_with_api_key();
        cfg.email.mailgun.api_key = String::new();
        match build_comms(&cfg) {
            Err(msg) => assert!(msg.contains("api_key"), "got: {msg}"),
            Ok(_) => panic!("expected Err for empty api_key"),
        }
    }

    #[test]
    fn build_comms_with_enabled_true_and_missing_domain_returns_error() {
        let mut cfg = enabled_mailgun_config_with_api_key();
        cfg.email.mailgun.domain = String::new();
        match build_comms(&cfg) {
            Err(msg) => assert!(msg.contains("domain"), "got: {msg}"),
            Ok(_) => panic!("expected Err for empty domain"),
        }
    }

    #[test]
    fn build_comms_with_invalid_region_returns_error() {
        let mut cfg = enabled_mailgun_config_with_api_key();
        cfg.email.mailgun.region = "asia".into();
        match build_comms(&cfg) {
            Err(msg) => assert!(msg.contains("region"), "got: {msg}"),
            Ok(_) => panic!("expected Err for invalid region"),
        }
    }

    #[test]
    fn build_comms_accepts_eu_region() {
        let mut cfg = enabled_mailgun_config_with_api_key();
        cfg.email.mailgun.region = "eu".into();
        let _ = build_comms(&cfg).expect("build with eu region");
    }

    #[test]
    fn build_comms_with_enabled_true_and_stub_provider_succeeds() {
        let mut cfg = enabled_mailgun_config_with_api_key();
        cfg.email.provider = CommsEmailProvider::Stub;
        let _ = build_comms(&cfg).expect("build with explicit stub provider");
    }

    /// `TemplateRenderer::new` parses every embedded template and pre-renders
    /// each branding partial against the supplied branding context. A
    /// successful `build_comms` call therefore implies the embedded defaults
    /// are syntactically valid and reference no unknown tokens.
    #[test]
    fn build_comms_pre_renders_embedded_templates_and_partials() {
        let cfg = enabled_mailgun_config_with_api_key();
        let _ = build_comms(&cfg).expect("templates and partials must build cleanly");
    }
}
