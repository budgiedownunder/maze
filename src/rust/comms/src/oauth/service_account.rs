use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};

use crate::error::CommsError;
use crate::oauth::clock::{Clock, SystemClock};
use crate::oauth::OAuthTokenSource;

/// Configuration for `ServiceAccountTokenSource`. Carries the fields needed
/// to construct a JWT-bearer assertion against Google's token endpoint:
///
/// - `client_email` — the service account address; used as the JWT `iss`
///   claim (and as `sub` if `subject` is `None`).
/// - `private_key_pem` — RSA private key in PKCS#8 PEM. Validated when
///   `ServiceAccountTokenSource::new()` is called; bad PEMs surface there.
/// - `private_key_id` — Google's `kid` for the JWT header. Optional.
/// - `token_uri` — the token endpoint URL; also the JWT `aud` claim.
/// - `scopes` — joined with spaces for the JWT `scope` claim.
/// - `subject` — domain-wide delegation: the user being impersonated.
///   Set this for "send mail as `noreply@company.com`" Workspace flows.
/// - `assertion_lifetime` — claim `exp - iat`. Google caps at 3600 s.
/// - `refresh_skew` — shorten the cache lifetime by this much, ensuring
///   the source returns a token that still has at least `refresh_skew` of
///   remaining validity from the resource server's perspective.
pub struct ServiceAccountConfig {
    pub client_email: String,
    pub private_key_pem: String,
    pub private_key_id: Option<String>,
    pub token_uri: String,
    pub scopes: Vec<String>,
    pub subject: Option<String>,
    pub assertion_lifetime: Duration,
    pub refresh_skew: Duration,
}

#[derive(Debug, Deserialize)]
struct GoogleServiceAccountJson {
    client_email: String,
    private_key: String,
    token_uri: String,
    #[serde(default)]
    private_key_id: Option<String>,
}

impl ServiceAccountConfig {
    /// Parse a Google service-account JSON key file. `scopes` are the
    /// Google API scopes the token must cover (e.g.
    /// `["https://www.googleapis.com/auth/gmail.send"]`). `subject` is set
    /// separately if domain-wide delegation is in play.
    ///
    /// Missing or empty `private_key`, `client_email`, or `token_uri`
    /// fields surface as `CommsError::Config` here, not at first send.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms::ServiceAccountConfig;
    ///
    /// let json = r#"{
    ///   "client_email": "svc@project.iam.gserviceaccount.com",
    ///   "private_key": "-----BEGIN PRIVATE KEY-----\nstub\n-----END PRIVATE KEY-----\n",
    ///   "token_uri": "https://oauth2.googleapis.com/token",
    ///   "private_key_id": "abc123"
    /// }"#;
    /// let cfg = ServiceAccountConfig::from_json_str(
    ///     json,
    ///     vec!["https://www.googleapis.com/auth/gmail.send".into()],
    /// )
    /// .expect("parse JSON");
    /// assert_eq!(cfg.client_email, "svc@project.iam.gserviceaccount.com");
    /// assert!(cfg.subject.is_none());
    /// ```
    pub fn from_json_str(json: &str, scopes: Vec<String>) -> Result<Self, CommsError> {
        let parsed: GoogleServiceAccountJson = serde_json::from_str(json)
            .map_err(|e| CommsError::Config(format!("service account JSON: {e}")))?;
        if parsed.private_key.trim().is_empty() {
            return Err(CommsError::Config(
                "service account JSON missing 'private_key'".into(),
            ));
        }
        if parsed.client_email.trim().is_empty() {
            return Err(CommsError::Config(
                "service account JSON missing 'client_email'".into(),
            ));
        }
        if parsed.token_uri.trim().is_empty() {
            return Err(CommsError::Config(
                "service account JSON missing 'token_uri'".into(),
            ));
        }
        Ok(Self {
            client_email: parsed.client_email,
            private_key_pem: parsed.private_key,
            private_key_id: parsed.private_key_id,
            token_uri: parsed.token_uri,
            scopes,
            subject: None,
            assertion_lifetime: Duration::from_secs(3600),
            refresh_skew: Duration::from_secs(60),
        })
    }
}

#[derive(Debug, Serialize)]
struct JwtClaims<'a> {
    iss: &'a str,
    scope: String,
    aud: &'a str,
    iat: i64,
    exp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    sub: Option<&'a str>,
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

/// OAuth2 JWT-bearer flow against Google's token endpoint. The assertion is
/// signed with the service account's RSA private key and exchanged for an
/// access token. The token is cached in-memory until it expires; no
/// refresh tokens.
pub struct ServiceAccountTokenSource {
    config: ServiceAccountConfig,
    encoding_key: EncodingKey,
    http: reqwest::Client,
    clock: Arc<dyn Clock>,
    cached: Mutex<Option<CachedToken>>,
}

impl ServiceAccountTokenSource {
    /// Construct with the default `reqwest` client and `SystemClock`. The
    /// PEM is validated up-front so a malformed `private_key_pem` surfaces
    /// here rather than at first send.
    ///
    /// # Examples
    ///
    /// Shape of the call (no_run because a real RSA PEM is required for
    /// the `EncodingKey` validation to succeed)
    /// ```no_run
    /// use comms::{ServiceAccountConfig, ServiceAccountTokenSource};
    ///
    /// let json = std::fs::read_to_string("svc-account.json")
    ///     .expect("read service-account JSON");
    /// let cfg = ServiceAccountConfig::from_json_str(
    ///     &json,
    ///     vec!["https://www.googleapis.com/auth/gmail.send".into()],
    /// )
    /// .expect("parse JSON");
    /// let _source = ServiceAccountTokenSource::new(cfg).expect("build");
    /// ```
    pub fn new(config: ServiceAccountConfig) -> Result<Self, CommsError> {
        let encoding_key = EncodingKey::from_rsa_pem(config.private_key_pem.as_bytes())
            .map_err(|e| CommsError::Config(format!("service account private_key: {e}")))?;
        let http = reqwest::Client::builder()
            .build()
            .map_err(|e| CommsError::Config(format!("reqwest client: {e}")))?;
        Ok(Self::with_http_and_clock(
            config,
            encoding_key,
            http,
            Arc::new(SystemClock),
        ))
    }

    pub(crate) fn with_http_and_clock(
        config: ServiceAccountConfig,
        encoding_key: EncodingKey,
        http: reqwest::Client,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            config,
            encoding_key,
            http,
            clock,
            cached: Mutex::new(None),
        }
    }

    fn build_assertion(&self, now: DateTime<Utc>) -> Result<String, CommsError> {
        let mut header = Header::new(Algorithm::RS256);
        header.kid = self.config.private_key_id.clone();
        let lifetime = chrono::Duration::from_std(self.config.assertion_lifetime)
            .unwrap_or_else(|_| chrono::Duration::seconds(3600));
        let exp = (now + lifetime).timestamp();
        let claims = JwtClaims {
            iss: &self.config.client_email,
            scope: self.config.scopes.join(" "),
            aud: &self.config.token_uri,
            iat: now.timestamp(),
            exp,
            sub: self.config.subject.as_deref(),
        };
        jsonwebtoken::encode(&header, &claims, &self.encoding_key)
            .map_err(|e| CommsError::Provider(format!("jwt encode: {e}")))
    }

    async fn mint(&self) -> Result<CachedToken, CommsError> {
        let now = self.clock.now();
        let assertion = self.build_assertion(now)?;
        let response = self
            .http
            .post(&self.config.token_uri)
            .form(&[
                ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
                ("assertion", assertion.as_str()),
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
impl OAuthTokenSource for ServiceAccountTokenSource {
    async fn access_token(&self) -> Result<String, CommsError> {
        let now = self.clock.now();
        {
            let guard = self
                .cached
                .lock()
                .expect("service account cache poisoned");
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
            .expect("service account cache poisoned");
        *guard = Some(fresh);
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oauth::clock::TestClock;
    use chrono::TimeZone;
    use jsonwebtoken::{Algorithm, DecodingKey, Validation};
    use pretty_assertions::assert_eq;
    use serde::Deserialize;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    // Test-only RSA keypair (PKCS#8 PEM). Not used in production: the
    // surrounding `mod tests` is gated by `#[cfg(test)]`, so this constant is
    // excluded from release builds and from the public API entirely.
    const TEST_PRIVATE_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQC9aR0FEhxKOpqm
3/vvxcbDrvr0hwztH2Og/EP17HDuLj6Z4mqlyIWEGRhPouSSA/ZvqiHNnyhGEIMT
w4RDYSP/0deUjt6uTrExOISYJKc3WD7x4ooiegaxkS3g3RWWJi8lNLbkHEpflhOp
NGkRiFN4AywOJoZHWDA7eOUfkJmOBtWB+3N+bWQDl4Hcr+GMXuMLwcFOith+vytN
A0UtGuvocrl7QoYu+86A0L5bMpc8V/6JiajV5nf/8BETWKfrLRpHn2o+mXxCcG8/
htepzJ8RSfIp/kSpO5Q0Kag8YJYIfhhiyqkf4numIBKRTChAFfeVl3vgYqHc2A+B
ydaM/O2FAgMBAAECggEACxXJRIA0VqXKYkTOjlBRWyd0+XWj7Ia/Qm8xHQXq8A2V
zVTCcBRlt/7t/M8oOGFx/UQEOW/8n+kcer4hEf5v6GtkBgY8gxAI77wCipLulF8e
Q/LBqdXhaWf5OuVFe6Wdcbx9jakYMzLp0KfIFGYZmHb7D5Lynd9L51cida/1RKoI
7sr64zy1VidbIfNkDq5eiIiJnRKv9gegoB86E3KZjPvzuIItSF3+j86bQVO+LN0j
KWO5h16PHvjRNx4n/TwgtiMFXaMXo9Q43+Gni/BLXCEI/bHi1als58S3VPZQNqte
HtlxBb9L2gJKP4R/0TRIN+wMbqvYBNMFMjfHOMXXcQKBgQDkrOl9mv8h8CtuQUaV
c+B8pAQUAjHCsm4B+LU7iEKI4Fmh/ebjLBEYAoLgdGKq8g097EtGU7iyIcKbfeaL
FH4erNPjR8MrsA4ZgdnUKU41GlEs9CuYyL7DJkJBVQcUgkPzTspHoq42oVNoObNo
b1o+3dqT2nd6XWTinMKxDc0EDQKBgQDUCxsgOTmO4AQJG49XXuocqJy6gZ+qZB9T
dkmvoHY0ich7z1PXb8Jezck9lbegscreFI7I0nePXl2AzR6bj6Az0c+IHydkKqv5
bQNAu/DT/nzRLPQ9+m8Yl4YUy6yXc3/wzWoliiGkJKkj1Kj6tEKNJy8d1pP5CD2t
kBjFagZZWQKBgGCAnBSWuX6QBTQFNg1SFnVjHhl3h5pbhFMuqwTRjwqGay0SokJS
UXBpduPUGeN8PJxaQLYQFMyPtLm72vPslQDK/KxYl4OzS2/2PX/sYoXEcmdfL5rN
dLuURLefc1pzUsu1/2VVwOFrGXDNkOnMvC/1ng1xT6SDD1UWxI7FfTRtAoGBANPB
Y42CmGB+hokx5Kw0NUf5essmt/TJmB8ZeezSKjm9f2FlYy06hrl2eQnvgjoQU7AE
h7M1vACJFIeUUIS5ohsd5ErkEcqOcr/chesXxSFwe+XJJwDeICRG7bfGzs1Qouwv
t1lV4NKzadZGgZocennML9l0eMGx4SZ7SMGdaEnBAoGAY/XNlpjerwexG+SX0JMi
i0gR3YnYmZFU2/Dbu3ttGwNSQ0/HbbAmNZADHZH6dBXy77Dt4sDQ8t7r1cSl5lw6
fz9ahPFfORmz/nqAs+az3elpZQgbgWMF8TRojqkKSTp0TPQqLZwtJ6TLWWXVctTv
dw+oAc9Da79GdzCTqO93/cQ=
-----END PRIVATE KEY-----
";

    const TEST_PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAvWkdBRIcSjqapt/778XG
w6769IcM7R9joPxD9exw7i4+meJqpciFhBkYT6LkkgP2b6ohzZ8oRhCDE8OEQ2Ej
/9HXlI7erk6xMTiEmCSnN1g+8eKKInoGsZEt4N0VliYvJTS25BxKX5YTqTRpEYhT
eAMsDiaGR1gwO3jlH5CZjgbVgftzfm1kA5eB3K/hjF7jC8HBTorYfr8rTQNFLRrr
6HK5e0KGLvvOgNC+WzKXPFf+iYmo1eZ3//ARE1in6y0aR59qPpl8QnBvP4bXqcyf
EUnyKf5EqTuUNCmoPGCWCH4YYsqpH+J7piASkUwoQBX3lZd74GKh3NgPgcnWjPzt
hQIDAQAB
-----END PUBLIC KEY-----
";

    #[derive(Debug, Deserialize)]
    struct DecodedClaims {
        iss: String,
        scope: String,
        aud: String,
        iat: i64,
        exp: i64,
        #[serde(default)]
        sub: Option<String>,
    }

    fn config_against(server_uri: &str) -> ServiceAccountConfig {
        ServiceAccountConfig {
            client_email: "test-svc@example.iam.gserviceaccount.com".into(),
            private_key_pem: TEST_PRIVATE_KEY_PEM.into(),
            private_key_id: Some("test-key-id".into()),
            token_uri: format!("{server_uri}/token"),
            scopes: vec!["https://www.googleapis.com/auth/gmail.send".into()],
            subject: None,
            assertion_lifetime: Duration::from_secs(3600),
            refresh_skew: Duration::from_secs(0),
        }
    }

    fn fixed_now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 4, 12, 0, 0).unwrap()
    }

    fn build_source(
        config: ServiceAccountConfig,
        clock: Arc<dyn Clock>,
    ) -> ServiceAccountTokenSource {
        let encoding_key =
            EncodingKey::from_rsa_pem(config.private_key_pem.as_bytes()).expect("test PEM");
        ServiceAccountTokenSource::with_http_and_clock(
            config,
            encoding_key,
            reqwest::Client::new(),
            clock,
        )
    }

    /// base64url alphabet is `[A-Za-z0-9_-]`; form encoding leaves it
    /// untouched, so the assertion field value is byte-identical to the JWT.
    fn parse_form_field<'a>(body: &'a str, field: &str) -> &'a str {
        for pair in body.split('&') {
            if let Some(rest) = pair.strip_prefix(field) {
                if let Some(value) = rest.strip_prefix('=') {
                    return value;
                }
            }
        }
        panic!("form field '{field}' not found in body: {body}");
    }

    #[tokio::test]
    async fn jwt_assertion_has_expected_shape_and_signature() {
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

        let mut cfg = config_against(&server.uri());
        cfg.subject = Some("noreply@company.com".into());
        let clock: Arc<dyn Clock> = Arc::new(TestClock::new(fixed_now()));
        let source = build_source(cfg, clock);
        let _ = source.access_token().await.expect("first mint");

        let received = server
            .received_requests()
            .await
            .expect("requests recorded");
        assert_eq!(received.len(), 1);
        let body = std::str::from_utf8(&received[0].body).expect("utf8 body");
        let assertion = parse_form_field(body, "assertion");

        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = false;
        validation.set_audience(&[format!("{}/token", server.uri())]);
        let decoding =
            DecodingKey::from_rsa_pem(TEST_PUBLIC_KEY_PEM.as_bytes()).expect("public PEM");
        let token_data =
            jsonwebtoken::decode::<DecodedClaims>(assertion, &decoding, &validation)
                .expect("JWT signature validates");

        assert_eq!(token_data.header.alg, Algorithm::RS256);
        assert_eq!(token_data.header.kid.as_deref(), Some("test-key-id"));
        assert_eq!(
            token_data.claims.iss,
            "test-svc@example.iam.gserviceaccount.com"
        );
        assert_eq!(
            token_data.claims.scope,
            "https://www.googleapis.com/auth/gmail.send"
        );
        assert_eq!(token_data.claims.aud, format!("{}/token", server.uri()));
        assert_eq!(token_data.claims.iat, fixed_now().timestamp());
        assert_eq!(token_data.claims.exp, fixed_now().timestamp() + 3600);
        assert_eq!(
            token_data.claims.sub.as_deref(),
            Some("noreply@company.com")
        );
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
        let source = build_source(config_against(&server.uri()), clock);

        let first = source.access_token().await.expect("first");
        let second = source.access_token().await.expect("second");
        assert_eq!(first, "the-token");
        assert_eq!(second, "the-token");
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
        let source = build_source(config_against(&server.uri()), clock_dyn);

        let first = source.access_token().await.expect("first");
        assert_eq!(first, "first-token");

        clock.advance(chrono::Duration::seconds(120));

        let second = source.access_token().await.expect("second");
        assert_eq!(second, "second-token");
    }

    #[test]
    fn from_json_str_rejects_missing_private_key() {
        let json = serde_json::json!({
            "client_email": "x@example.iam.gserviceaccount.com",
            "private_key": "",
            "token_uri": "https://oauth2.googleapis.com/token",
        });
        match ServiceAccountConfig::from_json_str(&json.to_string(), vec!["scope".into()]) {
            Err(CommsError::Config(msg)) => assert!(msg.contains("private_key"), "{msg}"),
            Err(other) => panic!("unexpected variant: {other:?}"),
            Ok(_) => panic!("expected Err"),
        }
    }

    #[test]
    fn from_json_str_rejects_missing_client_email() {
        let json = serde_json::json!({
            "client_email": "",
            "private_key": "-----BEGIN PRIVATE KEY-----\nabc\n-----END PRIVATE KEY-----\n",
            "token_uri": "https://oauth2.googleapis.com/token",
        });
        match ServiceAccountConfig::from_json_str(&json.to_string(), vec!["scope".into()]) {
            Err(CommsError::Config(msg)) => assert!(msg.contains("client_email"), "{msg}"),
            Err(other) => panic!("unexpected variant: {other:?}"),
            Ok(_) => panic!("expected Err"),
        }
    }

    #[test]
    fn new_rejects_malformed_private_key_pem() {
        let cfg = ServiceAccountConfig {
            client_email: "x@example.iam.gserviceaccount.com".into(),
            private_key_pem:
                "-----BEGIN PRIVATE KEY-----\nnot a real key\n-----END PRIVATE KEY-----\n"
                    .into(),
            private_key_id: None,
            token_uri: "https://oauth2.googleapis.com/token".into(),
            scopes: vec!["scope".into()],
            subject: None,
            assertion_lifetime: Duration::from_secs(3600),
            refresh_skew: Duration::from_secs(0),
        };
        match ServiceAccountTokenSource::new(cfg) {
            Err(CommsError::Config(msg)) => assert!(msg.contains("private_key"), "{msg}"),
            Err(other) => panic!("unexpected variant: {other:?}"),
            Ok(_) => panic!("expected Err"),
        }
    }
}
