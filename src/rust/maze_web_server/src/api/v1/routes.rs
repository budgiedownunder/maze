use actix_web::web;
use actix_web::middleware::from_fn;
use crate::api::v1::endpoints::{auth_reset, email_verification, handlers, scores, user_emails};
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
                // Scores
                .service(scores::record_score)
                // Users (self-service) - must come BEFORE /users/{id}
                .service(handlers::change_password_me)
                .service(handlers::update_profile_me)
                .service(handlers::get_me)
                .service(handlers::delete_me)
                .service(handlers::logout)
                .service(handlers::renew)
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
