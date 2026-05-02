use data_model::{Maze, MazeDefinition, UserEmail};
use maze::{GenerationAlgorithm, GeneratorOptions, MazePath, MazeSolution};
use storage::MazeItem;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::api::v1::endpoints::handlers::{
    AppFeaturesResponse,
    LoginRequest, LoginResponse, RenewResponse,
    SignupRequest, UserItem, CreateUserRequest, UpdateUserRequest,
    ChangePasswordRequest, UpdateProfileRequest};
use crate::api::v1::endpoints::user_emails::{AddUserEmailRequest, UserEmailsResponse};
use crate::oauth::OAuthProviderPublic;

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
        // Features
        crate::api::v1::endpoints::handlers::get_features,
        crate::api::v1::endpoints::handlers::update_admin_features,
        // Login, logout, renew, signup
        crate::api::v1::endpoints::handlers::login,
        crate::api::v1::endpoints::handlers::logout,
        crate::api::v1::endpoints::handlers::renew,
        crate::api::v1::endpoints::handlers::signup,
        // OAuth sign-in
        crate::api::v1::endpoints::handlers::oauth_start,
        crate::api::v1::endpoints::handlers::oauth_callback,
        // Self-service account
        crate::api::v1::endpoints::handlers::change_password_me,
        crate::api::v1::endpoints::handlers::update_profile_me,
        crate::api::v1::endpoints::handlers::get_me,
        crate::api::v1::endpoints::handlers::delete_me,
        // Self-service email management
        crate::api::v1::endpoints::user_emails::list_emails,
        crate::api::v1::endpoints::user_emails::add_email,
        crate::api::v1::endpoints::user_emails::delete_email,
        crate::api::v1::endpoints::user_emails::set_primary_email,
        crate::api::v1::endpoints::user_emails::verify_email_stub,
        // Mazes
        crate::api::v1::endpoints::handlers::get_mazes,
        crate::api::v1::endpoints::handlers::create_maze,
        crate::api::v1::endpoints::handlers::get_maze,
        crate::api::v1::endpoints::handlers::update_maze,
        crate::api::v1::endpoints::handlers::delete_maze,
        crate::api::v1::endpoints::handlers::get_maze_solution,
        crate::api::v1::endpoints::handlers::generate_maze,
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
            AppFeaturesResponse, OAuthProviderPublic,
            LoginRequest, LoginResponse, RenewResponse,
            SignupRequest, CreateUserRequest, UpdateUserRequest, UserItem,
            ChangePasswordRequest, UpdateProfileRequest,
            UserEmail, UserEmailsResponse, AddUserEmailRequest,
            Maze, MazeDefinition, MazeItem, MazePath, MazeSolution,
            GeneratorOptions, GenerationAlgorithm),

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
