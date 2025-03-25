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
    /// Generate a new Uuid
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
    /// Generates a new user id
    ///
    /// # Returns
    ///
    /// User id
    ///
    /// # Examples
    ///
    /// Initialize a user with a new user id and api key and then print it
    /// ```
    /// use data_model::User;
    /// let user = User {
    ///     id: User::new_id(),
    ///     is_admin: false,
    ///     username: "john_smith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "john_smith@company.com".to_string(),
    ///     password_hash: "encrypted_hash".to_string(),
    ///     api_key: User::new_api_key(),
    /// };
    /// println!("User: {:?}", user);
    pub fn new_id() -> Uuid {
        Self::generate_uuid()
    }
    /// Generates a new API key
    ///
    /// # Returns
    ///
    /// User id
    ///
    /// # Examples
    ///
    /// Initialize a user with a new user id and api key and then print it
    /// ```
    /// use data_model::User;
    /// let user = User {
    ///     id: User::new_id(),
    ///     is_admin: false,
    ///     username: "john_smith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "john_smith@company.com".to_string(),
    ///     password_hash: "encrypted_hash".to_string(),
    ///     api_key: User::new_api_key(),
    /// };
    /// println!("User: {:?}", user);
    pub fn new_api_key() -> Uuid {
        Self::generate_uuid()
    }
    /// Creates a new user with default content
    ///
    /// # Returns
    ///
    /// User instance
    ///
    /// # Examples
    ///
    /// Initialize a default user and then print it
    /// ```
    /// use data_model::User;
    /// let user = User::default();
    /// println!("User: {:?}", user);
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
    /// Generates the JSON string representation for the user
    ///
    /// # Returns
    ///
    /// JSON string representing the user
    ///
    ///
    /// # Examples
    ///
    /// Initialize a user, convert it to JSON and print it
    /// ```
    /// use data_model::User;
    /// let user = User {
    ///     id: User::new_id(),
    ///     is_admin: false,
    ///     username: "john_smith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "john_smith@company.com".to_string(),
    ///     password_hash: "encrypted_hash".to_string(),
    ///     api_key: User::new_api_key(),
    /// };
    /// match user.to_json() {
    ///     Ok(json) => {
    ///         println!("JSON: {}", json);
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to convert user to JSON => {}",
    ///           error
    ///        );
    ///     }
    /// }
    pub fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }

    /// Initializes a user instance by reading the JSON string content provided
    /// 
    pub fn from_json(&mut self, json: &str) -> Result<(), Error> {
        let temp: User = serde_json::from_str(json)?;
        *self = temp;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_default() {
        let _ = User::default();
    }

    #[test]
    fn can_serialize() {
        let user = User::default();
        let s = user.to_json().expect("Failed to serialize");
        assert_eq!(s, r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","email":"","password_hash":"","api_key":"00000000-0000-0000-0000-000000000000"}"#);
    }

    #[test]
    fn can_deserialize() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","email":"","password_hash":"","api_key":"00000000-0000-0000-0000-000000000000"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    fn can_create_user_id() {
        let id = User::new_id();
        assert_ne!(id, Uuid::nil());
    }

    #[test]
    fn can_create_api_key() {
        let key = User::new_api_key();
        assert_ne!(key, Uuid::nil());
    }
}