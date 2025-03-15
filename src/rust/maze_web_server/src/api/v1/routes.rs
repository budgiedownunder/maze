use actix_web::web; 
use actix_web::middleware::from_fn;
use crate::api::v1::handlers;
use crate::middleware::auth::auth_middleware;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(from_fn(auth_middleware))
            // Mazes
            .service(handlers::get_mazes)
            .service(handlers::create_maze)
            .service(handlers::delete_maze)
            .service(handlers::get_maze)
            .service(handlers::get_maze_solution)
            .service(handlers::solve_maze)
            .service(handlers::update_maze)
            // Users
            .service(handlers::get_users)
            .service(handlers::create_user)
            .service(handlers::delete_user)
            .service(handlers::get_user)
            .service(handlers::update_user)
        );
}
