//! Email-verification endpoints under `/api/v1/email-verifications/`.
//!
//! Two routes:
//!   * `POST /api/v1/email-verifications/request` — bearer-auth re-send
//!     of the verification email for a specified address on the caller's
//!     account. Idempotent: requesting verification for an
//!     already-verified row is a 200 no-op (no send). Re-issuing
//!     supersedes any prior outstanding token for the same (user, email)
//!     — only the most recent link works.
//!   * `POST /api/v1/email-verifications/confirm` — unguarded. Consumes
//!     the token, flips the targeted `user_emails` row to
//!     `verified = true, verified_at = now()`. Cross-user attacks fail
//!     because the consumed token's `user_id` is the source of truth for
//!     which user's email row to flip.
//!
//! The audit-log integration that records every verification attempt
//! (whether or not a send happens) lands in Step 3.8.
//!
//! Verification tokens carry [`TokenPurpose::EmailVerification`] with a
//! 24-hour TTL and a `target_email` populated.

use std::sync::Arc;

use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized},
    post, web, Error, HttpMessage, HttpRequest, HttpResponse,
};
use comms::{Comms, EmailAddress};
use data_model::{OneTimeToken, TokenPurpose, User};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use serde_json::json;
use storage::{Error as StoreError, SharedStore};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::config::app::AppConfig;

const VERIFICATION_TOKEN_TTL_HOURS: u32 = 24;
const VERIFICATION_TEMPLATE_ID: &str = "email_verification";
/// Path component appended to `[comms].public_base_url` to form the
/// verification link in the email body. The React/MAUI client owns the
/// page at this path; the link carries the token id as a query
/// parameter.
const VERIFICATION_LINK_PATH: &str = "/verify-email";

// ---------------------------------------------------------------------------
// Request / response shapes
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct EmailVerificationRequest {
    /// Email address on the caller's account to (re-)send verification for.
    pub email: String,
}

#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct EmailVerificationConfirmRequest {
    /// Token id carried by the verification link.
    pub token: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn get_authorized_user(req: &HttpRequest) -> Result<User, Error> {
    req.extensions()
        .get::<User>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized("Unauthorized request"))
}

fn build_verification_link(public_base_url: &str, token_id: Uuid) -> String {
    let base = public_base_url.trim_end_matches('/');
    format!("{base}{VERIFICATION_LINK_PATH}?token={token_id}")
}

fn first_name_from(user: &User) -> String {
    let trimmed = user.full_name.trim();
    if !trimmed.is_empty() {
        trimmed.split_whitespace().next().unwrap_or(trimmed).to_string()
    } else {
        user.username.clone()
    }
}

/// Creates a fresh verification token for `(user_id, email)`, after
/// purging any prior outstanding token for the same pair so re-issuing
/// supersedes the previous link. Returns the new token on success.
pub(crate) async fn issue_verification_token(
    store: &SharedStore,
    user_id: Uuid,
    email: &str,
) -> Result<OneTimeToken, StoreError> {
    let mut store_lock = store.write().await;
    // Supersede any outstanding tokens for this (user, email) pair so
    // re-issuance invalidates earlier links.
    store_lock
        .purge_email_verification_tokens(user_id, email)
        .await?;
    let token = OneTimeToken::new(
        user_id,
        TokenPurpose::EmailVerification,
        Some(email.to_string()),
        VERIFICATION_TOKEN_TTL_HOURS,
    );
    store_lock.create_token(&token).await?;
    Ok(token)
}

/// Renders the verification email and dispatches it via `Comms` on a
/// fire-and-forget task. Errors from the spawn are logged, not surfaced
/// — the verification flow is self-service so the sender response
/// already returned by the time the send resolves.
pub(crate) fn dispatch_verification_email(
    comms: Arc<Comms>,
    user: User,
    email: &str,
    public_base_url: &str,
    token_id: Uuid,
) {
    let verification_link = build_verification_link(public_base_url, token_id);
    let to = EmailAddress::with_name(email.to_string(), user.full_name.clone());
    let context = json!({
        "first_name": first_name_from(&user),
        "username": user.username,
        "full_name": user.full_name,
        "email": email,
        "verification_link": verification_link,
    });
    let user_id = user.id;
    let target = email.to_string();
    tokio::spawn(async move {
        match comms
            .send_template(VERIFICATION_TEMPLATE_ID, to, &context)
            .await
        {
            Ok(_) => info!(
                "email-verification send succeeded for user {user_id} address {target}"
            ),
            Err(err) => warn!(
                "email-verification send failed for user {user_id} address {target}: {err}"
            ),
        }
    });
}

// ---------------------------------------------------------------------------
// POST /api/v1/email-verifications/request
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Request a verification email",
    description = "Re-sends a verification email for an address attached to the \
                   authenticated user's account. Idempotent: requesting verification \
                   for an already-verified row returns 200 with no send. Re-issuance \
                   supersedes any prior outstanding token — only the most recent \
                   link works.",
    post,
    path = "/api/v1/email-verifications/request",
    request_body = EmailVerificationRequest,
    responses(
        (status = 200, description = "Request accepted (whether or not a send actually happened)"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "The address is not attached to the caller's account")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/email-verifications/request")]
pub async fn request_email_verification(
    body: web::Json<EmailVerificationRequest>,
    store: web::Data<SharedStore>,
    config: web::Data<AppConfig>,
    comms: web::Data<Comms>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let caller = get_authorized_user(&req)?;
    let target = body.email.trim().to_string();
    if target.is_empty() {
        return Err(ErrorBadRequest("Invalid request (missing email)"));
    }

    // Locate the row on the caller's account.
    let row_email_opt = caller
        .emails
        .iter()
        .find(|e| e.email.eq_ignore_ascii_case(&target))
        .map(|e| (e.email.clone(), e.verified));
    let Some((row_email, row_verified)) = row_email_opt else {
        return Err(ErrorNotFound(
            "Email is not attached to the caller's account",
        ));
    };

    if row_verified {
        // Idempotent: already-verified is a 200 no-op. We deliberately
        // do not issue a fresh token (no point) and do not send a
        // message (operational noise + spam risk).
        return Ok(HttpResponse::Ok().json(json!({"status": "ok"})));
    }

    let token = issue_verification_token(&store, caller.id, &row_email)
        .await
        .map_err(|err| {
            warn!(
                "email-verification request: token issue failed for user {} email {}: {err}",
                caller.id, row_email
            );
            ErrorInternalServerError("Failed to start email verification")
        })?;

    dispatch_verification_email(
        comms.clone().into_inner(),
        caller,
        &row_email,
        &config.comms.public_base_url,
        token.id,
    );

    Ok(HttpResponse::Ok().json(json!({"status": "ok"})))
}

// ---------------------------------------------------------------------------
// POST /api/v1/email-verifications/confirm
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Confirm an email verification",
    description = "Consumes the one-time token and flips the targeted \
                   `user_emails` row to `verified = true, verified_at = now()`. \
                   Cross-user attacks fail because the consumed token's \
                   `user_id` is the source of truth.",
    post,
    path = "/api/v1/email-verifications/confirm",
    request_body = EmailVerificationConfirmRequest,
    responses(
        (status = 204, description = "Email verified"),
        (status = 400, description = "Invalid / expired / already-consumed token")
    ),
    tags = ["v1"]
)]
#[post("/email-verifications/confirm")]
pub async fn confirm_email_verification(
    body: web::Json<EmailVerificationConfirmRequest>,
    store: web::Data<SharedStore>,
) -> Result<HttpResponse, Error> {
    let token_id = Uuid::parse_str(body.token.trim())
        .map_err(|_| ErrorBadRequest("Invalid or expired verification token"))?;

    let mut store_lock = store.write().await;
    let consumed = store_lock
        .consume_token(token_id)
        .await
        .map_err(|err| match err {
            StoreError::TokenIdNotFound(_)
            | StoreError::TokenAlreadyConsumed()
            | StoreError::TokenExpired() => {
                ErrorBadRequest("Invalid or expired verification token")
            }
            other => {
                warn!("email-verification confirm: consume_token failed: {other}");
                ErrorInternalServerError("Failed to consume verification token")
            }
        })?;

    if consumed.purpose != TokenPurpose::EmailVerification {
        return Err(ErrorBadRequest("Invalid or expired verification token"));
    }
    let Some(target) = consumed.target_email.as_deref() else {
        return Err(ErrorBadRequest("Invalid or expired verification token"));
    };

    // The token's user_id is authoritative — it doesn't matter which
    // bearer (if any) the caller is using, the email-row flip happens on
    // the user identified by the token. That defeats cross-user attacks.
    store_lock
        .mark_email_verified(consumed.user_id, target)
        .await
        .map_err(|err| match err {
            StoreError::UserEmailNotFound(_) | StoreError::UserIdNotFound(_) => {
                // Row was deleted between issue and confirm. The token
                // is "valid" but the address it pointed at is gone.
                ErrorBadRequest("Invalid or expired verification token")
            }
            other => {
                warn!("email-verification confirm: mark_email_verified failed: {other}");
                ErrorInternalServerError("Failed to verify email")
            }
        })?;

    info!(
        "email-verification confirm succeeded for user {} email {}",
        consumed.user_id, target
    );
    Ok(HttpResponse::NoContent().finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_verification_link_appends_token_query() {
        let link = build_verification_link("https://maze.example.com", Uuid::nil());
        assert_eq!(
            link,
            "https://maze.example.com/verify-email?token=00000000-0000-0000-0000-000000000000"
        );
    }

    #[test]
    fn build_verification_link_strips_trailing_slash() {
        let link = build_verification_link("https://maze.example.com/", Uuid::nil());
        assert!(
            !link.contains("//verify-email"),
            "must not double-slash: {link}"
        );
    }

    #[test]
    fn first_name_uses_first_word_of_full_name() {
        let user = User {
            full_name: "Alice Wonderland".to_string(),
            username: "alice".to_string(),
            ..User::default()
        };
        assert_eq!(first_name_from(&user), "Alice");
    }

    #[test]
    fn first_name_falls_back_to_username_when_full_name_blank() {
        let user = User {
            full_name: String::new(),
            username: "alice".to_string(),
            ..User::default()
        };
        assert_eq!(first_name_from(&user), "alice");
    }
}
