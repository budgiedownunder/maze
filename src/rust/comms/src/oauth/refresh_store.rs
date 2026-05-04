use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::CommsError;

/// A persisted refresh token. Carrier for refresh-token-based OAuth flows;
/// the recommended `comms` paths (client_credentials, service-account) do
/// not use refresh tokens at all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshToken {
    pub token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
}

/// Pluggable persistence for refresh tokens. The trait surface is reserved
/// for future use cases (multi-tenant SaaS, end-user OAuth flows). No
/// implementations ship today — recommended deployments mint access tokens
/// on demand from `client_id`/`client_secret` (Microsoft) or service-account
/// JWT-bearer assertions (Google) and don't need persistence.
#[async_trait]
pub trait RefreshTokenStore: Send + Sync {
    async fn load(&self, key: &str) -> Result<Option<RefreshToken>, CommsError>;

    async fn store(&self, key: &str, token: &RefreshToken) -> Result<(), CommsError>;

    async fn delete(&self, key: &str) -> Result<(), CommsError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;

    #[test]
    fn refresh_token_round_trips_through_serde_json() {
        let t = RefreshToken {
            token: "abcdef".into(),
            expires_at: Some(Utc.with_ymd_and_hms(2026, 5, 4, 12, 0, 0).unwrap()),
        };
        let json = serde_json::to_string(&t).expect("serialize");
        let round_trip: RefreshToken = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(t, round_trip);
    }

    #[test]
    fn refresh_token_omits_expires_at_when_none() {
        let t = RefreshToken {
            token: "x".into(),
            expires_at: None,
        };
        let json = serde_json::to_string(&t).expect("serialize");
        assert!(!json.contains("expires_at"));
    }
}
