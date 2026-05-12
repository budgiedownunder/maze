use data_model::{is_valid_email_format, Error as DataModelError, User, UserValidationError};
use crate::Error;

/// Validates a single email address for storage operations that take only
/// the address (no surrounding `User`). Centralises the "is it empty? does
/// it match the data-model regex?" pair that every email-management
/// `UserStore` method runs up front.
///
/// # Returns
///
/// `Ok(())` if the address is non-empty and well-formed,
/// `Err(Error::UserEmailMissing)` if blank,
/// `Err(Error::UserEmailInvalid)` otherwise.
///
/// # Examples
///
/// Probe a few candidate addresses before passing them to a store method
/// ```
/// use storage::validation::validate_email_format;
///
/// assert!(validate_email_format("alice@example.com").is_ok());
/// assert!(validate_email_format("").is_err());
/// assert!(validate_email_format("not-an-email").is_err());
/// ```
pub fn validate_email_format(email: &str) -> Result<(), Error> {
    if email.trim().is_empty() {
        return Err(Error::UserEmailMissing());
    }
    if !is_valid_email_format(email) {
        return Err(Error::UserEmailInvalid());
    }
    Ok(())
}

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

/// Validates that a maze of `rows × cols` fits within the supplied per-store
/// cell-count cap. Each `MazeStore` impl calls this from `create_maze` /
/// `update_maze` so the limit is enforced uniformly. `saturating_mul` keeps
/// the comparison meaningful when the inputs are pathologically large —
/// instead of panicking on overflow or wrapping to a small number that
/// silently passes the check, the product clamps to `usize::MAX` and the
/// guard rejects.
///
/// # Returns
///
/// `Ok(())` if `rows × cols ≤ max`,
/// `Err(Error::MazeHasTooManyCells { rows, cols, max })` otherwise.
///
/// # Examples
///
/// Probe a 60×60 and a 70×60 grid against a 3,600-cell cap
/// ```
/// use storage::validation::validate_maze_cell_count;
///
/// assert!(validate_maze_cell_count(60, 60, 3_600).is_ok());
/// assert!(validate_maze_cell_count(70, 60, 3_600).is_err());
/// ```
pub fn validate_maze_cell_count(rows: usize, cols: usize, max: usize) -> Result<(), Error> {
    if rows.saturating_mul(cols) > max {
        return Err(Error::MazeHasTooManyCells { rows, cols, max });
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
            deleted_at: None,
            created_at: chrono::Utc::now(),
            last_sign_in_at: None,
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

    // ─── validate_maze_cell_count ────────────────────────────────────────

    #[test]
    fn validate_maze_cell_count_accepts_at_cap() {
        validate_maze_cell_count(60, 60, 3_600).expect("at-cap should pass");
    }

    #[test]
    fn validate_maze_cell_count_accepts_just_under_cap() {
        validate_maze_cell_count(60, 59, 3_600).expect("under-cap should pass");
    }

    #[test]
    fn validate_maze_cell_count_rejects_over_cap() {
        let err = validate_maze_cell_count(61, 60, 3_600)
            .expect_err("over-cap should fail");
        match err {
            Error::MazeHasTooManyCells { rows, cols, max } => {
                assert_eq!(rows, 61);
                assert_eq!(cols, 60);
                assert_eq!(max, 3_600);
            }
            other => panic!("expected MazeHasTooManyCells, got {other:?}"),
        }
    }

    #[test]
    fn validate_maze_cell_count_saturates_on_overflow() {
        // Pathological inputs: rows × cols would wrap, but saturating_mul
        // clamps to usize::MAX which is always > max, so the guard rejects.
        let err = validate_maze_cell_count(usize::MAX, 2, 3_600)
            .expect_err("overflow should not bypass the cap");
        assert!(matches!(err, Error::MazeHasTooManyCells { .. }));
    }
}