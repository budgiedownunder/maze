//! Password-reset endpoints under `/api/v1/password-reset/`.
//!
//! Two unguarded routes:
//!   * `POST /api/v1/password-reset/request` — anti-enumeration request.
//!     Always returns 200 regardless of whether the email matches a user,
//!     whether the matching email is verified, or whether the user has a
//!     password to reset (OAuth-only accounts). Every request is recorded
//!     in the email audit log: known-recipient requests record a Pending
//!     row that the spawned send task flips to Accepted/Failed; the
//!     anti-enumeration unknown-email path records a Pending-only row
//!     with `recipient_user_id = None` (no send is attempted).
//!   * `POST /api/v1/password-reset/confirm` — consume token, update
//!     password, clear `user.logins`. Returns 204 on success, 400 on a
//!     missing/expired/already-consumed token or a weak new password.
//!
//! Reset tokens carry `TokenPurpose::PasswordReset` with a 1-hour TTL.
//! The token id is the secret carried in the reset link sent to the user;
//! storage is responsible for race-free single-use enforcement.

use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError},
    post, web, Error, HttpResponse,
};
use comms::{Comms, EmailAddress};
use data_model::{OneTimeToken, TokenPurpose};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use storage::{Error as StoreError, SharedStore};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::config::app::AppConfig;
use crate::service::audit::{record_and_dispatch, record_pending_only};
use crate::service::auth::AuthService;

/// Reset tokens live for one hour. Long enough that a user can step away
/// from their inbox briefly; short enough that a stolen email archive
/// doesn't yield a months-old usable token.
const RESET_TOKEN_TTL_HOURS: u32 = 1;

const RESET_TEMPLATE_ID: &str = "password_reset";
/// Path component appended to `[comms].public_base_url` to form the
/// reset link in the email body. The React/MAUI client owns the page at
/// this path; the link carries the token id as a query parameter.
const RESET_LINK_PATH: &str = "/reset-password";

// ---------------------------------------------------------------------------
// Request / response shapes
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct PasswordResetConfirmRequest {
    /// Token id carried by the reset link.
    pub token: String,
    pub new_password: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate_password_complexity(password: &str) -> Result<(), Error> {
    if password.len() < 8 {
        return Err(ErrorBadRequest(
            "Invalid request (password must be at least 8 characters)".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(ErrorBadRequest(
            "Invalid request (password must contain at least one uppercase letter)".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(ErrorBadRequest(
            "Invalid request (password must contain at least one lowercase letter)".to_string(),
        ));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(ErrorBadRequest(
            "Invalid request (password must contain at least one digit)".to_string(),
        ));
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(ErrorBadRequest(
            "Invalid request (password must contain at least one special character)".to_string(),
        ));
    }
    Ok(())
}

fn build_reset_link(public_base_url: &str, token_id: Uuid) -> String {
    let base = public_base_url.trim_end_matches('/');
    format!("{base}{RESET_LINK_PATH}?token={token_id}")
}

fn first_name_from(user: &data_model::User) -> String {
    let trimmed = user.full_name.trim();
    if !trimmed.is_empty() {
        trimmed.split_whitespace().next().unwrap_or(trimmed).to_string()
    } else {
        user.username.clone()
    }
}

// ---------------------------------------------------------------------------
// POST /api/v1/password-reset/request
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Request a password reset",
    description = "Always returns 200 to avoid leaking whether the email is registered. \
                   When the email matches a user with a verified address and a settable \
                   password, a one-time reset token is created and emailed via the \
                   configured comms provider. OAuth-only accounts (no password to reset) \
                   silently no-op.",
    post,
    path = "/api/v1/password-reset/request",
    request_body = PasswordResetRequest,
    responses(
        (status = 200, description = "Request accepted (whether or not a send actually happened)")
    ),
    tags = ["v1"]
)]
#[post("/password-reset/request")]
pub async fn request_password_reset(
    body: web::Json<PasswordResetRequest>,
    store: web::Data<SharedStore>,
    config: web::Data<AppConfig>,
    comms: web::Data<Comms>,
) -> Result<HttpResponse, Error> {
    let email = body.email.trim().to_string();

    // Anti-enumeration: even with an obviously-malformed email, hand back
    // the same 200 response we'd give for a real address. Provider-side
    // failures, missing users, OAuth-only users, and unverified emails
    // are all silent no-ops from the caller's perspective.
    if email.is_empty() {
        return Ok(HttpResponse::Ok().json(json!({"status": "ok"})));
    }

    let provider_name = comms.email_provider_name().unwrap_or("none");

    // Locate the user via verified-email lookup. Unverified rows are
    // already invisible to `find_user_by_verified_email` at the storage
    // layer, so we don't need a second filter here.
    let user_opt = {
        let store_lock = store.read().await;
        match store_lock.find_user_by_verified_email(&email).await {
            Ok(user) => Some(user),
            Err(StoreError::UserNotFound()) => None,
            Err(err) => {
                warn!("password-reset request: store lookup failed for {email:?}: {err}");
                // Fail closed but still hand the caller a 200 to avoid
                // leaking the failure mode.
                return Ok(HttpResponse::Ok().json(json!({"status": "ok"})));
            }
        }
    };

    let Some(user) = user_opt else {
        info!("password-reset request: no verified-email match for the supplied address");
        // Anti-enumeration recon row: we record the request itself with
        // `recipient_user_id = None` for rate-limit / abuse forensics,
        // even though no send fires.
        if let Err(err) = record_pending_only(
            store.get_ref().clone(),
            RESET_TEMPLATE_ID,
            &email,
            provider_name,
        )
        .await
        {
            warn!("password-reset request: recon audit row failed: {err}");
        }
        return Ok(HttpResponse::Ok().json(json!({"status": "ok"})));
    };

    // OAuth-only users have an empty `password_hash` — there is nothing
    // to reset, so the request silently no-ops. Returning 200 keeps
    // attacker-facing behaviour identical to the unknown-email path.
    if user.password_hash.is_empty() {
        info!(
            "password-reset request: skipping OAuth-only user '{}'",
            user.id
        );
        return Ok(HttpResponse::Ok().json(json!({"status": "ok"})));
    }

    // Issue the token. Storage owns single-use enforcement and expiry
    // bookkeeping; we only need to plant a fresh row.
    let token = OneTimeToken::new(user.id, TokenPurpose::PasswordReset, None, RESET_TOKEN_TTL_HOURS);
    {
        let mut store_lock = store.write().await;
        if let Err(err) = store_lock.create_token(&token).await {
            warn!("password-reset request: create_token failed for user {}: {err}", user.id);
            return Ok(HttpResponse::Ok().json(json!({"status": "ok"})));
        }
    }

    // Record the audit row + dispatch the send on a fire-and-forget
    // task. `record_and_dispatch` writes a Pending row first, then the
    // spawned task updates it to Accepted (with provider_message_id) on
    // success or Failed (with an error_class) on provider failure.
    let reset_link = build_reset_link(&config.comms.public_base_url, token.id);
    let to = EmailAddress::with_name(user.email().to_string(), user.full_name.clone());
    let context = json!({
        "first_name": first_name_from(&user),
        "username": user.username,
        "full_name": user.full_name,
        "email": user.email(),
        "reset_link": reset_link,
    });
    let user_id = user.id;
    if let Err(err) = record_and_dispatch(
        store.get_ref().clone(),
        comms.clone().into_inner(),
        RESET_TEMPLATE_ID,
        Some(user_id),
        None,
        Some(token.id),
        to,
        context,
    )
    .await
    {
        warn!("password-reset request: record_and_dispatch failed for user {user_id}: {err}");
    }

    Ok(HttpResponse::Ok().json(json!({"status": "ok"})))
}

// ---------------------------------------------------------------------------
// POST /api/v1/password-reset/confirm
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Confirm a password reset",
    description = "Consumes the one-time token, updates the user's password, and \
                   invalidates every active login session. Returns 204 on success, \
                   400 on an invalid / expired / already-consumed token or a weak \
                   new password.",
    post,
    path = "/api/v1/password-reset/confirm",
    request_body = PasswordResetConfirmRequest,
    responses(
        (status = 204, description = "Password updated"),
        (status = 400, description = "Invalid token or weak password")
    ),
    tags = ["v1"]
)]
#[post("/password-reset/confirm")]
pub async fn confirm_password_reset(
    body: web::Json<PasswordResetConfirmRequest>,
    store: web::Data<SharedStore>,
    auth_service: web::Data<AuthService>,
) -> Result<HttpResponse, Error> {
    let token_id = Uuid::parse_str(body.token.trim())
        .map_err(|_| ErrorBadRequest("Invalid or expired reset token"))?;

    validate_password_complexity(&body.new_password)?;

    let new_password_hash = auth_service.hash_password(&body.new_password).map_err(|err| {
        warn!("password-reset confirm: hash failed: {err}");
        ErrorInternalServerError("Failed to update password")
    })?;

    let mut store_lock = store.write().await;
    let consumed = store_lock.consume_token(token_id).await.map_err(|err| match err {
        StoreError::TokenIdNotFound(_)
        | StoreError::TokenAlreadyConsumed()
        | StoreError::TokenExpired() => ErrorBadRequest("Invalid or expired reset token"),
        other => {
            warn!("password-reset confirm: consume_token failed: {other}");
            ErrorInternalServerError("Failed to consume reset token")
        }
    })?;

    if consumed.purpose != TokenPurpose::PasswordReset {
        return Err(ErrorBadRequest("Invalid or expired reset token"));
    }

    // Load the user, rotate the password, and clear every active login.
    // A reset is a "this account may be compromised" signal — preserving
    // existing sessions would let an attacker with a stolen bearer token
    // outlast the password change.
    let mut user = store_lock.get_user(consumed.user_id).await.map_err(|err| match err {
        StoreError::UserIdNotFound(_) => ErrorBadRequest("Invalid or expired reset token"),
        other => {
            warn!("password-reset confirm: get_user failed: {other}");
            ErrorInternalServerError("Failed to update password")
        }
    })?;

    user.password_hash = new_password_hash;
    user.logins.clear();
    store_lock.update_user(&mut user).await.map_err(|err| {
        warn!("password-reset confirm: update_user failed: {err}");
        ErrorInternalServerError("Failed to update password")
    })?;

    info!("password-reset confirm succeeded for user {}", user.id);
    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_reset_link_appends_token_query() {
        let link = build_reset_link("https://maze.example.com", Uuid::nil());
        assert_eq!(
            link,
            "https://maze.example.com/reset-password?token=00000000-0000-0000-0000-000000000000"
        );
    }

    #[test]
    fn build_reset_link_strips_trailing_slash() {
        let link = build_reset_link("https://maze.example.com/", Uuid::nil());
        assert!(
            !link.contains("//reset-password"),
            "must not double-slash the join: {link}"
        );
    }

    #[test]
    fn first_name_uses_first_word_of_full_name() {
        let user = data_model::User {
            full_name: "Alice Wonderland".to_string(),
            username: "alice".to_string(),
            ..data_model::User::default()
        };
        assert_eq!(first_name_from(&user), "Alice");
    }

    #[test]
    fn first_name_falls_back_to_username_when_full_name_blank() {
        let user = data_model::User {
            full_name: String::new(),
            username: "alice".to_string(),
            ..data_model::User::default()
        };
        assert_eq!(first_name_from(&user), "alice");
    }
}
