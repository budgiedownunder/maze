//! OAuth sign-in subsystem
//!
//! All OAuth logic is hidden behind the [`OAuthConnector`] trait. Handlers,
//! account-resolution, and front-end-facing types know nothing about Google,
//! GitHub, or Auth0 directly — they only call methods on the trait. This is
//! what lets a future `Auth0Connector` implementation drop in alongside the
//! current [`internal::InternalOAuthConnector`] without disturbing handlers,
//! storage, or front-end code.

pub mod account;
pub mod internal;
pub mod state;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use utoipa::ToSchema;

/// Shared, ref-counted, type-erased connector handle. Built once at server
/// startup from `AppConfig`, cloned cheaply into per-request handlers via
/// `actix_web::web::Data`.
pub type SharedOAuthConnector = Arc<dyn OAuthConnector>;

// ---- Public types shared across connectors ----------------------------------

/// One OAuth provider exposed to the front end. Rendered into
/// `AppFeaturesResponse.oauth_providers` so the React / MAUI clients can
/// decide which buttons to display.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
pub struct OAuthProviderPublic {
    /// Canonical provider name, e.g. "google" or "github". Stable across
    /// connector implementations so account links survive a future move from
    /// the internal connector to Auth0.
    pub name: String,
    /// Human-readable label rendered on the button.
    pub display_name: String,
}

/// Where the OAuth flow originated. The handler uses this to decide how to
/// hand the bearer token back to the client at the end:
/// - `Web`     — redirect to `/oauth/callback#token=...` for the SPA to ingest.
/// - `Mobile`  — redirect to `{mobile_redirect_scheme}://oauth-callback?token=...`
///   for `WebAuthenticator` to capture inside the MAUI app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlowOrigin {
    Web,
    Mobile,
}

/// Per-flow state that the server stores in the CSRF cookie at `begin` time
/// and reads back at `callback` time. Encrypted-at-rest is unnecessary because
/// the cookie is `HttpOnly; Secure; SameSite=Lax` and only ever round-trips
/// to this server, but every field except `created_at` is immaterial after
/// the flow completes — the cookie is single-use.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PersistedState {
    /// CSRF nonce. Must match the `state` query parameter on the callback.
    pub state: String,
    /// PKCE verifier paired with the challenge sent in the authorize URL.
    pub pkce_verifier: String,
    /// Whether the flow was initiated for the web SPA or the mobile app.
    pub origin: FlowOrigin,
    /// Canonical provider name. Validated against the `{provider}` path
    /// parameter on the callback to defend against a swap-the-provider attack.
    pub provider: String,
    /// When the flow started. Used to enforce the 10-minute TTL.
    pub created_at_unix: i64,
}

/// Result of [`OAuthConnector::begin`]: the authorize URL the front end (or
/// the MAUI WebAuthenticator) should send the user to, plus the state to
/// persist in the CSRF cookie.
#[derive(Debug, Clone)]
pub struct BeginFlow {
    pub authorize_url: String,
    pub persisted: PersistedState,
}

/// Result of [`OAuthConnector::complete`]: a connector-agnostic view of the
/// user as the IdP describes them. The handler layer never sees provider-
/// specific token shapes — only this normalised form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalisedIdentity {
    /// Canonical provider name (e.g. "google", "github"). Stored verbatim on
    /// the user's `OAuthIdentity` row, so callbacks resolve the right user
    /// regardless of which connector produced the identity.
    pub provider: String,
    /// Stable, opaque, provider-side user id. `sub` for OIDC, the numeric id
    /// for GitHub. Never the email — emails change.
    pub provider_user_id: String,
    /// Email returned by the provider. May be `None` when the provider does
    /// not expose one (rare; we also reject the sign-in earlier in that case
    /// for the email-link branch). Always an absolute address when present.
    pub email: Option<String>,
    /// Whether the provider asserts the email is verified. Auto-link to an
    /// existing password account is gated on this being `true`.
    pub email_verified: bool,
    /// Optional human-readable display name. Best-effort; absent on some
    /// providers / scopes.
    pub display_name: Option<String>,
}

/// Errors raised by an [`OAuthConnector`]. Crafted so that the handler can
/// map each variant to a specific HTTP status / log level without needing to
/// know which connector raised it.
#[derive(Debug)]
pub enum OAuthError {
    /// The connector does not know about a provider with this canonical name,
    /// or it knows about it but it is disabled. → 404.
    UnknownOrDisabledProvider(String),
    /// The CSRF state cookie did not match the callback's `state` query param,
    /// or the cookie was missing / expired. → 400.
    InvalidState(String),
    /// The provider responded but did not expose the data we need (e.g. no
    /// `sub` claim, no verified email when one was required). → 502.
    ProviderResponse(String),
    /// Network or HTTP error talking to the provider. → 502.
    ProviderTransport(String),
    /// Internal misconfiguration that should have been caught at config-load
    /// time but wasn't. → 500.
    Misconfigured(String),
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OAuthError::UnknownOrDisabledProvider(s) => write!(f, "unknown or disabled OAuth provider: {s}"),
            OAuthError::InvalidState(s) => write!(f, "invalid OAuth state: {s}"),
            OAuthError::ProviderResponse(s) => write!(f, "OAuth provider response error: {s}"),
            OAuthError::ProviderTransport(s) => write!(f, "OAuth provider transport error: {s}"),
            OAuthError::Misconfigured(s) => write!(f, "OAuth misconfiguration: {s}"),
        }
    }
}

impl std::error::Error for OAuthError {}

// ---- The connector trait ----------------------------------------------------

/// The seam between the OAuth handler layer and any specific identity-broker
/// implementation. v1 ships with [`internal::InternalOAuthConnector`]; later
/// drops can plug an `Auth0Connector`, `KeycloakConnector`, etc. behind the
/// same interface.
#[async_trait]
pub trait OAuthConnector: Send + Sync {
    /// Providers this connector exposes to the front end. Read once at
    /// startup and surfaced via `GET /api/v1/features`. Order is
    /// implementation-defined; the front end sorts as needed.
    fn enabled_providers(&self) -> Vec<OAuthProviderPublic>;

    /// Build the provider's authorize URL plus the per-flow state to persist
    /// in the CSRF cookie. The trait does no IO of its own — implementors
    /// only generate URLs from configuration data.
    async fn begin(&self, provider: &str, origin: FlowOrigin) -> Result<BeginFlow, OAuthError>;

    /// Exchange the IdP's `code` (paired with the cookie's PKCE verifier)
    /// for whatever provider-side tokens it issues, fetch userinfo, and
    /// return the normalised identity. `cookie_state` and `returned_state`
    /// are validated by the handler before this method is called, so
    /// implementors can rely on them matching.
    async fn complete(
        &self,
        provider: &str,
        code: &str,
        cookie_state: &PersistedState,
    ) -> Result<NormalisedIdentity, OAuthError>;
}

/// Connector used when `[oauth] enabled = false`. Reports zero providers and
/// rejects every flow with `UnknownOrDisabledProvider`. Lets the rest of the
/// app treat the connector as always-present without `Option` plumbing.
pub struct NoOpConnector;

#[async_trait]
impl OAuthConnector for NoOpConnector {
    fn enabled_providers(&self) -> Vec<OAuthProviderPublic> { Vec::new() }

    async fn begin(&self, provider: &str, _origin: FlowOrigin) -> Result<BeginFlow, OAuthError> {
        Err(OAuthError::UnknownOrDisabledProvider(provider.to_string()))
    }

    async fn complete(
        &self,
        provider: &str,
        _code: &str,
        _cookie_state: &PersistedState,
    ) -> Result<NormalisedIdentity, OAuthError> {
        Err(OAuthError::UnknownOrDisabledProvider(provider.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flow_origin_serialises_lowercase() {
        // The handler reads `origin` off the query string; the React API
        // client sends "web" / "mobile". Lock this in.
        assert_eq!(serde_json::to_string(&FlowOrigin::Web).unwrap(), "\"web\"");
        assert_eq!(serde_json::to_string(&FlowOrigin::Mobile).unwrap(), "\"mobile\"");
    }
}
