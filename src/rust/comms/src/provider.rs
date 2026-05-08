use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::retry::RetryPolicy;

/// Cross-cutting concerns shared by every provider, regardless of medium.
///
/// Per-medium traits (currently `EmailProvider`) extend `Provider` and add
/// the typed `send_*` method for their message shape. This split lets the
/// type system prevent wiring a provider into the wrong medium's slot.
pub trait Provider: Send + Sync {
    /// Stable identifier used in logs and metrics labels (e.g. `"mailgun"`).
    fn name(&self) -> &'static str;

    /// Retry policy applied to a single send when the provider returns a
    /// transient error.
    fn retry_policy(&self) -> &RetryPolicy;
}

/// Receipt returned by a successful `send_*`. `provider_message_id` is
/// best-effort — some providers return one in the response body, others don't.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    pub provider: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_message_id: Option<String>,
    pub accepted_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn delivery_receipt_round_trips_through_serde_json() {
        let receipt = DeliveryReceipt {
            provider: "mailgun".into(),
            provider_message_id: Some("<20260504.abc@mg.example.com>".into()),
            accepted_at: DateTime::<Utc>::from_timestamp(1_714_823_400, 0).expect("valid ts"),
        };
        let json = serde_json::to_string(&receipt).expect("serialize");
        let round_trip: DeliveryReceipt = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(receipt, round_trip);
    }

    #[test]
    fn delivery_receipt_omits_message_id_when_none() {
        let receipt = DeliveryReceipt {
            provider: "stub".into(),
            provider_message_id: None,
            accepted_at: DateTime::<Utc>::from_timestamp(0, 0).expect("valid ts"),
        };
        let json = serde_json::to_string(&receipt).expect("serialize");
        assert!(!json.contains("provider_message_id"));
    }
}
