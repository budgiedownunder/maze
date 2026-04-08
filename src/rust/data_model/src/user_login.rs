use crate::{Error, wrappers::{generate_now, generate_uuid}};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a user login
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct UserLogin {
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

impl Default for UserLogin {
    fn default() -> Self {
        Self::new(0, None, None)
    }
}

impl UserLogin {
    /// Creates a new user login with a given expiry, ip address (optional) and device info (optional)
    ///
    /// # Returns
    ///
    /// User login
    ///
    /// # Examples
    ///
    /// Initialize a user login, convert it to JSON and print it
    /// ```
    /// use data_model::UserLogin;
    /// let login = UserLogin::new(24, Some("123.456.789.012".to_string()), Some("Device info string".to_string()));
    /// match login.to_json() {
    ///     Ok(json) => {
    ///         println!("JSON: {}", json);
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to convert login to JSON => {}",
    ///           error
    ///        );
    ///     }
    /// }
    pub fn new(expiry_hours: u32, ip_address: Option<String>, device_info: Option<String>) -> UserLogin {
        let now = generate_now();
        UserLogin {
            id: generate_uuid(),
            created_at: now,
            expires_at: now + Duration::hours(expiry_hours.into()),
            ip_address,
            device_info,
        }
    }

    /// Extends the expiry of this login by setting expires_at to the current time plus the given number of hours
    ///
    /// # Returns
    ///
    /// Nothing
    ///
    /// # Examples
    ///
    /// Create a login, renew it, and verify the expiry has been extended
    /// ```
    /// use data_model::UserLogin;
    /// let mut login = UserLogin::new(24, None, None);
    /// let original_expiry = login.expires_at;
    /// login.renew(24);
    /// assert!(login.expires_at > original_expiry);
    /// ```
    pub fn renew(&mut self, expiry_hours: u32) {
        self.expires_at = generate_now() + Duration::hours(expiry_hours.into());
    }

    /// Generates the JSON string representation for the user login
    ///
    /// # Returns
    ///
    /// JSON string representing the user login
    ///
    /// # Examples
    ///
    /// Initialize a user login, convert it to JSON and print it
    /// ```
    /// use data_model::UserLogin;
    /// let login = UserLogin::new(24, Some("123.456.789.012".to_string()), Some("Device info string".to_string()));
    /// match login.to_json() {
    ///     Ok(json) => {
    ///         println!("JSON: {}", json);
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to convert login to JSON => {}",
    ///           error
    ///        );
    ///     }
    /// }
    pub fn to_json(&self) -> Result<String, Error> {
        Ok(serde_json::to_string(&self)?)
    }

    /// Initializes a user login instance by reading the JSON string content provided
    ///
    /// # Returns
    ///
    /// This function will return an error if the JSON could not be read
    ///
    /// # Examples
    ///
    /// Create a default user login and then reinitialize it from a JSON string definition
    /// ```
    /// use data_model::UserLogin;
    /// let mut login = UserLogin::default();
    /// let json = r#"{"id":"89a94185-e27f-4ce5-8841-f74da4962ed6","created_at":"2025-03-31T07:48:22.742367800Z","expires_at":"2025-04-01T07:48:22.742367800Z","device_info":"Some device information","ip_address":"123.456.789.123"}"#;
    /// match login.from_json(&json) {
    ///     Ok(()) => {
    ///         println!(
    ///             "JSON successfully read user login  =>  {:?}",
    ///             login
    ///         );
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to read JSON into login => {}",
    ///             error
    ///        );
    ///     }
    /// }
    pub fn from_json(&mut self, json: &str) -> Result<(), Error> {
        let temp: UserLogin = serde_json::from_str(json)?;
        *self = temp;
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn can_create() {
        let _ = UserLogin::new(24, Some("123.456.789.123".to_string()), Some("Some device information".to_string()));
    }

    #[test]
    fn can_serialize_and_deserialize() {
        let login_created = UserLogin::new(24, Some("123.456.789.123".to_string()), Some("Some device information".to_string()));
        let json = login_created.to_json().expect("Failed to serialize");
        let mut login_loaded  = UserLogin::new(0, Some("".to_string()), Some("".to_string()));
        if let Err(err) = login_loaded.from_json(&json) {
            panic!("failed to deserialize: {}", err);
        }
        assert_eq!(login_created, login_loaded);
    }

    #[test]
    fn can_renew() {
        let mut login = UserLogin::new(24, None, None);
        let original_expiry = login.expires_at;
        login.renew(24);
        assert!(login.expires_at > original_expiry);
    }

}
