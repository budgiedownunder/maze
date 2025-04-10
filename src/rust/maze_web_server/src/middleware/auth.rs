use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{Next},
    Error, error::ErrorUnauthorized,
    HttpMessage,
    web,
};
use std::sync::{RwLockReadGuard, RwLock, Arc};
use storage::{SharedStore, Store};
use log::error;
use uuid::Uuid;

fn get_store_read_lock(
    store: &Arc<RwLock<Box<dyn Store>>>,
) -> Result<RwLockReadGuard<'_, Box<dyn Store>>, Error> {
    store.read().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire store read lock")
    })
}

/// Wrapper type used to store the login token ID in Actix request extensions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct LoginId(pub uuid::Uuid);

/// Wrapper type used to store the API key  in Actix request extensions.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ApiKey(pub uuid::Uuid);

///  Authorization middleware
pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody + 'static>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let store = req
        .app_data::<web::Data<SharedStore>>() 
        .expect("Store is missing from app_data")
        .clone();

     if let Some(auth_header) = req.headers().get("Authorization") {
        let raw = auth_header.to_str().map_err(|_| reject_unauthorized(&req, "Invalid header"))?;
        if let Some(token_str) = raw.strip_prefix("Bearer ") {
            let login_id = Uuid::parse_str(token_str).map_err(|_| reject_unauthorized(&req, "Invalid token format"))?;
            if let Some(user) = {
                let store_lock = get_store_read_lock(&store)?;
                store_lock.find_user_by_login_id(login_id).ok()
            } {
                req.extensions_mut().insert(LoginId(login_id)); 
                req.extensions_mut().insert(user);
                return next.call(req).await;
            }
        }        
    }

    if let Some(api_key) = req.headers().get("X-API-KEY") {
        if let Ok(api_key_str) = api_key.to_str() {
            if let Ok(api_key) = Uuid::parse_str(api_key_str) {
                if let Some(user) = {
                    let store_lock = get_store_read_lock(&store)?;
                    store_lock.find_user_by_api_key(api_key).ok()
                } {
                    req.extensions_mut().insert(ApiKey(api_key)); 
                    req.extensions_mut().insert(user);
                    return next.call(req).await;
                }
            }
        }
    }

    // If neither succeeded, return explicit error
    Err(reject_unauthorized(&req, "Unauthorized request"))
}

///  Generates an unauthorized request response
fn reject_unauthorized(req: &ServiceRequest, reason: &str) -> Error {
    error!(
        "Rejected request: {}. Path: {}, IP: {:?}",
        reason,
        req.path(),
        req.peer_addr()
    );

    ErrorUnauthorized(reason.to_string())
}