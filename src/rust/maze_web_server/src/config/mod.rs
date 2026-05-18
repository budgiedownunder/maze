//!Application configuration
pub mod app;
pub mod comms;
pub mod game;
pub use app::{
    AppConfig, AppFeaturesConfig, ConnectorKind, InternalConnectorConfig, InternalProviderConfig,
    OAuthConfig,
};
pub use comms::{
    CommsAppConfig, CommsBrandingConfig, CommsEmailConfig, CommsEmailProvider, CommsValidation,
    MailgunAppConfig,
};
pub use game::{GameConfig, Play3dConfig, Play3dDifficultyConfig};