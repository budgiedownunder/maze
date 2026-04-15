use auth::config::PasswordHashConfig;
use config::{Config, ConfigBuilder, File,  builder::DefaultState};
use log::info;
use serde::{Deserialize, Serialize};

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
            .add_source(File::with_name("config.toml").required(false));

        builder = set_env_overrides(builder)?;
        let settings = builder.build()?;
        settings.try_deserialize().or_else(|_| Ok(AppConfig::default()))
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
}