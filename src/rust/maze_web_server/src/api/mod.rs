//! OpenAPI support
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
// OpenAPI support - openapi JSON
// **************************************************************************************************
pub fn get_openapi_v1() -> utoipa::openapi::OpenApi {
    v1::openapi::ApiDocV1::openapi()
}

// **************************************************************************************************
// Swagger UI support - endpoint registration
// Note: we use lazy static here so that SWAGGER_UI_V1 is only initialized once (as opposed to 
// per worker)
// **************************************************************************************************
lazy_static! {
    static ref SWAGGER_UI_V1: SwaggerUi = {
        SwaggerUi::new("api-docs/v1/swagger-ui/{_:.*}").url("/api-docs/v1/openapi.json", get_openapi_v1())
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
        Redoc::with_url("/api-docs/v1/redoc", get_openapi_v1().clone())
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

#[cfg(test)]
mod tests {
    use super::get_openapi_v1;

    // The lenient wire enums document their allowed lowercase values as an
    // enumerated `string` schema (not an opaque `string`), and the model fields
    // reference those named components rather than inlining a bare string.
    #[test]
    fn lenient_enums_expose_their_wire_values_in_openapi() {
        let doc = serde_json::to_value(get_openapi_v1()).expect("serialize openapi");
        let schemas = &doc["components"]["schemas"];

        for (name, values) in [
            ("Visibility", vec!["private", "shared", "public", "curated"]),
            ("Rotation", vec!["static", "daily"]),
            ("PlayMode", vec!["arcade", "campaign"]),
        ] {
            let schema = &schemas[name];
            assert_eq!(schema["type"], "string", "{name} should be a string schema");
            assert_eq!(schema["enum"], serde_json::json!(values), "{name} should enumerate its wire values");
        }

        // A model field points at the named component (possibly wrapped in an
        // `allOf` to carry the field description) — never an inline string.
        let visibility = schemas["GameDefinition"]["properties"]["visibility"].to_string();
        assert!(visibility.contains("#/components/schemas/Visibility"), "visibility should ref the component, got {visibility}");
        let play_mode = schemas["GameCollection"]["properties"]["playMode"].to_string();
        assert!(play_mode.contains("#/components/schemas/PlayMode"), "playMode should ref the component, got {play_mode}");
    }
}



