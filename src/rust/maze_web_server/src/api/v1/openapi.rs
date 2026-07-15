use data_model::{CollectionItem, GameCollection, GameDefinition, GranteeSummary, Maze, MazeDefinition, UserEmail};
use maze::{GenerationAlgorithm, GeneratorOptions, MazePath, MazeSolution};
use storage::MazeItem;
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi,
};

use crate::api::v1::endpoints::auth_reset::{PasswordResetConfirmRequest, PasswordResetRequest};
use crate::api::v1::endpoints::email_verification::{
    EmailVerificationConfirmRequest, EmailVerificationRequest,
};
use crate::api::v1::endpoints::handlers::{
    AppFeaturesResponse, Play3dConfigResponse,
    LoginRequest, LoginResponse, RenewResponse,
    SignupRequest, UserItem, CreateUserRequest, UpdateUserRequest,
    ChangePasswordRequest, UpdateProfileRequest,
    UserLookupEntry, UserLookupResponse, UsersListResponse};
use crate::api::v1::endpoints::avatar::AvatarUpdatedResponse;
use crate::api::v1::endpoints::game_collections::{
    GameCollectionSharesResponse, GameCollectionDetailResponse,
    GameCollectionListResponse, GameCollectionRequest, SetGameCollectionItemsRequest,
};
use crate::api::v1::endpoints::game_definitions::{
    GameDefinitionSharesResponse, GameDefinitionListResponse, GameDefinitionRequest, GamePlayResponse,
};
use crate::api::v1::endpoints::featured_game_items::{
    FeaturedGameItemEntry, FeaturedGameItemResponse, FeaturedGameItemsListResponse,
    ReorderFeaturedGameItemsRequest,
};
use crate::api::v1::endpoints::game_shared::{ImageUpdatedResponse, SetGameSharesRequest};
use crate::api::v1::endpoints::scores::{CompletedChallengesRequest, CompletedChallengesResponse, RecordScoreRequest, ResetScoresResponse, ScoreboardResponse, ScoreResponse};
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
        // Game presets
        crate::api::v1::endpoints::handlers::get_play3d_config,
        // Login, logout, renew, signup
        crate::api::v1::endpoints::handlers::login,
        crate::api::v1::endpoints::handlers::logout,
        crate::api::v1::endpoints::handlers::renew,
        crate::api::v1::endpoints::handlers::signup,
        // Password reset
        crate::api::v1::endpoints::auth_reset::request_password_reset,
        crate::api::v1::endpoints::auth_reset::confirm_password_reset,
        // Email verification
        crate::api::v1::endpoints::email_verification::request_email_verification,
        crate::api::v1::endpoints::email_verification::confirm_email_verification,
        // OAuth sign-in
        crate::api::v1::endpoints::handlers::oauth_start,
        crate::api::v1::endpoints::handlers::oauth_callback,
        // Self-service account
        crate::api::v1::endpoints::handlers::change_password_me,
        crate::api::v1::endpoints::handlers::update_profile_me,
        crate::api::v1::endpoints::handlers::get_me,
        crate::api::v1::endpoints::handlers::delete_me,
        // Self-service avatar
        crate::api::v1::endpoints::avatar::upload_avatar,
        crate::api::v1::endpoints::avatar::delete_avatar,
        crate::api::v1::endpoints::avatar::get_avatar,
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
        // Game definitions
        crate::api::v1::endpoints::game_definitions::create_game_definition,
        crate::api::v1::endpoints::game_definitions::list_game_definitions,
        crate::api::v1::endpoints::game_definitions::get_game_definition,
        crate::api::v1::endpoints::game_definitions::update_game_definition,
        crate::api::v1::endpoints::game_definitions::reshuffle_game_definition,
        crate::api::v1::endpoints::game_definitions::delete_game_definition,
        crate::api::v1::endpoints::game_definitions::list_game_definition_shares,
        crate::api::v1::endpoints::game_definitions::set_game_definition_shares,
        crate::api::v1::endpoints::game_definitions::upload_game_definition_image,
        crate::api::v1::endpoints::game_definitions::delete_game_definition_image,
        crate::api::v1::endpoints::game_definitions::serve_game_definition_image,
        // Game collections
        crate::api::v1::endpoints::game_collections::create_game_collection,
        crate::api::v1::endpoints::game_collections::list_game_collections,
        crate::api::v1::endpoints::game_collections::get_game_collection,
        crate::api::v1::endpoints::game_collections::update_game_collection,
        crate::api::v1::endpoints::game_collections::delete_game_collection,
        crate::api::v1::endpoints::game_collections::set_game_collection_items,
        crate::api::v1::endpoints::game_collections::list_game_collection_shares,
        crate::api::v1::endpoints::game_collections::set_game_collection_shares,
        crate::api::v1::endpoints::game_collections::upload_game_collection_image,
        crate::api::v1::endpoints::game_collections::delete_game_collection_image,
        crate::api::v1::endpoints::game_collections::serve_game_collection_image,
        // Featured catalogue
        crate::api::v1::endpoints::featured_game_items::get_featured_game_items,
        crate::api::v1::endpoints::featured_game_items::set_featured_game_items_order,
        // Scores
        crate::api::v1::endpoints::scores::record_score,
        crate::api::v1::endpoints::scores::get_leaderboard,
        crate::api::v1::endpoints::scores::get_my_history,
        crate::api::v1::endpoints::scores::get_my_completed_challenges,
        crate::api::v1::endpoints::scores::reset_leaderboard,
        // User lookup (share people-picker)
        crate::api::v1::endpoints::handlers::lookup_users,
        // Users (admin)
        crate::api::v1::endpoints::handlers::get_users,
        crate::api::v1::endpoints::handlers::create_user,
        crate::api::v1::endpoints::handlers::get_user,
        crate::api::v1::endpoints::handlers::update_user,
        crate::api::v1::endpoints::handlers::delete_user,

    ),
    components(
        schemas(
            AppFeaturesResponse, OAuthProviderPublic, Play3dConfigResponse,
            LoginRequest, LoginResponse, RenewResponse,
            SignupRequest, CreateUserRequest, UpdateUserRequest, UserItem,
            UserLookupEntry, UserLookupResponse, UsersListResponse,
            ChangePasswordRequest, UpdateProfileRequest,
            PasswordResetRequest, PasswordResetConfirmRequest,
            EmailVerificationRequest, EmailVerificationConfirmRequest,
            UserEmail, UserEmailsResponse, AddUserEmailRequest,
            Maze, MazeDefinition, MazeItem, MazePath, MazeSolution,
            GeneratorOptions, GenerationAlgorithm,
            CompletedChallengesRequest, CompletedChallengesResponse, RecordScoreRequest, ResetScoresResponse, ScoreResponse, ScoreboardResponse,
            GameDefinition, GameDefinitionRequest, GameDefinitionListResponse, GamePlayResponse,
            SetGameSharesRequest, GameDefinitionSharesResponse, GranteeSummary,
            GameCollection, CollectionItem, GameCollectionRequest, GameCollectionListResponse,
            GameCollectionDetailResponse, SetGameCollectionItemsRequest,
            GameCollectionSharesResponse, ImageUpdatedResponse,
            FeaturedGameItemResponse, FeaturedGameItemsListResponse,
            FeaturedGameItemEntry, ReorderFeaturedGameItemsRequest,
            AvatarUpdatedResponse),

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
