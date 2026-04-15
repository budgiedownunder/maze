use actix_web::web; 
use actix_web::middleware::from_fn;
use crate::api::v1::endpoints::handlers;
use crate::middleware::auth::auth_middleware;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg
        // Unguarded routes
        .service(handlers::get_features)
        .service(handlers::login)
        .service(handlers::signup)
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
                // Users (self-service) - must come BEFORE /users/{id}
                .service(handlers::change_password_me)
                .service(handlers::update_profile_me)
                .service(handlers::get_me)
                .service(handlers::delete_me)
                .service(handlers::logout)
                .service(handlers::renew)
                // Users (admin)
                .service(handlers::get_users)
                .service(handlers::create_user)
                .service(handlers::delete_user)
                .service(handlers::get_user)
                .service(handlers::update_user)
        );
}
