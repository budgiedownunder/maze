//! Per-provider `EmailProvider` and `SmsProvider` implementations. Each
//! provider is gated behind its own `provider-*` feature flag so consumers
//! pull in only the transports they actually use.

#[cfg(feature = "provider-mailgun")]
pub mod mailgun;
#[cfg(feature = "provider-mailgun")]
pub use mailgun::{MailgunConfig, MailgunProvider, MailgunRegion};
