use crate::service::auth::AuthService;

use data_model::{Maze, User};
use maze::{Error as MazeError, MazeSolution, MazeSolver};
use storage::{Error as StoreError, MazeItem, Store, SharedStore};
use actix_web::{delete, get, post, put, web, web::Query, HttpMessage, HttpRequest, HttpResponse, Error, error::ErrorUnauthorized};
use serde::{Deserialize, Serialize};
use std::sync::{RwLockReadGuard, RwLockWriteGuard, RwLock, Arc};
use urlencoding::encode;
use utoipa::ToSchema;
use uuid::Uuid;
// **************************************************************************************************
// Private utility functions
// **************************************************************************************************
fn get_authorized_user(req: HttpRequest, admin_required: bool) -> Result<User, Error> {
    if let Some(user) = req.extensions().get::<User>() {
        if admin_required && !user.is_admin {
            return Err(ErrorUnauthorized( "Unauthorized request"));
        }
        Ok(user.clone())
    } else {
        Err(ErrorUnauthorized( "Unauthorized request"))
    }
}

fn get_store_read_lock(
    store: &web::Data<Arc<RwLock<Box<dyn Store>>>>,
) -> Result<RwLockReadGuard<'_, Box<dyn Store>>, Error> {
    store.read().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire store read lock")
    })
}

fn get_store_write_lock(
    store: &web::Data<Arc<RwLock<Box<dyn Store>>>>,
) -> Result<RwLockWriteGuard<'_, Box<dyn Store>>, Error> {
    store.write().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire store write lock")
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
    actix_web::error::ErrorInternalServerError(format!("Error hashing password: {}", err))
}

// User-related errors
fn get_user_create_internal_error(err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error creating user: {}", err))
}

fn get_user_update_internal_error(err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error updating user: {}", err))
}

fn get_user_not_found_error(id: String) -> Error {
    actix_web::error::ErrorNotFound(format!("User with id '{}' not found", id))
}

fn get_user_exists_error() -> Error {
    actix_web::error::ErrorConflict("User with the given username or email already exists".to_string())
}

fn get_invalid_request_error(reason: &str) -> Error {
    actix_web::error::ErrorBadRequest(format!("Invalid request ({})", reason))
}

fn get_missing_username_request_error() -> Error {
    get_invalid_request_error("missing username")
}

fn get_missing_password_request_error() -> Error {
    get_invalid_request_error("missing password")
}

fn get_invalid_email_request_error() -> Error {
    get_invalid_request_error("invalid email")
}

fn get_user_fetch_internal_error(id: Uuid, err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error fetching user item with id '{}': {}", id, err))
}

// Maze-related errors
fn get_mazes_fetch_internal_error(err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error fetching maze items: {}", err))
}

fn get_maze_create_internal_error(err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error creating maze: {}", err))
}

fn get_maze_not_found_error(id: &str) -> Error {
    actix_web::error::ErrorNotFound(format!("Maze with id '{}' not found", id))
}

fn get_maze_exists_error(id: &str) -> Error {
    actix_web::error::ErrorConflict(format!("Maze with id '{}' already exists", id))
}

fn get_maze_fetch_internal_error(id: &str, err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error fetching maze item with id '{}': {}", id, err))
}

fn get_maze_id_mismatch_error(url_id: &str, maze_id: &str) -> Error {
    actix_web::error::ErrorBadRequest(format!("URL ID '{}' and body maze ID '{}' do not match", url_id, maze_id))
}

pub (crate) fn get_maze_solve_error_string(err: &MazeError) -> String {
    format!("The maze could not be solved: {}", err)
}

fn get_maze_solve_error(err: &MazeError) -> Error {
    actix_web::error::ErrorUnprocessableEntity(get_maze_solve_error_string(err))
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
    #[schema(value_type = String)] // Treat as string during serlialization
    /// Login token id
    pub login_token_id: Uuid,
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
    auth_service: web::Data<AuthService>,
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    // TO DO - validate username/password and generate/store login token  
    let store_lock = get_store_write_lock(&store)?;
    let login_token_id = Uuid::new_v4();    

    Ok(HttpResponse::Ok().json(LoginResponse {
        login_token_id: login_token_id,
    }))
}
// **************************************************************************************************
// Endpoint: GET /api/v1/logout
// Handler:  logout()
// **************************************************************************************************
#[utoipa::path(
    summary = "Logout",
    description = "This endpoint attempts to logout a user baed on the login bearer token provided in the header",
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
    auth_service: web::Data<AuthService>,
    store: web::Data<SharedStore>,  
    req: HttpRequest
) -> Result<HttpResponse, Error> {
    
    // TO DO - return 204 if API key provided but no login token 
    let _ = get_authorized_user(req, false)?;

    // TO DO- implement logout
    Ok(HttpResponse::NoContent().finish())
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
    let _ = get_authorized_user(req, true)?;
    let store_users = store_lock.get_users().map_err(|err| {
        get_mazes_fetch_internal_error(&err)
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
                login_tokens: None,
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
    let _ = get_authorized_user(req, true)?;
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
    println!("*** get_user() called ****");

    let store_lock = get_store_read_lock(&store)?;
    let _ = get_authorized_user(req, true)?;
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
    let mut store_lock = get_store_write_lock(&store)?;
    let _ = get_authorized_user(req, true)?;
    let id = user_id_from_str(&path.into_inner())?;
    let update_req_data = update_req.into_inner();

    match store_lock.get_user(id) {
        Ok(mut user) => {
            update_req_data.apply_to_store_user(&mut user);
            match store_lock.update_user(&mut user) {
                Ok(_) =>  Ok(HttpResponse::Ok().json(UserItem::from_store_user(&user))),
                Err(err) => {
                    match err {
                        StoreError::UserEmailExists() | StoreError::UserNameExists()  => Err(get_user_exists_error()),
                        StoreError::UserNameMissing() => Err(get_missing_username_request_error()),
                        StoreError::UserEmailInvalid() => Err(get_invalid_email_request_error()),
                        _ => Err(get_user_update_internal_error(&err))
                    }    
                }
            } 
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
        (status = 404, description = "User not found")
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
    let _ = get_authorized_user(req, true)?;
    let id = user_id_from_str(&path.into_inner())?;

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
    let user = get_authorized_user(req, false)?;
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
    let user = get_authorized_user(req, false)?;
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
    let user = get_authorized_user(req, false)?;
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
    let user = get_authorized_user(req, false)?;
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
    let user = get_authorized_user(req, false)?;
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
    let user = get_authorized_user(req, false)?;
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
    let _ = get_authorized_user(req, false)?;
    let maze: Maze = req_maze.into_inner();
    match maze.solve() {
        Ok(solution) => Ok(HttpResponse::Ok().json(solution)),
        Err(err) => Err(get_maze_solve_error(&err))
    }
}
