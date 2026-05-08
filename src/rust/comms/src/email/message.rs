use serde::{Deserialize, Serialize};

use crate::email::EmailAddress;

/// A fully-rendered email message ready for an `EmailProvider` to dispatch.
/// Provider impls translate the fields into their wire format.
///
/// Visible content (subject, body, branding markup like logos/headers/footers)
/// belongs in `subject`, `body_text`, and `body_html`. The `headers` field is
/// for protocol-level metadata only — see its field-level docs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EmailMessage {
    pub from: EmailAddress,
    pub to: Vec<EmailAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<EmailAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub bcc: Vec<EmailAddress>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<EmailAddress>,
    pub subject: String,
    pub body_text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body_html: Option<String>,
    /// RFC 5322 / SMTP wire headers — protocol-level metadata such as
    /// `Message-ID`, `List-Unsubscribe`, or custom `X-*` tracking tags.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<(String, String)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn sample_message() -> EmailMessage {
        EmailMessage {
            from: EmailAddress::with_name("noreply@example.com", "Maze"),
            to: vec![EmailAddress::with_name("alice@example.com", "Alice Example")],
            cc: vec![EmailAddress::new("audit@example.com")],
            bcc: vec![],
            reply_to: Some(EmailAddress::new("support@example.com")),
            subject: "Reset your password".into(),
            body_text: "Hi Alice, click https://example.com/reset?token=...".into(),
            body_html: Some("<p>Hi Alice, <a href=\"...\">click here</a></p>".into()),
            headers: vec![("X-Maze-Template".into(), "password_reset".into())],
            idempotency_key: Some("reset-abc123".into()),
        }
    }

    #[test]
    fn email_message_round_trips_through_serde_json() {
        let msg = sample_message();
        let json = serde_json::to_string(&msg).expect("serialize");
        let round_trip: EmailMessage = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(msg, round_trip);
    }

    #[test]
    fn email_message_omits_empty_optional_fields() {
        let minimal = EmailMessage {
            from: EmailAddress::new("noreply@example.com"),
            to: vec![EmailAddress::new("alice@example.com")],
            cc: vec![],
            bcc: vec![],
            reply_to: None,
            subject: "hi".into(),
            body_text: "hi".into(),
            body_html: None,
            headers: vec![],
            idempotency_key: None,
        };
        let json = serde_json::to_string(&minimal).expect("serialize");
        assert!(!json.contains("\"cc\""));
        assert!(!json.contains("\"bcc\""));
        assert!(!json.contains("\"reply_to\""));
        assert!(!json.contains("\"body_html\""));
        assert!(!json.contains("\"headers\""));
        assert!(!json.contains("\"idempotency_key\""));
    }
}
