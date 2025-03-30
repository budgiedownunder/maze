use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, Params, password_hash::SaltString};
use rand_core::OsRng; 
use crate::config::PasswordHashConfig;

/// Hashes a plain-text password using the Argon2id algorithm and the provided configuration.
///
/// A cryptographically secure random salt is generated automatically and included in the final hash.
/// The output hash is encoded in the [PHC string format](https://github.com/P-H-C/phc-string-format)
/// and can be safely stored in a database for later verification.
///
/// # Arguments
///
/// * `password` - The plain-text password to hash.
/// * `cfg` - A reference to a `PasswordHashConfig` struct containing Argon2 parameters.
///
/// # Returns
///
/// A `Result` containing the encoded password hash on success, or a `argon2::password_hash::Error` on failure.
///
/// # Example
///
/// ```rust
/// use auth::hashing::hash_password;
/// use auth::config::PasswordHashConfig;
///
/// let cfg = PasswordHashConfig {
///     mem_cost: 65536,
///     time_cost: 3,
///     lanes: 4,
///     hash_length: 32,
/// };
///
/// let password = "my_secure_password";
/// let hash = hash_password(password, &cfg).expect("Failed to hash password");
///
/// println!("Password hash: {}", hash);
/// assert!(hash.starts_with("$argon2id$"));
/// ```
pub fn hash_password(password: &str, cfg: &PasswordHashConfig) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    let params = Params::new(
        cfg.mem_cost,
        cfg.time_cost,
        cfg.lanes,
        Some(cfg.hash_length),
    )?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string()) // includes salt in the output
}
/// Verifies that a plain-text password matches a previously generated Argon2 hash.
///
/// The hash must be in [PHC string format](https://github.com/P-H-C/phc-string-format), 
/// which includes information about the algorithm, salt, and parameters used. This is the format
/// produced by [`hash_password`](fn.hash_password.html).
///
/// # Arguments
///
/// * `hash` - A PHC-encoded Argon2 hash string (e.g., from your database).
/// * `password` - The plain-text password to verify.
///
/// # Returns
///
/// A `Result` containing `true` if the password matches the hash, or `false` if it does not.
/// Returns an error if the hash cannot be parsed or if verification fails unexpectedly.
///
/// # Example
///
/// ```rust
/// use auth::hashing::{hash_password, verify_password};
/// use auth::config::PasswordHashConfig;
///
/// let cfg = PasswordHashConfig {
///     mem_cost: 65536,
///     time_cost: 3,
///     lanes: 4,
///     hash_length: 32,
/// };
///
/// let password = "my_secure_password";
/// let hash = hash_password(password, &cfg).expect("Hashing failed");
///
/// let is_valid = verify_password(&hash, password).expect("Verification failed");
/// assert!(is_valid);
///
/// let wrong = "wrong_password";
/// let is_valid = verify_password(&hash, wrong).expect("Verification failed");
/// assert!(!is_valid);
/// ```
pub fn verify_password(hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn get_hash_config() -> PasswordHashConfig {
        PasswordHashConfig::default()
    }    

    #[test]
    fn hash_password_should_succeed() {
        let password = "super_secure_password";
        let result = hash_password(password, &get_hash_config());
        assert!(result.is_ok(), "Password hashing should succeed");

        let hash = result.unwrap();
        assert!(
            hash.starts_with("$argon2id$"),
            "Hash should use argon2id and be correctly formatted"
        );
    }

    #[test]
    fn verify_password_should_succeed() {
        let password = "super_secure_password";
        let hash = hash_password(password, &get_hash_config()).expect("Password hashing did not succeed");

        let is_valid = verify_password(&hash, password).expect("Password verification generated an unexpected error");
        assert!(is_valid, "Password verification failed (but success was expected");
    }

    #[test]
    fn verify_password_should_fail() {
        let good_password = "super_secure_password";
        let hash = hash_password(good_password, &get_hash_config()).expect("Password hashing did not succeed");
        let bad_password = "wrong_password";
        let is_valid = verify_password(&hash, bad_password).expect("Password verification generated an unexpected error");
        assert!(!is_valid, "Password verification succeeded (but failure was expected");
    }

}    