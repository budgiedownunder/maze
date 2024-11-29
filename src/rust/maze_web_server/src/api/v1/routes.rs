use actix_web::web;
use crate::api::v1::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::get_maze_list)
       .service(handlers::create_maze)
       .service(handlers::get_maze)
       .service(handlers::update_maze)
       .service(handlers::delete_maze)
       .service(handlers::get_maze_solution)
       .service(handlers::solve_maze);
}
