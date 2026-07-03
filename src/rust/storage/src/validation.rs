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

/// Validates that the number of `'K'` (key) and `'D'` (door) cells in
/// `grid` fits within `max`. Each `MazeStore` impl calls this from
/// `create_maze` / `update_maze` to refuse over-cap mazes before they
/// reach storage — the key-aware solver tracks each `'K'` and `'D'` as
/// a bit in a `u32` mask, so its search is exponential in their sum and
/// it refuses to solve above `max::maze::MAX_TOTAL_FEATURES`. The cap
/// here mirrors that limit so a saved maze can always be solved.
///
/// # Returns
///
/// `Ok(())` if `keys + doors ≤ max`,
/// `Err(Error::MazeHasTooManyFeatures { keys, doors, max })` otherwise.
///
/// # Examples
///
/// Probe a grid with 5 keys + 4 doors against an 16-feature cap
/// ```
/// use storage::validation::validate_maze_feature_count;
///
/// let grid: Vec<Vec<char>> = vec![
///     vec!['S', 'K', 'D', 'K', 'F'],
///     vec!['K', 'D', 'K', 'D', 'K'],
/// ];
/// assert!(validate_maze_feature_count(&grid, 16).is_ok());
/// ```
/// Validates that the serialised maze definition fits within the supplied
/// per-store byte cap. A store that persists the definition into a
/// length-bounded column (e.g. `SqlStore`'s `VARCHAR(16000)`) calls this from
/// `create_maze` / `update_maze` on the exact string it is about to write, so
/// an over-cap maze is refused with a clear error rather than truncated or
/// rejected by the database. The cell-count cap is a proxy for size that
/// assumes plain single-character cells; per-cell entity overrides inflate a
/// cell well beyond that, so this byte check is the authoritative storage
/// guard.
///
/// # Returns
///
/// `Ok(())` if `bytes ≤ max`,
/// `Err(Error::MazeDefinitionTooLarge { bytes, max })` otherwise.
///
/// # Examples
///
/// Probe a 12 KB and a 20 KB serialised definition against a 16 KB cap
/// ```
/// use storage::validation::validate_maze_definition_size;
///
/// assert!(validate_maze_definition_size(12_000, 16_000).is_ok());
/// assert!(validate_maze_definition_size(20_000, 16_000).is_err());
/// ```
pub fn validate_maze_definition_size(bytes: usize, max: usize) -> Result<(), Error> {
    if bytes > max {
        return Err(Error::MazeDefinitionTooLarge { bytes, max });
    }
    Ok(())
}

/// Validates that a serialised game-definition `config` fits within the
/// supplied per-store byte cap. Mirrors [`validate_maze_definition_size`]: each
/// `GameStore` backend calls this from `create` / `update` on the exact `config`
/// JSON it is about to persist, so an over-cap config is refused with a clear
/// error rather than truncated or rejected by the database. A game definition
/// stores no per-cell grid (the maze is regenerated from `seed`), so its config
/// is tiny in practice and never approaches the cap today — the guard exists so
/// a future expansion of the config's content cannot silently overflow the
/// storage column.
///
/// # Returns
///
/// `Ok(())` if `bytes ≤ max`,
/// `Err(Error::GameDefinitionConfigTooLarge { bytes, max })` otherwise.
///
/// # Examples
///
/// Probe a small and an over-cap serialised config against a 16 KB cap
/// ```
/// use storage::validation::validate_game_definition_config_size;
///
/// assert!(validate_game_definition_config_size(500, 16_000).is_ok());
/// assert!(validate_game_definition_config_size(16_001, 16_000).is_err());
/// ```
pub fn validate_game_definition_config_size(bytes: usize, max: usize) -> Result<(), Error> {
    if bytes > max {
        return Err(Error::GameDefinitionConfigTooLarge { bytes, max });
    }
    Ok(())
}

pub fn validate_maze_feature_count(grid: &[Vec<char>], max: usize) -> Result<(), Error> {
    let mut keys = 0usize;
    let mut doors = 0usize;
    for row in grid {
        for &ch in row {
            match ch {
                'K' => keys += 1,
                'D' => doors += 1,
                _ => {}
            }
        }
    }
    if keys + doors > max {
        return Err(Error::MazeHasTooManyFeatures { keys, doors, max });
    }
    Ok(())
}

/// Validates that a maze carries no more enemies / health pickups / treasure than
/// [`maze::MAX_ENEMY_COUNT`] / [`maze::MAX_HEALTH_COUNT`] / [`maze::MAX_TREASURE_COUNT`]
/// respectively — the same per-type caps generation enforces, applied to authored
/// mazes so the editor cannot paint a maze whose in-game object count exceeds what
/// generation would ever place. (An unbounded treasure pile, for instance,
/// overwhelms a mobile GPU with per-chest lights and sparkles.) Each store calls
/// this from `create_maze` / `update_maze`. Doors are not checked here — they fall
/// under the combined key + door cap validated by [`validate_maze_feature_count`].
///
/// # Returns
///
/// `Ok(())` if every limited type is at or under its cap,
/// `Err(Error::MazeHasTooManyObjects { kind, count, max })` for the first type
/// that exceeds it.
///
/// # Examples
///
/// A maze with a handful of each object passes; one with more treasure than the
/// cap is refused
/// ```
/// use storage::validation::validate_maze_object_counts;
///
/// let ok: Vec<Vec<char>> = vec![vec!['S', 'E', 'T', 'H', 'F']];
/// assert!(validate_maze_object_counts(&ok).is_ok());
///
/// let too_much_treasure: Vec<Vec<char>> = vec![std::iter::repeat_n('T', 13).collect()];
/// assert!(validate_maze_object_counts(&too_much_treasure).is_err());
/// ```
pub fn validate_maze_object_counts(grid: &[Vec<char>]) -> Result<(), Error> {
    let mut enemies = 0usize;
    let mut health = 0usize;
    let mut treasure = 0usize;
    for row in grid {
        for &ch in row {
            match ch {
                'E' => enemies += 1,
                'H' => health += 1,
                'T' => treasure += 1,
                _ => {}
            }
        }
    }
    for (count, max, kind) in [
        (enemies, maze::MAX_ENEMY_COUNT, "enemies"),
        (health, maze::MAX_HEALTH_COUNT, "health pickups"),
        (treasure, maze::MAX_TREASURE_COUNT, "treasure items"),
    ] {
        if count > max {
            return Err(Error::MazeHasTooManyObjects { kind, count, max });
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
            deleted_at: None,
            created_at: chrono::Utc::now(),
            last_sign_in_at: None,
            avatar_updated_at: None,
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

    // ─── validate_maze_definition_size ───────────────────────────────────

    #[test]
    fn validate_maze_definition_size_accepts_at_cap() {
        validate_maze_definition_size(16_000, 16_000).expect("at-cap should pass");
    }

    #[test]
    fn validate_maze_definition_size_accepts_under_cap() {
        validate_maze_definition_size(12_345, 16_000).expect("under-cap should pass");
    }

    #[test]
    fn validate_maze_definition_size_rejects_over_cap() {
        let err = validate_maze_definition_size(16_001, 16_000).expect_err("over-cap should fail");
        match err {
            Error::MazeDefinitionTooLarge { bytes, max } => {
                assert_eq!(bytes, 16_001);
                assert_eq!(max, 16_000);
            }
            other => panic!("expected MazeDefinitionTooLarge, got {other:?}"),
        }
    }

    // ─── validate_game_definition_config_size ────────────────────────────

    #[test]
    fn validate_game_definition_config_size_accepts_at_cap() {
        validate_game_definition_config_size(16_000, 16_000).expect("at-cap should pass");
    }

    #[test]
    fn validate_game_definition_config_size_rejects_over_cap() {
        let err = validate_game_definition_config_size(16_001, 16_000)
            .expect_err("over-cap should fail");
        match err {
            Error::GameDefinitionConfigTooLarge { bytes, max } => {
                assert_eq!(bytes, 16_001);
                assert_eq!(max, 16_000);
            }
            other => panic!("expected GameDefinitionConfigTooLarge, got {other:?}"),
        }
    }

    // ─── validate_maze_feature_count ─────────────────────────────────────

    fn grid_with_keys_and_doors(keys: usize, doors: usize) -> Vec<Vec<char>> {
        let mut row: Vec<char> = std::iter::repeat_n('K', keys).collect();
        row.extend(std::iter::repeat_n('D', doors));
        vec![row]
    }

    #[test]
    fn validate_maze_feature_count_accepts_at_cap() {
        validate_maze_feature_count(&grid_with_keys_and_doors(8, 8), 16)
            .expect("at-cap should pass");
    }

    #[test]
    fn validate_maze_feature_count_accepts_just_under_cap() {
        let grid: Vec<Vec<char>> = vec![vec!['K', 'D', 'K', 'D', 'K', 'D']]; // 3+3=6
        validate_maze_feature_count(&grid, 16).expect("under-cap should pass");
    }

    #[test]
    fn validate_maze_feature_count_rejects_over_cap() {
        let err = validate_maze_feature_count(&grid_with_keys_and_doors(9, 8), 16)
            .expect_err("over-cap should fail");
        match err {
            Error::MazeHasTooManyFeatures { keys, doors, max } => {
                assert_eq!(keys, 9);
                assert_eq!(doors, 8);
                assert_eq!(max, 16);
            }
            other => panic!("expected MazeHasTooManyFeatures, got {other:?}"),
        }
    }

    #[test]
    fn validate_maze_feature_count_ignores_other_cells() {
        // S/F/W/' ' don't count toward the budget.
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W', 'F'],
            vec![' ', 'W', ' ', ' '],
        ];
        validate_maze_feature_count(&grid, 0).expect("no K/D so any cap passes");
    }

    // ─── validate_maze_object_counts ─────────────────────────────────────

    fn grid_with_cell(ch: char, n: usize) -> Vec<Vec<char>> {
        vec![std::iter::repeat_n(ch, n).collect()]
    }

    #[test]
    fn validate_maze_object_counts_accepts_each_type_at_cap() {
        validate_maze_object_counts(&grid_with_cell('E', maze::MAX_ENEMY_COUNT))
            .expect("enemies at cap should pass");
        validate_maze_object_counts(&grid_with_cell('H', maze::MAX_HEALTH_COUNT))
            .expect("health at cap should pass");
        validate_maze_object_counts(&grid_with_cell('T', maze::MAX_TREASURE_COUNT))
            .expect("treasure at cap should pass");
    }

    #[test]
    fn validate_maze_object_counts_rejects_over_cap_treasure() {
        let err = validate_maze_object_counts(&grid_with_cell('T', maze::MAX_TREASURE_COUNT + 1))
            .expect_err("over-cap treasure should fail");
        match err {
            Error::MazeHasTooManyObjects { kind, count, max } => {
                assert_eq!(kind, "treasure items");
                assert_eq!(count, maze::MAX_TREASURE_COUNT + 1);
                assert_eq!(max, maze::MAX_TREASURE_COUNT);
            }
            other => panic!("expected MazeHasTooManyObjects, got {other:?}"),
        }
    }

    #[test]
    fn validate_maze_object_counts_rejects_over_cap_enemies() {
        let err = validate_maze_object_counts(&grid_with_cell('E', maze::MAX_ENEMY_COUNT + 1))
            .expect_err("over-cap enemies should fail");
        assert!(matches!(
            err,
            Error::MazeHasTooManyObjects { kind: "enemies", .. }
        ));
    }

    #[test]
    fn validate_maze_object_counts_ignores_keys_doors_and_terrain() {
        // K/D (counted by the feature validator) and S/F/W/' ' don't count here.
        let grid: Vec<Vec<char>> = vec![
            vec!['S', 'K', 'D', 'W', 'F'],
            vec![' ', 'K', 'D', ' ', ' '],
        ];
        validate_maze_object_counts(&grid).expect("no E/H/T so it passes");
    }
}