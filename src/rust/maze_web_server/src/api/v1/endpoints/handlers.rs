use crate::config::app::AppConfig;
use crate::middleware::auth::{ApiKey, LoginId};
use crate::service::auth::AuthService;
use crate::SharedFeatures;


use data_model::{Maze, User};
use maze::{Error as MazeError, Generator, GeneratorOptions, MazeSolution, MazeSolver};
use storage::{Error as StoreError, MazeItem, Store, SharedStore};

use actix_web::{delete, get, post, put, web, web::Query, HttpMessage, HttpRequest, HttpResponse, Error,
    error::{ErrorBadRequest, ErrorConflict, ErrorForbidden, ErrorInternalServerError, ErrorNotFound, ErrorUnauthorized, ErrorUnprocessableEntity, InternalError}
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::{RwLockReadGuard, RwLockWriteGuard, RwLock, Arc};
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

fn verify_user_credentials(store: &web::Data<SharedStore>, auth_service: &AuthService,
    username: &str, password: &str) -> Result<User, Error> {

    if username.trim().is_empty() || password.trim().is_empty() {
        return Err(ErrorUnprocessableEntity("Username and password must be provided"));
    }

    let store_lock = get_store_read_lock(store)?;

    let user = store_lock.find_user_by_name(username).map_err(|err| {
        match err {
            StoreError::UserNotFound() => ErrorUnauthorized( "Invalid username or password"),
            _ => ErrorInternalServerError("Failed to process login request"),
        }
    })?;

    let password_matches = auth_service.verify_password(&user.password_hash, password).map_err(|err| {
        log::error!("Password verification failed: {:?}", err);
        ErrorInternalServerError("Internal authentication error")
    })?;

    if !password_matches {
        return Err(ErrorUnauthorized( "Invalid username or password"));
    }

    Ok(user)
}

fn get_store_read_lock(
    store: &web::Data<Arc<RwLock<Box<dyn Store>>>>,
) -> Result<RwLockReadGuard<'_, Box<dyn Store>>, Error> {
    store.read().map_err(|_| {
        ErrorInternalServerError("Failed to acquire store read lock")
    })
}

fn get_store_write_lock(
    store: &web::Data<Arc<RwLock<Box<dyn Store>>>>,
) -> Result<RwLockWriteGuard<'_, Box<dyn Store>>, Error> {
    store.write().map_err(|_| {
        ErrorInternalServerError("Failed to acquire store write lock")
    })
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
    ErrorInternalServerError(format!("Error hashing password: {}", err))
}

// User-related errors
fn get_users_fetch_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error fetching users: {}", err))
}
fn get_user_create_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error creating user: {}", err))
}

fn get_user_update_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error updating user: {}", err))
}

fn get_user_not_found_error(id: String) -> Error {
    ErrorNotFound(format!("User with id '{}' not found", id))
}

fn get_user_exists_error() -> Error {
    ErrorConflict("User with the given username or email already exists".to_string())
}

fn get_invalid_request_error(reason: &str) -> Error {
    ErrorBadRequest(format!("Invalid request ({})", reason))
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

fn get_user_fetch_internal_error(id: Uuid, err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error fetching user item with id '{}': {}", id, err))
}

fn get_cannot_delete_last_admin_error() -> Error {
    ErrorConflict("Cannot delete the last admin account".to_string())
}

fn is_last_admin(store_lock: &RwLockWriteGuard<'_, Box<dyn Store>>, user_id: Uuid) -> Result<bool, Error> {
    let admins = store_lock.get_admin_users().map_err(|err| get_users_fetch_internal_error(&err))?;
    Ok(admins.len() == 1 && admins[0].id == user_id)
}

// Maze-related errors
fn get_mazes_fetch_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error fetching maze items: {}", err))
}

fn get_maze_create_internal_error(err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error creating maze: {}", err))
}

fn get_maze_not_found_error(id: &str) -> Error {
    ErrorNotFound(format!("Maze with id '{}' not found", id))
}

fn get_maze_exists_error(id: &str) -> Error {
    ErrorConflict(format!("Maze with id '{}' already exists", id))
}

fn get_maze_fetch_internal_error(id: &str, err: &StoreError) -> Error {
    ErrorInternalServerError(format!("Error fetching maze item with id '{}': {}", id, err))
}

fn get_maze_id_mismatch_error(url_id: &str, maze_id: &str) -> Error {
    ErrorBadRequest(format!("URL ID '{}' and body maze ID '{}' do not match", url_id, maze_id))
}

pub (crate) fn get_maze_solve_error_string(err: &MazeError) -> String {
    format!("The maze could not be solved: {}", err)
}

fn get_maze_solve_error(err: &MazeError) -> Error {
    ErrorUnprocessableEntity(get_maze_solve_error_string(err))
}

pub (crate) fn get_maze_generate_error_string(err: &MazeError) -> String {
    format!("The maze could not be generated: {}", err)
}

fn get_maze_generate_error(err: &MazeError) -> Error {
    ErrorUnprocessableEntity(get_maze_generate_error_string(err))
}

fn update_store_user<F>(
    mut store_lock: RwLockWriteGuard<'_, Box<dyn Store>>, 
    user: &mut User,
    handle_internal_error: F,
) -> Result<HttpResponse, Error> 
where
    F: Fn(&StoreError) -> Error,
{
    match store_lock.update_user(user) {
        Ok(_) =>  Ok(HttpResponse::Ok().json(UserItem::from_store_user(user))),
        Err(err) => {
            match err {
                StoreError::UserEmailExists() | StoreError::UserNameExists()  => Err(get_user_exists_error()),
                StoreError::UserNameMissing() => Err(get_missing_username_request_error()),
                StoreError::UserEmailInvalid() => Err(get_invalid_email_request_error()),
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
    /// Email address
    pub email: String,
}
 
impl UserItem {
    pub fn from_store_user(user: &User) -> UserItem {
        UserItem {
            id:  user.id,
            is_admin: user.is_admin,
            username: user.username.clone(),
            full_name: user.full_name.clone(),
            email: user.email.clone(),
        }
    }    
}
// **************************************************************************************************
// Endpoint: GET /api/v1/features
// Handler:  get_features()
// **************************************************************************************************
/// Response body for `GET /api/v1/features`
#[derive(Serialize, Deserialize, ToSchema, Debug, Clone)]
pub struct AppFeaturesResponse {
    /// Whether new users can self-register via the signup endpoint
    pub allow_signup: bool,
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
) -> Result<HttpResponse, Error> {
    let features_lock = features.read().map_err(|_| {
        ErrorInternalServerError("Failed to acquire features read lock")
    })?;
    Ok(HttpResponse::Ok().json(AppFeaturesResponse {
        allow_signup: features_lock.allow_signup,
    }))
}

// **************************************************************************************************
// Endpoint: PUT /api/v1/admin/features
// Handler:  update_admin_features()
// **************************************************************************************************

fn update_features_in_config(config_path: &str, new_features: &AppFeaturesResponse) -> Result<(), Error> {
    let content = std::fs::read_to_string(config_path).unwrap_or_default();
    let mut doc = content.parse::<toml_edit::DocumentMut>().map_err(|e| {
        ErrorInternalServerError(format!("Failed to parse config file: {}", e))
    })?;
    if doc.get("features").is_none() {
        doc["features"] = toml_edit::table();
    }
    doc["features"]["allow_signup"] = toml_edit::value(new_features.allow_signup);
    std::fs::write(config_path, doc.to_string()).map_err(|e| {
        ErrorInternalServerError(format!("Failed to write config file: {}", e))
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
) -> Result<HttpResponse, Error> {
    get_authorized_user(&req, true)?;

    let new_features = body.into_inner();
    update_features_in_config(&config.config_path, &new_features)?;

    let mut features_lock = features.write().map_err(|_| {
        ErrorInternalServerError("Failed to acquire features write lock")
    })?;
    features_lock.allow_signup = new_features.allow_signup;

    Ok(HttpResponse::Ok().json(AppFeaturesResponse {
        allow_signup: features_lock.allow_signup,
    }))
}

// **************************************************************************************************
// Endpoint: POST /api/v1/signup
// Handler:  signup()
// **************************************************************************************************
/// Signup request
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct SignupRequest {
    /// Username
    pub username: String,
    /// Full name
    pub full_name: String,
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
                username: self.username.clone(),
                full_name: self.full_name.clone(),
                email: self.email.clone(),
                password_hash,
                api_key: Uuid::nil(),
                logins: vec![],
            }
        )
    }
}

#[utoipa::path(
    summary = "Sign up as a new user",
    description = "This endpoint registers a new (non-admin) user account",
    post,
    path = "/api/v1/signup",
    request_body = SignupRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserItem),
        (status = 400, description = "Invalid request"),
        (status = 403, description = "Signup is disabled on this server"),
        (status = 409, description = "User with the given username or email already exists")
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
    let features_lock = features.read().map_err(|_| {
        ErrorInternalServerError("Failed to acquire features read lock")
    })?;
    if !features_lock.allow_signup {
        return Err(ErrorForbidden("Signup is disabled on this server"));
    }
    drop(features_lock);

    let mut store_lock = get_store_write_lock(&store)?;
    let signup_req_data: SignupRequest = signup_req.into_inner();
    let mut store_user = signup_req_data.into_user(&auth_service)?;

    match store_lock.create_user(&mut store_user) {
        Ok(()) => Ok(
            HttpResponse::Created()
            .insert_header(("Location", "/api/v1/users/me"))
            .json(UserItem::from_store_user(&store_user))
        ),
        Err(err) => {
            match err {
                StoreError::UserEmailExists() | StoreError::UserNameExists() => Err(get_user_exists_error()),
                StoreError::UserNameMissing() => Err(get_missing_username_request_error()),
                StoreError::UserPasswordMissing() => Err(get_missing_password_request_error()),
                _ => Err(get_user_create_internal_error(&err))
            }
        }
    }
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
    let mut store_lock = get_store_write_lock(&store)?;
    let user = get_authorized_user(&req, false)?;

    if is_last_admin(&store_lock, user.id)? {
        return Err(get_cannot_delete_last_admin_error());
    }

    match store_lock.delete_user(user.id) {
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
/// Change password request
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct ChangePasswordRequest {
    /// Current password
    pub current_password: String,
    /// New password
    pub new_password: String,
}

#[utoipa::path(
    summary = "Changes the authenticated user's password",
    description = "This endpoint allows the currently authenticated user to change their password by providing their current password and a new password",
    put,
    path = "/api/v1/users/me/password",
    request_body = ChangePasswordRequest,
    responses(
        (status = 204, description = "Password changed successfully"),
        (status = 400, description = "Invalid request (e.g. weak new password or empty current password)"),
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

    if change_req_data.current_password.is_empty() {
        return Err(get_invalid_request_error("current password must be provided"));
    }

    let password_matches = auth_service
        .verify_password(&user.password_hash, &change_req_data.current_password)
        .map_err(|err| {
            log::error!("Password verification failed: {:?}", err);
            ErrorInternalServerError("Internal authentication error")
        })?;

    if !password_matches {
        return Err(ErrorUnauthorized("Current password is incorrect"));
    }

    validate_password_complexity(&change_req_data.new_password)?;

    let new_hash = auth_service
        .hash_password(&change_req_data.new_password)
        .map_err(|err| get_hash_password_internal_error(&err))?;

    let mut store_lock = get_store_write_lock(&store)?;
    user.password_hash = new_hash;

    match store_lock.update_user(&mut user) {
        Ok(_) => Ok(HttpResponse::NoContent().finish()),
        Err(err) => Err(get_user_update_internal_error(&err)),
    }
}
// **************************************************************************************************
// Endpoint: PUT /api/v1/users/me/profile
// Handler:  update_profile_me()
// **************************************************************************************************
/// Update profile request
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct UpdateProfileRequest {
    /// Username
    pub username: String,
    /// Full name
    pub full_name: String,
    /// Email address
    pub email: String,
}

impl UpdateProfileRequest {
    pub fn apply_to_store_user(&self, user: &mut User) {
        user.username = self.username.clone();
        user.full_name = self.full_name.clone();
        user.email = self.email.clone();
    }
}

#[utoipa::path(
    summary = "Updates the authenticated user's profile",
    description = "This endpoint allows the currently authenticated user to update their username, full name, and email address",
    put,
    path = "/api/v1/users/me/profile",
    request_body = UpdateProfileRequest,
    responses(
        (status = 200, description = "Profile updated successfully", body = UserItem),
        (status = 400, description = "Invalid request (e.g. empty username or invalid email)"),
        (status = 401, description = "Unauthorized request"),
        (status = 409, description = "Username or email already in use by another user")
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
    let store_lock = get_store_write_lock(&store)?;
    update_req.into_inner().apply_to_store_user(&mut user);
    update_store_user(store_lock, &mut user, |err| get_user_update_internal_error(err))
}
// **************************************************************************************************
// Endpoint: GET /api/v1/login
// Handler:  login()
// **************************************************************************************************
/// Login request
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct LoginRequest {
    /// Username
    pub username: String,
    /// Full name 
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
    description = "This endpoint attempts to login a user by username + password",
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
    let mut user = verify_user_credentials(&store, &auth_service, &login_req.username, &login_req.password)?;
    let login_expiry_hours = config.security.login_expiry_hours;
    let login = user.create_login(login_expiry_hours, get_caller_ip_address(&req), get_caller_device_info(&req));
    let store_lock = get_store_write_lock(&store)?;
    update_store_user(store_lock, &mut user, |err| {
        get_user_update_internal_error(err)
    })?;

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
    let store_lock = get_store_write_lock(&store)?;

    user.remove_login(login_id);

    update_store_user(store_lock, &mut user, |err| {
        get_user_update_internal_error(err)
    })?;         

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
    let store_lock = get_store_write_lock(&store)?;
    update_store_user(store_lock, &mut user, |err| {
        get_user_update_internal_error(err)
    })?;
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
    let store_lock = get_store_read_lock(&store)?;
    let _ = get_authorized_user(&req, true)?;
    let store_users = store_lock.get_users().map_err(|err| {
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
                username: self.username.clone(),
                full_name: self.full_name.clone(),
                email: self.email.clone(),
                password_hash,
                api_key: Uuid::nil(),
                logins: vec![],
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
    let mut store_lock = get_store_write_lock(&store)?;
    let _ = get_authorized_user(&req, true)?;
    let create_req_data: CreateUserRequest = create_req.into_inner();
    let mut store_user = create_req_data.into_user(&auth_service)?;

    match store_lock.create_user(&mut store_user) {
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
    let store_lock = get_store_read_lock(&store)?;
    let _ = get_authorized_user(&req, true)?;
    let id = user_id_from_str(&path.into_inner())?;

    match store_lock.get_user(id) {
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
        user.username = self.username.clone();
        user.full_name = self.full_name.clone();
        user.email = self.email.clone();
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
    let store_lock = get_store_write_lock(&store)?;
    let _ = get_authorized_user(&req, true)?;
    let id = user_id_from_str(&path.into_inner())?;
    let update_req_data = update_req.into_inner();

    match store_lock.get_user(id) {
        Ok(mut user) => {
            update_req_data.apply_to_store_user(&mut user);
            update_store_user(store_lock, &mut user, |err| {
                get_user_update_internal_error(err)
            })            
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
    let mut store_lock = get_store_write_lock(&store)?;
    let _ = get_authorized_user(&req, true)?;
    let id = user_id_from_str(&path.into_inner())?;

    if is_last_admin(&store_lock, id)? {
        return Err(get_cannot_delete_last_admin_error());
    }

    match store_lock.delete_user(id) {
        Ok(()) => {
            Ok(HttpResponse::Ok().body(format!("user with id '{}' deleted", id)))
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
    let store_lock = get_store_read_lock(&store)?;
    let user = get_authorized_user(&req, false)?;
    let stored_items = store_lock.get_maze_items(&user, include_definitions).map_err(|err| {
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
    let mut store_lock = get_store_write_lock(&store)?;
    let user = get_authorized_user(&req, false)?;
    let mut maze: Maze = req_maze.into_inner();

    match store_lock.create_maze(&user, &mut maze) {
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
    let store_lock = get_store_read_lock(&store)?;
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();

    match store_lock.get_maze(&user, &id) {
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
    let mut store_lock = get_store_write_lock(&store)?;
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();
    let mut maze = req_maze.into_inner();

    if id != maze.id {
        return Err(get_maze_id_mismatch_error(&id, &maze.id));
    }

    match store_lock.update_maze(&user, &mut maze) {
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
    let mut store_lock = get_store_write_lock(&store)?;
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();

    match store_lock.delete_maze(&user, &id) {
        Ok(()) => {
            Ok(HttpResponse::Ok().body(format!("maze with id '{}' deleted", id)))
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
    let store_lock = get_store_read_lock(&store)?;
    let user = get_authorized_user(&req, false)?;
    let id = path.into_inner();

    match store_lock.get_maze(&user, &id) {
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
