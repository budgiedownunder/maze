//! GitHub OAuth2 provider client.
//!
//! GitHub does not implement OIDC, so we use the plain `oauth2` crate for
//! the authorize / token-exchange dance and then talk to the REST API
//! directly to fetch the authenticated user and their verified emails.
//!
//! The pure-function helper [`pick_primary_verified_email`] is what
//! determines which of the user's email addresses we treat as authoritative;
//! it is unit-tested with fixture lists so the selection rule is exercised
//! without hitting the network.

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
use serde::Deserialize;

pub const CANONICAL_NAME: &str = "github";
const AUTH_URL: &str = "https://github.com/login/oauth/authorize";
const TOKEN_URL: &str = "https://github.com/login/oauth/access_token";
const USER_URL: &str = "https://api.github.com/user";
const EMAILS_URL: &str = "https://api.github.com/user/emails";
const USER_AGENT: &str = "maze_web_server";

pub struct GitHubProviderClient {
    client: BasicClient,
    http: reqwest::Client,
}

impl GitHubProviderClient {
    pub fn new(cfg: &InternalProviderConfig) -> Result<Self, OAuthError> {
        let client = BasicClient::new(
            ClientId::new(cfg.client_id.clone()),
            Some(ClientSecret::new(cfg.client_secret.clone())),
            AuthUrl::new(AUTH_URL.to_string())
                .map_err(|e| OAuthError::Misconfigured(format!("github auth url: {e}")))?,
            Some(
                TokenUrl::new(TOKEN_URL.to_string())
                    .map_err(|e| OAuthError::Misconfigured(format!("github token url: {e}")))?,
            ),
        )
        .set_redirect_uri(
            RedirectUrl::new(cfg.redirect_uri.clone())
                .map_err(|e| OAuthError::Misconfigured(format!("github redirect_uri invalid: {e}")))?,
        );
        let http = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|e| OAuthError::Misconfigured(format!("github http client: {e}")))?;
        Ok(Self { client, http })
    }
}

#[async_trait]
impl ProviderClient for GitHubProviderClient {
    async fn begin(&self, state: &str) -> Result<BeginAuthorize, OAuthError> {
        let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
        let state_owned = state.to_string();
        let (auth_url, _csrf) = self
            .client
            .authorize_url(move || CsrfToken::new(state_owned.clone()))
            .add_scope(Scope::new("read:user".to_string()))
            .add_scope(Scope::new("user:email".to_string()))
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
            .map_err(|e| OAuthError::ProviderTransport(format!("github token exchange: {e}")))?;

        let access_token = token.access_token().secret();

        let user: GitHubUser = self
            .http
            .get(USER_URL)
            .bearer_auth(access_token)
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| OAuthError::ProviderTransport(format!("github /user: {e}")))?
            .error_for_status()
            .map_err(|e| OAuthError::ProviderResponse(format!("github /user status: {e}")))?
            .json()
            .await
            .map_err(|e| OAuthError::ProviderResponse(format!("github /user json: {e}")))?;

        let emails: Vec<GitHubEmail> = self
            .http
            .get(EMAILS_URL)
            .bearer_auth(access_token)
            .header(reqwest::header::ACCEPT, "application/vnd.github+json")
            .send()
            .await
            .map_err(|e| OAuthError::ProviderTransport(format!("github /user/emails: {e}")))?
            .error_for_status()
            .map_err(|e| OAuthError::ProviderResponse(format!("github /user/emails status: {e}")))?
            .json()
            .await
            .map_err(|e| OAuthError::ProviderResponse(format!("github /user/emails json: {e}")))?;

        let primary_verified = pick_primary_verified_email(&emails);
        Ok(NormalisedIdentity {
            provider: CANONICAL_NAME.to_string(),
            provider_user_id: user.id.to_string(),
            email: primary_verified.clone(),
            email_verified: primary_verified.is_some(),
            display_name: user.name.or(Some(user.login)),
        })
    }
}

#[derive(Debug, Deserialize)]
struct GitHubUser {
    id: i64,
    login: String,
    name: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GitHubEmail {
    pub email: String,
    pub primary: bool,
    pub verified: bool,
}

/// From GitHub's `/user/emails` response, return the first email that is both
/// `primary` and `verified`. Returns `None` if no such address exists — the
/// caller treats that as `email_verified = false` and refuses to auto-link.
pub fn pick_primary_verified_email(emails: &[GitHubEmail]) -> Option<String> {
    emails
        .iter()
        .find(|e| e.primary && e.verified)
        .map(|e| e.email.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn email(addr: &str, primary: bool, verified: bool) -> GitHubEmail {
        GitHubEmail {
            email: addr.to_string(),
            primary,
            verified,
        }
    }

    #[test]
    fn picks_primary_verified_email() {
        let emails = vec![
            email("alt@example.com", false, true),
            email("primary@example.com", true, true),
            email("private@example.com", false, true),
        ];
        assert_eq!(
            pick_primary_verified_email(&emails).as_deref(),
            Some("primary@example.com")
        );
    }

    #[test]
    fn returns_none_when_primary_not_verified() {
        let emails = vec![
            email("alt@example.com", false, true),
            email("primary@example.com", true, false),
        ];
        assert!(pick_primary_verified_email(&emails).is_none());
    }

    #[test]
    fn returns_none_when_verified_not_primary() {
        let emails = vec![
            email("verified@example.com", false, true),
            email("primary-unverified@example.com", true, false),
        ];
        assert!(pick_primary_verified_email(&emails).is_none());
    }

    #[test]
    fn returns_none_for_empty_list() {
        assert!(pick_primary_verified_email(&[]).is_none());
    }

    #[test]
    fn returns_first_match_when_multiple_primary_verified() {
        // GitHub should never return more than one primary, but be defensive.
        let emails = vec![
            email("first@example.com", true, true),
            email("second@example.com", true, true),
        ];
        assert_eq!(
            pick_primary_verified_email(&emails).as_deref(),
            Some("first@example.com")
        );
    }
}
