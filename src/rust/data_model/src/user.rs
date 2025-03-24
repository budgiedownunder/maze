use crate::Error;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a user of the system 
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct User {
    #[schema(value_type = String)] // Treat as string during serlialization
    /// User ID
    pub id: Uuid,
    /// Is administrator?
    pub is_admin: bool,
    /// Username
    pub username: String,
    /// Full name 
    pub full_name: String,
    /// Email address
    pub email: String,
    /// Password hash (encrypted)
    pub password_hash: String,
    #[schema(value_type = String)] // Treat as string during serlialization
    /// API key
    pub api_key: Uuid,
}

impl User {

    /// Generates a new Uuid
    fn generate_uuid() -> uuid::Uuid {
        #[cfg(not(feature = "uuid-disable-random"))]
        {
            uuid::Uuid::new_v4()
        }
    
        #[cfg(feature = "uuid-disable-random")]
        {
            uuid::Uuid::nil()
        }
    }
    /// Creates a new user id
    pub fn new_id() -> Uuid {
        Self::generate_uuid()
    }

    /// Creates a new API key
    pub fn new_api_key() -> Uuid {
        Self::generate_uuid()
    }

    /// Returns a User instance initialized with the defautl values
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> User {
        User {
            id: Uuid::nil(),
            is_admin: false,
            username: "".to_string(),
            full_name: "".to_string(),
            email: "".to_string(),
            password_hash: "".to_string(),
            api_key: Uuid::nil(),
        }
    }

    /// Converts the instance to a JSON string
    pub fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }
}