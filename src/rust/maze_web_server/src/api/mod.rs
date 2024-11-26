pub mod v1;

use actix_web::web;
use lazy_static::lazy_static;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// Lazy static so Swagger json is only initialized once (as opposed to per worker)
lazy_static! {
    static ref SWAGGER_UI: SwaggerUi = {
        let openapi_v1 = v1::openapi::ApiDocV1::openapi();
        SwaggerUi::new("api-docs/v1/swagger-ui/{_:.*}").url("/api-docs/v1/openapi.json", openapi_v1)
    };
}

pub fn register_api() -> actix_web::Scope {
    web::scope("api")
        .service(
            web::scope("v1")
                .configure(v1::routes::configure)
            )
}

pub fn register_swagger_ui() -> SwaggerUi {
    SWAGGER_UI.clone()
}
