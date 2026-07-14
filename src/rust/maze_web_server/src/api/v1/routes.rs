use actix_web::web;
use actix_web::middleware::from_fn;
use crate::api::v1::endpoints::{auth_reset, avatar, email_verification, featured_game_items, game_collections, game_definitions, handlers, scores, user_emails};
use crate::middleware::auth::auth_middleware;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Unguarded routes
        .service(handlers::get_features)
        .service(handlers::get_play3d_config)
        .service(handlers::login)
        .service(handlers::signup)
        // Password reset (unguarded — the secret reset-token id is the credential)
        .service(auth_reset::request_password_reset)
        .service(auth_reset::confirm_password_reset)
        // Email verification — confirm is unguarded (the secret token is the credential).
        // The request endpoint is guarded so a caller can only ask to re-send for an
        // address attached to their own account; registered inside the guarded scope below.
        .service(email_verification::confirm_email_verification)
        // OAuth (unguarded — the cookie + state nonce are the CSRF protection)
        .service(handlers::oauth_start)
        .service(handlers::oauth_callback)
        // Guarded routes
        .service(
            web::scope("")
                .wrap(from_fn(auth_middleware))
                // Mazes
                .service(handlers::get_mazes)
                .service(handlers::create_maze)
                .service(handlers::delete_maze)
                .service(handlers::get_maze)
                .service(handlers::get_maze_solution)
                .service(handlers::solve_maze)
                .service(handlers::generate_maze)
                .service(handlers::update_maze)
                // Game definitions (stored 3D games)
                .service(game_definitions::create_game_definition)
                .service(game_definitions::list_game_definitions)
                .service(game_definitions::list_game_definition_shares)
                .service(game_definitions::set_game_definition_shares)
                .service(game_definitions::upload_game_definition_image)
                .service(game_definitions::delete_game_definition_image)
                .service(game_definitions::serve_game_definition_image)
                .service(game_definitions::get_game_definition)
                .service(game_definitions::update_game_definition)
                .service(game_definitions::reshuffle_game_definition)
                .service(game_definitions::delete_game_definition)
                // Game collections (ordered groupings of definitions)
                .service(game_collections::create_game_collection)
                .service(game_collections::list_game_collections)
                .service(game_collections::set_game_collection_items)
                .service(game_collections::list_game_collection_shares)
                .service(game_collections::set_game_collection_shares)
                .service(game_collections::upload_game_collection_image)
                .service(game_collections::delete_game_collection_image)
                .service(game_collections::serve_game_collection_image)
                .service(game_collections::get_game_collection)
                .service(game_collections::update_game_collection)
                .service(game_collections::delete_game_collection)
                // Featured catalogue (admin-ordered curated defs + collections)
                .service(featured_game_items::get_featured_game_items)
                .service(featured_game_items::set_featured_game_items_order)
                // Scores
                .service(scores::record_score)
                .service(scores::get_my_history)
                .service(scores::get_leaderboard)
                .service(scores::reset_leaderboard)
                // Users (self-service) - must come BEFORE /users/{id}
                // Username-prefix lookup for the share people-picker; the literal
                // /users/lookup path must be registered before /users/{id}.
                .service(handlers::lookup_users)
                .service(handlers::change_password_me)
                .service(handlers::update_profile_me)
                .service(handlers::get_me)
                .service(handlers::delete_me)
                .service(handlers::logout)
                .service(handlers::renew)
                // Users (self-service: avatar). The GET is guarded like the rest
                // of the API but readable for ANY user id (not just the caller),
                // so a signed-in viewer sees other players' avatars on boards.
                .service(avatar::upload_avatar)
                .service(avatar::delete_avatar)
                .service(avatar::get_avatar)
                // Users (self-service: email management)
                .service(user_emails::list_emails)
                .service(user_emails::add_email)
                .service(user_emails::delete_email)
                .service(user_emails::set_primary_email)
                .service(user_emails::verify_email_stub)
                // Email verification request — guarded so the caller is authenticated
                .service(email_verification::request_email_verification)
                // Features
                .service(handlers::update_admin_features)
                // Users (admin)
                .service(handlers::get_users)
                .service(handlers::create_user)
                .service(handlers::delete_user)
                .service(handlers::get_user)
                .service(handlers::update_user)
        );
}
