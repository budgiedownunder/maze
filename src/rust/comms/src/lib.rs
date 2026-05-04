//! `comms` — outbound communications crate.
//!
//! Provides per-medium provider traits (`EmailProvider`, `SmsProvider`) over a thin
//! shared super-trait (`Provider`), typed message and recipient types, an error
//! taxonomy classified by transience, and a bounded-retry policy.

pub mod email;
pub mod error;
pub mod oauth;
pub mod orchestrator;
pub mod provider;
pub mod recipient;
pub mod retry;
pub mod sms;
pub mod template;

pub use email::{EmailAddress, EmailMessage, EmailProvider};
#[cfg(feature = "stub")]
pub use email::StubEmailProvider;
pub use error::CommsError;
pub use oauth::{Clock, OAuthTokenSource, RefreshToken, RefreshTokenStore, SystemClock};
pub use orchestrator::Comms;
#[cfg(feature = "oauth2-microsoft")]
pub use oauth::{ClientCredentialsConfig, ClientCredentialsTokenSource};
#[cfg(feature = "oauth2-google")]
pub use oauth::{ServiceAccountConfig, ServiceAccountTokenSource};
pub use provider::{DeliveryReceipt, Provider};
pub use recipient::Recipient;
pub use retry::RetryPolicy;
pub use sms::{PhoneNumber, SmsMessage, SmsProvider};
#[cfg(feature = "stub")]
pub use sms::StubSmsProvider;
pub use template::{
    AppContext, BrandingContext, Channel, EmbeddedTemplateLoader, FsTemplateLoader,
    LayeredTemplateLoader, RenderedTemplate, TemplateContext, TemplateLoader, TemplateRenderer,
    TemplateSource,
};
