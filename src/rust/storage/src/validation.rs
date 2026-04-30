use data_model::{Error as DataModelError, User, UserValidationError};
use crate::Error;

/// Validates the fields within a user object for create/update within a store
///
/// # Examples
///
/// Validate the default user content. This will fail as the default User content
/// contains some empty fields that need to be populated prior to saving to a store.
///
/// ```
/// use data_model::User;
/// use storage::validation::validate_user_fields;
/// use uuid::Uuid;
///
/// let user = User::default();
/// match validate_user_fields(&user) {
///     Ok(_) => {
///         println!("The User object passed the field validation test for storage");
///     }
///     Err(error) => {
///         println!(
///             "The User object failed the field validation test for storage => {}",
///             error
///         );
///     }
/// }
/// ```
pub fn validate_user_fields(user: &User) -> Result<(), Error> {
    if let Err(DataModelError::UserValidation(error)) = user.validate() {
        match error {
            UserValidationError::EmailInvalid => return Err(Error::UserEmailInvalid()),
            UserValidationError::EmailMissing => return Err(Error::UserEmailMissing()),
            UserValidationError::IdMissing => return Err(Error::UserIdMissing()),
            UserValidationError::PasswordMissing => return Err(Error::UserPasswordMissing()),
            UserValidationError::UsernameMissing => return Err(Error::UserNameMissing()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_model::UserEmail;
    use uuid::Uuid;

    // Initialize a User struct
    fn init_valid_user() -> User {
        User {
            id: User::new_id(),
            is_admin: false,
            username: "john_smith".to_string(),
            full_name:"John Smith".to_string(),
            emails: vec![UserEmail::new_primary_verified("john_smith@company.com")],
            password_hash: "a_password_hash".to_string(),
            api_key: User::new_api_key(),
            logins: vec![],
            oauth_identities: vec![],
        }
    }

    fn run_validation_test(user: &User) {
        if let Err(error) = validate_user_fields(user) {
            panic!("{error}'");
        }
    }

    #[test]
    fn validation_should_succeed_for_valid_user() {
        let user = init_valid_user();
        run_validation_test(&user);
    }

    #[test]
    #[should_panic(expected = "No id provided for the user")]
    fn validation_should_fail_for_missing_id() {
        let mut user = init_valid_user();
        user.id = Uuid::nil();
        run_validation_test(&user);
    }

    #[test]
    #[should_panic(expected = "No username provided for the user")]
    fn validation_should_fail_for_missing_username() {
        let mut user = init_valid_user();
        user.username = "".to_string();
        run_validation_test(&user);
    }

    #[test]
    #[should_panic(expected = "No password provided for the user")]
    fn validation_should_fail_for_missing_password() {
        let mut user = init_valid_user();
        user.password_hash = "".to_string();
        run_validation_test(&user);
    }

    #[test]
    #[should_panic(expected = "No email address provided for the user")]
    fn validation_should_fail_for_missing_email() {
        let mut user = init_valid_user();
        user.emails.clear();
        run_validation_test(&user);
    }

    #[test]
    #[should_panic(expected = "The email address is invalid")]
    fn validation_should_fail_for_invalid_email() {
        let mut user = init_valid_user();
        user.emails[0].email = "bad_email_address".to_string();
        run_validation_test(&user);
    }

}