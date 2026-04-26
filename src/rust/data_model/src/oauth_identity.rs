use crate::wrappers::generate_now;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a link between a maze user and an external OAuth identity provider.
///
/// `provider_email` is the most recent verified email observed from this provider.
/// It is refreshed on every successful sign-in (not a one-shot link-time snapshot
/// and not a fallback for `User.email`). It powers the future "Linked accounts"
/// UI and is the natural ingestion point for multi-email support: every
/// successful OAuth sign-in is one fresh "verified by provider X" observation.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct OAuthIdentity {
    /// Canonical provider name, e.g. "google" or "github".
    pub provider: String,
    /// Stable provider-side user id (`sub` for OIDC, numeric id for GitHub).
    pub provider_user_id: String,
    /// Most recent verified email observed from this provider, or `None` if the
    /// provider did not return one. Refreshed on every successful sign-in.
    pub provider_email: Option<String>,
    /// When this identity was first linked to the user.
    pub linked_at: DateTime<Utc>,
    /// When this identity was most recently used for a successful sign-in.
    pub last_seen_at: DateTime<Utc>,
}

impl OAuthIdentity {
    pub fn new(provider: String, provider_user_id: String, provider_email: Option<String>) -> Self {
        let now = generate_now();
        Self {
            provider,
            provider_user_id,
            provider_email,
            linked_at: now,
            last_seen_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn new_sets_linked_at_and_last_seen_at_to_now() {
        let id = OAuthIdentity::new(
            "google".to_string(),
            "sub-123".to_string(),
            Some("alice@example.com".to_string()),
        );
        assert_eq!(id.provider, "google");
        assert_eq!(id.provider_user_id, "sub-123");
        assert_eq!(id.provider_email.as_deref(), Some("alice@example.com"));
        assert_eq!(id.linked_at, id.last_seen_at);
    }

    #[test]
    fn new_allows_missing_email() {
        let id = OAuthIdentity::new("github".to_string(), "42".to_string(), None);
        assert!(id.provider_email.is_none());
    }

    #[test]
    fn can_roundtrip_json() {
        let id = OAuthIdentity::new(
            "google".to_string(),
            "sub-456".to_string(),
            Some("alice@example.com".to_string()),
        );
        let json = serde_json::to_string(&id).expect("serialize");
        let back: OAuthIdentity = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(id, back);
    }
}
