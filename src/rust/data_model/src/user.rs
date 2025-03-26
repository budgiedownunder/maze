use crate::{Error, UserValidationError};
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
    /// # Returns
    ///
    /// This function will return an error if the JSON could not be read
    ///
    /// # Examples
    ///
    /// Create a default user and then reinitialize it from a JSON string definition
    /// ```
    /// use data_model::User;
    /// let mut user = User::default();
    /// let json = r#"{"id":"02345678-1234-5678-1234-567890123456","is_admin":false,"username":"john_smith","full_name":"John Smith","email":"john_smith@company.com","password_hash":"some_password_hash","api_key":"12345678-1234-5678-1234-567890123456"}"#;
    /// match user.from_json(json) {
    ///     Ok(()) => {
    ///         println!(
    ///             "JSON successfully read into User => username = {}",
    ///             user.username
    ///         );
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to read JSON into user => {}",
    ///             error
    ///        );
    ///     }
    /// }
    pub fn from_json(&mut self, json: &str) -> Result<(), Error> {
        let temp: User = serde_json::from_str(json)?;
        *self = temp;
        Ok(())
    }
    /// Validates the content of a user object
    ///
    /// # Returns
    ///
    /// JSON string representing the user
    ///
    ///
    /// # Examples
    ///
    /// Initialize a user with an invalid email address (missing @) and validate it
    /// ```
    /// use data_model::User;
    /// let user = User {
    ///     id: User::new_id(),
    ///     is_admin: false,
    ///     username: "john_smith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "bad_email".to_string(),
    ///     password_hash: "encrypted_hash".to_string(),
    ///     api_key: User::new_api_key(),
    /// };
    /// match user.validate() {
    ///     Ok(_) => {
    ///         println!("Validation passed");
    ///     }
    ///     Err(error) => {
    ///        println!(
    ///            "Validation failed => {}",
    ///           error
    ///        );
    ///     }
    /// }
    pub fn validate(&self) -> Result<(), Error> {
        if self.id == Uuid::nil() {
            return Err(Error::UserValidation(UserValidationError::IdMissing));
        }
        if self.username.is_empty() {
            return Err(Error::UserValidation(UserValidationError::UsernameMissing));
        }
        if !self.email.is_empty() && !self.email.contains("@") {
            return Err(Error::UserValidation(UserValidationError::EmailInvalid));
        }
        if self.password_hash.is_empty() {
            return Err(Error::UserValidation(UserValidationError::PasswordMissing));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `id`\", line: 1, column: 126))"
    )]    
    fn cannot_deserialize_with_missing_id() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"is_admin":false,"username":"","full_name":"","email":"","password_hash":"","api_key":"00000000-0000-0000-0000-000000000000"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `is_admin`\", line: 1, column: 153))"
    )]    
    fn cannot_deserialize_with_missing_is_admin() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","username":"","full_name":"","email":"","password_hash":"","api_key":"00000000-0000-0000-0000-000000000000"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `username`\", line: 1, column: 156))"
    )]    
    fn cannot_deserialize_with_missing_username() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"full_name":"","email":"","password_hash":"","api_key":"00000000-0000-0000-0000-000000000000"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `full_name`\", line: 1, column: 155))"
    )]    
    fn cannot_deserialize_with_missing_full_name() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","email":"","password_hash":"","api_key":"00000000-0000-0000-0000-000000000000"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `email`\", line: 1, column: 159))"
    )]    
    fn cannot_deserialize_with_missing_email() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","password_hash":"","api_key":"00000000-0000-0000-0000-000000000000"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `password_hash`\", line: 1, column: 151))"
    )]    
    fn cannot_deserialize_with_missing_password_hash() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","email":"","api_key":"00000000-0000-0000-0000-000000000000"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `api_key`\", line: 1, column: 121))"
    )]    
    fn cannot_deserialize_with_missing_api_key() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","email":"","password_hash":""}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    fn create_valid_user() -> User {
        User {
            id: User::new_id(),
            is_admin: false,
            username: "john_smith".to_string(),
            full_name: "John Smith".to_string(),
            email: "john_smith@company.com".to_string(),
            password_hash: "encrypted_hash".to_string(),
            api_key: User::new_api_key(),
        }
    }

    fn validate_user(user: &User) {
        match user.validate() {
            Ok(_) => {},
            Err(error) => panic!("{}", error)
        }
    }

   #[test]
    fn user_validation_should_pass() {
        let user = create_valid_user();
        validate_user(&user);
    }

    #[test]
    #[should_panic(
        expected = "No id provided for the user"
    )]    
    fn user_validation_should_fail_with_missing_ids() {
        let mut user = create_valid_user();
        user.id = Uuid::nil();
        validate_user(&user);
    }

    #[test]
    #[should_panic(
        expected = "No username provided for the user"
    )]    
    fn user_validation_should_fail_with_missing_username() {
        let mut user = create_valid_user();
        user.username = "".to_string();
        validate_user(&user);
    }

    #[test]
    #[should_panic(
        expected = "Invalid email address"
    )]    
    fn user_validation_should_fail_with_bad_email() {
        let mut user = create_valid_user();
        user.email = "bad_email".to_string();
        validate_user(&user);
    }

    #[test]
    #[should_panic(
        expected = "No password provided for the user"
    )]    
    fn user_validation_should_fail_with_missing_password_hash() {
        let mut user = create_valid_user();
        user.password_hash = "".to_string();
        validate_user(&user);
    }
}