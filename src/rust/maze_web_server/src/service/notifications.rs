//! `Comms` factory. Builds a fully wired `Comms` instance from the
//! `[comms]` config section: embedded default templates and branding
//! partials are compiled in via `include_str!`, and the email provider is
//! selected per `enabled` and `email.provider`. When `enabled = false`
//! the factory always returns a `StubEmailProvider`, so a deployment can
//! omit provider credentials and still run cleanly.

use std::sync::Arc;
use std::time::Duration;

use comms::template::renderer::BrandingPartialSources;
use comms::{
    AppContext, BrandingContext, ClientCredentialsConfig, ClientCredentialsTokenSource, Comms,
    EmailAddress, EmailProvider, EmbeddedTemplateLoader, MailgunConfig, MailgunProvider,
    MailgunRegion, OAuthTokenSource, ServiceAccountConfig, ServiceAccountTokenSource,
    SmtpOAuth2Config, SmtpOAuth2Provider, SmtpTls, StubEmailProvider, TemplateLoader,
    TemplateRenderer,
};

use crate::config::comms::{
    CommsAppConfig, CommsEmailProvider, SmtpOauth2AppConfig, SmtpOauth2Vendor,
};

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
///   provider, `Mailgun` for the HTTP API, `SmtpOauth2` for SMTP+XOAUTH2
///   against either Microsoft 365 (client-credentials) or Google
///   Workspace (service-account).
///
/// Errors:
/// - empty `mailgun.api_key` when `provider = Mailgun` and `enabled = true`
///   (the api key is environment-only — see
///   `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY`).
/// - empty `mailgun.domain` when `provider = Mailgun` and `enabled = true`.
/// - unrecognised `mailgun.region` value (must be `"us"` or `"eu"`).
/// - empty `smtp_oauth2.host` / `username` when `provider = SmtpOauth2`
///   and `enabled = true`.
/// - unrecognised `smtp_oauth2.tls` value (must be `"starttls"`,
///   `"implicit"`, or `"plain"`).
/// - missing or unreadable Microsoft client_secret / Google
///   service-account JSON for the active `vendor`.
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
    // Resolution order for the `{{ app_name }}` template token:
    //   1. comms.branding.app_name  — explicit product name (preferred)
    //   2. comms.email.from_name    — From-header display name fallback
    //   3. comms.branding.company_name — final fallback
    // Lets operators set "Maze" for the in-body product name distinct
    // from "The Maze Team" for the From-header signature, while keeping
    // existing configs (which only set from_name) working unchanged.
    let app_name = if !cfg.branding.app_name.is_empty() {
        cfg.branding.app_name.clone()
    } else if !cfg.email.from_name.is_empty() {
        cfg.email.from_name.clone()
    } else {
        cfg.branding.company_name.clone()
    };
    let app = AppContext {
        app_name,
        from_name: cfg.email.from_name.clone(),
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

/// Build the From `EmailAddress` from `cfg.email.from` and
/// `cfg.email.from_name`. Returns `None` if `from` is empty (operator
/// hasn't configured a sender), in which case `Comms::send_template`
/// will surface a `Config` error at first call.
pub fn build_default_from(cfg: &CommsAppConfig) -> Option<EmailAddress> {
    if cfg.email.from.is_empty() {
        None
    } else if cfg.email.from_name.is_empty() {
        Some(EmailAddress::new(cfg.email.from.clone()))
    } else {
        Some(EmailAddress::with_name(
            cfg.email.from.clone(),
            cfg.email.from_name.clone(),
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
        CommsEmailProvider::SmtpOauth2 => {
            let smtp_cfg = &cfg.email.smtp_oauth2;
            if smtp_cfg.host.is_empty() {
                return Err("[comms.email.smtp_oauth2] host is empty".to_string());
            }
            if smtp_cfg.username.is_empty() {
                return Err("[comms.email.smtp_oauth2] username is empty".to_string());
            }
            let tls = match smtp_cfg.tls.as_str() {
                "starttls" => SmtpTls::StartTls,
                "implicit" => SmtpTls::Implicit,
                "plain" => SmtpTls::Plain,
                other => {
                    return Err(format!(
                        "[comms.email.smtp_oauth2] tls '{other}' is invalid (use 'starttls', 'implicit', or 'plain')"
                    ));
                }
            };
            let token_source = build_smtp_oauth2_token_source(smtp_cfg)?;
            let provider = SmtpOAuth2Provider::new(SmtpOAuth2Config {
                host: smtp_cfg.host.clone(),
                port: smtp_cfg.port,
                tls,
                username: smtp_cfg.username.clone(),
                token_source,
            })
            .map_err(|e| format!("smtp_oauth2: {e}"))?;
            Ok(Some(Arc::new(provider)))
        }
    }
}

/// Construct the OAuth token source matching `vendor`. Microsoft
/// `client_credentials` builds a `ClientCredentialsTokenSource` from the
/// resolved tenant/client/secret triple; Google `service_account` reads
/// the JSON key file from disk and builds a `ServiceAccountTokenSource`,
/// optionally setting the JWT `sub` claim from `delegated_subject`.
fn build_smtp_oauth2_token_source(
    smtp_cfg: &SmtpOauth2AppConfig,
) -> Result<Arc<dyn OAuthTokenSource>, String> {
    match smtp_cfg.vendor {
        SmtpOauth2Vendor::Microsoft => {
            let m = &smtp_cfg.microsoft;
            if m.tenant_id.is_empty() {
                return Err("[comms.email.smtp_oauth2.microsoft] tenant_id is empty".to_string());
            }
            if m.client_id.is_empty() {
                return Err("[comms.email.smtp_oauth2.microsoft] client_id is empty".to_string());
            }
            if m.client_secret.is_empty() {
                return Err(
                    "[comms.email.smtp_oauth2.microsoft] client_secret is empty (set MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_SECRET)"
                        .to_string(),
                );
            }
            let scope = if m.scopes.is_empty() {
                "https://outlook.office.com/SMTP.Send".to_string()
            } else {
                m.scopes.join(" ")
            };
            let cc_cfg = ClientCredentialsConfig {
                tenant_id: m.tenant_id.clone(),
                client_id: m.client_id.clone(),
                client_secret: m.client_secret.clone(),
                scope,
                token_endpoint_url: None,
                refresh_skew: Duration::from_secs(60),
            };
            let source = ClientCredentialsTokenSource::new(cc_cfg)
                .map_err(|e| format!("smtp_oauth2 microsoft token source: {e}"))?;
            Ok(Arc::new(source))
        }
        SmtpOauth2Vendor::Google => {
            let g = &smtp_cfg.google;
            if g.service_account_json_path.is_empty() {
                return Err(
                    "[comms.email.smtp_oauth2.google] service_account_json_path is empty"
                        .to_string(),
                );
            }
            let json = std::fs::read_to_string(&g.service_account_json_path).map_err(|e| {
                format!(
                    "[comms.email.smtp_oauth2.google] read service_account_json_path '{}': {e}",
                    g.service_account_json_path
                )
            })?;
            let scopes = if g.scopes.is_empty() {
                vec!["https://www.googleapis.com/auth/gmail.send".to_string()]
            } else {
                g.scopes.clone()
            };
            let mut sa_cfg = ServiceAccountConfig::from_json_str(&json, scopes)
                .map_err(|e| format!("smtp_oauth2 google service account: {e}"))?;
            if !g.delegated_subject.is_empty() {
                sa_cfg.subject = Some(g.delegated_subject.clone());
            }
            let source = ServiceAccountTokenSource::new(sa_cfg)
                .map_err(|e| format!("smtp_oauth2 google token source: {e}"))?;
            Ok(Arc::new(source))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::comms::{
        CommsAppConfig, CommsBrandingConfig, CommsEmailAuditConfig, CommsEmailConfig,
        CommsEmailProvider, MailgunAppConfig, SmtpOauth2AppConfig, SmtpOauth2Vendor,
        SmtpOauth2GoogleConfig, SmtpOauth2MicrosoftConfig,
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
                app_name: String::new(),
            },
            email: CommsEmailConfig {
                provider: CommsEmailProvider::Mailgun,
                from: "noreply@example.com".into(),
                from_name: "The Maze Team".into(),
                templates_dir: "config/email_templates".into(),
                mailgun: MailgunAppConfig {
                    domain: "mg.example.com".into(),
                    region: "us".into(),
                    api_key: "test-resolved-api-key".into(),
                },
                smtp_oauth2: SmtpOauth2AppConfig::default(),
                audit: CommsEmailAuditConfig::default(),
            },
        }
    }

    fn enabled_smtp_oauth2_microsoft_config_with_secret() -> CommsAppConfig {
        CommsAppConfig {
            enabled: true,
            public_base_url: "https://maze.example.com".into(),
            branding: CommsBrandingConfig {
                company_name: "Maze, Inc.".into(),
                company_address: "123 Example St".into(),
                company_url: "https://maze.example.com".into(),
                logo_url: "https://maze.example.com/static/logo.png".into(),
                app_name: String::new(),
            },
            email: CommsEmailConfig {
                provider: CommsEmailProvider::SmtpOauth2,
                from: "noreply@contoso.com".into(),
                from_name: "The Maze Team".into(),
                templates_dir: "config/email_templates".into(),
                mailgun: MailgunAppConfig::default(),
                smtp_oauth2: SmtpOauth2AppConfig {
                    host: "smtp.office365.com".into(),
                    port: 587,
                    tls: "starttls".into(),
                    username: "noreply@contoso.com".into(),
                    vendor: SmtpOauth2Vendor::Microsoft,
                    microsoft: SmtpOauth2MicrosoftConfig {
                        tenant_id: "00000000-0000-0000-0000-000000000000".into(),
                        client_id: "11111111-1111-1111-1111-111111111111".into(),
                        scopes: vec!["https://outlook.office.com/SMTP.Send".into()],
                        client_secret: "test-resolved-secret".into(),
                    },
                    google: SmtpOauth2GoogleConfig::default(),
                },
                audit: CommsEmailAuditConfig::default(),
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

    // ───────────────────────────────────── SMTP+XOAUTH2 wiring tests

    #[test]
    fn build_comms_with_smtp_oauth2_microsoft_succeeds() {
        let cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        let _ = build_comms(&cfg).expect("build with smtp+oauth2 microsoft");
    }

    #[test]
    fn build_comms_with_smtp_oauth2_missing_host_returns_error() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        cfg.email.smtp_oauth2.host = String::new();
        let err = build_comms(&cfg).err().expect("missing host must reject");
        assert!(err.contains("host"), "{err}");
    }

    #[test]
    fn build_comms_with_smtp_oauth2_missing_username_returns_error() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        cfg.email.smtp_oauth2.username = String::new();
        let err = build_comms(&cfg).err().expect("missing username must reject");
        assert!(err.contains("username"), "{err}");
    }

    #[test]
    fn build_comms_with_smtp_oauth2_invalid_tls_returns_error() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        cfg.email.smtp_oauth2.tls = "wat".into();
        let err = build_comms(&cfg).err().expect("invalid tls must reject");
        assert!(err.contains("tls"), "{err}");
    }

    #[test]
    fn build_comms_accepts_implicit_tls_for_smtp_oauth2() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        cfg.email.smtp_oauth2.tls = "implicit".into();
        cfg.email.smtp_oauth2.port = 465;
        let _ = build_comms(&cfg).expect("implicit tls must build");
    }

    #[test]
    fn build_comms_with_smtp_oauth2_microsoft_missing_tenant_returns_error() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        cfg.email.smtp_oauth2.microsoft.tenant_id = String::new();
        let err = build_comms(&cfg).err().expect("missing tenant must reject");
        assert!(err.contains("tenant_id"), "{err}");
    }

    #[test]
    fn build_comms_with_smtp_oauth2_microsoft_missing_client_id_returns_error() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        cfg.email.smtp_oauth2.microsoft.client_id = String::new();
        let err = build_comms(&cfg).err().expect("missing client_id must reject");
        assert!(err.contains("client_id"), "{err}");
    }

    #[test]
    fn build_comms_with_smtp_oauth2_microsoft_missing_client_secret_returns_error() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        cfg.email.smtp_oauth2.microsoft.client_secret = String::new();
        let err = build_comms(&cfg).err().expect("missing client_secret must reject");
        assert!(err.contains("client_secret"), "{err}");
    }

    #[test]
    fn build_comms_with_smtp_oauth2_google_missing_path_returns_error() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        cfg.email.smtp_oauth2.vendor = SmtpOauth2Vendor::Google;
        // service_account_json_path defaults to empty in the Microsoft helper.
        let err = build_comms(&cfg).err().expect("missing path must reject");
        assert!(err.contains("service_account_json_path"), "{err}");
    }

    #[test]
    fn build_comms_with_smtp_oauth2_google_unreadable_path_returns_error() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config_with_secret();
        cfg.email.smtp_oauth2.vendor = SmtpOauth2Vendor::Google;
        cfg.email.smtp_oauth2.google.service_account_json_path =
            "/this/path/does/not/exist/sa.json".into();
        let err = build_comms(&cfg).err().expect("unreadable path must reject");
        // Either the path appears or the OS error word does — both acceptable.
        assert!(
            err.contains("service_account_json_path") || err.to_lowercase().contains("read"),
            "{err}"
        );
    }
}
