use crate::{
    Error,
    wrappers::{generate_now, generate_uuid},
};
use chrono::{DateTime, Duration, SubsecRound, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// What flow this one-time token enables. Storage backends serialise the
/// variant as the kebab-case strings shown in the discriminant comments
/// below — matching the values written into the SQL `purpose` column so
/// FileStore JSON files and SQL rows agree on the wire format.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Eq, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TokenPurpose {
    /// Forgot-password reset flow. 1 hour expiry. Consumption invalidates
    /// every entry in `User.logins`.
    PasswordReset,
    /// Admin-initiated invitation flow. 7 day expiry. Acceptance sets the
    /// invited email to `verified = true, verified_at = now()`.
    Invite,
    /// Self-service or signup-time email verification. 24 hour expiry.
    /// Re-issuing supersedes any outstanding token for the same address.
    EmailVerification,
}

/// A single-use, time-bounded token used to authorise an out-of-band
/// action — password reset, invitation acceptance, or email verification.
///
/// Lifecycle:
///   * Created by a handler that has authenticated the *intent* of the
///     action (e.g. a `POST /password-reset/request` handler that found a
///     verified email match).
///   * Sent to the user via email; the `id` is the secret carried in the
///     reset/verify/invite link.
///   * Consumed exactly once when the link is followed: `consumed_at` is
///     atomically populated. A second consume attempt fails.
///   * Expires after the per-purpose TTL; expired tokens are invisible to
///     `find_token` and removed wholesale by `purge_expired`.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct OneTimeToken {
    #[schema(value_type = String)]
    /// Token id — also the secret carried in the link sent to the user.
    /// 122 bits of entropy via `Uuid::new_v4()`.
    pub id: Uuid,
    #[schema(value_type = String)]
    /// User the token was issued for.
    pub user_id: Uuid,
    /// Which flow this token enables.
    pub purpose: TokenPurpose,
    /// For [`TokenPurpose::EmailVerification`]: the specific `user_emails`
    /// row to flip on consumption. `None` for the password-reset and
    /// invitation flows.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_email: Option<String>,
    /// When the token was issued.
    pub created_at: DateTime<Utc>,
    /// When the token expires. After this instant `find_token` returns
    /// `not-found` and `consume_token` rejects.
    pub expires_at: DateTime<Utc>,
    /// When the token was consumed, if it has been. `None` for an active,
    /// unconsumed token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub consumed_at: Option<DateTime<Utc>>,
}

impl OneTimeToken {
    /// Creates a fresh, unconsumed token for the given user + purpose, with
    /// `expires_at = now() + expiry_hours`.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::{OneTimeToken, TokenPurpose};
    /// use uuid::Uuid;
    ///
    /// let user_id = Uuid::new_v4();
    /// let token = OneTimeToken::new(user_id, TokenPurpose::PasswordReset, None, 1);
    /// assert_eq!(token.user_id, user_id);
    /// assert_eq!(token.purpose, TokenPurpose::PasswordReset);
    /// assert!(token.consumed_at.is_none());
    /// assert!(token.expires_at > token.created_at);
    /// ```
    pub fn new(
        user_id: Uuid,
        purpose: TokenPurpose,
        target_email: Option<String>,
        expiry_hours: u32,
    ) -> OneTimeToken {
        // Truncate to millisecond precision so the in-memory value
        // round-trips bit-exactly through the SqlStore backend (whose
        // canonical timestamp format is `to_rfc3339_opts(Millis, true)`)
        // and the FileStore backend (which uses the same precision via
        // `generate_now_millis`). Without truncation the SqlStore round
        // trip silently drops sub-millisecond digits.
        let now = generate_now().trunc_subsecs(3);
        OneTimeToken {
            id: generate_uuid(),
            user_id,
            purpose,
            target_email,
            created_at: now,
            expires_at: now + Duration::hours(expiry_hours.into()),
            consumed_at: None,
        }
    }

    /// Returns true if `now > expires_at`.
    pub fn is_expired(&self) -> bool {
        generate_now() > self.expires_at
    }

    /// Returns true if `consumed_at` is populated.
    pub fn is_consumed(&self) -> bool {
        self.consumed_at.is_some()
    }

    /// Generates the JSON string representation for the token.
    pub fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }

    /// Initialises a token instance by reading a JSON string.
    pub fn from_json(&mut self, json: &str) -> Result<(), Error> {
        let temp: OneTimeToken = serde_json::from_str(json)?;
        *self = temp;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn fresh(purpose: TokenPurpose, target_email: Option<&str>, expiry_hours: u32) -> OneTimeToken {
        OneTimeToken::new(
            Uuid::new_v4(),
            purpose,
            target_email.map(|s| s.to_string()),
            expiry_hours,
        )
    }

    #[test]
    fn can_create() {
        let token = fresh(TokenPurpose::PasswordReset, None, 1);
        assert_ne!(token.id, Uuid::nil());
        assert!(!token.is_consumed());
        assert!(!token.is_expired());
    }

    #[test]
    fn can_serialize_and_deserialize_password_reset() {
        let token = fresh(TokenPurpose::PasswordReset, None, 1);
        let json = token.to_json().expect("serialize");
        let mut back = fresh(TokenPurpose::PasswordReset, None, 0);
        back.from_json(&json).expect("deserialize");
        assert_eq!(back, token);
    }

    #[test]
    fn can_serialize_and_deserialize_email_verification_with_target_email() {
        let token = fresh(
            TokenPurpose::EmailVerification,
            Some("alice@example.com"),
            24,
        );
        let json = token.to_json().expect("serialize");
        let mut back = fresh(TokenPurpose::PasswordReset, None, 0);
        back.from_json(&json).expect("deserialize");
        assert_eq!(back, token);
    }

    #[test]
    fn can_serialize_and_deserialize_invite() {
        let token = fresh(TokenPurpose::Invite, None, 24 * 7);
        let json = token.to_json().expect("serialize");
        let mut back = fresh(TokenPurpose::PasswordReset, None, 0);
        back.from_json(&json).expect("deserialize");
        assert_eq!(back, token);
    }

    #[test]
    fn purpose_serialises_as_snake_case() {
        // Lock in the wire-format strings so FileStore JSON files and SQL
        // `purpose` rows agree across backends.
        let token = fresh(TokenPurpose::PasswordReset, None, 1);
        let json = token.to_json().expect("serialize");
        assert!(json.contains("\"purpose\":\"password_reset\""), "{json}");

        let token = fresh(TokenPurpose::Invite, None, 1);
        let json = token.to_json().expect("serialize");
        assert!(json.contains("\"purpose\":\"invite\""), "{json}");

        let token = fresh(TokenPurpose::EmailVerification, None, 1);
        let json = token.to_json().expect("serialize");
        assert!(json.contains("\"purpose\":\"email_verification\""), "{json}");
    }

    #[test]
    fn consumed_at_is_omitted_when_none() {
        // skip_serializing_if = Option::is_none keeps the absent case out
        // of the wire form for unconsumed tokens.
        let token = fresh(TokenPurpose::PasswordReset, None, 1);
        let json = token.to_json().expect("serialize");
        assert!(
            !json.contains("\"consumed_at\""),
            "unconsumed token must not carry consumed_at: {json}"
        );
    }

    #[test]
    fn target_email_is_omitted_when_none() {
        let token = fresh(TokenPurpose::PasswordReset, None, 1);
        let json = token.to_json().expect("serialize");
        assert!(
            !json.contains("\"target_email\""),
            "no target_email must not appear in wire form: {json}"
        );
    }

    #[test]
    fn unknown_purpose_variant_fails_strict() {
        // Strict deserialisation of `purpose` — unknown variants must be
        // rejected so a malformed stored row surfaces immediately rather
        // than silently round-tripping.
        let bogus = r#"{"id":"00000000-0000-0000-0000-000000000000","user_id":"00000000-0000-0000-0000-000000000000","purpose":"not_a_real_purpose","created_at":"2026-05-05T00:00:00Z","expires_at":"2026-05-05T01:00:00Z"}"#;
        let mut t = fresh(TokenPurpose::PasswordReset, None, 1);
        let err = t.from_json(bogus).expect_err("unknown variant must fail");
        assert!(format!("{err}").contains("unknown variant"), "got {err}");
    }
}
