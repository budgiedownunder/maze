pub mod v1;

use actix_web::web;
use lazy_static::lazy_static;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};

// **************************************************************************************************
// API support - endpoint registration
// **************************************************************************************************
pub fn register_api() -> actix_web::Scope {
    web::scope("api")
        .service(
            web::scope("v1")
                .configure(v1::routes::configure)
            )
}

// **************************************************************************************************
// Swagger UI support - endpoint registration
// Note: we use lazy static here so that SWAGGER_UI_V1 is only initialized once (as opposed to 
// per worker)
// **************************************************************************************************
lazy_static! {
    static ref SWAGGER_UI_V1: SwaggerUi = {
        let openapi_v1 = v1::openapi::ApiDocV1::openapi();
        SwaggerUi::new("api-docs/v1/swagger-ui/{_:.*}").url("/api-docs/v1/openapi.json", openapi_v1)
    };
}

pub fn register_swagger_ui() -> SwaggerUi {
    SWAGGER_UI_V1.clone()
}

// **************************************************************************************************
// ReDoc support - endpoint registration
// Note: we use lazy static here so that REDOC_V1 is only initialized once (as opposed to per worker)
// **************************************************************************************************
lazy_static! {
    static ref REDOC_V1: Redoc<utoipa::openapi::OpenApi>  = {
        let openapi_v1 = v1::openapi::ApiDocV1::openapi();
        Redoc::with_url("/api-docs/v1/redoc", openapi_v1.clone())
    };
}

pub fn register_redoc() -> Redoc<utoipa::openapi::OpenApi> {
    REDOC_V1.clone()
}

// **************************************************************************************************
// RapiDoc support - endpoint registration
// **************************************************************************************************
pub fn register_rapidoc() -> RapiDoc {
    RapiDoc::new("/api-docs/v1/openapi.json").path("/api-docs/v1/rapidoc")
}



