use crate::{Error, UserEmail, wrappers::{generate_now, generate_uuid}, OAuthIdentity, UserLogin, UserValidationError};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;
use utoipa::ToSchema;
use uuid::Uuid;

/// Checks that an email address has the shape `local@domain.tld` with no
/// whitespace in any part. Exposed for storage backends and other layers
/// that need to validate user-supplied addresses outside of the full
/// [`User::validate`] flow.
///
/// # Examples
///
/// ```
/// use data_model::is_valid_email_format;
///
/// assert!(is_valid_email_format("alice@example.com"));
/// assert!(!is_valid_email_format(""));
/// assert!(!is_valid_email_format("not-an-email"));
/// assert!(!is_valid_email_format("alice @example.com"));
/// ```
pub fn is_valid_email_format(email: &str) -> bool {
    static RE: OnceLock<Regex> = OnceLock::new();
    let re = RE.get_or_init(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").expect("Invalid email regex"));
    re.is_match(email)
}

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
    /// Email addresses attached to this user. Exactly one row has
    /// `is_primary = true` at all times for a well-formed user.
    pub emails: Vec<UserEmail>,
    /// Password hash (encrypted)
    pub password_hash: String,
    #[schema(value_type = String)] // Treat as string during serlialization
    /// API key
    pub api_key: Uuid,
    // Logins
    pub logins: Vec<UserLogin>,
    /// External OAuth identities linked to this user. Empty for users that have
    /// only ever signed in with a password. `serde(default)` keeps existing user
    /// JSON files (written before this field existed) readable without migration.
    #[serde(default)]
    pub oauth_identities: Vec<OAuthIdentity>,
    /// Soft-delete marker. `None` for active users; `Some(timestamp)` for
    /// soft-deleted users. Storage backends apply a `deleted_at IS NULL`
    /// filter on every read path so soft-deleted users are invisible to
    /// the application layer. `serde(default)` keeps user JSON files
    /// written before this field existed readable without migration;
    /// `skip_serializing_if = Option::is_none` keeps the absent case out
    /// of the serialised form for active users.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<DateTime<Utc>>,
    /// Account creation timestamp. Set to `Utc::now()` at user creation;
    /// immutable thereafter. Storage migration v8 backfills existing rows
    /// to a fixed sentinel timestamp — accuracy for pre-migration users
    /// doesn't matter (this isn't a live system yet) but the field must
    /// always be populated so callers can rely on it being present.
    pub created_at: DateTime<Utc>,
    /// Sticky timestamp of the user's most recent successful sign-in (any
    /// method — credentials or OAuth). `None` for a freshly-created user
    /// who has never signed in; `Some(timestamp)` from the first successful
    /// sign-in onwards. Sign-out and bearer-token expiry both prune entries
    /// from `logins`, but `last_sign_in_at` is sticky — once `Some`, never
    /// `None` again. The welcome-banner trigger is therefore
    ///
    /// ```text
    /// is_first_sign_in = last_sign_in_at.is_none() && logins.is_empty()
    /// ```
    ///
    /// (captured before the sign-in handler updates either field). The
    /// `last_sign_in_at.is_none()` half is the primary signal — the
    /// `logins.is_empty()` clause is a defensive guard against the
    /// pathological case where the user has an active session but no
    /// recorded `last_sign_in_at` (e.g. a deployment gap where logins
    /// were created before this field existed, or external data
    /// manipulation that cleared `last_sign_in_at` without clearing
    /// `logins`).
    /// `serde(default)` + `skip_serializing_if` keep older user JSON files
    /// readable; migration v8 backfills existing rows to the migration-run
    /// timestamp so existing users don't false-positive as first-time on
    /// their next sign-in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_sign_in_at: Option<DateTime<Utc>>,
    /// Timestamp of the user's most recent avatar upload, or `None` when the
    /// user has no avatar. Doubles as the "has an avatar" signal and the
    /// cache-buster for the avatar image URL. The avatar bytes themselves
    /// live **outside** the `User` record (a file in the FileStore, a
    /// `user_avatars` row in the SqlStore) so loading a user never drags the
    /// image along; this lightweight marker is all the user-facing surfaces
    /// (profile DTO, scoreboard rows) need to decide between rendering the
    /// image and the generic placeholder. The storage layer keeps it in
    /// lock-step with the stored bytes — `UserStore::set_user_avatar` stamps
    /// it, `UserStore::clear_user_avatar` resets it to `None`.
    /// `serde(default)` + `skip_serializing_if` keep older user JSON files
    /// readable and omit the field for users without an avatar.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub avatar_updated_at: Option<DateTime<Utc>>,
}

impl User {
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
    /// use data_model::{User, UserEmail};
    /// let user = User {
    ///     id: User::new_id(),
    ///     is_admin: false,
    ///     username: "john_smith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("john_smith@company.com")],
    ///     password_hash: "encrypted_hash".to_string(),
    ///     api_key: User::new_api_key(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// println!("User: {:?}", user);
    pub fn new_id() -> Uuid {
        generate_uuid()
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
    /// use data_model::{User, UserEmail};
    /// let user = User {
    ///     id: User::new_id(),
    ///     is_admin: false,
    ///     username: "john_smith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("john_smith@company.com")],
    ///     password_hash: "encrypted_hash".to_string(),
    ///     api_key: User::new_api_key(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// println!("User: {:?}", user);
    pub fn new_api_key() -> Uuid {
        generate_uuid()
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
            emails: vec![],
            password_hash: "".to_string(),
            api_key: Uuid::nil(),
            logins: vec![],
            oauth_identities: vec![],
            deleted_at: None,
            created_at: generate_now(),
            last_sign_in_at: None,
            avatar_updated_at: None,
        }
    }
    /// Returns true if the user is active (i.e. not soft-deleted). Storage
    /// backends already filter soft-deleted rows out of every read path, so
    /// in practice loaded users are always active; this helper exists for
    /// callers that hold a `User` from another source and want to assert
    /// the invariant explicitly.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::User;
    ///
    /// let mut user = User::default();
    /// assert!(user.is_active());
    /// user.deleted_at = Some(chrono::Utc::now());
    /// assert!(!user.is_active());
    /// ```
    pub fn is_active(&self) -> bool {
        self.deleted_at.is_none()
    }
    /// Returns true if the next successful sign-in for this user should be
    /// treated as their **first** sign-in (i.e. should trigger the welcome
    /// banner on the client). Captures the state observed by the sign-in
    /// handler **before** it appends a new login or flips `last_sign_in_at`.
    ///
    /// The sticky `last_sign_in_at.is_none()` check is the primary signal;
    /// the `logins.is_empty()` clause is a defensive guard against the
    /// pathological case where the user has an active session but no
    /// recorded `last_sign_in_at` (deployment gap, or external data
    /// manipulation). Once a user signs in successfully under the current
    /// code path the handler sets `last_sign_in_at = Some(now)`, so future
    /// calls return false.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::Utc;
    /// use data_model::User;
    ///
    /// // Brand-new user with no sessions and no recorded sign-in.
    /// let mut user = User::default();
    /// user.last_sign_in_at = None;
    /// user.logins.clear();
    /// assert!(user.is_first_sign_in());
    ///
    /// // Pre-v8 backfilled user: last_sign_in_at populated by migration.
    /// user.last_sign_in_at = Some(Utc::now());
    /// assert!(!user.is_first_sign_in());
    /// ```
    pub fn is_first_sign_in(&self) -> bool {
        self.last_sign_in_at.is_none() && self.logins.is_empty()
    }
    /// Returns the user's primary [`UserEmail`] row, if any.
    ///
    /// A well-formed user loaded from a `UserStore` always has exactly one
    /// row with `is_primary = true`, so in practice this is always `Some`.
    /// `Option` is returned because the type system can't prove the
    /// invariant — callers that don't know they have a primary acknowledge
    /// the case statically rather than risking a panic.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::{User, UserEmail};
    ///
    /// let mut user = User::default();
    /// assert!(user.primary_email().is_none());
    ///
    /// user.emails.push(UserEmail::new_primary_verified("alice@example.com"));
    /// let primary = user.primary_email().expect("primary set");
    /// assert!(primary.is_primary);
    /// assert_eq!(primary.email, "alice@example.com");
    /// ```
    pub fn primary_email(&self) -> Option<&UserEmail> {
        self.emails.iter().find(|e| e.is_primary)
    }
    /// Returns the user's primary email address, or an empty string if no
    /// primary is set. Convenience accessor for callers that previously
    /// read `user.email` directly.
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::{User, UserEmail};
    ///
    /// let mut user = User::default();
    /// assert_eq!(user.email(), "");
    /// user.emails.push(UserEmail::new_primary_verified("alice@example.com"));
    /// assert_eq!(user.email(), "alice@example.com");
    /// ```
    pub fn email(&self) -> &str {
        self.primary_email().map(|e| e.email.as_str()).unwrap_or("")
    }
    /// Returns true if the user has a verified email row matching the given
    /// address (case-insensitive).
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::{User, UserEmail};
    ///
    /// let mut user = User::default();
    /// user.emails.push(UserEmail::new_primary_verified("alice@example.com"));
    /// assert!(user.has_verified_email("ALICE@example.com"));
    /// assert!(!user.has_verified_email("bob@example.com"));
    /// ```
    pub fn has_verified_email(&self, email: &str) -> bool {
        self.emails
            .iter()
            .any(|e| e.verified && e.email.eq_ignore_ascii_case(email))
    }
    /// Sets the user's primary email address. If a primary row already
    /// exists, its `email` is updated in place (`verified` and `verified_at`
    /// are unchanged — callers in the email-management API decide whether a
    /// change should flip the verified flag). If no rows exist yet, a new
    /// primary, verified row is added (used during signup-style flows).
    ///
    /// # Examples
    ///
    /// Seeding the primary email on a fresh `User::default`
    /// ```
    /// use data_model::User;
    ///
    /// let mut user = User::default();
    /// user.set_primary_email_address("alice@example.com");
    /// assert_eq!(user.email(), "alice@example.com");
    /// assert_eq!(user.emails.len(), 1);
    /// assert!(user.emails[0].is_primary);
    /// ```
    pub fn set_primary_email_address(&mut self, email: &str) {
        if let Some(row) = self.emails.iter_mut().find(|e| e.is_primary) {
            row.email = email.to_string();
        } else {
            self.emails.push(UserEmail::new_primary_verified(email));
        }
    }
    /// Generates the JSON string representation for the user
    ///
    /// # Returns
    ///
    /// JSON string representing the user
    ///
    /// # Examples
    ///
    /// Initialize a user, convert it to JSON and print it
    /// ```
    /// use data_model::{User, UserEmail};
    /// let user = User {
    ///     id: User::new_id(),
    ///     is_admin: false,
    ///     username: "john_smith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("john_smith@company.com")],
    ///     password_hash: "encrypted_hash".to_string(),
    ///     api_key: User::new_api_key(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
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
    /// let json = r#"{"id":"02345678-1234-5678-1234-567890123456","is_admin":false,"username":"john_smith","full_name":"John Smith","emails":[{"email":"john_smith@company.com","is_primary":true,"verified":true,"verified_at":null}],"password_hash":"some_password_hash","api_key":"12345678-1234-5678-1234-567890123456","logins":[],"created_at":"2026-01-01T00:00:00Z"}"#;
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
    /// # Examples
    ///
    /// Initialize a user with an invalid email address and validate it
    /// ```
    /// use data_model::{User, UserEmail};
    /// let user = User {
    ///     id: User::new_id(),
    ///     is_admin: false,
    ///     username: "john_smith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("bad_email")],
    ///     password_hash: "encrypted_hash".to_string(),
    ///     api_key: User::new_api_key(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
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
        if self.emails.is_empty() {
            return Err(Error::UserValidation(UserValidationError::EmailMissing));
        }
        for row in &self.emails {
            if row.email.trim().is_empty() {
                return Err(Error::UserValidation(UserValidationError::EmailMissing));
            }
            if !is_valid_email_format(&row.email) {
                return Err(Error::UserValidation(UserValidationError::EmailInvalid));
            }
        }
        // OAuth-only users carry an empty password_hash. Require a password
        // hash *only* when the user has no OAuth identity attached, so that
        // password-only and OAuth-only accounts both validate, while a
        // password-only user with no hash still fails (catches a real bug
        // where signup writes an empty password).
        if self.password_hash.is_empty() && self.oauth_identities.is_empty() {
            return Err(Error::UserValidation(UserValidationError::PasswordMissing));
        }
        Ok(())
    }
    /// Creates a user and then performs a login
    ///
    /// # Returns
    ///
    /// Boolean
    ///
    /// # Examples
    ///
    /// Initialize a user with valid details, create a login and then print the login id
    /// ```
    /// use data_model::{User, UserEmail};
    /// use uuid::Uuid;
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Peform a login
    /// let expiry_hours = 24;
    /// let ip_address = Some("123.456.789.012".to_string());
    /// let device_info = Some("Device info string".to_string());
    ///
    /// let login = user.create_login(expiry_hours, ip_address, device_info);
    /// println!("Created login with id = {}", login.id);
    /// ```
    pub fn create_login(&mut self, expiry_hours: u32, ip_address: Option<String>, device_info: Option<String>) -> UserLogin {
        let now = generate_now();
        self.logins.retain(|login| login.expires_at > now);
        let login = UserLogin::new(expiry_hours, ip_address, device_info);
        self.logins.push(login.clone());
        self.last_sign_in_at = Some(now);
        login
    }
    /// Creates a user and then performs a login
    ///
    /// # Returns
    ///
    /// Boolean
    ///
    /// # Examples
    ///
    /// Initialize a user with valid details, create a login and then remove it - printing the login details and status along the way
    /// ```
    /// use data_model::{User, UserEmail};
    /// use uuid::Uuid;
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Peform a login
    /// let expiry_hours = 24;
    /// let ip_address = Some("123.456.789.012".to_string());
    /// let device_info = Some("Device info string".to_string());
    ///
    /// let login = user.create_login(expiry_hours, ip_address, device_info);
    /// println!("Created login with id = {}, user now contains login = {}", login.id, user.contains_valid_login(login.id));
    /// user.remove_login(login.id);
    /// println!("Removed login with id = {}, user now contains login = {}", login.id, user.contains_valid_login(login.id));
    /// ```
    pub fn remove_login(&mut self, login_id: Uuid)  {
        self.logins.retain(|login| login.id != login_id);
    }
    /// Renews an existing login session by extending its expiry, if the login exists
    ///
    /// # Returns
    ///
    /// The updated login if found, or None if no login with the given id exists
    ///
    /// # Examples
    ///
    /// Create a user and a login, renew it, and verify the expiry has been extended
    /// ```
    /// use data_model::{User, UserEmail, UserLogin};
    /// use uuid::Uuid;
    ///
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// let login = user.create_login(24, None, None);
    /// let original_expiry = login.expires_at;
    /// let renewed = user.renew_login(login.id, 24).expect("Login not found");
    /// assert!(renewed.expires_at > original_expiry);
    /// ```
    pub fn renew_login(&mut self, login_id: Uuid, expiry_hours: u32) -> Option<UserLogin> {
        if let Some(login) = self.logins.iter_mut().find(|l| l.id == login_id) {
            login.renew(expiry_hours);
            Some(login.clone())
        } else {
            None
        }
    }
    /// Checks whether a user object contains the given login token id and, if so, that it has not expired
    ///
    /// # Returns
    ///
    /// Boolean
    ///
    /// # Examples
    ///
    /// Initialize a user with valid details and login token and then verify that the token is valid
    /// ```
    /// use data_model::{User, UserEmail, UserLogin};
    /// use uuid::Uuid;
    ///
    /// // Create the login
    /// let login = UserLogin::new(24, Some("123.456.789.012".to_string()), Some("Device info string".to_string()));
    /// let search_login_id = login.id;
    /// let logins = vec![login];
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins,
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Verify that the login is valid
    /// if user.contains_valid_login(search_login_id) {
    ///     println!("User login found");
    /// }
    /// else {
    ///     println!("User login not found");
    /// }
    /// ```
    pub fn contains_valid_login(&self, id: Uuid) -> bool {
        let now = generate_now();
        self.logins.iter().any(|t| t.id == id && t.expires_at > now)

    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone};
    use pretty_assertions::{assert_eq, assert_ne};

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
        let mut user = User::default();
        // Pin created_at so the round-trip JSON literal below is stable.
        user.created_at = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let s = user.to_json().expect("Failed to serialize");
        assert_eq!(s, r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","emails":[],"password_hash":"","api_key":"00000000-0000-0000-0000-000000000000","logins":[],"oauth_identities":[],"created_at":"2026-01-01T00:00:00Z"}"#);
    }

    #[test]
    fn can_deserialize() {
        let pinned = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let mut compare = User::default();
        compare.created_at = pinned;
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","emails":[],"password_hash":"","api_key":"00000000-0000-0000-0000-000000000000","logins":[],"oauth_identities":[],"created_at":"2026-01-01T00:00:00Z"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    fn can_deserialize_legacy_user_without_oauth_identities() {
        // User JSON written before the oauth_identities field existed must
        // continue to deserialize cleanly thanks to #[serde(default)] —
        // `created_at` is pinned here because that field IS required.
        let pinned = chrono::DateTime::parse_from_rfc3339("2026-01-01T00:00:00Z")
            .unwrap()
            .with_timezone(&chrono::Utc);
        let mut compare = User::default();
        compare.created_at = pinned;
        let mut loaded = User::default();
        let legacy = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","emails":[],"password_hash":"","api_key":"00000000-0000-0000-0000-000000000000","logins":[],"created_at":"2026-01-01T00:00:00Z"}"#;
        loaded.from_json(legacy).expect("Failed to deserialize legacy user JSON");
        assert_eq!(loaded, compare);
        assert!(loaded.oauth_identities.is_empty());
    }

    #[test]
    fn can_deserialize_user_without_deleted_at_field() {
        // User JSON written before the deleted_at field existed must continue
        // to deserialize cleanly thanks to #[serde(default)] — the field is
        // populated with `None` (active user).
        let mut loaded = User::default();
        let pre_field = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","emails":[],"password_hash":"","api_key":"00000000-0000-0000-0000-000000000000","logins":[],"oauth_identities":[],"created_at":"2026-01-01T00:00:00Z"}"#;
        loaded.from_json(pre_field).expect("deserialize pre-deleted_at user");
        assert!(loaded.deleted_at.is_none());
    }

    #[test]
    fn is_active_reflects_deleted_at_state() {
        let mut user = create_valid_user();
        assert!(user.is_active());
        user.deleted_at = Some(chrono::Utc.with_ymd_and_hms(2026, 5, 5, 12, 0, 0).unwrap());
        assert!(!user.is_active());
    }

    #[test]
    fn deleted_at_round_trips_through_json() {
        let mut user = create_valid_user();
        let when = chrono::Utc.with_ymd_and_hms(2026, 5, 5, 12, 0, 0).unwrap();
        user.deleted_at = Some(when);
        let json = user.to_json().expect("serialize");
        // Field appears in the wire form when populated.
        assert!(json.contains("\"deleted_at\""), "{json}");
        let mut back = User::default();
        back.from_json(&json).expect("deserialize");
        assert_eq!(user, back);
        assert_eq!(back.deleted_at, Some(when));
    }

    #[test]
    fn deleted_at_omitted_from_serialised_form_when_none() {
        // skip_serializing_if = "Option::is_none" keeps the absent case out
        // of the wire form for active users.
        let user = create_valid_user();
        let json = user.to_json().expect("serialize");
        assert!(
            !json.contains("\"deleted_at\""),
            "active user should not carry deleted_at in the wire form: {json}"
        );
    }

    #[test]
    fn can_roundtrip_user_with_multiple_oauth_identities() {
        let mut user = create_valid_user();
        user.oauth_identities.push(OAuthIdentity::new(
            "google".to_string(),
            "google-sub-123".to_string(),
            Some("alice@example.com".to_string()),
        ));
        user.oauth_identities.push(OAuthIdentity::new(
            "github".to_string(),
            "42".to_string(),
            Some("alice@example.com".to_string()),
        ));
        let json = user.to_json().expect("Failed to serialize");
        let mut back = User::default();
        back.from_json(&json).expect("Failed to deserialize");
        assert_eq!(user, back);
        assert_eq!(back.oauth_identities.len(), 2);
        assert_eq!(back.oauth_identities[0].provider, "google");
        assert_eq!(back.oauth_identities[1].provider, "github");
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `logins`\""
    )]
    fn cannot_deserialize_with_missing_logins() {
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","emails":[],"password_hash":"","api_key":"00000000-0000-0000-0000-000000000000"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `id`\""
    )]
    fn cannot_deserialize_with_missing_id() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"is_admin":false,"username":"","full_name":"","emails":[],"password_hash":"","api_key":"00000000-0000-0000-0000-000000000000","logins":[]}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `is_admin`\""
    )]
    fn cannot_deserialize_with_missing_is_admin() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","username":"","full_name":"","emails":[],"password_hash":"","api_key":"00000000-0000-0000-0000-000000000000","logins":[]}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `username`\""
    )]
    fn cannot_deserialize_with_missing_username() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"full_name":"","emails":[],"password_hash":"","api_key":"00000000-0000-0000-0000-000000000000","logins":[]}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `full_name`\""
    )]
    fn cannot_deserialize_with_missing_full_name() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","emails":[],"password_hash":"","api_key":"00000000-0000-0000-0000-000000000000","logins":[]}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `emails`\""
    )]
    fn cannot_deserialize_with_missing_emails() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","password_hash":"","api_key":"00000000-0000-0000-0000-000000000000","logins":[]}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `password_hash`\""
    )]
    fn cannot_deserialize_with_missing_password_hash() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","emails":[],"api_key":"00000000-0000-0000-0000-000000000000","logins":[]}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    #[test]
    #[should_panic(
        expected = "Failed to deserialize: Serialization(Error(\"missing field `api_key`\""
    )]
    fn cannot_deserialize_with_missing_api_key() {
        let compare = User::default();
        let mut loaded = User::default();
        let s = r#"{"id":"00000000-0000-0000-0000-000000000000","is_admin":false,"username":"","full_name":"","emails":[],"password_hash":"","logins":[],"created_at":"2026-01-01T00:00:00Z"}"#;
        loaded.from_json(s).expect("Failed to deserialize");
        assert_eq!(loaded, compare);
    }

    fn create_valid_user() -> User {
        User {
            id: User::new_id(),
            is_admin: false,
            username: "john_smith".to_string(),
            full_name: "John Smith".to_string(),
            emails: vec![UserEmail::new_primary_verified("john_smith@company.com")],
            password_hash: "encrypted_hash".to_string(),
            api_key: User::new_api_key(),
            logins: vec![],
            oauth_identities: vec![],
            deleted_at: None,
            created_at: generate_now(),
            last_sign_in_at: None,
            avatar_updated_at: None,
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
    #[should_panic(expected = "No email address provided for the user")]
    fn user_validation_should_fail_with_missing_email() {
        let mut user = create_valid_user();
        user.emails.clear();
        validate_user(&user);
    }

    #[test]
    #[should_panic(expected = "Invalid email address")]
    fn user_validation_should_fail_with_bad_email() {
        let mut user = create_valid_user();
        user.emails[0].email = "bad_email".to_string();
        validate_user(&user);
    }

    #[test]
    #[should_panic(expected = "Invalid email address")]
    fn user_validation_should_fail_with_email_missing_tld() {
        let mut user = create_valid_user();
        user.emails[0].email = "x@y".to_string();
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

    #[test]
    fn user_validation_should_pass_for_oauth_only_user_without_password() {
        // OAuth-only users carry an empty password_hash. As long as they have
        // at least one OAuth identity attached, validation must succeed.
        let mut user = create_valid_user();
        user.password_hash = "".to_string();
        let primary_email = user.email().to_string();
        user.oauth_identities.push(OAuthIdentity::new(
            "google".to_string(),
            "sub-x".to_string(),
            Some(primary_email),
        ));
        validate_user(&user);
    }

    #[test]
    fn primary_email_returns_primary_row() {
        let user = create_valid_user();
        let primary = user.primary_email().expect("primary expected");
        assert!(primary.is_primary);
        assert_eq!(primary.email, "john_smith@company.com");
    }

    #[test]
    fn primary_email_returns_none_when_no_primary_row() {
        let user = User::default();
        assert!(user.primary_email().is_none());
    }

    #[test]
    fn email_accessor_returns_primary_address() {
        let user = create_valid_user();
        assert_eq!(user.email(), "john_smith@company.com");
    }

    #[test]
    fn email_accessor_returns_empty_when_no_primary() {
        let user = User::default();
        assert_eq!(user.email(), "");
    }

    #[test]
    fn has_verified_email_matches_case_insensitively() {
        let user = create_valid_user();
        assert!(user.has_verified_email("JOHN_smith@Company.COM"));
        assert!(!user.has_verified_email("other@example.com"));
    }

    #[test]
    fn set_primary_email_address_updates_existing_primary() {
        let mut user = create_valid_user();
        user.set_primary_email_address("new@example.com");
        assert_eq!(user.email(), "new@example.com");
        assert_eq!(user.emails.len(), 1);
        assert!(user.emails[0].is_primary);
    }

    #[test]
    fn set_primary_email_address_adds_row_when_no_primary() {
        let mut user = User::default();
        user.set_primary_email_address("new@example.com");
        assert_eq!(user.email(), "new@example.com");
        assert_eq!(user.emails.len(), 1);
        assert!(user.emails[0].is_primary);
        assert!(user.emails[0].verified);
    }

    fn create_valid_user_with_login() -> (User, Uuid) {
        let mut user = create_valid_user();
        let login = user.create_login(1, Some("123.456.789.012".to_string()), Some("Device info string".to_string()));
        (user, login.id)
    }

    #[test]
    fn user_login_token_should_be_found() {
        let (user, login_id) = create_valid_user_with_login();
        assert!(user.contains_valid_login(login_id));
    }

    #[test]
    fn user_login_token_should_not_be_found() {
        let (user, _) = create_valid_user_with_login();
        let bad_login_id = generate_uuid();
        assert!(!user.contains_valid_login(bad_login_id));
    }

    #[test]
    fn user_should_be_able_to_login() {
        let (user, login_id) = create_valid_user_with_login();
        assert!(user.contains_valid_login(login_id));
    }

    #[test]
    fn user_should_be_able_to_logout() {
        let (mut user, login_id) = create_valid_user_with_login();
        assert!(user.contains_valid_login(login_id));
        user.remove_login(login_id);
        assert!(!user.contains_valid_login(login_id));
    }

    #[test]
    fn create_login_purges_expired_logins() {
        let mut user = create_valid_user();

        // Add two expired logins directly
        let expired_login = UserLogin {
            id: generate_uuid(),
            created_at: generate_now() - Duration::hours(48),
            expires_at: generate_now() - Duration::hours(24),
            device_info: None,
            ip_address: None,
        };
        user.logins.push(expired_login.clone());
        user.logins.push(UserLogin {
            id: generate_uuid(),
            created_at: generate_now() - Duration::hours(48),
            expires_at: generate_now() - Duration::hours(24),
            device_info: None,
            ip_address: None,
        });

        // Add one valid login
        let valid_login = user.create_login(24, None, None);
        assert_eq!(user.logins.len(), 1);
        assert!(user.contains_valid_login(valid_login.id));
        assert!(!user.contains_valid_login(expired_login.id));
    }

    #[test]
    fn user_ignore_bad_login_id_on_logout() {
        let (mut user, _) = create_valid_user_with_login();
        let login_count_before = user.logins.len();
        assert_ne!(login_count_before, 0);
        let bad_login_id = generate_uuid();
        user.remove_login(bad_login_id);
        let login_count_after = user.logins.len();
        assert_eq!(login_count_after, login_count_before);
    }

    #[test]
    fn can_renew_login() {
        let (mut user, login_id) = create_valid_user_with_login();
        let original_expiry = user.logins[0].expires_at;
        let renewed = user.renew_login(login_id, 24).expect("Login not found");
        assert!(renewed.expires_at > original_expiry);
        assert_eq!(renewed.id, login_id);
    }

    #[test]
    fn renew_login_returns_none_for_unknown_id() {
        let (mut user, _) = create_valid_user_with_login();
        let unknown_id = generate_uuid();
        assert!(user.renew_login(unknown_id, 24).is_none());
    }

}
