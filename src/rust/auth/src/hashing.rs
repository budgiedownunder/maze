use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, Params, password_hash::SaltString};
use rand_core::OsRng; 
use crate::config::PasswordHashConfig;

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

pub fn verify_password(hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    let argon2 = Argon2::default();

    Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn get_hash_config() -> PasswordHashConfig {
        PasswordHashConfig {
            mem_cost: 65536,
            time_cost: 3,
            lanes: 4,
            hash_length: 32,
        }
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