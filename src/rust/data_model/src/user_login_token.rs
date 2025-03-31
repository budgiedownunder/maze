use crate::{Error, wrappers::generate_uuid};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a user login token 
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct UserLoginToken {
    #[schema(value_type = String)] // Treat as string during serlialization
    /// Token ID
    pub id: Uuid,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Expiry timestamp
    pub expires_at: DateTime<Utc>,
    /// Device information where login occurred
    pub device_info: Option<String>,
    /// IP address where login occurred
    pub ip_address: Option<String>,
}

impl Default for UserLoginToken {
    fn default() -> Self {
        Self::new(0, None, None)
    }
}

impl UserLoginToken {
    /// Creates a new user login token with a given expiry, ip address (optional) and device info (optional)
    ///
    /// # Returns
    ///
    /// User login token
    ///
    /// # Examples
    ///
    /// Initialize a user login token, convert it to JSON and print it
    /// ```
    /// use data_model::UserLoginToken;
    /// let login_token = UserLoginToken::new(24, Some("123.456.789.012".to_string()), Some("Device info string".to_string()));
    /// match login_token.to_json() {
    ///     Ok(json) => {
    ///         println!("JSON: {}", json);
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to convert login token to JSON => {}",
    ///           error
    ///        );
    ///     }
    /// }
    pub fn new(expiry_hours: i64, ip_address: Option<String>, device_info: Option<String>) -> UserLoginToken {
        let now = Utc::now();
        UserLoginToken {
            id: generate_uuid(),
            created_at: now,
            expires_at: now + Duration::hours(expiry_hours),
            ip_address,
            device_info,
        }
    }
    /// Generates the JSON string representation for the user login token
    ///
    /// # Returns
    ///
    /// JSON string representing the user login token
    ///
    /// # Examples
    ///
    /// Initialize a user login token, convert it to JSON and print it
    /// ```
    /// use data_model::UserLoginToken;
    /// let login_token = UserLoginToken::new(24, Some("123.456.789.012".to_string()), Some("Device info string".to_string()));
    /// match login_token.to_json() {
    ///     Ok(json) => {
    ///         println!("JSON: {}", json);
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to convert login token to JSON => {}",
    ///           error
    ///        );
    ///     }
    /// }
    pub fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }
    /// Initializes a user login token instance by reading the JSON string content provided
    /// 
    /// # Returns
    ///
    /// This function will return an error if the JSON could not be read
    ///
    /// # Examples
    ///
    /// Create a default user login token and then reinitialize it from a JSON string definition
    /// ```
    /// use data_model::UserLoginToken;
    /// let mut login_token = UserLoginToken::default();
    /// let json = r#"{"id":"89a94185-e27f-4ce5-8841-f74da4962ed6","created_at":"2025-03-31T07:48:22.742367800Z","expires_at":"2025-04-01T07:48:22.742367800Z","device_info":"Some device information","ip_address":"123.456.789.123"}"#;
    /// match login_token.from_json(&json) {
    ///     Ok(()) => {
    ///         println!(
    ///             "JSON successfully read user login token  => username = {:?}",
    ///             login_token
    ///         );
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to read JSON into login token => {}",
    ///             error
    ///        );
    ///     }
    /// }
    pub fn from_json(&mut self, json: &str) -> Result<(), Error> {
        let temp: UserLoginToken = serde_json::from_str(json)?;
        *self = temp;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create() {
        let _ = UserLoginToken::new(24, Some("123.456.789.123".to_string()), Some("Some device information".to_string()));
    }

    #[test]
    fn can_serialize_and_deserialize() {
        let token_created = UserLoginToken::new(24, Some("123.456.789.123".to_string()), Some("Some device information".to_string()));
        let json = token_created.to_json().expect("Failed to serialize");
        let mut token_loaded  = UserLoginToken::new(0, Some("".to_string()), Some("".to_string()));
        if let Err(err) = token_loaded.from_json(&json) {
            panic!("failed to deserialize: {}", err);
        }
        assert_eq!(token_created, token_loaded);
    }

}        