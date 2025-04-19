use chrono::Utc;

#[cfg(feature = "wasm-lite")]
use chrono::TimeZone;

/// Wrapper function for generating a new Uuid. If the "wasm-lite" feature is enabled, then will return nil - otherwise a random Uuid is generated and returned.
pub fn generate_uuid() -> uuid::Uuid {
    #[cfg(not(feature = "wasm-lite"))]
    {
        uuid::Uuid::new_v4()
    }

    #[cfg(feature = "wasm-lite")]
    {
        uuid::Uuid::nil()
    }
}
/// Wrapper function for generating the current timestamp. If the "wasm-lite" feature is enabled, then will return nil - otherwise the current timestamp is returned.
pub fn generate_now() -> chrono::DateTime<Utc> {
    #[cfg(not(feature = "wasm-lite"))]
    {
        Utc::now()
    }

    #[cfg(feature = "wasm-lite")]
    {
        Utc.timestamp_opt(0, 0).unwrap()
    }
}    
