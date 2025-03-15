use maze::{Maze, MazeError, Solution};
use storage::{MazeItem, Store, SharedStore, StoreError, User};
use actix_web::{delete, get, post, put, web, web::Query, HttpMessage, HttpRequest, HttpResponse, Error, error::ErrorUnauthorized};
use serde::{Deserialize, Serialize};
use std::sync::{RwLockReadGuard, RwLockWriteGuard, RwLock, Arc};
use urlencoding::encode;
use utoipa::ToSchema;
use uuid::Uuid;

// **************************************************************************************************
// Private utility functions
// **************************************************************************************************
fn get_authorized_user(req: HttpRequest, is_admin: bool) -> Result<User, Error> {
    if let Some(user) = req.extensions().get::<User>() {
        if is_admin && !user.is_admin {
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

fn get_maze_solve_error_string(err: &MazeError) -> String {
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
        ("api_key" = [])
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
    pub fn to_store_user(&self) -> User {
        User {
            id: Uuid::nil(),
            is_admin: self.is_admin,
            username: self.username.clone(),
            full_name: self.full_name.clone(),
            email: self.email.clone(),
            password_hash: self.password.clone(), // TO DO => HASH
            api_key: Uuid::nil(),
        }
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
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[post("/users")]
pub async fn create_user(
    req: HttpRequest,
    create_req: web::Json<CreateUserRequest>,
    store: web::Data<SharedStore>,  
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store)?;
    let _ = get_authorized_user(req, true)?;
    let create_req_data: CreateUserRequest = create_req.into_inner();
    let mut store_user = create_req_data.to_store_user();

    match store_lock.create_user(&mut store_user) {
        Ok(()) => Ok(
            HttpResponse::Created()
            .insert_header(("Location", format!("/api/v1/users/{}", encode(&store_user.id.to_string()))))
            .json(UserItem::from_store_user(&store_user))
        ),
        Err(err) => {
            match err {
                StoreError::UserEmailExists() | StoreError::UserNameExists()  => Err(get_user_exists_error()),
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
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[get("/users/{id}")]
pub async fn get_user(
    req: HttpRequest,
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
) -> Result<HttpResponse, Error> {
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
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[put("/users/{id}")]
pub async fn update_user(
    req: HttpRequest,
    path: web::Path<String>, 
    update_req: web::Json<UpdateUserRequest>,
    store: web::Data<SharedStore>,  
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store)?;
    let _ = get_authorized_user(req, false)?;
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
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[delete("/users/{id}")]
pub async fn delete_user(
    req: HttpRequest,
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
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
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[get("/mazes")]
pub async fn get_mazes(
    req: HttpRequest,
    store: web::Data<SharedStore>,
    query: Query<GetMazeListQueryParams>
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
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[post("/mazes")]
pub async fn create_maze(
    req: HttpRequest,
    req_maze: web::Json<Maze>,
    store: web::Data<SharedStore>,  
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
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[get("/mazes/{id}")]
pub async fn get_maze(
    req: HttpRequest,
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
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
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[put("/mazes/{id}")]
pub async fn update_maze(
    req: HttpRequest,
    path: web::Path<String>, 
    req_maze: web::Json<Maze>,
    store: web::Data<SharedStore>,  
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
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[delete("/mazes/{id}")]
pub async fn delete_maze(
    req: HttpRequest,
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
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
        (status = 200, description = "Maze solved successfully", body = Solution),
        (status = 401, description = "Unauthorized request"),
        (status = 404, description = "Maze not found"),
        (status = 422, description = "Maze could not be solved")    ),
    security(
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[get("/mazes/{id}/solution")]
pub async fn get_maze_solution(
    req: HttpRequest,
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
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
        (status = 200, description = "Maze solved successfully", body = Solution),
        (status = 400, description = "Invalid request"),
        (status = 401, description = "Unauthorized request"),
        (status = 422, description = "Maze could not be solved")
    ),
    security(
        ("api_key" = [])
    ),
    tags = ["v1"]
)]
#[post("/solve-maze")]
pub async fn solve_maze(
    req: HttpRequest,
    req_maze: web::Json<Maze>,  
) -> Result<HttpResponse, Error> {
    let _ = get_authorized_user(req, false)?;
    let maze: Maze = req_maze.into_inner();
    match maze.solve() {
        Ok(solution) => Ok(HttpResponse::Ok().json(solution)),
        Err(err) => Err(get_maze_solve_error(&err))
    }
}

#[cfg(test)]
mod tests {
    // **************************************************************************************************
    // Unit tests for API and documentation endpoints, via injection of MockStore
    // **************************************************************************************************
    use crate::api::v1::handlers;
    use crate::api::v1::handlers::get_maze_solve_error_string;
    use crate::middleware::auth::auth_middleware;
    use crate::api::v1::openapi::ApiDocV1;

    use maze::{Definition, Maze, MazeError, Solution, Path, Point};
    use storage::{SharedStore, Store, store::MazeStore, store::UserStore, store::Manage, StoreError, MazeItem, User};

    use actix_web::{http::StatusCode, test, dev::{Service, ServiceResponse}, web, App, middleware::from_fn, Error};
    
    use actix_http; 
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    use utoipa::OpenApi;
    use utoipa_rapidoc::RapiDoc;
    use utoipa_redoc::{Redoc, Servable};
    use utoipa_swagger_ui::SwaggerUi;
    use uuid::Uuid;
    
    #[derive(Clone, Debug)]
    struct MockMaze {
        id: String,
        name: String,
        maze: Maze,
    }

    impl MockMaze {
        pub fn to_maze_item(&self, include_definitions: bool) -> MazeItem {
            MazeItem {
                id: self.id.clone(),
                name: self.name.clone(),
                definition: if include_definitions {
                    Some(serde_json::to_string(&self.maze.definition).expect("Failed to serialize"))
                } else {
                    None
                },
            }
        }

        fn create_id_from_name(name: &str) -> String {
            format!("{}.json", name)
        }
    } 

    struct MockUser {
        user: User,
        mazes: HashMap<String, MockMaze>,
    }

    struct MockStore {
        users: HashMap<Uuid, MockUser>,
    }

    impl MockStore {
        pub fn new(startup_content: StoreStartupContent) -> Self {
            MockStore {
                users: new_users_map(startup_content)
            }
        }

        fn get_mock_user(&self, id: Uuid) -> Result<&MockUser, StoreError> {
            if let Some(mock_user) = self.users.get(&id) {
                return Ok(mock_user);
            } 
            Err(StoreError::UserIdNotFound(id.to_string()))
        }

        fn get_mock_user_mut(&mut self, id: Uuid) -> Result<&mut MockUser, StoreError> {
            if let Some(mock_user) = self.users.get_mut(&id) {
                return Ok(mock_user);
            } 
            Err(StoreError::UserIdNotFound(id.to_string()))
        }

        fn get_api_key_to_use(&self, valid: bool) -> Uuid {
            if valid {
                if let Some((_k, v)) = self.users.iter().next() {
                    return v.user.api_key;
                }                
            }    
            Uuid::new_v4()
        }

    }

    impl MazeStore for MockStore {

        fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?; 
            let id = MockMaze::create_id_from_name(&maze.name);

            if mock_user.mazes.contains_key(&id) {
                return Err(StoreError::MazeIdExists(id.to_string()));
            }

            mock_user.mazes.insert(
                id.to_string(),
                MockMaze {
                    id,
                    name: maze.name.to_string(),
                    maze: maze.clone(),
            });

            Ok(())
        }

        fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?; 
            if mock_user.mazes.remove(id).is_some() {
                Ok(())                
            } else {
                Err(StoreError::MazeIdNotFound(id.to_string()))
            }
        }

        fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?; 
            if mock_user.mazes.contains_key(&maze.id) {
                mock_user.mazes.insert(
                    maze.id.to_string(),
                    MockMaze {
                        id: maze.id.to_string(),
                        name: maze.name.to_string(),
                        maze: maze.clone(),
                });
                return Ok(());
            }
            Err(StoreError::MazeIdNotFound(maze.id.to_string()))
        }

        fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, StoreError> {
            let mock_user = self.get_mock_user(owner.id)?; 
            if let Some(mock_maze) = mock_user.mazes.get(id) {
                return Ok(mock_maze.maze.clone());
            }
            Err(StoreError::MazeIdNotFound(id.to_string()))
        }

        fn find_maze_by_name(&self, _owner: &User, _name: &str) -> Result<MazeItem, StoreError> {
            Err(StoreError::Other("Mock interface not implemented".to_string()))
        }

        fn get_maze_items(&self, owner: &User, include_definitions: bool) -> Result<Vec<MazeItem>, StoreError> {
            let mock_user = self.get_mock_user(owner.id)?; 
            let mut items: Vec<MazeItem> = maze_items_from_map(&mock_user.mazes, include_definitions);
            items.sort_by_key(|item| item.name.clone());
            Ok(items)
        }
    }

    impl UserStore for MockStore {
        /// Adds the default admin user to the store if it doesn't already exist, else returns it 
        fn init_default_admin_user(&mut self, _username: &str, _password_hash: &str) -> Result<User, StoreError> {
            Err(StoreError::Other("init_default_admin_user() not implemented for MockStore".to_string()))
        }
        /// Adds a new user to the store and sets the allocated `id` within the user object
        fn create_user(&mut self, _user: &mut User) -> Result<(), StoreError> {
            Err(StoreError::Other("create_user() not implemented for MockStore".to_string()))
        }
        /// Deletes a user from the store
        fn delete_user(&mut self, _id: Uuid) -> Result<(), StoreError> {
            Err(StoreError::Other("deletee_user() not implemented for MockStore".to_string()))
        }
        /// Updates a user within the store
        fn update_user(&mut self, _user: &mut User) -> Result<(), StoreError> {
            Err(StoreError::Other("update_user() not implemented for MockStore".to_string()))
        }
        /// Loads a user from the store
        fn get_user(&self, _id: Uuid) -> Result<User, StoreError> {
            Err(StoreError::Other("get_user() not implemented for MockStore".to_string()))
        }
        /// Locates a user by their username within the store
        fn find_user_by_name(&self, _name: &str) -> Result<User, StoreError> {
            Err(StoreError::Other("find_user_by_name() not implemented for MockStore".to_string()))
        }
        /// Locates a user by their api key within the store
        fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user.api_key == api_key {
                    return Ok(v.user.clone());
                }
            }            
            Err(StoreError::UserNotFound())
        }
        /// Returns the list of users within the store, sorted
        /// alphabetically by username in ascending order
        fn get_users(&self) -> Result<Vec<User>, StoreError> {
            Err(StoreError::Other("get_users() not implemented for MockStore".to_string()))
        }
    }

    impl Manage for MockStore {
        fn empty(&mut self) -> Result<(), StoreError> {
            self.users = HashMap::new();
            Ok(())
        }    
    }    

    impl Store for MockStore {}

    #[derive(Clone)]
    enum StoreStartupContent {
        Empty,
        OneMaze,
        TwoMazes,
        ThreeMazes,
        SolutionTestMazes,
    }

    fn maze_store_items_to_maze_items(from: Vec<MockMaze>, include_definitions: bool) -> Vec<MazeItem> {
        from.iter()
            .map( |value| value.to_maze_item(include_definitions))
            .collect()
    }

    fn mazes_to_map(mazes: &Vec<MockMaze>) -> HashMap<String, MockMaze> {
        let mut map: HashMap<String, MockMaze> = HashMap::new();
        for maze in mazes {
            map.insert(maze.id.clone(), maze.clone());
        }
        map 
    }

    fn maze_items_from_map(from: &HashMap<String, MockMaze>, include_definitions: bool) -> Vec<MazeItem> {
        from.iter()
            .map(|(_key, value) | MazeItem {
                    id: value.id.clone(),
                    name: value.name.clone(),
                    definition: if include_definitions {
                        Some(serde_json::to_string(&value.maze.definition).expect("Failed to serialize"))
                    } else {
                        None
                    },                   
            })
            .collect()
    }

    fn new_solvable_maze(id: &str, name: &str) -> Maze {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', 'W', ' ', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', 'F'],
            vec!['W', ' ', 'W', ' ', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec!['W', 'W', ' ', ' ', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
        ];
        let mut maze:Maze = Maze::new(Definition::from_vec(grid));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
    }    

    fn new_solvable_maze_store_item(id: &str, name: &str) -> MockMaze {
        MockMaze {
            id: id.to_string(),
            name: name.to_string(),
            maze: new_solvable_maze(id, name),
        }
    }

    fn new_solve_test_maze(id: &str, name: &str, with_start: bool, with_finish: bool, with_block: bool) -> Maze {
        let start_char:char = if with_start {'S'} else {' '};
        let finish_char:char = if with_finish {'F'} else {' '};
        let block_char:char = if with_block {'W'} else {' '};

        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![start_char, 'W', ' '],
            vec![' ', 'W', ' '],
            vec![' ', block_char, finish_char],
        ];
        let mut maze:Maze = Maze::new(Definition::from_vec(grid));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
    }

    fn get_solve_test_maze_solution() -> Solution {
        let path = Path {
            points: vec![
                Point { row: 0, col: 0 },
                Point { row: 1, col: 0 },
                Point { row: 2, col: 0 },
                Point { row: 2, col: 1 },
                Point { row: 2, col: 2 },
            ],
        };
        Solution::new(path)       
    } 


    fn new_solve_test_maze_store_item(id: &str, name: &str, with_start: bool, with_finish: bool, with_block: bool) -> MockMaze {
        MockMaze {
            id: id.to_string(),
            name: name.to_string(),
            maze: new_solve_test_maze(id, name, with_start, with_finish, with_block),
        }
    }

    fn get_startup_content(startup_content: StoreStartupContent, sort_asc: bool) -> Vec<MockMaze> {
        let mut result: Vec<MockMaze>;
        match startup_content {
            StoreStartupContent::Empty => {
                result = Vec::new();
            } 
            StoreStartupContent::OneMaze => {
                result = vec![
                    new_solvable_maze_store_item("maze_a.json", "maze_a")
                ]
            } 
            StoreStartupContent::TwoMazes => {
                result = vec![
                    new_solvable_maze_store_item("maze_b.json", "maze_b"),
                    new_solvable_maze_store_item("maze_a.json", "maze_a"),
                ]
            } 
            StoreStartupContent::ThreeMazes => {
                result = vec![
                    new_solvable_maze_store_item("maze_c.json", "maze_c"),
                    new_solvable_maze_store_item("maze_b.json", "maze_b"),
                    new_solvable_maze_store_item("maze_a.json", "maze_a"),
                ]
            } 
            StoreStartupContent::SolutionTestMazes => {
                result = vec![
                    new_solve_test_maze_store_item("solvable.json", "solvable", true, true, false),
                    new_solve_test_maze_store_item("no_start.json", "no_start", false, true, false),
                    new_solve_test_maze_store_item("no_finish.json", "no_finish", true, false, false),
                    new_solve_test_maze_store_item("no_solution.json", "no_solution", true, true, true),
                ]
            } 
        }
        
        if sort_asc {
            result.sort_by_key(|item| item.name.clone());
        }

        result
    }

    fn new_mazes_map(startup_content: StoreStartupContent) -> HashMap<String, MockMaze> {
        mazes_to_map(&get_startup_content(startup_content, false))
    }

    fn new_user(username: &str, is_admin: bool) -> User {
        let mut user = User::default();
        user.id = Uuid::new_v4();
        user.username = username.to_string();
        user.is_admin = is_admin;
        user
    }

    fn new_mock_user(username: &str, is_admin: bool, startup_content: StoreStartupContent) -> MockUser {
        let user =  new_user(username, is_admin);
        MockUser {
            user,
            mazes: new_mazes_map(startup_content),
        } 
    }

    fn new_shared_mock_maze_store(startup_content: StoreStartupContent) -> SharedStore {
        Arc::new(RwLock::new(Box::new(MockStore::new(startup_content))))
    }

    fn new_shared_mock_maze_store_2(mock_store: MockStore) -> SharedStore {
        Arc::new(RwLock::new(Box::new(mock_store)))
    }

    fn new_users_map(startup_content: StoreStartupContent) -> HashMap<Uuid, MockUser> {
        let mock_user = new_mock_user("user1", false, startup_content);
        let mut map: HashMap<Uuid, MockUser> = HashMap::new();
        map.insert(mock_user.user.id, mock_user);
        map
    }
    
    fn configure_mock_app(app: &mut web::ServiceConfig, mock_store: SharedStore)  {
    
        app.app_data(web::Data::new(mock_store.clone()))
            .service(
                web::scope("/api/v1")
                    .wrap(from_fn(auth_middleware))
                    .service(handlers::get_mazes)
                    .service(handlers::create_maze)
                    .service(handlers::get_maze)
                    .service(handlers::update_maze)
                    .service(handlers::delete_maze)
                    .service(handlers::get_maze_solution)
                    .service(handlers::solve_maze)
                )
                .service(SwaggerUi::new("api-docs/v1/swagger-ui/{_:.*}").url("/api-docs/v1/openapi.json", ApiDocV1::openapi()))
                .service(Redoc::with_url("/api-docs/v1/redoc", ApiDocV1::openapi()))
                .service(RapiDoc::new("/api-docs/v1/openapi.json").path("/api-docs/v1/rapidoc"));
    }

    fn create_test_get_request(url: &str, api_key: Option<Uuid>) -> actix_http::Request { 
        let mut req = test::TestRequest::get().uri(url);

        if let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }
        req.to_request()
    }

    fn create_test_post_request<T: serde::Serialize>(url: &str, api_key: Option<Uuid>, body_obj: &T) -> actix_http::Request { 
        let mut req = test::TestRequest::post().uri(url);

        if let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }
        req.set_json(body_obj).to_request()
    }

    fn create_test_put_request<T: serde::Serialize>(url: &str, api_key: Option<Uuid>, body_obj: &T) -> actix_http::Request { 
        let mut req = test::TestRequest::put().uri(url);

        if let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }
        req.set_json(body_obj).to_request()
    }

    fn create_test_delete_request(url: &str, api_key: Option<Uuid>) -> actix_http::Request { 
        let mut req = test::TestRequest::delete().uri(url);

        if let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }
        req.to_request()
    }


    fn create_shared_mock_store(startup_content: StoreStartupContent, get_valid_api_key: bool) -> (SharedStore, Uuid) {
        let mock_store = MockStore::new(startup_content.clone());
        let api_key = mock_store.get_api_key_to_use(get_valid_api_key); 
        let shared_mock_store = new_shared_mock_maze_store_2(mock_store);
        (shared_mock_store, api_key)
    }

    async fn create_test_app(
        startup_content: &StoreStartupContent,
        get_valid_api_key:bool                 
    ) -> (impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>, Uuid) {
        let (shared_mock_store, api_key) = create_shared_mock_store(startup_content.clone(), get_valid_api_key);
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, shared_mock_store)),
        )
        .await;

        (app, api_key)
    }

    async fn run_get_mazes_test(use_valid_api_key:bool, startup_content: StoreStartupContent, include_definitions: bool) {
        let (app, api_key) = create_test_app(&startup_content, use_valid_api_key).await;
        let path_str = format!("/api/v1/mazes?includeDefinitions={}", include_definitions);
        let req = create_test_get_request(&path_str, Some(api_key));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let maze_items: Vec<MazeItem> = serde_json::from_slice(&body).expect("failed to deserialize response");
        assert_eq!(
            maze_items,
            maze_store_items_to_maze_items(get_startup_content(startup_content, true), include_definitions)
        );        
    }

    async fn run_create_maze_test(
        startup_content: StoreStartupContent, 
        maze: Maze,
        expected_status_code: StatusCode, 
        ) {
        let (app, api_key) = create_test_app(&startup_content, true).await;
        let url = "/api/v1/mazes".to_string();
        let req = create_test_post_request(&url, Some(api_key), &maze);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            let mut maze_copy = maze.clone();
            maze_copy.id = MockMaze::create_id_from_name(&maze.name);
            assert_eq!(maze_copy, response_maze);        
        }
    }

    async fn run_get_maze_test(
            startup_content: StoreStartupContent, 
            id: &str, 
            expected_status_code: StatusCode, 
            expected_maze: Option<Maze>
        ) {
        let (app, api_key) = create_test_app(&startup_content, true).await;
        let url = format!("/api/v1/mazes/{}", id);
        let req = create_test_get_request(&url, Some(api_key));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Verify content
            let body = test::read_body(resp).await;
            let maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            match expected_maze {
                Some(value) => { assert_eq!(maze, value); }        
                None => { panic!("No maze comparison value provided for get_maze() test!"); }
            }
        }
    }

    async fn run_update_maze_test(
            startup_content: StoreStartupContent, 
            id: &str, 
            maze: Maze,
            expected_status_code: StatusCode, 
        ) {
        let (app, api_key) = create_test_app(&startup_content, true).await;
        let url = format!("/api/v1/mazes/{}", id);
        let req = create_test_put_request(&url, Some(api_key), &maze);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            assert_eq!(maze, response_maze);        
        }
    }

    async fn run_delete_maze_test(
            startup_content: StoreStartupContent, 
            id: &str, 
            expected_status_code: StatusCode 
        ) {
        let (app, api_key) = create_test_app(&startup_content, true).await;
        let url = format!("/api/v1/mazes/{}", id);
        let req = create_test_delete_request(&url, Some(api_key));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Confirm it has been deleted
            let url2 = format!("/api/v1/mazes/{}", id);
            let req2 = create_test_get_request(&url2, Some(api_key));
            let resp2 = test::call_service(&app, req2).await;
            assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
        }
    }

    async fn validate_solution_response(
        context: &str,
        resp: actix_web::dev::ServiceResponse,
        expected_status_code: StatusCode,
        expected_solution: Option<Solution>,
        expected_err_message: Option<String>
    ) {
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Confirm and validate solution response
            let body = test::read_body(resp).await;
            let solution: Solution = serde_json::from_slice(&body).expect("failed to deserialize response");
             match expected_solution {
                Some(value) => { assert_eq!(solution, value);}        
                None => { panic!("{}", format!("No maze solution comparison value provided for {} test!", context)); }
            }
        }
        else {
            match expected_err_message {
                Some(value) => { 
                    // Validate error response
                    let body = test::read_body(resp).await;
                    let error_message = String::from_utf8(body.to_vec()).expect("Failed to parse body as UTF-8");
                    assert_eq!(error_message, value);
                }        
                None => { panic!("{}", format!("No maze solution error message provided for {} test!", context)); }
            }
        }
    }

    fn get_no_start_cell_error_str() -> String {
        get_maze_solve_error_string(&MazeError::new("no start cell found within maze".to_string()))
    }

    fn get_no_finish_cell_error_str() -> String {
        get_maze_solve_error_string(&MazeError::new("no finish cell found within maze".to_string()))
    }

    fn get_no_solution_error_str() -> String {
        get_maze_solve_error_string(&MazeError::new("no solution found".to_string()))    
    }

    async fn run_get_maze_solution_test(
        startup_content: StoreStartupContent, 
        id: &str, 
        expected_status_code: StatusCode,
        expected_solution: Option<Solution>,
        expected_err_message: Option<String>
        ) {
        let (app, api_key) = create_test_app(&startup_content, true).await;
        let url = format!("/api/v1/mazes/{}/solution", id);
        let req = create_test_get_request(&url, Some(api_key));
        let resp = test::call_service(&app, req).await;

        validate_solution_response("get_maze_solution()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_solve_maze_test(
        maze: Maze,
        expected_status_code: StatusCode,
        expected_solution: Option<Solution>,
        expected_err_message: Option<String>
        ) {
        let (app, api_key) = create_test_app(&StoreStartupContent::Empty, true).await;
        let url = "/api/v1/solve-maze".to_string();
        let req = create_test_post_request(&url, Some(api_key), &maze);
        let resp = test::call_service(&app, req).await;

        validate_solution_response("solve_maze()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_get_url_test(
        url: &str
        ) {

        let mock_store = new_shared_mock_maze_store(StoreStartupContent::Empty);

        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, mock_store)),
        )
        .await;

        let req = create_test_get_request(url, None);
        let resp = test::call_service(&app, req).await;
    
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn test_get_mazes_with_no_mazes_and_invalid_api_key() {
        run_get_mazes_test(false, StoreStartupContent::Empty, false).await;
    }
 
    #[actix_web::test]
    async fn test_get_mazes_with_no_mazes() {
        run_get_mazes_test(true, StoreStartupContent::Empty, false).await;
    }

    #[actix_web::test]
    async fn test_get_mazes_with_one_maze_without_definitions() {
        run_get_mazes_test(true, StoreStartupContent::OneMaze, false).await;
    }

    #[actix_web::test]
    async fn test_get_mazes_with_one_maze_with_defintions() {
        run_get_mazes_test(true, StoreStartupContent::OneMaze, true).await;
    }
 
    #[actix_web::test]
    async fn test_get_mazes_with_two_mazes_that_require_sorting_without_definitions() {
        run_get_mazes_test(true, StoreStartupContent::TwoMazes, false).await;
    }

    #[actix_web::test]
    async fn test_get_mazes_with_two_mazes_that_require_sorting_with_definitions() {
        run_get_mazes_test(true, StoreStartupContent::TwoMazes, true).await;
    }

    #[actix_web::test]
    async fn test_get_mazes_with_three_mazes_that_require_sorting_without_definitions() {
        run_get_mazes_test(true, StoreStartupContent::ThreeMazes, false).await;
    }

    #[actix_web::test]
    async fn test_get_mazes_with_three_mazes_that_require_sorting_with_definitions() {
        run_get_mazes_test(true, StoreStartupContent::ThreeMazes, true).await;
    }

    #[actix_web::test]
    async fn test_create_maze_that_does_not_exist() {
        run_create_maze_test(StoreStartupContent::ThreeMazes, new_solvable_maze("", "maze_d"), StatusCode::CREATED).await;
    }

    #[actix_web::test]
    async fn test_create_maze_that_already_exists() {
        run_create_maze_test(StoreStartupContent::ThreeMazes, new_solvable_maze("", "maze_a"), StatusCode::CONFLICT).await;
    }

    #[actix_web::test]
    async fn test_get_maze_that_exists() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_get_maze_test(StoreStartupContent::ThreeMazes, id, StatusCode::OK, Some(new_solvable_maze(id, name))).await;
    }

    #[actix_web::test]
    async fn test_update_maze_that_exists() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(StoreStartupContent::ThreeMazes, id, new_solvable_maze(id, name), StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn test_update_maze_that_does_not_exist() {
        let id = "maze_d.json";
        let name = "maze_d";
        run_update_maze_test(StoreStartupContent::ThreeMazes, id, new_solvable_maze(id, name), StatusCode::NOT_FOUND).await;
    }

    #[actix_web::test]
    async fn test_update_maze_with_mismatching_id() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(StoreStartupContent::ThreeMazes, id, new_solvable_maze("some_other_id", name), StatusCode::BAD_REQUEST).await;
    }

    #[actix_web::test]
    async fn test_get_maze_that_does_not_exist() {
        run_get_maze_test(StoreStartupContent::ThreeMazes, "does_not_exist.json", StatusCode::NOT_FOUND, None).await;
    }

    #[actix_web::test]
    async fn test_delete_maze_that_exists() {
        run_delete_maze_test(StoreStartupContent::ThreeMazes, "maze_a.json", StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn test_delete_maze_that_does_not_exist() {
        run_delete_maze_test(StoreStartupContent::ThreeMazes, "does_not_exist.json", StatusCode::NOT_FOUND).await;
    }

    #[actix_web::test]
    async fn test_get_maze_solution_that_should_succeed() {
        run_get_maze_solution_test(
            StoreStartupContent::SolutionTestMazes, "solvable.json", StatusCode::OK, 
            Some(get_solve_test_maze_solution()), None
        ).await;
    }

    #[actix_web::test]
    async fn test_get_maze_solution_should_fail_with_no_start() {
        run_get_maze_solution_test(
            StoreStartupContent::SolutionTestMazes, "no_start.json", StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_start_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_get_maze_solution_should_fail_with_no_finish() {
        run_get_maze_solution_test(
            StoreStartupContent::SolutionTestMazes, "no_finish.json", StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_get_maze_solution_should_fail_with_no_solution() {
        run_get_maze_solution_test(
            StoreStartupContent::SolutionTestMazes, "no_solution.json", StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_solution_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_solve_maze_that_should_succeed() {
        run_solve_maze_test(
            new_solve_test_maze("", "", true, true, false), 
            StatusCode::OK, 
            Some(get_solve_test_maze_solution()), 
            None
        ).await;
    }

    #[actix_web::test]
    async fn test_solve_maze_should_fail_with_no_start() {
        run_solve_maze_test(
            new_solve_test_maze("", "", false, true, false), 
            StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_start_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_solve_maze_should_fail_with_no_finish() {
        run_solve_maze_test(
            new_solve_test_maze("", "", true, false, false), 
            StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_solve_maze_should_fail_with_no_solution() {
        run_solve_maze_test(
            new_solve_test_maze("", "", true, true, true), 
            StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_solution_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_load_swagger_ui_page() {
        run_get_url_test("/api-docs/v1/swagger-ui/").await;
    }

    #[actix_web::test]
    async fn test_load_openapi_json() {
        run_get_url_test("/api-docs/v1/openapi.json").await;
    }

    #[actix_web::test]
    async fn test_load_redoc_page() {
        run_get_url_test("/api-docs/v1/redoc").await;
    }

    #[actix_web::test]
    async fn test_load_rapidoc_page() {
        run_get_url_test("/api-docs/v1/rapidoc").await;
    }
}
