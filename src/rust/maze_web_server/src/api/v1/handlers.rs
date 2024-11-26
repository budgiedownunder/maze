use storage::MazeItem;
use storage::SharedStore;
use actix_web::{get, web, HttpResponse, Responder};

#[utoipa::path(
    get,
    path = "/api/v1/mazes",
    responses(
        (status = 200, description = "Maze definitions", body=[MazeItem])
    ),
    tags = ["v1"] // Explicitly set the tag name
)]
#[get("/mazes")]
pub async fn get_maze_list(store: web::Data<SharedStore>) -> impl Responder {
    let store_lock = match store.read() {
        Ok(lock) => lock,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to acquire store lock");
        }
    };

    match store_lock.get_maze_items() {
        Ok(stored_items) => HttpResponse::Ok().json(stored_items),
        Err(err) => {
            eprintln!("Error fetching maze items: {}", err);
            HttpResponse::InternalServerError().body(format!("Error: {}", err))
        }
    }
}
