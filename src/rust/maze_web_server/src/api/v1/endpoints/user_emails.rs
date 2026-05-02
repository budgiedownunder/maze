//! Email-management endpoints under `/api/v1/users/me/emails`.
//!
//! All routes require authentication and operate on the caller's own
//! emails. The shape of the requests/responses is symmetric across the
//! five handlers — list, add, delete, set-primary, verify (stub).
//!
//! Storage-level errors are mapped to clean 4xx HTTP responses:
//!   * `UserEmailExists`        → 409 Conflict
//!   * `UserEmailIsLast`        → 409 Conflict (with explanatory body)
//!   * `UserEmailIsPrimary`     → 409 Conflict (with explanatory body)
//!   * `UserEmailNotVerified`   → 409 Conflict (cannot promote unverified)
//!   * `UserEmailNotFound(_)`   → 404 Not Found
//!   * `UserEmailMissing` /
//!     `UserEmailInvalid`       → 400 Bad Request

use actix_web::{
    delete, get, post, put, web, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{ErrorBadRequest, ErrorConflict, ErrorInternalServerError, ErrorNotFound,
            ErrorUnauthorized, ErrorNotImplemented},
};
use data_model::{User, UserEmail};
use serde::{Deserialize, Serialize};
use storage::{Error as StoreError, SharedStore};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use utoipa::ToSchema;

// ---------------------------------------------------------------------------
// Local helpers (mirror the private utilities in handlers.rs to keep this
// module self-contained without touching handler-private code paths).
// ---------------------------------------------------------------------------

fn get_authorized_user(req: &HttpRequest) -> Result<User, Error> {
    req.extensions()
        .get::<User>()
        .cloned()
        .ok_or_else(|| ErrorUnauthorized("Unauthorized request"))
}

async fn get_store_read_lock(
    store: &web::Data<Arc<RwLock<Box<dyn storage::Store>>>>,
) -> RwLockReadGuard<'_, Box<dyn storage::Store>> {
    store.read().await
}

async fn get_store_write_lock(
    store: &web::Data<Arc<RwLock<Box<dyn storage::Store>>>>,
) -> RwLockWriteGuard<'_, Box<dyn storage::Store>> {
    store.write().await
}

/// Maps a `storage::Error` from the email-management methods onto an HTTP
/// error. Anything not in the email-mgmt set falls through to a 500 with
/// the underlying error's display string.
fn map_store_error(err: StoreError) -> Error {
    match err {
        StoreError::UserEmailExists() => {
            ErrorConflict("Email is already taken")
        }
        StoreError::UserEmailIsLast() => ErrorConflict(
            "Cannot remove the user's only email address",
        ),
        StoreError::UserEmailIsPrimary() => ErrorConflict(
            "Cannot remove the primary email; promote another email first",
        ),
        StoreError::UserEmailNotVerified() => ErrorConflict(
            "An unverified email cannot be promoted to primary",
        ),
        StoreError::UserEmailNotFound(email) => {
            ErrorNotFound(format!("Email '{email}' is not registered for this user"))
        }
        StoreError::UserEmailMissing() | StoreError::UserEmailInvalid() => {
            ErrorBadRequest(format!("Invalid email address: {err}"))
        }
        StoreError::UserIdNotFound(id) => {
            ErrorNotFound(format!("User with id '{id}' not found"))
        }
        other => {
            log::warn!("user_emails store error: {other}");
            ErrorInternalServerError("Failed to process email-management request")
        }
    }
}

/// Reloads the caller's user record from the store. Used after every write
/// so the response reflects the persisted shape (rather than the in-memory
/// snapshot that triggered the request).
async fn reload_user(
    store: &dyn storage::Store,
    user_id: uuid::Uuid,
) -> Result<User, Error> {
    store.get_user(user_id).await.map_err(|err| {
        log::error!("user_emails reload failed for {user_id}: {err}");
        ErrorInternalServerError("Failed to refresh user after email change")
    })
}

// ---------------------------------------------------------------------------
// Request / response shapes
// ---------------------------------------------------------------------------

/// Response body for `GET /api/v1/users/me/emails` and the body of every
/// successful write — callers always get back the full, current set.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct UserEmailsResponse {
    pub emails: Vec<UserEmail>,
}

impl UserEmailsResponse {
    fn from_user(user: &User) -> Self {
        Self { emails: user.emails.clone() }
    }
}

/// Request body for `POST /api/v1/users/me/emails`. Only the address is
/// caller-controlled; verification flag is determined server-side (see
/// the handler doc-comment for the policy).
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddUserEmailRequest {
    pub email: String,
}

// ---------------------------------------------------------------------------
// GET /api/v1/users/me/emails
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "List the authenticated user's email addresses",
    description = "Returns every email row attached to the caller, including primary status, verification status, and verification timestamp.",
    get,
    path = "/api/v1/users/me/emails",
    responses(
        (status = 200, description = "User's emails", body = UserEmailsResponse),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/users/me/emails")]
pub async fn list_emails(
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let caller = get_authorized_user(&req)?;
    // Reload from the store rather than trusting the request-extension
    // snapshot — keeps the response symmetric with the four write endpoints
    // (which all reload after their mutation) and means a stale snapshot
    // can't surface stale primary/verified state.
    let store_lock = get_store_read_lock(&store).await;
    let updated = reload_user(&**store_lock, caller.id).await?;
    Ok(HttpResponse::Ok().json(UserEmailsResponse::from_user(&updated)))
}

// ---------------------------------------------------------------------------
// POST /api/v1/users/me/emails
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Add a new email address to the authenticated user",
    description = "Adds a new (non-primary) email row. The new row is created with `verified = true` for now; once email verification ships, this is where the verification mail is sent and the row is created `verified = false` until the link is clicked.",
    post,
    path = "/api/v1/users/me/emails",
    request_body = AddUserEmailRequest,
    responses(
        (status = 201, description = "Email added; full email list returned", body = UserEmailsResponse),
        (status = 400, description = "Invalid email address"),
        (status = 401, description = "Unauthorized request"),
        (status = 409, description = "Email is already taken")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/users/me/emails")]
pub async fn add_email(
    add_req: web::Json<AddUserEmailRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let caller = get_authorized_user(&req)?;
    let mut store_lock = get_store_write_lock(&store).await;
    // `verified = true` while email-send-support is unshipped — see the
    // handler doc-comment. Once the verification flow lands, this becomes
    // `false` and the verify endpoint becomes real.
    store_lock
        .add_user_email(caller.id, &add_req.email, true)
        .await
        .map_err(map_store_error)?;
    let updated = reload_user(&**store_lock, caller.id).await?;
    Ok(HttpResponse::Created().json(UserEmailsResponse::from_user(&updated)))
}

// ---------------------------------------------------------------------------
// DELETE /api/v1/users/me/emails/{email}
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Remove an email address from the authenticated user",
    description = "Rejects with 409 if the address is the user's only email or their primary; in the latter case the caller must promote another email first.",
    delete,
    path = "/api/v1/users/me/emails/{email}",
    params(
        ("email" = String, Path, description = "Email address to remove (URL-encoded)")
    ),
    responses(
        (status = 200, description = "Email removed; remaining emails returned", body = UserEmailsResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Email is not registered for this user"),
        (status = 409, description = "Email cannot be removed (last email or primary)")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[delete("/users/me/emails/{email}")]
pub async fn delete_email(
    path: web::Path<String>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let caller = get_authorized_user(&req)?;
    let email = path.into_inner();
    let mut store_lock = get_store_write_lock(&store).await;
    store_lock
        .remove_user_email(caller.id, &email)
        .await
        .map_err(map_store_error)?;
    let updated = reload_user(&**store_lock, caller.id).await?;
    Ok(HttpResponse::Ok().json(UserEmailsResponse::from_user(&updated)))
}

// ---------------------------------------------------------------------------
// PUT /api/v1/users/me/emails/{email}/primary
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Promote an email to the authenticated user's primary",
    description = "Atomically clears `is_primary` on every other row and sets it on the target. Rejects with 409 if the target is unverified — promoting an unverified row would let a session-hijacker redirect password resets to an attacker-controlled mailbox.",
    put,
    path = "/api/v1/users/me/emails/{email}/primary",
    params(
        ("email" = String, Path, description = "Email address to promote (URL-encoded)")
    ),
    responses(
        (status = 200, description = "Email promoted; full email list returned", body = UserEmailsResponse),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Email is not registered for this user"),
        (status = 409, description = "Email cannot be promoted (unverified)")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[put("/users/me/emails/{email}/primary")]
pub async fn set_primary_email(
    path: web::Path<String>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let caller = get_authorized_user(&req)?;
    let email = path.into_inner();
    let mut store_lock = get_store_write_lock(&store).await;
    store_lock
        .set_primary_email(caller.id, &email)
        .await
        .map_err(map_store_error)?;
    let updated = reload_user(&**store_lock, caller.id).await?;
    Ok(HttpResponse::Ok().json(UserEmailsResponse::from_user(&updated)))
}

// ---------------------------------------------------------------------------
// POST /api/v1/users/me/emails/{email}/verify (stub)
// ---------------------------------------------------------------------------

#[utoipa::path(
    summary = "Verify an email address (stub)",
    description = "Placeholder for the email-verification flow. Returns 501 Not Implemented until the email-send infrastructure ships.",
    post,
    path = "/api/v1/users/me/emails/{email}/verify",
    params(
        ("email" = String, Path, description = "Email address to verify (URL-encoded)")
    ),
    responses(
        (status = 501, description = "Verification flow is not yet implemented"),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/users/me/emails/{email}/verify")]
pub async fn verify_email_stub(
    _path: web::Path<String>,
    _store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let _caller = get_authorized_user(&req)?;
    // Reads the auth context so the 501 only fires for authenticated
    // callers; an unauth request still gets a 401, mirroring the other
    // handlers in this module.
    Err(ErrorNotImplemented("Email verification flow is not yet implemented"))
}
