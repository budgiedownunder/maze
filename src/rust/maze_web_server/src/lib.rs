mod api;

use storage::SharedStore;
use storage::get_store;

use actix_web::{ App, middleware::Logger, HttpServer, web};
use std::sync::Arc;
use std::sync::RwLock;

/// Runs the Maze Web Server, which hosts the Maze Web API. This uses [`actix`](https://actix.rs/) to serve the API and 
/// [`utoipa`](https://docs.rs/utoipa/latest/utoipa/) to publish it as an `OpenAPI`-compliant interface
/// for use in third party products such as `Swagger`. In addition, the server also publishes its own 
/// Swagger-related endpoints that can be used to manually test the API in user-friendly web pages (e.g. `/api-docs/v1/swagger-ui/`). 
pub async fn run_server() -> std::io::Result<()> {
    // TO DO - make these  environment/config settings with defaults
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let address = "127.0.0.1:8080";
    let max_workers = std::thread::available_parallelism()?; // This is actix_web's default too

    let store: SharedStore = Arc::new(RwLock::new(get_store(storage::StoreType::File)?));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(store.clone())) // Share the store
            .wrap(Logger::default())
            .service(api::register_api())
            .service(api::register_redoc())
            .service(api::register_rapidoc())
            .service(api::register_swagger_ui())
    })
    .bind(address)?
    .workers(usize::from(max_workers))
    .run()
    .await
}
  