//! `comms` — outbound communications crate.
//!
//! Provides an `EmailProvider` trait over a thin shared `Provider`
//! super-trait, typed `EmailMessage` / `EmailAddress` types, an error
//! taxonomy classified by transience, and a bounded-retry policy. The
//! per-medium trait pattern (one trait per medium over `Provider`) is the
//! intentional extension point — a new medium is one trait + one slot
//! on `Comms`, not a redesign.

pub mod email;
pub mod error;
pub mod oauth;
pub mod orchestrator;
pub mod provider;
pub mod providers;
pub mod retry;
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
#[cfg(feature = "oauth2-refresh-token")]
pub use oauth::{RefreshTokenConfig, RefreshTokenTokenSource};
pub use provider::{DeliveryReceipt, Provider};
#[cfg(feature = "provider-mailgun")]
pub use providers::{MailgunConfig, MailgunProvider, MailgunRegion};
#[cfg(feature = "provider-smtp-oauth2")]
pub use providers::{SmtpOAuth2Config, SmtpOAuth2Provider, SmtpTls};
pub use retry::RetryPolicy;
pub use template::{
    AppContext, BrandingContext, BrandingPartialSources, EmbeddedTemplateLoader, FsTemplateLoader,
    LayeredTemplateLoader, RenderedTemplate, TemplateContext, TemplateLoader, TemplateRenderer,
    TemplateSource,
};
