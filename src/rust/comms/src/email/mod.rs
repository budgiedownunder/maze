pub mod message;

pub use message::EmailMessage;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::CommsError;
use crate::provider::{DeliveryReceipt, Provider};

/// An email recipient or sender address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailAddress {
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl EmailAddress {
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            display_name: None,
        }
    }

    pub fn with_name(address: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            display_name: Some(display_name.into()),
        }
    }
}

/// Trait implemented by email-medium providers. The provider receives a fully
/// rendered `EmailMessage` and returns a `DeliveryReceipt` on success or a
/// `CommsError` (classified as transient or permanent) on failure.
#[async_trait]
pub trait EmailProvider: Provider {
    async fn send_email(&self, msg: &EmailMessage) -> Result<DeliveryReceipt, CommsError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn email_address_round_trips_through_serde_json() {
        let addr = EmailAddress::with_name("alice@example.com", "Alice Example");
        let json = serde_json::to_string(&addr).expect("serialize");
        let round_trip: EmailAddress = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(addr, round_trip);
    }

    #[test]
    fn email_address_omits_display_name_when_none() {
        let addr = EmailAddress::new("alice@example.com");
        let json = serde_json::to_string(&addr).expect("serialize");
        assert!(!json.contains("display_name"));
    }
}
