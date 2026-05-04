use serde::{Deserialize, Serialize};

use crate::email::EmailAddress;
use crate::sms::PhoneNumber;

/// The destination of a send. Each variant is dispatched to its matching
/// per-medium provider (`EmailProvider` for `Email`, `SmsProvider` for `Sms`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recipient {
    Email(EmailAddress),
    Sms(PhoneNumber),
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn email_recipient_round_trips_through_serde_json() {
        let r = Recipient::Email(EmailAddress::with_name("alice@example.com", "Alice"));
        let json = serde_json::to_string(&r).expect("serialize");
        let round_trip: Recipient = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(r, round_trip);
    }

    #[test]
    fn sms_recipient_round_trips_through_serde_json() {
        let r = Recipient::Sms(PhoneNumber::new("+15555550199"));
        let json = serde_json::to_string(&r).expect("serialize");
        let round_trip: Recipient = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(r, round_trip);
    }
}
