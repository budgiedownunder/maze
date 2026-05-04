//! `comms` — outbound communications crate.
//!
//! Provides per-medium provider traits (`EmailProvider`, `SmsProvider`) over a thin
//! shared super-trait (`Provider`), typed message and recipient types, an error
//! taxonomy classified by transience, and a bounded-retry policy.

pub mod email;
pub mod error;
pub mod provider;
pub mod recipient;
pub mod retry;
pub mod sms;

pub use email::{EmailAddress, EmailMessage, EmailProvider};
pub use error::CommsError;
pub use provider::{DeliveryReceipt, Provider};
pub use recipient::Recipient;
pub use retry::RetryPolicy;
pub use sms::{PhoneNumber, SmsMessage, SmsProvider};
