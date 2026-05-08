//! OAuth2 token sources used by `comms` providers that authenticate via a
//! bearer token (Microsoft Graph, Gmail API, etc.).
//!
//! The `OAuthTokenSource` trait is the abstraction providers depend on. Each
//! flow lives in a sibling module gated by its own feature flag:
//!
//! - `client_credentials` (`oauth2-microsoft`) — Microsoft Azure AD
//!   client_credentials flow for app-only access to a fixed mailbox.
//! - `service_account` (`oauth2-google`) — Google Workspace JWT-bearer
//!   flow with optional domain-wide delegation.
//! - `refresh_token` (`oauth2-refresh-token`) — generic OAuth2
//!   refresh-token flow, used by per-user accounts where the consent
//!   dance happens once out-of-band and the long-lived refresh token is
//!   fed to the server as a secret.
//!
//! `RefreshTokenStore` is a trait surface only — no implementations ship
//! today. It exists so that, if a future flow needs persisted refresh
//! tokens (multi-tenant SaaS, end-user OAuth), the abstraction is already
//! in place.

mod clock;
mod refresh_store;

pub use clock::{Clock, SystemClock};
pub use refresh_store::{RefreshToken, RefreshTokenStore};

#[cfg(feature = "oauth2-microsoft")]
pub mod client_credentials;
#[cfg(feature = "oauth2-microsoft")]
pub use client_credentials::{ClientCredentialsConfig, ClientCredentialsTokenSource};

#[cfg(feature = "oauth2-google")]
pub mod service_account;
#[cfg(feature = "oauth2-google")]
pub use service_account::{ServiceAccountConfig, ServiceAccountTokenSource};

#[cfg(feature = "oauth2-refresh-token")]
pub mod refresh_token;
#[cfg(feature = "oauth2-refresh-token")]
pub use refresh_token::{RefreshTokenConfig, RefreshTokenTokenSource};

use async_trait::async_trait;

use crate::error::CommsError;

/// Source of a fresh OAuth2 access token. Implementations cache tokens up
/// to their expiry and re-mint on demand. Calls within the cache window
/// are local; the first call after expiry hits the network.
#[async_trait]
pub trait OAuthTokenSource: Send + Sync {
    async fn access_token(&self) -> Result<String, CommsError>;
}
