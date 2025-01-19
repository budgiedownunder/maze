use actix_web::{
    body::MessageBody,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{Next},
    Error, web,
    error::ErrorUnauthorized
};

use crate::config::AppConfig;

use log::error; // Import logging

///  Authorization middleware
pub async fn auth_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {

    // pre-processing
    if let Some(config) = req.app_data::<web::Data<AppConfig>>() {
        if let Some(cookie) = req.cookie("AuthToken") {
            if cookie.value() == "" {
                return Err(reject_unauthorized(&req, "Missing auth token"));
            }

            if cookie.value() != config.security.auth_token {
                return Err(reject_unauthorized(&req, "Bad auth token"));
            }
            
        } else {
            return Err(reject_unauthorized(&req, "Missing auth token"));
        }
    }

    // Make req est
    next.call(req).await

    // post-processing
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