use crate::config::app::AppConfig;
use crate::middleware::auth::{ApiKey, LoginId};
use crate::oauth::{
    account, state as oauth_state, FlowOrigin, OAuthConnector, OAuthError, OAuthProviderPublic,
};
use crate::service::auth::AuthService;
use crate::SharedFeatures;


use data_model::{Maze, User};
use maze::{Error as MazeError, Generator, GeneratorOptions, MazeSolution, MazeSolver};
use storage::{Error as StoreError, MazeItem, Store, SharedStore};

use actix_web::{cookie::{Cookie, SameSite, time::Duration as CookieDuration}, delete, get, post, put, web, web::Query, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{ErrorBadRequest, ErrorConflict, ErrorForbidden, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized, ErrorUnprocessableEntity, InternalError}
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use urlencoding::encode;
use utoipa::ToSchema;
use uuid::Uuid;
// **************************************************************************************************
// Private utility functions
// **************************************************************************************************

fn get_caller_ip_address(req: &HttpRequest) -> Option<String> {
    req
    .headers()
    .get("X-Forwarded-For")
    .and_then(|hdr| hdr.to_str().ok())
    .map(|s| s.split(',').next().unwrap_or("").trim().to_string())
    .or_else(|| req.peer_addr().map(|addr| addr.ip().to_string()))
}

fn get_caller_device_info(req: &HttpRequest) -> Option<String> {
    req
    .headers()
    .get("User-Agent")
    .and_then(|ua| ua.to_str().ok())
    .map(|s| s.to_string())   
}

fn get_authorized_user(req: &HttpRequest, admin_required: bool) -> Result<User, Error> {
    if let Some(user) = req.extensions().get::<User>() {
        if admin_required && !user.is_admin {
            return Err(ErrorUnauthorized( "Unauthorized request"));
        }
        Ok(user.clone())
    } else {
        Err(ErrorUnauthorized( "Unauthorized request"))
    }
}

fn get_logout_details(req: &HttpRequest) -> Result<(User, uuid::Uuid), Error> {
    let has_api_key = req.extensions().get::<ApiKey>().is_some();
    let login_id = req.extensions()
        .get::<LoginId>()
        .copied()
        .ok_or_else(|| {
            if has_api_key {
                log::info!("Returning logout complete");
                InternalError::from_response("Logout complete", HttpResponse::NoContent().finish()).into()
            } else {
                log::warn!("Returning unauthorized: missing login id token");
                ErrorUnauthorized("Missing login id token")
            }
        })?
        .0;


    let user = get_authorized_user(req, false)?;

    Ok((user, login_id))
}

async fn verify_user_credentials(store: &web::Data<SharedStore>, auth_service: &AuthService,
    email: &str, password: &str) -> Result<User, Error> {

    if email.trim().is_empty() || password.trim().is_empty() {
        return Err(ErrorUnprocessableEntity("Email and password must be provided"));
    }

    let user = {
        let store_lock = get_store_read_lock(store).await;
        store_lock.find_user_by_verified_email(email).await.map_err(|err| {
            match err {
                StoreError::UserNotFound() => ErrorUnauthorized("Invalid email or password"),
                _ => {
                    log::warn!("login failed for {email:?}: {err}");
                    ErrorInternalServerError("Failed to process login request")
                }
            }
        })?
    };

    let password_matches = auth_service.verify_password(&user.password_hash, password).map_err(|err| {
        log::error!("Password verification failed: {err:?}");
        ErrorInternalServerError("Internal authentication error")
    })?;

    if !password_matches {
        return Err(ErrorUnauthorized("Invalid email or password"));
    }

    Ok(user)
}

async fn get_store_read_lock(
    store: &web::Data<Arc<RwLock<Box<dyn Store>>>>,
) -> RwLockReadGuard<'_, Box<dyn Store>> {
    store.read().await
}

async fn get_store_write_lock(
    store: &web::Data<Arc<RwLock<Box<dyn Store>>>>,
) -> RwLockWriteGuard<'_, Box<dyn Store>> {
    store.write().await
}

// User ID functions 
fn user_id_from_str(value: &str) -> Result<Uuid, Error> {
    match Uuid::parse_str(value) {
        Ok(id) => Ok(id),
        Err(_) => Err(get_user_not_found_error(value.to_string())),
    }
}

// Password-related errors
fn get_hash_password_internal_error(err: &argon2::password_hash::Error) -> Error {
    ErrorInternalServerError(format!("Error hashing password: {err}"))
}

// User-related errors
fn get_users_fetch_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error fetching users: {err}"))
}
fn get_user_create_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error creating user: {err}"))
}

fn get_user_update_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error updating user: {err}"))
}

fn get_user_not_found_error(id: String) -> Error {
    ErrorNotFound(format!("User with id '{id}' not found"))
}

fn get_user_exists_error() -> Error {
    ErrorConflict("User with the given username or email already exists".to_string())
}

fn get_invalid_request_error(reason: &str) -> Error {
    ErrorBadRequest(format!("Invalid request ({reason})"))
}

fn get_missing_username_request_error() -> Error {
    get_invalid_request_error("missing username")
}

fn get_missing_password_request_error() -> Error {
    get_invalid_request_error("missing password")
}

fn validate_password_complexity(password: &str) -> Result<(), Error> {
    if password.len() < 8 {
        return Err(get_invalid_request_error("password must be at least 8 characters"));
    }
    if !password.chars().any(|c| c.is_uppercase()) {
        return Err(get_invalid_request_error("password must contain at least one uppercase letter"));
    }
    if !password.chars().any(|c| c.is_lowercase()) {
        return Err(get_invalid_request_error("password must contain at least one lowercase letter"));
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Err(get_invalid_request_error("password must contain at least one digit"));
    }
    if !password.chars().any(|c| !c.is_alphanumeric()) {
        return Err(get_invalid_request_error("password must contain at least one special character"));
    }
    Ok(())
}

fn get_invalid_email_request_error() -> Error {
    get_invalid_request_error("invalid email")
}

fn get_missing_email_request_error() -> Error {
    get_invalid_request_error("missing email")
}

fn get_user_fetch_internal_error(id: Uuid, err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error fetching user item with id '{id}': {err}"))
}

fn get_cannot_delete_last_admin_error() -> Error {
    ErrorConflict("Cannot delete the last admin account".to_string())
}

async fn is_last_admin(store_lock: &RwLockWriteGuard<'_, Box<dyn Store>>, user_id: Uuid) -> Result<bool, Error> {
    let admins = store_lock.get_admin_users().await.map_err(|err| get_users_fetch_internal_error(&err))?;
    Ok(admins.len() == 1 && admins[0].id == user_id)
}

// Maze-related errors
fn get_mazes_fetch_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error fetching maze items: {err}"))
}

fn get_maze_create_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error creating maze: {err}"))
}

fn get_maze_not_found_error(id: &str) -> Error {
    ErrorNotFound(format!("Maze with id '{id}' not found"))
}

fn get_maze_exists_error(id: &str) -> Error {
    ErrorConflict(format!("Maze with id '{id}' already exists"))
}

fn get_maze_fetch_internal_error(id: &str, err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error fetching maze item with id '{id}': {err}"))
}

fn get_maze_id_mismatch_error(url_id: &str, maze_id: &str) -> Error {
    ErrorBadRequest(format!("URL ID '{url_id}' and body maze ID '{maze_id}' do not match"))
}

pub (crate) fn get_maze_solve_error_string(err: &MazeError) -> String {
    format!("The maze could not be solved: {err}")
}

fn get_maze_solve_error(err: &MazeError) -> Error {
    ErrorUnprocessableEntity(get_maze_solve_error_string(err))
}

pub (crate) fn get_maze_generate_error_string(err: &MazeError) -> String {
    format!("The maze could not be generated: {err}")
}

fn get_maze_generate_error(err: &MazeError) -> Error {
    ErrorUnprocessableEntity(get_maze_generate_error_string(err))
}

async fn update_store_user<F>(
    mut store_lock: RwLockWriteGuard<'_, Box<dyn Store>>,
    user: &mut User,
    handle_internal_error: F,
) -> Result<HttpResponse, Error>
where
    F: Fn(&StoreError) -> Error,
{
    match store_lock.update_user(user).await {
        Ok(_) =>  Ok(HttpResponse::Ok().json(UserItem::from_store_user(user))),
        Err(err) => {
            match err {
                StoreError::UserEmailExists() | StoreError::UserNameExists()  => Err(get_user_exists_error()),
                StoreError::UserNameMissing() => Err(get_missing_username_request_error()),
                StoreError::UserEmailInvalid() => Err(get_invalid_email_request_error()),
                StoreError::UserEmailMissing() => Err(get_missing_email_request_error()),
                _ => Err(handle_internal_error(&err))
            }
        }
    }
}

/// Contains the summary details for a user
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct UserItem {
    #[schema(value_type = String)] // Treat as string during serlialization
    /// User ID
    pub id: Uuid,
    /// Is administrator?
    pub is_admin: bool,
    /// Username
    pub username: String,
    /// Full name
    pub full_name: String,
    /// Primary email address. Equals the `email` of whichever row in
    /// `emails` is the primary, or an empty string if the user somehow
    /// has no primary (a should-not-happen state surfaced as empty rather
    /// than a 500).
    pub email: String,
    /// All email addresses attached to this user, including primary status,
    /// verification status, and verification timestamp.
    #[serde(default)]
    pub emails: Vec<data_model::UserEmail>,
    /// Whether the user has a password set. `false` for OAuth-only users
    /// who haven't yet added a password as a second login method —
    /// front-ends use this to choose between the "Change" and "Set"
    /// variants of the password popup. The hash itself is never exposed.
    #[serde(default)]
    pub has_password: bool,
}

impl UserItem {
    pub fn from_store_user(user: &User) -> UserItem {
        UserItem {
            id:  user.id,
            is_admin: user.is_admin,
            username: user.username.clone(),
            full_name: user.full_name.clone(),
            email: user.email().to_string(),
            emails: user.emails.clone(),
            has_password: !user.password_hash.is_empty(),
        }
    }
}
// **************************************************************************************************
// Endpoint: GET /api/v1/features
// Handler:  get_features()
// **************************************************************************************************
/// Response body for `GET /api/v1/features`. Also accepted as the request body
/// for `PUT /api/v1/admin/features`; only `allow_signup` is mutable from there
/// and `oauth_providers` is sourced from the live connector at response time.
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone, Default)]
pub struct AppFeaturesResponse {
    /// Whether new users can self-register via the signup endpoint
    pub allow_signup: bool,
    /// OAuth providers currently enabled on this server. Empty when
    /// `[oauth].enabled = false` or no providers are configured. Read-only on
    /// the admin update endpoint.
    #[serde(default)]
    pub oauth_providers: Vec<OAuthProviderPublic>,
}

fn build_features_response(
    allow_signup: bool,
    connector: &dyn OAuthConnector,
) -> AppFeaturesResponse {
    AppFeaturesResponse {
        allow_signup,
        oauth_providers: connector.enabled_providers(),
    }
}

#[utoipa::path(
    summary = "Returns the server's active feature flags",
    description = "Returns the feature flags that control which capabilities are available to users. No authentication required.",
    get,
    path = "/api/v1/features",
    responses(
        (status = 200, description = "Feature flags retrieved successfully", body = AppFeaturesResponse),
        (status = 500, description = "Internal server error")
    ),
    tags = ["v1"]
)]
#[get("/features")]
pub async fn get_features(
    features: web::Data<SharedFeatures>,
    connector: web::Data<crate::oauth::SharedOAuthConnector>,
) -> Result<HttpResponse, Error> {
    let features_lock = features.read().map_err(|_| {
        ErrorInternalServerError("Failed to acquire features read lock")
    })?;
    Ok(HttpResponse::Ok().json(build_features_response(
        features_lock.allow_signup,
        connector.as_ref().as_ref(),
    )))
}

// **************************************************************************************************
// Endpoint: PUT /api/v1/admin/features
// Handler:  update_admin_features()
// **************************************************************************************************

fn update_features_in_config(config_path: &str, new_features: &AppFeaturesResponse) -> Result<(), Error> {
    let content = std::fs::read_to_string(config_path).unwrap_or_default();
    let mut doc = content.parse::<toml_edit::DocumentMut>().map_err(|e| {
        ErrorInternalServerError(format!("Failed to parse config file: {e}"))
    })?;
    if doc.get("features").is_none() {
        doc["features"] = toml_edit::table();
    }
    doc["features"]["allow_signup"] = toml_edit::value(new_features.allow_signup);
    std::fs::write(config_path, doc.to_string()).map_err(|e| {
        ErrorInternalServerError(format!("Failed to write config file: {e}"))
    })?;
    Ok(())
}

#[utoipa::path(
    summary = "Update server application feature flags",
    description = "Updates the server's active feature flags. Changes take effect immediately and are persisted to config.toml.",
    put,
    path = "/api/v1/admin/features",
    request_body = AppFeaturesResponse,
    responses(
        (status = 200, description = "Features updated successfully", body = AppFeaturesResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[put("/admin/features")]
pub async fn update_admin_features(
    req: HttpRequest,
    body: web::Json<AppFeaturesResponse>,
    features: web::Data<SharedFeatures>,
    config: web::Data<AppConfig>,
    connector: web::Data<crate::oauth::SharedOAuthConnector>,
) -> Result<HttpResponse, Error> {
    get_authorized_user(&req, true)?;

    let new_features = body.into_inner();
    update_features_in_config(&config.config_path, &new_features)?;

    let mut features_lock = features.write().map_err(|_| {
        ErrorInternalServerError("Failed to acquire features write lock")
    })?;
    features_lock.allow_signup = new_features.allow_signup;

    Ok(HttpResponse::Ok().json(build_features_response(
        features_lock.allow_signup,
        connector.as_ref().as_ref(),
    )))
}

// **************************************************************************************************
// Endpoint: POST /api/v1/signup
// Handler:  signup()
// **************************************************************************************************
/// Signup request
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct SignupRequest {
    /// Email address
    pub email: String,
    /// Password
    pub password: String,
}

impl SignupRequest {
    pub fn into_user(&self, auth_service: &AuthService) -> Result<User, Error> {
        validate_password_complexity(&self.password)?;
        let password_hash = if self.password.is_empty() {
            "".to_string()
        } else {
            auth_service
                .hash_password(&self.password)
                .map_err(|err| get_hash_password_internal_error(&err))?
        };
        Ok(
            User {
                id: Uuid::nil(),
                is_admin: false,
                username: "".to_string(),
                full_name: "".to_string(),
                emails: vec![data_model::UserEmail::new_primary_verified(&self.email)],
                password_hash,
                api_key: Uuid::nil(),
                logins: vec![],
                oauth_identities: vec![],
            }
        )
    }
}

/// Derives a candidate username from an email address local part.
/// Keeps alphanumeric and underscore characters (lowercased), replaces everything else with `_`,
/// then trims leading/trailing underscores. Falls back to `"user"` if the result is empty.
fn generate_username_from_email(email: &str) -> String {
    let local = email.split('@').next().unwrap_or("user");
    let sanitized: String = local
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c.to_ascii_lowercase() } else { '_' })
        .collect();
    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() { "user".to_string() } else { trimmed.to_string() }
}

#[utoipa::path(
    summary = "Sign up as a new user",
    description = "This endpoint registers a new (non-admin) user account. A username is auto-generated from the email address and can be personalised later via the profile endpoint.",
    post,
    path = "/api/v1/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserItem),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Signup is disabled on this server"),
        (status = 409, description = "User with the given email already exists")
    ),
    tags = ["v1"]
)]
#[post("/signup")]
pub async fn signup(
    signup_req: web::Json<SignupRequest>,
    auth_service: web::Data<AuthService>,
    store: web::Data<SharedStore>,
    features: web::Data<SharedFeatures>,
) -> Result<HttpResponse, Error> {
    let allow_signup = {
        let features_lock = features.read().map_err(|_| {
            ErrorInternalServerError("Failed to acquire features read lock")
        })?;
        features_lock.allow_signup
    };
    if !allow_signup {
        return Err(ErrorForbidden("Signup is disabled on this server"));
    }

    let mut store_lock = get_store_write_lock(&store).await;
    let signup_req_data: SignupRequest = signup_req.into_inner();
    let mut store_user = signup_req_data.into_user(&auth_service)?;
    let base_username = generate_username_from_email(&signup_req_data.email);

    for attempt in 0u8..=5 {
        store_user.username = if attempt == 0 {
            base_username.clone()
        } else {
            format!("{}_{}", base_username, &Uuid::new_v4().to_string().replace('-', "")[..6])
        };
        match store_lock.create_user(&mut store_user).await {
            Ok(()) => return Ok(
                HttpResponse::Created()
                .insert_header(("Location", "/api/v1/users/me"))
                .json(UserItem::from_store_user(&store_user))
            ),
            Err(StoreError::UserNameExists()) if attempt < 5 => continue,
            Err(err) => return match err {
                StoreError::UserEmailExists() | StoreError::UserNameExists() => Err(get_user_exists_error()),
                StoreError::UserPasswordMissing() => Err(get_missing_password_request_error()),
                _ => Err(get_user_create_internal_error(&err))
            }
        }
    }
    unreachable!()
}
// **************************************************************************************************
// Endpoints: GET /api/v1/auth/oauth/{provider}/start
//            GET /api/v1/auth/oauth/{provider}/callback
// Handlers:  oauth_start, oauth_callback
// **************************************************************************************************

#[derive(Deserialize, Debug)]
pub struct OAuthStartQuery {
    /// Where the flow originated. "web" → SPA fragment redirect; "mobile" →
    /// custom URL-scheme redirect for the MAUI WebAuthenticator.
    pub origin: FlowOrigin,
    /// Optional opaque client-supplied state. Echoed back unchanged on the
    /// final mobile-redirect URL. Used by Windows WinUIEx WebAuthenticator
    /// (and similar URL-scheme brokers) to correlate the in-flight
    /// AuthenticateAsync task with the eventual app activation. The server
    /// does not interpret this value.
    pub state: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OAuthCallbackQuery {
    pub code: Option<String>,
    pub state: Option<String>,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

const STATE_COOKIE_PATH: &str = "/api/v1/auth/oauth";

fn build_state_cookie<'a>(value: String) -> Cookie<'a> {
    Cookie::build(oauth_state::COOKIE_NAME, value)
        .path(STATE_COOKIE_PATH)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::seconds(oauth_state::TTL_SECONDS))
        .finish()
}

fn clear_state_cookie<'a>() -> Cookie<'a> {
    Cookie::build(oauth_state::COOKIE_NAME, "")
        .path(STATE_COOKIE_PATH)
        .http_only(true)
        .secure(true)
        .same_site(SameSite::Lax)
        .max_age(CookieDuration::seconds(0))
        .finish()
}

fn oauth_error_to_actix(err: OAuthError) -> Error {
    match err {
        OAuthError::UnknownOrDisabledProvider(_) => ErrorNotFound(err.to_string()),
        OAuthError::InvalidState(_) => ErrorBadRequest(err.to_string()),
        OAuthError::ProviderResponse(_) | OAuthError::ProviderTransport(_) => {
            log::warn!("OAuth provider error: {err}");
            ErrorInternalServerError("OAuth provider error")
        }
        OAuthError::Misconfigured(_) => {
            log::error!("OAuth misconfiguration: {err}");
            ErrorInternalServerError("OAuth misconfiguration")
        }
    }
}

fn web_callback_url(token_id: Uuid, expires_at: DateTime<Utc>, is_new_user: bool) -> String {
    let mut url = format!(
        "/oauth/callback#token={}&expires_at={}",
        token_id,
        encode(&expires_at.to_rfc3339()),
    );
    if is_new_user {
        url.push_str("&new_user=true");
    }
    url
}

fn mobile_callback_url(
    scheme: &str,
    token_id: Uuid,
    expires_at: DateTime<Utc>,
    client_state: Option<&str>,
    is_new_user: bool,
) -> String {
    // Params live in the URL FRAGMENT (`#token=...&...`) rather than the query
    // string. The mobile final response is a 200 OK HTML bridge page (see
    // `mobile_callback_html`) that triggers `maze-app://...` activation via
    // meta-refresh, NOT a direct 302 to the custom scheme. Two forces still
    // make fragment the right choice:
    //
    //   1. Facebook appends a literal `#_=_` fragment to every OAuth redirect.
    //      That fragment rides the Facebook→server-callback 302 and ends up
    //      on the bridge page's URL. Meta-refresh-initiated navigations
    //      preserve the source URL's fragment when the target URL has none,
    //      so a query-only `maze-app://...?token=...` target would arrive at
    //      WinUIEx as `maze-app://...?token=...#_=_`. Putting our params in
    //      the fragment gives the target an explicit fragment, which
    //      overrides any fragment otherwise inherited from the source page.
    //   2. WinUIEx parses fragment-when-present and query-when-absent.
    //      Because of (1) the URL reaching WinUIEx will always have a
    //      fragment for the Facebook flow, so emitting our params as the
    //      fragment is also what WinUIEx needs to actually see them.
    let mut url = format!(
        "{scheme}://oauth-callback#token={}&expires_at={}",
        token_id,
        encode(&expires_at.to_rfc3339()),
    );
    if let Some(s) = client_state {
        // The OAuth standard query-parameter name for this is `state`; the
        // server treats it as opaque, so URL-encode it on the way out.
        url.push_str(&format!("&state={}", encode(s)));
    }
    if is_new_user {
        url.push_str("&new_user=true");
    }
    url
}

fn web_error_url(reason: &str) -> String {
    format!("/login?error={}", encode(reason))
}

/// Mobile error URL — uses the SAME host (`oauth-callback`) as the success
/// redirect, distinguished by a `reason` fragment parameter. The MAUI
/// `WebAuthenticator` (and `WinUIEx` on Windows) filters incoming
/// URL-scheme activations by callback URL host, so the error path must share
/// the success path's host or the in-flight `AuthenticateAsync` will never
/// resolve. Echoes `client_state` for the same reason `mobile_callback_url`
/// does — WinUIEx correlates the activation via the original signinId.
fn mobile_error_url(scheme: &str, reason: &str, client_state: Option<&str>) -> String {
    // Same fragment-not-query rationale as `mobile_callback_url`.
    let mut url = format!("{scheme}://oauth-callback#reason={}", encode(reason));
    if let Some(s) = client_state {
        url.push_str(&format!("&state={}", encode(s)));
    }
    url
}

/// Wrap a `maze-app://oauth-callback#…` redirect target in a tiny HTML page
/// that fully loads in the system browser, then triggers the protocol handler
/// client-side via `<meta http-equiv="refresh">`. Returning HTML instead of a
/// `302` to a custom scheme stops the system browser tab from spinning forever
/// after the OS hands activation off to the MAUI app.
///
/// Why meta-refresh and not `<script>window.location.replace(...)</script>`:
/// modern browsers (Edge, Chrome) BLOCK JS-initiated navigations to custom
/// schemes unless a fresh user gesture is in scope. After Facebook's "Continue
/// as XXX" click that gesture is still in scope, so JS works; after Google /
/// GitHub auto-redirects from a consent screen the user already approved no
/// gesture is in scope, so JS is silently blocked and the user has to click
/// the manual "Open Maze app" fallback. Meta-refresh is a server-directed
/// navigation and is honored by browsers without a gesture, so it fires
/// uniformly across all three providers. The visible link below remains as a
/// manual fallback if the OS protocol handler fails to fire for any reason.
fn mobile_callback_html(target_url: &str) -> String {
    // Inside an HTML attribute value `&` must be escaped to `&amp;`;
    // percent-encoding has already removed every other HTML-special char
    // from the URL, so this single substitution suffices for both the
    // meta-refresh `content` attribute and the `<a href>` fallback.
    let escaped_url = target_url.replace('&', "&amp;");
    format!(
        "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <meta http-equiv=\"refresh\" content=\"0;url={escaped_url}\">\n\
         <title>Sign-in complete</title>\n\
         <style>\n\
         body {{ font-family: system-ui, -apple-system, Segoe UI, sans-serif; text-align: center; padding: 3rem 1rem; color: #333; }}\n\
         h1 {{ font-size: 1.4rem; margin-bottom: 0.5rem; }}\n\
         p {{ font-size: 1rem; margin: 0.5rem 0; }}\n\
         a.btn {{ display: inline-block; margin-top: 1rem; padding: 0.6rem 1.2rem; background: #1f6feb; color: white; text-decoration: none; border-radius: 6px; }}\n\
         </style>\n\
         </head>\n\
         <body>\n\
         <h1>Sign-in complete</h1>\n\
         <p>Returning you to the Maze app — you can close this tab.</p>\n\
         <p><a class=\"btn\" href=\"{escaped_url}\">Open Maze app</a></p>\n\
         </body>\n\
         </html>\n"
    )
}

/// Build the final HTTP response for an OAuth callback. The response shape
/// depends on origin:
///   - `Web`:    `302 Location: <web_url>` (SPA reads `#token=…` from URL).
///   - `Mobile`: `200 OK` HTML page that triggers the `maze-app://` protocol
///     handler client-side. Returning HTML instead of `302` to a custom
///     scheme stops the system browser tab from spinning indefinitely after
///     the OS hands activation off to the MAUI app.
///
/// In both cases the in-flight state cookie is cleared.
fn oauth_callback_response(origin: FlowOrigin, web_url: &str, mobile_url: &str) -> HttpResponse {
    match origin {
        FlowOrigin::Web => HttpResponse::Found()
            .insert_header(("Location", web_url))
            .cookie(clear_state_cookie())
            .finish(),
        FlowOrigin::Mobile => HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .cookie(clear_state_cookie())
            .body(mobile_callback_html(mobile_url)),
    }
}

#[utoipa::path(
    summary = "Begin an OAuth sign-in flow",
    description = "Generates a provider authorize URL with state + PKCE, sets a short-lived CSRF cookie, and 302-redirects to the provider. Identical response shape for both origin=web and origin=mobile; the origin only controls where the *callback* redirects at the end of the flow. Returning a redirect (rather than JSON) on mobile is what lets the platform browser carry the state cookie through the round trip.",
    get,
    path = "/api/v1/auth/oauth/{provider}/start",
    params(
        ("provider" = String, Path, description = "Canonical provider name, e.g. 'google' or 'github'"),
        ("origin" = String, Query, description = "'web' or 'mobile'")
    ),
    responses(
        (status = 302, description = "Redirect to provider authorize URL"),
        (status = 400, description = "Invalid origin or provider"),
        (status = 404, description = "Unknown or disabled provider"),
        (status = 500, description = "Internal error")
    ),
    tags = ["v1"]
)]
#[get("/auth/oauth/{provider}/start")]
pub async fn oauth_start(
    path: web::Path<String>,
    query: web::Query<OAuthStartQuery>,
    connector: web::Data<crate::oauth::SharedOAuthConnector>,
) -> Result<HttpResponse, Error> {
    let provider = path.into_inner();
    let mut begin = connector
        .as_ref()
        .as_ref()
        .begin(&provider, query.origin)
        .await
        .map_err(oauth_error_to_actix)?;
    // Capture the client-supplied state (if any) into the cookie so we can
    // echo it back unchanged on the final mobile-redirect URL. WinUIEx
    // WebAuthenticator on Windows needs this for activation correlation.
    begin.persisted.client_state = query.state.clone();
    let cookie_value = oauth_state::encode(&begin.persisted)
        .map_err(|e| ErrorInternalServerError(format!("oauth state encode: {e}")))?;
    let cookie = build_state_cookie(cookie_value);

    Ok(HttpResponse::Found()
        .insert_header(("Location", begin.authorize_url))
        .cookie(cookie)
        .finish())
}

#[utoipa::path(
    summary = "Complete an OAuth sign-in flow",
    description = "The provider redirects here after consent. The handler validates the CSRF cookie + state, exchanges the code via the connector, resolves or creates the user, mints a bearer token, then hands the token back to the client. Web origin: 302 to the SPA at `/oauth/callback#token=...`. Mobile origin: 200 OK with a small HTML bridge page that triggers a `maze-app://oauth-callback#token=...` activation client-side via `<meta http-equiv=\"refresh\">` (returning HTML rather than 302-ing directly to the custom scheme stops the system browser tab from spinning forever after the OS hands activation off to the MAUI app).",
    get,
    path = "/api/v1/auth/oauth/{provider}/callback",
    params(
        ("provider" = String, Path, description = "Canonical provider name"),
        ("code" = Option<String>, Query, description = "Authorization code from the provider"),
        ("state" = Option<String>, Query, description = "CSRF state nonce echoed by the provider"),
        ("error" = Option<String>, Query, description = "Provider-side error code (if the user denied consent, etc.)")
    ),
    responses(
        (status = 200, description = "Mobile origin: HTML bridge page that triggers the `maze-app://` protocol activation client-side"),
        (status = 302, description = "Web origin: redirects to the SPA at `/oauth/callback#token=...`"),
        (status = 400, description = "Invalid or expired state cookie"),
        (status = 404, description = "Unknown or disabled provider"),
        (status = 500, description = "Internal error")
    ),
    tags = ["v1"]
)]
#[get("/auth/oauth/{provider}/callback")]
pub async fn oauth_callback(
    req: HttpRequest,
    path: web::Path<String>,
    query: web::Query<OAuthCallbackQuery>,
    connector: web::Data<crate::oauth::SharedOAuthConnector>,
    config: web::Data<AppConfig>,
    features: web::Data<SharedFeatures>,
    store: web::Data<SharedStore>,
) -> Result<HttpResponse, Error> {
    let provider_path = path.into_inner();
    let scheme = config.oauth.mobile_redirect_scheme.clone();

    // Decode the cookie first so we know whether to redirect-with-error to web
    // or mobile. Without it we have no recoverable origin and must fall back.
    let cookie_str = req
        .cookie(oauth_state::COOKIE_NAME)
        .map(|c| c.value().to_string());
    let persisted = match cookie_str.as_ref().map(|v| oauth_state::decode(v)) {
        Some(Ok(p)) => p,
        _ => {
            // No cookie / corrupt cookie → we have no persisted client_state to
            // echo. WinUIEx may not be able to correlate the activation in this
            // path; the broker's TTL fallback handles it.
            return Ok(redirect_with_clear(
                FlowOrigin::Web,
                &scheme,
                &web_error_url("invalid_state"),
                &mobile_error_url(&scheme, "invalid_state", None),
            ));
        }
    };
    let now_unix = Utc::now().timestamp();
    if oauth_state::is_expired(&persisted, now_unix) {
        return Ok(redirect_with_clear(
            persisted.origin,
            &scheme,
            &web_error_url("state_expired"),
            &mobile_error_url(&scheme, "state_expired", persisted.client_state.as_deref()),
        ));
    }
    if !persisted.provider.eq_ignore_ascii_case(&provider_path) {
        return Ok(redirect_with_clear(
            persisted.origin,
            &scheme,
            &web_error_url("provider_mismatch"),
            &mobile_error_url(&scheme, "provider_mismatch", persisted.client_state.as_deref()),
        ));
    }
    let returned_state = match query.state.as_deref() {
        Some(s) => s,
        None => {
            return Ok(redirect_with_clear(
                persisted.origin,
                &scheme,
                &web_error_url("missing_state"),
                &mobile_error_url(&scheme, "missing_state", persisted.client_state.as_deref()),
            ));
        }
    };
    if returned_state != persisted.state {
        return Ok(redirect_with_clear(
            persisted.origin,
            &scheme,
            &web_error_url("state_mismatch"),
            &mobile_error_url(&scheme, "state_mismatch", persisted.client_state.as_deref()),
        ));
    }
    if let Some(provider_error) = query.error.as_deref() {
        let reason = format!("provider_error:{provider_error}");
        return Ok(redirect_with_clear(
            persisted.origin,
            &scheme,
            &web_error_url(&reason),
            &mobile_error_url(&scheme, &reason, persisted.client_state.as_deref()),
        ));
    }
    let code = match query.code.as_deref() {
        Some(c) => c,
        None => {
            return Ok(redirect_with_clear(
                persisted.origin,
                &scheme,
                &web_error_url("missing_code"),
                &mobile_error_url(&scheme, "missing_code", persisted.client_state.as_deref()),
            ));
        }
    };

    let identity = match connector
        .as_ref()
        .as_ref()
        .complete(&provider_path, code, &persisted)
        .await
    {
        Ok(id) => id,
        Err(err) => {
            log::warn!("oauth complete failed: {err}");
            return Ok(redirect_with_clear(
                persisted.origin,
                &scheme,
                &web_error_url("provider_response"),
                &mobile_error_url(&scheme, "provider_response", persisted.client_state.as_deref()),
            ));
        }
    };

    // Resolve to a user (or create one), inside the store write lock.
    let allow_signup = features
        .read()
        .map_err(|_| ErrorInternalServerError("features lock"))?
        .allow_signup;
    let mut store_lock = get_store_write_lock(&store).await;
    let outcome = {
        // Trait upcast Store → UserStore so the connector-agnostic resolver
        // doesn't need to know about MazeStore / Manage.
        let user_store: &mut dyn storage::UserStore = &mut **store_lock;
        account::resolve(user_store, &identity, allow_signup).await
    };
    let outcome = match outcome {
        Ok(o) => o,
        Err(account::ResolveError::SignupDisabled) => {
            return Ok(redirect_with_clear(
                persisted.origin,
                &scheme,
                &web_error_url("signup_disabled"),
                &mobile_error_url(&scheme, "signup_disabled", persisted.client_state.as_deref()),
            ));
        }
        Err(account::ResolveError::EmailNotVerified) => {
            return Ok(redirect_with_clear(
                persisted.origin,
                &scheme,
                &web_error_url("email_not_verified"),
                &mobile_error_url(&scheme, "email_not_verified", persisted.client_state.as_deref()),
            ));
        }
        Err(account::ResolveError::MissingEmail) => {
            return Ok(redirect_with_clear(
                persisted.origin,
                &scheme,
                &web_error_url("missing_email"),
                &mobile_error_url(&scheme, "missing_email", persisted.client_state.as_deref()),
            ));
        }
        Err(account::ResolveError::Store(e)) => {
            log::error!("oauth resolve store error: {e}");
            return Ok(redirect_with_clear(
                persisted.origin,
                &scheme,
                &web_error_url("store_error"),
                &mobile_error_url(&scheme, "store_error", persisted.client_state.as_deref()),
            ));
        }
    };

    let (mut user, is_new_user) = match outcome {
        account::ResolveOutcome::SignedIn(u) => (u, false),
        account::ResolveOutcome::Created(u) => (u, true),
    };
    let new_login = user.create_login(
        config.security.login_expiry_hours,
        get_caller_ip_address(&req),
        get_caller_device_info(&req),
    );
    store_lock
        .update_user(&mut user)
        .await
        .map_err(|e| get_user_update_internal_error(&e))?;
    drop(store_lock);

    let token_id = new_login.id;
    let expires_at = new_login.expires_at;
    let web_url = web_callback_url(token_id, expires_at, is_new_user);
    let mobile_url = mobile_callback_url(
        &scheme,
        token_id,
        expires_at,
        persisted.client_state.as_deref(),
        is_new_user,
    );
    Ok(oauth_callback_response(persisted.origin, &web_url, &mobile_url))
}

/// Helper: dispatch to `oauth_callback_response` based on origin, clearing
/// the state cookie on the way out. Used for the error redirect paths so
/// callers don't have to remember to clear. The `_scheme` arg is unused but
/// retained for caller-site symmetry with `mobile_*_url` builders.
fn redirect_with_clear(origin: FlowOrigin, _scheme: &str, web_url: &str, mobile_url: &str) -> HttpResponse {
    oauth_callback_response(origin, web_url, mobile_url)
}

// **************************************************************************************************
// Endpoint: GET /api/v1/users/me
// Handler:  get_me()
// **************************************************************************************************
#[utoipa::path(
    summary = "Returns the profile of the currently authenticated user",
    description = "This endpoint returns the profile of the currently authenticated user",
    get,
    path = "/api/v1/users/me",
    responses(
        (status = 200, description = "User profile retrieved successfully", body = UserItem),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/users/me")]
pub async fn get_me(
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let user = get_authorized_user(&req, false)?;
    Ok(HttpResponse::Ok().json(UserItem::from_store_user(&user)))
}
// **************************************************************************************************
// Endpoint: DELETE /api/v1/users/me
// Handler:  delete_me()
// **************************************************************************************************
#[utoipa::path(
    summary = "Deletes the currently authenticated user's account",
    description = "This endpoint deletes the currently authenticated user's account and all their associated mazes",
    delete,
    path = "/api/v1/users/me",
    responses(
        (status = 204, description = "Account deleted successfully"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "User not found"),
        (status = 409, description = "Cannot delete the last admin account")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[delete("/users/me")]
pub async fn delete_me(
    store: web::Data<SharedStore>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store).await;
    let user = get_authorized_user(&req, false)?;

    if is_last_admin(&store_lock, user.id).await? {
        return Err(get_cannot_delete_last_admin_error());
    }

    match store_lock.delete_user(user.id).await {
        Ok(()) => Ok(HttpResponse::NoContent().finish()),
        Err(err) => {
            match err {
                StoreError::UserIdNotFound(id) => Err(get_user_not_found_error(id)),
                _ => Err(get_user_fetch_internal_error(user.id, &err))
            }
        }
    }
}
// **************************************************************************************************
// Endpoint: PUT /api/v1/users/me/password
// Handler:  change_password_me()
// **************************************************************************************************
/// Set-or-change password request. The endpoint handles two flows
/// based on whether the authenticated user already has a password:
///
///   * **Change** (user has a password) — `current_password` is required
///     and verified before the password is rotated.
///   * **Set** (OAuth-only user adding a password as a second login
///     method) — `current_password` must be omitted; there is nothing
///     to verify against.
///
/// `deny_unknown_fields` rejects unknown keys cleanly so an out-of-date
/// client doesn't get a silent partial success.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct ChangePasswordRequest {
    /// Current password — required when the user already has one,
    /// must be omitted when setting an initial password.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_password: Option<String>,
    /// New password
    pub new_password: String,
}

#[utoipa::path(
    summary = "Sets or changes the authenticated user's password",
    description = "Set-or-change endpoint. When the user already has a password (`has_password = true` on `/me`), `current_password` is required and verified; when they don't (OAuth-only user adding a password as a second login method), `current_password` must be omitted.",
    put,
    path = "/api/v1/users/me/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 204, description = "Password set or changed successfully"),
        (status = 400, description = "Invalid request (weak new password, missing/extraneous current_password)"),
        (status = 401, description = "Unauthorized request or incorrect current password")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[put("/users/me/password")]
pub async fn change_password_me(
    change_req: web::Json<ChangePasswordRequest>,
    auth_service: web::Data<AuthService>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut user = get_authorized_user(&req, false)?;
    let change_req_data = change_req.into_inner();
    let user_has_password = !user.password_hash.is_empty();

    match (user_has_password, change_req_data.current_password.as_deref()) {
        // Change path — user has a password, must prove they know it.
        (true, Some(current)) if !current.is_empty() => {
            let password_matches = auth_service
                .verify_password(&user.password_hash, current)
                .map_err(|err| {
                    log::error!("Password verification failed: {err:?}");
                    ErrorInternalServerError("Internal authentication error")
                })?;
            if !password_matches {
                return Err(ErrorUnauthorized("Current password is incorrect"));
            }
        }
        (true, _) => {
            return Err(get_invalid_request_error(
                "current_password is required to change an existing password",
            ));
        }
        // Set path — OAuth-only user adding a password. `current_password`
        // must be omitted; sending an empty string or any value is a
        // client bug worth surfacing rather than silently ignoring.
        (false, None) => { /* allowed — fall through to validation */ }
        (false, Some(_)) => {
            return Err(get_invalid_request_error(
                "current_password must be omitted when no password is set",
            ));
        }
    }

    validate_password_complexity(&change_req_data.new_password)?;

    let new_hash = auth_service
        .hash_password(&change_req_data.new_password)
        .map_err(|err| get_hash_password_internal_error(&err))?;

    let mut store_lock = get_store_write_lock(&store).await;
    user.password_hash = new_hash;

    match store_lock.update_user(&mut user).await {
        Ok(_) => {
            // Defence-in-depth audit log for both branches: a session-
            // hijacker that successfully sets/rotates a password leaves a
            // trace here. When email-send-support ships, this is where
            // the notification mail to the primary email gets fired.
            if user_has_password {
                log::info!(
                    "password changed for user {} (primary email: {})",
                    user.id,
                    user.email()
                );
            } else {
                log::info!(
                    "initial password set for user {} (primary email: {})",
                    user.id,
                    user.email()
                );
            }
            Ok(HttpResponse::NoContent().finish())
        }
        Err(err) => Err(get_user_update_internal_error(&err)),
    }
}
// **************************************************************************************************
// Endpoint: PUT /api/v1/users/me/profile
// Handler:  update_profile_me()
// **************************************************************************************************
/// Update profile request. Email mutation lives on the dedicated
/// `/api/v1/users/me/emails/*` endpoints — this endpoint covers only
/// username and full name. `deny_unknown_fields` rejects any request that
/// still includes an `email` field (or any other unknown field) with a
/// 400, so an out-of-date client can't silently get a "success" response
/// while its email payload is dropped on the floor.
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct UpdateProfileRequest {
    /// Username
    pub username: String,
    /// Full name
    pub full_name: String,
}

impl UpdateProfileRequest {
    pub fn apply_to_store_user(&self, user: &mut User) {
        // Edge-trim silently — `" alice"` and `"alice"` would otherwise be
        // stored as distinct usernames, leaving whitespace-padded display
        // values that surface as identity collisions. Mid-string spaces are
        // preserved (e.g. `"Mary Jane"` round-trips unchanged). Whitespace-
        // only input collapses to `""` and falls through to the existing
        // empty-username rejection in storage validation.
        user.username = self.username.trim().to_string();
        user.full_name = self.full_name.trim().to_string();
    }
}

#[utoipa::path(
    summary = "Updates the authenticated user's profile",
    description = "This endpoint allows the currently authenticated user to update their username and full name. Email management is on the dedicated /api/v1/users/me/emails endpoints.",
    put,
    path = "/api/v1/users/me/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserItem),
        (status = 400, description = "Invalid request (e.g. empty username, or unknown field such as `email`)"),
        (status = 401, description = "Unauthorized request"),
        (status = 409, description = "Username already in use by another user")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[put("/users/me/profile")]
pub async fn update_profile_me(
    update_req: web::Json<UpdateProfileRequest>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut user = get_authorized_user(&req, false)?;
    let store_lock = get_store_write_lock(&store).await;
    update_req.into_inner().apply_to_store_user(&mut user);
    update_store_user(store_lock, &mut user, get_user_update_internal_error).await
}
// **************************************************************************************************
// Endpoint: GET /api/v1/login
// Handler:  login()
// **************************************************************************************************
/// Login request
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct LoginRequest {
    /// Email address
    pub email: String,
    /// Password
    pub password: String,
}
/// Login response
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct LoginResponse {
    #[schema(value_type = String,  example = "550e8400-e29b-41d4-a716-446655440000")]
    /// Login token id
    pub login_token_id: Uuid,

    #[schema(format = "date-time", example = "2025-04-01T12:00:00Z")]
    /// Expiry timestamp of the login token
    pub login_token_expires_at: DateTime<Utc>,
}
#[utoipa::path(
    summary = "Login",
    description = "This endpoint attempts to login a user by email + password",
    post,
    path = "/api/v1/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login sucessful", body=[LoginResponse]),
        (status = 401, description = "Unauthorized request"),
        (status = 422, description = "Login could not be processed"),
    ),
    tags = ["v1"]
)]
#[post("/login")]
pub async fn login(
    login_req: web::Json<LoginRequest>,
    config: web::Data<AppConfig>,
    auth_service: web::Data<AuthService>,
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let mut user = verify_user_credentials(&store, &auth_service, &login_req.email, &login_req.password).await?;
    let login_expiry_hours = config.security.login_expiry_hours;
    let login = user.create_login(login_expiry_hours, get_caller_ip_address(&req), get_caller_device_info(&req));
    let store_lock = get_store_write_lock(&store).await;
    update_store_user(store_lock, &mut user, |err| {
        get_user_update_internal_error(err)
    }).await?;

    Ok(HttpResponse::Ok().json(LoginResponse {
        login_token_id: login.id,
        login_token_expires_at: login.expires_at, 
    }))           
}
// **************************************************************************************************
// Endpoint: GET /api/v1/logout
// Handler:  logout()
// **************************************************************************************************
#[utoipa::path(
    summary = "Logout",
    description = "This endpoint attempts to logout a user based on the bearer login token provided in the header",
    post,
    path = "/api/v1/logout",
    responses(
        (status = 204, description = "Logout sucessful"),
        (status = 401, description = "Unauthorized request"),
    ),
    security(
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/logout")]
pub async fn logout(
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let (mut user, login_id) = get_logout_details(&req)?;
    let store_lock = get_store_write_lock(&store).await;

    user.remove_login(login_id);

    update_store_user(store_lock, &mut user, |err| {
        get_user_update_internal_error(err)
    }).await?;         

    Ok(HttpResponse::NoContent().finish())
}
// **************************************************************************************************
// Endpoint: POST /api/v1/login/renew
// Handler:  renew()
// **************************************************************************************************
/// Renew login token response
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct RenewResponse {
    #[schema(value_type = String, example = "550e8400-e29b-41d4-a716-446655440000")]
    /// Login token id
    pub login_token_id: Uuid,
    #[schema(format = "date-time", example = "2025-04-01T12:00:00Z")]
    /// Updated expiry timestamp of the login token
    pub login_token_expires_at: DateTime<Utc>,
}
/// Extends the lifetime of the current bearer login token without re-authenticating
#[utoipa::path(
    summary = "Renew login token",
    description = "Extends the lifetime of the current bearer login token without re-authenticating. The same token ID is retained. API keys are not accepted.",
    post,
    path = "/api/v1/login/renew",
    responses(
        (status = 200, description = "Token renewed successfully", body = RenewResponse),
        (status = 401, description = "Unauthorized — token missing, expired, or API key used"),
    ),
    security(
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/login/renew")]
pub async fn renew(
    config: web::Data<AppConfig>,
    store: web::Data<SharedStore>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    let login_id = req.extensions()
        .get::<LoginId>()
        .copied()
        .ok_or_else(|| ErrorUnauthorized("Unauthorized request"))?;
    let mut user = get_authorized_user(&req, false)?;
    let login_expiry_hours = config.security.login_expiry_hours;
    let renewed = user.renew_login(login_id.0, login_expiry_hours)
        .ok_or_else(|| ErrorUnauthorized("Unauthorized request"))?;
    let store_lock = get_store_write_lock(&store).await;
    update_store_user(store_lock, &mut user, |err| {
        get_user_update_internal_error(err)
    }).await?;
    Ok(HttpResponse::Ok().json(RenewResponse {
        login_token_id: renewed.id,
        login_token_expires_at: renewed.expires_at,
    }))
}
// **************************************************************************************************
// Endpoint: GET /api/v1/users
// Handler:  get_users()
// **************************************************************************************************
#[utoipa::path(
    summary = "Returns the list of registered users",
    description = "This endpoint returns the list of register users",
    get,
    path = "/api/v1/users",
    responses(
        (status = 200, description = "User list loaded sucessfully", body=[UserItem]),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/users")]
pub async fn get_users(
    req: HttpRequest,
    store: web::Data<SharedStore>
) -> Result<HttpResponse, Error> {
    let store_lock = get_store_read_lock(&store).await;
    let _ = get_authorized_user(&req, true)?;
    let store_users = store_lock.get_users().await.map_err(|err| {
        get_users_fetch_internal_error(&err)
    })?;

    let user_items: Vec<UserItem> = store_users
        .iter()
        .map(UserItem::from_store_user)
        .collect();

    Ok(HttpResponse::Ok().json(user_items))
}
// **************************************************************************************************
// Endpoint: POST /api/v1/users/
// Handler:  create_user()
// **************************************************************************************************
/// Create user request
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct CreateUserRequest {
    /// Is administrator?
    pub is_admin: bool,
    /// Username
    pub username: String,
    /// Full name 
    pub full_name: String,
    /// Email address
    pub email: String,
    /// Password
    pub password: String,
}

impl CreateUserRequest {
    pub fn into_user(&self, auth_service: &AuthService) -> Result<User, Error> {
        let password_hash = if self.password.is_empty() {
            "".to_string()
        } else {
            auth_service
                .hash_password(&self.password)
                .map_err(|err| get_hash_password_internal_error(&err))?
        };
        Ok(
            User {
                id: Uuid::nil(),
                is_admin: self.is_admin,
                // Edge-trim silently — see UpdateProfileRequest::apply_to_store_user
                // for the rationale; same rule applies on admin-side create.
                username: self.username.trim().to_string(),
                full_name: self.full_name.trim().to_string(),
                emails: vec![data_model::UserEmail::new_primary_verified(&self.email)],
                password_hash,
                api_key: Uuid::nil(),
                logins: vec![],
                oauth_identities: vec![],
            }
        )
    }
}

#[utoipa::path(
    summary = "Creates a new user",
    description = "This endpoint creates a new user and, if successful, returns the newly created user item containing its allocated ID",
    post,
    path = "/api/v1/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserItem),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized request"),
        (status = 409, description = "User with the given username or email already exists")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/users")]
pub async fn create_user(
    create_req: web::Json<CreateUserRequest>,
    auth_service: web::Data<AuthService>,
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store).await;
    let _ = get_authorized_user(&req, true)?;
    let create_req_data: CreateUserRequest = create_req.into_inner();
    let mut store_user = create_req_data.into_user(&auth_service)?;

    match store_lock.create_user(&mut store_user).await {
        Ok(()) => Ok(
            HttpResponse::Created()
            .insert_header(("Location", format!("/api/v1/users/{}", encode(&store_user.id.to_string()))))
            .json(UserItem::from_store_user(&store_user))
        ),
        Err(err) => {
            match err {
                StoreError::UserEmailExists() | StoreError::UserNameExists()  => Err(get_user_exists_error()),
                StoreError::UserNameMissing() => Err(get_missing_username_request_error()),
                StoreError::UserPasswordMissing() => Err(get_missing_password_request_error()),
                 _ => Err(get_user_create_internal_error(&err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: GET /api/v1/users/{id}
// Handler:  get_user()
// **************************************************************************************************
#[utoipa::path(
    summary = "Loads an existing user",
    description = "This endpoint attempts to load a user item given its ID and, if successful, returns the details",
    get,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the user to retrieve")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = UserItem),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "User not found")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/users/{id}")]
pub async fn get_user(
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let store_lock = get_store_read_lock(&store).await;
    let _ = get_authorized_user(&req, true)?;
    let id = user_id_from_str(&path.into_inner())?;

    match store_lock.get_user(id).await {
        Ok(user) => Ok(HttpResponse::Ok().json(UserItem::from_store_user(&user))),
        Err(err) => {
            match err {
               StoreError::UserIdNotFound(id) => Err(get_user_not_found_error(id)),
                _ => Err(get_user_fetch_internal_error(id, &err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: PUT /api/v1/users/{id}
// Handler:  update_user()
// **************************************************************************************************
/// Update user request
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct UpdateUserRequest {
    /// Is administrator?
    pub is_admin: bool,
    /// Username
    pub username: String,
    /// Full name 
    pub full_name: String,
    /// Email address
    pub email: String,
}

impl UpdateUserRequest {
    pub fn apply_to_store_user(&self, user: &mut User) {
        user.is_admin = self.is_admin;
        // Edge-trim silently — see UpdateProfileRequest::apply_to_store_user
        // for the rationale; same rule applies on admin-side update.
        user.username = self.username.trim().to_string();
        user.full_name = self.full_name.trim().to_string();
        user.set_primary_email_address(&self.email);
    }
}

#[utoipa::path(
    summary = "Updates an existing user",
    description = "This endpoint attempts to update an existing user given its ID and, if successful, returns the updated details",
    put,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the user to update")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserItem),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "User not found"),
        (status = 409, description = "User with the given username or email already exists")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[put("/users/{id}")]
pub async fn update_user(
    update_req: web::Json<UpdateUserRequest>,
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let store_lock = get_store_write_lock(&store).await;
    let _ = get_authorized_user(&req, true)?;
    let id = user_id_from_str(&path.into_inner())?;
    let update_req_data = update_req.into_inner();

    match store_lock.get_user(id).await {
        Ok(mut user) => {
            update_req_data.apply_to_store_user(&mut user);
            update_store_user(store_lock, &mut user, |err| {
                get_user_update_internal_error(err)
            }).await

        },
        Err(err) => {
            match err {
               StoreError::UserIdNotFound(id) => Err(get_user_not_found_error(id)),
                _ => Err(get_user_fetch_internal_error(id, &err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: DELETE /api/v1/users/{id}
// Handler:  delete_user()
// **************************************************************************************************
#[utoipa::path(
    summary = "Deletes an existing user",
    description = "This endpoint attempts to delete an existing user given its ID",
    delete,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the user to delete")
    ),
    responses(
        (status = 200, description = "User deleted successfully"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "User not found"),
        (status = 409, description = "Cannot delete the last admin account")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[delete("/users/{id}")]
pub async fn delete_user(
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store).await;
    let _ = get_authorized_user(&req, true)?;
    let id = user_id_from_str(&path.into_inner())?;

    if is_last_admin(&store_lock, id).await? {
        return Err(get_cannot_delete_last_admin_error());
    }

    match store_lock.delete_user(id).await {
        Ok(()) => {
            Ok(HttpResponse::Ok().body(format!("user with id '{id}' deleted")))
        }    
        Err(err) => {
            match err {
                StoreError::UserIdNotFound(id) => Err(get_user_not_found_error(id)),
                _ => Err(get_user_fetch_internal_error(id, &err))
            }
        }
    }
}
// **************************************************************************************************
// Endpoint: GET /api/v1/mazes
// Handler:  get_mazes()
// **************************************************************************************************
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")] 
struct GetMazeListQueryParams {
    include_definitions: Option<bool>,
}
#[utoipa::path(
    summary = "Returns the list of available mazes",
    description = "This endpoint returns the list of maze IDs, names and (optionally) their definitions that the user currently has access to",
    get,
    path = "/api/v1/mazes",
    params(
        ("includeDefinitions" = bool, Query, description = "Include the definitions for the mazes (default: false)")
    ),    
    responses(
        (status = 200, description = "Maze list loaded sucessfully", body=[MazeItem]),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized request")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/mazes")]
pub async fn get_mazes(
    query: Query<GetMazeListQueryParams>,
    store: web::Data<SharedStore>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let include_definitions = query.include_definitions.unwrap_or(false); 
    let store_lock = get_store_read_lock(&store).await;
    let user = get_authorized_user(&req, false)?;
    let stored_items = store_lock.get_maze_items(&user, include_definitions).await.map_err(|err| {
        get_mazes_fetch_internal_error(&err)
    })?;
    Ok(HttpResponse::Ok().json(stored_items))    
}
// **************************************************************************************************
// Endpoint: POST /api/v1/mazes/
// Handler:  create_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Creates a new maze",
    description = "This endpoint creates a new maze and, if successful, returns the newly created maze object containing its allocated ID",
    post,
    path = "/api/v1/mazes",
    request_body = Maze,
    responses(
        (status = 201, description = "Maze created successfully", body = Maze),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized request"),
        (status = 409, description = "Maze with the given id already exists")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/mazes")]
pub async fn create_maze(
    req_maze: web::Json<Maze>,
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store).await;
    let user = get_authorized_user(&req, false)?;
    let mut maze: Maze = req_maze.into_inner();

    match store_lock.create_maze(&user, &mut maze).await {
        Ok(()) => Ok(
                HttpResponse::Created()
                .insert_header(("Location", format!("/api/v1/mazes/{}", encode(&maze.id))))
                .json(maze)),
        Err(err) => {
            match err {
                StoreError::MazeIdExists(id) => Err(get_maze_exists_error(&id)),
                _ => Err(get_maze_create_internal_error(&err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: GET /api/v1/maze/{id}
// Handler:  get_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Loads an existing maze",
    description = "This endpoint attempts to load a maze given its ID and, if successful, returns the maze definition",
    get,
    path = "/api/v1/mazes/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to retrieve")
    ),
    responses(
        (status = 200, description = "Maze retrieved successfully", body = Maze),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Maze not found")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/mazes/{id}")]
pub async fn get_maze(
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let store_lock = get_store_read_lock(&store).await;
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();

    match store_lock.get_maze(&user, &id).await {
        Ok(maze) => Ok(HttpResponse::Ok().json(maze)),
        Err(err) => {
            match err {
               StoreError::MazeIdNotFound(id) => Err(get_maze_not_found_error(&id)),
                _ => Err(get_maze_fetch_internal_error(&id, &err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: PUT /api/v1/mazes/{id}
// Handler:  update_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Updates an existing maze",
    description = "This endpoint attempts to update an existing maze given its ID and, if successful, returns the updated maze definition",
    put,
    path = "/api/v1/mazes/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to update")
    ),
    request_body = Maze,
    responses(
        (status = 200, description = "Maze updated successfully", body = Maze),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Maze not found")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[put("/mazes/{id}")]
pub async fn update_maze(
    req_maze: web::Json<Maze>,
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store).await;
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let mut maze = req_maze.into_inner();

    if id != maze.id {
        return Err(get_maze_id_mismatch_error(&id, &maze.id));
    }

    match store_lock.update_maze(&user, &mut maze).await {
        Ok(_) => Ok(HttpResponse::Ok().json(maze)),
        Err(err) => {
            match err {
               StoreError::MazeIdNotFound(id) => Err(get_maze_not_found_error(&id)),
                _ => Err(get_maze_fetch_internal_error(&id, &err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: DELETE /api/v1/mazes/{id}
// Handler:  delete_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Deletes an existing maze",
    description = "This endpoint attempts to delete an existing maze given its ID",
    delete,
    path = "/api/v1/mazes/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to delete")
    ),
    responses(
        (status = 200, description = "Maze deleted successfully"),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Maze not found")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[delete("/mazes/{id}")]
pub async fn delete_maze(
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store).await;
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();

    match store_lock.delete_maze(&user, &id).await {
        Ok(()) => {
            Ok(HttpResponse::Ok().body(format!("maze with id '{id}' deleted")))
        }    
        Err(err) => {
            match err {
                StoreError::MazeIdNotFound(id) => Err(get_maze_not_found_error(&id)),
                _ => Err(get_maze_fetch_internal_error(&id, &err))
            }
        }
    }
}
// **************************************************************************************************
// Endpoint: GET /api/v1/mazes/{id}/solution
// Handler:  get_maze_solution()
// **************************************************************************************************
#[utoipa::path(
    summary = "Attempts to solve an existing maze",
    description = "This endpoint attempts to solve a maze given its ID and, if successful, returns a maze solution containing the solution path",
    get,
    path = "/api/v1/mazes/{id}/solution",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to solve")
    ),
    responses(
        (status = 200, description = "Maze solved successfully", body = MazeSolution),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Maze not found"),
        (status = 422, description = "Maze could not be solved")    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[get("/mazes/{id}/solution")]
pub async fn get_maze_solution(
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let store_lock = get_store_read_lock(&store).await;
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();

    match store_lock.get_maze(&user, &id).await {
        Ok(maze) => {
            match maze.solve() {
                Ok(solution) => Ok(HttpResponse::Ok().json(solution)),
                Err(err) => Err(get_maze_solve_error(&err))
            }
        }    
        Err(err) => {
            match err {
               StoreError::MazeIdNotFound(id) => Err(get_maze_not_found_error(&id)),
                _ => Err(get_maze_fetch_internal_error(&id, &err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: POST /api/v1/solve-maze/
// Handler:  solve_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Attempts to solve a maze definition that is supplied by the caller",
    description = "This endpoint attempts to solve a maze definition that is supplied by the caller and, if successful, returns a maze solution containing the solution path",
    post,
    path = "/api/v1/solve-maze",
    request_body = Maze,
    responses(
        (status = 200, description = "Maze solved successfully", body = MazeSolution),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized request"),
        (status = 422, description = "Maze could not be solved")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/solve-maze")]
pub async fn solve_maze(
    req_maze: web::Json<Maze>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let _ = get_authorized_user(&req, false)?;
    let maze: Maze = req_maze.into_inner();
    match maze.solve() {
        Ok(solution) => Ok(HttpResponse::Ok().json(solution)),
        Err(err) => Err(get_maze_solve_error(&err))
    }
}
// **************************************************************************************************
// Endpoint: POST /api/v1/mazes/generate
// Handler:  generate_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Generates a new maze from the provided options",
    description = "This endpoint generates a new maze using the supplied generator options and, if successful, returns the generated maze definition. The returned maze will have empty id and name fields; use POST /api/v1/mazes to persist it.",
    post,
    path = "/api/v1/mazes/generate",
    request_body = GeneratorOptions,
    responses(
        (status = 200, description = "Maze generated successfully", body = Maze),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized request"),
        (status = 422, description = "Maze could not be generated")
    ),
    security(
        ("api_key" = []),
        ("login_token" = [])
    ),
    tags = ["v1"]
)]
#[post("/mazes/generate")]
pub async fn generate_maze(
    options: web::Json<GeneratorOptions>,
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    let _ = get_authorized_user(&req, false)?;
    let generator = Generator { options: options.into_inner() };
    match generator.generate() {
        Ok(maze) => Ok(HttpResponse::Ok().json(maze)),
        Err(err) => Err(get_maze_generate_error(&err))
    }
}
