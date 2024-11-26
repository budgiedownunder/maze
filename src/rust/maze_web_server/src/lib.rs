mod api;

use actix_web::{ App, middleware::Logger, HttpServer};

pub async fn run_server() -> std::io::Result<()> {
    // TO DO - make these  environment/config settings with defaults
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let address = "127.0.0.1:8080";
    let max_workers = std::thread::available_parallelism()?; // This is actix_web's default too

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .service(api::register_api())
            .service(api::register_swagger_ui())
    })
    .bind(address)?
    .workers(usize::from(max_workers))
    .run()
    .await
}
  