pub mod message;
#[cfg(feature = "stub")]
pub mod stub;

pub use message::SmsMessage;
#[cfg(feature = "stub")]
pub use stub::StubSmsProvider;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::CommsError;
use crate::provider::{DeliveryReceipt, Provider};

/// E.164-formatted phone number. The wrapper is a marker; validation is
/// the construction site's responsibility.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PhoneNumber(pub String);

impl PhoneNumber {
    pub fn new(e164: impl Into<String>) -> Self {
        Self(e164.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Trait implemented by SMS-medium providers. Sits alongside `EmailProvider`
/// over the same `Provider` super-trait so per-medium routing can dispatch
/// to a single typed slot.
#[async_trait]
pub trait SmsProvider: Provider {
    async fn send_sms(&self, msg: &SmsMessage) -> Result<DeliveryReceipt, CommsError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn phone_number_round_trips_through_serde_json() {
        let phone = PhoneNumber::new("+15551234567");
        let json = serde_json::to_string(&phone).expect("serialize");
        let round_trip: PhoneNumber = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(phone, round_trip);
        assert_eq!(round_trip.as_str(), "+15551234567");
    }
}
