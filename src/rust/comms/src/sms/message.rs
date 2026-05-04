use serde::{Deserialize, Serialize};

use crate::sms::PhoneNumber;

/// A fully-rendered SMS message ready for an `SmsProvider` to dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmsMessage {
    pub from: PhoneNumber,
    pub to: PhoneNumber,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn sms_message_round_trips_through_serde_json() {
        let msg = SmsMessage {
            from: PhoneNumber::new("+15550001111"),
            to: PhoneNumber::new("+15555550199"),
            body: "Maze: reset your password — https://example.com/r/abc".into(),
            idempotency_key: Some("reset-abc123".into()),
        };
        let json = serde_json::to_string(&msg).expect("serialize");
        let round_trip: SmsMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(msg, round_trip);
    }

    #[test]
    fn sms_message_omits_idempotency_key_when_none() {
        let msg = SmsMessage {
            from: PhoneNumber::new("+15550001111"),
            to: PhoneNumber::new("+15555550199"),
            body: "hi".into(),
            idempotency_key: None,
        };
        let json = serde_json::to_string(&msg).expect("serialize");
        assert!(!json.contains("idempotency_key"));
    }
}
