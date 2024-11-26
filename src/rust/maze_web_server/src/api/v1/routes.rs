use actix_web::web;
use crate::api::v1::handlers;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(handlers::get_maze_list);
}
