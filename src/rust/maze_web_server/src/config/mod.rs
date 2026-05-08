//!Application configuration
pub mod app;
pub mod comms;
pub use app::{
    AppConfig, AppFeaturesConfig, ConnectorKind, InternalConnectorConfig, InternalProviderConfig,
    OAuthConfig,
};
pub use comms::{
    CommsAppConfig, CommsBrandingConfig, CommsEmailConfig, CommsEmailProvider, CommsValidation,
    MailgunAppConfig,
};