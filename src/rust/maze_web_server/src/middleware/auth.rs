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
use log::error; // Import logging
use uuid::Uuid;

fn get_store_read_lock(
    store: &Arc<RwLock<Box<dyn Store>>>,
) -> Result<RwLockReadGuard<'_, Box<dyn Store>>, Error> {
    store.read().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire store read lock")
    })
}

///  Authorization middleware
pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let store = req
        .app_data::<web::Data<SharedStore>>() 
        .expect("Store is missing from app_data")
        .clone();

    if let Some(api_key) = req.headers().get("X-API-KEY") {
        if let Ok(api_key_str) = api_key.to_str() {
            if let Ok(api_key) = Uuid::parse_str(api_key_str) {
                if let Some(user) = {
                    let store_lock = get_store_read_lock(&store)?;
                    store_lock.find_user_by_api_key(api_key).ok()
                } {
                    req.extensions_mut().insert(user);
                    return next.call(req).await;
                }
            }
        }
    }
    // if let Some(cookie) = req.cookie("session_id") {
    //     if cookie.value() == "" {
    //         return Err(reject_unauthorized(&req, "Missing auth token"));
    //     }

    //     if cookie.value() != config.security.auth_token {
    //         return Err(reject_unauthorized(&req, "Bad auth token"));
    //     }
        
    // } else {
    //     return Err(reject_unauthorized(&req, "Missing auth token"));
    // }

    // post-processing
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