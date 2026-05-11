#[cfg(test)]
mod test_definitions {
    // **************************************************************************************************
    // Unit tests for API and documentation endpoints, via injection of MockStore
    // **************************************************************************************************
    use crate::api::v1::endpoints::auth_reset::{PasswordResetConfirmRequest, PasswordResetRequest};
    use crate::api::v1::endpoints::email_verification::{
        EmailVerificationConfirmRequest, EmailVerificationRequest,
    };
    use crate::api::v1::endpoints::handlers::{get_maze_solve_error_string, get_maze_generate_error_string};
    use crate::api::v1::endpoints::handlers::{AppFeaturesResponse, ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, SignupRequest, UpdateProfileRequest, UserItem, UpdateUserRequest};
    use crate::{create_app, config::app::{AppConfig, AppFeaturesConfig}, oauth::{NoOpConnector, SharedOAuthConnector}, service::notifications::{build_comms, build_default_from, build_renderer}, SharedFeatures};
    use comms::{Comms, StubEmailProvider};
    
    use actix_http;
    use actix_web::{http::StatusCode, test, dev::{Service, ServiceResponse}, web, Error, http::Method};
    use auth::{config::PasswordHashConfig, hashing::hash_password};
    use chrono::{DateTime, Utc};
    use data_model::{Maze, MazeDefinition, MazePoint, User, UserLogin};
    use maze::{Error as MazeError, GenerationAlgorithm, GeneratorOptions, MazePath, MazeSolution, MazeSolver};
    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use tokio::sync::{RwLock as AsyncRwLock, RwLockReadGuard};
    use storage::{Error as StoreError, SharedStore, Store, store::EmailAuditLog, store::MazeStore, store::TokenStore, store::UserStore, store::Manage, MazeItem, validation::validate_user_fields};
    use data_model::{AuditOutcome, EmailAuditEntry, OneTimeToken};
    use uuid::Uuid;

    const ADMIN_USERNAME_PREFIX:&str = "admin_";
    const USERNAME_PREFIX:&str = "user_";
    const VALID_USER_PASSWORD: &str = "Password1!";
    const INVALID_USERNAME: &str = "INVALID_USERNAME";
    const INVALID_EMAIL: &str = "invalid@example.com";
    const INVALID_USER_PASSWORD: &str = "BAD PASSWORD";

    const NEW_ADMIN_USERNAME_1: &str = "new_admin_1";
    const NEW_USERNAME_1: &str = "new_user_1";

    const VALID_ADMIN_USERNAME_1: &str = "admin_1";
    const VALID_ADMIN_USERNAME_2: &str = "admin_2";
    const VALID_USERNAME_1: &str = "user_1";
    const VALID_USERNAME_2: &str = "user_2";
    const VALID_USER_EMAIL_1: &str = "user_1@company.com";

    /**************/
    /* Mock maze  */
    /**************/
    #[derive(Clone, Debug)]
    struct MockMaze {
        id: String,
        name: String,
        maze: Maze,
    }

    impl MockMaze {
        pub fn to_maze_item(&self, include_definitions: bool) -> MazeItem {
            MazeItem {
                id: self.id.clone(),
                name: self.name.clone(),
                definition: if include_definitions {
                    Some(serde_json::to_string(&self.maze.definition).expect("Failed to serialize"))
                } else {
                    None
                },
            }
        }

        fn create_id_from_name(name: &str) -> String {
            format!("{name}.json")
        }
    }

    /**************/
    /* Mock user  */
    /**************/
    #[derive(Clone, Debug)]
   struct MockUser {
        user: User,
        mazes: HashMap<String, MockMaze>,
    }

    impl MockUser {
        fn default() -> MockUser {
            MockUser {
                user: User::default(),
                mazes: HashMap::new(),
            }
        }

        fn to_user_item(&self) -> UserItem {
            UserItem {
                id: self.user.id,
                is_admin: self.user.is_admin,
                username: self.user.username.clone(),
                full_name: self.user.full_name.clone(),
                email: self.user.email().to_string(),
                emails: self.user.emails.clone(),
                has_password: !self.user.password_hash.is_empty(),
            }
        }
        
        fn new_from_user(user: &User) -> Self {
            let mut new_user = user.clone();
            new_user.id = User::new_id();
            new_user.api_key = User::new_api_key();
            MockUser {
                user: new_user,
                mazes: HashMap::new(),
            }
        }        
    } 

    /**************/
    /* Mock store */
    /**************/
    struct MockStore {
        users: HashMap<Uuid, MockUser>,
        tokens: HashMap<Uuid, OneTimeToken>,
        audit_entries: HashMap<Uuid, EmailAuditEntry>,
    }

    impl MockStore {
        pub fn new(user_defs: &Vec<UserDefinition>) -> Self {
            MockStore {
                users: new_users_map(user_defs),
                tokens: HashMap::new(),
                audit_entries: HashMap::new(),
            }
        }

        fn get_mock_user(&self, id: Uuid) -> Result<&MockUser, StoreError> {
            if let Some(mock_user) = self.users.get(&id) {
                return Ok(mock_user);
            }
            Err(StoreError::UserIdNotFound(id.to_string()))
        }

        fn get_mock_user_mut(&mut self, id: Uuid) -> Result<&mut MockUser, StoreError> {
            if let Some(mock_user) = self.users.get_mut(&id) {
                return Ok(mock_user);
            }
            Err(StoreError::UserIdNotFound(id.to_string()))
        }

        /// Find the api key to use for a given username. If the username does not exist,
        /// return an invalid key to simulate an invalid access attempt
        fn get_api_key_to_use(&self, caller_username: Option<&str>) -> Uuid {
            if let Some(username) = caller_username {
                if let Ok(user) = MockStore::find_user_by_name_in_map(&self.users, username, Uuid::nil()) {
                    return user.api_key;
                }
            }
            User::new_api_key()
        }

        fn login_user_by_name_in_map(&mut self, username: &str) -> Result<UserLogin, StoreError> {
            for v in self.users.values_mut() {
                if v.user.username == username {
                    let login = v.user.create_login(24, Some("123.456.789.123".to_string()), Some("Some device information".to_string()));
                    return Ok(login.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }

        fn add_user_login(&mut self, username: Option<&str>) -> Result<Uuid, StoreError> {
            if let Some(username) = username {
                if let Ok(login) = self.login_user_by_name_in_map(username) {
                    return Ok(login.id);        
                }
            }
            Err(StoreError::UserNotFound())
        }

        /// Locates a user in a user map by their username
        fn find_user_by_name_in_map(user_map: &HashMap<Uuid, MockUser>, username: &str, ignore_id: Uuid) -> Result<User, StoreError> {
            for v in user_map.values() {
                if v.user.username == username && v.user.id != ignore_id{
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }    

        /// Locates a user id in a user map by their username
        fn find_user_id_by_name_in_map(user_map: &HashMap<Uuid, MockUser>, username: &str, ignore_id: Uuid) -> Uuid {
            match MockStore::find_user_by_name_in_map(user_map, username, ignore_id) {
                Ok(user) => user.id,
                _ => Uuid::nil(),
            }
        }

        /// Locates a user id in a user map by their username - return nil if it is not found
        fn find_user_id_by_name(&self, username: &str, ignore_id: Uuid) -> Uuid {
            match MockStore::find_user_by_name_in_map(&self.users, username, ignore_id) {
                Ok(user) => user.id,
                _ => Uuid::nil(),
            }
        }

        // Checks whether a given username exists in the file store
        fn user_name_exists(&self, name: &str, ignore_id: Uuid) -> bool {
            self.find_user_id_by_name(name, ignore_id) != Uuid::nil()
        }

        /// Locates a user by their email within the store. Looks across every
        /// row of every user (matching the SQL `user_emails.email` UNIQUE).
        fn find_user_by_email(&self, email: &str, ignore_id: Uuid) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user.id == ignore_id {
                    continue;
                }
                if v.user.emails.iter().any(|row| row.email.eq_ignore_ascii_case(email)) {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }

        // Checks whether a given user email exists in the file store
        fn user_email_exists(&self, email: &str, ignore_id: Uuid) -> bool {
            self.find_user_by_email(email, ignore_id).is_ok()
        }

        // Validate user content
        fn validate_user(&self, user: &User, ignore_id: Uuid) -> Result<(), StoreError> {
            validate_user_fields(user)?;
            // OAuth-only users have an empty password_hash; password-only
            // signup still requires one.
            if user.password_hash.is_empty() && user.oauth_identities.is_empty() {
                return Err(StoreError::UserPasswordMissing());
            }
            if self.user_name_exists(&user.username, ignore_id) {
                return Err(StoreError::UserNameExists());
            }
            for row in &user.emails {
                if self.user_email_exists(&row.email, ignore_id) {
                    return Err(StoreError::UserEmailExists());
                }
            }
            Ok(())
        }
    }

    #[async_trait]
    impl MazeStore for MockStore {

        async fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?;
            let id = MockMaze::create_id_from_name(&maze.name);

            if mock_user.mazes.contains_key(&id) {
                return Err(StoreError::MazeIdExists(id.to_string()));
            }

            maze.id = id.clone();

            mock_user.mazes.insert(
                id.to_string(),
                MockMaze {
                    id,
                    name: maze.name.to_string(),
                    maze: maze.clone(),
            });

            Ok(())
        }

        async fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?;
            if mock_user.mazes.remove(id).is_some() {
                Ok(())
            } else {
                Err(StoreError::MazeIdNotFound(id.to_string()))
            }
        }

        async fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?;
            if mock_user.mazes.contains_key(&maze.id) {
                mock_user.mazes.insert(
                    maze.id.to_string(),
                    MockMaze {
                        id: maze.id.to_string(),
                        name: maze.name.to_string(),
                        maze: maze.clone(),
                });
                return Ok(());
            }
            Err(StoreError::MazeIdNotFound(maze.id.to_string()))
        }

        async fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, StoreError> {
            let mock_user = self.get_mock_user(owner.id)?;
            if let Some(mock_maze) = mock_user.mazes.get(id) {
                return Ok(mock_maze.maze.clone());
            }
            Err(StoreError::MazeIdNotFound(id.to_string()))
        }

        async fn find_maze_by_name(&self, _owner: &User, _name: &str) -> Result<MazeItem, StoreError> {
            Err(StoreError::Other("Mock interface not implemented".to_string()))
        }

        async fn get_maze_items(&self, owner: &User, include_definitions: bool) -> Result<Vec<MazeItem>, StoreError> {
            let mock_user = self.get_mock_user(owner.id)?;
            let mut items: Vec<MazeItem> = maze_items_from_map(&mock_user.mazes, include_definitions);
            items.sort_by_key(|item| item.name.clone());
            Ok(items)
        }
    }

    #[async_trait]
    impl UserStore for MockStore {
        /// Adds the default admin user to the store if it doesn't already exist, else returns it
        async fn init_default_admin_user(&mut self, _username: &str, _email: &str, _password_hash: &str) -> Result<User, StoreError> {
            Err(StoreError::Other("init_default_admin_user() not implemented for MockStore".to_string()))
        }
        /// Adds a new user to the store and sets the allocated `id` within the user object
        async fn create_user(&mut self, user: &mut User) -> Result<(), StoreError> {
            let mock_user = MockUser::new_from_user(user);
            user.id = mock_user.user.id;
            self.validate_user(user, Uuid::nil())?;
            self.users.insert(mock_user.user.id, mock_user);
            Ok(())
        }
        /// Deletes a user from the store
        async fn delete_user(&mut self, id: Uuid) -> Result<(), StoreError> {
            if self.users.remove(&id).is_some() {
                Ok(())
            } else {
                Err(StoreError::UserIdNotFound(id.to_string()))
            }
        }
        /// Purges a user from the store. The MockStore collapses soft-delete
        /// + purge into a single hard-remove because the integration tests
        /// that use this fake exercise endpoint behaviour, not the storage
        /// layer's soft-delete semantics — those are covered by the storage
        /// crate's contract tests.
        async fn purge_user(&mut self, id: Uuid) -> Result<(), StoreError> {
            if id.is_nil() {
                return Err(StoreError::UserIdMissing());
            }
            if self.users.remove(&id).is_some() {
                Ok(())
            } else {
                Err(StoreError::UserIdNotFound(id.to_string()))
            }
        }
        /// Updates a user within the store
        async fn update_user(&mut self, user: &mut User) -> Result<(), StoreError> {
            self.validate_user(user, user.id)?;
            let mock_user = self.get_mock_user_mut(user.id)?;
            mock_user.user = user.clone();
            Ok(())
        }
        /// Loads a user from the store
        async fn get_user(&self, id: Uuid) -> Result<User, StoreError> {
            if let Some(mock_user) = self.users.get(&id) {
                return Ok(mock_user.user.clone());
            }
            Err(StoreError::UserIdNotFound(id.to_string()))
        }
        /// Locates a user by their username within the store
        async fn find_user_by_name(&self, name: &str) -> Result<User, StoreError> {
            MockStore::find_user_by_name_in_map(&self.users, name, Uuid::nil())
        }
        /// Locates a user by an email address within the store, returning
        /// the match only if the matching email row is verified. Mirrors
        /// the verified-only filter enforced by the real stores.
        async fn find_user_by_verified_email(&self, email: &str) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user
                    .emails
                    .iter()
                    .any(|row| row.verified && row.email.eq_ignore_ascii_case(email))
                {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }
        /// Locates a user by an email address regardless of verification
        /// state. Mirrors the real stores; used by the OAuth squat-reclaim
        /// path to inspect whether a colliding email belongs to a real
        /// account or a squatter.
        async fn find_user_by_email_any_state(&self, email: &str) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user
                    .emails
                    .iter()
                    .any(|row| row.email.eq_ignore_ascii_case(email))
                {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }
        /// Locates a user by their api key within the store
        async fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user.api_key == api_key {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }

        async fn find_user_by_login_id(&self, login_id: Uuid) -> Result<User, StoreError>{
            for v in self.users.values() {
                if v.user.contains_valid_login(login_id) {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }

        async fn find_user_by_oauth_identity(&self, provider: &str, provider_user_id: &str) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user.oauth_identities.iter().any(|i| {
                    i.provider.eq_ignore_ascii_case(provider) && i.provider_user_id == provider_user_id
                }) {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }
        /// Returns the list of users within the store, sorted
        /// alphabetically by username in ascending order
        async fn get_users(&self) -> Result<Vec<User>, StoreError> {
            let mut users: Vec<User> = self.users.values()
                .map( |value| value.user.clone())
                .collect();

            users.sort_by_key(|user| user.username.clone());
            Ok(users)
        }

        /// Returns the list of admin users within the store
        async fn get_admin_users(&self) -> Result<Vec<User>, StoreError> {
            let admins: Vec<User> = self.users.values()
                .filter(|v| v.user.is_admin)
                .map(|v| v.user.clone())
                .collect();
            Ok(admins)
        }

        async fn has_users(&self) -> Result<bool, StoreError> {
            Ok(!self.users.is_empty())
        }

        async fn has_active_admin_user(&self) -> Result<bool, StoreError> {
            Ok(self
                .users
                .values()
                .any(|v| v.user.is_admin && v.user.is_active()))
        }

        async fn add_user_email(
            &mut self,
            user_id: Uuid,
            email: &str,
            verified: bool,
        ) -> Result<data_model::UserEmail, StoreError> {
            if email.trim().is_empty() {
                return Err(StoreError::UserEmailMissing());
            }
            if !data_model::is_valid_email_format(email) {
                return Err(StoreError::UserEmailInvalid());
            }
            // Same-user duplicate.
            let existing = self.get_mock_user(user_id)?;
            if existing.user.emails.iter().any(|r| r.email.eq_ignore_ascii_case(email)) {
                return Err(StoreError::UserEmailExists());
            }
            // Cross-user duplicate.
            for v in self.users.values() {
                if v.user.id == user_id { continue; }
                if v.user.emails.iter().any(|r| r.email.eq_ignore_ascii_case(email)) {
                    return Err(StoreError::UserEmailExists());
                }
            }
            let row = data_model::UserEmail {
                email: email.to_string(),
                is_primary: false,
                verified,
                verified_at: if verified {
                    Some(chrono::Utc::now().with_timezone(&chrono::Utc))
                } else {
                    None
                },
            };
            let mock_user = self.get_mock_user_mut(user_id)?;
            mock_user.user.emails.push(row.clone());
            Ok(row)
        }

        async fn remove_user_email(
            &mut self,
            user_id: Uuid,
            email: &str,
        ) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(user_id)?;
            let idx = find_email_row_index(&mock_user.user, email)?;
            if mock_user.user.emails.len() == 1 {
                return Err(StoreError::UserEmailIsLast());
            }
            if mock_user.user.emails[idx].is_primary {
                return Err(StoreError::UserEmailIsPrimary());
            }
            mock_user.user.emails.remove(idx);
            // Mirror the production stores: drop OAuth identities whose
            // `provider_email` matches the removed address. See the trait
            // doc on `UserStore::remove_user_email`.
            mock_user
                .user
                .oauth_identities
                .retain(|id| match id.provider_email.as_deref() {
                    Some(addr) => !addr.eq_ignore_ascii_case(email),
                    None => true,
                });
            Ok(())
        }

        async fn set_primary_email(
            &mut self,
            user_id: Uuid,
            email: &str,
        ) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(user_id)?;
            let idx = find_email_row_index(&mock_user.user, email)?;
            if !mock_user.user.emails[idx].verified {
                return Err(StoreError::UserEmailNotVerified());
            }
            for (i, row) in mock_user.user.emails.iter_mut().enumerate() {
                row.is_primary = i == idx;
            }
            Ok(())
        }

        async fn mark_email_verified(
            &mut self,
            user_id: Uuid,
            email: &str,
        ) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(user_id)?;
            let idx = find_email_row_index(&mock_user.user, email)?;
            mock_user.user.emails[idx].verified = true;
            mock_user.user.emails[idx].verified_at = Some(chrono::Utc::now());
            Ok(())
        }
    }

    /// Locates the index of the email row matching `email` (case-insensitively)
    /// within `user.emails`, or returns `UserEmailNotFound`. Centralises the
    /// lookup that every email-mutating MockStore method runs before deciding
    /// the action — mirrors the `find_email_row_index` helper in `file_store.rs`.
    fn find_email_row_index(user: &User, email: &str) -> Result<usize, StoreError> {
        user.emails
            .iter()
            .position(|r| r.email.eq_ignore_ascii_case(email))
            .ok_or_else(|| StoreError::UserEmailNotFound(email.to_string()))
    }

    #[async_trait]
    impl Manage for MockStore {
        async fn empty(&mut self) -> Result<(), StoreError> {
            self.users = HashMap::new();
            Ok(())
        }
    }

    // In-memory TokenStore that mirrors the real storage backends'
    // behavioural contract closely enough for handler-level integration
    // tests. The `consume_token` race-free guarantee is provided by the
    // outer `RwLock<Box<dyn Store>>` + the per-method `&mut self` — at
    // that point the test runner already holds an exclusive lock so no
    // two `consume_token` calls can interleave.
    #[async_trait]
    impl TokenStore for MockStore {
        async fn create_token(&mut self, token: &OneTimeToken) -> Result<(), StoreError> {
            if token.id == Uuid::nil() {
                return Err(StoreError::Other("token id must not be nil".to_string()));
            }
            if self.tokens.contains_key(&token.id) {
                return Err(StoreError::TokenIdExists(token.id.to_string()));
            }
            self.tokens.insert(token.id, token.clone());
            Ok(())
        }
        async fn find_token(&self, id: Uuid) -> Result<OneTimeToken, StoreError> {
            match self.tokens.get(&id) {
                Some(t) if t.is_expired() => Err(StoreError::TokenIdNotFound(id.to_string())),
                Some(t) => Ok(t.clone()),
                None => Err(StoreError::TokenIdNotFound(id.to_string())),
            }
        }
        async fn consume_token(&mut self, id: Uuid) -> Result<OneTimeToken, StoreError> {
            let token = self.tokens.get_mut(&id)
                .ok_or_else(|| StoreError::TokenIdNotFound(id.to_string()))?;
            if token.is_consumed() {
                return Err(StoreError::TokenAlreadyConsumed());
            }
            if token.is_expired() {
                return Err(StoreError::TokenExpired());
            }
            token.consumed_at = Some(Utc::now());
            Ok(token.clone())
        }
        async fn purge_email_verification_tokens(
            &mut self,
            user_id: Uuid,
            target_email: &str,
        ) -> Result<u64, StoreError> {
            let before = self.tokens.len() as u64;
            self.tokens.retain(|_, t| {
                !(t.user_id == user_id
                    && t.purpose == data_model::TokenPurpose::EmailVerification
                    && t.target_email
                        .as_deref()
                        .map(|s| s.eq_ignore_ascii_case(target_email))
                        .unwrap_or(false))
            });
            Ok(before - self.tokens.len() as u64)
        }
        async fn purge_expired(&mut self) -> Result<u64, StoreError> {
            let before = self.tokens.len() as u64;
            self.tokens.retain(|_, t| !t.is_expired() || t.is_consumed());
            Ok(before - self.tokens.len() as u64)
        }
    }

    // In-memory EmailAuditLog. Mirrors the real backends' contract closely
    // enough for handler-level integration tests — happy path, recon row,
    // and provider-failure assertions all need to read back rows the
    // dispatch helpers wrote.
    #[async_trait]
    impl EmailAuditLog for MockStore {
        async fn record_pending(&mut self, entry: &EmailAuditEntry) -> Result<Uuid, StoreError> {
            if entry.id == Uuid::nil() {
                return Err(StoreError::Other("audit entry id must not be nil".to_string()));
            }
            if self.audit_entries.contains_key(&entry.id) {
                return Err(StoreError::AuditEntryIdExists(entry.id.to_string()));
            }
            self.audit_entries.insert(entry.id, entry.clone());
            Ok(entry.id)
        }
        async fn update_outcome(
            &mut self,
            id: Uuid,
            outcome: AuditOutcome,
            provider_message_id: Option<&str>,
            error_class: Option<&str>,
            error_message: Option<&str>,
        ) -> Result<(), StoreError> {
            if matches!(outcome, AuditOutcome::Pending) {
                return Err(StoreError::Other(
                    "update_outcome cannot move a row back to pending".to_string(),
                ));
            }
            let row = self.audit_entries.get_mut(&id)
                .ok_or_else(|| StoreError::AuditEntryIdNotFound(id.to_string()))?;
            row.outcome = outcome;
            row.provider_message_id = provider_message_id.map(|s| s.to_string());
            row.error_class = error_class.map(|s| s.to_string());
            row.error_message = error_message.map(|s| s.to_string());
            Ok(())
        }
        async fn find_audit_entry(&self, id: Uuid) -> Result<EmailAuditEntry, StoreError> {
            self.audit_entries
                .get(&id)
                .cloned()
                .ok_or_else(|| StoreError::AuditEntryIdNotFound(id.to_string()))
        }
        async fn find_recent_audit_entries_for_user(
            &self,
            user_id: Uuid,
            limit: u32,
        ) -> Result<Vec<EmailAuditEntry>, StoreError> {
            let mut matches: Vec<EmailAuditEntry> = self
                .audit_entries
                .values()
                .filter(|e| e.recipient_user_id == Some(user_id))
                .cloned()
                .collect();
            matches.sort_by(|a, b| {
                b.created_at
                    .cmp(&a.created_at)
                    .then_with(|| b.id.cmp(&a.id))
            });
            matches.truncate(limit as usize);
            Ok(matches)
        }
    }

    impl Store for MockStore {}

    /****************/
    /* Mock content */
    /****************/
    #[derive(Clone)]
    enum MazeContent {
        Empty,
        OneMaze,
        TwoMazes,
        ThreeMazes,
        SolutionTestMazes,
    }

    fn maze_store_mock_mazes_to_maze_items(from: Vec<MockMaze>, include_definitions: bool) -> Vec<MazeItem> {
        from.iter()
            .map( |value| value.to_maze_item(include_definitions))
            .collect()
    }

    fn mazes_to_map(mazes: &Vec<MockMaze>) -> HashMap<String, MockMaze> {
        let mut map: HashMap<String, MockMaze> = HashMap::new();
        for maze in mazes {
            map.insert(maze.id.clone(), maze.clone());
        }
        map
    }

    fn maze_items_from_map(from: &HashMap<String, MockMaze>, include_definitions: bool) -> Vec<MazeItem> {
        from.values().map(|value| MazeItem {
                    id: value.id.clone(),
                    name: value.name.clone(),
                    definition: if include_definitions {
                        Some(serde_json::to_string(&value.maze.definition).expect("Failed to serialize"))
                    } else {
                        None
                    },
            })
            .collect()
    }

    fn new_solvable_maze(id: &str, name: &str) -> Maze {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', 'W', ' ', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', 'F'],
            vec!['W', ' ', 'W', ' ', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec!['W', 'W', ' ', ' ', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
        ];
        let mut maze:Maze = Maze::new(MazeDefinition::from_vec(grid));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
    }

    fn new_solvable_maze_store_item(id: &str, name: &str) -> MockMaze {
        MockMaze {
            id: id.to_string(),
            name: name.to_string(),
            maze: new_solvable_maze(id, name),
        }
    }

    fn new_solve_test_maze(id: &str, name: &str, with_start: bool, with_finish: bool, with_block: bool) -> Maze {
        let start_char:char = if with_start {'S'} else {' '};
        let finish_char:char = if with_finish {'F'} else {' '};
        let block_char:char = if with_block {'W'} else {' '};

        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![start_char, 'W', ' '],
            vec![' ', 'W', ' '],
            vec![' ', block_char, finish_char],
        ];
        let mut maze:Maze = Maze::new(MazeDefinition::from_vec(grid));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
    }

    fn get_solve_test_maze_solution() -> MazeSolution {
        let path = MazePath {
            points: vec![
                MazePoint { row: 0, col: 0 },
                MazePoint { row: 1, col: 0 },
                MazePoint { row: 2, col: 0 },
                MazePoint { row: 2, col: 1 },
                MazePoint { row: 2, col: 2 },
            ],
        };
        MazeSolution::new(path)
    }


    fn new_solve_test_maze_store_item(id: &str, name: &str, with_start: bool, with_finish: bool, with_block: bool) -> MockMaze {
        MockMaze {
            id: id.to_string(),
            name: name.to_string(),
            maze: new_solve_test_maze(id, name, with_start, with_finish, with_block),
        }
    }

    fn get_maze_content(maze_content: MazeContent, sort_asc: bool) -> Vec<MockMaze> {
        let mut result: Vec<MockMaze>;
        match maze_content {
            MazeContent::Empty => {
                result = Vec::new();
            }
            MazeContent::OneMaze => {
                result = vec![
                    new_solvable_maze_store_item("maze_a.json", "maze_a")
                ]
            }
            MazeContent::TwoMazes => {
                result = vec![
                    new_solvable_maze_store_item("maze_b.json", "maze_b"),
                    new_solvable_maze_store_item("maze_a.json", "maze_a"),
                ]
            }
            MazeContent::ThreeMazes => {
                result = vec![
                    new_solvable_maze_store_item("maze_c.json", "maze_c"),
                    new_solvable_maze_store_item("maze_b.json", "maze_b"),
                    new_solvable_maze_store_item("maze_a.json", "maze_a"),
                ]
            }
            MazeContent::SolutionTestMazes => {
                result = vec![
                    new_solve_test_maze_store_item("solvable.json", "solvable", true, true, false),
                    new_solve_test_maze_store_item("no_start.json", "no_start", false, true, false),
                    new_solve_test_maze_store_item("no_finish.json", "no_finish", true, false, false),
                    new_solve_test_maze_store_item("no_solution.json", "no_solution", true, true, true),
                ]
            }
        }

        if sort_asc {
            result.sort_by_key(|item| item.name.clone());
        }

        result
    }

    fn new_mazes_map(maze_content: MazeContent) -> HashMap<String, MockMaze> {
        mazes_to_map(&get_maze_content(maze_content, false))
    }

    fn new_user(username: &str, is_admin: bool, password_hash: &str) -> User {
        let mut user = User::default();
        user.id = User::new_id();
        user.username = username.to_string();
        user.is_admin = is_admin;
        user.api_key = User::new_api_key();
        user.set_primary_email_address(&new_email(username));
        user.password_hash = password_hash.to_string();
        user
    }

    fn new_email(username: &str) -> String {
        format!("{username}@company.com")
    }

    #[derive(Clone)]
    struct UserDefinition {
        username: String,
        is_admin: bool,
        password_hash: String,
        mazes: MazeContent,
    }

    fn append_user_defs(user_defs: &mut Vec<UserDefinition>, num: i32, is_admin: bool, password_hash: &str, mazes: &MazeContent) {
        let username_prefix = if is_admin { ADMIN_USERNAME_PREFIX } else { USERNAME_PREFIX};
        for i in 1..(num+1) {
            user_defs.push( UserDefinition {
                username: format!("{username_prefix}{i}"),
                is_admin,
                password_hash: password_hash.to_string(), 
                mazes: mazes.clone(),
            });
        }
    }

    struct CreateUsersDef {
        num_admin_users: i32,
        num_users: i32,
        mazes: MazeContent,
    }

    impl CreateUsersDef {
        pub fn new(
            num_admin_users: i32,
            num_users: i32,
            mazes: MazeContent
        ) -> Self {
            CreateUsersDef {
                num_admin_users,
                num_users,
                mazes: mazes.clone(),
            }
        }    
    }

    fn create_user_defs(def: &CreateUsersDef) -> Vec<UserDefinition> {
        let mut user_defs = vec![];
        append_user_defs(&mut user_defs, def.num_users, false, "", &def.mazes);
        append_user_defs(&mut user_defs, def.num_admin_users, true, "", &def.mazes);
        user_defs
    }

    fn new_mock_user(user_def: &UserDefinition) -> MockUser {
        let user =  new_user(&user_def.username, user_def.is_admin, &user_def.password_hash);
        MockUser {
            user,
            mazes: new_mazes_map(user_def.mazes.clone()),
        }
    }

    fn new_shared_mock_maze_store(mock_store: MockStore) -> SharedStore {
        Arc::new(AsyncRwLock::new(Box::new(mock_store)))
    }

    fn new_users_map(user_defs:&Vec<UserDefinition>) -> HashMap<Uuid, MockUser> {
        let mut map: HashMap<Uuid, MockUser> = HashMap::new();
        for user_def in user_defs {
            let mock_user = new_mock_user(user_def);
            map.insert(mock_user.user.id, mock_user);
        }
        map
    }

    fn maze_store_mock_users_to_user_items(from: &HashMap<Uuid, MockUser>) -> Vec<UserItem> {
        let mut users: Vec<UserItem> = from.values()
            .map( |value| value.to_user_item())
            .collect();

       users.sort_by_key(|user| user.username.clone());
       users
    }

    fn create_test_request<T: Serialize>(
        method: Method,
        url: &str,
        api_key: Option<Uuid>,
        login_id: Option<Uuid>,
        json_body: Option<&T>,
    ) -> actix_http::Request {
        let mut req = test::TestRequest::default()
            .method(method)
            .uri(url);

        if let Some(login_id) = login_id {
            req = req.insert_header(("Authorization", format!("Bearer {login_id}")));
        }
        else if  let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }    

        if let Some(body) = json_body {
            req = req.set_json(body);
        }

        req.to_request()
    }    

    fn create_test_get_request(url: &str, api_key: Option<Uuid>, login_id: Option<Uuid>) -> actix_http::Request {
        create_test_request(Method::GET, url, api_key, login_id, None::<&()>)
    }

    fn create_test_post_request<T: serde::Serialize>(url: &str, api_key: Option<Uuid>, login_id: Option<Uuid>, body_obj: Option<&T>) -> actix_http::Request {
        create_test_request(Method::POST, url, api_key, login_id, body_obj)
    }

    fn create_test_put_request<T: serde::Serialize>(url: &str, api_key: Option<Uuid>, login_id: Option<Uuid>, body_obj: &T) -> actix_http::Request {
        create_test_request(Method::PUT, url, api_key, login_id, Some(body_obj))
    }

    fn create_test_delete_request(url: &str, api_key: Option<Uuid>, login_id: Option<Uuid>) -> actix_http::Request {
        create_test_request(Method::DELETE, url, api_key, login_id, None::<&()>)
    }

    fn create_shared_mock_store(
        user_defs:&Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
     ) -> (SharedStore, HashMap<Uuid, MockUser>, Uuid, Option<Uuid>) {
        let mut mock_store = MockStore::new(user_defs);
        let api_key = mock_store.get_api_key_to_use(caller_username);
        let mut login_id = None;
        if add_login {
            if let Ok(user_login_id) = mock_store.add_user_login(caller_username) {
                login_id = Some(user_login_id);
            }
        }
        let mock_users = mock_store.users.clone();
        let shared_mock_store = new_shared_mock_maze_store(mock_store);
        (shared_mock_store, mock_users, api_key, login_id)
    }

    fn set_valid_password_hashes(hash_config: &PasswordHashConfig, user_defs: &mut Vec<UserDefinition>) {
        let password_hash = match hash_password(VALID_USER_PASSWORD, hash_config) {
            Ok(hash) => hash,
            Err(_) => "".to_string(),            
        };
        for user_def in user_defs {
            user_def.password_hash = password_hash.to_string();
        }    
    }

    async fn create_test_app_with_config(
        user_defs: &mut Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
        features: SharedFeatures,
        app_config: AppConfig,
    ) -> (impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>, SharedStore, HashMap<Uuid, MockUser>, Option<Uuid>, Option<Uuid>) {
        set_valid_password_hashes(&app_config.security.password_hash, user_defs);

        let (shared_mock_store, mock_users, api_key, login_id) = create_shared_mock_store(user_defs, caller_username, add_login);
        let connector: SharedOAuthConnector = Arc::new(NoOpConnector);
        let comms = web::Data::new(build_comms(&app_config.comms).expect("test comms"));
        let app = test::init_service(
            create_app(&app_config.security.password_hash, web::Data::new(shared_mock_store.clone()), web::Data::new(features), web::Data::new(connector), comms, ".".to_string())
            .app_data(web::Data::new(app_config))
        )
        .await;

        (app, shared_mock_store, mock_users, Some(api_key), login_id)
    }

    async fn create_test_app_with_features(
        user_defs: &mut Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
        features: SharedFeatures,
    ) -> (impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>, SharedStore, HashMap<Uuid, MockUser>, Option<Uuid>, Option<Uuid>) {
        let mut config = AppConfig::default();
        config.security.password_hash = auth::config::PasswordHashConfig::for_testing();
        // Default the test app to a production-like comms state (`enabled = true`)
        // so the credentials sign-up + add-email paths behave as they do in
        // deployed servers — newly-created email rows are unverified pending
        // user click-through. Tests that need to drive the comms-disabled
        // branch (auto-verify + skip dispatch) construct their own AppConfig
        // and call `create_test_app_with_config` directly.
        config.comms.enabled = true;
        create_test_app_with_config(user_defs, caller_username, add_login, features, config).await
    }

    async fn create_test_app(
        user_defs: &mut Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
    ) -> (impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>, SharedStore, HashMap<Uuid, MockUser>, Option<Uuid>, Option<Uuid>) {
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        create_test_app_with_features(user_defs, caller_username, add_login, features).await
    }

    /// Variant of `create_test_app` that wires a freshly-constructed
    /// `StubEmailProvider` into the `Comms` orchestrator and returns a
    /// clone alongside the app so the test can inspect captured sends.
    /// Used by the password-reset / email-verification integration tests
    /// that need to assert on outbound emails.
    async fn create_test_app_with_stub_email(
        user_defs: &mut Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
    ) -> (
        impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>,
        SharedStore,
        HashMap<Uuid, MockUser>,
        Option<Uuid>,
        Option<Uuid>,
        StubEmailProvider,
    ) {
        create_test_app_with_stub_email_and_comms_enabled(
            user_defs, caller_username, add_login, true,
        )
        .await
    }

    /// Like `create_test_app_with_stub_email` but lets the test override
    /// `app_config.comms.enabled`. Used by the new gating tests that need
    /// to drive the `comms.enabled = false` branch (auto-verify on user
    /// creation, skip verification dispatch).
    async fn create_test_app_with_stub_email_and_comms_enabled(
        user_defs: &mut Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
        comms_enabled: bool,
    ) -> (
        impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>,
        SharedStore,
        HashMap<Uuid, MockUser>,
        Option<Uuid>,
        Option<Uuid>,
        StubEmailProvider,
    ) {
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let mut app_config = AppConfig::default();
        app_config.security.password_hash = auth::config::PasswordHashConfig::for_testing();
        app_config.comms.enabled = comms_enabled;
        // Drive a stable `public_base_url` so the reset-link assertion has
        // something deterministic to check against. Likewise pin a `from`
        // — without it, `comms.send_template` fails the dispatch with
        // `Config("no default_from_email configured")` and the stub
        // never sees a message.
        app_config.comms.public_base_url = "https://maze.test".to_string();
        app_config.comms.email.from = "noreply@maze.test".to_string();

        set_valid_password_hashes(&app_config.security.password_hash, user_defs);
        let (shared_mock_store, mock_users, api_key, login_id) =
            create_shared_mock_store(user_defs, caller_username, add_login);
        let connector: SharedOAuthConnector = Arc::new(NoOpConnector);

        let renderer = build_renderer(&app_config.comms).expect("test renderer");
        let default_from = build_default_from(&app_config.comms);
        let stub = StubEmailProvider::new();
        let comms = Comms::new(renderer, Some(Arc::new(stub.clone())), default_from);

        let app = test::init_service(
            create_app(
                &app_config.security.password_hash,
                web::Data::new(shared_mock_store.clone()),
                web::Data::new(features),
                web::Data::new(connector),
                web::Data::new(comms),
                ".".to_string(),
            )
            .app_data(web::Data::new(app_config)),
        )
        .await;

        (app, shared_mock_store, mock_users, Some(api_key), login_id, stub)
    }

    fn get_invalid_email_or_password_error_str() -> String {
        "Invalid email or password".to_string()
    }

    fn get_email_and_password_must_be_provided_error_str() -> String {
        "Email and password must be provided".to_string()
    }

    async fn get_store_read_lock(
        shared_store: &Arc<AsyncRwLock<Box<dyn Store>>>,
    ) -> RwLockReadGuard<'_, Box<dyn Store>> {
        shared_store.read().await
    }

    async fn verify_user_login_presence(shared_store: &Arc<AsyncRwLock<Box<dyn Store>>>, email: &str, login_id: Uuid, expected_presence: bool) {
        let store_lock = get_store_read_lock(shared_store).await;
        // Confirm login id associated with user
        match store_lock.find_user_by_verified_email(email).await {
            Ok(user) => {
                let presence = user.contains_valid_login(login_id);
                if presence != expected_presence {
                    panic!("{}", format!("User contains_login() returned an unexpected value (expected = {expected_presence}, returned = {presence})"));
                }
            },
            Err(err) => panic!("{}", format!("Failed to locate user for login id = {login_id} => {err}"))
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_login_logout_test(
        create_users_def: &CreateUsersDef,
        email: &str,
        password: &str,
        expected_login_status_code: StatusCode,
        expected_login_err_message: Option<String>,
        run_logout_test: bool,
        set_logout_login_id: bool,
        expected_logout_status_code: Option<StatusCode>,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, shared_store, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let login_url = "/api/v1/login".to_string();
        let login_request = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };
        let login_req = create_test_post_request(&login_url, None, None, Some(&login_request));
        let login_resp = test::call_service(&app, login_req).await;

        assert_eq!(login_resp.status(), expected_login_status_code);

        if expected_login_status_code == StatusCode::OK {
            let login_resp_body = test::read_body(login_resp).await;
            let login_response: LoginResponse = serde_json::from_slice(&login_resp_body).expect("failed to deserialize login response");
            let login_id = login_response.login_token_id;
            assert_ne!(login_id, Uuid::nil());
            assert_ne!(login_response.login_token_expires_at, DateTime::<Utc>::default());
            // Fresh users created by the test helper have `last_sign_in_at = None`
            // and `logins = []`, so their first successful login through this
            // helper should always report the welcome-banner trigger.
            assert!(
                login_response.is_first_sign_in,
                "fresh user's first login must report is_first_sign_in = true"
            );

            if run_logout_test {
                verify_user_login_presence(&shared_store, email, login_id, true).await;

                // Logout
                let logout_url = "/api/v1/logout".to_string();
                let logout_login_id = set_logout_login_id.then_some(login_id);
                let logout_req = create_test_post_request(&logout_url, None, logout_login_id, None::<&()>);
                let logout_resp = test::call_service(&app, logout_req).await;

                if let Some(expected_logout_status_code) = expected_logout_status_code {
                    assert_eq!(logout_resp.status(), expected_logout_status_code);
                    if expected_logout_status_code == StatusCode::NO_CONTENT {
                        verify_user_login_presence(&shared_store, email, login_id, false).await;
                    }
                }
            }

        } else {
            match expected_login_err_message {
                Some(value) => {
                    // Validate error response
                    let login_resp_body = test::read_body(login_resp).await;
                    let error_message = String::from_utf8(login_resp_body.to_vec()).expect("Failed to parse login response body as UTF-8");
                    assert_eq!(error_message, value);
                }
                None => { panic!("No error message provided for login test!"); }
            }
        }
    }

    async fn run_get_users_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let path_str = "/api/v1/users".to_string();
        let req = create_test_get_request(&path_str, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);
        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let user_items: Vec<UserItem> = serde_json::from_slice(&body).expect("failed to deserialize response");
            let expected_user_items = maze_store_mock_users_to_user_items(&mock_users);
            assert_eq!(user_items, expected_user_items);
        } 
    }

    impl CreateUserRequest {
        pub fn new(
            is_admin: bool,
            username: &str,
            full_name: &str,
            email: &str,
            password: &str
        ) -> CreateUserRequest {
            CreateUserRequest {
                is_admin,
                username: username.to_string(),
                full_name: full_name.to_string(),
                email: email.to_string(),
                password: password.to_string(),
            }
        }

        pub fn to_user_item(&self) -> UserItem {
            UserItem {
                id: Uuid::nil(),
                is_admin: self.is_admin,
                username: self.username.clone(),
                full_name: self.full_name.clone(),
                email: self.email.clone(),
                emails: vec![data_model::UserEmail::new_primary_verified(&self.email)],
                has_password: true,
            }
        }

    }    

    fn create_password(blank: bool) -> String {
        if blank {
            "".to_string()
        } else {
            "Password1!".to_string()
        }
    }

    fn new_create_user_request(is_admin: bool, username: &str, email: Option<&str>, blank_password: bool) -> CreateUserRequest {
        let email_use = if let Some(s) = email {
            s
        } else {
            &new_email(username)
        };

        CreateUserRequest::new(is_admin, username, 
            &format!("{username} full name"), 
            email_use, 
            &create_password(blank_password) 
        )    
    }

    async fn run_create_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        create_req: &CreateUserRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users".to_string();
        let req = create_test_post_request(&url, api_key, login_id, Some(&create_req));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::CREATED {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize response");
            let mut expected_user_response = create_req.to_user_item();
            expected_user_response.id = response_user.id;
            // `emails` carries a `verified_at` timestamp set at write time
            // by the store; the expected value built from the request can't
            // know the exact instant. Copy it from the response, then assert
            // the rest. The `emails` content is checked separately below.
            expected_user_response.emails = response_user.emails.clone();
            assert_eq!(expected_user_response, response_user);
            // Spot-check the emails shape (independent of timestamp).
            assert_eq!(response_user.emails.len(), 1);
            assert_eq!(response_user.emails[0].email, response_user.email);
            assert!(response_user.emails[0].is_primary);
            assert!(response_user.emails[0].verified);
        }
    }

    async fn run_get_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        target_username: &str,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let id = MockStore::find_user_id_by_name_in_map(&mock_users, target_username, Uuid::nil());
        let url = format!("/api/v1/users/{id}");
        let req = create_test_get_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize response");
            let dummy_user = MockUser::default();
            let expected_user = mock_users.get(&id).unwrap_or(&dummy_user);
            let expected_user_response = expected_user.to_user_item();
            assert_eq!(expected_user_response, response_user);
        }
    }

    impl UpdateUserRequest {
        pub fn new(
            is_admin: bool,
            username: &str,
            full_name: &str,
            email: &str
        ) -> UpdateUserRequest {
            UpdateUserRequest {
                is_admin,
                username: username.to_string(),
                full_name: full_name.to_string(),
                email: email.to_string()
            }
        }

        pub fn to_user_item(&self) -> UserItem {
            UserItem {
                id: Uuid::nil(),
                is_admin: self.is_admin,
                username: self.username.clone(),
                full_name: self.full_name.clone(),
                email: self.email.clone(),
                emails: vec![data_model::UserEmail::new_primary_verified(&self.email)],
                has_password: true,
            }
        }

    }    

    fn new_update_user_request(is_admin: bool, username: &str, email: Option<&str>) -> UpdateUserRequest {
        let email_use = if let Some(s) = email {
            s
        } else {
            &new_email(username)
        };

        UpdateUserRequest::new(is_admin, username, 
            &format!("Updated {username} full name"), 
            email_use
        )    
    }    

    async fn run_update_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        target_username: &str,
        update_req: &UpdateUserRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let id = MockStore::find_user_id_by_name_in_map(&mock_users, target_username, Uuid::nil());
        let url = format!("/api/v1/users/{id}");
        let req = create_test_put_request(&url, api_key, login_id, &update_req);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize response");
            let mut expected_response_user = update_req.to_user_item();
            expected_response_user.id = response_user.id;
            // emails carries a `verified_at` timestamp set at write time;
            // copy it across before asserting equality. See the equivalent
            // note in run_create_user_test.
            expected_response_user.emails = response_user.emails.clone();
            assert_eq!(expected_response_user, response_user);
        }
    }


    async fn run_delete_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        target_username: &str,
        expected_status_code: StatusCode
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let id = MockStore::find_user_id_by_name_in_map(&mock_users, target_username, Uuid::nil());
        let url = format!("/api/v1/users/{id}");
        let req = create_test_delete_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            if Some(target_username) == caller_username {
                return;
            }

            // Confirm it has been deleted
            let url2 = format!("/api/v1/users/{id}");
            let req2 = create_test_get_request(&url2, api_key, None);
            let resp2 = test::call_service(&app, req2).await;
            assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
        }
    }

    async fn run_get_mazes_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        include_definitions: bool,
        expected_maze_content:MazeContent
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let path_str = format!("/api/v1/mazes?includeDefinitions={include_definitions}");
        let req = create_test_get_request(&path_str, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let maze_items: Vec<MazeItem> = serde_json::from_slice(&body).expect("failed to deserialize response");
        assert_eq!(
            maze_items,
            maze_store_mock_mazes_to_maze_items(get_maze_content(expected_maze_content, true), include_definitions)
        );
    }

    async fn run_create_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        maze: Maze,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/mazes".to_string();
        let req = create_test_post_request(&url, api_key, login_id, Some(&maze));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::CREATED {
            let body = test::read_body(resp).await;
            let response_maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            let mut maze_copy = maze.clone();
            maze_copy.id = MockMaze::create_id_from_name(&maze.name);
            assert_eq!(maze_copy, response_maze);
        }
    }

    async fn run_get_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        id: &str,
        expected_status_code: StatusCode,
        expected_maze: Option<Maze>
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = format!("/api/v1/mazes/{id}");
        let req = create_test_get_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Verify content
            let body = test::read_body(resp).await;
            let maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            match expected_maze {
                Some(value) => { assert_eq!(maze, value); }
                None => { panic!("No maze comparison value provided for get_maze() test!"); }
            }
        }
    }

    async fn run_update_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        id: &str,
        maze: Maze,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = format!("/api/v1/mazes/{id}");
        let req = create_test_put_request(&url,api_key, login_id, &maze);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            assert_eq!(maze, response_maze);
        }
    }

    async fn run_delete_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        id: &str,
        expected_status_code: StatusCode
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = format!("/api/v1/mazes/{id}");
        let req = create_test_delete_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Confirm it has been deleted
            let url2 = format!("/api/v1/mazes/{id}");
            let req2 = create_test_get_request(&url2, api_key, login_id);
            let resp2 = test::call_service(&app, req2).await;
            assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
        }
    }

    async fn validate_solution_response(
        context: &str,
        resp: actix_web::dev::ServiceResponse,
        expected_status_code: StatusCode,
        expected_solution: Option<MazeSolution>,
        expected_err_message: Option<String>
    ) {
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Confirm and validate solution response
            let body = test::read_body(resp).await;
            let solution: MazeSolution = serde_json::from_slice(&body).expect("failed to deserialize response");
             match expected_solution {
                Some(value) => { assert_eq!(solution, value);}
                None => { panic!("{}", format!("No maze solution comparison value provided for {context} test!")); }
            }
        }
        else {
            match expected_err_message {
                Some(value) => {
                    // Validate error response
                    let body = test::read_body(resp).await;
                    let error_message = String::from_utf8(body.to_vec()).expect("Failed to parse body as UTF-8");
                    assert_eq!(error_message, value);
                }
                None => { panic!("{}", format!("No maze solution error message provided for {context} test!")); }
            }
        }
    }

    fn get_no_start_cell_error_str() -> String {
        get_maze_solve_error_string(&MazeError::Solve("no start cell found within maze".to_string()))
    }

    fn get_no_finish_cell_error_str() -> String {
        get_maze_solve_error_string(&MazeError::Solve("no finish cell found within maze".to_string()))
    }

    fn get_no_solution_error_str() -> String {
        get_maze_solve_error_string(&MazeError::Solve("no solution found".to_string()))
    }

    async fn run_get_maze_solution_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        id: &str,
        expected_status_code: StatusCode,
        expected_solution: Option<MazeSolution>,
        expected_err_message: Option<String>
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = format!("/api/v1/mazes/{id}/solution");
        let req = create_test_get_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;

        validate_solution_response("get_maze_solution()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_solve_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        maze: Maze,
        expected_status_code: StatusCode,
        expected_solution: Option<MazeSolution>,
        expected_err_message: Option<String>
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/solve-maze".to_string();
        let req = create_test_post_request(&url, api_key, login_id, Some(&maze));
        let resp = test::call_service(&app, req).await;

        validate_solution_response("solve_maze()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_get_url_test(
        url: &str
     ) {

        let (app, _, _, _, _) = create_test_app(&mut vec![], None, false).await;
        let req = create_test_get_request(url, None, None);
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
    /*********************************************************************/
    /* Endpoint tests                                                    */
    /*********************************************************************/
    /**********/
    /* Users  */
    /**********/

    // Reusable test wrapper functions
    async fn run_cannot_get_users_with_one_non_admin_user_with_non_admin_caller(use_login: bool) {
        run_get_users_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), Some(VALID_USERNAME_1), use_login, StatusCode::UNAUTHORIZED).await;
    }

    async fn run_can_get_users_with_one_admin_user_with_api_key(use_login: bool) {
        run_get_users_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), use_login, StatusCode::OK).await;
    }

    async fn run_can_get_users_with_one_admin_and_one_non_admin_user_with_api_key(use_login: bool) {
        run_get_users_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), use_login, StatusCode::OK).await;
    }

    async fn run_can_get_users_with_ten_admin_and_five_non_admin_users(use_login: bool) {
        run_get_users_test(&CreateUsersDef::new(10, 5, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_2), use_login, StatusCode::OK).await;
    }

    async fn run_can_create_non_existent_admin_user_with_admin_caller(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None , false),
            StatusCode::CREATED).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login,
            &new_create_user_request(true, "", None,  false),
            StatusCode::BAD_REQUEST).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login,
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None , true),
            StatusCode::BAD_REQUEST).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_non_admin_caller(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), 
            Some(VALID_USERNAME_1), use_login, 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None, false),
            StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, VALID_ADMIN_USERNAME_1, None , false), 
            StatusCode::CONFLICT).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, VALID_ADMIN_USERNAME_2, Some(&new_email(VALID_ADMIN_USERNAME_1)), false), 
            StatusCode::CONFLICT).await;
    }

    async fn run_can_create_non_existent_non_admin_user_with_admin_caller(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(false, NEW_USERNAME_1, None, false),
            StatusCode::CREATED).await;
    }

    async fn run_cannot_create_non_existent_non_admin_user_with_non_admin_caller(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), 
            Some(VALID_USERNAME_1), use_login, 
            &new_create_user_request(false, NEW_USERNAME_1, None, false),
            StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, VALID_USERNAME_1, None, false),
            StatusCode::CONFLICT).await;
    }

    async fn run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, VALID_USERNAME_2, Some(&new_email(VALID_USERNAME_1)), false),
            StatusCode::CONFLICT).await;
    }

    async fn run_can_get_user_that_exists_with_admin_caller(use_login: bool) {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          Some(VALID_ADMIN_USERNAME_1), use_login, 
                          VALID_USERNAME_1, StatusCode::OK).await;
    }

    async fn run_can_get_admin_user_that_exists_with_admin_caller(use_login: bool) {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          Some(VALID_ADMIN_USERNAME_1), use_login, 
                          VALID_ADMIN_USERNAME_1, StatusCode::OK).await;
    }

    async fn run_cannot_get_user_that_exists_with_non_admin_caller(use_login: bool) {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          Some(VALID_USERNAME_1), use_login, 
                          VALID_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_get_user_that_does_not_exist_with_admin_caller(use_login: bool) {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          Some(VALID_ADMIN_USERNAME_1), use_login, 
                          VALID_USERNAME_2, StatusCode::NOT_FOUND).await;
    }

    async fn run_can_update_admin_user_with_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(true, NEW_ADMIN_USERNAME_1, None),
            StatusCode::OK).await;
    }

    async fn run_cannot_update_admin_user_with_non_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(true, NEW_ADMIN_USERNAME_1, None),
            StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_update_admin_user_with_admin_caller_but_missing_username(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(true, "", None),
            StatusCode::BAD_REQUEST).await;
    }

    async fn run_cannot_update_admin_user_with_admin_caller_but_existing_username(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(true, VALID_ADMIN_USERNAME_2, None),
            StatusCode::CONFLICT).await;
    }

    async fn run_cannot_update_admin_user_with_admin_caller_but_existing_email(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login,
            VALID_ADMIN_USERNAME_1, &new_update_user_request(true, VALID_ADMIN_USERNAME_1, Some(&new_email(VALID_ADMIN_USERNAME_2))),
            StatusCode::CONFLICT).await;
    }

    async fn run_can_update_non_admin_user_with_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(false, NEW_USERNAME_1, None),
            StatusCode::OK).await;
    }

    async fn run_cannot_update_non_admin_user_with_admin_caller_but_missing_username(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(false, "", None),
            StatusCode::BAD_REQUEST).await;
    }

    async fn run_cannot_update_non_admin_user_with_admin_caller_but_existing_username(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 2, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(false, VALID_USERNAME_2, None),
            StatusCode::CONFLICT).await;
    }

    async fn run_cannot_update_non_admin_user_with_admin_caller_but_existing_email(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 2, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(false, VALID_USERNAME_1, Some(&new_email(VALID_USERNAME_2))),
            StatusCode::CONFLICT).await;
    }

    async fn run_can_upgrade_non_admin_user_to_admin_with_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(true, VALID_USERNAME_1, None),
            StatusCode::OK).await;
    }

    async fn run_can_downgrade_admin_user_to_non_admin_with_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(false, VALID_ADMIN_USERNAME_1, None),
            StatusCode::OK).await;
    }

    async fn run_can_delete_existing_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_2, StatusCode::OK).await;
    }
    
    async fn run_cannot_delete_last_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty),
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, StatusCode::CONFLICT).await;
    }


    async fn run_cannot_delete_non_existent_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_2, StatusCode::NOT_FOUND).await;
    }

    async fn run_can_delete_existing_non_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, StatusCode::OK).await;
    }

    async fn run_cannot_delete_non_existent_non_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, StatusCode::NOT_FOUND).await;
    }

    async fn run_cannot_delete_existing_admin_user_with_non_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 1, MazeContent::Empty), 
            Some(VALID_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_delete_existing_non_admin_user_with_non_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login, VALID_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_delete_me_when_last_admin(use_login: bool) {
        run_delete_me_test(
            &CreateUsersDef::new(1, 0, MazeContent::Empty),
            Some(VALID_ADMIN_USERNAME_1),
            use_login,
            StatusCode::CONFLICT,
        ).await;
    }

    async fn run_can_delete_me_when_not_last_admin(use_login: bool) {
        run_delete_me_test(
            &CreateUsersDef::new(2, 0, MazeContent::Empty),
            Some(VALID_ADMIN_USERNAME_1),
            use_login,
            StatusCode::NO_CONTENT,
        ).await;
    }

    async fn run_can_get_mazes_with_no_mazes(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), Some(VALID_USERNAME_1), use_login, false, MazeContent::Empty).await;
    }

    async fn run_can_get_mazes_with_one_maze_without_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::OneMaze), Some(VALID_USERNAME_1), use_login, false, MazeContent::OneMaze).await;
    }

    async fn run_can_get_mazes_with_one_maze_with_defintions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::OneMaze), Some(VALID_USERNAME_1), use_login, true, MazeContent::OneMaze).await;
    }

    async fn run_can_get_mazes_with_two_mazes_that_require_sorting_without_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::TwoMazes), Some(VALID_USERNAME_1), use_login, false, MazeContent::TwoMazes).await;
    }

    async fn run_can_get_mazes_with_two_mazes_that_require_sorting_with_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::TwoMazes), Some(VALID_USERNAME_1), use_login, true, MazeContent::TwoMazes).await;
    }

    async fn run_can_get_mazes_with_three_mazes_that_require_sorting_without_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, false, MazeContent::ThreeMazes).await;
    }

    async fn run_can_get_mazes_with_three_mazes_that_require_sorting_with_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, true, MazeContent::ThreeMazes).await;
    }

    async fn run_can_create_maze_that_does_not_exist(use_login: bool) {
        run_create_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, new_solvable_maze("", "maze_d"), StatusCode::CREATED).await;
    }

    async fn run_cannot_create_maze_that_already_exists(use_login: bool) {
        run_create_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, new_solvable_maze("", "maze_a"), StatusCode::CONFLICT).await;
    }

    async fn run_can_get_maze_that_exists(use_login: bool) {
        let id = "maze_a.json";
        let name = "maze_a";
        run_get_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, id, StatusCode::OK, Some(new_solvable_maze(id, name))).await;
    }

    async fn run_cannot_get_maze_that_does_not_exist(use_login: bool) {
        run_get_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, "does_not_exist.json", StatusCode::NOT_FOUND, None).await;
    }

    async fn run_can_update_maze_that_exists(use_login: bool) {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, id, new_solvable_maze(id, name), StatusCode::OK).await;
    }

    async fn run_cannot_update_maze_that_does_not_exist(use_login: bool) {
        let id = "maze_d.json";
        let name = "maze_d";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, id, new_solvable_maze(id, name), StatusCode::NOT_FOUND).await;
    }

    async fn run_cannot_update_maze_with_mismatching_id(use_login: bool) {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, id, new_solvable_maze("some_other_id", name), StatusCode::BAD_REQUEST).await;
    }

    async fn run_can_get_maze_solution_that_should_succeed(use_login: bool) {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), use_login, "solvable.json", StatusCode::OK,
            Some(get_solve_test_maze_solution()), None
        ).await;
    }

    async fn run_cannot_get_maze_solution_that_should_fail_with_no_start(use_login: bool) {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), use_login, "no_start.json", StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_start_cell_error_str())
        ).await;
    }

    async fn run_cannot_get_maze_solution_that_should_fail_with_no_finish(use_login: bool) {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), use_login, "no_finish.json", StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    async fn run_cannot_get_maze_solution_that_should_fail_with_no_solution(use_login: bool) {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), use_login, "no_solution.json", StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_solution_error_str())
        ).await;
    }

    async fn run_can_solve_maze_that_should_succeed(use_login: bool) {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            new_solve_test_maze("", "", true, true, false),
            StatusCode::OK,
            Some(get_solve_test_maze_solution()),
            None
        ).await;
    }

    async fn run_cannot_solve_maze_that_should_fail_with_no_start(use_login: bool) {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            new_solve_test_maze("", "", false, true, false),
            StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_start_cell_error_str())
        ).await;
    }

    async fn run_cannot_solve_maze_yhat_should_fail_with_no_finish(use_login: bool) {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            new_solve_test_maze("", "", true, false, false),
            StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    async fn run_cannot_solve_maze_that_should_fail_with_no_solution(use_login: bool) {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            new_solve_test_maze("", "", true, true, true),
            StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_solution_error_str())
        ).await;
    }

    // Login
    #[actix_web::test]
    async fn cannot_login_if_no_users_exist() {
        run_login_logout_test(&CreateUsersDef::new(0, 0, MazeContent::Empty),
            INVALID_EMAIL,
            INVALID_USER_PASSWORD,
            StatusCode::UNAUTHORIZED,
            Some(get_invalid_email_or_password_error_str()),
            false,
            false,
            None 
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_no_email() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            "",
            INVALID_USER_PASSWORD,
            StatusCode::UNPROCESSABLE_ENTITY,
            Some(get_email_and_password_must_be_provided_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_no_password() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            INVALID_EMAIL,
            "",
            StatusCode::UNPROCESSABLE_ENTITY,
            Some(get_email_and_password_must_be_provided_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_email_does_not_exist() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            INVALID_EMAIL,
            INVALID_USER_PASSWORD,
            StatusCode::UNAUTHORIZED,
            Some(get_invalid_email_or_password_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_email_format_is_invalid() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            "notanemail",
            INVALID_USER_PASSWORD,
            StatusCode::UNAUTHORIZED,
            Some(get_invalid_email_or_password_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_email_exists_and_bad_password() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            VALID_USER_EMAIL_1,
            INVALID_USER_PASSWORD,
            StatusCode::UNAUTHORIZED,
            Some(get_invalid_email_or_password_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn can_login_with_valid_credentials() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            VALID_USER_EMAIL_1,
            VALID_USER_PASSWORD,
            StatusCode::OK,
            None,
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn can_login_and_logout_with_valid_credentials() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            VALID_USER_EMAIL_1,
            VALID_USER_PASSWORD,
            StatusCode::OK,
            None,
            true,
            true,
            Some(StatusCode::NO_CONTENT)
        ).await;
    }

    /// Two consecutive logins with the same credentials: the second response
    /// must carry `is_first_sign_in = false` because `User::create_login` on
    /// the first login set `last_sign_in_at = Some(now)` (sticky).
    #[actix_web::test]
    async fn second_login_reports_is_first_sign_in_false() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let login_request = LoginRequest {
            email: VALID_USER_EMAIL_1.to_string(),
            password: VALID_USER_PASSWORD.to_string(),
        };

        // First login — fresh user, `is_first_sign_in` is true.
        let req1 = create_test_post_request("/api/v1/login", None, None, Some(&login_request));
        let resp1 = test::call_service(&app, req1).await;
        assert_eq!(resp1.status(), StatusCode::OK);
        let body1 = test::read_body(resp1).await;
        let login_response_1: LoginResponse =
            serde_json::from_slice(&body1).expect("deserialize first login response");
        assert!(login_response_1.is_first_sign_in, "first login must be is_first_sign_in = true");

        // Second login — same user, no logout in between. `last_sign_in_at`
        // is now `Some(...)` and `logins` is non-empty.
        let req2 = create_test_post_request("/api/v1/login", None, None, Some(&login_request));
        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), StatusCode::OK);
        let body2 = test::read_body(resp2).await;
        let login_response_2: LoginResponse =
            serde_json::from_slice(&body2).expect("deserialize second login response");
        assert!(!login_response_2.is_first_sign_in, "second login must be is_first_sign_in = false");
    }

    /// Login → logout → login. Sign-out removes the entry from `logins`,
    /// but `last_sign_in_at` is sticky, so the second login still reports
    /// `is_first_sign_in = false`. Guards against regressions to a
    /// `logins.is_empty()`-only check, which would silently fail this case.
    #[actix_web::test]
    async fn login_after_signout_reports_is_first_sign_in_false() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let login_request = LoginRequest {
            email: VALID_USER_EMAIL_1.to_string(),
            password: VALID_USER_PASSWORD.to_string(),
        };

        // First login.
        let req1 = create_test_post_request("/api/v1/login", None, None, Some(&login_request));
        let resp1 = test::call_service(&app, req1).await;
        assert_eq!(resp1.status(), StatusCode::OK);
        let body1 = test::read_body(resp1).await;
        let login_response_1: LoginResponse =
            serde_json::from_slice(&body1).expect("deserialize first login response");
        assert!(login_response_1.is_first_sign_in, "first login must be is_first_sign_in = true");

        // Logout — removes the login entry from `user.logins`.
        let logout_req = create_test_post_request(
            "/api/v1/logout",
            None,
            Some(login_response_1.login_token_id),
            None::<&()>,
        );
        let logout_resp = test::call_service(&app, logout_req).await;
        assert_eq!(logout_resp.status(), StatusCode::NO_CONTENT);

        // Second login post-logout — `logins` is empty again, but
        // `last_sign_in_at` is still `Some(...)` from the first login, so
        // the trigger correctly stays false.
        let req2 = create_test_post_request("/api/v1/login", None, None, Some(&login_request));
        let resp2 = test::call_service(&app, req2).await;
        assert_eq!(resp2.status(), StatusCode::OK);
        let body2 = test::read_body(resp2).await;
        let login_response_2: LoginResponse =
            serde_json::from_slice(&body2).expect("deserialize second login response");
        assert!(
            !login_response_2.is_first_sign_in,
            "post-signout login must be is_first_sign_in = false (sticky last_sign_in_at)"
        );
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_logout_if_login_id_not_set_in_logout_header() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            VALID_USER_EMAIL_1,
            VALID_USER_PASSWORD,
            StatusCode::OK,
            None,
            true,
            false,
            None
        ).await;
    }

    // Renew
    #[actix_web::test]
    async fn can_renew_with_valid_token() {
        use crate::api::v1::endpoints::handlers::RenewResponse;
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, shared_store, _, _, _) = create_test_app(&mut user_defs, None, false).await;

        // Log in first
        let login_request = LoginRequest { email: VALID_USER_EMAIL_1.to_string(), password: VALID_USER_PASSWORD.to_string() };
        let login_req = create_test_post_request("/api/v1/login", None, None, Some(&login_request));
        let login_resp = test::call_service(&app, login_req).await;
        assert_eq!(login_resp.status(), StatusCode::OK);

        let login_resp_body = test::read_body(login_resp).await;
        let login_response: LoginResponse = serde_json::from_slice(&login_resp_body).expect("failed to deserialize login response");
        let login_id = login_response.login_token_id;
        let original_expiry = login_response.login_token_expires_at;

        // Renew the token
        let renew_req = create_test_post_request("/api/v1/login/renew", None, Some(login_id), None::<&()>);
        let renew_resp = test::call_service(&app, renew_req).await;
        assert_eq!(renew_resp.status(), StatusCode::OK);

        let renew_resp_body = test::read_body(renew_resp).await;
        let renew_response: RenewResponse = serde_json::from_slice(&renew_resp_body).expect("failed to deserialize renew response");

        // Token ID is unchanged
        assert_eq!(renew_response.login_token_id, login_id);
        // Expiry is extended
        assert!(renew_response.login_token_expires_at >= original_expiry);
        // Login still present in store
        verify_user_login_presence(&shared_store, VALID_USER_EMAIL_1, login_id, true).await;
    }

    #[actix_web::test]
    async fn cannot_renew_with_api_key() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, api_key, _) = create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let renew_req = create_test_post_request("/api/v1/login/renew", api_key, None, None::<&()>);
        let renew_resp = test::call_service(&app, renew_req).await;
        assert_eq!(renew_resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_renew_without_auth_header() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let renew_req = create_test_post_request("/api/v1/login/renew", None, None, None::<&()>);
        test::call_service(&app, renew_req).await;
    }

    // Get users
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn run_test_cannot_get_users_with_no_users_with_invalid_api_key() {
        run_get_users_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, false, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn cannot_get_users_with_one_non_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_get_users_with_one_non_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_users_with_one_non_admin_user_with_non_admin_caller_with_login() {
        run_cannot_get_users_with_one_non_admin_user_with_non_admin_caller(true).await;
    }    

    #[actix_web::test]
    async fn can_get_users_with_one_admin_user_with_api_key() {
        run_can_get_users_with_one_admin_user_with_api_key(false).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_one_admin_user_with_login() {
        run_can_get_users_with_one_admin_user_with_api_key(true).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_one_admin_and_one_non_admin_user_with_api_key() {
        run_can_get_users_with_one_admin_and_one_non_admin_user_with_api_key(false).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_one_admin_and_one_non_admin_user_with_login() {
        run_can_get_users_with_one_admin_and_one_non_admin_user_with_api_key(true).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_ten_admin_and_five_non_admin_users_with_api_key() {
        run_can_get_users_with_ten_admin_and_five_non_admin_users(false).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_ten_admin_and_five_non_admin_users_with_login() {
        run_can_get_users_with_ten_admin_and_five_non_admin_users(true).await;
    }

    // Create user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_create_admin_user_with_invalid_api_key() {
        run_create_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None, false),
            StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_admin_user_with_admin_caller_with_api_key() {
        run_can_create_non_existent_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_admin_user_with_admin_caller_with_login() {
        run_can_create_non_existent_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username_with_login() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password_with_login() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_non_admin_caller_with_login() {
        run_cannot_create_non_existent_admin_user_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username_with_login() {
       run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email_with_login() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email(true).await;
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_create_non_admin_user_with_invalid_api_key() {
        run_create_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, 
            &new_create_user_request(false, NEW_USERNAME_1, None, false), 
            StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_non_admin_user_with_admin_caller_with_api_key() {
        run_can_create_non_existent_non_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_non_admin_user_with_admin_caller_with_login() {
        run_can_create_non_existent_non_admin_user_with_admin_caller(true).await;
    }
    
    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_create_non_existent_non_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_non_admin_caller_with_login() {
        run_cannot_create_non_existent_non_admin_user_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username_with_api_key() {
        run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username_with_login() {
        run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email_with_api_key() {
        run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email_with_login() {
        run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email(true).await;
    }

    // Get user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_user_that_exists_with_invalid_api_key() {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          None, false, 
                          VALID_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_get_user_that_exists_with_admin_caller_with_api_key() {
        run_can_get_user_that_exists_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_get_user_that_exists_with_admin_caller_with_login() {
        run_can_get_user_that_exists_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn can_get_admin_user_that_exists_with_admin_caller_with_api_key() {
        run_can_get_admin_user_that_exists_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_get_admin_user_that_exists_with_admin_caller_with_login() {
        run_can_get_admin_user_that_exists_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_exists_with_non_admin_caller_with_api_key() {
        run_cannot_get_user_that_exists_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_exists_with_non_admin_caller_with_login() {
        run_cannot_get_user_that_exists_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_does_not_exist_with_admin_caller_with_api_key() {
        run_cannot_get_user_that_does_not_exist_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_does_not_exist_with_admin_caller_with_login() {
        run_cannot_get_user_that_does_not_exist_with_admin_caller(true).await;
    }

    // Update user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_update_admin_user_with_invalid_api_key() {
        run_update_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, NEW_ADMIN_USERNAME_1,
            &new_update_user_request(true, NEW_ADMIN_USERNAME_1, None), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_update_admin_user_with_admin_caller_with_api_key() {
        run_can_update_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_update_admin_user_with_admin_caller_with_login() {
        run_can_update_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_update_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_non_admin_caller_with_login() {
        run_cannot_update_admin_user_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_missing_username_with_api_key() {
        run_cannot_update_admin_user_with_admin_caller_but_missing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_missing_username_with_login() {
        run_cannot_update_admin_user_with_admin_caller_but_missing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_username_with_api_key() {
        run_cannot_update_admin_user_with_admin_caller_but_existing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_username_with_login() {
        run_cannot_update_admin_user_with_admin_caller_but_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_email_with_api_key() {
        run_cannot_update_admin_user_with_admin_caller_but_existing_email(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_email_with_login() {
        run_cannot_update_admin_user_with_admin_caller_but_existing_email(true).await;
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_update_non_admin_user_with_invalid_api_key() {
        run_update_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, NEW_ADMIN_USERNAME_1,
            &new_update_user_request(false, NEW_ADMIN_USERNAME_1, None), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_update_non_admin_user_with_admin_caller_with_api_key() {
        run_can_update_non_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_update_non_admin_user_with_admin_caller_with_login() {
        run_can_update_non_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_missing_username_with_api_key() {
        run_cannot_update_non_admin_user_with_admin_caller_but_missing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_missing_username_with_login() {
        run_cannot_update_non_admin_user_with_admin_caller_but_missing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_username_with_api_key() {
        run_cannot_update_non_admin_user_with_admin_caller_but_existing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_username_with_login() {
        run_cannot_update_non_admin_user_with_admin_caller_but_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_email_with_api_key() {
        run_cannot_update_non_admin_user_with_admin_caller_but_existing_email(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_email_with_login() {
        run_cannot_update_non_admin_user_with_admin_caller_but_existing_email(true).await;
    }

    #[actix_web::test]
    async fn can_upgrade_non_admin_user_to_admin_with_admin_caller_with_api_key() {
        run_can_upgrade_non_admin_user_to_admin_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_upgrade_non_admin_user_to_admin_with_admin_caller_with_login() {
        run_can_upgrade_non_admin_user_to_admin_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn can_downgrade_admin_user_to_non_admin_with_admin_caller_with_api_key() {
        run_can_downgrade_admin_user_to_non_admin_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_downgrade_admin_user_to_non_admin_with_admin_caller_with_login() {
        run_can_downgrade_admin_user_to_non_admin_with_admin_caller(true).await;
    }

    // Delete user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_delete_user_with_invalid_api_key() {
        run_delete_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, NEW_ADMIN_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_admin_user_with_admin_caller_with_api_key() {
        run_can_delete_existing_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_admin_user_with_admin_caller_with_login() {
        run_can_delete_existing_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_last_admin_user_with_admin_caller_with_api_key() {
        run_cannot_delete_last_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_last_admin_user_with_admin_caller_with_login() {
        run_cannot_delete_last_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_admin_user_with_admin_caller_with_api_key() {
        run_cannot_delete_non_existent_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_admin_user_with_admin_caller_with_login() {
        run_cannot_delete_non_existent_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_non_admin_user_with_admin_caller_with_api_key() {
        run_can_delete_existing_non_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_non_admin_user_with_admin_caller_with_login() {
        run_can_delete_existing_non_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_non_admin_user_with_admin_caller_with_api_key() {
        run_cannot_delete_non_existent_non_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_non_admin_user_with_admin_caller_with_login() {
        run_cannot_delete_non_existent_non_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_delete_existing_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_admin_user_with_non_admin_caller_with_login() {
        run_cannot_delete_existing_admin_user_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_non_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_delete_existing_non_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_non_admin_user_with_non_admin_caller_with_login() {
        run_cannot_delete_existing_non_admin_user_with_non_admin_caller(true).await;
    }

    /**********/
    /* Mazes  */
    /**********/

    // Get mazes
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_mazes_with_no_mazes_with_invalid_api_key() {
        run_get_mazes_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, false, false, MazeContent::Empty).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_no_mazes_with_api_key() {
        run_can_get_mazes_with_no_mazes(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_no_mazes_with_login() {
        run_can_get_mazes_with_no_mazes(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_without_definitions_with_api_key() {
        run_can_get_mazes_with_one_maze_without_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_without_definitions_with_login() {
        run_can_get_mazes_with_one_maze_without_definitions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_with_defintions_with_api_key() {
        run_can_get_mazes_with_one_maze_with_defintions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_with_defintions_with_login() {
        run_can_get_mazes_with_one_maze_with_defintions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_without_definitions_with_api_key() {
        run_can_get_mazes_with_two_mazes_that_require_sorting_without_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_without_definitions_with_login() {
        run_can_get_mazes_with_two_mazes_that_require_sorting_without_definitions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_with_definitions_with_api_key() {
        run_can_get_mazes_with_two_mazes_that_require_sorting_with_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_with_definitions_with_login() {
        run_can_get_mazes_with_two_mazes_that_require_sorting_with_definitions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_without_definitions_with_api_key() {
        run_can_get_mazes_with_three_mazes_that_require_sorting_without_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_without_definitions_with_login() {
        run_can_get_mazes_with_three_mazes_that_require_sorting_without_definitions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_with_definitions_with_api_key() {
        run_can_get_mazes_with_three_mazes_that_require_sorting_with_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_with_definitions_with_login() {
        run_can_get_mazes_with_three_mazes_that_require_sorting_with_definitions(true).await;
    }

    // Create maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_create_maze_that_does_not_exist_with_invalid_api_key() {
        run_create_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), false, new_solvable_maze("", "maze_d"), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_create_maze_that_does_not_exist_with_api_key() {
        run_can_create_maze_that_does_not_exist(false).await;
    }

    #[actix_web::test]
    async fn can_create_maze_that_does_not_exist_with_login() {
        run_can_create_maze_that_does_not_exist(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_maze_that_already_exists_with_api_key() {
        run_cannot_create_maze_that_already_exists(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_maze_that_already_exists_with_login() {
        run_cannot_create_maze_that_already_exists(true).await;
    }

    // Get maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_maze_that_exists_with_invalid_api_key() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_get_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), false, id, StatusCode::UNAUTHORIZED, Some(new_solvable_maze(id, name))).await;
    }

    #[actix_web::test]
    async fn can_get_maze_that_exists_with_api_key() {
        run_can_get_maze_that_exists(false).await;
    }

    #[actix_web::test]
    async fn can_get_maze_that_exists_with_login() {
        run_can_get_maze_that_exists(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_that_does_not_exist_with_api_key() {
        run_cannot_get_maze_that_does_not_exist(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_that_does_not_exist_with_login() {
        run_cannot_get_maze_that_does_not_exist(true).await;
    }

    // Update maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_update_maze_that_exists_with_invalid_api_key() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), false, id, new_solvable_maze(id, name), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_update_maze_that_exists_with_api_key() {
        run_can_update_maze_that_exists(false).await;
    }

    #[actix_web::test]
    async fn can_update_maze_that_exists_with_login() {
        run_can_update_maze_that_exists(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_that_does_not_exist_with_api_key() {
        run_cannot_update_maze_that_does_not_exist(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_that_does_not_exist_with_login() {
        run_cannot_update_maze_that_does_not_exist(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_with_mismatching_id_with_api_key() {
        run_cannot_update_maze_with_mismatching_id(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_with_mismatching_id_with_login() {
        run_cannot_update_maze_with_mismatching_id(true).await;
    }

    // Delete maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_delete_maze_that_exists_with_invalid_api_key() {
        run_delete_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), false, "maze_a.json", StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_delete_maze_that_exists() {
        run_delete_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes),Some(VALID_USERNAME_1), false, "maze_a.json", StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn cannot_delete_maze_that_does_not_exist() {
        run_delete_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), false, "does_not_exist.json", StatusCode:: NOT_FOUND).await;
    }

    // Get maze solution
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_maze_solution_that_should_succeed_with_invalid_api_key() {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(INVALID_USERNAME), false, "solvable.json", StatusCode::UNAUTHORIZED,
            Some(get_solve_test_maze_solution()), None
        ).await;
    }

    #[actix_web::test]
    async fn can_get_maze_solution_that_should_succeed_with_api_key() {
        run_can_get_maze_solution_that_should_succeed(false).await;
    }

    #[actix_web::test]
    async fn can_get_maze_solution_that_should_succeed_with_login() {
        run_can_get_maze_solution_that_should_succeed(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_start_with_api_key() {
        run_cannot_get_maze_solution_that_should_fail_with_no_start(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_start_with_login() {
        run_cannot_get_maze_solution_that_should_fail_with_no_start(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_finish_with_api_key() {
        run_cannot_get_maze_solution_that_should_fail_with_no_finish(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_finish_with_login() {
        run_cannot_get_maze_solution_that_should_fail_with_no_finish(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_solution_with_api_key() {
        run_cannot_get_maze_solution_that_should_fail_with_no_solution(false).await;
    }

    // Solve maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn canot_solve_maze_that_should_succeed_with_invalid_api_key() {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(INVALID_USERNAME),
            false,
            new_solve_test_maze("", "", true, true, false),
            StatusCode::UNAUTHORIZED,
            Some(get_solve_test_maze_solution()),
            None
        ).await;
    }

    #[actix_web::test]
    async fn can_solve_maze_that_should_succeed_with_api_key() {
        run_can_solve_maze_that_should_succeed(false).await;
    }

    #[actix_web::test]
    async fn can_solve_maze_that_should_succeed_with_login() {
        run_can_solve_maze_that_should_succeed(true).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_start_with_api_key() {
        run_cannot_solve_maze_that_should_fail_with_no_start(false).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_start_with_login() {
        run_cannot_solve_maze_that_should_fail_with_no_start(true).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_yhat_should_fail_with_no_finish_with_api_key() {
        run_cannot_solve_maze_yhat_should_fail_with_no_finish(false).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_yhat_should_fail_with_no_finish_with_login() {
        run_cannot_solve_maze_yhat_should_fail_with_no_finish(true).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_solution_with_api_key() {
        run_cannot_solve_maze_that_should_fail_with_no_solution(false).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_solution_with_login() {
        run_cannot_solve_maze_that_should_fail_with_no_solution(true).await;
    }

    // **************************************************************************************************
    // Generate maze helpers
    // **************************************************************************************************

    fn new_generate_options(
        row_count: usize,
        col_count: usize,
        start: Option<MazePoint>,
        finish: Option<MazePoint>,
        min_spine_length: Option<usize>,
        max_retries: Option<usize>,
    ) -> GeneratorOptions {
        GeneratorOptions {
            row_count,
            col_count,
            algorithm: GenerationAlgorithm::RecursiveBacktracking,
            start,
            finish,
            min_spine_length,
            max_retries,
            branch_from_finish: None,
            seed: None,
        }
    }

    async fn validate_generate_response(
        context: &str,
        resp: actix_web::dev::ServiceResponse,
        expected_status_code: StatusCode,
        expected_rows: Option<usize>,
        expected_cols: Option<usize>,
        expected_err_message: Option<String>,
    ) {
        assert_eq!(resp.status(), expected_status_code);
        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            match (expected_rows, expected_cols) {
                (Some(rows), Some(cols)) => {
                    assert_eq!(maze.definition.row_count(), rows);
                    assert_eq!(maze.definition.col_count(), cols);
                }
                _ => panic!("{}", format!("No maze dimension comparison values provided for {context} test!")),
            }
            assert_eq!(maze.id, "", "{}: expected empty id", context);
            assert_eq!(maze.name, "", "{}: expected empty name", context);
            maze.solve().unwrap_or_else(|_| panic!("{context}: generated maze must be solvable"));
        } else if let Some(value) = expected_err_message {
            let body = test::read_body(resp).await;
            let error_message = String::from_utf8(body.to_vec()).expect("Failed to parse body as UTF-8");
            assert_eq!(error_message, value);
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_generate_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        options: GeneratorOptions,
        expected_status_code: StatusCode,
        expected_rows: Option<usize>,
        expected_cols: Option<usize>,
        expected_err_message: Option<String>,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/mazes/generate".to_string();
        let req = create_test_post_request(&url, api_key, login_id, Some(&options));
        let resp = test::call_service(&app, req).await;
        validate_generate_response("generate_maze()", resp, expected_status_code, expected_rows, expected_cols, expected_err_message).await;
    }

    fn get_generate_row_count_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("row_count must be at least 3".to_string()))
    }

    fn get_generate_col_count_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("col_count must be at least 3".to_string()))
    }

    fn get_generate_start_out_of_bounds_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("start is out of bounds".to_string()))
    }

    fn get_generate_finish_out_of_bounds_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("finish is out of bounds".to_string()))
    }

    fn get_generate_start_equals_finish_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("start and finish must be different cells".to_string()))
    }

    fn get_generate_max_retries_zero_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("max_retries is 0, no attempts made".to_string()))
    }

    async fn run_can_generate_maze_that_should_succeed(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, None, None, None),
            StatusCode::OK, Some(5), Some(5), None,
        ).await;
    }

    async fn run_can_generate_maze_with_minimum_row_count(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(3, 5, None, None, None, None),
            StatusCode::OK, Some(3), Some(5), None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_row_count_too_small(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(2, 5, None, None, None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_row_count_error_str()),
        ).await;
    }

    async fn run_can_generate_maze_with_minimum_col_count(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 3, None, None, None, None),
            StatusCode::OK, Some(5), Some(3), None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_col_count_too_small(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 2, None, None, None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_col_count_error_str()),
        ).await;
    }

    async fn run_can_generate_maze_with_explicit_start_and_finish(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, Some(MazePoint { row: 0, col: 0 }), Some(MazePoint { row: 4, col: 4 }), None, None),
            StatusCode::OK, Some(5), Some(5), None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_start_out_of_bounds(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, Some(MazePoint { row: 10, col: 10 }), None, None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_start_out_of_bounds_error_str()),
        ).await;
    }

    async fn run_cannot_generate_maze_with_finish_out_of_bounds(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, Some(MazePoint { row: 10, col: 10 }), None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_finish_out_of_bounds_error_str()),
        ).await;
    }

    async fn run_cannot_generate_maze_with_start_equals_finish(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, Some(MazePoint { row: 0, col: 0 }), Some(MazePoint { row: 0, col: 0 }), None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_start_equals_finish_error_str()),
        ).await;
    }

    async fn run_can_generate_maze_with_valid_min_spine_length(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, None, Some(3), None),
            StatusCode::OK, Some(5), Some(5), None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_impossible_min_spine_length(use_login: bool) {
        // min_spine_length=1000 is impossible for a 5×5 maze; max_retries=1 keeps the test fast
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, None, Some(1000), Some(1)),
            StatusCode::UNPROCESSABLE_ENTITY, None, None, None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_max_retries_zero(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, None, None, Some(0)),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_max_retries_zero_error_str()),
        ).await;
    }

    // Generate maze tests
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_generate_maze_with_invalid_api_key() {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(INVALID_USERNAME), false,
            new_generate_options(5, 5, None, None, None, None),
            StatusCode::UNAUTHORIZED, None, None, None,
        ).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_that_should_succeed_with_api_key() {
        run_can_generate_maze_that_should_succeed(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_that_should_succeed_with_login() {
        run_can_generate_maze_that_should_succeed(true).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_minimum_row_count_with_api_key() {
        run_can_generate_maze_with_minimum_row_count(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_minimum_row_count_with_login() {
        run_can_generate_maze_with_minimum_row_count(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_row_count_too_small_with_api_key() {
        run_cannot_generate_maze_with_row_count_too_small(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_row_count_too_small_with_login() {
        run_cannot_generate_maze_with_row_count_too_small(true).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_minimum_col_count_with_api_key() {
        run_can_generate_maze_with_minimum_col_count(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_minimum_col_count_with_login() {
        run_can_generate_maze_with_minimum_col_count(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_col_count_too_small_with_api_key() {
        run_cannot_generate_maze_with_col_count_too_small(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_col_count_too_small_with_login() {
        run_cannot_generate_maze_with_col_count_too_small(true).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_explicit_start_and_finish_with_api_key() {
        run_can_generate_maze_with_explicit_start_and_finish(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_explicit_start_and_finish_with_login() {
        run_can_generate_maze_with_explicit_start_and_finish(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_start_out_of_bounds_with_api_key() {
        run_cannot_generate_maze_with_start_out_of_bounds(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_start_out_of_bounds_with_login() {
        run_cannot_generate_maze_with_start_out_of_bounds(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_finish_out_of_bounds_with_api_key() {
        run_cannot_generate_maze_with_finish_out_of_bounds(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_finish_out_of_bounds_with_login() {
        run_cannot_generate_maze_with_finish_out_of_bounds(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_start_equals_finish_with_api_key() {
        run_cannot_generate_maze_with_start_equals_finish(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_start_equals_finish_with_login() {
        run_cannot_generate_maze_with_start_equals_finish(true).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_valid_min_spine_length_with_api_key() {
        run_can_generate_maze_with_valid_min_spine_length(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_valid_min_spine_length_with_login() {
        run_can_generate_maze_with_valid_min_spine_length(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_impossible_min_spine_length_with_api_key() {
        run_cannot_generate_maze_with_impossible_min_spine_length(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_impossible_min_spine_length_with_login() {
        run_cannot_generate_maze_with_impossible_min_spine_length(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_max_retries_zero_with_api_key() {
        run_cannot_generate_maze_with_max_retries_zero(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_max_retries_zero_with_login() {
        run_cannot_generate_maze_with_max_retries_zero(true).await;
    }

    // **************************************************************************************************
    // signup / get_me / delete_me helpers
    // **************************************************************************************************

    fn new_signup_request(email: &str, blank_password: bool) -> SignupRequest {
        SignupRequest {
            email: email.to_string(),
            password: create_password(blank_password),
        }
    }

    async fn run_signup_test(
        create_users_def: &CreateUsersDef,
        signup_req: &SignupRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        // No caller — signup is an unguarded endpoint
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let url = "/api/v1/signup".to_string();
        let req = create_test_post_request(&url, None, None, Some(signup_req));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::CREATED {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize signup response");
            // is_admin must always be false regardless of what the caller sends
            assert!(!response_user.is_admin, "signup must never create an admin user");
            assert!(!response_user.username.is_empty(), "auto-generated username must not be empty");
            assert_eq!(response_user.email, signup_req.email);
            assert_ne!(response_user.id, Uuid::nil());
        }
    }

    async fn run_get_me_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users/me".to_string();
        let req = create_test_get_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize get_me response");
            // Verify the returned profile matches the caller's own data
            if let Some(username) = caller_username {
                let caller_id = MockStore::find_user_id_by_name_in_map(&mock_users, username, Uuid::nil());
                let dummy_user = MockUser::default();
                let expected_user = mock_users.get(&caller_id).unwrap_or(&dummy_user);
                assert_eq!(response_user, expected_user.to_user_item());
            }
        }
    }

    async fn run_delete_me_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, shared_store, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users/me".to_string();
        let req = create_test_delete_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::NO_CONTENT {
            // Verify the caller's account is gone from the store
            if let Some(username) = caller_username {
                let store_lock = get_store_read_lock(&shared_store).await;
                assert!(
                    store_lock.find_user_by_name(username).await.is_err(),
                    "user '{username}' should have been deleted but was still found"
                );
            }
        }
    }

    // **************************************************************************************************
    // Tests: POST /api/v1/signup
    // **************************************************************************************************

    #[actix_web::test]
    async fn signup_with_valid_details_succeeds() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &new_signup_request(&new_email(NEW_USERNAME_1), false),
            StatusCode::CREATED,
        ).await;
    }

    #[actix_web::test]
    async fn signup_always_creates_non_admin_user() {
        // Even if an attacker crafts a request with is_admin, SignupRequest has no such field
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &new_signup_request(&new_email(NEW_USERNAME_1), false),
            StatusCode::CREATED,
        ).await;
    }

    #[actix_web::test]
    async fn can_signup_and_username_is_generated_from_email() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 0, MazeContent::Empty));
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_post_request("/api/v1/signup", None, None, Some(&SignupRequest {
            email: VALID_USER_EMAIL_1.to_string(),
            password: VALID_USER_PASSWORD.to_string(),
        }));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = test::read_body(resp).await;
        let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize signup response");
        assert!(response_user.username.starts_with(VALID_USERNAME_1), "expected username to start with '{}', got '{}'", VALID_USERNAME_1, response_user.username);
    }

    #[actix_web::test]
    async fn signup_creates_user_with_one_primary_unverified_email_row() {
        // Self-service signup creates the email row as the user's *claim*,
        // not as a verified address — the verification handler flips it
        // once the user clicks the link. (Admin-side `CreateUserRequest`
        // keeps the trusted-seed path with `verified = true`; that's
        // covered by separate tests.)
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 0, MazeContent::Empty));
        let (app, shared_store, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let signup_email = new_email(NEW_USERNAME_1);
        let req = create_test_post_request(
            "/api/v1/signup",
            None,
            None,
            Some(&new_signup_request(&signup_email, false)),
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        // The email is unverified, so `find_user_by_verified_email`
        // intentionally returns NotFound — that's the security contract
        // (an unverified address must be invisible to the lookup).
        let store_lock = shared_store.read().await;
        assert!(
            matches!(
                store_lock.find_user_by_verified_email(&signup_email).await,
                Err(StoreError::UserNotFound())
            ),
            "signup-seeded email must be unverified and therefore invisible to find_user_by_verified_email"
        );

        // The user row exists (locate by username instead) and carries
        // exactly one primary, unverified email matching the signup.
        let base_username = signup_email.split('@').next().unwrap();
        let user = store_lock
            .find_user_by_name(base_username)
            .await
            .expect("signup must produce a findable user");
        assert_eq!(
            user.emails.len(),
            1,
            "signup must seed exactly one email row, got {}",
            user.emails.len()
        );
        let row = &user.emails[0];
        assert_eq!(row.email, signup_email, "stored email must match signup request");
        assert!(row.is_primary, "signup-seeded email must be primary");
        assert!(!row.verified, "signup-seeded email must start unverified");
        assert!(
            row.verified_at.is_none(),
            "signup-seeded email verified_at must be None until the user verifies"
        );
    }

    #[actix_web::test]
    async fn signup_with_duplicate_email_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &new_signup_request(&new_email(VALID_USERNAME_1), false),
            StatusCode::CONFLICT,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_blank_password_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &new_signup_request(&new_email(NEW_USERNAME_1), true),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_short_password_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "Abc1!".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_no_uppercase_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "password1!".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_no_lowercase_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "PASSWORD1!".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_no_digit_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "Password!".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_no_special_character_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "Password1".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    // **************************************************************************************************
    // Tests: GET /api/v1/users/me
    // **************************************************************************************************

    #[actix_web::test]
    async fn get_me_as_regular_user_with_api_key_succeeds() {
        run_get_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            false,
            StatusCode::OK,
        ).await;
    }

    #[actix_web::test]
    async fn get_me_as_regular_user_with_login_succeeds() {
        run_get_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            true,
            StatusCode::OK,
        ).await;
    }

    #[actix_web::test]
    async fn get_me_as_admin_with_login_succeeds() {
        run_get_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_ADMIN_USERNAME_1),
            true,
            StatusCode::OK,
        ).await;
    }

    #[actix_web::test]
    #[should_panic]
    async fn get_me_unauthenticated_fails() {
        run_get_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            None,
            false,
            StatusCode::UNAUTHORIZED,
        ).await;
    }

    #[actix_web::test]
    async fn get_me_response_includes_emails_field_alongside_legacy_email() {
        // Asserts the dual shape of GET /me: the legacy `email` field is
        // populated with the primary's address (backwards-compat for
        // clients that haven't migrated), AND the new `emails` array
        // contains the full row data with primary/verified flags.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let req = create_test_get_request("/api/v1/users/me", api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let response_user: UserItem =
            serde_json::from_slice(&body).expect("failed to deserialize get_me response");
        let expected_email = new_email(VALID_USERNAME_1);
        assert_eq!(response_user.email, expected_email);
        assert_eq!(response_user.emails.len(), 1);
        let row = &response_user.emails[0];
        assert_eq!(row.email, expected_email);
        assert!(row.is_primary);
        assert!(row.verified);
    }

    // **************************************************************************************************
    // Tests: DELETE /api/v1/users/me
    // **************************************************************************************************

    #[actix_web::test]
    async fn delete_me_with_api_key_succeeds() {
        run_delete_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            false,
            StatusCode::NO_CONTENT,
        ).await;
    }

    #[actix_web::test]
    async fn delete_me_with_login_succeeds() {
        run_delete_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            true,
            StatusCode::NO_CONTENT,
        ).await;
    }

    #[actix_web::test]
    #[should_panic]
    async fn delete_me_unauthenticated_fails() {
        run_delete_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            None,
            false,
            StatusCode::UNAUTHORIZED,
        ).await;
    }

    #[actix_web::test]
    async fn delete_me_removes_user_from_store() {
        // Verifies the user is gone and subsequent auth with deleted credentials returns 401
        run_delete_me_test(
            &CreateUsersDef::new(1, 2, MazeContent::OneMaze),
            Some(VALID_USERNAME_1),
            false,
            StatusCode::NO_CONTENT,
        ).await;
    }

    #[actix_web::test]
    async fn cannot_delete_me_when_last_admin_with_api_key() {
        run_cannot_delete_me_when_last_admin(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_me_when_last_admin_with_login() {
        run_cannot_delete_me_when_last_admin(true).await;
    }

    #[actix_web::test]
    async fn can_delete_me_when_not_last_admin_with_api_key() {
        run_can_delete_me_when_not_last_admin(false).await;
    }

    #[actix_web::test]
    async fn can_delete_me_when_not_last_admin_with_login() {
        run_can_delete_me_when_not_last_admin(true).await;
    }

    #[actix_web::test]
    async fn delete_me_invalidates_subsequent_login_attempt() {
        // Pin the soft-delete contract at the handler layer: after the
        // account is deleted, a fresh login attempt with the original
        // email + password must fail with 401. Backed by the trait's
        // soft-delete read filter — `find_user_by_verified_email` no
        // longer sees the row, so the login lookup falls through to the
        // anti-enumeration "Invalid email or password" branch.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, api_key, _) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let original_email = new_email(VALID_USERNAME_1);
        let login_request = LoginRequest {
            email: original_email.clone(),
            password: VALID_USER_PASSWORD.to_string(),
        };

        // Sanity: the credentials work before the delete.
        let pre_req = create_test_post_request("/api/v1/login", None, None, Some(&login_request));
        let pre_resp = test::call_service(&app, pre_req).await;
        assert_eq!(pre_resp.status(), StatusCode::OK);

        // Delete the caller via their API key.
        let del_req = create_test_delete_request("/api/v1/users/me", api_key, None);
        let del_resp = test::call_service(&app, del_req).await;
        assert_eq!(del_resp.status(), StatusCode::NO_CONTENT);

        // The same credentials must now be rejected.
        let post_req = create_test_post_request("/api/v1/login", None, None, Some(&login_request));
        let post_resp = test::call_service(&app, post_req).await;
        assert_eq!(post_resp.status(), StatusCode::UNAUTHORIZED);
    }

    // **************************************************************************************************
    // Tests: POST /api/v1/password-reset/{request,confirm}
    // **************************************************************************************************

    /// Spin briefly until the captured-stub buffer has at least `min` rows
    /// or the budget elapses. The reset-request handler dispatches the
    /// email on a `tokio::spawn` task, so the HTTP response comes back
    /// before the send resolves.
    async fn await_stub_capture(stub: &StubEmailProvider, min: usize, max_ms: u64) {
        let start = std::time::Instant::now();
        while stub.len() < min && start.elapsed().as_millis() < u128::from(max_ms) {
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }

    fn extract_reset_token(url: &str) -> Option<Uuid> {
        let q = url.split_once('?')?.1;
        for pair in q.split('&') {
            let (k, v) = pair.split_once('=')?;
            if k == "token" {
                return Uuid::parse_str(v).ok();
            }
        }
        None
    }

    #[actix_web::test]
    async fn password_reset_request_happy_path_dispatches_email_with_reset_link() {
        // Real verified-email match → token created, send fired through stub.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;
        let email = new_email(VALID_USERNAME_1);

        let req = create_test_post_request(
            "/api/v1/password-reset/request",
            None,
            None,
            Some(&PasswordResetRequest { email: email.clone() }),
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);

        await_stub_capture(&stub, 1, 1000).await;
        assert_eq!(stub.len(), 1, "stub should have captured exactly one message");
        let captured = stub.last().expect("captured message");
        assert_eq!(captured.to.len(), 1);
        assert_eq!(captured.to[0].address, email);
        assert!(
            captured.body_text.contains("https://maze.test/reset-password?token="),
            "reset link should appear in body_text: {}",
            captured.body_text
        );
    }

    #[actix_web::test]
    async fn password_reset_confirm_updates_password_and_clears_logins() {
        // The post-reset "old login id rejected" assertion can't go through
        // `test::call_service` because the auth-middleware's
        // `ErrorUnauthorized` return propagates as `Err` rather than as a
        // 401 `ServiceResponse`, which makes `test::call_service` panic.
        // The cleared-`user.logins` invariant is covered by the storage
        // crate's contract tests; here we verify the contract-visible
        // surface — the password was actually rotated — via /login.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;
        let email = new_email(VALID_USERNAME_1);

        // Sanity: the original credentials work before the reset.
        let original_login = LoginRequest {
            email: email.clone(),
            password: VALID_USER_PASSWORD.to_string(),
        };
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/login",
                    None,
                    None,
                    Some(&original_login),
                ),
            )
            .await
            .status(),
            StatusCode::OK
        );

        // Request the reset, harvest the token from the captured email.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/password-reset/request",
                    None,
                    None,
                    Some(&PasswordResetRequest { email: email.clone() }),
                ),
            )
            .await
            .status(),
            StatusCode::OK
        );
        await_stub_capture(&stub, 1, 1000).await;
        let captured = stub.last().expect("captured");
        let link = captured
            .body_text
            .lines()
            .find(|l| l.contains("https://maze.test/reset-password?token="))
            .expect("reset link line");
        let token_id = extract_reset_token(link.trim()).expect("token in link");

        // Confirm with a fresh password.
        let new_password = "NewPassword1!";
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/password-reset/confirm",
                    None,
                    None,
                    Some(&PasswordResetConfirmRequest {
                        token: token_id.to_string(),
                        new_password: new_password.to_string(),
                    }),
                ),
            )
            .await
            .status(),
            StatusCode::NO_CONTENT
        );

        // The new password lets the user log in.
        let new_login = LoginRequest {
            email: email.clone(),
            password: new_password.to_string(),
        };
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request("/api/v1/login", None, None, Some(&new_login)),
            )
            .await
            .status(),
            StatusCode::OK
        );

        // The old password must be rejected.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request("/api/v1/login", None, None, Some(&original_login)),
            )
            .await
            .status(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[actix_web::test]
    async fn password_reset_request_for_unknown_email_returns_200_with_no_send() {
        // Reconnaissance / anti-enumeration: an email that doesn't match
        // any user must return the same 200 as a real match, with no send.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 0, MazeContent::Empty));
        let (app, _shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;

        let req = create_test_post_request(
            "/api/v1/password-reset/request",
            None,
            None,
            Some(&PasswordResetRequest {
                email: "ghost@example.com".to_string(),
            }),
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);

        // Give any in-flight spawn a moment to flush before asserting.
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(stub.len(), 0, "no send should fire for an unknown email");
    }

    #[actix_web::test]
    async fn password_reset_request_for_unverified_email_returns_200_with_no_send() {
        // The handler relies on `find_user_by_verified_email` which the
        // storage layer filters to verified rows only. An attacker who
        // squats an unverified address on a victim's account must not be
        // able to redirect resets to it.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;

        // Attach an unverified email to the existing user. We mutate the
        // store directly because the API requires bearer auth and we
        // want to keep this test focused on the reset-request behaviour.
        let user = {
            let store_lock = shared_store.read().await;
            store_lock
                .find_user_by_name(VALID_USERNAME_1)
                .await
                .expect("locate user_1")
        };
        {
            let mut store_lock = shared_store.write().await;
            store_lock
                .add_user_email(user.id, "shadow@example.com", false)
                .await
                .expect("add unverified email");
        }

        let req = create_test_post_request(
            "/api/v1/password-reset/request",
            None,
            None,
            Some(&PasswordResetRequest {
                email: "shadow@example.com".to_string(),
            }),
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(stub.len(), 0, "unverified email must not trigger a send");
    }

    #[actix_web::test]
    async fn password_reset_request_for_oauth_only_user_returns_200_with_no_send() {
        // OAuth-only users have an empty `password_hash` — there's no
        // password to reset, so the request silently no-ops. The 200
        // response is identical to the unknown-email path.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 0, MazeContent::Empty));
        let (app, shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;

        // Seed an OAuth-only user via the store directly.
        let mut oauth_user = User {
            id: Uuid::nil(),
            is_admin: false,
            username: "oauth_user".to_string(),
            full_name: "OAuth User".to_string(),
            emails: vec![data_model::UserEmail::new_primary_verified("oauth@example.com")],
            password_hash: String::new(),
            api_key: Uuid::nil(),
            logins: vec![],
            oauth_identities: vec![data_model::OAuthIdentity::new(
                "google".to_string(),
                "google-sub-1".to_string(),
                Some("oauth@example.com".to_string()),
            )],
            deleted_at: None,
            created_at: chrono::Utc::now(),
            last_sign_in_at: None,
        };
        {
            let mut store_lock = shared_store.write().await;
            store_lock.create_user(&mut oauth_user).await.expect("create OAuth user");
        }

        let req = create_test_post_request(
            "/api/v1/password-reset/request",
            None,
            None,
            Some(&PasswordResetRequest {
                email: "oauth@example.com".to_string(),
            }),
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(stub.len(), 0, "OAuth-only user must not trigger a send");
    }

    #[actix_web::test]
    async fn password_reset_confirm_rejects_second_consume_attempt() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;
        let email = new_email(VALID_USERNAME_1);

        // Issue + harvest the token.
        let req = create_test_post_request(
            "/api/v1/password-reset/request",
            None,
            None,
            Some(&PasswordResetRequest { email }),
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        await_stub_capture(&stub, 1, 1000).await;
        let captured = stub.last().expect("captured");
        let link = captured
            .body_text
            .lines()
            .find(|l| l.contains("https://maze.test/reset-password?token="))
            .expect("reset link line");
        let token_id = extract_reset_token(link.trim()).expect("token");

        // First consume succeeds.
        let confirm = PasswordResetConfirmRequest {
            token: token_id.to_string(),
            new_password: "NewPassword1!".to_string(),
        };
        let req = create_test_post_request(
            "/api/v1/password-reset/confirm",
            None,
            None,
            Some(&confirm),
        );
        assert_eq!(
            test::call_service(&app, req).await.status(),
            StatusCode::NO_CONTENT
        );

        // Second attempt with the same token must fail with 400.
        let req = create_test_post_request(
            "/api/v1/password-reset/confirm",
            None,
            None,
            Some(&confirm),
        );
        assert_eq!(
            test::call_service(&app, req).await.status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[actix_web::test]
    async fn password_reset_confirm_rejects_unknown_token() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _shared_store, _, _, _, _stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;

        let body = PasswordResetConfirmRequest {
            token: Uuid::new_v4().to_string(),
            new_password: "NewPassword1!".to_string(),
        };
        let req = create_test_post_request(
            "/api/v1/password-reset/confirm",
            None,
            None,
            Some(&body),
        );
        assert_eq!(
            test::call_service(&app, req).await.status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[actix_web::test]
    async fn password_reset_confirm_rejects_weak_new_password() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _shared_store, _, _, _, _stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;

        let body = PasswordResetConfirmRequest {
            token: Uuid::new_v4().to_string(),
            new_password: "weak".to_string(),
        };
        let req = create_test_post_request(
            "/api/v1/password-reset/confirm",
            None,
            None,
            Some(&body),
        );
        assert_eq!(
            test::call_service(&app, req).await.status(),
            StatusCode::BAD_REQUEST
        );
    }

    // **************************************************************************************************
    // Tests: POST /api/v1/email-verifications/{request,confirm}
    // **************************************************************************************************

    fn extract_verification_token(url: &str) -> Option<Uuid> {
        let q = url.split_once('?')?.1;
        for pair in q.split('&') {
            let (k, v) = pair.split_once('=')?;
            if k == "token" {
                return Uuid::parse_str(v).ok();
            }
        }
        None
    }

    fn harvest_verification_token(stub: &StubEmailProvider) -> Uuid {
        let captured = stub.last().expect("captured verification email");
        let link = captured
            .body_text
            .lines()
            .find(|l| l.contains("https://maze.test/verify-email?token="))
            .expect("verification link line");
        extract_verification_token(link.trim()).expect("token in link")
    }

    #[actix_web::test]
    async fn signup_dispatches_verification_email_and_confirm_flips_verified_flag() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 0, MazeContent::Empty));
        let (app, shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;
        let signup_email = new_email(NEW_USERNAME_1);

        // Signup → 201 + verification email captured.
        let signup_resp = test::call_service(
            &app,
            create_test_post_request(
                "/api/v1/signup",
                None,
                None,
                Some(&new_signup_request(&signup_email, false)),
            ),
        )
        .await;
        assert_eq!(signup_resp.status(), StatusCode::CREATED);
        await_stub_capture(&stub, 1, 1000).await;
        let token_id = harvest_verification_token(&stub);

        // The user_emails row starts unverified.
        let user_id = {
            let store_lock = shared_store.read().await;
            let user = store_lock
                .find_user_by_name(signup_email.split('@').next().unwrap())
                .await
                .expect("signup user");
            assert!(!user.emails[0].verified);
            assert!(user.emails[0].verified_at.is_none());
            user.id
        };

        // Confirm → 204 + the row flips.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/confirm",
                    None,
                    None,
                    Some(&EmailVerificationConfirmRequest {
                        token: token_id.to_string(),
                    }),
                ),
            )
            .await
            .status(),
            StatusCode::NO_CONTENT
        );

        let store_lock = shared_store.read().await;
        let user = store_lock.get_user(user_id).await.expect("user");
        assert!(user.emails[0].verified, "row must be verified after confirm");
        assert!(
            user.emails[0].verified_at.is_some(),
            "verified_at must be populated after confirm"
        );
    }

    #[actix_web::test]
    async fn add_email_dispatches_verification_email_and_confirm_flips_secondary() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, shared_store, _, api_key, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let secondary = "alice-secondary@example.com";

        // Add a secondary email via the handler.
        let add_resp = test::call_service(
            &app,
            create_test_post_request(
                "/api/v1/users/me/emails",
                api_key,
                None,
                Some(&AddUserEmailRequest {
                    email: secondary.to_string(),
                }),
            ),
        )
        .await;
        assert_eq!(add_resp.status(), StatusCode::CREATED);
        await_stub_capture(&stub, 1, 1000).await;
        let token_id = harvest_verification_token(&stub);

        // Sanity: the secondary lands unverified.
        {
            let store_lock = shared_store.read().await;
            let user = store_lock
                .find_user_by_name(VALID_USERNAME_1)
                .await
                .expect("alice");
            let row = user
                .emails
                .iter()
                .find(|r| r.email == secondary)
                .expect("secondary row");
            assert!(!row.verified);
        }

        // Confirm → row flips.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/confirm",
                    None,
                    None,
                    Some(&EmailVerificationConfirmRequest {
                        token: token_id.to_string(),
                    }),
                ),
            )
            .await
            .status(),
            StatusCode::NO_CONTENT
        );

        let store_lock = shared_store.read().await;
        let user = store_lock
            .find_user_by_name(VALID_USERNAME_1)
            .await
            .expect("alice");
        let row = user
            .emails
            .iter()
            .find(|r| r.email == secondary)
            .expect("secondary row");
        assert!(row.verified, "secondary must be verified after confirm");
    }

    #[actix_web::test]
    async fn email_verification_request_supersedes_prior_token() {
        // Re-issuing supersedes the previous outstanding token: the old
        // link stops working, only the latest one consumes successfully.
        // Use a seeded verified user + add an unverified secondary so we
        // have an authenticated caller (the seeded user's primary is
        // verified) and an address to verify (the secondary).
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _shared_store, _, api_key, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let secondary = "alice-supersede@example.com";

        // Add the secondary — handler issues token #1 + dispatches.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/users/me/emails",
                    api_key,
                    None,
                    Some(&AddUserEmailRequest {
                        email: secondary.to_string(),
                    }),
                ),
            )
            .await
            .status(),
            StatusCode::CREATED
        );
        await_stub_capture(&stub, 1, 1000).await;
        let first_token = harvest_verification_token(&stub);
        stub.clear();

        // Re-request via /email-verifications/request — supersedes #1.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/request",
                    api_key,
                    None,
                    Some(&EmailVerificationRequest {
                        email: secondary.to_string(),
                    }),
                ),
            )
            .await
            .status(),
            StatusCode::OK
        );
        await_stub_capture(&stub, 1, 1000).await;
        let second_token = harvest_verification_token(&stub);
        assert_ne!(
            first_token, second_token,
            "re-issuance must produce a fresh token id"
        );

        // The first token is now superseded — confirm fails.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/confirm",
                    None,
                    None,
                    Some(&EmailVerificationConfirmRequest {
                        token: first_token.to_string(),
                    }),
                ),
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );

        // The second token still works.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/confirm",
                    None,
                    None,
                    Some(&EmailVerificationConfirmRequest {
                        token: second_token.to_string(),
                    }),
                ),
            )
            .await
            .status(),
            StatusCode::NO_CONTENT
        );
    }

    #[actix_web::test]
    async fn email_verification_request_for_already_verified_is_idempotent_no_send() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _shared_store, _, api_key, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let primary = new_email(VALID_USERNAME_1);

        // The seeded primary is already verified. Re-requesting must
        // return 200 with no send.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/request",
                    api_key,
                    None,
                    Some(&EmailVerificationRequest { email: primary }),
                ),
            )
            .await
            .status(),
            StatusCode::OK
        );

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(
            stub.len(),
            0,
            "already-verified email must not trigger a send"
        );
    }

    #[actix_web::test]
    async fn email_verification_confirm_rejects_unknown_token() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _shared_store, _, _, _, _stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;

        let body = EmailVerificationConfirmRequest {
            token: Uuid::new_v4().to_string(),
        };
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/confirm",
                    None,
                    None,
                    Some(&body),
                ),
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[actix_web::test]
    async fn email_verification_confirm_with_password_reset_token_rejected() {
        // A consumed-token path that uses the wrong purpose must not
        // verify an email — defense-in-depth against stray cross-flow
        // tokens. (Storage is responsible for the cross-user case via
        // the user_id field of the token; that's exercised in the
        // storage contract suite.)
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;
        let email = new_email(VALID_USERNAME_1);

        // Issue a password-reset token.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/password-reset/request",
                    None,
                    None,
                    Some(&PasswordResetRequest { email }),
                ),
            )
            .await
            .status(),
            StatusCode::OK
        );
        await_stub_capture(&stub, 1, 1000).await;
        let captured = stub.last().expect("captured");
        let link = captured
            .body_text
            .lines()
            .find(|l| l.contains("https://maze.test/reset-password?token="))
            .expect("reset link line");
        let q = link.split_once('?').unwrap().1;
        let token_id = q
            .split('&')
            .find_map(|p| p.split_once('=').filter(|(k, _)| *k == "token").map(|(_, v)| v))
            .and_then(|v| Uuid::parse_str(v).ok())
            .expect("token");

        // Now try to use it as a verification token.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/confirm",
                    None,
                    None,
                    Some(&EmailVerificationConfirmRequest {
                        token: token_id.to_string(),
                    }),
                ),
            )
            .await
            .status(),
            StatusCode::BAD_REQUEST
        );
    }

    // **************************************************************************************************
    // Audit-log integration tests
    //
    // Every send goes through service::audit::record_and_dispatch (or
    // record_pending_only for the unknown-email recon path). The pending row
    // is written synchronously, then the spawn settles to Accepted/Failed
    // when the provider responds. These tests assert each write path lands
    // a row, and that the outcome / error_class / template_id match.
    // **************************************************************************************************

    /// Spin briefly until `find_recent_audit_entries_for_user(user_id, _)`
    /// surfaces a row matching `template_id` whose outcome is no longer
    /// `Pending`. The dispatch helper writes Pending synchronously and
    /// updates the outcome from inside `tokio::spawn`, so the HTTP
    /// response returns before the row settles.
    async fn await_audit_settled(
        shared_store: &SharedStore,
        user_id: Uuid,
        template_id: &str,
        max_ms: u64,
    ) -> EmailAuditEntry {
        let start = std::time::Instant::now();
        loop {
            {
                let store_lock = shared_store.read().await;
                let entries = store_lock
                    .find_recent_audit_entries_for_user(user_id, 10)
                    .await
                    .expect("find_recent_audit_entries_for_user");
                if let Some(entry) = entries.into_iter().find(|e| {
                    e.template_id == template_id && e.outcome != AuditOutcome::Pending
                }) {
                    return entry;
                }
            }
            if start.elapsed().as_millis() >= u128::from(max_ms) {
                let store_lock = shared_store.read().await;
                let entries = store_lock
                    .find_recent_audit_entries_for_user(user_id, 10)
                    .await
                    .expect("find_recent_audit_entries_for_user");
                panic!(
                    "audit row never settled (template={template_id}) for user {user_id}; latest entries: {entries:?}"
                );
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    }

    #[actix_web::test]
    async fn password_reset_request_records_pending_then_accepted_audit_row() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;
        let email = new_email(VALID_USERNAME_1);
        let user_id = {
            let store_lock = shared_store.read().await;
            store_lock
                .find_user_by_name(VALID_USERNAME_1)
                .await
                .expect("locate user")
                .id
        };

        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/password-reset/request",
                    None,
                    None,
                    Some(&PasswordResetRequest { email: email.clone() }),
                ),
            )
            .await
            .status(),
            StatusCode::OK
        );
        await_stub_capture(&stub, 1, 1000).await;

        let row = await_audit_settled(&shared_store, user_id, "password_reset", 1000).await;
        assert_eq!(row.outcome, AuditOutcome::Accepted);
        assert_eq!(row.template_id, "password_reset");
        assert_eq!(row.recipient_email, email);
        assert_eq!(row.recipient_user_id, Some(user_id));
        assert_eq!(row.provider, "stub_email");
        assert!(
            row.error_class.is_none(),
            "Accepted rows must not carry error_class: {:?}",
            row.error_class
        );
    }

    #[actix_web::test]
    async fn password_reset_request_records_failed_audit_row_on_provider_error() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;
        let email = new_email(VALID_USERNAME_1);
        let user_id = {
            let store_lock = shared_store.read().await;
            store_lock
                .find_user_by_name(VALID_USERNAME_1)
                .await
                .expect("locate user")
                .id
        };

        // Pre-load a permanent provider failure. RetryPolicy::no_retry()
        // (the stub default) means the orchestrator surfaces it on the
        // first attempt without re-driving the queue.
        stub.enqueue_failure(comms::CommsError::Provider("synthetic".into()));

        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/password-reset/request",
                    None,
                    None,
                    Some(&PasswordResetRequest { email: email.clone() }),
                ),
            )
            .await
            .status(),
            StatusCode::OK
        );

        let row = await_audit_settled(&shared_store, user_id, "password_reset", 1000).await;
        assert_eq!(row.outcome, AuditOutcome::Failed);
        assert_eq!(row.error_class.as_deref(), Some("provider"));
        assert!(
            row.provider_message_id.is_none(),
            "Failed rows must not carry a provider_message_id"
        );
        assert_eq!(stub.len(), 0, "failed sends must not capture the message");
    }

    #[actix_web::test]
    async fn email_verification_request_records_accepted_audit_row() {
        // Use a verified-primary user as the caller, then request
        // verification of a freshly-added secondary so there's an
        // unverified row to act on.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, shared_store, _, api_key, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let secondary = "alice-verify-audit@example.com";

        // Add the secondary — this dispatches its own verification email
        // (also audit-logged). Drain captures + the implicit audit row by
        // remembering the user id for filtering by template_id later.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/users/me/emails",
                    api_key,
                    None,
                    Some(&AddUserEmailRequest { email: secondary.to_string() }),
                ),
            )
            .await
            .status(),
            StatusCode::CREATED
        );
        await_stub_capture(&stub, 1, 1000).await;
        stub.clear();

        let user_id = {
            let store_lock = shared_store.read().await;
            store_lock
                .find_user_by_name(VALID_USERNAME_1)
                .await
                .expect("locate user")
                .id
        };

        // Now re-request verification for the secondary.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/request",
                    api_key,
                    None,
                    Some(&EmailVerificationRequest { email: secondary.to_string() }),
                ),
            )
            .await
            .status(),
            StatusCode::OK
        );
        await_stub_capture(&stub, 1, 1000).await;

        let row = await_audit_settled(&shared_store, user_id, "email_verification", 1000).await;
        assert_eq!(row.outcome, AuditOutcome::Accepted);
        assert_eq!(row.template_id, "email_verification");
        assert_eq!(row.recipient_email, secondary);
        assert_eq!(row.recipient_user_id, Some(user_id));
        assert_eq!(row.provider, "stub_email");
    }

    #[actix_web::test]
    async fn email_verification_request_records_failed_audit_row_on_provider_error() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, shared_store, _, api_key, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let secondary = "alice-verify-fail@example.com";

        // Add the secondary first — that dispatch is allowed to succeed.
        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/users/me/emails",
                    api_key,
                    None,
                    Some(&AddUserEmailRequest { email: secondary.to_string() }),
                ),
            )
            .await
            .status(),
            StatusCode::CREATED
        );
        await_stub_capture(&stub, 1, 1000).await;
        stub.clear();

        // Now arm the next dispatch to fail with a 502-like upstream error
        // — the row should land Failed with `provider_5xx`.
        stub.enqueue_failure(comms::CommsError::ProviderHttp {
            status: 502,
            body: "bad gateway".into(),
        });

        let user_id = {
            let store_lock = shared_store.read().await;
            store_lock
                .find_user_by_name(VALID_USERNAME_1)
                .await
                .expect("locate user")
                .id
        };

        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/email-verifications/request",
                    api_key,
                    None,
                    Some(&EmailVerificationRequest { email: secondary.to_string() }),
                ),
            )
            .await
            .status(),
            StatusCode::OK
        );

        // The Failed row carries error_class = "provider_5xx". Filter on
        // template_id so the earlier add_email Accepted row is ignored.
        let start = std::time::Instant::now();
        let failed = loop {
            let store_lock = shared_store.read().await;
            let entries = store_lock
                .find_recent_audit_entries_for_user(user_id, 10)
                .await
                .expect("find_recent_audit_entries_for_user");
            if let Some(row) = entries
                .into_iter()
                .find(|e| e.template_id == "email_verification" && e.outcome == AuditOutcome::Failed)
            {
                break row;
            }
            drop(store_lock);
            if start.elapsed().as_millis() > 1000 {
                panic!("Failed audit row never appeared for user {user_id}");
            }
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        };
        assert_eq!(failed.error_class.as_deref(), Some("provider_5xx"));
        assert!(failed.provider_message_id.is_none());
    }

    #[actix_web::test]
    async fn signup_records_accepted_audit_row_for_dispatched_verification_email() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 0, MazeContent::Empty));
        let (app, shared_store, _, _, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, None, false).await;
        let signup_email = new_email(NEW_USERNAME_1);

        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/signup",
                    None,
                    None,
                    Some(&new_signup_request(&signup_email, false)),
                ),
            )
            .await
            .status(),
            StatusCode::CREATED
        );
        await_stub_capture(&stub, 1, 1000).await;

        let user_id = {
            let store_lock = shared_store.read().await;
            store_lock
                .find_user_by_name(signup_email.split('@').next().unwrap())
                .await
                .expect("signup user")
                .id
        };

        let row = await_audit_settled(&shared_store, user_id, "email_verification", 1000).await;
        assert_eq!(row.outcome, AuditOutcome::Accepted);
        assert_eq!(row.recipient_email, signup_email);
        assert_eq!(row.recipient_user_id, Some(user_id));
        assert!(row.token_id.is_some(), "verification rows carry the token id");
    }

    #[actix_web::test]
    async fn add_email_records_accepted_audit_row_for_secondary_dispatch() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, shared_store, _, api_key, _, stub) =
            create_test_app_with_stub_email(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let secondary = "alice-add-audit@example.com";

        assert_eq!(
            test::call_service(
                &app,
                create_test_post_request(
                    "/api/v1/users/me/emails",
                    api_key,
                    None,
                    Some(&AddUserEmailRequest { email: secondary.to_string() }),
                ),
            )
            .await
            .status(),
            StatusCode::CREATED
        );
        await_stub_capture(&stub, 1, 1000).await;

        let user_id = {
            let store_lock = shared_store.read().await;
            store_lock
                .find_user_by_name(VALID_USERNAME_1)
                .await
                .expect("locate user")
                .id
        };

        let row = await_audit_settled(&shared_store, user_id, "email_verification", 1000).await;
        assert_eq!(row.outcome, AuditOutcome::Accepted);
        assert_eq!(row.recipient_email, secondary);
        assert_eq!(row.recipient_user_id, Some(user_id));
    }

    #[actix_web::test]
    async fn delete_me_frees_email_for_resignup() {
        // After a soft-delete the email is hard-deleted in the cascade, so a
        // brand-new signup with the same address must succeed.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, api_key, _) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let original_email = new_email(VALID_USERNAME_1);

        let del_req = create_test_delete_request("/api/v1/users/me", api_key, None);
        let del_resp = test::call_service(&app, del_req).await;
        assert_eq!(del_resp.status(), StatusCode::NO_CONTENT);

        let signup_req = create_test_post_request(
            "/api/v1/signup",
            None,
            None,
            Some(&new_signup_request(&original_email, false)),
        );
        let signup_resp = test::call_service(&app, signup_req).await;
        assert_eq!(
            signup_resp.status(),
            StatusCode::CREATED,
            "the original email must be re-claimable after the original user is soft-deleted"
        );
    }

    // **************************************************************************************************
    // change_password_me / update_profile_me helpers
    // **************************************************************************************************

    impl ChangePasswordRequest {
        pub fn new(current_password: &str, new_password: &str) -> ChangePasswordRequest {
            ChangePasswordRequest {
                current_password: Some(current_password.to_string()),
                new_password: new_password.to_string(),
            }
        }

        /// Builds a "set initial password" request — `current_password`
        /// omitted, used by the OAuth-only-set-initial test path.
        pub fn new_set_initial(new_password: &str) -> ChangePasswordRequest {
            ChangePasswordRequest {
                current_password: None,
                new_password: new_password.to_string(),
            }
        }
    }

    impl UpdateProfileRequest {
        pub fn new(username: &str, full_name: &str) -> UpdateProfileRequest {
            UpdateProfileRequest {
                username: username.to_string(),
                full_name: full_name.to_string(),
            }
        }
    }

    fn new_update_profile_request(username: &str) -> UpdateProfileRequest {
        UpdateProfileRequest::new(username, &format!("Updated {username} full name"))
    }

    async fn run_change_password_me_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        change_req: &ChangePasswordRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users/me/password".to_string();
        let req = create_test_put_request(&url, api_key, login_id, change_req);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);
    }

    async fn run_update_profile_me_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        update_req: &UpdateProfileRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users/me/profile".to_string();
        let req = create_test_put_request(&url, api_key, login_id, update_req);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize update_profile_me response");
            // id and is_admin come from the authenticated caller, not the request
            let caller_id = MockStore::find_user_id_by_name_in_map(&mock_users, caller_username.unwrap_or(""), Uuid::nil());
            let dummy_user = MockUser::default();
            let original_user = mock_users.get(&caller_id).unwrap_or(&dummy_user);
            assert_eq!(response_user.id, original_user.user.id);
            assert_eq!(response_user.is_admin, original_user.user.is_admin);
            assert_eq!(response_user.username, update_req.username);
            assert_eq!(response_user.full_name, update_req.full_name);
            // Email is no longer mutable through this endpoint — it must
            // round-trip from the original user unchanged.
            assert_eq!(response_user.email, original_user.user.email());
        }
    }

    // change_password_me scenario helpers
    async fn run_can_change_password_with_valid_current_password(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NewPassword1!"),
            StatusCode::NO_CONTENT,
        ).await;
    }

    async fn run_cannot_change_password_with_wrong_current_password(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(INVALID_USER_PASSWORD, "NewPassword1!"),
            StatusCode::UNAUTHORIZED,
        ).await;
    }

    async fn run_cannot_change_password_with_empty_current_password(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new("", "NewPassword1!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_too_short(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "Sh0rt!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_no_uppercase(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "nouppercase1!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_no_lowercase(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NOLOWERCASE1!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_no_digit(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NoDigitHere!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_no_special_char(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NoSpecial1"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    // update_profile_me scenario helpers
    async fn run_can_update_profile_with_new_username(use_login: bool) {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &new_update_profile_request("updated_username_1"),
            StatusCode::OK,
        ).await;
    }

    async fn run_can_update_profile_keeping_same_username(use_login: bool) {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &new_update_profile_request(VALID_USERNAME_1),
            StatusCode::OK,
        ).await;
    }

    async fn run_cannot_update_profile_with_existing_username(use_login: bool) {
        // user_2 tries to take user_1's username
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 2, MazeContent::Empty),
            Some(VALID_USERNAME_2),
            use_login,
            &new_update_profile_request(VALID_USERNAME_1),
            StatusCode::CONFLICT,
        ).await;
    }

    async fn run_cannot_update_profile_with_empty_username(use_login: bool) {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &UpdateProfileRequest::new("", "Some Full Name"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    /// Verifies that an old client still sending an `email` field gets a
    /// 400 rather than a silent partial success. `deny_unknown_fields` on
    /// `UpdateProfileRequest` rejects the body at JSON parse time. Crafts
    /// the request as raw JSON bytes since the typed struct no longer has
    /// the `email` field at compile time.
    async fn run_update_profile_with_email_field_returns_400(use_login: bool) {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), use_login).await;
        let body = serde_json::json!({
            "username": "updated_username_1",
            "full_name": "Updated Full Name",
            "email": "shouldnt@example.com",
        });
        let req = create_test_put_request("/api/v1/users/me/profile", api_key, login_id, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // change_password_me tests
    #[actix_web::test]
    async fn can_change_password_with_valid_current_password_with_api_key() {
        run_can_change_password_with_valid_current_password(false).await;
    }
    #[actix_web::test]
    async fn can_change_password_with_valid_current_password_with_login() {
        run_can_change_password_with_valid_current_password(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_wrong_current_password_with_api_key() {
        run_cannot_change_password_with_wrong_current_password(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_wrong_current_password_with_login() {
        run_cannot_change_password_with_wrong_current_password(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_empty_current_password_with_api_key() {
        run_cannot_change_password_with_empty_current_password(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_empty_current_password_with_login() {
        run_cannot_change_password_with_empty_current_password(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_too_short_with_api_key() {
        run_cannot_change_password_with_new_password_too_short(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_too_short_with_login() {
        run_cannot_change_password_with_new_password_too_short(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_uppercase_with_api_key() {
        run_cannot_change_password_with_new_password_no_uppercase(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_uppercase_with_login() {
        run_cannot_change_password_with_new_password_no_uppercase(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_lowercase_with_api_key() {
        run_cannot_change_password_with_new_password_no_lowercase(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_lowercase_with_login() {
        run_cannot_change_password_with_new_password_no_lowercase(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_digit_with_api_key() {
        run_cannot_change_password_with_new_password_no_digit(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_digit_with_login() {
        run_cannot_change_password_with_new_password_no_digit(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_special_char_with_api_key() {
        run_cannot_change_password_with_new_password_no_special_char(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_special_char_with_login() {
        run_cannot_change_password_with_new_password_no_special_char(true).await;
    }

    #[actix_web::test]
    #[should_panic]
    async fn cannot_change_password_unauthenticated() {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            None,
            false,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NewPassword1!"),
            StatusCode::UNAUTHORIZED,
        ).await;
    }

    // ─── set-initial-password (OAuth-only user adding a password) ───────

    /// Spins up an app with one OAuth-only user (empty password_hash, one
    /// linked OAuth identity) and signs them in with an api_key. Returns
    /// the typical create_test_app tuple plus the user's id. The test
    /// fixture's `set_valid_password_hashes` overwrites `password_hash`
    /// at signup, so we clear it via the store after the app exists —
    /// and add a stub OAuth identity in the same step so user validation
    /// accepts the empty hash.
    async fn oauth_only_user_test_app() -> (
        impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>,
        SharedStore,
        Uuid,
        Option<Uuid>,
        Option<Uuid>,
    ) {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, shared_store, mock_users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let user_id = MockStore::find_user_id_by_name_in_map(
            &mock_users,
            VALID_USERNAME_1,
            Uuid::nil(),
        );
        {
            let mut store_lock = shared_store.write().await;
            let mut user = store_lock.get_user(user_id).await.expect("user");
            user.password_hash = String::new();
            user.oauth_identities.push(data_model::OAuthIdentity::new(
                "google".into(),
                "set-initial-test-sub".into(),
                None,
            ));
            store_lock.update_user(&mut user).await.expect("seed oauth-only state");
        }
        (app, shared_store, user_id, api_key, login_id)
    }

    #[actix_web::test]
    async fn set_initial_password_succeeds_for_oauth_only_user() {
        let (app, shared_store, user_id, api_key, login_id) = oauth_only_user_test_app().await;

        // Sanity: GET /me reflects that no password is set.
        let req = create_test_get_request("/api/v1/users/me", api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let me: UserItem = serde_json::from_slice(&body).expect("UserItem");
        assert!(!me.has_password, "OAuth-only user must report has_password=false");

        // Send a set-initial request — `current_password` omitted.
        let req = create_test_put_request(
            "/api/v1/users/me/password",
            api_key,
            login_id,
            &ChangePasswordRequest::new_set_initial("NewSet1!"),
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // Verify the store now has a non-empty password_hash and that
        // GET /me reports has_password = true.
        let store_lock = shared_store.read().await;
        let stored = store_lock.get_user(user_id).await.expect("user");
        assert!(!stored.password_hash.is_empty(), "password_hash must be populated after set");
    }

    #[actix_web::test]
    async fn set_initial_password_rejects_when_current_password_present() {
        let (app, _, _, api_key, login_id) = oauth_only_user_test_app().await;
        // OAuth-only user but the client mistakenly sends a current_password.
        let req = create_test_put_request(
            "/api/v1/users/me/password",
            api_key,
            login_id,
            &ChangePasswordRequest::new("anything", "NewSet1!"),
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn change_password_rejects_when_current_password_omitted() {
        // User with a password tries the set-initial request shape.
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            false,
            &ChangePasswordRequest::new_set_initial("NewChange1!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    // update_profile_me tests
    #[actix_web::test]
    async fn can_update_profile_with_new_username_with_api_key() {
        run_can_update_profile_with_new_username(false).await;
    }
    #[actix_web::test]
    async fn can_update_profile_with_new_username_with_login() {
        run_can_update_profile_with_new_username(true).await;
    }

    #[actix_web::test]
    async fn can_update_profile_keeping_same_username_with_api_key() {
        run_can_update_profile_keeping_same_username(false).await;
    }
    #[actix_web::test]
    async fn can_update_profile_keeping_same_username_with_login() {
        run_can_update_profile_keeping_same_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_profile_with_existing_username_with_api_key() {
        run_cannot_update_profile_with_existing_username(false).await;
    }
    #[actix_web::test]
    async fn cannot_update_profile_with_existing_username_with_login() {
        run_cannot_update_profile_with_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_profile_with_empty_username_with_api_key() {
        run_cannot_update_profile_with_empty_username(false).await;
    }
    #[actix_web::test]
    async fn cannot_update_profile_with_empty_username_with_login() {
        run_cannot_update_profile_with_empty_username(true).await;
    }

    #[actix_web::test]
    async fn update_profile_with_email_field_returns_400_with_api_key() {
        run_update_profile_with_email_field_returns_400(false).await;
    }
    #[actix_web::test]
    async fn update_profile_with_email_field_returns_400_with_login() {
        run_update_profile_with_email_field_returns_400(true).await;
    }

    #[actix_web::test]
    #[should_panic]
    async fn cannot_update_profile_unauthenticated() {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            None,
            false,
            &new_update_profile_request(VALID_USERNAME_1),
            StatusCode::UNAUTHORIZED,
        ).await;
    }

    // **************************************************************************************************
    // Tests: silent edge-trim of username + full_name on profile-edit and admin-side create/update.
    // Server-side trim ensures `" alice"` and `"alice"` aren't stored as
    // distinct identities. Mid-string spaces (`"Mary Jane"`) are preserved.
    // Whitespace-only usernames collapse to empty strings and fall through
    // to the existing empty-username rejection.
    // **************************************************************************************************

    async fn run_update_profile_trims_whitespace(use_login: bool) {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), use_login).await;
        let req_body = UpdateProfileRequest::new("  alice_42  ", "  Alice Mary  ");
        let url = "/api/v1/users/me/profile".to_string();
        let req = create_test_put_request(&url, api_key, login_id, &req_body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let user: UserItem = serde_json::from_slice(&body)
            .expect("failed to deserialise update_profile_me response");
        assert_eq!(user.username, "alice_42");
        // Mid-string space preserved.
        assert_eq!(user.full_name, "Alice Mary");
    }

    #[actix_web::test]
    async fn update_profile_trims_whitespace_with_api_key() {
        run_update_profile_trims_whitespace(false).await;
    }
    #[actix_web::test]
    async fn update_profile_trims_whitespace_with_login() {
        run_update_profile_trims_whitespace(true).await;
    }

    #[actix_web::test]
    async fn update_profile_with_whitespace_only_username_returns_400() {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            false,
            &UpdateProfileRequest::new("   ", "Some Full Name"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn create_user_trims_whitespace_in_username_and_full_name() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 0, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_ADMIN_USERNAME_1), false).await;
        let create_req = CreateUserRequest::new(
            false,
            "  bob_99  ",
            "  Bob Jones  ",
            "bob.99@example.com",
            "Password1!",
        );
        let url = "/api/v1/users".to_string();
        let req = create_test_post_request(&url, api_key, login_id, Some(&create_req));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = test::read_body(resp).await;
        let user: UserItem = serde_json::from_slice(&body)
            .expect("failed to deserialise create_user response");
        assert_eq!(user.username, "bob_99");
        assert_eq!(user.full_name, "Bob Jones");
    }

    #[actix_web::test]
    async fn update_user_trims_whitespace_in_username_and_full_name() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, mock_users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_ADMIN_USERNAME_1), false).await;
        let target_id =
            MockStore::find_user_id_by_name_in_map(&mock_users, VALID_USERNAME_1, Uuid::nil());
        let target_email = new_email(VALID_USERNAME_1);
        let update_req = UpdateUserRequest::new(
            false,
            "  carol_77  ",
            "  Carol Smith  ",
            &target_email,
        );
        let url = format!("/api/v1/users/{target_id}");
        let req = create_test_put_request(&url, api_key, login_id, &update_req);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let user: UserItem = serde_json::from_slice(&body)
            .expect("failed to deserialise update_user response");
        assert_eq!(user.username, "carol_77");
        assert_eq!(user.full_name, "Carol Smith");
    }

    // **************************************************************************************************
    // Tests: /api/v1/users/me/emails (GET / POST / DELETE / PUT primary / POST verify-stub)
    // **************************************************************************************************

    use crate::api::v1::endpoints::user_emails::{AddUserEmailRequest, UserEmailsResponse};

    /// Spins up a test app with one regular user, signed in via login token,
    /// and returns the app + the user's id + a closure for building auth'd
    /// requests. Centralises the setup the email-management tests share.
    async fn me_emails_test_app() -> (
        impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>,
        SharedStore,
        Uuid,
        Option<Uuid>,
        Option<Uuid>,
    ) {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, shared_store, mock_users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let user_id = MockStore::find_user_id_by_name_in_map(
            &mock_users,
            VALID_USERNAME_1,
            Uuid::nil(),
        );
        (app, shared_store, user_id, api_key, login_id)
    }

    async fn parse_emails_response(resp: ServiceResponse) -> UserEmailsResponse {
        let body = test::read_body(resp).await;
        serde_json::from_slice(&body).expect("failed to deserialise UserEmailsResponse")
    }

    /// Path for an email-keyed action under `/api/v1/users/me/emails/...`.
    /// Centralises the URL-encoding of the address path segment so each
    /// callsite doesn't duplicate the `urlencoding::encode` boilerplate.
    fn email_path(email: &str, suffix: &str) -> String {
        let encoded = urlencoding::encode(email);
        if suffix.is_empty() {
            format!("/api/v1/users/me/emails/{encoded}")
        } else {
            format!("/api/v1/users/me/emails/{encoded}/{suffix}")
        }
    }

    /// POSTs `AddUserEmailRequest { email }` to `/api/v1/users/me/emails`
    /// against the provided test app, returning the response status. Used
    /// by other tests that need a secondary email row in place before
    /// they exercise their own scenario.
    async fn seed_secondary_email_via_handler(
        app: &impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>,
        api_key: Option<Uuid>,
        login_id: Option<Uuid>,
        email: &str,
    ) -> StatusCode {
        let req = create_test_post_request(
            "/api/v1/users/me/emails",
            api_key,
            login_id,
            Some(&AddUserEmailRequest { email: email.to_string() }),
        );
        let resp = test::call_service(app, req).await;
        resp.status()
    }

    #[actix_web::test]
    async fn list_emails_returns_users_email_rows() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        let req = create_test_get_request("/api/v1/users/me/emails", api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = parse_emails_response(resp).await;
        assert_eq!(body.emails.len(), 1);
        assert!(body.emails[0].is_primary);
        assert!(body.emails[0].verified);
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn list_emails_unauthenticated_returns_401() {
        let (app, _, _, _, _) = me_emails_test_app().await;
        let req = create_test_get_request("/api/v1/users/me/emails", None, None);
        let _ = test::call_service(&app, req).await;
    }

    #[actix_web::test]
    async fn add_email_succeeds_with_valid_address() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        let body = AddUserEmailRequest { email: "alice2@example.com".into() };
        let req = create_test_post_request("/api/v1/users/me/emails", api_key, login_id, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let response = parse_emails_response(resp).await;
        assert_eq!(response.emails.len(), 2);
        let new_row = response
            .emails
            .iter()
            .find(|r| r.email == "alice2@example.com")
            .expect("new row present");
        assert!(!new_row.is_primary);
    }

    #[actix_web::test]
    async fn add_email_with_comms_disabled_creates_verified_and_skips_dispatch() {
        // Comms disabled — the credentials add-email path must create the
        // new row already verified and must not attempt to issue or
        // dispatch a verification email (the user has no path to verify it,
        // so the row would otherwise be permanently stuck unverified, and
        // any spawn would be wasted work).
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, _, stub) =
            create_test_app_with_stub_email_and_comms_enabled(
                &mut user_defs, Some(VALID_USERNAME_1), false, false,
            ).await;
        let body = AddUserEmailRequest { email: "alice2@example.com".into() };
        let req = create_test_post_request(
            "/api/v1/users/me/emails", api_key, None, Some(&body),
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let response = parse_emails_response(resp).await;
        let new_row = response
            .emails
            .iter()
            .find(|r| r.email == "alice2@example.com")
            .expect("new row present");
        assert!(!new_row.is_primary);
        assert!(new_row.verified, "new email must be created verified when comms is disabled");
        assert!(new_row.verified_at.is_some(), "verified_at must be set when verified is true");

        assert_eq!(stub.len(), 0, "no verification email may be dispatched when comms is disabled");
    }

    #[actix_web::test]
    async fn add_email_with_invalid_format_returns_400() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        let body = AddUserEmailRequest { email: "not-an-email".into() };
        let req = create_test_post_request("/api/v1/users/me/emails", api_key, login_id, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn add_email_duplicate_on_same_user_returns_409() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        let body = AddUserEmailRequest { email: new_email(VALID_USERNAME_1) };
        let req = create_test_post_request("/api/v1/users/me/emails", api_key, login_id, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn add_email_duplicate_across_users_returns_409() {
        // Two regular users; user 1 tries to add user 2's email.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 2, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let body = AddUserEmailRequest { email: new_email(VALID_USERNAME_2) };
        let req = create_test_post_request("/api/v1/users/me/emails", api_key, login_id, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn delete_email_removes_non_primary_row() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        let secondary = "alice2@example.com";
        let _ = seed_secondary_email_via_handler(&app, api_key, login_id, secondary).await;

        let req = create_test_delete_request(&email_path(secondary, ""), api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let response = parse_emails_response(resp).await;
        assert_eq!(response.emails.len(), 1);
        assert!(response.emails.iter().all(|r| r.email != secondary));
    }

    #[actix_web::test]
    async fn delete_email_only_email_returns_409() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        let only_email = new_email(VALID_USERNAME_1);
        let req = create_test_delete_request(&email_path(&only_email, ""), api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn delete_email_primary_returns_409() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        // Add a secondary so the primary isn't also the last.
        let _ = seed_secondary_email_via_handler(&app, api_key, login_id, "alice2@example.com").await;

        // Now try to delete the primary — must be refused.
        let primary = new_email(VALID_USERNAME_1);
        let req = create_test_delete_request(&email_path(&primary, ""), api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn delete_email_unknown_address_returns_404() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        let req = create_test_delete_request(&email_path("unknown@example.com", ""), api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn set_primary_email_promotes_verified_secondary() {
        let (app, shared_store, user_id, api_key, login_id) = me_emails_test_app().await;

        // Add a secondary via the handler — it lands `verified = false`
        // and a verification email is dispatched. Mark it verified
        // directly via the store to simulate the user clicking the
        // verification link, then promote it.
        let secondary = "alice2@example.com";
        let _ = seed_secondary_email_via_handler(&app, api_key, login_id, secondary).await;
        {
            let mut store_lock = shared_store.write().await;
            store_lock
                .mark_email_verified(user_id, secondary)
                .await
                .expect("mark secondary verified");
        }

        // Promote it.
        let req = create_test_put_request(
            &email_path(secondary, "primary"),
            api_key,
            login_id,
            &serde_json::json!({}),
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let response = parse_emails_response(resp).await;
        let primary = response
            .emails
            .iter()
            .find(|r| r.is_primary)
            .expect("exactly one primary");
        assert_eq!(primary.email, secondary);

        // And in the store.
        let store_lock = shared_store.read().await;
        let stored = store_lock.get_user(user_id).await.expect("user");
        assert_eq!(stored.email(), secondary);
    }

    #[actix_web::test]
    async fn set_primary_email_rejects_unverified_target() {
        let (app, shared_store, user_id, api_key, login_id) = me_emails_test_app().await;

        // Insert an unverified secondary directly via the store, bypassing
        // the add-email handler (which currently sets verified = true).
        let unverified = "unverified@example.com";
        {
            let mut store_lock = shared_store.write().await;
            store_lock
                .add_user_email(user_id, unverified, false)
                .await
                .expect("seed unverified");
        }

        let req = create_test_put_request(
            &email_path(unverified, "primary"),
            api_key,
            login_id,
            &serde_json::json!({}),
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn set_primary_email_unknown_address_returns_404() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        let req = create_test_put_request(
            &email_path("unknown@example.com", "primary"),
            api_key,
            login_id,
            &serde_json::json!({}),
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn verify_email_endpoint_is_stub_returns_501() {
        let (app, _, _, api_key, login_id) = me_emails_test_app().await;
        let req = create_test_post_request::<()>(
            &email_path("anyone@example.com", "verify"),
            api_key,
            login_id,
            None,
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    }

    // API documentation page load
    #[actix_web::test]
    async fn can_load_swagger_ui_page() {
        run_get_url_test("/api-docs/v1/swagger-ui/").await;
    }

    #[actix_web::test]
    async fn can_load_openapi_json() {
        run_get_url_test("/api-docs/v1/openapi.json").await;
    }

    #[actix_web::test]
    async fn can_load_redoc_page() {
        run_get_url_test("/api-docs/v1/redoc").await;
    }

    #[actix_web::test]
    async fn can_load_rapidoc_page() {
        run_get_url_test("/api-docs/v1/rapidoc").await;
    }

    // **************************************************************************************************
    // Tests: GET /api/v1/features
    // **************************************************************************************************
    #[actix_web::test]
    async fn get_features_returns_defaults() {
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_get_request("/api/v1/features", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let response: AppFeaturesResponse = serde_json::from_slice(&body).expect("failed to deserialize features response");
        assert!(response.allow_signup);
        // `email_enabled` mirrors `comms.enabled`, which defaults to true.
        assert!(response.email_enabled);
    }

    #[actix_web::test]
    async fn get_features_respects_config() {
        let mut user_defs = vec![];
        let features = AppFeaturesConfig { allow_signup: false };
        let features: SharedFeatures = Arc::new(RwLock::new(features));
        let (app, _, _, _, _) = create_test_app_with_features(&mut user_defs, None, false, features).await;
        let req = create_test_get_request("/api/v1/features", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let response: AppFeaturesResponse = serde_json::from_slice(&body).expect("failed to deserialize features response");
        assert!(!response.allow_signup);
    }

    #[actix_web::test]
    async fn get_features_email_enabled_reflects_comms_disabled() {
        let mut user_defs = vec![];
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let mut app_config = AppConfig::default();
        app_config.comms.enabled = false;
        let (app, _, _, _, _) =
            create_test_app_with_config(&mut user_defs, None, false, features, app_config).await;
        let req = create_test_get_request("/api/v1/features", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let response: AppFeaturesResponse = serde_json::from_slice(&body)
            .expect("failed to deserialize features response");
        assert!(!response.email_enabled);
    }

    #[actix_web::test]
    async fn get_features_no_auth_required() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        // No api_key or login_id — endpoint must be accessible without authentication
        let req = create_test_get_request("/api/v1/features", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // **************************************************************************************************
    // Tests: PUT /api/v1/admin/features
    // **************************************************************************************************

    fn make_admin_features_config_toml(allow_signup: bool) -> (AppConfig, std::path::PathBuf) {
        let temp_path = std::env::temp_dir().join(format!("maze_test_{}.toml", Uuid::new_v4()));
        std::fs::write(&temp_path, format!("[features]\nallow_signup = {allow_signup}\n")).unwrap();
        let config = AppConfig { config_path: temp_path.to_string_lossy().to_string(), ..AppConfig::default() };
        (config, temp_path)
    }

    #[actix_web::test]
    async fn cannot_update_admin_features_with_non_admin_caller_with_api_key() {
        let admin_username = &format!("{ADMIN_USERNAME_PREFIX}1");
        let non_admin_username = &format!("{USERNAME_PREFIX}1");
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, api_key, _) = create_test_app(&mut user_defs, Some(non_admin_username), false).await;
        let _ = admin_username;
        let req = create_test_put_request("/api/v1/admin/features", api_key, None, &AppFeaturesResponse { allow_signup: false, ..Default::default() });
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn cannot_update_admin_features_with_non_admin_caller_with_login() {
        let non_admin_username = &format!("{USERNAME_PREFIX}1");
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, _, login_id) = create_test_app(&mut user_defs, Some(non_admin_username), true).await;
        let req = create_test_put_request("/api/v1/admin/features", None, login_id, &AppFeaturesResponse { allow_signup: false, ..Default::default() });
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn update_admin_features_updates_live_state() {
        let admin_username = &format!("{ADMIN_USERNAME_PREFIX}1");
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 0, MazeContent::Empty));
        let (app_config, temp_path) = make_admin_features_config_toml(true);
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let (app, _, _, api_key, _) = create_test_app_with_config(&mut user_defs, Some(admin_username), false, features, app_config).await;

        // Disable signup via admin PUT
        let put_req = create_test_put_request("/api/v1/admin/features", api_key, None, &AppFeaturesResponse { allow_signup: false, ..Default::default() });
        let put_resp = test::call_service(&app, put_req).await;
        assert_eq!(put_resp.status(), StatusCode::OK);
        let body = test::read_body(put_resp).await;
        let response: AppFeaturesResponse = serde_json::from_slice(&body).expect("failed to deserialize response");
        assert!(!response.allow_signup);

        // GET /features now reflects the new value
        let get_req = create_test_get_request("/api/v1/features", None, None);
        let get_resp = test::call_service(&app, get_req).await;
        let body = test::read_body(get_resp).await;
        let features_response: AppFeaturesResponse = serde_json::from_slice(&body).expect("failed to deserialize features response");
        assert!(!features_response.allow_signup);

        let _ = std::fs::remove_file(&temp_path);
    }

    #[actix_web::test]
    async fn update_admin_features_persists_to_config_toml() {
        let admin_username = &format!("{ADMIN_USERNAME_PREFIX}1");
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 0, MazeContent::Empty));
        let (app_config, temp_path) = make_admin_features_config_toml(true);
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let (app, _, _, api_key, _) = create_test_app_with_config(&mut user_defs, Some(admin_username), false, features, app_config).await;

        let put_req = create_test_put_request("/api/v1/admin/features", api_key, None, &AppFeaturesResponse { allow_signup: false, ..Default::default() });
        let put_resp = test::call_service(&app, put_req).await;
        assert_eq!(put_resp.status(), StatusCode::OK);

        // Verify the temp config file was updated on disk
        let content = std::fs::read_to_string(&temp_path).expect("failed to read temp config file");
        let parsed: toml::Table = content.parse().expect("failed to parse updated config toml");
        let allow_signup = parsed["features"]["allow_signup"].as_bool().expect("allow_signup missing");
        assert!(!allow_signup);

        let _ = std::fs::remove_file(&temp_path);
    }

    #[actix_web::test]
    async fn signup_blocked_when_allow_signup_disabled() {
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig { allow_signup: false }));
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app_with_features(&mut user_defs, None, false, features).await;
        let req = create_test_post_request("/api/v1/signup", None, None, Some(&SignupRequest {
            email: VALID_USER_EMAIL_1.to_string(),
            password: VALID_USER_PASSWORD.to_string(),
        }));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn signup_allowed_when_allow_signup_enabled() {
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig { allow_signup: true }));
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app_with_features(&mut user_defs, None, false, features).await;
        let req = create_test_post_request("/api/v1/signup", None, None, Some(&SignupRequest {
            email: VALID_USER_EMAIL_1.to_string(),
            password: VALID_USER_PASSWORD.to_string(),
        }));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn signup_with_comms_disabled_creates_verified_primary_and_skips_dispatch() {
        // Comms disabled — the credentials sign-up path must create the
        // primary email already verified (so the user can sign in and use
        // their account in DEV without a verification step), and must not
        // attempt to issue or dispatch a verification email since there is
        // no working channel for it.
        let mut user_defs = vec![];
        let (app, shared_store, _, _, _, stub) =
            create_test_app_with_stub_email_and_comms_enabled(&mut user_defs, None, false, false).await;
        let req = create_test_post_request("/api/v1/signup", None, None, Some(&SignupRequest {
            email: VALID_USER_EMAIL_1.to_string(),
            password: VALID_USER_PASSWORD.to_string(),
        }));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = test::read_body(resp).await;
        let user: UserItem = serde_json::from_slice(&body).expect("UserItem");
        let primary = user.emails.iter().find(|e| e.is_primary).expect("primary row");
        assert!(primary.verified, "primary email must be created verified when comms is disabled");
        assert!(primary.verified_at.is_some(), "verified_at must be set when verified is true");

        // Persisted store agrees.
        let store_lock = shared_store.read().await;
        let stored = store_lock
            .find_user_by_verified_email(VALID_USER_EMAIL_1)
            .await
            .expect("user must be findable by verified email");
        assert!(stored.email().eq_ignore_ascii_case(VALID_USER_EMAIL_1));

        // No verification dispatch attempted — the gating is synchronous so
        // the stub must be empty by the time the request returns (the
        // dispatch path tokio::spawn'd the send only when entered).
        assert_eq!(stub.len(), 0, "no verification email may be dispatched when comms is disabled");
    }

    // ============================================================================
    // OAuth handler tests
    // ============================================================================

    use crate::oauth::{
        BeginFlow, FlowOrigin, NormalisedIdentity, OAuthConnector, OAuthError,
        OAuthProviderPublic, PersistedState,
    };
    use async_trait::async_trait;

    /// Test connector that returns canned values. Configured via its fields
    /// so each test can drive specific branches of the handler.
    struct FakeConnector {
        providers: Vec<OAuthProviderPublic>,
        authorize_url: String,
        state_nonce: String,
        identity: Option<NormalisedIdentity>,
        complete_error: Option<OAuthError>,
    }

    impl FakeConnector {
        fn google_only() -> Self {
            Self {
                providers: vec![OAuthProviderPublic {
                    name: "google".into(),
                    display_name: "Google".into(),
                }],
                authorize_url: "https://provider.example.com/authorize?fake=1".into(),
                state_nonce: "fake-state-nonce".into(),
                identity: None,
                complete_error: None,
            }
        }
    }

    #[async_trait]
    impl OAuthConnector for FakeConnector {
        fn enabled_providers(&self) -> Vec<OAuthProviderPublic> { self.providers.clone() }

        async fn begin(&self, provider: &str, origin: FlowOrigin) -> Result<BeginFlow, OAuthError> {
            if !self.providers.iter().any(|p| p.name == provider) {
                return Err(OAuthError::UnknownOrDisabledProvider(provider.into()));
            }
            Ok(BeginFlow {
                authorize_url: self.authorize_url.clone(),
                persisted: PersistedState {
                    state: self.state_nonce.clone(),
                    pkce_verifier: "fake-pkce-verifier".into(),
                    origin,
                    provider: provider.to_string(),
                    created_at_unix: chrono::Utc::now().timestamp(),
                    client_state: None,
                },
            })
        }

        async fn complete(
            &self,
            _provider: &str,
            _code: &str,
            _cookie_state: &PersistedState,
        ) -> Result<NormalisedIdentity, OAuthError> {
            if let Some(err_msg) = self.complete_error.as_ref().map(|e| e.to_string()) {
                return Err(OAuthError::ProviderResponse(err_msg));
            }
            self.identity
                .clone()
                .ok_or_else(|| OAuthError::ProviderResponse("test connector has no identity".into()))
        }
    }

    async fn create_test_app_with_oauth_connector(
        connector: Arc<dyn OAuthConnector>,
    ) -> impl Service<actix_http::Request, Response = ServiceResponse, Error = Error> {
        let mut user_defs: Vec<UserDefinition> = vec![];
        let app_config = AppConfig::default();
        let features: SharedFeatures = Arc::new(RwLock::new(app_config.features.clone()));
        set_valid_password_hashes(&app_config.security.password_hash, &mut user_defs);
        let (shared_mock_store, _, _, _) = create_shared_mock_store(&user_defs, None, false);
        let comms = web::Data::new(build_comms(&app_config.comms).expect("test comms"));
        test::init_service(
            create_app(
                &app_config.security.password_hash,
                web::Data::new(shared_mock_store),
                web::Data::new(features),
                web::Data::new(connector as crate::oauth::SharedOAuthConnector),
                comms,
                ".".to_string(),
            )
            .app_data(web::Data::new(app_config)),
        )
        .await
    }

    #[actix_web::test]
    async fn oauth_start_web_origin_redirects_with_state_cookie() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector.clone()).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/start?origin=web")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").expect("Location header").to_str().unwrap();
        assert_eq!(location, "https://provider.example.com/authorize?fake=1");
        let cookie = resp
            .headers()
            .get_all("set-cookie")
            .find_map(|h| h.to_str().ok().filter(|s| s.starts_with("maze_oauth_state=")))
            .expect("state cookie present");
        assert!(cookie.contains("HttpOnly"), "cookie must be HttpOnly: {cookie}");
        assert!(cookie.contains("Secure"), "cookie must be Secure: {cookie}");
        assert!(cookie.contains("SameSite=Lax"), "cookie must be SameSite=Lax: {cookie}");
    }

    #[actix_web::test]
    async fn oauth_start_mobile_origin_redirects_with_state_cookie() {
        // Same response shape as web origin (302 + Set-Cookie). Returning a
        // redirect — rather than JSON for the mobile client to dispatch — is
        // what lets the platform browser carry the state cookie through the
        // round trip; a JSON-then-fetch design would land the cookie in the
        // mobile client's HTTP cookie jar instead of the system browser's.
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/start?origin=mobile")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").expect("Location header").to_str().unwrap();
        assert_eq!(location, "https://provider.example.com/authorize?fake=1");
        let cookie = resp
            .headers()
            .get_all("set-cookie")
            .find_map(|h| h.to_str().ok().filter(|s| s.starts_with("maze_oauth_state=")))
            .expect("state cookie present");
        assert!(cookie.contains("HttpOnly"), "cookie must be HttpOnly: {cookie}");
        assert!(cookie.contains("Secure"), "cookie must be Secure: {cookie}");
    }

    #[actix_web::test]
    async fn oauth_start_unknown_provider_returns_404() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/microsoft/start?origin=web")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn oauth_callback_with_no_state_cookie_redirects_to_login_with_error() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=xyz")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.starts_with("/login?error=invalid_state"), "got: {location}");
    }

    #[actix_web::test]
    async fn oauth_callback_with_state_mismatch_redirects_to_login_with_error() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;

        // Build a valid cookie value with state "real-state".
        let persisted = PersistedState {
            state: "real-state".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=different-state")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.starts_with("/login?error=state_mismatch"), "got: {location}");
    }

    #[actix_web::test]
    async fn oauth_callback_with_provider_path_mismatch_returns_error() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;

        // Cookie says google, URL path says github.
        let persisted = PersistedState {
            state: "s".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/github/callback?code=abc&state=s")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.starts_with("/login?error=provider_mismatch"), "got: {location}");
    }

    #[actix_web::test]
    async fn oauth_callback_happy_path_redirects_with_token() {
        let mut connector = FakeConnector::google_only();
        connector.identity = Some(NormalisedIdentity {
            provider: "google".into(),
            provider_user_id: "google-sub-1".into(),
            email: Some("oauth_user@example.com".into()),
            email_verified: true,
            display_name: Some("Oauth User".into()),
        });
        let connector: Arc<dyn OAuthConnector> = Arc::new(connector);
        let app = create_test_app_with_oauth_connector(connector).await;

        let persisted = PersistedState {
            state: "real-state".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=real-state")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.starts_with("/oauth/callback#token="), "got: {location}");
        assert!(location.contains("&expires_at="), "got: {location}");
        // Cleared state cookie must accompany success too.
        let cookie = resp
            .headers()
            .get_all("set-cookie")
            .find_map(|h| h.to_str().ok().filter(|s| s.starts_with("maze_oauth_state=")))
            .expect("state cookie present (cleared)");
        assert!(cookie.contains("Max-Age=0"), "cleared cookie should set Max-Age=0: {cookie}");
    }

    #[actix_web::test]
    async fn oauth_callback_mobile_origin_uses_same_host_for_errors_with_reason_and_state() {
        // The MAUI WebAuthenticator (and WinUIEx) filter incoming custom-scheme
        // activations by host of the registered CallbackUrl. Errors must
        // therefore use the SAME host as success, distinguished by a `reason`
        // query parameter, with `client_state` echoed so WinUIEx can correlate.
        let connector: Arc<dyn OAuthConnector> = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;

        // Build a cookie whose state DOES NOT match the callback's `state` query.
        let persisted = PersistedState {
            state: "real-state".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Mobile,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: Some(r#"{"appInstanceId":"","signinId":"abc-123"}"#.to_string()),
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=DIFFERENT")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Mobile origin returns a 200 HTML bridge page (not a 302) so the
        // system browser tab doesn't spin forever after the OS hands the
        // `maze-app://` activation to the MAUI app; see `mobile_callback_html`.
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let body = std::str::from_utf8(&body).unwrap();
        // SAME host as success (oauth-callback), not a different oauth-error host.
        // Params live in the fragment to sidestep Facebook's `#_=_`; see
        // `mobile_callback_url` for the full rationale.
        assert!(
            body.contains("maze-app://oauth-callback#"),
            "error must use same host as success in HTML body: {body}"
        );
        // Reason instead of token.
        assert!(body.contains("reason=state_mismatch"), "got: {body}");
        // client_state echoed back so WinUIEx can correlate. The HTML body
        // embeds the URL inside attribute values (meta-refresh `content` and
        // `<a href>`), where `&` is HTML-escaped to `&amp;` for valid HTML.
        assert!(
            body.contains("&amp;state=%7B%22appInstanceId%22%3A%22%22%2C%22signinId%22%3A%22abc-123%22%7D"),
            "client_state must be echoed url-encoded in HTML body: {body}"
        );
    }

    #[actix_web::test]
    async fn oauth_callback_new_user_flag_reflects_account_resolve_outcome() {
        // The post-Step-7 polish item #1 needs the client to know whether an
        // OAuth flow created a new user (so the Account UI can be opened with
        // a welcome banner) or signed in an existing one (no banner). The
        // server signals this via `&new_user=true` on the redirect URL when
        // and only when account::resolve returns `Created`. This test calls
        // the callback twice for the same identity:
        //   1. First call → empty store → branch 3 → Created → new_user=true.
        //   2. Second call → identity now exists → branch 1 → SignedIn → no flag.
        let mut connector = FakeConnector::google_only();
        connector.identity = Some(NormalisedIdentity {
            provider: "google".into(),
            provider_user_id: "google-sub-flag".into(),
            email: Some("flag_user@example.com".into()),
            email_verified: true,
            display_name: None,
        });
        let connector: Arc<dyn OAuthConnector> = Arc::new(connector);
        let app = create_test_app_with_oauth_connector(connector).await;

        let persisted = PersistedState {
            state: "s".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        // First callback: account::resolve creates a new user.
        let first = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=s")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let first_resp = test::call_service(&app, first).await;
        assert_eq!(first_resp.status(), StatusCode::FOUND);
        let first_location = first_resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(
            first_location.contains("&new_user=true"),
            "first sign-in (Created) must flag new_user: {first_location}"
        );

        // Second callback for the same identity: account::resolve finds the
        // existing user via (provider, provider_user_id) → SignedIn → no flag.
        let second = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=s")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let second_resp = test::call_service(&app, second).await;
        assert_eq!(second_resp.status(), StatusCode::FOUND);
        let second_location = second_resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(
            !second_location.contains("new_user"),
            "returning user (SignedIn) must not flag new_user: {second_location}"
        );
    }

    #[actix_web::test]
    async fn oauth_callback_mobile_origin_echoes_client_state_on_redirect() {
        // WinUIEx WebAuthenticator (and similar URL-scheme brokers) need their
        // client-supplied `state` echoed back on the maze-app:// callback so
        // they can correlate the activation with the in-flight task.
        let mut connector = FakeConnector::google_only();
        connector.identity = Some(NormalisedIdentity {
            provider: "google".into(),
            provider_user_id: "google-sub-mobile".into(),
            email: Some("mobile_user@example.com".into()),
            email_verified: true,
            display_name: None,
        });
        let connector: Arc<dyn OAuthConnector> = Arc::new(connector);
        let app = create_test_app_with_oauth_connector(connector).await;

        let persisted = PersistedState {
            state: "real-state".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Mobile,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: Some(r#"{"appInstanceId":"","signinId":"abc-123"}"#.to_string()),
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=real-state")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Mobile success returns a 200 HTML bridge page (not a 302); see
        // `mobile_callback_html` for the rationale.
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("maze-app://oauth-callback#token="), "got: {body}");
        // The URL lives inside HTML attribute values, so inter-param `&`
        // is escaped to `&amp;` for valid HTML.
        assert!(body.contains("&amp;expires_at="), "got: {body}");
        // Critical: client_state must be present and percent-encoded.
        assert!(
            body.contains("&amp;state=%7B%22appInstanceId%22%3A%22%22%2C%22signinId%22%3A%22abc-123%22%7D"),
            "client_state must be echoed back url-encoded: {body}"
        );
    }

    #[actix_web::test]
    async fn get_features_includes_oauth_providers_from_connector() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;
        let req = test::TestRequest::get().uri("/api/v1/features").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let features: AppFeaturesResponse = test::read_body_json(resp).await;
        assert_eq!(features.oauth_providers.len(), 1);
        assert_eq!(features.oauth_providers[0].name, "google");
        assert_eq!(features.oauth_providers[0].display_name, "Google");
    }

    #[actix_web::test]
    async fn get_features_returns_empty_oauth_providers_with_noop_connector() {
        // Default test app uses NoOpConnector via create_test_app_with_features.
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig { allow_signup: true }));
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app_with_features(&mut user_defs, None, false, features).await;
        let req = test::TestRequest::get().uri("/api/v1/features").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: AppFeaturesResponse = test::read_body_json(resp).await;
        assert!(body.oauth_providers.is_empty());
    }

    #[actix_web::test]
    async fn oauth_callback_extends_session_via_renew() {
        // Locks in shared session-lifetime path: the bearer token issued by
        // OAuth callback participates in the existing /login/renew flow
        // exactly like a password-issued token.
        let mut connector = FakeConnector::google_only();
        connector.identity = Some(NormalisedIdentity {
            provider: "google".into(),
            provider_user_id: "google-sub-renew".into(),
            email: Some("renew_user@example.com".into()),
            email_verified: true,
            display_name: None,
        });
        let connector: Arc<dyn OAuthConnector> = Arc::new(connector);
        let app = create_test_app_with_oauth_connector(connector).await;

        let persisted = PersistedState {
            state: "s".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let cb = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=s")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, cb).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        // location like "/oauth/callback#token=<uuid>&expires_at=..."
        let token_id = location
            .strip_prefix("/oauth/callback#token=")
            .and_then(|s| s.split('&').next())
            .expect("token id present in location");

        let renew_req = test::TestRequest::post()
            .uri("/api/v1/login/renew")
            .insert_header(("Authorization", format!("Bearer {token_id}")))
            .to_request();
        let renew_resp = test::call_service(&app, renew_req).await;
        assert_eq!(renew_resp.status(), StatusCode::OK, "OAuth-issued token must work with /login/renew");
    }
}
