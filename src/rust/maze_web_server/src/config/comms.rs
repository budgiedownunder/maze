//! `[comms]` configuration: types, validation, and env-override wiring
//! consumed by `AppConfig::load`.

use config::{ConfigBuilder, builder::DefaultState};
use serde::{Deserialize, Serialize};

/// Top-level `[comms]` configuration block.
///
/// Composed onto `AppConfig`. When `enabled = false` the server uses an
/// in-memory stub so downstream call sites can keep calling `send_*` without
/// contacting any provider — useful for dev and CI. When `enabled = true`,
/// the active provider is selected by `email.provider` and its sub-table is
/// consulted; secrets (api keys etc.) are read from the environment, never
/// from this config.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommsAppConfig {
    /// Master switch. When false, no provider is contacted; sends are
    /// captured by an in-memory stub and logged. Useful for dev and CI.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_ENABLED`.
    #[serde(default = "default_comms_enabled")]
    pub enabled: bool,

    /// Directory holding filesystem-override templates and partials.
    /// Templates here override embedded defaults; templates not present
    /// here fall back to the embedded copies.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_TEMPLATES_DIR`.
    #[serde(default = "default_comms_templates_dir")]
    pub templates_dir: String,

    /// Public base URL the server is reachable at, used to build links
    /// inside templates (e.g. `{{ reset_link }}`, `{{ verification_link }}`).
    /// Required when `enabled = true`.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_PUBLIC_BASE_URL`.
    #[serde(default)]
    pub public_base_url: String,

    /// Default from-address used when `send_template` synthesises an
    /// email message. Required when `enabled = true`.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_DEFAULT_FROM_EMAIL`.
    #[serde(default)]
    pub default_from_email: String,

    /// Default display name paired with `default_from_email`.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_DEFAULT_FROM_NAME`.
    #[serde(default)]
    pub default_from_name: String,

    /// Branding values fed into the partial templates (`{{ logo }}`,
    /// `{{ header }}`, `{{ footer }}`).
    #[serde(default)]
    pub branding: CommsBrandingConfig,

    /// Email-medium configuration: provider discriminator + per-provider
    /// sub-tables. Only the section matching `provider` is consulted at
    /// dispatch time.
    #[serde(default)]
    pub email: CommsEmailConfig,
}

impl Default for CommsAppConfig {
    fn default() -> Self {
        Self {
            enabled: default_comms_enabled(),
            templates_dir: default_comms_templates_dir(),
            public_base_url: String::new(),
            default_from_email: String::new(),
            default_from_name: String::new(),
            branding: CommsBrandingConfig::default(),
            email: CommsEmailConfig::default(),
        }
    }
}

/// `[comms.branding]` sub-table. Values are surfaced verbatim in the
/// branding partial templates.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct CommsBrandingConfig {
    /// Company / brand name. Substituted into `{{ header }}` and `{{ footer }}`.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_NAME`.
    #[serde(default)]
    pub company_name: String,
    /// Postal/legal address used in the compliance footer.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_ADDRESS`.
    #[serde(default)]
    pub company_address: String,
    /// Absolute URL of the logo referenced from the HTML logo partial.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_BRANDING_LOGO_URL`.
    #[serde(default)]
    pub logo_url: String,
}

/// `[comms.email]` sub-table. The `provider` discriminator selects which
/// per-provider sub-table is active; the others are ignored at dispatch
/// time but accepted at parse time so an operator can keep them as
/// commented-out reference templates.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CommsEmailConfig {
    /// Active email provider for this deployment.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_EMAIL_PROVIDER`.
    #[serde(default = "default_comms_email_provider")]
    pub provider: CommsEmailProvider,
    /// Mailgun-specific settings; consulted only when `provider = "mailgun"`.
    #[serde(default)]
    pub mailgun: MailgunAppConfig,
}

impl Default for CommsEmailConfig {
    fn default() -> Self {
        Self {
            provider: default_comms_email_provider(),
            mailgun: MailgunAppConfig::default(),
        }
    }
}

/// Active email-medium provider. Variants are added as their `comms` impls
/// land; unknown values in TOML produce a clear deserialisation error.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CommsEmailProvider {
    /// In-memory capture stub. Used when `[comms].enabled = false` or for
    /// dev / CI work where contacting a real provider is undesirable.
    #[default]
    Stub,
    /// Mailgun HTTP API. Sub-config in `[comms.email.mailgun]`. The API key
    /// is environment-only — `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY`.
    Mailgun,
}

/// `[comms.email.mailgun]` sub-table.
///
/// `api_key` is **never** read from `config.toml`. It is sourced exclusively
/// from `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY` so secrets never land
/// in committed files or container images.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MailgunAppConfig {
    /// Sending domain registered with Mailgun (e.g. `mg.example.com`).
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_DOMAIN`.
    #[serde(default)]
    pub domain: String,
    /// Mailgun regional endpoint: `"us"` (default) or `"eu"`.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_REGION`.
    #[serde(default = "default_comms_email_mailgun_region")]
    pub region: String,
    /// Resolved at startup from `MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY`.
    /// Skipped during (de)serialisation — never read from or written to the
    /// config file.
    #[serde(skip)]
    pub api_key: String,
}

impl Default for MailgunAppConfig {
    fn default() -> Self {
        Self {
            domain: String::new(),
            region: default_comms_email_mailgun_region(),
            api_key: String::new(),
        }
    }
}

/// Outcome of `CommsAppConfig::resolve_and_validate`. Carries any
/// environment-resolution warnings that should be logged at startup.
///
/// `comms` is treated as soft state: failures degrade the notifications
/// surface but don't block server startup. That contrasts with `[oauth]`
/// and `[storage.sql]`, where missing secrets hard-fail `AppConfig::load`.
#[derive(Debug, Default, Clone)]
pub struct CommsValidation {
    pub warnings: Vec<String>,
}

impl CommsAppConfig {
    /// Resolve env-only secrets into the config and collect warnings.
    ///
    /// When `enabled = false`, returns an empty validation without
    /// inspecting anything. When `enabled = true`, reads each required env
    /// var for the active provider, populates the corresponding `#[serde(skip)]`
    /// fields, and accumulates a warning for every missing or empty value
    /// rather than returning early — so the operator sees the full set of
    /// problems in one log pass.
    pub fn resolve_and_validate(&mut self) -> CommsValidation {
        let mut warnings = Vec::new();
        if !self.enabled {
            return CommsValidation { warnings };
        }
        match self.email.provider {
            CommsEmailProvider::Stub => {}
            CommsEmailProvider::Mailgun => {
                let env = "MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY";
                match std::env::var(env) {
                    Ok(v) if !v.is_empty() => {
                        self.email.mailgun.api_key = v;
                    }
                    Ok(_) => {
                        warnings.push(format!(
                            "[comms.email.mailgun] env var \"{env}\" is set but empty; sends will fail"
                        ));
                    }
                    Err(_) => {
                        warnings.push(format!(
                            "[comms.email.mailgun] env var \"{env}\" is not set; sends will fail"
                        ));
                    }
                }
                if self.email.mailgun.domain.trim().is_empty() {
                    warnings.push(
                        "[comms.email.mailgun] domain is empty; sends will fail".to_string(),
                    );
                }
            }
        }
        if self.public_base_url.trim().is_empty() {
            warnings.push(
                "[comms].public_base_url is empty; template links will be malformed".to_string(),
            );
        }
        if self.default_from_email.trim().is_empty() {
            warnings.push(
                "[comms].default_from_email is empty; send_template calls will fail".to_string(),
            );
        }
        CommsValidation { warnings }
    }
}

/// Apply `MAZE_WEB_SERVER_COMMS_*` environment-variable overrides to the
/// supplied config builder. Called from `AppConfig::set_env_overrides` so
/// every comms key participates in the same precedence pipeline as the
/// rest of the config (defaults < TOML < env).
///
/// The api-key-like env vars (`*_API_KEY`, future passwords) are **not**
/// handled here — they're resolved into the `#[serde(skip)]` fields by
/// `resolve_and_validate` so secrets never live in the `config` crate's
/// value tree (which gets logged via `AppConfig::log_config`).
pub(crate) fn apply_env_overrides(
    mut builder: ConfigBuilder<DefaultState>,
) -> Result<ConfigBuilder<DefaultState>, config::ConfigError> {
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_ENABLED") {
        builder = builder.set_override("comms.enabled", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_TEMPLATES_DIR") {
        builder = builder.set_override("comms.templates_dir", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_PUBLIC_BASE_URL") {
        builder = builder.set_override("comms.public_base_url", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_DEFAULT_FROM_EMAIL") {
        builder = builder.set_override("comms.default_from_email", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_DEFAULT_FROM_NAME") {
        builder = builder.set_override("comms.default_from_name", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_NAME") {
        builder = builder.set_override("comms.branding.company_name", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_ADDRESS") {
        builder = builder.set_override("comms.branding.company_address", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_BRANDING_LOGO_URL") {
        builder = builder.set_override("comms.branding.logo_url", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_PROVIDER") {
        builder = builder.set_override("comms.email.provider", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_DOMAIN") {
        builder = builder.set_override("comms.email.mailgun.domain", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_REGION") {
        builder = builder.set_override("comms.email.mailgun.region", v)?;
    }
    Ok(builder)
}

fn default_comms_enabled() -> bool {
    false
}
fn default_comms_templates_dir() -> String {
    "data/comms_templates".to_string()
}
fn default_comms_email_provider() -> CommsEmailProvider {
    CommsEmailProvider::Stub
}
fn default_comms_email_mailgun_region() -> String {
    "us".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test envelope so we can deserialise a `[comms]` block via
    /// `toml::from_str` without involving `AppConfig` (which doesn't carry
    /// the field yet in this sub-step).
    #[derive(Debug, Deserialize, Default)]
    struct Envelope {
        #[serde(default)]
        comms: CommsAppConfig,
    }

    #[test]
    fn full_comms_block_with_branding_and_mailgun_deserialises() {
        let toml = r#"
            [comms]
            enabled = true
            templates_dir = "data/comms_templates"
            public_base_url = "https://maze.example.com"
            default_from_email = "noreply@example.com"
            default_from_name = "Maze"

            [comms.branding]
            company_name = "Maze, Inc."
            company_address = "123 Example St"
            logo_url = "https://maze.example.com/static/logo.png"

            [comms.email]
            provider = "mailgun"

            [comms.email.mailgun]
            domain = "mg.example.com"
            region = "eu"
        "#;
        let env: Envelope = toml::from_str(toml).expect("parse");
        let cfg = env.comms;
        assert!(cfg.enabled);
        assert_eq!(cfg.templates_dir, "data/comms_templates");
        assert_eq!(cfg.public_base_url, "https://maze.example.com");
        assert_eq!(cfg.default_from_email, "noreply@example.com");
        assert_eq!(cfg.default_from_name, "Maze");
        assert_eq!(cfg.branding.company_name, "Maze, Inc.");
        assert_eq!(cfg.branding.company_address, "123 Example St");
        assert_eq!(cfg.branding.logo_url, "https://maze.example.com/static/logo.png");
        assert_eq!(cfg.email.provider, CommsEmailProvider::Mailgun);
        assert_eq!(cfg.email.mailgun.domain, "mg.example.com");
        assert_eq!(cfg.email.mailgun.region, "eu");
        // api_key is env-only — never deserialised from TOML.
        assert!(cfg.email.mailgun.api_key.is_empty());
    }

    #[test]
    fn defaults_apply_when_section_absent() {
        let env: Envelope = toml::from_str("").expect("parse");
        let cfg = env.comms;
        assert!(!cfg.enabled);
        assert_eq!(cfg.templates_dir, "data/comms_templates");
        assert!(cfg.public_base_url.is_empty());
        assert!(cfg.default_from_email.is_empty());
        assert!(cfg.default_from_name.is_empty());
        assert!(cfg.branding.company_name.is_empty());
        assert!(cfg.branding.company_address.is_empty());
        assert!(cfg.branding.logo_url.is_empty());
        assert_eq!(cfg.email.provider, CommsEmailProvider::Stub);
        assert!(cfg.email.mailgun.domain.is_empty());
        assert_eq!(cfg.email.mailgun.region, "us");
        assert!(cfg.email.mailgun.api_key.is_empty());
    }

    #[test]
    fn empty_comms_section_uses_field_defaults() {
        let env: Envelope = toml::from_str("[comms]\n").expect("parse");
        let cfg = env.comms;
        assert!(!cfg.enabled);
        assert_eq!(cfg.templates_dir, "data/comms_templates");
        assert_eq!(cfg.email.provider, CommsEmailProvider::Stub);
        assert_eq!(cfg.email.mailgun.region, "us");
    }

    #[test]
    fn unknown_provider_discriminator_returns_clear_deserialisation_error() {
        let toml = r#"
            [comms]
            enabled = true

            [comms.email]
            provider = "unknown"
        "#;
        let err = toml::from_str::<Envelope>(toml).expect_err("must reject");
        let msg = err.to_string();
        assert!(msg.contains("unknown"), "error should name the bad value: {msg}");
        // Must not have panicked; we only got here because deserialisation
        // returned `Err` cleanly.
    }

    #[test]
    fn resolve_and_validate_when_disabled_emits_no_warnings() {
        let mut cfg = CommsAppConfig {
            enabled: false,
            ..CommsAppConfig::default()
        };
        let result = cfg.resolve_and_validate();
        assert!(result.warnings.is_empty(), "got warnings: {:?}", result.warnings);
    }

    fn enabled_mailgun_config() -> CommsAppConfig {
        CommsAppConfig {
            enabled: true,
            public_base_url: "https://maze.example.com".into(),
            default_from_email: "noreply@example.com".into(),
            email: CommsEmailConfig {
                provider: CommsEmailProvider::Mailgun,
                mailgun: MailgunAppConfig {
                    domain: "mg.example.com".into(),
                    region: "us".into(),
                    api_key: String::new(),
                },
            },
            ..CommsAppConfig::default()
        }
    }

    /// Combined into a single test so the two env-var states (unset / set)
    /// run sequentially. Splitting them across tests would race each other
    /// since cargo runs tests in parallel by default and they share the
    /// same canonical env-var name.
    #[test]
    fn resolve_and_validate_handles_mailgun_api_key_env_var() {
        let env_var = "MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_API_KEY";
        // Snapshot any prior value so we can restore it on exit.
        let prior = std::env::var(env_var).ok();

        // Case 1: env var unset → warning, api_key remains empty.
        unsafe {
            std::env::remove_var(env_var);
        }
        let mut cfg = enabled_mailgun_config();
        let result = cfg.resolve_and_validate();
        let joined = result.warnings.join("\n");
        assert!(
            joined.contains(env_var),
            "warning should name the env var; got: {joined}"
        );
        assert!(
            cfg.email.mailgun.api_key.is_empty(),
            "api_key should remain empty when env var is unset"
        );

        // Case 2: env var set → no warning, api_key populated.
        unsafe {
            std::env::set_var(env_var, "test-resolved-api-key");
        }
        let mut cfg = enabled_mailgun_config();
        let result = cfg.resolve_and_validate();
        assert_eq!(cfg.email.mailgun.api_key, "test-resolved-api-key");
        let joined = result.warnings.join("\n");
        assert!(
            !joined.contains(env_var),
            "no warning should mention the env var when it's set; got: {joined}"
        );

        // Restore prior state so other tests in this binary aren't affected.
        match prior {
            Some(v) => unsafe { std::env::set_var(env_var, v) },
            None => unsafe { std::env::remove_var(env_var) },
        }
    }

    #[test]
    fn resolve_and_validate_warns_about_missing_public_base_url_when_enabled() {
        let mut cfg = CommsAppConfig {
            enabled: true,
            public_base_url: String::new(),
            default_from_email: "noreply@example.com".into(),
            email: CommsEmailConfig::default(), // Stub provider, no env var needed
            ..CommsAppConfig::default()
        };
        let result = cfg.resolve_and_validate();
        let joined = result.warnings.join("\n");
        assert!(
            joined.contains("public_base_url"),
            "should warn about empty public_base_url; got: {joined}"
        );
    }

    #[test]
    fn resolve_and_validate_warns_about_missing_default_from_email_when_enabled() {
        let mut cfg = CommsAppConfig {
            enabled: true,
            public_base_url: "https://maze.example.com".into(),
            default_from_email: String::new(),
            email: CommsEmailConfig::default(),
            ..CommsAppConfig::default()
        };
        let result = cfg.resolve_and_validate();
        let joined = result.warnings.join("\n");
        assert!(
            joined.contains("default_from_email"),
            "should warn about empty default_from_email; got: {joined}"
        );
    }

    /// Build a builder seeded with the same defaults `AppConfig::load` uses
    /// for the comms keys, plus the supplied inline TOML.
    fn builder_with_defaults_and_toml(
        toml: &str,
    ) -> config::ConfigBuilder<config::builder::DefaultState> {
        config::Config::builder()
            .set_default("comms.enabled", false)
            .unwrap()
            .set_default("comms.templates_dir", "data/comms_templates")
            .unwrap()
            .set_default("comms.email.provider", "stub")
            .unwrap()
            .set_default("comms.email.mailgun.region", "us")
            .unwrap()
            .add_source(config::File::from_str(toml, config::FileFormat::Toml))
    }

    /// Snapshot/restore helper for env-var mutation tests. Replaces the
    /// pre-existing values during the closure and restores them on exit so
    /// other parallel tests aren't perturbed by leftover state.
    fn with_env_vars<F: FnOnce()>(vars: &[(&str, &str)], f: F) {
        let priors: Vec<_> = vars
            .iter()
            .map(|(k, _)| (*k, std::env::var(k).ok()))
            .collect();
        for (k, v) in vars {
            unsafe { std::env::set_var(k, v) };
        }
        f();
        for (k, prior) in priors {
            match prior {
                Some(v) => unsafe { std::env::set_var(k, v) },
                None => unsafe { std::env::remove_var(k) },
            }
        }
    }

    /// Sole owner of `MAZE_WEB_SERVER_COMMS_PUBLIC_BASE_URL` in the test
    /// suite — no other test reads or writes this variable, so the snapshot
    /// pattern is safe under `cargo test`'s default thread pool.
    #[test]
    fn apply_env_overrides_lets_env_var_win_over_toml() {
        let toml = r#"
            [comms]
            public_base_url = "https://from-toml.example.com"
        "#;
        with_env_vars(
            &[(
                "MAZE_WEB_SERVER_COMMS_PUBLIC_BASE_URL",
                "https://from-env.example.com",
            )],
            || {
                let builder = builder_with_defaults_and_toml(toml);
                let builder = apply_env_overrides(builder).expect("apply env overrides");
                let settings = builder.build().expect("build");
                let cfg: CommsAppConfig = settings.get("comms").expect("deserialize");
                assert_eq!(cfg.public_base_url, "https://from-env.example.com");
            },
        );
    }

    /// Round-trip test through the full builder pipeline. Uses a distinct
    /// set of env vars from the other env-touching tests in this module so
    /// the suite is safe to run with the default parallel thread pool.
    #[test]
    fn apply_env_overrides_full_pipeline_propagates_multiple_keys() {
        let toml = r#"
            [comms]
            enabled = false
            default_from_email = "noreply@from-toml.example.com"
            default_from_name = "Maze (TOML)"

            [comms.email]
            provider = "stub"

            [comms.email.mailgun]
            domain = "from-toml.mg.example.com"
        "#;
        with_env_vars(
            &[
                ("MAZE_WEB_SERVER_COMMS_DEFAULT_FROM_NAME", "Maze (env)"),
                ("MAZE_WEB_SERVER_COMMS_EMAIL_PROVIDER", "mailgun"),
                (
                    "MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_DOMAIN",
                    "from-env.mg.example.com",
                ),
            ],
            || {
                let builder = builder_with_defaults_and_toml(toml);
                let builder = apply_env_overrides(builder).expect("apply env overrides");
                let settings = builder.build().expect("build");
                let cfg: CommsAppConfig = settings.get("comms").expect("deserialize");

                // env wins over TOML where both are present
                assert_eq!(cfg.default_from_name, "Maze (env)");
                assert_eq!(cfg.email.provider, CommsEmailProvider::Mailgun);
                assert_eq!(cfg.email.mailgun.domain, "from-env.mg.example.com");
                // TOML wins over default where env is unset
                assert_eq!(
                    cfg.default_from_email,
                    "noreply@from-toml.example.com"
                );
                // default wins where neither TOML nor env is set
                assert_eq!(cfg.email.mailgun.region, "us");
                assert_eq!(cfg.templates_dir, "data/comms_templates");
            },
        );
    }
}
