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

    /// Public base URL the server is reachable at, used to build links
    /// inside templates (e.g. `{{ reset_link }}`, `{{ verification_link }}`).
    /// Required when `enabled = true`.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_PUBLIC_BASE_URL`.
    #[serde(default)]
    pub public_base_url: String,

    /// Branding values fed into the partial templates (`{{ logo }}`,
    /// `{{ header }}`, `{{ footer }}`).
    #[serde(default)]
    pub branding: CommsBrandingConfig,

    /// Email-medium configuration: provider discriminator, default sender
    /// identity, templates directory, and per-provider sub-tables. Only the
    /// section matching `provider` is consulted at dispatch time.
    #[serde(default)]
    pub email: CommsEmailConfig,
}

impl Default for CommsAppConfig {
    fn default() -> Self {
        Self {
            enabled: default_comms_enabled(),
            public_base_url: String::new(),
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
    /// Public URL for the company's website (the brand's home on the web).
    /// Distinct from `[comms].public_base_url`, which is where this server
    /// deployment is reachable. Templates can reference `{{ company_url }}`
    /// for ad-hoc links to the company site that aren't tied to a specific
    /// server endpoint.
    ///
    /// If left empty, `resolve_and_validate` populates it from
    /// `[comms].public_base_url` at startup, so single-tenant deployments
    /// where the marketing site and the server share a domain don't have
    /// to set both.
    ///
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_URL`.
    #[serde(default)]
    pub company_url: String,
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
    /// Default from-address used when `send_template` synthesises an
    /// email message. Required when `[comms].enabled = true`.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_EMAIL_DEFAULT_FROM`.
    #[serde(default)]
    pub default_from: String,
    /// Default display name paired with `default_from`.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_EMAIL_DEFAULT_FROM_NAME`.
    #[serde(default = "default_comms_email_default_from_name")]
    pub default_from_name: String,
    /// Directory holding filesystem-override templates and partials for
    /// the email medium. Templates here override embedded defaults;
    /// templates not present here fall back to the embedded copies.
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_EMAIL_TEMPLATES_DIR`.
    #[serde(default = "default_comms_email_templates_dir")]
    pub templates_dir: String,
    /// Mailgun-specific settings; consulted only when `provider = "mailgun"`.
    #[serde(default)]
    pub mailgun: MailgunAppConfig,
    /// SMTP+XOAUTH2 settings; consulted only when `provider = "smtp_oauth2"`.
    #[serde(default)]
    pub smtp_oauth2: SmtpOauth2AppConfig,
}

impl Default for CommsEmailConfig {
    fn default() -> Self {
        Self {
            provider: default_comms_email_provider(),
            default_from: String::new(),
            default_from_name: default_comms_email_default_from_name(),
            templates_dir: default_comms_email_templates_dir(),
            mailgun: MailgunAppConfig::default(),
            smtp_oauth2: SmtpOauth2AppConfig::default(),
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
    /// SMTP transport authenticated with XOAUTH2 against an OAuth token
    /// source. Sub-config in `[comms.email.smtp_oauth2]`, with per-flow
    /// `[comms.email.smtp_oauth2.microsoft]` (Azure AD client-credentials)
    /// and `[comms.email.smtp_oauth2.google]` (Workspace service-account
    /// with optional domain-wide delegation) sub-tables. The Microsoft
    /// client secret is environment-only —
    /// `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_SECRET`.
    /// The Google service-account private key lives in the JSON file at
    /// `service_account_json_path`.
    SmtpOauth2,
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

/// `[comms.email.smtp_oauth2]` sub-table.
///
/// Selects the SMTP relay (`host` / `port` / `tls`), the SASL identity
/// (`username` — typically the From-address mailbox), and which OAuth
/// token-source vendor to use (`vendor`). Each vendor has its own
/// per-vendor sub-table; only the one matching `vendor` is consulted at
/// dispatch time.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SmtpOauth2AppConfig {
    /// SMTP relay hostname (e.g. `smtp.office365.com`, `smtp.gmail.com`).
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_HOST`.
    #[serde(default)]
    pub host: String,
    /// SMTP submission port. Defaults to 587 (STARTTLS); use 465 for
    /// implicit TLS. Can be overridden with
    /// `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_PORT`.
    #[serde(default = "default_smtp_oauth2_port")]
    pub port: u16,
    /// Transport-security mode: `"starttls"` (default — port 587),
    /// `"implicit"` (port 465), or `"plain"` (no TLS — dev/test only).
    /// Can be overridden with `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_TLS`.
    #[serde(default = "default_smtp_oauth2_tls")]
    pub tls: String,
    /// SASL identity presented during AUTH XOAUTH2. For company-mailbox
    /// flows this is the mailbox address being sent from.
    /// Can be overridden with
    /// `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_USERNAME`.
    #[serde(default)]
    pub username: String,
    /// Discriminator selecting which per-vendor sub-table is active.
    /// Can be overridden with
    /// `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_VENDOR`.
    #[serde(default)]
    pub vendor: SmtpOauth2Vendor,
    /// Microsoft Azure AD client-credentials settings.
    #[serde(default)]
    pub microsoft: SmtpOauth2MicrosoftConfig,
    /// Google Workspace service-account settings.
    #[serde(default)]
    pub google: SmtpOauth2GoogleConfig,
}

impl Default for SmtpOauth2AppConfig {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: default_smtp_oauth2_port(),
            tls: default_smtp_oauth2_tls(),
            username: String::new(),
            vendor: SmtpOauth2Vendor::default(),
            microsoft: SmtpOauth2MicrosoftConfig::default(),
            google: SmtpOauth2GoogleConfig::default(),
        }
    }
}

/// OAuth vendor used to obtain the bearer token presented to the SMTP
/// server via XOAUTH2. Each variant pins a single OAuth flow chosen for
/// its server-side bulk-send fitness — `client_credentials` for Microsoft,
/// `service_account` (JWT-bearer) for Google. If a vendor ever offers a
/// second viable flow we'd want to support, that's a separate
/// discriminator alongside this one rather than a new variant.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SmtpOauth2Vendor {
    /// Microsoft Azure AD `client_credentials` flow against a registered
    /// app with `Mail.Send` (or equivalent) constrained to a single
    /// mailbox by Application Access Policy. Used by Microsoft 365.
    #[default]
    Microsoft,
    /// Google service-account JWT-bearer flow with optional domain-wide
    /// delegation. Used by Google Workspace.
    Google,
}

/// `[comms.email.smtp_oauth2.microsoft]` sub-table.
///
/// `client_secret` is **never** read from `config.toml`. It is sourced
/// exclusively from
/// `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_SECRET` so
/// secrets never land in committed files or container images.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SmtpOauth2MicrosoftConfig {
    /// Azure AD tenant identifier (a UUID, or `common` / `organizations`).
    /// Can be overridden with
    /// `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_TENANT_ID`.
    #[serde(default)]
    pub tenant_id: String,
    /// Azure AD application (client) identifier. Can be overridden with
    /// `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_ID`.
    #[serde(default)]
    pub client_id: String,
    /// OAuth scopes to request. Defaults to
    /// `["https://outlook.office.com/SMTP.Send"]`.
    #[serde(default = "default_smtp_oauth2_microsoft_scopes")]
    pub scopes: Vec<String>,
    /// Resolved at startup from the env var listed above. Skipped during
    /// (de)serialisation — never read from or written to the config file.
    #[serde(skip)]
    pub client_secret: String,
}

impl Default for SmtpOauth2MicrosoftConfig {
    fn default() -> Self {
        Self {
            tenant_id: String::new(),
            client_id: String::new(),
            scopes: default_smtp_oauth2_microsoft_scopes(),
            client_secret: String::new(),
        }
    }
}

/// `[comms.email.smtp_oauth2.google]` sub-table.
///
/// The service-account private key is read from the JSON key file at
/// `service_account_json_path` at first send. The path itself is fine to
/// keep in the config file; the key material lives on disk and is access-
/// controlled via the deployment's filesystem permissions.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SmtpOauth2GoogleConfig {
    /// Filesystem path to the GCP service-account JSON key file.
    /// Can be overridden with
    /// `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_SERVICE_ACCOUNT_JSON_PATH`.
    #[serde(default)]
    pub service_account_json_path: String,
    /// Workspace mailbox the service account impersonates via domain-wide
    /// delegation (e.g. `noreply@company.com`). Empty disables impersonation.
    /// Can be overridden with
    /// `MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_DELEGATED_SUBJECT`.
    #[serde(default)]
    pub delegated_subject: String,
    /// OAuth scopes to request. Defaults to
    /// `["https://www.googleapis.com/auth/gmail.send"]`.
    #[serde(default = "default_smtp_oauth2_google_scopes")]
    pub scopes: Vec<String>,
}

impl Default for SmtpOauth2GoogleConfig {
    fn default() -> Self {
        Self {
            service_account_json_path: String::new(),
            delegated_subject: String::new(),
            scopes: default_smtp_oauth2_google_scopes(),
        }
    }
}

/// Outcome of `CommsAppConfig::resolve_and_validate`. Carries any
/// environment-resolution warnings that should be logged at startup.
///
/// Two failure tiers:
///   * Universally-required, file-only TOML fields (`public_base_url`,
///     `email.default_from`) hard-fail config load via the `Err` arm of
///     the function's `Result`, mirroring how `[oauth]` and
///     `[storage.sql]` handle their required values.
///   * Env-var-sourced provider secrets (Mailgun API key, SmtpOauth2
///     client secret, etc.) emit warnings that go into this struct.
///     They're soft because operators legitimately defer secret setup
///     (CI smoke tests, secret managers, sidecars).
#[derive(Debug, Default, Clone)]
pub struct CommsValidation {
    pub warnings: Vec<String>,
}

impl CommsAppConfig {
    /// Resolve env-only secrets and derived defaults into the config, and
    /// collect warnings.
    ///
    /// Always applied (regardless of `enabled`): if `branding.company_url`
    /// is empty, it inherits the value of `public_base_url`. This keeps
    /// the logged config (`AppConfig::log_config`) honest about what
    /// `{{ company_url }}` will render as.
    ///
    /// When `enabled = false`, returns an empty validation without
    /// inspecting anything else. When `enabled = true`:
    ///   * Hard-fails (returns `Err`) if `public_base_url` or
    ///     `email.default_from` are empty — both are required by every
    ///     provider for `send_template` to synthesise an outbound
    ///     message, and neither has an env-var deferral path. Both
    ///     errors are reported in one message so the operator sees the
    ///     full set in one log pass.
    ///   * Reads each required env var for the active provider and
    ///     populates the corresponding `#[serde(skip)]` fields.
    ///   * Accumulates warnings for empty / missing env-var-sourced
    ///     secrets rather than returning early — same one-pass rationale.
    pub fn resolve_and_validate(&mut self) -> Result<CommsValidation, String> {
        let mut warnings = Vec::new();

        // Branding fallback: if the operator didn't set company_url, use
        // public_base_url. Useful for single-tenant deployments where the
        // marketing site and the server share a domain.
        if self.branding.company_url.is_empty() {
            self.branding.company_url = self.public_base_url.clone();
        }

        if !self.enabled {
            return Ok(CommsValidation { warnings });
        }

        // Universally-required, file-only fields. Collect both before
        // failing so the operator sees the full picture instead of
        // having to fix one, restart, fix the next.
        let mut hard_errors: Vec<String> = Vec::new();
        if self.public_base_url.trim().is_empty() {
            hard_errors.push(
                "[comms].public_base_url is required when comms.enabled = true (used to build verification / reset links inside templates)".to_string(),
            );
        }
        if self.email.default_from.trim().is_empty() {
            hard_errors.push(
                "[comms.email].default_from is required when comms.enabled = true (used as the sender identity for every outbound message)".to_string(),
            );
        }
        if !hard_errors.is_empty() {
            return Err(hard_errors.join("; "));
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
            CommsEmailProvider::SmtpOauth2 => {
                if self.email.smtp_oauth2.host.trim().is_empty() {
                    warnings.push(
                        "[comms.email.smtp_oauth2] host is empty; sends will fail".to_string(),
                    );
                }
                if self.email.smtp_oauth2.port == 0 {
                    warnings.push(
                        "[comms.email.smtp_oauth2] port is 0; sends will fail".to_string(),
                    );
                }
                if self.email.smtp_oauth2.username.trim().is_empty() {
                    warnings.push(
                        "[comms.email.smtp_oauth2] username is empty; sends will fail".to_string(),
                    );
                }
                match self.email.smtp_oauth2.vendor {
                    SmtpOauth2Vendor::Microsoft => {
                        let env =
                            "MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_SECRET";
                        match std::env::var(env) {
                            Ok(v) if !v.is_empty() => {
                                self.email.smtp_oauth2.microsoft.client_secret = v;
                            }
                            Ok(_) => {
                                warnings.push(format!(
                                    "[comms.email.smtp_oauth2.microsoft] env var \"{env}\" is set but empty; sends will fail"
                                ));
                            }
                            Err(_) => {
                                warnings.push(format!(
                                    "[comms.email.smtp_oauth2.microsoft] env var \"{env}\" is not set; sends will fail"
                                ));
                            }
                        }
                        if self.email.smtp_oauth2.microsoft.tenant_id.trim().is_empty() {
                            warnings.push(
                                "[comms.email.smtp_oauth2.microsoft] tenant_id is empty; sends will fail"
                                    .to_string(),
                            );
                        }
                        if self.email.smtp_oauth2.microsoft.client_id.trim().is_empty() {
                            warnings.push(
                                "[comms.email.smtp_oauth2.microsoft] client_id is empty; sends will fail"
                                    .to_string(),
                            );
                        }
                    }
                    SmtpOauth2Vendor::Google => {
                        if self
                            .email
                            .smtp_oauth2
                            .google
                            .service_account_json_path
                            .trim()
                            .is_empty()
                        {
                            warnings.push(
                                "[comms.email.smtp_oauth2.google] service_account_json_path is empty; sends will fail"
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }
        Ok(CommsValidation { warnings })
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
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_PUBLIC_BASE_URL") {
        builder = builder.set_override("comms.public_base_url", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_NAME") {
        builder = builder.set_override("comms.branding.company_name", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_ADDRESS") {
        builder = builder.set_override("comms.branding.company_address", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_BRANDING_COMPANY_URL") {
        builder = builder.set_override("comms.branding.company_url", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_BRANDING_LOGO_URL") {
        builder = builder.set_override("comms.branding.logo_url", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_PROVIDER") {
        builder = builder.set_override("comms.email.provider", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_DEFAULT_FROM") {
        builder = builder.set_override("comms.email.default_from", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_DEFAULT_FROM_NAME") {
        builder = builder.set_override("comms.email.default_from_name", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_TEMPLATES_DIR") {
        builder = builder.set_override("comms.email.templates_dir", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_DOMAIN") {
        builder = builder.set_override("comms.email.mailgun.domain", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_MAILGUN_REGION") {
        builder = builder.set_override("comms.email.mailgun.region", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_HOST") {
        builder = builder.set_override("comms.email.smtp_oauth2.host", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_PORT") {
        builder = builder.set_override("comms.email.smtp_oauth2.port", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_TLS") {
        builder = builder.set_override("comms.email.smtp_oauth2.tls", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_USERNAME") {
        builder = builder.set_override("comms.email.smtp_oauth2.username", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_VENDOR") {
        builder = builder.set_override("comms.email.smtp_oauth2.vendor", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_TENANT_ID") {
        builder = builder.set_override("comms.email.smtp_oauth2.microsoft.tenant_id", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_ID") {
        builder = builder.set_override("comms.email.smtp_oauth2.microsoft.client_id", v)?;
    }
    if let Ok(v) =
        std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_SERVICE_ACCOUNT_JSON_PATH")
    {
        builder = builder.set_override("comms.email.smtp_oauth2.google.service_account_json_path", v)?;
    }
    if let Ok(v) = std::env::var("MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_DELEGATED_SUBJECT")
    {
        builder = builder.set_override("comms.email.smtp_oauth2.google.delegated_subject", v)?;
    }
    Ok(builder)
}

pub(crate) fn default_comms_enabled() -> bool {
    false
}
fn default_comms_email_provider() -> CommsEmailProvider {
    CommsEmailProvider::Stub
}
pub(crate) fn default_comms_email_default_from_name() -> String {
    "The Maze Team".to_string()
}
pub(crate) fn default_comms_email_templates_dir() -> String {
    "config/email_templates".to_string()
}
pub(crate) fn default_comms_email_mailgun_region() -> String {
    "us".to_string()
}
pub(crate) fn default_smtp_oauth2_port() -> u16 {
    587
}
pub(crate) fn default_smtp_oauth2_tls() -> String {
    "starttls".to_string()
}
pub(crate) fn default_smtp_oauth2_microsoft_scopes() -> Vec<String> {
    vec!["https://outlook.office.com/SMTP.Send".to_string()]
}
pub(crate) fn default_smtp_oauth2_google_scopes() -> Vec<String> {
    vec!["https://www.googleapis.com/auth/gmail.send".to_string()]
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
            public_base_url = "https://maze.example.com"

            [comms.branding]
            company_name = "Maze, Inc."
            company_address = "123 Example St"
            company_url = "https://acme.example.com"
            logo_url = "https://maze.example.com/static/logo.png"

            [comms.email]
            provider = "mailgun"
            default_from = "noreply@example.com"
            default_from_name = "Maze"
            templates_dir = "config/email_templates"

            [comms.email.mailgun]
            domain = "mg.example.com"
            region = "eu"
        "#;
        let env: Envelope = toml::from_str(toml).expect("parse");
        let cfg = env.comms;
        assert!(cfg.enabled);
        assert_eq!(cfg.public_base_url, "https://maze.example.com");
        assert_eq!(cfg.branding.company_name, "Maze, Inc.");
        assert_eq!(cfg.branding.company_address, "123 Example St");
        assert_eq!(cfg.branding.company_url, "https://acme.example.com");
        assert_eq!(cfg.branding.logo_url, "https://maze.example.com/static/logo.png");
        assert_eq!(cfg.email.provider, CommsEmailProvider::Mailgun);
        assert_eq!(cfg.email.default_from, "noreply@example.com");
        assert_eq!(cfg.email.default_from_name, "Maze");
        assert_eq!(cfg.email.templates_dir, "config/email_templates");
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
        assert!(cfg.public_base_url.is_empty());
        assert!(cfg.branding.company_name.is_empty());
        assert!(cfg.branding.company_address.is_empty());
        assert!(cfg.branding.company_url.is_empty());
        assert!(cfg.branding.logo_url.is_empty());
        assert_eq!(cfg.email.provider, CommsEmailProvider::Stub);
        assert!(cfg.email.default_from.is_empty());
        assert_eq!(cfg.email.default_from_name, "The Maze Team");
        assert_eq!(cfg.email.templates_dir, "config/email_templates");
        assert!(cfg.email.mailgun.domain.is_empty());
        assert_eq!(cfg.email.mailgun.region, "us");
        assert!(cfg.email.mailgun.api_key.is_empty());
    }

    #[test]
    fn empty_comms_section_uses_field_defaults() {
        let env: Envelope = toml::from_str("[comms]\n").expect("parse");
        let cfg = env.comms;
        assert!(!cfg.enabled);
        assert_eq!(cfg.email.provider, CommsEmailProvider::Stub);
        assert_eq!(cfg.email.default_from_name, "The Maze Team");
        assert_eq!(cfg.email.templates_dir, "config/email_templates");
        assert_eq!(cfg.email.mailgun.region, "us");
    }

    #[test]
    fn resolve_and_validate_falls_back_company_url_to_public_base_url_when_empty() {
        let mut cfg = CommsAppConfig {
            public_base_url: "https://maze.example.com".into(),
            branding: CommsBrandingConfig {
                company_url: String::new(),
                ..CommsBrandingConfig::default()
            },
            ..CommsAppConfig::default()
        };
        let _ = cfg.resolve_and_validate();
        assert_eq!(cfg.branding.company_url, "https://maze.example.com");
    }

    #[test]
    fn resolve_and_validate_keeps_explicit_company_url() {
        let mut cfg = CommsAppConfig {
            public_base_url: "https://maze.example.com".into(),
            branding: CommsBrandingConfig {
                company_url: "https://acme.example.com".into(),
                ..CommsBrandingConfig::default()
            },
            ..CommsAppConfig::default()
        };
        let _ = cfg.resolve_and_validate();
        assert_eq!(cfg.branding.company_url, "https://acme.example.com");
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
        let result = cfg.resolve_and_validate().expect("disabled never hard-fails");
        assert!(result.warnings.is_empty(), "got warnings: {:?}", result.warnings);
    }

    #[test]
    fn resolve_and_validate_when_disabled_does_not_check_universals() {
        // Both universals empty + enabled = false → still Ok. The
        // hard-fail tier only applies when sends are about to actually run.
        let mut cfg = CommsAppConfig {
            enabled: false,
            public_base_url: String::new(),
            email: CommsEmailConfig {
                default_from: String::new(),
                ..CommsEmailConfig::default()
            },
            ..CommsAppConfig::default()
        };
        let result = cfg.resolve_and_validate();
        assert!(result.is_ok(), "disabled comms must never hard-fail; got: {result:?}");
    }

    fn enabled_mailgun_config() -> CommsAppConfig {
        CommsAppConfig {
            enabled: true,
            public_base_url: "https://maze.example.com".into(),
            email: CommsEmailConfig {
                provider: CommsEmailProvider::Mailgun,
                default_from: "noreply@example.com".into(),
                default_from_name: "The Maze Team".into(),
                templates_dir: "config/email_templates".into(),
                mailgun: MailgunAppConfig {
                    domain: "mg.example.com".into(),
                    region: "us".into(),
                    api_key: String::new(),
                },
                smtp_oauth2: SmtpOauth2AppConfig::default(),
            },
            ..CommsAppConfig::default()
        }
    }

    fn enabled_smtp_oauth2_microsoft_config() -> CommsAppConfig {
        CommsAppConfig {
            enabled: true,
            public_base_url: "https://maze.example.com".into(),
            email: CommsEmailConfig {
                provider: CommsEmailProvider::SmtpOauth2,
                default_from: "noreply@contoso.com".into(),
                default_from_name: "The Maze Team".into(),
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
                        client_id: "00000000-0000-0000-0000-000000000000".into(),
                        scopes: vec!["https://outlook.office.com/SMTP.Send".into()],
                        client_secret: String::new(),
                    },
                    google: SmtpOauth2GoogleConfig::default(),
                },
            },
            ..CommsAppConfig::default()
        }
    }

    fn enabled_smtp_oauth2_google_config() -> CommsAppConfig {
        CommsAppConfig {
            enabled: true,
            public_base_url: "https://maze.example.com".into(),
            email: CommsEmailConfig {
                provider: CommsEmailProvider::SmtpOauth2,
                default_from: "noreply@company.com".into(),
                default_from_name: "The Maze Team".into(),
                templates_dir: "config/email_templates".into(),
                mailgun: MailgunAppConfig::default(),
                smtp_oauth2: SmtpOauth2AppConfig {
                    host: "smtp.gmail.com".into(),
                    port: 587,
                    tls: "starttls".into(),
                    username: "noreply@company.com".into(),
                    vendor: SmtpOauth2Vendor::Google,
                    microsoft: SmtpOauth2MicrosoftConfig::default(),
                    google: SmtpOauth2GoogleConfig {
                        service_account_json_path: "/etc/maze/gcp-service-account.json".into(),
                        delegated_subject: "noreply@company.com".into(),
                        scopes: vec!["https://www.googleapis.com/auth/gmail.send".into()],
                    },
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
        let result = cfg.resolve_and_validate().expect("universals satisfied");
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
        let result = cfg.resolve_and_validate().expect("universals satisfied");
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
    fn resolve_and_validate_hard_fails_for_missing_public_base_url_when_enabled() {
        let mut cfg = CommsAppConfig {
            enabled: true,
            public_base_url: String::new(),
            email: CommsEmailConfig {
                default_from: "noreply@example.com".into(),
                ..CommsEmailConfig::default()
            },
            ..CommsAppConfig::default()
        };
        let err = cfg
            .resolve_and_validate()
            .expect_err("missing public_base_url must hard-fail when enabled");
        assert!(
            err.contains("public_base_url"),
            "error should name the missing field; got: {err}"
        );
    }

    #[test]
    fn resolve_and_validate_hard_fails_for_missing_default_from_when_enabled() {
        let mut cfg = CommsAppConfig {
            enabled: true,
            public_base_url: "https://maze.example.com".into(),
            email: CommsEmailConfig {
                default_from: String::new(),
                ..CommsEmailConfig::default()
            },
            ..CommsAppConfig::default()
        };
        let err = cfg
            .resolve_and_validate()
            .expect_err("missing default_from must hard-fail when enabled");
        assert!(
            err.contains("default_from"),
            "error should name the missing field; got: {err}"
        );
    }

    #[test]
    fn resolve_and_validate_reports_both_universal_errors_in_one_message() {
        let mut cfg = CommsAppConfig {
            enabled: true,
            public_base_url: String::new(),
            email: CommsEmailConfig {
                default_from: String::new(),
                ..CommsEmailConfig::default()
            },
            ..CommsAppConfig::default()
        };
        let err = cfg
            .resolve_and_validate()
            .expect_err("both universals empty must hard-fail");
        // Single error string covers both so the operator sees the full
        // picture in one log line instead of fix-restart-fix-restart.
        assert!(
            err.contains("public_base_url"),
            "combined error should include public_base_url; got: {err}"
        );
        assert!(
            err.contains("default_from"),
            "combined error should include default_from; got: {err}"
        );
    }

    #[test]
    fn resolve_and_validate_hard_fails_only_when_universals_empty() {
        // Whitespace-only is treated the same as empty — `.trim()` first.
        let mut cfg = CommsAppConfig {
            enabled: true,
            public_base_url: "   ".into(),
            email: CommsEmailConfig {
                default_from: "\t\n".into(),
                ..CommsEmailConfig::default()
            },
            ..CommsAppConfig::default()
        };
        let err = cfg
            .resolve_and_validate()
            .expect_err("whitespace-only universals must hard-fail");
        assert!(err.contains("public_base_url"));
        assert!(err.contains("default_from"));
    }

    #[test]
    fn full_smtp_oauth2_block_with_microsoft_and_google_subtables_deserialises() {
        let toml = r#"
            [comms]
            enabled = true
            public_base_url = "https://maze.example.com"

            [comms.email]
            provider = "smtp_oauth2"
            default_from = "noreply@contoso.com"

            [comms.email.smtp_oauth2]
            host = "smtp.office365.com"
            port = 587
            tls = "starttls"
            username = "noreply@contoso.com"
            vendor = "microsoft"

            [comms.email.smtp_oauth2.microsoft]
            tenant_id = "00000000-0000-0000-0000-000000000000"
            client_id = "11111111-1111-1111-1111-111111111111"
            scopes = ["https://outlook.office.com/SMTP.Send"]

            [comms.email.smtp_oauth2.google]
            service_account_json_path = "/etc/maze/gcp-service-account.json"
            delegated_subject = "noreply@company.com"
            scopes = ["https://www.googleapis.com/auth/gmail.send"]
        "#;
        let env: Envelope = toml::from_str(toml).expect("parse");
        let cfg = env.comms;
        assert_eq!(cfg.email.provider, CommsEmailProvider::SmtpOauth2);
        assert_eq!(cfg.email.smtp_oauth2.host, "smtp.office365.com");
        assert_eq!(cfg.email.smtp_oauth2.port, 587);
        assert_eq!(cfg.email.smtp_oauth2.tls, "starttls");
        assert_eq!(cfg.email.smtp_oauth2.username, "noreply@contoso.com");
        assert_eq!(
            cfg.email.smtp_oauth2.vendor,
            SmtpOauth2Vendor::Microsoft
        );
        assert_eq!(
            cfg.email.smtp_oauth2.microsoft.tenant_id,
            "00000000-0000-0000-0000-000000000000"
        );
        assert_eq!(
            cfg.email.smtp_oauth2.microsoft.client_id,
            "11111111-1111-1111-1111-111111111111"
        );
        assert_eq!(
            cfg.email.smtp_oauth2.microsoft.scopes,
            vec!["https://outlook.office.com/SMTP.Send".to_string()]
        );
        // client_secret is env-only — never deserialised from TOML.
        assert!(cfg.email.smtp_oauth2.microsoft.client_secret.is_empty());
        // Both per-vendor sub-tables parse even when only one is active.
        assert_eq!(
            cfg.email.smtp_oauth2.google.service_account_json_path,
            "/etc/maze/gcp-service-account.json"
        );
        assert_eq!(
            cfg.email.smtp_oauth2.google.delegated_subject,
            "noreply@company.com"
        );
    }

    #[test]
    fn smtp_oauth2_defaults_apply_when_section_absent() {
        let env: Envelope = toml::from_str("").expect("parse");
        let cfg = env.comms;
        assert!(cfg.email.smtp_oauth2.host.is_empty());
        assert_eq!(cfg.email.smtp_oauth2.port, 587);
        assert_eq!(cfg.email.smtp_oauth2.tls, "starttls");
        assert!(cfg.email.smtp_oauth2.username.is_empty());
        assert_eq!(
            cfg.email.smtp_oauth2.vendor,
            SmtpOauth2Vendor::Microsoft
        );
        assert_eq!(
            cfg.email.smtp_oauth2.microsoft.scopes,
            vec!["https://outlook.office.com/SMTP.Send".to_string()]
        );
        assert_eq!(
            cfg.email.smtp_oauth2.google.scopes,
            vec!["https://www.googleapis.com/auth/gmail.send".to_string()]
        );
    }

    /// Single test for the Microsoft client_secret env var so the two
    /// states (unset / set) run sequentially. Splitting them across tests
    /// would race under `cargo test`'s default thread pool.
    #[test]
    fn resolve_and_validate_handles_smtp_oauth2_microsoft_client_secret_env_var() {
        let env_var = "MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_SECRET";
        let prior = std::env::var(env_var).ok();

        unsafe { std::env::remove_var(env_var) };
        let mut cfg = enabled_smtp_oauth2_microsoft_config();
        let result = cfg.resolve_and_validate().expect("universals satisfied");
        let joined = result.warnings.join("\n");
        assert!(joined.contains(env_var), "{joined}");
        assert!(cfg.email.smtp_oauth2.microsoft.client_secret.is_empty());

        unsafe { std::env::set_var(env_var, "test-resolved-secret") };
        let mut cfg = enabled_smtp_oauth2_microsoft_config();
        let result = cfg.resolve_and_validate().expect("universals satisfied");
        assert_eq!(
            cfg.email.smtp_oauth2.microsoft.client_secret,
            "test-resolved-secret"
        );
        let joined = result.warnings.join("\n");
        assert!(!joined.contains(env_var), "{joined}");

        match prior {
            Some(v) => unsafe { std::env::set_var(env_var, v) },
            None => unsafe { std::env::remove_var(env_var) },
        }
    }

    #[test]
    fn resolve_and_validate_does_not_read_microsoft_secret_when_google_flow_active() {
        let env_var = "MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_MICROSOFT_CLIENT_SECRET";
        let prior = std::env::var(env_var).ok();

        unsafe { std::env::set_var(env_var, "should-not-be-read") };
        let mut cfg = enabled_smtp_oauth2_google_config();
        let _ = cfg.resolve_and_validate();
        // Microsoft secret stays empty — the active flow is Google.
        assert!(cfg.email.smtp_oauth2.microsoft.client_secret.is_empty());

        match prior {
            Some(v) => unsafe { std::env::set_var(env_var, v) },
            None => unsafe { std::env::remove_var(env_var) },
        }
    }

    #[test]
    fn resolve_and_validate_warns_about_missing_smtp_oauth2_host_username_and_microsoft_fields() {
        let mut cfg = enabled_smtp_oauth2_microsoft_config();
        cfg.email.smtp_oauth2.host = String::new();
        cfg.email.smtp_oauth2.username = String::new();
        cfg.email.smtp_oauth2.microsoft.tenant_id = String::new();
        cfg.email.smtp_oauth2.microsoft.client_id = String::new();
        let result = cfg.resolve_and_validate().expect("universals satisfied");
        let joined = result.warnings.join("\n");
        assert!(joined.contains("host"), "{joined}");
        assert!(joined.contains("username"), "{joined}");
        assert!(joined.contains("tenant_id"), "{joined}");
        assert!(joined.contains("client_id"), "{joined}");
    }

    #[test]
    fn resolve_and_validate_warns_about_missing_google_service_account_json_path() {
        let mut cfg = enabled_smtp_oauth2_google_config();
        cfg.email.smtp_oauth2.google.service_account_json_path = String::new();
        let result = cfg.resolve_and_validate().expect("universals satisfied");
        let joined = result.warnings.join("\n");
        assert!(joined.contains("service_account_json_path"), "{joined}");
    }

    /// Build a builder seeded with the same defaults `AppConfig::load` uses
    /// for the comms keys, plus the supplied inline TOML.
    fn builder_with_defaults_and_toml(
        toml: &str,
    ) -> config::ConfigBuilder<config::builder::DefaultState> {
        config::Config::builder()
            .set_default("comms.enabled", default_comms_enabled())
            .unwrap()
            .set_default("comms.email.provider", "stub")
            .unwrap()
            .set_default(
                "comms.email.default_from_name",
                default_comms_email_default_from_name(),
            )
            .unwrap()
            .set_default(
                "comms.email.templates_dir",
                default_comms_email_templates_dir(),
            )
            .unwrap()
            .set_default(
                "comms.email.mailgun.region",
                default_comms_email_mailgun_region(),
            )
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

    /// Round-trip test for the smtp_oauth2-specific env vars. Uses a
    /// disjoint set of env vars from the other env-touching tests in this
    /// module so the suite stays safe under the default parallel thread
    /// pool.
    #[test]
    fn apply_env_overrides_propagates_smtp_oauth2_keys() {
        let toml = r#"
            [comms]
            enabled = true

            [comms.email]
            provider = "smtp_oauth2"

            [comms.email.smtp_oauth2]
            host = "from-toml.example.com"
            username = "from-toml@example.com"
        "#;
        with_env_vars(
            &[
                (
                    "MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_HOST",
                    "from-env.example.com",
                ),
                (
                    "MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_VENDOR",
                    "google",
                ),
                (
                    "MAZE_WEB_SERVER_COMMS_EMAIL_SMTP_OAUTH2_GOOGLE_DELEGATED_SUBJECT",
                    "delegate@example.com",
                ),
            ],
            || {
                let builder = builder_with_defaults_and_toml(toml);
                let builder = apply_env_overrides(builder).expect("apply env overrides");
                let settings = builder.build().expect("build");
                let cfg: CommsAppConfig = settings.get("comms").expect("deserialize");
                // env wins over TOML where both are present
                assert_eq!(cfg.email.smtp_oauth2.host, "from-env.example.com");
                // TOML wins over default where env is unset
                assert_eq!(cfg.email.smtp_oauth2.username, "from-toml@example.com");
                // env reaches into the discriminator
                assert_eq!(
                    cfg.email.smtp_oauth2.vendor,
                    SmtpOauth2Vendor::Google
                );
                // env reaches into per-vendor sub-tables
                assert_eq!(
                    cfg.email.smtp_oauth2.google.delegated_subject,
                    "delegate@example.com"
                );
                // serde defaults still apply for absent fields
                assert_eq!(cfg.email.smtp_oauth2.port, 587);
                assert_eq!(cfg.email.smtp_oauth2.tls, "starttls");
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

            [comms.email]
            provider = "stub"
            default_from = "noreply@from-toml.example.com"
            default_from_name = "Maze (TOML)"

            [comms.email.mailgun]
            domain = "from-toml.mg.example.com"
        "#;
        with_env_vars(
            &[
                ("MAZE_WEB_SERVER_COMMS_EMAIL_DEFAULT_FROM_NAME", "Maze (env)"),
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
                assert_eq!(cfg.email.default_from_name, "Maze (env)");
                assert_eq!(cfg.email.provider, CommsEmailProvider::Mailgun);
                assert_eq!(cfg.email.mailgun.domain, "from-env.mg.example.com");
                // TOML wins over default where env is unset
                assert_eq!(
                    cfg.email.default_from,
                    "noreply@from-toml.example.com"
                );
                // default wins where neither TOML nor env is set
                assert_eq!(cfg.email.mailgun.region, "us");
                assert_eq!(cfg.email.templates_dir, "config/email_templates");
            },
        );
    }
}
