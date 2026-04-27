//! Facebook OAuth2 provider client.
//!
//! Facebook does not implement OIDC, so we use the plain `oauth2` crate for
//! the authorize / token-exchange dance and then talk to the Graph API
//! directly to fetch the authenticated user's id, name, and email. Pattern
//! mirrors [`super::github`] — both are OAuth2 with a custom userinfo
//! endpoint.
//!
//! The pure-function helper [`parse_userinfo`] turns a `/me` response into a
//! [`NormalisedIdentity`] and is unit-tested with fixture JSON so the
//! parsing path is covered without hitting the network.
//!
//! **Email verification policy**: Facebook does not expose an
//! `email_verified` flag on the userinfo response. Their account-level
//! verification is implicit — you cannot add an email to a Facebook account
//! without verifying it. We therefore treat `email_verified = true` whenever
//! `email` is present, matching how Auth0 and Clerk treat Facebook by
//! default. If `email` is absent (the user declined the `email` scope at
//! consent), the identity is returned with `email = None` and the account
//! resolver refuses to email-link or create-new — same behaviour as GitHub
//! when `pick_primary_verified_email` returns `None`.

use crate::config::InternalProviderConfig;
use crate::oauth::internal::{BeginAuthorize, ProviderClient};
use crate::oauth::{NormalisedIdentity, OAuthError};
use async_trait::async_trait;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, Scope, TokenResponse, TokenUrl,
};

pub const CANONICAL_NAME: &str = "facebook";
const AUTH_URL: &str = "https://www.facebook.com/v18.0/dialog/oauth";
const TOKEN_URL: &str = "https://graph.facebook.com/v18.0/oauth/access_token";
const USERINFO_URL: &str = "https://graph.facebook.com/v18.0/me?fields=id,email,name";
const USER_AGENT: &str = "maze_web_server";

pub struct FacebookProviderClient {
    client: BasicClient,
    http: reqwest::Client,
}

impl FacebookProviderClient {
    pub fn new(cfg: &InternalProviderConfig) -> Result<Self, OAuthError> {
        let client = BasicClient::new(
            ClientId::new(cfg.client_id.clone()),
            Some(ClientSecret::new(cfg.client_secret.clone())),
            AuthUrl::new(AUTH_URL.to_string())
                .map_err(|e| OAuthError::Misconfigured(format!("facebook auth url: {e}")))?,
            Some(
                TokenUrl::new(TOKEN_URL.to_string())
                    .map_err(|e| OAuthError::Misconfigured(format!("facebook token url: {e}")))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(cfg.redirect_uri.clone())
                .map_err(|e| OAuthError::Misconfigured(format!("facebook redirect_uri invalid: {e}")))?,
        );
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| OAuthError::Misconfigured(format!("facebook http client: {e}")))?;
        Ok(Self { client, http })
    }
}

#[async_trait]
impl ProviderClient for FacebookProviderClient {
    async fn begin(&self, state: &str) -> Result<BeginAuthorize, OAuthError> {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let state_owned = state.to_string();
        let (auth_url, _csrf) = self
            .client
            .authorize_url(move || CsrfToken::new(state_owned.clone()))
            .add_scope(Scope::new("public_profile".to_string()))
            .add_scope(Scope::new("email".to_string()))
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
        let token = self
            .client
            .exchange_code(AuthorizationCode::new(code.to_string()))
            .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|e| OAuthError::ProviderTransport(format!("facebook token exchange: {e}")))?;

        let access_token = token.access_token().secret();

        let userinfo: serde_json::Value = self
            .http
            .get(USERINFO_URL)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| OAuthError::ProviderTransport(format!("facebook /me: {e}")))?
            .error_for_status()
            .map_err(|e| OAuthError::ProviderResponse(format!("facebook /me status: {e}")))?
            .json()
            .await
            .map_err(|e| OAuthError::ProviderResponse(format!("facebook /me json: {e}")))?;

        parse_userinfo(&userinfo)
    }
}

/// Pure-function userinfo parser. Takes a JSON object with the Facebook Graph
/// API `/me` response shape (`id`, optional `email`, optional `name`) and
/// returns a [`NormalisedIdentity`] with `provider = "facebook"`. Errors when
/// `id` is absent — every other field is optional. `email_verified` is `true`
/// whenever `email` is present (see module docs for rationale).
pub fn parse_userinfo(claims: &serde_json::Value) -> Result<NormalisedIdentity, OAuthError> {
    let obj = claims
        .as_object()
        .ok_or_else(|| OAuthError::ProviderResponse("facebook: claims not a JSON object".into()))?;

    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| OAuthError::ProviderResponse("facebook: missing 'id' field".into()))?
        .to_string();
    let email = obj.get("email").and_then(|v| v.as_str()).map(str::to_string);
    let display_name = obj.get("name").and_then(|v| v.as_str()).map(str::to_string);
    // Facebook account-level verification is implicit when email is present.
    let email_verified = email.is_some();

    Ok(NormalisedIdentity {
        provider: CANONICAL_NAME.to_string(),
        provider_user_id: id,
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
            "id": "10001234567890",
            "email": "alice@example.com",
            "name": "Alice Example"
        });
        let id = parse_userinfo(&claims).unwrap();
        assert_eq!(id.provider, "facebook");
        assert_eq!(id.provider_user_id, "10001234567890");
        assert_eq!(id.email.as_deref(), Some("alice@example.com"));
        assert!(id.email_verified, "email present implies verified for Facebook");
        assert_eq!(id.display_name.as_deref(), Some("Alice Example"));
    }

    #[test]
    fn parse_userinfo_treats_missing_email_as_unverified() {
        // User declined the `email` scope at consent — Facebook returns the
        // /me response without `email`. We surface this as `email = None` +
        // `email_verified = false` so the account resolver refuses to
        // email-link or create-new.
        let claims = serde_json::json!({ "id": "100099999", "name": "No Email User" });
        let id = parse_userinfo(&claims).unwrap();
        assert_eq!(id.provider_user_id, "100099999");
        assert!(id.email.is_none());
        assert!(!id.email_verified);
    }

    #[test]
    fn parse_userinfo_allows_missing_name() {
        let claims = serde_json::json!({ "id": "x", "email": "a@b.c" });
        let id = parse_userinfo(&claims).unwrap();
        assert_eq!(id.provider_user_id, "x");
        assert_eq!(id.email.as_deref(), Some("a@b.c"));
        assert!(id.display_name.is_none());
        assert!(id.email_verified);
    }

    #[test]
    fn parse_userinfo_errors_when_id_missing() {
        let claims = serde_json::json!({ "email": "a@b.c", "name": "No Id" });
        let err = parse_userinfo(&claims).unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("id"), "got: {msg}");
    }

    #[test]
    fn parse_userinfo_errors_for_non_object_input() {
        let claims = serde_json::json!("not an object");
        assert!(parse_userinfo(&claims).is_err());
    }
}
