use data_model::{Maze, MazeDefinition};
use maze::{MazePath, MazeSolution};
use storage::MazeItem;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::api::v1::endpoints::handlers::{
    LoginRequest, LoginResponse,
    SignupRequest, UserItem, CreateUserRequest, UpdateUserRequest};

struct ApiKeyAuth;

impl Modify for ApiKeyAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("X-API-Key"))),
            );
        }
    }
}

struct LoginTokenAuth;

impl utoipa::Modify for LoginTokenAuth {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "login_token",
                utoipa::openapi::security::SecurityScheme::ApiKey(
                    utoipa::openapi::security::ApiKey::Header(
                        utoipa::openapi::security::ApiKeyValue::with_description(
                            "Authorization",
                            "Bearer <login_token_id>",
                        ),
                    ),
                ),
            );
        }
    }
}


#[derive(OpenApi)]
#[openapi(
    info (
        title="Maze REST Web API",
        version = "1.0.0",
        description = "RESTful Web API for managing and solving mazes",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    paths(
        // Login, logout, signup
        crate::api::v1::endpoints::handlers::login,
        crate::api::v1::endpoints::handlers::logout,
        crate::api::v1::endpoints::handlers::signup,
        // Self-service account
        crate::api::v1::endpoints::handlers::get_me,
        crate::api::v1::endpoints::handlers::delete_me,
        // Mazes
        crate::api::v1::endpoints::handlers::get_mazes,
        crate::api::v1::endpoints::handlers::create_maze,
        crate::api::v1::endpoints::handlers::get_maze,
        crate::api::v1::endpoints::handlers::update_maze,
        crate::api::v1::endpoints::handlers::delete_maze,
        crate::api::v1::endpoints::handlers::get_maze_solution,
        crate::api::v1::endpoints::handlers::solve_maze,
        // Users (admin)
        crate::api::v1::endpoints::handlers::get_users,
        crate::api::v1::endpoints::handlers::create_user,
        crate::api::v1::endpoints::handlers::get_user,
        crate::api::v1::endpoints::handlers::update_user,
        crate::api::v1::endpoints::handlers::delete_user,

    ),
    components(
        schemas(
            LoginRequest, LoginResponse,
            SignupRequest, CreateUserRequest, UpdateUserRequest, UserItem,
            Maze, MazeDefinition, MazeItem, MazePath, MazeSolution),

    ),
    servers(
        (url = "https://localhost:8443", description = "Local development server")
    ),
    tags(
        (name = "Maze Web API v1", description = "Version 1 of the Maze Web API")
    ),
    modifiers(&ApiKeyAuth, &LoginTokenAuth)
)]
pub struct ApiDocV1;
