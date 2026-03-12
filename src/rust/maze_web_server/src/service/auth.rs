use argon2;
use auth::{config::PasswordHashConfig, hashing::{hash_password, verify_password}};

/// `AuthService` provides password hashing and verification functionality.
///
/// This service wraps low-level cryptographic operations and exposes a clean interface
/// for use in route handlers and other parts of the application. It requires a
/// `PasswordHashConfig` to be provided during initialization, which defines parameters
/// such as memory cost and hash length.
///
/// # Example
///
/// ```
/// use auth::config::PasswordHashConfig;
/// use maze_web_server::service::auth::AuthService;
///
/// let config = PasswordHashConfig {
///     mem_cost: 65536,
///     time_cost: 3,
///     lanes: 4,
///     hash_length: 32,
/// };
///
/// let auth_service = AuthService::new(config);
/// let hash = auth_service.hash_password("mypassword").unwrap();
/// assert!(auth_service.verify_password(&hash, "mypassword").unwrap());
/// ```
#[derive(Debug, Clone)]
pub struct AuthService {
    config: PasswordHashConfig,
}

impl AuthService {
    /// Creates a new instance of `AuthService` with the given configuration.
    pub fn new(config: PasswordHashConfig) -> Self {
        Self { config }
    }

    /// Hashes a plain-text password using Argon2 with the configured parameters.
    ///
    /// The result is a PHC-encoded string containing the algorithm, salt, and hash.
    ///
    /// # Arguments
    ///
    /// * `password` - The plain-text password to hash.
    ///
    /// # Returns
    ///
    /// A `Result` containing the encoded hash string, or an error if hashing fails.
    ///
    /// # Example
    ///
    /// Hash a password and print the hashed value
    /// ```
    /// use auth::config::PasswordHashConfig;
    /// use maze_web_server::service::auth::AuthService;
    ///
    /// let config = PasswordHashConfig {
    ///     mem_cost: 65536,
    ///     time_cost: 3,
    ///     lanes: 4,
    ///     hash_length: 32,
    /// };
    ///
    /// let auth_service = AuthService::new(config);
    /// let hash = auth_service.hash_password("password123").unwrap();
    /// println!("Hashed value = {}", hash);
    /// ```
    pub fn hash_password(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        hash_password(password, &self.config)
    }
    /// Verifies a plain-text password against a previously hashed value.
    ///
    /// # Arguments
    ///
    /// * `hash` - A PHC-encoded password hash string.
    /// * `password` - The plain-text password to verify.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if the password is correct, `Ok(false)` if it's incorrect,
    /// or an error if verification fails (e.g., invalid hash format).
    ///
    /// # Example
    ///
    /// Hash a password and compare it to a bad password  
    /// ```
    /// use auth::config::PasswordHashConfig;
    /// use maze_web_server::service::auth::AuthService;
    ///
    /// let config = PasswordHashConfig {
    ///     mem_cost: 65536,
    ///     time_cost: 3,
    ///     lanes: 4,
    ///     hash_length: 32,
    /// };
    ///
    /// let auth_service = AuthService::new(config);
    /// let hash = auth_service.hash_password("actual_password").unwrap();
    /// let is_match = auth_service.verify_password(&hash, "bad_password").unwrap();
    /// println!("Password matches = {}", is_match);
    /// ```
    pub fn verify_password(
        &self,
        hash: &str,
        password: &str,
    ) -> Result<bool, argon2::password_hash::Error> {
        verify_password(hash, password)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use auth::config::PasswordHashConfig;

    fn sample_config() -> PasswordHashConfig {
        PasswordHashConfig {
            mem_cost: 65536,
            time_cost: 3,
            lanes: 4,
            hash_length: 32,
        }
    }

    #[test]
    fn verify_password_round_trip_success() {
        let service = AuthService::new(sample_config());
        let password = "super_secret";

        let hash = service.hash_password(password).expect("Should hash password");
        let verified = service.verify_password(&hash, password).expect("Should verify password");

        assert!(verified);
    }

    #[test]
    fn verify_password_verification_failure() {
        let service = AuthService::new(sample_config());
        let hash = service.hash_password("correct_password").unwrap();

        let result = service.verify_password(&hash, "wrong_password").unwrap();
        assert!(!result);
    }

    #[test]
    fn verify_password_hash_format() {
        let service = AuthService::new(sample_config());
        let hash = service.hash_password("abc123").unwrap();

        assert!(
            hash.starts_with("$argon2id$"),
            "Hash should use the argon2id algorithm"
        );
    }
}
