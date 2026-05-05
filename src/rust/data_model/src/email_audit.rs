use crate::{
    Error,
    wrappers::{generate_now, generate_uuid},
};
use chrono::{DateTime, SubsecRound, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Final state of an email send attempt. Backends serialise the variant as
/// the snake_case strings shown in the comments below — matching the values
/// written into the SQL `outcome` column so FileStore JSON files and SQL
/// rows agree on the wire format. Unknown variants are rejected by serde
/// (`deny_unknown_fields` on the row, plus the absence of a `serde(other)`
/// fallback here) so a corrupt stored value surfaces immediately rather
/// than silently round-tripping.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum AuditOutcome {
    /// Row inserted before the send attempt; not yet resolved.
    Pending,
    /// Provider accepted the message. `provider_message_id` populated.
    Accepted,
    /// Send failed. `error_class` populated with a coarse taxonomy.
    Failed,
}

/// One audit log entry — recorded synchronously *before* every send attempt
/// (`outcome = Pending`) and updated to `Accepted` or `Failed` once the
/// provider resolves. Captures *intent and authorization*; provider-side
/// delivery telemetry is collected separately and ingested via webhook
/// (out of scope for this plan).
///
/// **Never store the rendered body or any expansion containing a secret**
/// (reset link, invite link, verification link). The audit log records
/// `template_id` and `token_id`, which are sufficient to answer "did we
/// send X to user Y?" without making the log itself a credential cache.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct EmailAuditEntry {
    #[schema(value_type = String)]
    /// Audit row id.
    pub id: Uuid,
    /// When the row was first inserted (the `pending` write).
    pub created_at: DateTime<Utc>,
    #[schema(value_type = Option<String>)]
    /// Recipient user id. `None` only when the recipient email did not
    /// match any user (anti-enumeration reset path).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient_user_id: Option<Uuid>,
    /// Recipient email address as sent. Stored verbatim so the audit row
    /// remains useful after the user_emails row is freed.
    pub recipient_email: String,
    /// Which template was rendered. Free-form at the schema level; the
    /// application maps to known ids (`password_reset`, `invitation`,
    /// `email_verification`, ...).
    pub template_id: String,
    #[schema(value_type = Option<String>)]
    /// Linked one-time token, if any. `None` for templates that don't
    /// carry a token (future expansion).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_id: Option<Uuid>,
    #[schema(value_type = Option<String>)]
    /// User who triggered the send. For self-service flows the recipient
    /// *is* the trigger, in which case `triggered_by_user_id` either
    /// matches `recipient_user_id` (admin-on-behalf-of paths) or is
    /// `None` (the user triggered their own request). Distinguishing
    /// staff-initiated sends from self-service is the column's main
    /// purpose; it also adds a forensics dimension when querying for
    /// abuse patterns ("which trigger user issued these reset
    /// requests?").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triggered_by_user_id: Option<Uuid>,
    /// Provider name (e.g. `mailgun`, `msgraph`, `stub`).
    pub provider: String,
    /// Provider-side message id, populated on `accepted` outcome. `None`
    /// otherwise — the send may have failed before the provider returned
    /// an id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_message_id: Option<String>,
    /// Resolved outcome of the send.
    pub outcome: AuditOutcome,
    /// Coarse error taxonomy populated when `outcome = Failed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_class: Option<String>,
}

impl EmailAuditEntry {
    /// Builds a fresh row with `outcome = Pending`. `created_at` is
    /// truncated to millisecond precision so the in-memory value
    /// round-trips bit-exactly through both backends (`SqlStore` writes
    /// `to_rfc3339_opts(Millis, true)`).
    pub fn new_pending(
        recipient_user_id: Option<Uuid>,
        recipient_email: &str,
        template_id: &str,
        token_id: Option<Uuid>,
        triggered_by_user_id: Option<Uuid>,
        provider: &str,
    ) -> EmailAuditEntry {
        EmailAuditEntry {
            id: generate_uuid(),
            created_at: generate_now().trunc_subsecs(3),
            recipient_user_id,
            recipient_email: recipient_email.to_string(),
            template_id: template_id.to_string(),
            token_id,
            triggered_by_user_id,
            provider: provider.to_string(),
            provider_message_id: None,
            outcome: AuditOutcome::Pending,
            error_class: None,
        }
    }

    /// Generates the JSON representation of the audit row.
    pub fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }

    /// Initialises a row by reading a JSON string.
    pub fn from_json(&mut self, json: &str) -> Result<(), Error> {
        let temp: EmailAuditEntry = serde_json::from_str(json)?;
        *self = temp;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn pending_row() -> EmailAuditEntry {
        EmailAuditEntry::new_pending(
            Some(Uuid::new_v4()),
            "alice@example.com",
            "password_reset",
            Some(Uuid::new_v4()),
            None,
            "stub",
        )
    }

    #[test]
    fn new_pending_starts_in_pending_state() {
        let row = pending_row();
        assert_eq!(row.outcome, AuditOutcome::Pending);
        assert!(row.provider_message_id.is_none());
        assert!(row.error_class.is_none());
        assert_ne!(row.id, Uuid::nil());
    }

    #[test]
    fn round_trips_through_json() {
        let row = pending_row();
        let json = row.to_json().expect("serialize");
        let mut back = pending_row();
        back.from_json(&json).expect("deserialize");
        assert_eq!(back, row);
    }

    #[test]
    fn round_trips_with_accepted_outcome() {
        let mut row = pending_row();
        row.outcome = AuditOutcome::Accepted;
        row.provider_message_id = Some("provider-123".to_string());
        let json = row.to_json().expect("serialize");
        let mut back = pending_row();
        back.from_json(&json).expect("deserialize");
        assert_eq!(back, row);
        assert_eq!(back.outcome, AuditOutcome::Accepted);
        assert_eq!(back.provider_message_id.as_deref(), Some("provider-123"));
    }

    #[test]
    fn round_trips_with_failed_outcome() {
        let mut row = pending_row();
        row.outcome = AuditOutcome::Failed;
        row.error_class = Some("provider_unavailable".to_string());
        let json = row.to_json().expect("serialize");
        let mut back = pending_row();
        back.from_json(&json).expect("deserialize");
        assert_eq!(back, row);
    }

    #[test]
    fn outcome_serialises_as_snake_case() {
        // Lock in the wire-format strings so FileStore JSON files and SQL
        // `outcome` rows agree across backends.
        let mut row = pending_row();
        let json = row.to_json().expect("serialize");
        assert!(json.contains("\"outcome\":\"pending\""), "{json}");

        row.outcome = AuditOutcome::Accepted;
        let json = row.to_json().expect("serialize");
        assert!(json.contains("\"outcome\":\"accepted\""), "{json}");

        row.outcome = AuditOutcome::Failed;
        let json = row.to_json().expect("serialize");
        assert!(json.contains("\"outcome\":\"failed\""), "{json}");
    }

    #[test]
    fn unknown_outcome_variant_is_rejected_strict() {
        let bogus = r#"{"id":"00000000-0000-0000-0000-000000000000","created_at":"2026-05-05T00:00:00Z","recipient_email":"x@y.com","template_id":"t","provider":"p","outcome":"delivered"}"#;
        let mut row = pending_row();
        let err = row
            .from_json(bogus)
            .expect_err("unknown outcome variant must fail");
        assert!(format!("{err}").contains("unknown variant"), "got {err}");
    }

    #[test]
    fn unknown_field_is_rejected_strict() {
        let bogus = r#"{"id":"00000000-0000-0000-0000-000000000000","created_at":"2026-05-05T00:00:00Z","recipient_email":"x@y.com","template_id":"t","provider":"p","outcome":"pending","mystery":1}"#;
        let mut row = pending_row();
        let err = row
            .from_json(bogus)
            .expect_err("unknown field must fail");
        assert!(format!("{err}").contains("unknown field"), "got {err}");
    }

    #[test]
    fn anti_enumeration_row_omits_recipient_user_id() {
        let row = EmailAuditEntry::new_pending(
            None,
            "ghost@example.com",
            "password_reset",
            None,
            None,
            "stub",
        );
        let json = row.to_json().expect("serialize");
        assert!(!json.contains("\"recipient_user_id\""), "{json}");
        let mut back = EmailAuditEntry::new_pending(
            Some(Uuid::new_v4()),
            "x@y.com",
            "x",
            None,
            None,
            "stub",
        );
        back.from_json(&json).expect("deserialize");
        assert!(back.recipient_user_id.is_none());
    }
}
