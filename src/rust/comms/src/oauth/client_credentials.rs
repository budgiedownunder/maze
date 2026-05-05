use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::error::CommsError;
use crate::oauth::clock::{Clock, SystemClock};
use crate::oauth::OAuthTokenSource;

/// Configuration for `ClientCredentialsTokenSource`.
///
/// `tenant_id` is the Azure AD tenant identifier (a UUID, or `common` /
/// `organizations`). `scope` is typically `https://graph.microsoft.com/.default`
/// for Microsoft Graph access. Override `token_endpoint_url` for tests or
/// non-standard sovereign clouds; otherwise the URL is derived from
/// `tenant_id`.
///
/// `refresh_skew` shortens the cache lifetime by this much, ensuring the
/// source returns a token that still has at least `refresh_skew` of
/// remaining validity from the perspective of the resource server.
pub struct ClientCredentialsConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub scope: String,
    pub token_endpoint_url: Option<String>,
    pub refresh_skew: Duration,
}

impl ClientCredentialsConfig {
    /// Returns the resolved Azure AD token endpoint URL.
    /// `token_endpoint_url` overrides the default if set; otherwise the
    /// URL is derived from `tenant_id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms::ClientCredentialsConfig;
    /// use std::time::Duration;
    ///
    /// let cfg = ClientCredentialsConfig {
    ///     tenant_id: "00000000-0000-0000-0000-000000000000".into(),
    ///     client_id: "app-id".into(),
    ///     client_secret: "secret".into(),
    ///     scope: "https://graph.microsoft.com/.default".into(),
    ///     token_endpoint_url: None,
    ///     refresh_skew: Duration::from_secs(60),
    /// };
    /// assert_eq!(
    ///     cfg.token_endpoint(),
    ///     "https://login.microsoftonline.com/00000000-0000-0000-0000-000000000000/oauth2/v2.0/token"
    /// );
    /// ```
    pub fn token_endpoint(&self) -> String {
        self.token_endpoint_url.clone().unwrap_or_else(|| {
            format!(
                "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
                self.tenant_id
            )
        })
    }
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

/// OAuth2 client_credentials flow against an Azure AD tenant. Mints and
/// caches access tokens in-memory; no refresh token, no rotation.
///
/// Cloning is intentionally not supported — wrap an instance in `Arc` if
/// multiple owners need to share its cache.
pub struct ClientCredentialsTokenSource {
    config: ClientCredentialsConfig,
    http: reqwest::Client,
    clock: Arc<dyn Clock>,
    cached: Mutex<Option<CachedToken>>,
}

impl ClientCredentialsTokenSource {
    /// Construct with the default `reqwest` client and `SystemClock`.
    /// Returns an error if the underlying HTTP client fails to build.
    /// Tokens are minted lazily on the first `OAuthTokenSource::get_access_token`
    /// call against the configured Azure AD tenant; nothing happens on the
    /// network at construction time.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms::{ClientCredentialsConfig, ClientCredentialsTokenSource};
    /// use std::time::Duration;
    ///
    /// let cfg = ClientCredentialsConfig {
    ///     tenant_id: "00000000-0000-0000-0000-000000000000".into(),
    ///     client_id: "app-id".into(),
    ///     client_secret: "secret".into(),
    ///     scope: "https://graph.microsoft.com/.default".into(),
    ///     token_endpoint_url: None,
    ///     refresh_skew: Duration::from_secs(60),
    /// };
    /// let _source = ClientCredentialsTokenSource::new(cfg).expect("build token source");
    /// ```
    pub fn new(config: ClientCredentialsConfig) -> Result<Self, CommsError> {
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| CommsError::Config(format!("reqwest client: {e}")))?;
        Ok(Self::with_http_and_clock(config, http, Arc::new(SystemClock)))
    }

    pub(crate) fn with_http_and_clock(
        config: ClientCredentialsConfig,
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
        let url = self.config.token_endpoint();
        let response = self
            .http
            .post(&url)
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
                ("grant_type", "client_credentials"),
                ("scope", self.config.scope.as_str()),
            ])
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
impl OAuthTokenSource for ClientCredentialsTokenSource {
    async fn access_token(&self) -> Result<String, CommsError> {
        let now = self.clock.now();
        {
            let guard = self
                .cached
                .lock()
                .expect("client credentials cache poisoned");
            if let Some(cached) = guard.as_ref() {
                if cached.expires_at > now {
                    return Ok(cached.token.clone());
                }
            }
        }

        let fresh = self.mint().await?;
        let token = fresh.token.clone();
        let mut guard = self
            .cached
            .lock()
            .expect("client credentials cache poisoned");
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

    fn config_against(server_uri: &str) -> ClientCredentialsConfig {
        ClientCredentialsConfig {
            tenant_id: "test-tenant".into(),
            client_id: "test-client-id".into(),
            client_secret: "test-client-secret".into(),
            scope: "https://graph.microsoft.com/.default".into(),
            token_endpoint_url: Some(format!("{server_uri}/oauth2/v2.0/token")),
            refresh_skew: Duration::from_secs(0),
        }
    }

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 4, 12, 0, 0).unwrap()
    }

    fn token_source(
        server_uri: &str,
        clock: Arc<dyn Clock>,
    ) -> ClientCredentialsTokenSource {
        ClientCredentialsTokenSource::with_http_and_clock(
            config_against(server_uri),
            reqwest::Client::new(),
            clock,
        )
    }

    #[test]
    fn token_endpoint_defaults_to_microsoft_login_url() {
        let cfg = ClientCredentialsConfig {
            tenant_id: "11111111-1111-1111-1111-111111111111".into(),
            client_id: "x".into(),
            client_secret: "y".into(),
            scope: "z".into(),
            token_endpoint_url: None,
            refresh_skew: Duration::from_secs(0),
        };
        assert_eq!(
            cfg.token_endpoint(),
            "https://login.microsoftonline.com/11111111-1111-1111-1111-111111111111/oauth2/v2.0/token"
        );
    }

    #[tokio::test]
    async fn mints_an_access_token_from_the_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth2/v2.0/token"))
            .and(body_string_contains("client_id=test-client-id"))
            .and(body_string_contains("client_secret=test-client-secret"))
            .and(body_string_contains("grant_type=client_credentials"))
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
            .and(path("/oauth2/v2.0/token"))
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
            .and(path("/oauth2/v2.0/token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "access_token": "first-token",
                "token_type": "Bearer",
                "expires_in": 60,
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/oauth2/v2.0/token"))
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
            .and(path("/oauth2/v2.0/token"))
            .respond_with(ResponseTemplate::new(400).set_body_string("invalid_request"))
            .mount(&server)
            .await;

        let clock: Arc<dyn Clock> = Arc::new(TestClock::new(fixed_now()));
        let source = token_source(&server.uri(), clock);

        let err = source.access_token().await.expect_err("must reject");
        assert!(!err.is_transient(), "expected permanent: {err:?}");
        match err {
            CommsError::ProviderHttp { status, .. } => assert_eq!(status, 400),
            other => panic!("unexpected variant: {other:?}"),
        }
    }

    #[tokio::test]
    async fn maps_5xx_to_transient_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/oauth2/v2.0/token"))
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
}
