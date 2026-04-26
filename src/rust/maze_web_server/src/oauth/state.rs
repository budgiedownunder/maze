//! CSRF + PKCE state cookie for the OAuth flow.
//!
//! At `begin` time the connector hands us a `PersistedState` containing the
//! CSRF nonce, PKCE verifier, origin (web/mobile), provider, and a creation
//! timestamp. We serialise it to JSON, base64-encode it, and set it as
//! `HttpOnly; Secure; SameSite=Lax; Max-Age=600` on the response. At
//! `callback` time the same cookie comes back; we decode it, check that it
//! has not aged past the TTL, validate that the cookie's `state` matches the
//! `state` query param the IdP echoed back, and pass the rest to the
//! connector's `complete()` method.

use crate::oauth::PersistedState;
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// Name of the cookie holding the per-flow state. Single name across all
/// providers because at most one OAuth flow per browser can be in flight at a
/// time — the cookie is overwritten when a new flow starts and deleted on
/// callback success.
pub const COOKIE_NAME: &str = "maze_oauth_state";

/// Maximum age of a state cookie in seconds. Ten minutes is generous enough
/// for the consent screen but tight enough that a stolen / replayed cookie
/// is useless after a coffee break.
pub const TTL_SECONDS: i64 = 600;

/// Encode the persisted state as a base64-url-no-pad JSON blob suitable for
/// stuffing into a cookie value. Returns the encoded string.
pub fn encode(state: &PersistedState) -> Result<String, String> {
    let json = serde_json::to_vec(state).map_err(|e| format!("state serialise failed: {e}"))?;
    Ok(URL_SAFE_NO_PAD.encode(json))
}

/// Decode a cookie value produced by [`encode`].
pub fn decode(encoded: &str) -> Result<PersistedState, String> {
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|e| format!("state base64 decode failed: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("state json decode failed: {e}"))
}

/// True if the cookie is older than [`TTL_SECONDS`] when measured against `now_unix`.
pub fn is_expired(state: &PersistedState, now_unix: i64) -> bool {
    now_unix.saturating_sub(state.created_at_unix) > TTL_SECONDS
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::FlowOrigin;

    fn sample(now: i64) -> PersistedState {
        PersistedState {
            state: "csrf-nonce-abc".to_string(),
            pkce_verifier: "pkce-verifier-xyz".to_string(),
            origin: FlowOrigin::Web,
            provider: "google".to_string(),
            created_at_unix: now,
        }
    }

    #[test]
    fn round_trip_preserves_all_fields() {
        let original = sample(1_700_000_000);
        let encoded = encode(&original).expect("encode");
        let decoded = decode(&encoded).expect("decode");
        assert_eq!(original, decoded);
    }

    #[test]
    fn cookie_value_is_url_safe() {
        let encoded = encode(&sample(0)).unwrap();
        for ch in encoded.chars() {
            assert!(
                ch.is_ascii_alphanumeric() || ch == '-' || ch == '_',
                "encoded value must be url-safe-no-pad; saw: {ch}"
            );
        }
    }

    #[test]
    fn fresh_state_is_not_expired() {
        let now = 1_700_000_500;
        let state = sample(now - 60); // one minute ago
        assert!(!is_expired(&state, now));
    }

    #[test]
    fn state_at_exact_ttl_is_not_expired() {
        let now = 1_700_000_500;
        let state = sample(now - TTL_SECONDS); // exactly TTL ago
        assert!(!is_expired(&state, now));
    }

    #[test]
    fn state_past_ttl_is_expired() {
        let now = 1_700_000_500;
        let state = sample(now - TTL_SECONDS - 1);
        assert!(is_expired(&state, now));
    }

    #[test]
    fn corrupt_cookie_value_returns_error() {
        assert!(decode("not-valid-base64-!!!").is_err());
        // valid base64 but not valid JSON
        let garbage = URL_SAFE_NO_PAD.encode("not json");
        assert!(decode(&garbage).is_err());
    }
}
