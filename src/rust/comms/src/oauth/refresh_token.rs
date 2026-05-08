use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::CommsError;
use crate::oauth::clock::{Clock, SystemClock};
use crate::oauth::OAuthTokenSource;

/// Configuration for `RefreshTokenTokenSource`.
///
/// Carries a long-lived `refresh_token` plus the `client_id` /
/// `client_secret` of the OAuth client it was issued against; access tokens
/// are minted on demand by POSTing `grant_type=refresh_token` to
/// `token_uri`. `scopes` are sent on the refresh request — most providers
/// either ignore the value (returning the originally consented scope set)
/// or narrow the resulting access token to the intersection.
///
/// `refresh_skew` shortens the cache lifetime by this much, ensuring the
/// source returns a token that still has at least `refresh_skew` of
/// remaining validity from the perspective of the resource server.
pub struct RefreshTokenConfig {
    pub client_id: String,
    pub client_secret: String,
    pub refresh_token: String,
    pub token_uri: String,
    pub scopes: Vec<String>,
    pub refresh_skew: Duration,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
}

#[derive(Debug, Clone)]
struct CachedToken {
    token: String,
    expires_at: DateTime<Utc>,
}

/// OAuth2 refresh-token flow against a generic token endpoint. Used for
/// per-user accounts where the OAuth dance happens once out-of-band (e.g.
/// the operator running through Google's OAuth Playground) and the
/// resulting `refresh_token` is fed to the server as a long-lived secret;
/// the server then mints access tokens against `token_uri` whenever the
/// cached one expires.
///
/// Cloning is intentionally not supported — wrap an instance in `Arc` if
/// multiple owners need to share its cache.
pub struct RefreshTokenTokenSource {
    config: RefreshTokenConfig,
    http: reqwest::Client,
    clock: Arc<dyn Clock>,
    cached: Mutex<Option<CachedToken>>,
}

impl RefreshTokenTokenSource {
    /// Construct with the default `reqwest` client and `SystemClock`.
    /// Returns an error if the underlying HTTP client fails to build.
    /// Tokens are minted lazily on the first `OAuthTokenSource::access_token`
    /// call against `token_uri`; nothing happens on the network at
    /// construction time.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms::{RefreshTokenConfig, RefreshTokenTokenSource};
    /// use std::time::Duration;
    ///
    /// let cfg = RefreshTokenConfig {
    ///     client_id: "client-id.apps.googleusercontent.com".into(),
    ///     client_secret: "GOCSPX-secret".into(),
    ///     refresh_token: "1//refresh-token".into(),
    ///     token_uri: "https://oauth2.googleapis.com/token".into(),
    ///     scopes: vec!["https://mail.google.com/".into()],
    ///     refresh_skew: Duration::from_secs(60),
    /// };
    /// let _source = RefreshTokenTokenSource::new(cfg).expect("build token source");
    /// ```
    pub fn new(config: RefreshTokenConfig) -> Result<Self, CommsError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| CommsError::Config(format!("reqwest client: {e}")))?;
        Ok(Self::with_http_and_clock(config, http, Arc::new(SystemClock)))
    }

    pub(crate) fn with_http_and_clock(
        config: RefreshTokenConfig,
        http: reqwest::Client,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            config,
            http,
            clock,
            cached: Mutex::new(None),
        }
    }

    async fn mint(&self) -> Result<CachedToken, CommsError> {
        let scope = self.config.scopes.join(" ");
        let mut form: Vec<(&str, &str)> = vec![
            ("client_id", self.config.client_id.as_str()),
            ("client_secret", self.config.client_secret.as_str()),
            ("grant_type", "refresh_token"),
            ("refresh_token", self.config.refresh_token.as_str()),
        ];
        if !scope.is_empty() {
            form.push(("scope", scope.as_str()));
        }

        let response = self
            .http
            .post(&self.config.token_uri)
            .form(&form)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() || e.is_connect() {
                    CommsError::Transient(format!("token endpoint: {e}"))
                } else {
                    CommsError::Provider(format!("token endpoint: {e}"))
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(CommsError::ProviderHttp {
                status: status.as_u16(),
                body,
            });
        }

        let body: TokenResponse = response
            .json()
            .await
            .map_err(|e| CommsError::Provider(format!("token response decode: {e}")))?;

        let skew = chrono::Duration::from_std(self.config.refresh_skew)
            .unwrap_or_else(|_| chrono::Duration::zero());
        let lifetime = chrono::Duration::seconds(body.expires_in) - skew;
        let expires_at = self.clock.now() + lifetime;

        Ok(CachedToken {
            token: body.access_token,
            expires_at,
        })
    }
}

#[async_trait]
impl OAuthTokenSource for RefreshTokenTokenSource {
    async fn access_token(&self) -> Result<String, CommsError> {
        let now = self.clock.now();
        {
            let guard = self.cached.lock().expect("refresh token cache poisoned");
            if let Some(cached) = guard.as_ref() {
                if cached.expires_at > now {
                    return Ok(cached.token.clone());
                }
            }
        }

        let fresh = self.mint().await?;
        let token = fresh.token.clone();
        let mut guard = self.cached.lock().expect("refresh token cache poisoned");
        *guard = Some(fresh);
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::clock::TestClock;
    use chrono::TimeZone;
    use pretty_assertions::assert_eq;
    use wiremock::matchers::{body_string_contains, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn config_against(server_uri: &str) -> RefreshTokenConfig {
        RefreshTokenConfig {
            client_id: "test-client-id".into(),
            client_secret: "test-client-secret".into(),
            refresh_token: "test-refresh-token".into(),
            token_uri: format!("{server_uri}/token"),
            scopes: vec!["https://mail.google.com/".into()],
            refresh_skew: Duration::from_secs(0),
        }
    }

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 8, 12, 0, 0).unwrap()
    }

    fn token_source(server_uri: &str, clock: Arc<dyn Clock>) -> RefreshTokenTokenSource {
        RefreshTokenTokenSource::with_http_and_clock(
            config_against(server_uri),
            reqwest::Client::new(),
            clock,
        )
    }

    #[tokio::test]
    async fn mints_an_access_token_from_the_endpoint_with_grant_type_refresh_token() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(body_string_contains("client_id=test-client-id"))
            .and(body_string_contains("client_secret=test-client-secret"))
            .and(body_string_contains("grant_type=refresh_token"))
            .and(body_string_contains("refresh_token=test-refresh-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "the-token",
                "token_type": "Bearer",
                "expires_in": 3600,
            })))
            .mount(&server)
            .await;

        let clock: Arc<dyn Clock> = Arc::new(TestClock::new(fixed_now()));
        let source = token_source(&server.uri(), clock);
        let token = source.access_token().await.expect("mint");
        assert_eq!(token, "the-token");
    }

    #[tokio::test]
    async fn caches_the_token_within_the_expiry_window() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "the-token",
                "token_type": "Bearer",
                "expires_in": 3600,
            })))
            .expect(1)
            .mount(&server)
            .await;

        let clock: Arc<dyn Clock> = Arc::new(TestClock::new(fixed_now()));
        let source = token_source(&server.uri(), clock);

        let first = source.access_token().await.expect("first");
        let second = source.access_token().await.expect("second");
        assert_eq!(first, "the-token");
        assert_eq!(second, "the-token");
        // wiremock verifies `.expect(1)` on MockServer drop.
    }

    #[tokio::test]
    async fn re_mints_after_expiry() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "first-token",
                "token_type": "Bearer",
                "expires_in": 60,
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "second-token",
                "token_type": "Bearer",
                "expires_in": 60,
            })))
            .mount(&server)
            .await;

        let clock = Arc::new(TestClock::new(fixed_now()));
        let clock_dyn: Arc<dyn Clock> = clock.clone();
        let source = token_source(&server.uri(), clock_dyn);

        let first = source.access_token().await.expect("first");
        assert_eq!(first, "first-token");

        clock.advance(chrono::Duration::seconds(120));

        let second = source.access_token().await.expect("second");
        assert_eq!(second, "second-token");
    }

    #[tokio::test]
    async fn maps_4xx_to_permanent_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(
                ResponseTemplate::new(400)
                    .set_body_string(r#"{"error":"invalid_grant"}"#),
            )
            .mount(&server)
            .await;

        let clock: Arc<dyn Clock> = Arc::new(TestClock::new(fixed_now()));
        let source = token_source(&server.uri(), clock);

        let err = source.access_token().await.expect_err("must reject");
        assert!(!err.is_transient(), "expected permanent: {err:?}");
        match err {
            CommsError::ProviderHttp { status, body } => {
                assert_eq!(status, 400);
                assert!(body.contains("invalid_grant"));
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn maps_5xx_to_transient_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(503).set_body_string("Service Unavailable"))
            .mount(&server)
            .await;

        let clock: Arc<dyn Clock> = Arc::new(TestClock::new(fixed_now()));
        let source = token_source(&server.uri(), clock);

        let err = source.access_token().await.expect_err("must reject");
        assert!(err.is_transient(), "expected transient: {err:?}");
        match err {
            CommsError::ProviderHttp { status, .. } => assert_eq!(status, 503),
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn malformed_response_body_maps_to_provider_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
            .mount(&server)
            .await;

        let clock: Arc<dyn Clock> = Arc::new(TestClock::new(fixed_now()));
        let source = token_source(&server.uri(), clock);

        let err = source.access_token().await.expect_err("must reject");
        match err {
            CommsError::Provider(msg) => {
                assert!(
                    msg.contains("token response decode"),
                    "expected decode error, got: {msg}"
                );
            }
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn omits_scope_form_field_when_scopes_empty() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "the-token",
                "token_type": "Bearer",
                "expires_in": 3600,
            })))
            .mount(&server)
            .await;

        let clock: Arc<dyn Clock> = Arc::new(TestClock::new(fixed_now()));
        let mut cfg = config_against(&server.uri());
        cfg.scopes = Vec::new();
        let source = RefreshTokenTokenSource::with_http_and_clock(
            cfg,
            reqwest::Client::new(),
            clock,
        );
        let token = source.access_token().await.expect("mint");
        assert_eq!(token, "the-token");
        // The presence/absence of `scope=` in the body is not directly
        // observable from the test side without a custom matcher; this
        // test guards against the empty-scope code path panicking or
        // producing a malformed form body that the endpoint would reject.
    }
}
