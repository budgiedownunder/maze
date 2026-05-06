//! Per-provider `EmailProvider` implementations. Each provider is gated
//! behind its own `provider-*` feature flag so consumers pull in only
//! the transports they actually use.

#[cfg(feature = "provider-mailgun")]
pub mod mailgun;
#[cfg(feature = "provider-mailgun")]
pub use mailgun::{MailgunConfig, MailgunProvider, MailgunRegion};

#[cfg(feature = "provider-smtp-oauth2")]
pub mod smtp_oauth2;
#[cfg(feature = "provider-smtp-oauth2")]
pub use smtp_oauth2::{SmtpOAuth2Config, SmtpOAuth2Provider, SmtpTls};
