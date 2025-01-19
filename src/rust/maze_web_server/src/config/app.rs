use config::{Config, ConfigBuilder, File,  builder::DefaultState};
use serde::Deserialize;

// Security Configuration settings
#[derive(Debug, Deserialize, Default, Clone)]
pub struct SecurityConfig {
    #[serde(default = "default_security_cert_file")]
    pub cert_file: String,

    #[serde(default = "default_security_key_file")]
    pub key_file: String,

    #[serde(default = "default_security_auth_token")]
    pub auth_token: String,

}

///  Application Configuration settings
#[derive(Debug, Deserialize, Default, Clone)]
pub struct AppConfig {
    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default)]
    pub security: SecurityConfig,
}

// Default values
fn default_port() -> u16 { 8443 }
fn default_security_cert_file() -> String { "cert.pem".to_string() }
fn default_security_key_file() -> String { "key.pem".to_string() }
fn default_security_auth_token() -> String { "".to_string() }

/// Application Configuration
impl AppConfig {
     pub fn load() -> Result<Self, config::ConfigError> {
        let mut builder = Config::builder()
            .set_default("port", 8443)?
            .set_default("security.cert_file", default_security_cert_file())?
            .set_default("security.key_file", default_security_key_file())?
            .set_default("security.auth_token", default_security_auth_token())?
            .add_source(File::with_name("config.toml").required(false));

        builder = set_env_overrides(builder)?;
        let settings = builder.build()?;
        settings.try_deserialize().or_else(|_| Ok(AppConfig::default()))
    }
}

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

    if let Ok(auth_token) = std::env::var(get_app_env_name("SECURITY_AUTH_TOKEN")) {
        builder = builder.set_override("security.auth_token", auth_token)?;
    }

    Ok(builder)
}

/// Returns the applicaion environment name for a given setting 
fn get_app_env_name(setting_name: &str) -> String {
    return format!("MAZE_WEB_SERVER_{}", setting_name);
}