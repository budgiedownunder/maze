use auth::config::PasswordHashConfig;
use config::{Config, ConfigBuilder, File,  builder::DefaultState};
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security configuration including TLS certificate paths and password hashing parameters.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SecurityConfig {
    /// Path to the TLS certificate file.
    /// Can be overridden with `MAZE_WEB_SERVER_SECURITY_CERT_FILE`.
    #[serde(default = "default_security_cert_file")]
    pub cert_file: String,

    /// Path to the TLS private key file.
    /// Can be overridden with `MAZE_WEB_SERVER_SECURITY_KEY_FILE`.
    #[serde(default = "default_security_key_file")]
    pub key_file: String,

    /// Password hashing configuration (algorithm, iterations, etc).
    /// Typically defined only in the config file and not overridden via env.
    #[serde(default)]
    pub password_hash: PasswordHashConfig,    

    /// Login token expiry in hours.
    /// Typically defined only in the config file and not overridden via env.
    #[serde(default = "default_security_login_expiry_hours")]
    pub login_expiry_hours: u32,

}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            cert_file: default_security_cert_file(),
            key_file: default_security_key_file(),
            password_hash: PasswordHashConfig::default(),
            login_expiry_hours: default_security_login_expiry_hours(),
        }
    }
}

/// Logging configuration controlling log file output.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    /// Directory to write log files to (relative to the server working directory).
    /// Can be overridden with `MAZE_WEB_SERVER_LOGGING_LOG_DIR`.
    #[serde(default = "default_logging_log_dir")]
    pub log_dir: String,

    /// Minimum log level to capture.
    /// Valid values: `error`, `warn`, `info`, `debug`, `trace`.
    /// Can be overridden with `MAZE_WEB_SERVER_LOGGING_LOG_LEVEL`.
    #[serde(default = "default_logging_log_level")]
    pub log_level: String,

    /// Prefix used verbatim at the start of each log file name, including any separator.
    /// Log files are named `{log_file_prefix}{YYYY-MM-DD}.log`.
    /// Can be overridden with `MAZE_WEB_SERVER_LOGGING_LOG_FILE_PREFIX`.
    #[serde(default = "default_logging_log_file_prefix")]
    pub log_file_prefix: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            log_dir: default_logging_log_dir(),
            log_level: default_logging_log_level(),
            log_file_prefix: default_logging_log_file_prefix(),
        }
    }
}

/// Feature flags that apply to all users equally.
/// Controlled via the `[features]` section of `config.toml`, environment variables,
/// or the admin API (`PUT /api/v1/admin/features`).
///
/// For future per-user feature gating, a separate `UserFeaturesConfig` type
/// should be added (stored per user in the data store, not in `config.toml`).
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppFeaturesConfig {
    /// Whether new users can self-register via the signup endpoint.
    /// Can be overridden with `MAZE_WEB_SERVER_FEATURES_ALLOW_SIGNUP`.
    #[serde(default = "default_features_allow_signup")]
    pub allow_signup: bool,
}

impl Default for AppFeaturesConfig {
    fn default() -> Self {
        Self { allow_signup: default_features_allow_signup() }
    }
}

/// Selects which `OAuthConnector` implementation the server uses.
///
/// New connectors (e.g. `Auth0`) become valid values once their implementation
/// lands. `Internal` is the default and the only connector implemented today.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ConnectorKind {
    #[default]
    Internal,
    Auth0,
}

/// Configuration for one OAuth provider exposed by the internal connector.
///
/// `client_secret` is not read from `config.toml`; it is resolved at load time
/// from the environment variable named by `client_secret_env`. This keeps the
/// secret out of any committed file.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct InternalProviderConfig {
    /// Whether this provider is enabled. Disabled providers are not surfaced
    /// to the front end and the server will not initiate a flow against them.
    #[serde(default)]
    pub enabled: bool,

    /// Human-readable label rendered on the front-end button (e.g. "Google").
    #[serde(default)]
    pub display_name: String,

    /// OAuth/OIDC client id issued by the provider.
    #[serde(default)]
    pub client_id: String,

    /// Name of the environment variable that holds the client secret.
    /// e.g. "MAZE_OAUTH_GOOGLE_SECRET".
    #[serde(default)]
    pub client_secret_env: String,

    /// The server's own callback URL the provider redirects to. Must be
    /// registered in the provider's developer console.
    #[serde(default)]
    pub redirect_uri: String,

    /// Resolved client secret. Populated by `OAuthConfig::resolve_and_validate`
    /// from the env var named in `client_secret_env`. Never read from the toml
    /// file or written to the serialised form.
    #[serde(skip)]
    pub client_secret: String,
}

/// Configuration for the built-in OAuth connector that speaks OAuth/OIDC
/// directly to each provider.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct InternalConnectorConfig {
    /// Per-provider configuration keyed by canonical provider name
    /// ("google", "github", etc.).
    #[serde(default)]
    pub providers: HashMap<String, InternalProviderConfig>,
}

/// Placeholder configuration for the (future) Auth0 connector. Held as
/// `Option<Auth0ConnectorConfig>` on `OAuthConfig` so that the section can be
/// present in `config.toml` without forcing the connector to exist yet.
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Auth0ConnectorConfig {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret_env: String,
    #[serde(default)]
    pub audience: String,
    #[serde(skip)]
    pub client_secret: String,
}

/// OAuth configuration. The top-level `[oauth]` table selects which connector
/// implementation is used and is the entry point for everything else.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OAuthConfig {
    /// Master switch. When false, OAuth buttons are hidden in the front ends
    /// regardless of any per-provider config below, and no validation is run
    /// against the selected connector's section.
    #[serde(default = "default_oauth_enabled")]
    pub enabled: bool,

    /// Which connector implementation to use. "internal" is the default and
    /// the only one shipping today; "auth0" is reserved for a future drop-in.
    #[serde(default)]
    pub connector: ConnectorKind,

    /// URL scheme the MAUI app uses as its OAuth redirect target. Lives on
    /// the top-level [oauth] table because every connector ends the flow the
    /// same way: by handing the app a bearer token via this scheme.
    #[serde(default = "default_mobile_redirect_scheme")]
    pub mobile_redirect_scheme: String,

    /// Internal-connector configuration. Required when `enabled = true` and
    /// `connector = Internal`; ignored otherwise.
    #[serde(default)]
    pub internal: Option<InternalConnectorConfig>,

    /// Auth0-connector configuration placeholder. Not consumed by any code in
    /// v1; future connector implementation will read it.
    #[serde(default)]
    pub auth0: Option<Auth0ConnectorConfig>,
}

impl Default for OAuthConfig {
    fn default() -> Self {
        Self {
            enabled: default_oauth_enabled(),
            connector: ConnectorKind::default(),
            mobile_redirect_scheme: default_mobile_redirect_scheme(),
            internal: None,
            auth0: None,
        }
    }
}

impl OAuthConfig {
    /// Validates the config and resolves any environment-backed secrets into
    /// `client_secret` fields. Idempotent, so safe to call from tests.
    ///
    /// Errors when `enabled = true` and:
    /// - the section required by the selected connector is missing or has no
    ///   enabled providers, or
    /// - any enabled provider names a `client_secret_env` whose env var is not
    ///   set (a typo or missed deployment step that would otherwise show up
    ///   only on the first user sign-in attempt).
    ///
    /// When `enabled = false`, returns Ok without inspecting anything.
    pub fn resolve_and_validate(&mut self) -> Result<(), String> {
        if !self.enabled {
            return Ok(());
        }
        match self.connector {
            ConnectorKind::Internal => {
                let internal = self.internal.as_mut().ok_or_else(|| {
                    "[oauth] enabled = true with connector = \"internal\" requires an [oauth.internal] section with at least one enabled provider".to_string()
                })?;

                // Walk every enabled provider, collecting *all* issues. Reporting
                // one error at a time forces a fix-restart-fix-restart loop;
                // batching lets the operator see the full diagnostic in one shot.
                let mut issues: Vec<String> = Vec::new();
                let mut enabled_count = 0;
                for (name, provider) in internal.providers.iter_mut() {
                    if !provider.enabled {
                        continue;
                    }
                    enabled_count += 1;

                    if provider.client_id.trim().is_empty() {
                        issues.push(format!(
                            "[oauth.internal.providers.{name}] is enabled but client_id is empty"
                        ));
                    }
                    if provider.redirect_uri.trim().is_empty() {
                        issues.push(format!(
                            "[oauth.internal.providers.{name}] is enabled but redirect_uri is empty"
                        ));
                    }

                    // Resolve the secret. We always check whether the env var is
                    // populated, even if client_secret_env itself is empty — it
                    // is more useful for the operator to learn that the field
                    // is empty *and* (separately) that the named env var would
                    // also need setting, rather than chaining the diagnostic.
                    if provider.client_secret_env.trim().is_empty() {
                        issues.push(format!(
                            "[oauth.internal.providers.{name}] is enabled but client_secret_env is empty"
                        ));
                    } else {
                        match std::env::var(&provider.client_secret_env) {
                            Ok(secret) if secret.is_empty() => {
                                issues.push(format!(
                                    "[oauth.internal.providers.{name}] env var \"{}\" is set but empty",
                                    provider.client_secret_env
                                ));
                            }
                            Ok(secret) => {
                                provider.client_secret = secret;
                            }
                            Err(_) => {
                                issues.push(format!(
                                    "[oauth.internal.providers.{name}] client_secret_env=\"{}\" is not set in the environment",
                                    provider.client_secret_env
                                ));
                            }
                        }
                    }
                }

                if enabled_count == 0 {
                    return Err(
                        "[oauth] enabled = true with connector = \"internal\" requires at least one enabled provider in [oauth.internal.providers.*]"
                            .to_string(),
                    );
                }
                if !issues.is_empty() {
                    let mut msg = String::from("OAuth configuration is invalid:");
                    for issue in &issues {
                        msg.push_str("\n  - ");
                        msg.push_str(issue);
                    }
                    return Err(msg);
                }
            }
            ConnectorKind::Auth0 => {
                return Err(
                    "[oauth] connector = \"auth0\" is not yet implemented; use connector = \"internal\" or set [oauth].enabled = false"
                        .to_string(),
                );
            }
        }
        Ok(())
    }
}

/// Application configuration settings loaded from config.toml or environment variables.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    /// Port to bind the server to (e.g., 8443 for HTTPS).
    /// Can be overridden with `MAZE_WEB_SERVER_PORT`.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Security-related configuration such as TLS cert paths and password hashing policy.
    #[serde(default)]
    pub security: SecurityConfig,

    /// Logging configuration controlling log file output.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Path to the built React app dist directory (relative to the server working directory,
    /// or absolute). If the directory does not exist, the server runs as API-only.
    /// Can be overridden with `MAZE_WEB_SERVER_STATIC_DIR`.
    #[serde(default = "default_static_dir")]
    pub static_dir: String,

    /// Feature flags controlling which capabilities are available to end users.
    #[serde(default)]
    pub features: AppFeaturesConfig,

    /// OAuth configuration: connector selection plus per-connector settings.
    #[serde(default)]
    pub oauth: OAuthConfig,

    /// Path to the config file that was loaded. Not read from the config file itself —
    /// used by the admin API to persist runtime feature-flag changes back to disk.
    #[serde(skip, default = "default_config_path")]
    pub config_path: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: default_port(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            static_dir: default_static_dir(),
            features: AppFeaturesConfig::default(),
            oauth: OAuthConfig::default(),
            config_path: default_config_path(),
        }
    }
}


// Default values
fn default_port() -> u16 { 8443 }
fn default_static_dir() -> String { "static".to_string() }
fn default_config_path() -> String { "config.toml".to_string() }
fn default_security_cert_file() -> String { "cert.pem".to_string() }
fn default_security_key_file() -> String { "key.pem".to_string() }
fn default_security_login_expiry_hours() -> u32 { 24 }
fn default_logging_log_dir() -> String { "logs".to_string() }
fn default_logging_log_level() -> String { "info".to_string() }
fn default_logging_log_file_prefix() -> String { "maze_web_server_".to_string() }
fn default_features_allow_signup() -> bool { true }
fn default_oauth_enabled() -> bool { false }
fn default_mobile_redirect_scheme() -> String { "maze-app".to_string() }

/// Application Configuration
impl AppConfig {
     pub fn load() -> Result<Self, config::ConfigError> {
        let mut builder = Config::builder()
            .set_default("port", 8443)?
            .set_default("security.cert_file", default_security_cert_file())?
            .set_default("security.key_file", default_security_key_file())?
            .set_default("logging.log_dir", default_logging_log_dir())?
            .set_default("logging.log_level", default_logging_log_level())?
            .set_default("logging.log_file_prefix", default_logging_log_file_prefix())?
            .set_default("static_dir", default_static_dir())?
            .set_default("features.allow_signup", default_features_allow_signup())?
            .set_default("oauth.enabled", default_oauth_enabled())?
            .set_default("oauth.connector", "internal")?
            .set_default("oauth.mobile_redirect_scheme", default_mobile_redirect_scheme())?
            .add_source(File::with_name("config.toml").required(false));

        builder = set_env_overrides(builder)?;
        let settings = builder.build()?;
        let mut cfg: AppConfig = settings
            .try_deserialize()
            .or_else(|_| Ok::<_, config::ConfigError>(AppConfig::default()))?;
        cfg.oauth
            .resolve_and_validate()
            .map_err(config::ConfigError::Message)?;
        Ok(cfg)
    }

    /// Logs the configuration using the `log` crate at `info` level.
    pub fn log_config(&self) {
        match serde_json::to_string_pretty(self) {
            Ok(json) => {
                info!("Loaded AppConfig:\n{json}");
            }
            Err(err) => {
                log::error!("Failed to serialize AppConfig: {err}");
            }
        }
    }}

/// Moves environment variable overrides into a separate function
fn set_env_overrides(mut builder: ConfigBuilder<DefaultState>) -> Result<ConfigBuilder<DefaultState>, config::ConfigError> {
    if let Ok(port) = std::env::var(get_app_env_name("PORT")) {
        builder = builder.set_override("port", port)?;
    }

    if let Ok(cert_file) = std::env::var(get_app_env_name("SECURITY_CERT_FILE")) {
        builder = builder.set_override("security.cert_file", cert_file)?;
    }

    if let Ok(key_file) = std::env::var(get_app_env_name("SECURITY_KEY_FILE")) {
        builder = builder.set_override("security.key_file", key_file)?;
    }

    if let Ok(log_dir) = std::env::var(get_app_env_name("LOGGING_LOG_DIR")) {
        builder = builder.set_override("logging.log_dir", log_dir)?;
    }

    if let Ok(log_level) = std::env::var(get_app_env_name("LOGGING_LOG_LEVEL")) {
        builder = builder.set_override("logging.log_level", log_level)?;
    }

    if let Ok(log_file_prefix) = std::env::var(get_app_env_name("LOGGING_LOG_FILE_PREFIX")) {
        builder = builder.set_override("logging.log_file_prefix", log_file_prefix)?;
    }

    if let Ok(static_dir) = std::env::var(get_app_env_name("STATIC_DIR")) {
        builder = builder.set_override("static_dir", static_dir)?;
    }

    if let Ok(allow_signup) = std::env::var(get_app_env_name("FEATURES_ALLOW_SIGNUP")) {
        builder = builder.set_override("features.allow_signup", allow_signup)?;
    }

    if let Ok(enabled) = std::env::var(get_app_env_name("OAUTH_ENABLED")) {
        builder = builder.set_override("oauth.enabled", enabled)?;
    }

    if let Ok(connector) = std::env::var(get_app_env_name("OAUTH_CONNECTOR")) {
        builder = builder.set_override("oauth.connector", connector)?;
    }

    if let Ok(scheme) = std::env::var(get_app_env_name("OAUTH_MOBILE_REDIRECT_SCHEME")) {
        builder = builder.set_override("oauth.mobile_redirect_scheme", scheme)?;
    }

    Ok(builder)
}

/// Returns the applicaion environment name for a given setting
fn get_app_env_name(setting_name: &str) -> String {
    format!("MAZE_WEB_SERVER_{setting_name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logging_config_deserialises_from_toml() {
        let toml = r#"
            [logging]
            log_dir = "my_logs"
            log_level = "debug"
            log_file_prefix = "my-app-"
        "#;
        let cfg: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.logging.log_dir, "my_logs");
        assert_eq!(cfg.logging.log_level, "debug");
        assert_eq!(cfg.logging.log_file_prefix, "my-app-");
    }

    #[test]
    fn logging_config_uses_defaults_when_section_absent() {
        let cfg: AppConfig = toml::from_str("").unwrap();
        assert_eq!(cfg.logging.log_dir, "logs");
        assert_eq!(cfg.logging.log_level, "info");
        assert_eq!(cfg.logging.log_file_prefix, "maze_web_server_");
    }

    #[test]
    fn logging_config_custom_prefix_deserialises_from_toml() {
        let toml = r#"
            [logging]
            log_file_prefix = "my-app-"
        "#;
        let cfg: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.logging.log_file_prefix, "my-app-");
    }

    #[test]
    fn static_dir_deserialises_from_toml() {
        let toml = r#"static_dir = "../../react/maze_web_server/dist""#;
        let cfg: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.static_dir, "../../react/maze_web_server/dist");
    }

    #[test]
    fn static_dir_uses_default_when_absent() {
        let cfg: AppConfig = toml::from_str("").unwrap();
        assert_eq!(cfg.static_dir, "static");
    }

    #[test]
    fn app_features_config_deserialises_from_toml() {
        let toml = r#"
            [features]
            allow_signup = false
        "#;
        let cfg: AppConfig = toml::from_str(toml).unwrap();
        assert!(!cfg.features.allow_signup);
    }

    #[test]
    fn app_features_config_uses_defaults_when_section_absent() {
        let cfg: AppConfig = toml::from_str("").unwrap();
        assert!(cfg.features.allow_signup);
    }

    #[test]
    fn app_features_config_uses_defaults_when_field_absent() {
        let cfg: AppConfig = toml::from_str("[features]").unwrap();
        assert!(cfg.features.allow_signup);
    }

    #[test]
    fn config_path_is_not_read_from_toml() {
        // config_path is a meta field — it must never be populated from TOML
        let cfg: AppConfig = toml::from_str("").unwrap();
        assert_eq!(cfg.config_path, "config.toml");
    }

    // -------- OAuth config --------

    #[test]
    fn oauth_defaults_to_disabled_internal_when_section_absent() {
        let cfg: AppConfig = toml::from_str("").unwrap();
        assert!(!cfg.oauth.enabled);
        assert_eq!(cfg.oauth.connector, ConnectorKind::Internal);
        assert_eq!(cfg.oauth.mobile_redirect_scheme, "maze-app");
        assert!(cfg.oauth.internal.is_none());
    }

    #[test]
    fn oauth_connector_defaults_to_internal_when_omitted() {
        let toml = r#"
            [oauth]
            enabled = false
        "#;
        let cfg: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.oauth.connector, ConnectorKind::Internal);
    }

    #[test]
    fn oauth_connector_kind_deserialises_lowercase() {
        let toml = r#"
            [oauth]
            connector = "auth0"
        "#;
        let cfg: AppConfig = toml::from_str(toml).unwrap();
        assert_eq!(cfg.oauth.connector, ConnectorKind::Auth0);
    }

    #[test]
    fn oauth_internal_provider_section_deserialises() {
        let toml = r#"
            [oauth]
            enabled = true

            [oauth.internal.providers.google]
            enabled = true
            display_name = "Google"
            client_id = "google-client-id"
            client_secret_env = "MAZE_OAUTH_GOOGLE_SECRET_TEST_DESER"
            redirect_uri = "https://maze.example.com/api/v1/auth/oauth/google/callback"
        "#;
        let cfg: AppConfig = toml::from_str(toml).unwrap();
        let internal = cfg.oauth.internal.expect("internal section");
        let google = internal.providers.get("google").expect("google provider");
        assert!(google.enabled);
        assert_eq!(google.display_name, "Google");
        assert_eq!(google.client_id, "google-client-id");
        assert_eq!(google.client_secret_env, "MAZE_OAUTH_GOOGLE_SECRET_TEST_DESER");
        assert_eq!(
            google.redirect_uri,
            "https://maze.example.com/api/v1/auth/oauth/google/callback"
        );
        // client_secret is never read from toml
        assert!(google.client_secret.is_empty());
    }

    #[test]
    fn resolve_and_validate_is_noop_when_oauth_disabled() {
        let mut cfg = OAuthConfig::default();
        // No internal section at all — validation must still pass because
        // enabled is false.
        assert!(cfg.resolve_and_validate().is_ok());
    }

    #[test]
    fn resolve_and_validate_errors_when_internal_section_missing() {
        let mut cfg = OAuthConfig {
            enabled: true,
            connector: ConnectorKind::Internal,
            internal: None,
            ..OAuthConfig::default()
        };
        let err = cfg.resolve_and_validate().unwrap_err();
        assert!(err.contains("[oauth.internal]"), "error message should reference the missing section: {err}");
    }

    #[test]
    fn resolve_and_validate_errors_when_no_enabled_providers() {
        let mut providers = HashMap::new();
        providers.insert(
            "google".to_string(),
            InternalProviderConfig {
                enabled: false,
                display_name: "Google".to_string(),
                client_id: "id".to_string(),
                client_secret_env: "X".to_string(),
                redirect_uri: "https://example.com/cb".to_string(),
                client_secret: String::new(),
            },
        );
        let mut cfg = OAuthConfig {
            enabled: true,
            connector: ConnectorKind::Internal,
            internal: Some(InternalConnectorConfig { providers }),
            ..OAuthConfig::default()
        };
        let err = cfg.resolve_and_validate().unwrap_err();
        assert!(err.contains("at least one enabled provider"), "got: {err}");
    }

    #[test]
    fn resolve_and_validate_errors_when_secret_env_var_unset() {
        let env_var = "MAZE_OAUTH_TEST_UNSET_SECRET_VAR_DO_NOT_SET";
        unsafe { std::env::remove_var(env_var); }
        let mut providers = HashMap::new();
        providers.insert(
            "google".to_string(),
            InternalProviderConfig {
                enabled: true,
                display_name: "Google".to_string(),
                client_id: "id".to_string(),
                client_secret_env: env_var.to_string(),
                redirect_uri: "https://example.com/cb".to_string(),
                client_secret: String::new(),
            },
        );
        let mut cfg = OAuthConfig {
            enabled: true,
            connector: ConnectorKind::Internal,
            internal: Some(InternalConnectorConfig { providers }),
            ..OAuthConfig::default()
        };
        let err = cfg.resolve_and_validate().unwrap_err();
        assert!(err.contains(env_var), "error should name the env var: {err}");
    }

    #[test]
    fn resolve_and_validate_resolves_secret_from_env_var() {
        let env_var = "MAZE_OAUTH_TEST_RESOLVE_SECRET_VAR";
        unsafe { std::env::set_var(env_var, "the-resolved-secret"); }
        let mut providers = HashMap::new();
        providers.insert(
            "google".to_string(),
            InternalProviderConfig {
                enabled: true,
                display_name: "Google".to_string(),
                client_id: "id".to_string(),
                client_secret_env: env_var.to_string(),
                redirect_uri: "https://example.com/cb".to_string(),
                client_secret: String::new(),
            },
        );
        let mut cfg = OAuthConfig {
            enabled: true,
            connector: ConnectorKind::Internal,
            internal: Some(InternalConnectorConfig { providers }),
            ..OAuthConfig::default()
        };
        cfg.resolve_and_validate().expect("should resolve cleanly");
        let google = cfg.internal.unwrap().providers.remove("google").unwrap();
        assert_eq!(google.client_secret, "the-resolved-secret");
        unsafe { std::env::remove_var(env_var); }
    }

    #[test]
    fn resolve_and_validate_errors_for_auth0_connector() {
        let mut cfg = OAuthConfig {
            enabled: true,
            connector: ConnectorKind::Auth0,
            ..OAuthConfig::default()
        };
        let err = cfg.resolve_and_validate().unwrap_err();
        assert!(err.contains("auth0"), "got: {err}");
        assert!(err.contains("not yet implemented"), "got: {err}");
    }

    #[test]
    fn resolve_and_validate_reports_all_issues_in_one_error() {
        // The case the operator typically hits: they enabled a provider but
        // never filled in client_id / redirect_uri / set the secret env var.
        // The error must name every problem so they can fix them in one pass.
        let env_var = "MAZE_OAUTH_TEST_BATCH_SECRET_DO_NOT_SET";
        unsafe { std::env::remove_var(env_var); }
        let mut providers = HashMap::new();
        providers.insert(
            "google".to_string(),
            InternalProviderConfig {
                enabled: true,
                display_name: "Google".to_string(),
                client_id: String::new(),
                client_secret_env: env_var.to_string(),
                redirect_uri: String::new(),
                client_secret: String::new(),
            },
        );
        let mut cfg = OAuthConfig {
            enabled: true,
            connector: ConnectorKind::Internal,
            internal: Some(InternalConnectorConfig { providers }),
            ..OAuthConfig::default()
        };
        let err = cfg.resolve_and_validate().unwrap_err();
        assert!(err.contains("client_id is empty"), "missing client_id error: {err}");
        assert!(err.contains("redirect_uri is empty"), "missing redirect_uri error: {err}");
        assert!(err.contains(env_var), "missing env var name in error: {err}");
        assert!(err.contains("is not set in the environment"), "missing env-unset diagnostic: {err}");
    }

    #[test]
    fn resolve_and_validate_errors_when_enabled_provider_has_empty_client_id() {
        let mut providers = HashMap::new();
        providers.insert(
            "google".to_string(),
            InternalProviderConfig {
                enabled: true,
                display_name: "Google".to_string(),
                client_id: String::new(),
                client_secret_env: "X".to_string(),
                redirect_uri: "https://example.com/cb".to_string(),
                client_secret: String::new(),
            },
        );
        let mut cfg = OAuthConfig {
            enabled: true,
            connector: ConnectorKind::Internal,
            internal: Some(InternalConnectorConfig { providers }),
            ..OAuthConfig::default()
        };
        let err = cfg.resolve_and_validate().unwrap_err();
        assert!(err.contains("client_id is empty"), "got: {err}");
    }
}