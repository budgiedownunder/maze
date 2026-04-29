pub mod api;
pub mod config;
pub mod middleware;
pub mod oauth;
pub mod service;
mod utils;

use actix_files::{Files, NamedFile};
use actix_service::Service;
use actix_web::{ App, HttpRequest, middleware::Logger, HttpServer, web};
use actix_web::http::header::{CACHE_CONTROL, HeaderName, HeaderValue};
use auth::{config::PasswordHashConfig, hashing::hash_password};
use config::app::{AppConfig, AppFeaturesConfig, SqlStorageConfig, StorageConfig, StorageKind};
use rustls::{ServerConfig, Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};
use service::auth::AuthService;
use std::sync::Arc;
use std::sync::RwLock;
use tokio::sync::RwLock as AsyncRwLock;
use std::{fs::File, io::{self, BufReader}};
use storage::{get_store, SharedStore, Store, Error as StoreError};

pub type SharedFeatures = Arc<RwLock<AppFeaturesConfig>>;

const DEFAULT_ADMIN_ACCOUNT_USERNAME: &str = "admin";
const DEFAULT_ADMIN_ACCOUNT_EMAIL: &str = "admin@maze.local";
const DEFAULT_ADMIN_ACCOUNT_PASSWORD: &str = "Admin1!";

/// Loads the rust_ls configuration for the server session (see: https://docs.rs/rustls/latest/rustls/server/struct.ServerConfig.html)
fn load_rustls_config(config: &AppConfig) -> io::Result<ServerConfig> {

    let cert_file_name = &config.security.cert_file;
    let key_file_name =  &config.security.key_file;

    let mut cert_reader = BufReader::new(File::open(cert_file_name).unwrap_or_else(|_| panic!("Cannot open private key file '{key_file_name}'")));
    let key_reader = &mut BufReader::new(File::open(key_file_name).unwrap_or_else(|_| panic!("Cannot open private key file '{key_file_name}'")));

    let cert_chain = certs(&mut cert_reader)?
        .into_iter()
        .map(Certificate)
        .collect::<Vec<_>>();

    if cert_chain.is_empty() {
        panic!("{}", format!("No certificates found in '{cert_file_name}'! Ensure it's PKCS#8 format."));
    }
    
    let mut keys = pkcs8_private_keys(key_reader)?;

    if keys.is_empty() {
        panic!("{}", format!("No private keys found in '{key_file_name}'! Ensure it's PKCS#8 format."));
    }

    let key = PrivateKey(keys.remove(0));

    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("TLS setup error: {e:?}")))?;

    Ok(config)
}

/// Constructs the server bind address on which the server will listen for requests
fn construct_bind_address(port: u16) -> String {
    format!("0.0.0.0:{port}")
}

/// Adds the default admin account to the store if no users are registered
async fn init_user_accounts(hash_config: &PasswordHashConfig, store: &mut Box<dyn Store>) -> Result<(), StoreError> {
    if !store.has_users().await? {
        let password_hash = match hash_password(DEFAULT_ADMIN_ACCOUNT_PASSWORD, hash_config) {
            Ok(hash) => hash,
            Err(error) => return Err(StoreError::Other(format!("{error}"))),
        };
        store.init_default_admin_user(DEFAULT_ADMIN_ACCOUNT_USERNAME, DEFAULT_ADMIN_ACCOUNT_EMAIL, &password_hash).await?;
    }
    Ok(())
}

/// Creates a configured Actix App instance with all routes, middleware, and shared state.
/// Can be reused in both production (`main.rs`) and tests.
pub fn create_app(
    hash_config: &PasswordHashConfig,
    store: web::Data<SharedStore>,
    features: web::Data<SharedFeatures>,
    oauth_connector: web::Data<oauth::SharedOAuthConnector>,
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
        .app_data(features)
        .app_data(oauth_connector)
        .service(api::register_api())
        .service(api::register_redoc())
        .service(api::register_rapidoc())
        .service(api::register_swagger_ui())
        .wrap_fn(|req, srv| {
            let path = req.path().to_owned();
            let fut = srv.call(req);
            async move {
                let mut res = fut.await?;
                // /assets/   = Vite content-hashed filenames         → immutable
                // /game/*.wasm = fetched via versioned WASM_URL (?v=N)  → immutable
                //   Increment ?v=N in index.html after each wasm-pack rebuild to bust this cache.
                // /game/*.js = wasm-bindgen wrapper, fixed filename     → no-cache
                //   Always fresh so new exports are picked up without a forced cache clear.
                if path.starts_with("/assets/")
                    || (path.starts_with("/game/") && path.ends_with(".wasm"))
                {
                    res.headers_mut().insert(
                        CACHE_CONTROL,
                        HeaderValue::from_static("public, max-age=31536000, immutable"),
                    );
                } else if path.starts_with("/game/") && path.ends_with(".js") {
                    res.headers_mut().insert(
                        CACHE_CONTROL,
                        HeaderValue::from_static("no-cache"),
                    );
                }
                // Cross-origin isolation for /game/ — enables WebAssembly.Module storage in IndexedDB
                if path == "/game/" {
                    res.headers_mut().insert(
                        CACHE_CONTROL,
                        HeaderValue::from_static("no-cache"),
                    );
                    res.headers_mut().insert(
                        HeaderName::from_static("cross-origin-opener-policy"),
                        HeaderValue::from_static("same-origin"),
                    );
                    res.headers_mut().insert(
                        HeaderName::from_static("cross-origin-embedder-policy"),
                        HeaderValue::from_static("require-corp"),
                    );
                }
                Ok(res)
            }
        })
        .configure(move |cfg| {
            if std::path::Path::new(&static_dir).is_dir() {
                let index_path = format!("{static_dir}/index.html");

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
                let game_path = format!("{static_dir}/game/index.html");
                let game = {
                    let path = game_path.clone();
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
                   .route("/play",         web::get().to(spa.clone()))
                   .route("/play/{tail}",  web::get().to(spa.clone()))
                   .route("/game",         web::get().to(|| async {
                       actix_web::HttpResponse::PermanentRedirect()
                           .insert_header(("Location", "/game/"))
                           .finish()
                   }))
                   .route("/game/",        web::get().to(game.clone()))
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
                log::info!("static_dir '{static_dir}' does not exist — running as API-only");
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
    let store_config = build_store_config(&config.storage)
        .map_err(std::io::Error::other)?;
    let mut store = get_store(store_config).await?;

    init_user_accounts(&config.security.password_hash, &mut store).await?;

    let max_workers = std::thread::available_parallelism()?;
    let shared_store: SharedStore = Arc::new(AsyncRwLock::new(store));
    let features: SharedFeatures = Arc::new(RwLock::new(config.features.clone()));
    let oauth_connector: oauth::SharedOAuthConnector = build_oauth_connector(&config)?;

    HttpServer::new(move || {
        create_app(&config.security.password_hash, web::Data::new(shared_store.clone()), web::Data::new(features.clone()), web::Data::new(oauth_connector.clone()), config.static_dir.clone())
        .app_data(web::Data::new(config.clone()))
        .wrap(Logger::default())
    })
    .bind_rustls(bind_address, rustls_config)?
    .workers(usize::from(max_workers))
    .run()
    .await
}

/// Translates the user-facing [`StorageConfig`] into the lower-level
/// [`storage::StoreConfig`] understood by [`get_store`].
///
/// The SQL branch assembles a SQLx connection URL from discrete config fields
/// and appends driver-specific TLS / connect-timeout query parameters. The
/// password is taken from the in-memory `SqlStorageConfig.password` (which
/// `StorageConfig::resolve_password_from_env` will already have populated from
/// the `MAZE_WEB_SERVER_STORAGE_SQL_PASSWORD` env var).
fn build_store_config(storage: &StorageConfig) -> Result<storage::StoreConfig, String> {
    match storage.kind {
        StorageKind::File => Ok(storage::StoreConfig::File(storage::FileStoreConfig {
            data_dir: storage.file.data_dir.clone(),
        })),
        StorageKind::Sql => {
            let url = assemble_sql_url(&storage.sql)?;
            Ok(storage::StoreConfig::Sql(storage::SqlStoreConfig {
                url,
                max_connections: storage.sql.max_connections,
                auto_create_database: storage.sql.auto_create_database,
                idle_timeout_secs: storage.sql.idle_timeout_secs,
                acquire_timeout_secs: storage.sql.acquire_timeout_secs,
            }))
        }
    }
}

/// Assembles a SQLx connection URL from a [`SqlStorageConfig`].
///
/// `connect_timeout_secs` and `require_tls` are encoded as driver-specific
/// query parameters because `AnyPoolOptions` does not expose them
/// generically. SQLite ignores both (no network).
fn assemble_sql_url(sql: &SqlStorageConfig) -> Result<String, String> {
    let driver = sql.driver.to_ascii_lowercase();
    match driver.as_str() {
        "sqlite" => {
            if sql.path.trim().is_empty() {
                return Err("[storage.sql] driver = \"sqlite\" requires a non-empty path".into());
            }
            Ok(format!("sqlite:{}", sql.path))
        }
        "postgres" | "postgresql" | "mysql" => {
            if sql.host.trim().is_empty() {
                return Err(format!("[storage.sql] driver = \"{driver}\" requires a non-empty host"));
            }
            if sql.database.trim().is_empty() {
                return Err(format!("[storage.sql] driver = \"{driver}\" requires a non-empty database"));
            }
            if sql.username.trim().is_empty() {
                return Err(format!("[storage.sql] driver = \"{driver}\" requires a non-empty username"));
            }
            let scheme = if driver == "mysql" { "mysql" } else { "postgres" };
            let user = urlencoding::encode(&sql.username);
            let pass = urlencoding::encode(&sql.password);
            let auth = if sql.password.is_empty() {
                user.into_owned()
            } else {
                format!("{user}:{pass}")
            };
            let host_port = if sql.port == 0 {
                sql.host.clone()
            } else {
                format!("{}:{}", sql.host, sql.port)
            };
            let mut url = format!("{scheme}://{auth}@{host_port}/{}", sql.database);
            let mut params: Vec<(&'static str, String)> = Vec::new();
            if sql.require_tls {
                if driver == "mysql" {
                    let mode = if sql.ca_cert_path.is_empty() { "REQUIRED" } else { "VERIFY_CA" };
                    params.push(("ssl-mode", mode.into()));
                    if !sql.ca_cert_path.is_empty() {
                        params.push(("ssl-ca", sql.ca_cert_path.clone()));
                    }
                } else {
                    let mode = if sql.ca_cert_path.is_empty() { "require" } else { "verify-full" };
                    params.push(("sslmode", mode.into()));
                    if !sql.ca_cert_path.is_empty() {
                        params.push(("sslrootcert", sql.ca_cert_path.clone()));
                    }
                }
            }
            // `connect_timeout_secs` deliberately *not* appended to the URL.
            // sqlx-postgres / sqlx-mysql don't recognise it as a URL parameter
            // (they warn and ignore). It's still accepted as a config field
            // for parity with the deployment-topology examples; whether to
            // bind it onto pool options later is a follow-up.
            let _ = sql.connect_timeout_secs;
            if !params.is_empty() {
                url.push('?');
                let parts: Vec<String> = params
                    .into_iter()
                    .map(|(k, v)| format!("{}={}", k, urlencoding::encode(&v)))
                    .collect();
                url.push_str(&parts.join("&"));
            }
            Ok(url)
        }
        other => Err(format!(
            "[storage.sql] unknown driver \"{other}\" — expected \"sqlite\", \"postgres\", or \"mysql\""
        )),
    }
}

/// Build the [`oauth::SharedOAuthConnector`] selected by `config.oauth`.
/// Returns a [`oauth::NoOpConnector`] when OAuth is disabled. The Auth0
/// arm is intentionally `unimplemented` until a future drop adds the
/// connector — config-load already errors with a clear message in that case.
fn build_oauth_connector(config: &AppConfig) -> std::io::Result<oauth::SharedOAuthConnector> {
    if !config.oauth.enabled {
        return Ok(Arc::new(oauth::NoOpConnector));
    }
    match config.oauth.connector {
        config::ConnectorKind::Internal => {
            let connector = oauth::internal::InternalOAuthConnector::from_config(&config.oauth)
                .map_err(|e| std::io::Error::other(format!("oauth connector init: {e}")))?;
            Ok(Arc::new(connector))
        }
        config::ConnectorKind::Auth0 => Err(std::io::Error::other(
            "oauth connector = \"auth0\" is not yet implemented",
        )),
    }
}
  