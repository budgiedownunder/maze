//! Google OIDC provider client.
//!
//! Uses the `openidconnect` crate so that ID-token signature verification,
//! issuer matching, and audience checking happen with battle-tested code
//! rather than hand-rolled JWT parsing.
//!
//! The pure-function helper [`parse_userinfo`] turns a set of OIDC claims
//! into a [`NormalisedIdentity`] and is unit-tested with fixture claims so
//! the parsing path is covered without hitting the network.

use crate::config::InternalProviderConfig;
use crate::oauth::internal::{BeginAuthorize, ProviderClient};
use crate::oauth::{NormalisedIdentity, OAuthError};
use async_trait::async_trait;
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{
    AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use std::collections::HashMap;
use tokio::sync::OnceCell;

/// Canonical name for this provider — written to `OAuthIdentity.provider`.
pub const CANONICAL_NAME: &str = "google";
const ISSUER: &str = "https://accounts.google.com";

pub struct GoogleProviderClient {
    client_id: ClientId,
    client_secret: ClientSecret,
    redirect_uri: RedirectUrl,
    metadata: OnceCell<CoreProviderMetadata>,
}

impl GoogleProviderClient {
    pub fn new(cfg: &InternalProviderConfig) -> Result<Self, OAuthError> {
        Ok(Self {
            client_id: ClientId::new(cfg.client_id.clone()),
            client_secret: ClientSecret::new(cfg.client_secret.clone()),
            redirect_uri: RedirectUrl::new(cfg.redirect_uri.clone())
                .map_err(|e| OAuthError::Misconfigured(format!("google redirect_uri invalid: {e}")))?,
            metadata: OnceCell::new(),
        })
    }

    /// Discover Google's OIDC metadata once per process. Subsequent calls
    /// reuse the cached `CoreProviderMetadata`.
    async fn metadata(&self) -> Result<&CoreProviderMetadata, OAuthError> {
        self.metadata
            .get_or_try_init(|| async {
                let issuer = IssuerUrl::new(ISSUER.to_string())
                    .map_err(|e| OAuthError::Misconfigured(format!("issuer url: {e}")))?;
                CoreProviderMetadata::discover_async(issuer, async_http_client)
                    .await
                    .map_err(|e| OAuthError::ProviderTransport(format!("oidc discovery: {e}")))
            })
            .await
    }

    async fn build_client(&self) -> Result<CoreClient, OAuthError> {
        let metadata = self.metadata().await?.clone();
        Ok(CoreClient::from_provider_metadata(
            metadata,
            self.client_id.clone(),
            Some(self.client_secret.clone()),
        )
        .set_redirect_uri(self.redirect_uri.clone()))
    }
}

#[async_trait]
impl ProviderClient for GoogleProviderClient {
    async fn begin(&self, state: &str) -> Result<BeginAuthorize, OAuthError> {
        let client = self.build_client().await?;
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let state_owned = state.to_string();
        let (auth_url, _csrf, _nonce) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                move || CsrfToken::new(state_owned.clone()),
                Nonce::new_random,
            )
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(challenge)
            .url();
        Ok(BeginAuthorize {
            authorize_url: auth_url.to_string(),
            pkce_verifier: verifier.secret().clone(),
        })
    }

    async fn complete(
        &self,
        code: &str,
        pkce_verifier: &str,
    ) -> Result<NormalisedIdentity, OAuthError> {
        let client = self.build_client().await?;
        let token_response = client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| OAuthError::ProviderTransport(format!("google token exchange: {e}")))?;

        let id_token = token_response
            .id_token()
            .ok_or_else(|| OAuthError::ProviderResponse("google: no id_token in response".into()))?;
        let claims = id_token
            .claims(&client.id_token_verifier(), |_: Option<&Nonce>| Ok::<(), String>(()))
            .map_err(|e| OAuthError::ProviderResponse(format!("google id_token verify: {e}")))?;

        // Pull only what we need; pass through the pure-function parser so
        // the parsing logic is unit-testable in isolation.
        let mut bag: HashMap<&str, serde_json::Value> = HashMap::new();
        bag.insert("sub", serde_json::Value::String(claims.subject().to_string()));
        if let Some(email) = claims.email() {
            bag.insert("email", serde_json::Value::String(email.to_string()));
        }
        if let Some(verified) = claims.email_verified() {
            bag.insert("email_verified", serde_json::Value::Bool(verified));
        }
        if let Some(name) = claims.name().and_then(|locales| locales.get(None)) {
            bag.insert("name", serde_json::Value::String(name.as_str().to_string()));
        }
        parse_userinfo(&serde_json::Value::Object(bag.into_iter().map(|(k, v)| (k.to_string(), v)).collect()))
    }
}

/// Pure-function userinfo parser. Takes a JSON object with the OIDC standard
/// claim names and returns a [`NormalisedIdentity`] with `provider = "google"`.
/// Errors when `sub` is absent — every other field is optional.
pub fn parse_userinfo(claims: &serde_json::Value) -> Result<NormalisedIdentity, OAuthError> {
    let obj = claims
        .as_object()
        .ok_or_else(|| OAuthError::ProviderResponse("google: claims not a JSON object".into()))?;

    let sub = obj
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OAuthError::ProviderResponse("google: missing 'sub' claim".into()))?
        .to_string();
    let email = obj.get("email").and_then(|v| v.as_str()).map(str::to_string);
    let email_verified = obj
        .get("email_verified")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let display_name = obj.get("name").and_then(|v| v.as_str()).map(str::to_string);

    Ok(NormalisedIdentity {
        provider: CANONICAL_NAME.to_string(),
        provider_user_id: sub,
        email,
        email_verified,
        display_name,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_userinfo_happy_path() {
        let claims = serde_json::json!({
            "sub": "1234567890",
            "email": "alice@example.com",
            "email_verified": true,
            "name": "Alice Example"
        });
        let id = parse_userinfo(&claims).unwrap();
        assert_eq!(id.provider, "google");
        assert_eq!(id.provider_user_id, "1234567890");
        assert_eq!(id.email.as_deref(), Some("alice@example.com"));
        assert!(id.email_verified);
        assert_eq!(id.display_name.as_deref(), Some("Alice Example"));
    }

    #[test]
    fn parse_userinfo_treats_missing_email_verified_as_false() {
        let claims = serde_json::json!({
            "sub": "1234567890",
            "email": "alice@example.com"
        });
        let id = parse_userinfo(&claims).unwrap();
        assert!(!id.email_verified, "absent claim must default to false to keep auto-link safe");
    }

    #[test]
    fn parse_userinfo_allows_missing_email_and_name() {
        let claims = serde_json::json!({ "sub": "x" });
        let id = parse_userinfo(&claims).unwrap();
        assert_eq!(id.provider_user_id, "x");
        assert!(id.email.is_none());
        assert!(id.display_name.is_none());
        assert!(!id.email_verified);
    }

    #[test]
    fn parse_userinfo_errors_when_sub_missing() {
        let claims = serde_json::json!({ "email": "a@b.c", "email_verified": true });
        let err = parse_userinfo(&claims).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("sub"), "got: {msg}");
    }

    #[test]
    fn parse_userinfo_errors_for_non_object_input() {
        let claims = serde_json::json!("not an object");
        assert!(parse_userinfo(&claims).is_err());
    }
}
