pub mod api;
pub mod config;
pub mod middleware;
pub mod service;
mod utils;

use actix_files::{Files, NamedFile};
use actix_web::{ App, HttpRequest, middleware::Logger, HttpServer, web};
use auth::{config::PasswordHashConfig, hashing::hash_password};
use config::app::{AppConfig, AppFeaturesConfig};
use rustls::{ServerConfig, Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use service::auth::AuthService;
use std::sync::Arc;
use std::sync::RwLock;
use std::{fs::File, io::{self, BufReader}};
use storage::{get_store, SharedStore, Store, Error as StoreError};

pub type SharedFeatures = Arc<RwLock<AppFeaturesConfig>>;

const DEFAULT_ADMIN_ACCOUNT_USERNAME:&str = "admin";
const DEFAULT_ADMIN_ACCOUNT_PASSWORD:&str = "Admin1!";

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

/// Adds the default admin account to the store if no users are registered
fn init_user_accounts(hash_config: &PasswordHashConfig, store: &mut Box<dyn Store>) -> Result<(), StoreError> {
    let users = store.get_users()?;
    if users.is_empty() {
        let password_hash = match hash_password(DEFAULT_ADMIN_ACCOUNT_PASSWORD, hash_config) {
            Ok(hash) => hash,
            Err(error) => return Err(StoreError::Other(format!("{}", error))),
        };
        store.init_default_admin_user(DEFAULT_ADMIN_ACCOUNT_USERNAME, &password_hash)?;
    }
    Ok(())    
}

/// Creates a configured Actix App instance with all routes, middleware, and shared state.
/// Can be reused in both production (`main.rs`) and tests.
pub fn create_app(
    hash_config: &PasswordHashConfig,
    store: web::Data<SharedStore>,
    shared_features: web::Data<SharedFeatures>,
    static_dir: String,
) -> App<impl actix_service::ServiceFactory<
    actix_web::dev::ServiceRequest,
    Config = (),
    Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
    Error = actix_web::Error,
    InitError = (),
>> {
    let auth_service = web::Data::new(AuthService::new(hash_config.clone()));

    App::new()
        .app_data(auth_service)
        .app_data(store)
        .app_data(shared_features)
        .service(api::register_api())
        .service(api::register_redoc())
        .service(api::register_rapidoc())
        .service(api::register_swagger_ui())
        .configure(move |cfg| {
            if std::path::Path::new(&static_dir).is_dir() {
                let index_path = format!("{}/index.html", static_dir);

                // Register known SPA routes explicitly.
                // This prevents actix-files from attempting to decode the URL path as
                // a filesystem path — which rejects segments containing characters like
                // '\' before the default_handler can fire.
                let spa = {
                    let path = index_path.clone();
                    move |req: HttpRequest| {
                        let path = path.clone();
                        async move {
                            NamedFile::open_async(&path)
                                .await
                                .map(|f| f.into_response(&req))
                        }
                    }
                };
                cfg.route("/login",        web::get().to(spa.clone()))
                   .route("/signup",       web::get().to(spa.clone()))
                   .route("/mazes",        web::get().to(spa.clone()))
                   .route("/mazes/{tail}", web::get().to(spa.clone()))
                   .service(
                       Files::new("/", &static_dir)
                           .index_file("index.html")
                           .default_handler({
                               web::get().to(move |req: HttpRequest| {
                                   let path = index_path.clone();
                                   async move {
                                       NamedFile::open_async(&path)
                                           .await
                                           .map(|f| f.into_response(&req))
                                   }
                               })
                           }),
                   );
            } else {
                log::info!("static_dir '{}' does not exist — running as API-only", static_dir);
            }
        })
}


/// Runs the Maze Web Server, which hosts the Maze Web API. This uses [`actix`](https://actix.rs/) to serve the API and 
/// [`utoipa`](https://docs.rs/utoipa/latest/utoipa/) to publish it as an `OpenAPI`-compliant interface
/// for use in third party products such as `Swagger`. In addition, the server also publishes its own 
/// Swagger-related endpoints that can be used to manually test the API in user-friendly web pages (e.g. `/api-docs/v1/swagger-ui/`). 
pub async fn run_server() -> std::io::Result<()> {
    let config = AppConfig::load().expect("Failed to load configuration settings");
    utils::logger::init(&config.logging.log_dir, &config.logging.log_level, &config.logging.log_file_prefix)
        .expect("Failed to initialise logger");
    config.log_config();
  
    let bind_address = construct_bind_address(config.port);
    let rustls_config = load_rustls_config(&config)?;
    let file_config = storage::FileStoreConfig::default();
    let mut store = get_store(storage::StoreConfig::File(file_config))?;

    init_user_accounts(&config.security.password_hash, &mut store)?;

    let max_workers = std::thread::available_parallelism()?;
    let shared_store: SharedStore = Arc::new(RwLock::new(store));
    let shared_features: SharedFeatures = Arc::new(RwLock::new(config.features.clone()));

    HttpServer::new(move || {
        create_app(&config.security.password_hash, web::Data::new(shared_store.clone()), web::Data::new(shared_features.clone()), config.static_dir.clone())
        .app_data(web::Data::new(config.clone()))
        .wrap(Logger::default())
    })
    .bind_rustls(bind_address, rustls_config)?
    .workers(usize::from(max_workers))
    .run()
    .await
}
  