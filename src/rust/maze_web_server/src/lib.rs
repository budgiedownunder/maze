pub mod api;
pub mod config;
pub mod middleware;

use storage::{get_store, SharedStore};

use actix_web::{ App, middleware::Logger, HttpServer, web};
use config::app::AppConfig;
use rustls::{ServerConfig, Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::sync::Arc;
use std::sync::RwLock;
use std::{fs::File, io::{self, BufReader}};

/// Loads the rust_ls configuration for the server session (see: https://docs.rs/rustls/latest/rustls/server/struct.ServerConfig.html)
fn load_rustls_config(config: &AppConfig) -> io::Result<ServerConfig> {

    let cert_file_name = &config.security.cert_file;
    let key_file_name =  &config.security.key_file;

    let mut cert_reader = BufReader::new(File::open(cert_file_name).unwrap_or_else(|_| panic!("Cannot open private key file '{}'", key_file_name)));
    let key_reader = &mut BufReader::new(File::open(key_file_name).unwrap_or_else(|_| panic!("Cannot open private key file '{}'", key_file_name)));

    let cert_chain = certs(&mut cert_reader)?
        .into_iter()
        .map(Certificate)
        .collect::<Vec<_>>();

    if cert_chain.is_empty() {
        panic!("{}", format!("No certificates found in '{}'! Ensure it's PKCS#8 format.", cert_file_name));
    }
    
    let mut keys = pkcs8_private_keys(key_reader)?;

    if keys.is_empty() {
        panic!("{}", format!("No private keys found in '{}'! Ensure it's PKCS#8 format.", key_file_name));
    }

    let key = PrivateKey(keys.remove(0));

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("TLS setup error: {:?}", e)))?;

    Ok(config)
}

/// Constructs the server bind address on which the server will listen for requests
fn construct_bind_address(port: u16) -> String {
    format!("0.0.0.0:{}", port)
}

/// Runs the Maze Web Server, which hosts the Maze Web API. This uses [`actix`](https://actix.rs/) to serve the API and 
/// [`utoipa`](https://docs.rs/utoipa/latest/utoipa/) to publish it as an `OpenAPI`-compliant interface
/// for use in third party products such as `Swagger`. In addition, the server also publishes its own 
/// Swagger-related endpoints that can be used to manually test the API in user-friendly web pages (e.g. `/api-docs/v1/swagger-ui/`). 
pub async fn run_server() -> std::io::Result<()> {
    let config = AppConfig::load().expect("Failed to load configuration");
  
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let bind_address = construct_bind_address(config.port);

    let rustls_config = load_rustls_config(&config)?;
    let max_workers = std::thread::available_parallelism()?; // This is actix_web's default too
    let file_config = storage::FileStoreConfig::default();
    let mut store = get_store(storage::StoreConfig::File(file_config))?;
    let users = store.get_users()?;
    if users.is_empty() {
        store.init_default_admin_user("admin", "dummy_hash_password")?;
    }
    let shared_store: SharedStore = Arc::new(RwLock::new(store));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(shared_store.clone()))  // Share the store
            .app_data(web::Data::new(config.clone())) // Share the config
            .wrap(Logger::default())
            .service(api::register_api())
            .service(api::register_redoc())
            .service(api::register_rapidoc())
            .service(api::register_swagger_ui())
    })
    .bind_rustls(bind_address, rustls_config)?
    .workers(usize::from(max_workers))
    .run()
    .await
}
  