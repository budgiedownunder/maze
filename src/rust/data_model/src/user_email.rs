use crate::wrappers::generate_now;
use chrono::{DateTime, SubsecRound, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// One email address attached to a [`User`](crate::User).
///
/// A user may carry multiple email rows; exactly one row is `is_primary`
/// at any time. `verified` indicates that ownership of the address has
/// been proven (either by clicking a verification link or by an OAuth
/// provider asserting `email_verified = true`).
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct UserEmail {
    /// The email address itself.
    pub email: String,
    /// Whether this is the user's primary (canonical) contact address.
    /// Exactly one row per user has `is_primary = true` at all times.
    pub is_primary: bool,
    /// Whether ownership of this address has been verified.
    pub verified: bool,
    /// When this address was last verified, or `None` if it has never
    /// been verified.
    pub verified_at: Option<DateTime<Utc>>,
}

impl UserEmail {
    /// Builds a primary, verified email row with `verified_at` set to now,
    /// truncated to millisecond precision so it round-trips losslessly through
    /// the SQL store's RFC 3339 storage format (the SqlStore writes timestamps
    /// at millisecond precision per the `0001_initial.sql` design).
    /// Used when seeding accounts created via signup, OAuth, or admin-init.
    pub fn new_primary_verified(email: &str) -> Self {
        Self {
            email: email.to_string(),
            is_primary: true,
            verified: true,
            verified_at: Some(generate_now().trunc_subsecs(3)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_primary_verified_sets_flags() {
        let row = UserEmail::new_primary_verified("alice@example.com");
        assert_eq!(row.email, "alice@example.com");
        assert!(row.is_primary);
        assert!(row.verified);
        assert!(row.verified_at.is_some());
    }

    #[test]
    fn can_roundtrip_json() {
        let row = UserEmail::new_primary_verified("alice@example.com");
        let json = serde_json::to_string(&row).expect("serialize");
        let back: UserEmail = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(row, back);
    }
}
