use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a user login token 
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct UserLoginToken {
    #[schema(value_type = String)] // Treat as string during serlialization
    /// Token ID
    pub token: Uuid,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Expiry timestamp
    expires_at: DateTime<Utc>,
    /// Device information where login occurred
    device_info: Option<String>,
    /// IP address where login occurred
    ip_address: Option<String>,
}