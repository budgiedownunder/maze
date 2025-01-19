use actix_web::web; 
use actix_web::middleware::from_fn;
use crate::api::v1::handlers;
use crate::middleware::auth::auth_middleware;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("")
            .wrap(from_fn(auth_middleware))
            .service(handlers::get_maze_list)
            .service(handlers::create_maze)
            .service(handlers::get_maze)
            .service(handlers::update_maze)
            .service(handlers::delete_maze)
            .service(handlers::get_maze_solution)
            .service(handlers::solve_maze)
    );
}
