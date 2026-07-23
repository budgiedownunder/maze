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
    use crate::api::v1::endpoints::handlers::{AppFeaturesResponse, ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, Play3dConfigResponse, SignupRequest, UpdateProfileRequest, UserItem, UpdateUserRequest, UserLookupResponse, UsersListResponse};
    use crate::api::v1::endpoints::scores::{BoardDatesResponse, CompletedChallengesRequest, CompletedChallengesResponse, RecordScoreRequest, ResetScoresResponse, ScoreboardResponse, ScoreResponse};
    use crate::{create_app, config::app::{AppConfig, AppFeaturesConfig}, oauth::{NoOpConnector, SharedOAuthConnector}, service::notifications::{build_comms, build_default_from, build_renderer}, SharedFeatures};
    use comms::{Comms, StubEmailProvider};
    
    use actix_http;
    use actix_web::{http::StatusCode, test, dev::{Service, ServiceResponse}, web, Error, http::Method};
    use auth::{config::PasswordHashConfig, hashing::hash_password};
    use chrono::{DateTime, Utc};
    use data_model::{CollectionItem, FeaturedGameItem, FeaturedGameItemKind, GameCollection, GameCollectionMeta, GameDefinition, GranteeSummary, Maze, MazeDefinition, MazePoint, PlayMode, Rotation, User, UserLogin, Visibility};
    use crate::api::v1::endpoints::game_definitions::{GameDefinitionSharesResponse, GameDefinitionListResponse, GameDefinitionRequest};
    use crate::api::v1::endpoints::game_shared::SetGameSharesRequest;
    use crate::api::v1::endpoints::game_collections::{GameCollectionSharesResponse, GameCollectionListResponse, GameCollectionRequest, SetGameCollectionItemsRequest};
    use maze::{Error as MazeError, GenerationAlgorithm, GeneratorOptions, MazePath, MazeSolution, MazeSolver};
    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use tokio::sync::{RwLock as AsyncRwLock, RwLockReadGuard};
    use storage::{Error as StoreError, SharedStore, Store, store::EmailAuditLog, store::GameListSort, store::GameStore, store::MazeStore, store::TokenStore, store::UserStore, store::Manage, store::ScoreStore, store::ScoreEntry, store::ScoreboardEntry, store::ScoreOrdering, store::ScoreMetric, store::SortDirection, MazeItem, validation::{validate_maze_cell_count, validate_maze_feature_count, validate_user_fields}};
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
        avatar: Option<Vec<u8>>,
    }

    impl MockUser {
        fn default() -> MockUser {
            MockUser {
                user: User::default(),
                mazes: HashMap::new(),
                avatar: None,
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
                avatar_updated_at: self.user.avatar_updated_at,
            }
        }
        
        fn new_from_user(user: &User) -> Self {
            let mut new_user = user.clone();
            new_user.id = User::new_id();
            new_user.api_key = User::new_api_key();
            MockUser {
                user: new_user,
                mazes: HashMap::new(),
                avatar: None,
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
        scores: Vec<ScoreEntry>,
        game_definitions: Vec<GameDefinition>,
        game_collections: Vec<GameCollection>,
        def_grantees: HashMap<Uuid, Vec<Uuid>>,
        col_grantees: HashMap<Uuid, Vec<Uuid>>,
        def_images: HashMap<Uuid, Vec<u8>>,
        col_images: HashMap<Uuid, Vec<u8>>,
        featured_game_items: Vec<(FeaturedGameItemKind, Uuid)>,
    }

    impl MockStore {
        pub fn new(user_defs: &Vec<UserDefinition>) -> Self {
            MockStore {
                users: new_users_map(user_defs),
                tokens: HashMap::new(),
                audit_entries: HashMap::new(),
                scores: Vec::new(),
                game_definitions: Vec::new(),
                game_collections: Vec::new(),
                def_grantees: HashMap::new(),
                col_grantees: HashMap::new(),
                def_images: HashMap::new(),
                col_images: HashMap::new(),
                featured_game_items: Vec::new(),
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

        /// Owner-scoping check for a game definition: a definition not owned by
        /// `owner` is indistinguishable from absent.
        fn owned_def_or_not_found(&self, owner: &User, id: Uuid) -> Result<(), StoreError> {
            match self.game_definitions.iter().find(|d| d.id == id) {
                Some(d) if d.owner_id == owner.id => Ok(()),
                _ => Err(StoreError::GameDefinitionIdNotFound(id.to_string())),
            }
        }

        /// Owner-scoping check for a game collection.
        fn owned_collection_or_not_found(&self, owner: &User, id: Uuid) -> Result<(), StoreError> {
            match self.game_collections.iter().find(|c| c.meta.id == id) {
                Some(c) if c.meta.owner_id == owner.id => Ok(()),
                _ => Err(StoreError::GameCollectionIdNotFound(id.to_string())),
            }
        }

        fn game_collection_mut(&mut self, id: Uuid) -> Option<&mut GameCollection> {
            self.game_collections.iter_mut().find(|c| c.meta.id == id)
        }

        /// Appends `(kind, id)` to the featured list unless already present.
        fn featured_game_items_append(&mut self, kind: FeaturedGameItemKind, id: Uuid) {
            if !self.featured_game_items.iter().any(|(k, i)| *k == kind && *i == id) {
                self.featured_game_items.push((kind, id));
            }
        }

        /// Removes `(kind, id)` from the featured list; the survivors stay dense
        /// (the index is the sort_order).
        fn featured_game_items_remove(&mut self, kind: FeaturedGameItemKind, id: Uuid) {
            self.featured_game_items.retain(|(k, i)| !(*k == kind && *i == id));
        }

        /// Reconciles the featured row for a visibility transition.
        fn featured_game_items_reconcile(&mut self, kind: FeaturedGameItemKind, id: Uuid, old: Visibility, new: Visibility) {
            match (old == Visibility::Curated, new == Visibility::Curated) {
                (false, true) => self.featured_game_items_append(kind, id),
                (true, false) => self.featured_game_items_remove(kind, id),
                _ => {}
            }
        }

        /// Wraps a board page into `ScoreboardEntry`s, resolving each row's
        /// username from the mock users when requested (mirrors the real
        /// stores' storage-owned resolution).
        fn attach_mock_usernames(
            &self,
            page: Vec<ScoreEntry>,
            include_usernames: bool,
        ) -> Vec<ScoreboardEntry> {
            page.into_iter()
                .map(|entry| {
                    let (username, avatar_updated_at) = if include_usernames {
                        match self.users.get(&entry.user_id) {
                            Some(u) => (
                                Some(u.user.username.clone()),
                                u.user.avatar_updated_at,
                            ),
                            None => (None, None),
                        }
                    } else {
                        (None, None)
                    };
                    ScoreboardEntry {
                        entry,
                        username,
                        avatar_updated_at,
                    }
                })
                .collect()
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

    /// Cell-count cap exposed by `MockStore` to exercise the handler-level
    /// cap path. Matches `SqlStore::MAX_MAZE_CELLS` so the over-cap tests
    /// can use the same dimensions that fail on a real SQL deployment.
    const MOCK_MAX_MAZE_CELLS: usize = 3_600;
    /// A small per-user maze cap for the MockStore — well above what any handler
    /// test creates for one user, but low enough that the cap test can fill to it
    /// cheaply.
    const MOCK_MAX_MAZES_PER_USER: usize = 20;

    #[async_trait]
    impl MazeStore for MockStore {
        fn max_maze_cells(&self) -> Option<usize> {
            Some(MOCK_MAX_MAZE_CELLS)
        }

        fn max_mazes_per_user(&self) -> Option<usize> {
            Some(MOCK_MAX_MAZES_PER_USER)
        }

        async fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?;
            validate_maze_cell_count(
                maze.definition.row_count(),
                maze.definition.col_count(),
                MOCK_MAX_MAZE_CELLS,
            )?;
            validate_maze_feature_count(&maze.definition.grid, maze::MAX_TOTAL_FEATURES)?;
            if mock_user.mazes.len() >= MOCK_MAX_MAZES_PER_USER {
                return Err(StoreError::MazeCountLimitReached {
                    count: mock_user.mazes.len(),
                    max: MOCK_MAX_MAZES_PER_USER,
                });
            }
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
            validate_maze_cell_count(
                maze.definition.row_count(),
                maze.definition.col_count(),
                MOCK_MAX_MAZE_CELLS,
            )?;
            validate_maze_feature_count(&maze.definition.grid, maze::MAX_TOTAL_FEATURES)?;
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
        async fn set_user_avatar(&mut self, id: Uuid, png_bytes: Vec<u8>) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(id)?;
            mock_user.avatar = Some(png_bytes);
            // Stamp the marker in lock-step with the bytes, mirroring the real stores.
            mock_user.user.avatar_updated_at = Some(chrono::Utc::now());
            Ok(())
        }
        async fn get_user_avatar(&self, id: Uuid) -> Result<Option<Vec<u8>>, StoreError> {
            Ok(self.users.get(&id).and_then(|u| u.avatar.clone()))
        }
        async fn clear_user_avatar(&mut self, id: Uuid) -> Result<(), StoreError> {
            // Idempotent: clearing an unknown user or one with no avatar is a no-op.
            if let Some(mock_user) = self.users.get_mut(&id) {
                mock_user.avatar = None;
                mock_user.user.avatar_updated_at = None;
            }
            Ok(())
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
        async fn get_users(&self, limit: u32, offset: u32) -> Result<Vec<User>, StoreError> {
            let mut users: Vec<User> = self.users.values()
                .map( |value| value.user.clone())
                .collect();
            users.sort_by(|a, b| a.username.cmp(&b.username).then(a.id.cmp(&b.id)));
            Ok(users.into_iter().skip(offset as usize).take(limit as usize).collect())
        }

        async fn search_users_by_username_prefix(&self, prefix: &str, limit: u32, offset: u32) -> Result<Vec<User>, StoreError> {
            let prefix = prefix.trim().to_lowercase();
            if prefix.is_empty() {
                return Ok(Vec::new());
            }
            let mut users: Vec<User> = self.users.values()
                .map(|value| value.user.clone())
                .filter(|u| u.username.to_lowercase().starts_with(&prefix))
                .collect();
            users.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()).then(a.id.cmp(&b.id)));
            Ok(users.into_iter().skip(offset as usize).take(limit as usize).collect())
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

    // Minimal stub — the server does not exercise scoring yet (the record +
    // leaderboard handlers land in a later step, which will flesh this out).
    // Present so `MockStore` satisfies the `Store` supertrait bound.
    #[async_trait]
    impl ScoreStore for MockStore {
        async fn record_score(&mut self, entry: &ScoreEntry) -> Result<Uuid, StoreError> {
            // Mirror the real stores' subject invariant (exactly one of
            // maze_id / challenge) so the handler's 400 path is exercised.
            // `is_some() == is_some()` is true when both or neither are set.
            if entry.maze_id.is_some() == entry.challenge.is_some() {
                return Err(StoreError::Other(
                    "score entry must set exactly one of maze_id / challenge".to_string(),
                ));
            }
            self.scores.push(entry.clone());
            Ok(entry.id)
        }
        async fn maze_leaderboard(
            &self,
            maze_id: &str,
            ordering: ScoreOrdering,
            limit: u32,
            offset: u32,
            include_usernames: bool,
        ) -> Result<Vec<ScoreboardEntry>, StoreError> {
            let page = mock_paged_board(
                &self.scores,
                |e| e.maze_id.as_deref() == Some(maze_id),
                ordering,
                limit,
                offset,
            );
            Ok(self.attach_mock_usernames(page, include_usernames))
        }
        async fn challenge_leaderboard(
            &self,
            challenge: &str,
            ordering: ScoreOrdering,
            limit: u32,
            offset: u32,
            include_usernames: bool,
        ) -> Result<Vec<ScoreboardEntry>, StoreError> {
            let page = mock_paged_board(
                &self.scores,
                |e| e.challenge.as_deref() == Some(challenge),
                ordering,
                limit,
                offset,
            );
            Ok(self.attach_mock_usernames(page, include_usernames))
        }
        async fn user_history(
            &self,
            user_id: Uuid,
            limit: u32,
            offset: u32,
        ) -> Result<Vec<ScoreEntry>, StoreError> {
            let mut matched: Vec<ScoreEntry> =
                self.scores.iter().filter(|e| e.user_id == user_id).cloned().collect();
            // Recent first: recorded_at DESC, id DESC (mirrors FileStore/SqlStore).
            matched.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at).then(b.id.cmp(&a.id)));
            Ok(matched.into_iter().skip(offset as usize).take(limit as usize).collect())
        }
        async fn completed_challenges(
            &self,
            user_id: Uuid,
            challenges: &[String],
        ) -> Result<Vec<String>, StoreError> {
            let wanted: std::collections::HashSet<&str> = challenges.iter().map(String::as_str).collect();
            let mut done: std::collections::HashSet<String> = std::collections::HashSet::new();
            for entry in self.scores.iter().filter(|e| e.user_id == user_id) {
                if let Some(challenge) = entry.challenge.as_deref() {
                    if wanted.contains(challenge) {
                        done.insert(challenge.to_string());
                    }
                }
            }
            Ok(done.into_iter().collect())
        }
        async fn challenges_with_prefix(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
            let mut distinct: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
            for entry in &self.scores {
                if let Some(challenge) = entry.challenge.as_deref() {
                    if challenge.starts_with(prefix) {
                        distinct.insert(challenge.to_string());
                    }
                }
            }
            Ok(distinct.into_iter().collect())
        }
        async fn clear_maze_scores(&mut self, maze_id: &str) -> Result<u64, StoreError> {
            let before = self.scores.len();
            self.scores.retain(|e| e.maze_id.as_deref() != Some(maze_id));
            Ok((before - self.scores.len()) as u64)
        }
        async fn clear_challenge_scores(&mut self, challenge: &str) -> Result<u64, StoreError> {
            let before = self.scores.len();
            self.scores.retain(|e| e.challenge.as_deref() != Some(challenge));
            Ok((before - self.scores.len()) as u64)
        }
        async fn clear_challenge_scores_prefix(&mut self, prefix: &str) -> Result<u64, StoreError> {
            let dated = format!("{prefix}:");
            let before = self.scores.len();
            self.scores.retain(|e| {
                !e.challenge
                    .as_deref()
                    .is_some_and(|c| c == prefix || c.starts_with(&dated))
            });
            Ok((before - self.scores.len()) as u64)
        }
    }

    /// Orders two score entries by `ordering`, mirroring the FileStore /
    /// SqlStore tie-break (primary metric per direction, the other metric, then
    /// recorded_at ASC, then id ASC). Used by the MockStore board queries so the
    /// handler tests exercise real ordering + paging.
    fn mock_score_cmp(ordering: ScoreOrdering, a: &ScoreEntry, b: &ScoreEntry) -> std::cmp::Ordering {
        let primary = match ordering.metric {
            ScoreMetric::Time => a.elapsed_ms.cmp(&b.elapsed_ms),
            ScoreMetric::Score => a.score.cmp(&b.score),
        };
        let primary = match ordering.direction {
            SortDirection::Ascending => primary,
            SortDirection::Descending => primary.reverse(),
        };
        let secondary = match ordering.metric {
            ScoreMetric::Time => b.score.cmp(&a.score),
            ScoreMetric::Score => a.elapsed_ms.cmp(&b.elapsed_ms),
        };
        primary
            .then(secondary)
            .then(a.recorded_at.cmp(&b.recorded_at))
            .then(a.id.cmp(&b.id))
    }

    fn mock_paged_board(
        entries: &[ScoreEntry],
        keep: impl Fn(&ScoreEntry) -> bool,
        ordering: ScoreOrdering,
        limit: u32,
        offset: u32,
    ) -> Vec<ScoreEntry> {
        let mut matched: Vec<ScoreEntry> = entries.iter().filter(|e| keep(e)).cloned().collect();
        matched.sort_by(|a, b| mock_score_cmp(ordering, a, b));
        matched.into_iter().skip(offset as usize).take(limit as usize).collect()
    }

    /// Sorts a list of definitions/collections case-insensitively by name.
    fn sort_by_name_ci<T>(items: &mut [T], name: impl Fn(&T) -> &str) {
        items.sort_by_key(|item| name(item).to_lowercase());
    }

    /// Rewrites each item's `sort_order` to its index (dense `0..n`).
    fn renumber_items(items: &mut [CollectionItem]) {
        for (index, item) in items.iter_mut().enumerate() {
            item.sort_order = index as u32;
        }
    }

    /// Small per-user game caps for the MockStore — above what any handler test
    /// creates for one user, but low enough that the cap tests can fill to them.
    const MOCK_MAX_DEFINITIONS_PER_USER: usize = 10;
    const MOCK_MAX_COLLECTIONS_PER_USER: usize = 10;

    #[async_trait]
    impl GameStore for MockStore {
        fn max_definitions_per_user(&self) -> Option<usize> {
            Some(MOCK_MAX_DEFINITIONS_PER_USER)
        }

        fn max_collections_per_user(&self) -> Option<usize> {
            Some(MOCK_MAX_COLLECTIONS_PER_USER)
        }

        // ── Definitions ──

        async fn create_game_definition(&mut self, owner: &User, definition: &mut GameDefinition) -> Result<(), StoreError> {
            if definition.name.trim().is_empty() {
                return Err(StoreError::GameDefinitionNameMissing());
            }
            if self.game_definitions.iter().any(|d| d.owner_id == owner.id && d.name.eq_ignore_ascii_case(&definition.name)) {
                return Err(StoreError::GameDefinitionNameAlreadyExists(definition.name.clone()));
            }
            let count = self.game_definitions.iter().filter(|d| d.owner_id == owner.id).count();
            if count >= MOCK_MAX_DEFINITIONS_PER_USER {
                return Err(StoreError::GameDefinitionCountLimitReached { count, max: MOCK_MAX_DEFINITIONS_PER_USER });
            }
            definition.owner_id = owner.id;
            if definition.id.is_nil() {
                definition.id = Uuid::new_v4();
            }
            let now = Utc::now();
            definition.created_at = now;
            definition.updated_at = now;
            self.game_definitions.push(definition.clone());
            if definition.visibility == Visibility::Curated {
                self.featured_game_items_append(FeaturedGameItemKind::Definition, definition.id);
            }
            Ok(())
        }

        async fn get_game_definition(&self, id: Uuid) -> Result<GameDefinition, StoreError> {
            self.game_definitions.iter().find(|d| d.id == id).cloned()
                .ok_or_else(|| StoreError::GameDefinitionIdNotFound(id.to_string()))
        }

        async fn update_game_definition(&mut self, owner: &User, definition: &mut GameDefinition) -> Result<(), StoreError> {
            let existing = self.game_definitions.iter().find(|d| d.id == definition.id).cloned()
                .ok_or_else(|| StoreError::GameDefinitionIdNotFound(definition.id.to_string()))?;
            if existing.owner_id != owner.id {
                return Err(StoreError::GameDefinitionIdNotFound(definition.id.to_string()));
            }
            if definition.name.trim().is_empty() {
                return Err(StoreError::GameDefinitionNameMissing());
            }
            if self.game_definitions.iter().any(|d| d.id != definition.id && d.owner_id == owner.id && d.name.eq_ignore_ascii_case(&definition.name)) {
                return Err(StoreError::GameDefinitionNameAlreadyExists(definition.name.clone()));
            }
            definition.owner_id = owner.id;
            definition.created_at = existing.created_at;
            definition.updated_at = Utc::now();
            if let Some(slot) = self.game_definitions.iter_mut().find(|d| d.id == definition.id) {
                *slot = definition.clone();
            }
            self.featured_game_items_reconcile(FeaturedGameItemKind::Definition, definition.id, existing.visibility, definition.visibility);
            Ok(())
        }

        async fn delete_game_definition(&mut self, owner: &User, id: Uuid) -> Result<(), StoreError> {
            self.owned_def_or_not_found(owner, id)?;
            self.game_definitions.retain(|d| d.id != id);
            self.def_grantees.remove(&id);
            // Drop the game from every collection listing it (membership carries no
            // FK, so nothing else removes these), re-compacting the survivors.
            for collection in &mut self.game_collections {
                if collection.items.iter().any(|i| i.definition_id == id) {
                    collection.items.retain(|i| i.definition_id != id);
                    for (index, item) in collection.items.iter_mut().enumerate() {
                        item.sort_order = index as u32;
                    }
                }
            }
            self.featured_game_items_remove(FeaturedGameItemKind::Definition, id);
            Ok(())
        }

        async fn grant_game_definition_access(&mut self, owner: &User, id: Uuid, grantee: Uuid) -> Result<(), StoreError> {
            self.owned_def_or_not_found(owner, id)?;
            let grantees = self.def_grantees.entry(id).or_default();
            if !grantees.contains(&grantee) {
                grantees.push(grantee);
            }
            Ok(())
        }

        async fn revoke_game_definition_access(&mut self, owner: &User, id: Uuid, grantee: Uuid) -> Result<(), StoreError> {
            self.owned_def_or_not_found(owner, id)?;
            if let Some(grantees) = self.def_grantees.get_mut(&id) {
                grantees.retain(|g| *g != grantee);
            }
            Ok(())
        }

        async fn set_game_definition_grantees(&mut self, owner: &User, id: Uuid, grantees: &[Uuid]) -> Result<(), StoreError> {
            self.owned_def_or_not_found(owner, id)?;
            let mut seen = std::collections::HashSet::new();
            let cleaned: Vec<Uuid> = grantees.iter().copied().filter(|g| *g != owner.id && seen.insert(*g)).collect();
            self.def_grantees.insert(id, cleaned);
            Ok(())
        }

        async fn get_game_definitions_for_owner(&self, owner: &User) -> Result<Vec<GameDefinition>, StoreError> {
            let mut defs: Vec<GameDefinition> = self.game_definitions.iter().filter(|d| d.owner_id == owner.id).cloned().collect();
            sort_by_name_ci(&mut defs, |d| &d.name);
            Ok(defs)
        }

        async fn get_visible_game_definitions(&self, viewer: &User, limit: u32, offset: u32) -> Result<Vec<GameDefinition>, StoreError> {
            let mut defs: Vec<GameDefinition> = self.game_definitions.iter()
                .filter(|d| d.owner_id == viewer.id
                    || matches!(d.visibility, Visibility::Public | Visibility::Curated)
                    || (d.visibility == Visibility::Shared
                        && self.def_grantees.get(&d.id).is_some_and(|g| g.contains(&viewer.id))))
                .cloned().collect();
            defs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then(a.id.cmp(&b.id)));
            Ok(defs.into_iter().skip(offset as usize).take(limit as usize).collect())
        }

        async fn get_shared_game_definitions(&self, viewer: &User, limit: u32, offset: u32) -> Result<Vec<GameDefinition>, StoreError> {
            let mut defs: Vec<GameDefinition> = self.game_definitions.iter()
                .filter(|d| d.owner_id != viewer.id
                    && d.visibility == Visibility::Shared
                    && self.def_grantees.get(&d.id).is_some_and(|g| g.contains(&viewer.id)))
                .cloned().collect();
            defs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()).then(a.id.cmp(&b.id)));
            Ok(defs.into_iter().skip(offset as usize).take(limit as usize).collect())
        }

        async fn get_public_game_definitions(&self, viewer: &User, name_query: Option<&str>, sort: GameListSort, limit: u32, offset: u32) -> Result<Vec<GameDefinition>, StoreError> {
            let needle = name_query.map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty());
            let mut defs: Vec<GameDefinition> = self.game_definitions.iter()
                .filter(|d| d.visibility == Visibility::Public
                    && d.owner_id != viewer.id
                    && needle.as_ref().is_none_or(|n| d.name.to_lowercase().contains(n)))
                .cloned().collect();
            defs.sort_by(|a, b| match sort {
                GameListSort::Name => a.name.to_lowercase().cmp(&b.name.to_lowercase()).then(a.id.cmp(&b.id)),
                GameListSort::Newest => b.created_at.cmp(&a.created_at).then(a.id.cmp(&b.id)),
            });
            Ok(defs.into_iter().skip(offset as usize).take(limit as usize).collect())
        }

        async fn get_game_definition_grantees(&self, id: Uuid) -> Result<Vec<Uuid>, StoreError> {
            Ok(self.def_grantees.get(&id).cloned().unwrap_or_default())
        }

        async fn get_game_definition_grantee_summaries(&self, id: Uuid) -> Result<Vec<GranteeSummary>, StoreError> {
            let ids = self.def_grantees.get(&id).cloned().unwrap_or_default();
            let mut out: Vec<GranteeSummary> = ids
                .into_iter()
                .filter_map(|gid| self.users.get(&gid).map(|u| GranteeSummary { id: gid, username: u.user.username.clone(), avatar_updated_at: u.user.avatar_updated_at }))
                .collect();
            out.sort_by(|a, b| a.username.cmp(&b.username));
            Ok(out)
        }

        async fn set_game_definition_image(&mut self, owner: &User, id: Uuid, png_bytes: Vec<u8>) -> Result<(), StoreError> {
            let def = self.game_definitions.iter_mut().find(|d| d.id == id && d.owner_id == owner.id)
                .ok_or_else(|| StoreError::GameDefinitionIdNotFound(id.to_string()))?;
            def.image_updated_at = Some(Utc::now());
            self.def_images.insert(id, png_bytes);
            Ok(())
        }

        async fn get_game_definition_image(&self, id: Uuid) -> Result<Option<Vec<u8>>, StoreError> {
            Ok(self.def_images.get(&id).cloned())
        }

        async fn clear_game_definition_image(&mut self, owner: &User, id: Uuid) -> Result<(), StoreError> {
            if let Some(def) = self.game_definitions.iter_mut().find(|d| d.id == id && d.owner_id == owner.id) {
                def.image_updated_at = None;
                self.def_images.remove(&id);
            }
            Ok(())
        }

        // ── Collections ──

        async fn create_game_collection(&mut self, owner: &User, collection: &mut GameCollection) -> Result<(), StoreError> {
            if collection.meta.name.trim().is_empty() {
                return Err(StoreError::GameCollectionNameMissing());
            }
            if self.game_collections.iter().any(|c| c.meta.owner_id == owner.id && c.meta.name.eq_ignore_ascii_case(&collection.meta.name)) {
                return Err(StoreError::GameCollectionNameAlreadyExists(collection.meta.name.clone()));
            }
            let count = self.game_collections.iter().filter(|c| c.meta.owner_id == owner.id).count();
            if count >= MOCK_MAX_COLLECTIONS_PER_USER {
                return Err(StoreError::GameCollectionCountLimitReached { count, max: MOCK_MAX_COLLECTIONS_PER_USER });
            }
            collection.meta.owner_id = owner.id;
            if collection.meta.id.is_nil() {
                collection.meta.id = Uuid::new_v4();
            }
            let now = Utc::now();
            collection.meta.created_at = now;
            collection.meta.updated_at = now;
            renumber_items(&mut collection.items);
            self.game_collections.push(collection.clone());
            if collection.meta.visibility == Visibility::Curated {
                self.featured_game_items_append(FeaturedGameItemKind::Collection, collection.meta.id);
            }
            Ok(())
        }

        async fn get_game_collection(&self, id: Uuid) -> Result<GameCollection, StoreError> {
            let mut collection = self.game_collections.iter().find(|c| c.meta.id == id).cloned()
                .ok_or_else(|| StoreError::GameCollectionIdNotFound(id.to_string()))?;
            collection.items.sort_by_key(|i| i.sort_order);
            Ok(collection)
        }

        async fn update_game_collection(&mut self, owner: &User, collection: &mut GameCollection) -> Result<(), StoreError> {
            let existing = self.game_collections.iter().find(|c| c.meta.id == collection.meta.id).cloned()
                .ok_or_else(|| StoreError::GameCollectionIdNotFound(collection.meta.id.to_string()))?;
            if existing.meta.owner_id != owner.id {
                return Err(StoreError::GameCollectionIdNotFound(collection.meta.id.to_string()));
            }
            if collection.meta.name.trim().is_empty() {
                return Err(StoreError::GameCollectionNameMissing());
            }
            if self.game_collections.iter().any(|c| c.meta.id != collection.meta.id && c.meta.owner_id == owner.id && c.meta.name.eq_ignore_ascii_case(&collection.meta.name)) {
                return Err(StoreError::GameCollectionNameAlreadyExists(collection.meta.name.clone()));
            }
            // Metadata-only: preserve membership + created_at.
            collection.meta.owner_id = owner.id;
            collection.meta.created_at = existing.meta.created_at;
            collection.items = existing.items;
            collection.meta.updated_at = Utc::now();
            if let Some(slot) = self.game_collection_mut(collection.meta.id) {
                *slot = collection.clone();
            }
            self.featured_game_items_reconcile(FeaturedGameItemKind::Collection, collection.meta.id, existing.meta.visibility, collection.meta.visibility);
            Ok(())
        }

        async fn delete_game_collection(&mut self, owner: &User, id: Uuid) -> Result<(), StoreError> {
            self.owned_collection_or_not_found(owner, id)?;
            self.game_collections.retain(|c| c.meta.id != id);
            self.col_grantees.remove(&id);
            self.featured_game_items_remove(FeaturedGameItemKind::Collection, id);
            Ok(())
        }

        async fn set_game_collection_items(&mut self, owner: &User, collection_id: Uuid, ordered: &[Uuid]) -> Result<(), StoreError> {
            self.owned_collection_or_not_found(owner, collection_id)?;
            let collection = self.game_collection_mut(collection_id).expect("owned collection exists");
            let mut seen = std::collections::HashSet::new();
            collection.items = ordered
                .iter()
                .filter(|id| seen.insert(**id))
                .enumerate()
                .map(|(index, id)| CollectionItem { definition_id: *id, sort_order: index as u32 })
                .collect();
            collection.meta.updated_at = Utc::now();
            Ok(())
        }

        async fn grant_game_collection_access(&mut self, owner: &User, id: Uuid, grantee: Uuid) -> Result<(), StoreError> {
            self.owned_collection_or_not_found(owner, id)?;
            let grantees = self.col_grantees.entry(id).or_default();
            if !grantees.contains(&grantee) {
                grantees.push(grantee);
            }
            Ok(())
        }

        async fn revoke_game_collection_access(&mut self, owner: &User, id: Uuid, grantee: Uuid) -> Result<(), StoreError> {
            self.owned_collection_or_not_found(owner, id)?;
            if let Some(grantees) = self.col_grantees.get_mut(&id) {
                grantees.retain(|g| *g != grantee);
            }
            Ok(())
        }

        async fn set_game_collection_grantees(&mut self, owner: &User, id: Uuid, grantees: &[Uuid]) -> Result<(), StoreError> {
            self.owned_collection_or_not_found(owner, id)?;
            let mut seen = std::collections::HashSet::new();
            let cleaned: Vec<Uuid> = grantees.iter().copied().filter(|g| *g != owner.id && seen.insert(*g)).collect();
            self.col_grantees.insert(id, cleaned);
            Ok(())
        }

        async fn get_game_collections_for_owner(&self, owner: &User) -> Result<Vec<GameCollection>, StoreError> {
            let mut cols: Vec<GameCollection> = self.game_collections.iter().filter(|c| c.meta.owner_id == owner.id).cloned().collect();
            sort_by_name_ci(&mut cols, |c| &c.meta.name);
            Ok(cols)
        }

        async fn get_visible_game_collections(&self, viewer: &User, limit: u32, offset: u32) -> Result<Vec<GameCollection>, StoreError> {
            let mut cols: Vec<GameCollection> = self.game_collections.iter()
                .filter(|c| c.meta.owner_id == viewer.id
                    || matches!(c.meta.visibility, Visibility::Public | Visibility::Curated)
                    || (c.meta.visibility == Visibility::Shared
                        && self.col_grantees.get(&c.meta.id).is_some_and(|g| g.contains(&viewer.id))))
                .cloned().collect();
            cols.sort_by(|a, b| a.meta.name.to_lowercase().cmp(&b.meta.name.to_lowercase()).then(a.meta.id.cmp(&b.meta.id)));
            Ok(cols.into_iter().skip(offset as usize).take(limit as usize).collect())
        }

        async fn get_shared_game_collections(&self, viewer: &User, limit: u32, offset: u32) -> Result<Vec<GameCollection>, StoreError> {
            let mut cols: Vec<GameCollection> = self.game_collections.iter()
                .filter(|c| c.meta.owner_id != viewer.id
                    && c.meta.visibility == Visibility::Shared
                    && self.col_grantees.get(&c.meta.id).is_some_and(|g| g.contains(&viewer.id)))
                .cloned().collect();
            cols.sort_by(|a, b| a.meta.name.to_lowercase().cmp(&b.meta.name.to_lowercase()).then(a.meta.id.cmp(&b.meta.id)));
            Ok(cols.into_iter().skip(offset as usize).take(limit as usize).collect())
        }

        async fn get_public_game_collections(&self, viewer: &User, name_query: Option<&str>, sort: GameListSort, limit: u32, offset: u32) -> Result<Vec<GameCollection>, StoreError> {
            let needle = name_query.map(|s| s.trim().to_lowercase()).filter(|s| !s.is_empty());
            let mut cols: Vec<GameCollection> = self.game_collections.iter()
                .filter(|c| c.meta.visibility == Visibility::Public
                    && c.meta.owner_id != viewer.id
                    && needle.as_ref().is_none_or(|n| c.meta.name.to_lowercase().contains(n)))
                .cloned().collect();
            cols.sort_by(|a, b| match sort {
                GameListSort::Name => a.meta.name.to_lowercase().cmp(&b.meta.name.to_lowercase()).then(a.meta.id.cmp(&b.meta.id)),
                GameListSort::Newest => b.meta.created_at.cmp(&a.meta.created_at).then(a.meta.id.cmp(&b.meta.id)),
            });
            Ok(cols.into_iter().skip(offset as usize).take(limit as usize).collect())
        }

        async fn get_game_collection_grantees(&self, id: Uuid) -> Result<Vec<Uuid>, StoreError> {
            Ok(self.col_grantees.get(&id).cloned().unwrap_or_default())
        }

        async fn get_game_collection_grantee_summaries(&self, id: Uuid) -> Result<Vec<GranteeSummary>, StoreError> {
            let ids = self.col_grantees.get(&id).cloned().unwrap_or_default();
            let mut out: Vec<GranteeSummary> = ids
                .into_iter()
                .filter_map(|gid| self.users.get(&gid).map(|u| GranteeSummary { id: gid, username: u.user.username.clone(), avatar_updated_at: u.user.avatar_updated_at }))
                .collect();
            out.sort_by(|a, b| a.username.cmp(&b.username));
            Ok(out)
        }

        async fn set_game_collection_image(&mut self, owner: &User, id: Uuid, png_bytes: Vec<u8>) -> Result<(), StoreError> {
            let collection = self.game_collections.iter_mut().find(|c| c.meta.id == id && c.meta.owner_id == owner.id)
                .ok_or_else(|| StoreError::GameCollectionIdNotFound(id.to_string()))?;
            collection.meta.image_updated_at = Some(Utc::now());
            self.col_images.insert(id, png_bytes);
            Ok(())
        }

        async fn get_game_collection_image(&self, id: Uuid) -> Result<Option<Vec<u8>>, StoreError> {
            Ok(self.col_images.get(&id).cloned())
        }

        async fn clear_game_collection_image(&mut self, owner: &User, id: Uuid) -> Result<(), StoreError> {
            if let Some(collection) = self.game_collections.iter_mut().find(|c| c.meta.id == id && c.meta.owner_id == owner.id) {
                collection.meta.image_updated_at = None;
                self.col_images.remove(&id);
            }
            Ok(())
        }

        async fn reorder_featured_game_items(&mut self, ordered: &[(FeaturedGameItemKind, Uuid)]) -> Result<(), StoreError> {
            let mut seen = std::collections::HashSet::new();
            let deduped: Vec<(FeaturedGameItemKind, Uuid)> = ordered.iter().copied().filter(|entry| seen.insert(*entry)).collect();
            for (kind, id) in &deduped {
                let visibility = match kind {
                    FeaturedGameItemKind::Definition => self.get_game_definition(*id).await?.visibility,
                    FeaturedGameItemKind::Collection => self.get_game_collection(*id).await?.meta.visibility,
                };
                if visibility != Visibility::Curated {
                    return Err(StoreError::FeaturedGameItemNotCurated { kind: kind.as_wire_str(), id: id.to_string() });
                }
            }
            self.featured_game_items = deduped;
            Ok(())
        }

        async fn list_featured_game_items(&self) -> Result<Vec<FeaturedGameItem>, StoreError> {
            let mut items = Vec::new();
            for (kind, id) in &self.featured_game_items {
                match kind {
                    FeaturedGameItemKind::Definition => {
                        if let Some(def) = self.game_definitions.iter().find(|d| d.id == *id) {
                            items.push(FeaturedGameItem::Definition(def.clone()));
                        }
                    }
                    FeaturedGameItemKind::Collection => {
                        if let Some(collection) = self.game_collections.iter().find(|c| c.meta.id == *id) {
                            items.push(FeaturedGameItem::Collection(collection.clone()));
                        }
                    }
                }
            }
            Ok(items)
        }

        async fn reconcile_featured_game_items(&mut self) -> Result<(), StoreError> {
            let have: std::collections::HashSet<(FeaturedGameItemKind, Uuid)> =
                self.featured_game_items.iter().copied().collect();
            let mut defs: Vec<(String, Uuid)> = self.game_definitions.iter()
                .filter(|d| d.visibility == Visibility::Curated)
                .map(|d| (d.name.to_lowercase(), d.id)).collect();
            defs.sort();
            for (_, id) in defs {
                if !have.contains(&(FeaturedGameItemKind::Definition, id)) {
                    self.featured_game_items.push((FeaturedGameItemKind::Definition, id));
                }
            }
            let mut cols: Vec<(String, Uuid)> = self.game_collections.iter()
                .filter(|c| c.meta.visibility == Visibility::Curated)
                .map(|c| (c.meta.name.to_lowercase(), c.meta.id)).collect();
            cols.sort();
            for (_, id) in cols {
                if !have.contains(&(FeaturedGameItemKind::Collection, id)) {
                    self.featured_game_items.push((FeaturedGameItemKind::Collection, id));
                }
            }
            Ok(())
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

    fn new_sized_maze(id: &str, name: &str, rows: usize, cols: usize) -> Maze {
        let mut maze = Maze::new(MazeDefinition::new(rows, cols));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
    }

    /// Builds a maze with 9 'K' + 8 'D' cells (17 > maze::MAX_TOTAL_FEATURES)
    /// so the handler / store K + D cap rejects it.
    fn new_too_many_features_maze(id: &str, name: &str) -> Maze {
        let mut row: Vec<char> = vec!['S'];
        row.extend(std::iter::repeat_n('K', 9));
        row.extend(std::iter::repeat_n('D', 8));
        row.push('F');
        let mut maze: Maze = Maze::new(MazeDefinition::from_vec(vec![row]));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
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
            avatar: None,
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
            let page: UsersListResponse = serde_json::from_slice(&body).expect("failed to deserialize response");
            let expected_user_items = maze_store_mock_users_to_user_items(&mock_users);
            assert_eq!(page.users, expected_user_items);
            assert!(!page.has_more, "the default page holds every mock user");
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
                avatar_updated_at: None,
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
                avatar_updated_at: None,
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

    #[actix_web::test]
    async fn cannot_create_maze_that_exceeds_cell_cap() {
        // 61 × 60 = 3,660 cells, over the MockStore cap of 3,600.
        run_create_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            true,
            new_sized_maze("", "over_cap_maze", 61, 60),
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
    }

    #[actix_web::test]
    async fn can_create_maze_with_game_settings_that_round_trips() {
        let mut maze = new_solvable_maze("", "settings_maze");
        maze.game_settings = Some(serde_json::json!({
            "skyType": "dungeon",
            "wallType": "lava",
            "timerSeconds": 90
        }));
        // run_create_maze_test asserts the POST response maze equals the input
        // (Maze equality compares full JSON), proving game_settings survives
        // the create handler's deserialize → store → response-serialize path.
        run_create_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            true,
            maze,
            StatusCode::CREATED,
        )
        .await;
    }

    #[actix_web::test]
    async fn cannot_create_maze_that_exceeds_feature_cap() {
        // 9 keys + 8 doors = 17 > maze::MAX_TOTAL_FEATURES (16). The
        // store-level validate_maze_feature_count rejects, the handler maps
        // it to 422.
        run_create_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            true,
            new_too_many_features_maze("", "over_feature_cap_maze"),
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
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

    #[actix_web::test]
    async fn cannot_update_maze_that_exceeds_cell_cap() {
        // 70 × 60 = 4,200 cells, over the MockStore cap of 3,600.
        let id = "maze_a.json";
        run_update_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::ThreeMazes),
            Some(VALID_USERNAME_1),
            true,
            id,
            new_sized_maze(id, "maze_a", 70, 60),
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_that_exceeds_feature_cap() {
        // 9 keys + 8 doors = 17 > maze::MAX_TOTAL_FEATURES (16).
        let id = "maze_a.json";
        run_update_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::ThreeMazes),
            Some(VALID_USERNAME_1),
            true,
            id,
            new_too_many_features_maze(id, "maze_a"),
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
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
            door_count: None,
            spare_doors: None,
            spare_keys: None,
            enemy_count: None,
            health_count: None,
            treasure_count: None,
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

    #[actix_web::test]
    async fn cannot_generate_maze_that_exceeds_cell_cap() {
        // 61 × 60 = 3,660 cells, over the MockStore cap of 3,600.
        // The handler rejects before invoking Generator::generate(), so the
        // dimensions need only satisfy the basic min-3 rule.
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            true,
            new_generate_options(61, 60, None, None, None, None),
            StatusCode::UNPROCESSABLE_ENTITY,
            None,
            None,
            Some(
                "Maze is too large: 61×60 = 3660 cells exceeds the 3600-cell limit"
                    .to_string(),
            ),
        )
        .await;
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
            avatar_updated_at: None,
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

    #[actix_web::test]
    async fn get_features_includes_max_maze_cells_from_store() {
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_get_request("/api/v1/features", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let response: AppFeaturesResponse =
            serde_json::from_slice(&body).expect("failed to deserialize features response");
        // MockStore reports MOCK_MAX_MAZE_CELLS = 3_600 via MazeStore::max_maze_cells.
        assert_eq!(response.max_maze_cells, Some(3_600));
    }

    // **************************************************************************************************
    // Tests: GET /api/v1/game/play3d-config
    // **************************************************************************************************
    #[actix_web::test]
    async fn get_play3d_config_returns_easy_preset_from_defaults() {
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=easy", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.difficulty, "easy");
        assert_eq!(body.rows, 8);
        assert_eq!(body.cols, 8);
        assert_eq!(body.timer_seconds, 120);
        assert_eq!(body.seed, 8_080_808);
        assert_eq!(body.min_solution_length, 12);
        assert_eq!(body.minimap_cell_px, 10);
        assert_eq!(body.minimap_radius, 5);
        assert_eq!(body.title, "Maze 3D");
        assert_eq!(body.door_count, 2);
        assert_eq!(body.spare_doors, 0);
        assert_eq!(body.spare_keys, 0);
        assert_eq!(body.enemy_count, 1);
        assert_eq!(body.health_count, 2);
        assert_eq!(body.treasure_count, 3);
        assert_eq!(body.enemy_type, "goblin");
        assert_eq!(body.health_style, "heart");
        assert_eq!(body.enemy_move_period_ms, 1800);
        assert_eq!(body.max_hp, 3);
        // Easy is a single-level run; the rest of the group is at its defaults.
        assert_eq!(body.levels.count, 1);
        assert_eq!(body.levels.finish_type, "ladder");
        assert_eq!(body.levels.difficulty_change, "easier");
        assert!(body.levels.reset_bag);
        assert_eq!(body.levels.alignment, "edge");
        assert!(!body.levels.perimeter_random);
        assert!(!body.levels.hide_completed_enemies);
        assert!(body.levels.top.is_none());
    }

    #[actix_web::test]
    async fn get_play3d_config_returns_tricky_and_hard_presets() {
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;

        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=tricky", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.difficulty, "tricky");
        assert_eq!(body.rows, 15);
        assert_eq!(body.cols, 15);
        assert_eq!(body.timer_seconds, 240);
        assert_eq!(body.seed, 15_151_515);
        assert_eq!(body.min_solution_length, 24);
        assert_eq!(body.door_count, 3);
        assert_eq!(body.spare_doors, 2);
        assert_eq!(body.spare_keys, 1);
        assert_eq!(body.enemy_count, 3);
        assert_eq!(body.health_count, 3);
        assert_eq!(body.treasure_count, 5);
        assert_eq!(body.enemy_type, "goblin");
        assert_eq!(body.health_style, "heart");
        assert_eq!(body.enemy_move_period_ms, 1500);
        assert_eq!(body.max_hp, 3);
        assert_eq!(body.levels.count, 2, "tricky is a two-level run by default");

        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=hard", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.difficulty, "hard");
        assert_eq!(body.rows, 25);
        assert_eq!(body.cols, 25);
        assert_eq!(body.timer_seconds, 420);
        assert_eq!(body.seed, 25_252_525);
        assert_eq!(body.min_solution_length, 44);
        assert_eq!(body.door_count, 4);
        assert_eq!(body.spare_doors, 3);
        assert_eq!(body.spare_keys, 1);
        assert_eq!(body.enemy_count, 5);
        assert_eq!(body.health_count, 4);
        assert_eq!(body.treasure_count, 8);
        assert_eq!(body.enemy_type, "goblin");
        assert_eq!(body.health_style, "heart");
        assert_eq!(body.enemy_move_period_ms, 1200);
        assert_eq!(body.max_hp, 3);
        assert_eq!(body.levels.count, 3, "hard is a three-level run by default");
    }

    #[actix_web::test]
    async fn get_play3d_config_returns_levels_group_and_clamps_the_count() {
        use crate::config::game::{
            DifficultyChangeConfig, FinishTypeConfig, LayeredAlignmentConfig, LevelsConfig,
            TopLevelConfig, SkyTypeConfig, MAX_LEVEL_COUNT,
        };
        let mut user_defs = vec![];
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let mut app_config = AppConfig::default();
        app_config.security.password_hash = auth::config::PasswordHashConfig::for_testing();
        app_config.comms.enabled = true;
        // A fully-specified levels group with an over-the-cap count + a top override.
        app_config.game.play3d.easy.levels = LevelsConfig {
            count: 99,
            finish_type: FinishTypeConfig::Random,
            difficulty_change: DifficultyChangeConfig::Harder,
            reset_bag: false,
            alignment: LayeredAlignmentConfig::Centre,
            taper: true,
            perimeter_random: true,
            hide_completed_enemies: true,
            top: Some(TopLevelConfig {
                sky_type: Some(SkyTypeConfig::Day),
                perimeter_walls: Some(false),
            }),
        };
        let (app, _, _, _, _) =
            create_test_app_with_config(&mut user_defs, None, false, features, app_config).await;

        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=easy", None, None);
        let resp = test::call_service(&app, req).await;
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.levels.count, MAX_LEVEL_COUNT, "count clamps to MAX_LEVEL_COUNT");
        assert_eq!(body.levels.finish_type, "random");
        assert_eq!(body.levels.difficulty_change, "harder");
        assert!(!body.levels.reset_bag);
        assert_eq!(body.levels.alignment, "centre");
        assert!(body.levels.taper);
        assert!(body.levels.perimeter_random);
        assert!(body.levels.hide_completed_enemies);
        let top = body.levels.top.expect("top override is surfaced");
        assert_eq!(top.sky_type.as_deref(), Some("day"));
        assert_eq!(top.perimeter_walls, Some(false));
    }

    #[actix_web::test]
    async fn get_play3d_config_seed_is_fixed_across_repeated_calls() {
        // Regression guard: leaderboard fairness relies on the seed being the
        // configured constant, not minted per request. Two back-to-back calls
        // must return identical seeds.
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;

        let req1 = create_test_get_request("/api/v1/game/play3d-config?difficulty=easy", None, None);
        let resp1 = test::call_service(&app, req1).await;
        let body1: Play3dConfigResponse = test::read_body_json(resp1).await;

        let req2 = create_test_get_request("/api/v1/game/play3d-config?difficulty=easy", None, None);
        let resp2 = test::call_service(&app, req2).await;
        let body2: Play3dConfigResponse = test::read_body_json(resp2).await;

        assert_eq!(body1.seed, body2.seed);
        assert_eq!(body1.min_solution_length, body2.min_solution_length);
    }

    #[actix_web::test]
    async fn get_play3d_config_normalises_case_of_difficulty_query() {
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=Easy", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.difficulty, "easy");
        assert_eq!(body.rows, 8);
    }

    #[actix_web::test]
    async fn get_play3d_config_returns_400_for_unknown_difficulty() {
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=banana", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn get_play3d_config_returns_400_when_difficulty_query_missing() {
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_get_request("/api/v1/game/play3d-config", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn get_play3d_config_no_auth_required() {
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=easy", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn get_play3d_config_respects_per_difficulty_title_override() {
        let mut user_defs = vec![];
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let mut app_config = AppConfig::default();
        app_config.security.password_hash = auth::config::PasswordHashConfig::for_testing();
        app_config.comms.enabled = true;
        app_config.game.play3d.title = "MAZE 3D DAILY".to_string();
        app_config.game.play3d.easy.title = Some("MAZE 3D — EASY".to_string());
        let (app, _, _, _, _) =
            create_test_app_with_config(&mut user_defs, None, false, features, app_config).await;

        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=easy", None, None);
        let resp = test::call_service(&app, req).await;
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.title, "MAZE 3D — EASY");

        // Tricky has no override → falls back to the parent default.
        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=tricky", None, None);
        let resp = test::call_service(&app, req).await;
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.title, "MAZE 3D DAILY");
    }

    #[actix_web::test]
    async fn get_play3d_config_returns_default_minimap_size_and_honours_overrides() {
        let mut user_defs = vec![];
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let mut app_config = AppConfig::default();
        app_config.security.password_hash = auth::config::PasswordHashConfig::for_testing();
        app_config.comms.enabled = true;
        // Easy keeps the shipped defaults; hard gets a bigger minimap.
        app_config.game.play3d.hard.minimap_cell_px = 14;
        app_config.game.play3d.hard.minimap_radius = 9;
        let (app, _, _, _, _) =
            create_test_app_with_config(&mut user_defs, None, false, features, app_config).await;

        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=easy", None, None);
        let resp = test::call_service(&app, req).await;
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.minimap_cell_px, 10);
        assert_eq!(body.minimap_radius, 5);

        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=hard", None, None);
        let resp = test::call_service(&app, req).await;
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.minimap_cell_px, 14);
        assert_eq!(body.minimap_radius, 9);
    }

    #[actix_web::test]
    async fn get_play3d_config_returns_door_and_key_holder_styles() {
        use crate::config::game::{DoorStyleConfig, KeyHolderStyleConfig};
        let mut user_defs = vec![];
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let mut app_config = AppConfig::default();
        app_config.security.password_hash = auth::config::PasswordHashConfig::for_testing();
        app_config.comms.enabled = true;
        // Easy gets explicit non-default styles; tricky keeps the defaults.
        app_config.game.play3d.easy.door_style = DoorStyleConfig::Portcullis;
        app_config.game.play3d.easy.key_holder = KeyHolderStyleConfig::Chest;
        let (app, _, _, _, _) =
            create_test_app_with_config(&mut user_defs, None, false, features, app_config).await;

        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=easy", None, None);
        let resp = test::call_service(&app, req).await;
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.door_style, "portcullis");
        assert_eq!(body.key_holder, "chest");

        let req = create_test_get_request("/api/v1/game/play3d-config?difficulty=tricky", None, None);
        let resp = test::call_service(&app, req).await;
        let body: Play3dConfigResponse = test::read_body_json(resp).await;
        assert_eq!(body.door_style, "swing");
        assert_eq!(body.key_holder, "pedestal");
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

    // -----------------------------------------------------------------------
    // POST /api/v1/scores
    // -----------------------------------------------------------------------

    /// Resolves the user id the mock store allocated for a username, so the
    /// score tests can assert the server set `user_id` from the session.
    fn caller_user_id(mock_users: &HashMap<Uuid, MockUser>, username: &str) -> Uuid {
        mock_users
            .values()
            .find(|u| u.user.username == username)
            .expect("caller present in mock store")
            .user
            .id
    }

    #[tokio::test]
    async fn record_score_with_maze_subject_succeeds() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, mock_users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        let body = RecordScoreRequest {
            maze_id: Some("My Maze.json".to_string()),
            challenge: None,
            score: 7,
            elapsed_ms: 42_137,
        };
        let req = create_test_post_request("/api/v1/scores", api_key, login_id, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let bytes = test::read_body(resp).await;
        let recorded: ScoreResponse = serde_json::from_slice(&bytes).expect("ScoreResponse");
        assert_eq!(recorded.maze_id.as_deref(), Some("My Maze.json"));
        assert_eq!(recorded.challenge, None);
        assert_eq!(recorded.score, 7);
        assert_eq!(recorded.elapsed_ms, 42_137);
        // Server-owned identity: user_id comes from the session, not the body.
        assert_eq!(recorded.user_id, caller_user_id(&mock_users, VALID_USERNAME_1));
        assert_ne!(recorded.id, Uuid::nil());
    }

    #[tokio::test]
    async fn record_score_with_challenge_subject_succeeds() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        let body = RecordScoreRequest {
            maze_id: None,
            challenge: Some("hard:12345".to_string()),
            score: 3,
            elapsed_ms: 9_001,
        };
        let req = create_test_post_request("/api/v1/scores", api_key, login_id, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);

        let bytes = test::read_body(resp).await;
        let recorded: ScoreResponse = serde_json::from_slice(&bytes).expect("ScoreResponse");
        assert_eq!(recorded.maze_id, None);
        assert_eq!(recorded.challenge.as_deref(), Some("hard:12345"));
    }

    #[tokio::test]
    async fn record_score_with_both_subjects_is_bad_request() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        let body = RecordScoreRequest {
            maze_id: Some("My Maze.json".to_string()),
            challenge: Some("hard:12345".to_string()),
            score: 1,
            elapsed_ms: 1,
        };
        let req = create_test_post_request("/api/v1/scores", api_key, login_id, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn record_score_with_no_subject_is_bad_request() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        let body = RecordScoreRequest {
            maze_id: None,
            challenge: None,
            score: 1,
            elapsed_ms: 1,
        };
        let req = create_test_post_request("/api/v1/scores", api_key, login_id, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    // No api key, no login token → the auth middleware rejects before the
    // handler runs. As elsewhere in this suite, a middleware-level rejection
    // surfaces through `call_service` as a panic (the guarded scope returns an
    // `Err`, not a response), so this is asserted via `should_panic`.
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn record_score_unauthenticated_is_rejected() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, _, _) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        let body = RecordScoreRequest {
            maze_id: Some("My Maze.json".to_string()),
            challenge: None,
            score: 1,
            elapsed_ms: 1,
        };
        let req = create_test_post_request("/api/v1/scores", None, None, Some(&body));
        test::call_service(&app, req).await;
    }

    // -----------------------------------------------------------------------
    // POST /api/v1/scores/me/completed  (campaign progress)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn completed_challenges_returns_the_scored_subset() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        // The caller scored on def:a and def:c (not def:b).
        for challenge in ["def:a", "def:c"] {
            let body = RecordScoreRequest { maze_id: None, challenge: Some(challenge.to_string()), score: 1, elapsed_ms: 1000 };
            let req = create_test_post_request("/api/v1/scores", api_key, login_id, Some(&body));
            assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CREATED);
        }

        let body = CompletedChallengesRequest { challenges: vec!["def:a".to_string(), "def:b".to_string(), "def:c".to_string()] };
        let req = create_test_post_request("/api/v1/scores/me/completed", api_key, login_id, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let out: CompletedChallengesResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        let mut completed = out.completed;
        completed.sort();
        assert_eq!(completed, vec!["def:a".to_string(), "def:c".to_string()]);
    }

    #[tokio::test]
    async fn completed_challenges_rejects_an_oversized_request() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let challenges: Vec<String> = (0..201).map(|i| format!("def:{i}")).collect();
        let body = CompletedChallengesRequest { challenges };
        let req = create_test_post_request("/api/v1/scores/me/completed", api_key, login_id, Some(&body));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn completed_challenges_unauthenticated_is_rejected() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, _, _) = create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let body = CompletedChallengesRequest { challenges: vec!["def:a".to_string()] };
        let req = create_test_post_request("/api/v1/scores/me/completed", None, None, Some(&body));
        test::call_service(&app, req).await;
    }

    #[actix_web::test]
    async fn board_dates_lists_a_daily_games_dated_boards_newest_first() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        let daily = seed_game_definition(&store, &me, "Daily", Visibility::Public, Rotation::Daily).await;
        // Two dated boards (across two users on the 14th → one distinct board) plus
        // the static board, which must NOT surface as a date.
        let dated = [
            (me.id, format!("def:{}:2026-07-14", daily.id)),
            (other.id, format!("def:{}:2026-07-14", daily.id)),
            (me.id, format!("def:{}:2026-07-15", daily.id)),
            (me.id, format!("def:{}", daily.id)),
        ];
        for (user_id, challenge) in dated {
            store.write().await.record_score(&ScoreEntry {
                id: Uuid::new_v4(), user_id, maze_id: None,
                challenge: Some(challenge), score: 1, elapsed_ms: 1, recorded_at: Utc::now(),
            }).await.expect("record");
        }

        let url = format!("/api/v1/scores/board-dates?definition_id={}", daily.id);
        let resp = test::call_service(&app, create_test_get_request(&url, Some(me.api_key), None)).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: BoardDatesResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(body.dates, vec!["2026-07-15".to_string(), "2026-07-14".to_string()]);

        // A private game owned by someone else → 403 (its board isn't readable).
        let hidden = seed_game_definition(&store, &other, "Hidden", Visibility::Private, Rotation::Daily).await;
        let url = format!("/api/v1/scores/board-dates?definition_id={}", hidden.id);
        assert_eq!(
            test::call_service(&app, create_test_get_request(&url, Some(me.api_key), None)).await.status(),
            StatusCode::FORBIDDEN
        );

        // A malformed id → 400.
        let req = create_test_get_request("/api/v1/scores/board-dates?definition_id=not-a-uuid", Some(me.api_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
    }

    // -----------------------------------------------------------------------
    // GET /api/v1/scores  (leaderboard)  +  GET /api/v1/scores/me  (history)
    // -----------------------------------------------------------------------

    /// Three runs whose time order (fastest first) and score order (highest
    /// first) are deliberately different, so a test can tell the orderings
    /// apart: by time → [B, C, A]; by score → [A, C, B].
    const SCORE_SEED: [(u64, u64); 3] = [
        // (score, elapsed_ms)
        (10, 300), // A
        (5, 100),  // B
        (8, 200),  // C
    ];

    async fn seed_maze_scores(
        app: &impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>,
        api_key: Option<Uuid>,
        login_id: Option<Uuid>,
        maze_id: &str,
    ) {
        for (score, elapsed_ms) in SCORE_SEED {
            let body = RecordScoreRequest {
                maze_id: Some(maze_id.to_string()),
                challenge: None,
                score,
                elapsed_ms,
            };
            let req = create_test_post_request("/api/v1/scores", api_key, login_id, Some(&body));
            let resp = test::call_service(app, req).await;
            assert_eq!(resp.status(), StatusCode::CREATED);
        }
    }

    async fn read_board(
        app: &impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>,
        url: &str,
        api_key: Option<Uuid>,
        login_id: Option<Uuid>,
    ) -> ScoreboardResponse {
        let req = create_test_get_request(url, api_key, login_id);
        let resp = test::call_service(app, req).await;
        assert_eq!(resp.status(), StatusCode::OK, "GET {url}");
        let bytes = test::read_body(resp).await;
        serde_json::from_slice(&bytes).expect("ScoreboardResponse")
    }

    #[tokio::test]
    async fn leaderboard_orders_by_time_then_score() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        seed_maze_scores(&app, api_key, login_id, "maze-1").await;

        // Default metric is time, default direction fastest-first → [B, C, A].
        let by_time = read_board(&app, "/api/v1/scores?maze_id=maze-1", api_key, login_id).await;
        let times: Vec<u64> = by_time.scores.iter().map(|s| s.elapsed_ms).collect();
        assert_eq!(times, vec![100, 200, 300]);
        assert!(!by_time.has_more);
        assert_eq!(by_time.offset, 0);

        // metric=score → highest-first → [A, C, B].
        let by_score =
            read_board(&app, "/api/v1/scores?maze_id=maze-1&metric=score", api_key, login_id).await;
        let scores: Vec<u64> = by_score.scores.iter().map(|s| s.score).collect();
        assert_eq!(scores, vec![10, 8, 5]);
    }

    #[tokio::test]
    async fn leaderboard_pages_with_limit_offset_and_has_more() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        seed_maze_scores(&app, api_key, login_id, "maze-1").await;

        // Page 1 (limit=2) of the time board [B, C, A] → [B, C], more to come.
        let page1 =
            read_board(&app, "/api/v1/scores?maze_id=maze-1&limit=2", api_key, login_id).await;
        assert_eq!(page1.scores.iter().map(|s| s.elapsed_ms).collect::<Vec<_>>(), vec![100, 200]);
        assert_eq!(page1.limit, 2);
        assert!(page1.has_more);

        // Page 2 (offset=2) → [A], no more.
        let page2 = read_board(
            &app,
            "/api/v1/scores?maze_id=maze-1&limit=2&offset=2",
            api_key,
            login_id,
        )
        .await;
        assert_eq!(page2.scores.iter().map(|s| s.elapsed_ms).collect::<Vec<_>>(), vec![300]);
        assert!(!page2.has_more);
    }

    #[tokio::test]
    async fn leaderboard_caps_limit_at_server_max() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        seed_maze_scores(&app, api_key, login_id, "maze-1").await;

        // Ask for far more than the cap — the effective limit echoed back is 100.
        let board =
            read_board(&app, "/api/v1/scores?maze_id=maze-1&limit=100000", api_key, login_id).await;
        assert_eq!(board.limit, 100);
        assert_eq!(board.scores.len(), 3);
        assert!(!board.has_more);
    }

    #[tokio::test]
    async fn challenge_leaderboard_reads_curated_subject() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        for (score, elapsed_ms) in SCORE_SEED {
            let body = RecordScoreRequest {
                maze_id: None,
                challenge: Some("hard:12345".to_string()),
                score,
                elapsed_ms,
            };
            let req = create_test_post_request("/api/v1/scores", api_key, login_id, Some(&body));
            assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CREATED);
        }

        let board =
            read_board(&app, "/api/v1/scores?challenge=hard:12345", api_key, login_id).await;
        assert_eq!(board.scores.iter().map(|s| s.elapsed_ms).collect::<Vec<_>>(), vec![100, 200, 300]);
        assert!(board.scores.iter().all(|s| s.challenge.as_deref() == Some("hard:12345")));
    }

    #[tokio::test]
    async fn leaderboard_requires_exactly_one_subject() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        // Neither subject.
        let req = create_test_get_request("/api/v1/scores", api_key, login_id);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);

        // Both subjects.
        let req = create_test_get_request(
            "/api/v1/scores?maze_id=maze-1&challenge=hard:1",
            api_key,
            login_id,
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
    }

    // -----------------------------------------------------------------------
    // DELETE /api/v1/scores  (reset a leaderboard)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn reset_maze_leaderboard_by_owner_clears_it() {
        // The caller owns "maze_a.json" (MazeContent::OneMaze).
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::OneMaze));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        seed_maze_scores(&app, api_key, login_id, "maze_a.json").await;
        assert_eq!(
            read_board(&app, "/api/v1/scores?maze_id=maze_a.json", api_key, login_id).await.scores.len(),
            3
        );

        let req = create_test_delete_request("/api/v1/scores?maze_id=maze_a.json", api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: ResetScoresResponse =
            serde_json::from_slice(&test::read_body(resp).await).expect("ResetScoresResponse");
        assert_eq!(body.deleted, 3);

        // The board is now empty.
        assert!(read_board(&app, "/api/v1/scores?maze_id=maze_a.json", api_key, login_id)
            .await
            .scores
            .is_empty());
    }

    #[tokio::test]
    async fn reset_maze_leaderboard_by_non_owner_is_forbidden() {
        // The caller has no mazes, so it does not own "maze_a.json".
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        // Recording a score doesn't check ownership, so a board can exist.
        seed_maze_scores(&app, api_key, login_id, "maze_a.json").await;

        let req = create_test_delete_request("/api/v1/scores?maze_id=maze_a.json", api_key, login_id);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
        // The board is untouched.
        assert_eq!(
            read_board(&app, "/api/v1/scores?maze_id=maze_a.json", api_key, login_id).await.scores.len(),
            3
        );
    }

    #[tokio::test]
    async fn reset_challenge_leaderboard_by_admin_clears_it() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 0, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_ADMIN_USERNAME_1), true).await;
        for (score, elapsed_ms) in SCORE_SEED {
            let body = RecordScoreRequest {
                maze_id: None,
                challenge: Some("hard:12345".to_string()),
                score,
                elapsed_ms,
            };
            let req = create_test_post_request("/api/v1/scores", api_key, login_id, Some(&body));
            assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CREATED);
        }

        let req = create_test_delete_request("/api/v1/scores?challenge=hard:12345", api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: ResetScoresResponse =
            serde_json::from_slice(&test::read_body(resp).await).expect("ResetScoresResponse");
        assert_eq!(body.deleted, 3);
        assert!(read_board(&app, "/api/v1/scores?challenge=hard:12345", api_key, login_id)
            .await
            .scores
            .is_empty());
    }

    #[tokio::test]
    async fn reset_challenge_leaderboard_by_non_admin_is_forbidden() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let req = create_test_delete_request("/api/v1/scores?challenge=hard:12345", api_key, login_id);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn reset_leaderboard_requires_exactly_one_subject() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 0, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_ADMIN_USERNAME_1), true).await;
        // Neither subject.
        let req = create_test_delete_request("/api/v1/scores", api_key, login_id);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
        // Both subjects.
        let req = create_test_delete_request(
            "/api/v1/scores?maze_id=maze_a.json&challenge=hard:1",
            api_key,
            login_id,
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn reset_leaderboard_unauthenticated_is_rejected() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, _, _) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let req = create_test_delete_request("/api/v1/scores?challenge=hard:1", None, None);
        test::call_service(&app, req).await;
    }

    #[tokio::test]
    async fn leaderboard_rejects_bad_metric_and_direction() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        let req = create_test_get_request(
            "/api/v1/scores?maze_id=maze-1&metric=bogus",
            api_key,
            login_id,
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);

        let req = create_test_get_request(
            "/api/v1/scores?maze_id=maze-1&direction=sideways",
            api_key,
            login_id,
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn history_returns_callers_runs_paged() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        seed_maze_scores(&app, api_key, login_id, "maze-1").await;

        // All three runs belong to the caller; page size 2 → has_more.
        let page1 = read_board(&app, "/api/v1/scores/me?limit=2", api_key, login_id).await;
        assert_eq!(page1.scores.len(), 2);
        assert!(page1.has_more);
        assert!(page1.scores.iter().all(|s| s.maze_id.as_deref() == Some("maze-1")));

        let page2 = read_board(&app, "/api/v1/scores/me?limit=2&offset=2", api_key, login_id).await;
        assert_eq!(page2.scores.len(), 1);
        assert!(!page2.has_more);
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn leaderboard_unauthenticated_is_rejected() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, _, _) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let req = create_test_get_request("/api/v1/scores?maze_id=maze-1", None, None);
        test::call_service(&app, req).await;
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn history_unauthenticated_is_rejected() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, _, _, _, _) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let req = create_test_get_request("/api/v1/scores/me", None, None);
        test::call_service(&app, req).await;
    }

    // -----------------------------------------------------------------------
    // GET /api/v1/scores — player usernames (include_usernames)
    // -----------------------------------------------------------------------

    /// The api key the mock store allocated for a username, so a test can post a
    /// run as a player other than the caller.
    fn api_key_for(mock_users: &HashMap<Uuid, MockUser>, username: &str) -> Uuid {
        mock_users
            .values()
            .find(|u| u.user.username == username)
            .expect("user present in mock store")
            .user
            .api_key
    }

    async fn post_challenge_score(
        app: &impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>,
        api_key: Option<Uuid>,
        login_id: Option<Uuid>,
        challenge: &str,
        score: u64,
        elapsed_ms: u64,
    ) {
        let body = RecordScoreRequest {
            maze_id: None,
            challenge: Some(challenge.to_string()),
            score,
            elapsed_ms,
        };
        let req = create_test_post_request("/api/v1/scores", api_key, login_id, Some(&body));
        assert_eq!(test::call_service(app, req).await.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn leaderboard_includes_usernames_for_multiple_players() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 2, MazeContent::Empty));
        let (app, _, mock_users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let user2_key = api_key_for(&mock_users, VALID_USERNAME_2);

        // user_1 (the caller) posts via login token; user_2 posts via api key.
        post_challenge_score(&app, api_key, login_id, "easy:1", 5, 1000).await;
        post_challenge_score(&app, Some(user2_key), None, "easy:1", 9, 2000).await;

        // Default (param omitted) → usernames resolved + present for both players.
        let board = read_board(&app, "/api/v1/scores?challenge=easy:1", api_key, login_id).await;
        assert_eq!(board.scores.len(), 2);
        let names: std::collections::HashSet<String> =
            board.scores.iter().filter_map(|s| s.username.clone()).collect();
        assert!(names.contains(VALID_USERNAME_1), "caller username present: {names:?}");
        assert!(names.contains(VALID_USERNAME_2), "other player username present: {names:?}");
    }

    #[tokio::test]
    async fn leaderboard_omits_usernames_when_excluded() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 2, MazeContent::Empty));
        let (app, _, mock_users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let user2_key = api_key_for(&mock_users, VALID_USERNAME_2);
        post_challenge_score(&app, api_key, login_id, "easy:1", 5, 1000).await;
        post_challenge_score(&app, Some(user2_key), None, "easy:1", 9, 2000).await;

        // include_usernames=false → every row's username is absent.
        let board = read_board(
            &app,
            "/api/v1/scores?challenge=easy:1&include_usernames=false",
            api_key,
            login_id,
        )
        .await;
        assert_eq!(board.scores.len(), 2);
        assert!(board.scores.iter().all(|s| s.username.is_none()));

        // Explicit include_usernames=true → present (parity with the default).
        let board_true = read_board(
            &app,
            "/api/v1/scores?challenge=easy:1&include_usernames=true",
            api_key,
            login_id,
        )
        .await;
        assert!(board_true.scores.iter().all(|s| s.username.is_some()));
    }

    // **************************************************************************************************
    // Tests: avatar endpoints
    //   POST   /api/v1/users/me/avatar
    //   DELETE /api/v1/users/me/avatar
    //   GET    /api/v1/users/{id}/avatar
    // **************************************************************************************************

    /// Encodes a tiny solid-colour image to the given format for upload tests.
    fn encode_test_image(width: u32, height: u32, format: image::ImageFormat) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(width, height, image::Rgb([20, 120, 200]));
        let mut bytes = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut std::io::Cursor::new(&mut bytes), format)
            .expect("encode test image");
        bytes
    }

    /// Builds a `multipart/form-data` body with a single `file` part, returning
    /// the body bytes and the boundary to put on the Content-Type header.
    fn multipart_file_body(filename: &str, content_type: &str, data: &[u8]) -> (Vec<u8>, String) {
        let boundary = "avatartestboundary7f3c".to_string();
        let mut body = Vec::new();
        body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
        body.extend_from_slice(
            format!("Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n")
                .as_bytes(),
        );
        body.extend_from_slice(format!("Content-Type: {content_type}\r\n\r\n").as_bytes());
        body.extend_from_slice(data);
        body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
        (body, boundary)
    }

    /// Builds an authenticated `POST /users/me/avatar` multipart request.
    fn avatar_upload_request(
        boundary: &str,
        body: Vec<u8>,
        login_id: Uuid,
    ) -> actix_http::Request {
        test::TestRequest::post()
            .uri("/api/v1/users/me/avatar")
            .insert_header(("Authorization", format!("Bearer {login_id}")))
            .insert_header((
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            ))
            .set_payload(body)
            .to_request()
    }

    const PNG_SIGNATURE: [u8; 4] = [0x89, 0x50, 0x4E, 0x47];

    #[actix_web::test]
    async fn upload_avatar_stores_canonical_png_and_serves_it() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, _users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let login = login_id.expect("login id");

        // Upload a non-square PNG → 200 + the new marker.
        let png = encode_test_image(10, 20, image::ImageFormat::Png);
        let (body, boundary) = multipart_file_body("a.png", "image/png", &png);
        let resp = test::call_service(&app, avatar_upload_request(&boundary, body, login)).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let _updated: crate::api::v1::endpoints::avatar::AvatarUpdatedResponse =
            test::read_body_json(resp).await;

        // The profile now carries the marker.
        let me_resp =
            test::call_service(&app, create_test_get_request("/api/v1/users/me", api_key, login_id))
                .await;
        let me: UserItem = test::read_body_json(me_resp).await;
        assert!(
            me.avatar_updated_at.is_some(),
            "profile must carry avatar_updated_at after upload"
        );

        // GET the avatar as a signed-in viewer — any id is readable when authed.
        let get_resp = test::call_service(
            &app,
            create_test_get_request(&format!("/api/v1/users/{}/avatar", me.id), api_key, login_id),
        )
        .await;
        assert_eq!(get_resp.status(), StatusCode::OK);
        let content_type = get_resp
            .headers()
            .get(actix_web::http::header::CONTENT_TYPE)
            .expect("content-type")
            .to_str()
            .expect("ascii content-type")
            .to_string();
        assert_eq!(content_type, "image/png");
        let served = test::read_body(get_resp).await;
        assert_eq!(&served[..4], &PNG_SIGNATURE, "served bytes must be a PNG");
    }

    #[actix_web::test]
    async fn upload_avatar_converts_jpeg_to_png() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, _users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let login = login_id.expect("login id");

        let jpeg = encode_test_image(12, 12, image::ImageFormat::Jpeg);
        let (body, boundary) = multipart_file_body("a.jpg", "image/jpeg", &jpeg);
        let resp = test::call_service(&app, avatar_upload_request(&boundary, body, login)).await;
        assert_eq!(resp.status(), StatusCode::OK);

        let me_resp =
            test::call_service(&app, create_test_get_request("/api/v1/users/me", api_key, login_id))
                .await;
        let me: UserItem = test::read_body_json(me_resp).await;
        let get_resp = test::call_service(
            &app,
            create_test_get_request(&format!("/api/v1/users/{}/avatar", me.id), api_key, login_id),
        )
        .await;
        assert_eq!(get_resp.status(), StatusCode::OK);
        let served = test::read_body(get_resp).await;
        assert_eq!(
            &served[..4],
            &PNG_SIGNATURE,
            "a JPEG upload must be re-encoded and served as PNG"
        );
    }

    #[actix_web::test]
    async fn upload_avatar_rejects_non_image() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, _users, _api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let login = login_id.expect("login id");

        // Bytes that aren't a decodable image — even though the part claims PNG,
        // the server validates by decoding, so this is rejected.
        let (body, boundary) = multipart_file_body("a.png", "image/png", b"this is not an image");
        let resp = test::call_service(&app, avatar_upload_request(&boundary, body, login)).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    #[should_panic]
    async fn upload_avatar_unauthenticated_fails() {
        // The auth middleware short-circuits unauthenticated guarded requests
        // with an error, which `call_service` surfaces as a panic — the same
        // convention the other `*_unauthenticated_fails` tests use.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, _users, _api_key, _login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        let png = encode_test_image(8, 8, image::ImageFormat::Png);
        let (body, boundary) = multipart_file_body("a.png", "image/png", &png);
        let req = test::TestRequest::post()
            .uri("/api/v1/users/me/avatar")
            .insert_header((
                "Content-Type",
                format!("multipart/form-data; boundary={boundary}"),
            ))
            .set_payload(body)
            .to_request();
        let _ = test::call_service(&app, req).await;
    }

    #[actix_web::test]
    async fn delete_avatar_clears_marker_and_image() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, _users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let login = login_id.expect("login id");

        // Upload, then delete.
        let png = encode_test_image(8, 8, image::ImageFormat::Png);
        let (body, boundary) = multipart_file_body("a.png", "image/png", &png);
        let up = test::call_service(&app, avatar_upload_request(&boundary, body, login)).await;
        assert_eq!(up.status(), StatusCode::OK);

        let del = test::call_service(
            &app,
            create_test_delete_request("/api/v1/users/me/avatar", api_key, login_id),
        )
        .await;
        assert_eq!(del.status(), StatusCode::NO_CONTENT);

        // Marker gone from the profile.
        let me_resp =
            test::call_service(&app, create_test_get_request("/api/v1/users/me", api_key, login_id))
                .await;
        let me: UserItem = test::read_body_json(me_resp).await;
        assert!(
            me.avatar_updated_at.is_none(),
            "avatar_updated_at must be cleared after delete"
        );

        // Image now 404s.
        let get_resp = test::call_service(
            &app,
            create_test_get_request(&format!("/api/v1/users/{}/avatar", me.id), api_key, login_id),
        )
        .await;
        assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn get_avatar_returns_404_when_unset() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, _users, api_key, login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;

        let me_resp =
            test::call_service(&app, create_test_get_request("/api/v1/users/me", api_key, login_id))
                .await;
        let me: UserItem = test::read_body_json(me_resp).await;

        let get_resp = test::call_service(
            &app,
            create_test_get_request(&format!("/api/v1/users/{}/avatar", me.id), api_key, login_id),
        )
        .await;
        assert_eq!(get_resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    #[should_panic]
    async fn get_avatar_unauthenticated_fails() {
        // The serve route is guarded like the rest of the API — an
        // unauthenticated GET is rejected by the auth middleware (surfaced as a
        // call_service panic, matching the other *_unauthenticated_fails tests).
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, _users, _api_key, _login_id) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), true).await;
        let req = test::TestRequest::get()
            .uri(&format!("/api/v1/users/{}/avatar", Uuid::new_v4()))
            .to_request();
        let _ = test::call_service(&app, req).await;
    }

    // ****************************************************************************
    // Game-definition endpoint tests (POST/GET/PUT/DELETE + shares + publish)
    // ****************************************************************************

    fn user_by_name(mock_users: &HashMap<Uuid, MockUser>, username: &str) -> User {
        MockStore::find_user_by_name_in_map(mock_users, username, Uuid::nil())
            .expect("mock user exists")
    }

    fn sample_game_config() -> serde_json::Value {
        serde_json::json!({ "rows": 6, "cols": 6, "seed": 0, "levels": { "count": 2 } })
    }

    fn definition_request(name: &str, visibility: Visibility, rotation: Rotation) -> GameDefinitionRequest {
        GameDefinitionRequest {
            name: name.to_string(),
            description: None,
            visibility,
            rotation,
            config: sample_game_config(),
        }
    }

    /// Seeds a definition straight into the store (storage does no access policy),
    /// returning it with its minted id — the fixture for the read/publish tests.
    async fn seed_game_definition(
        store: &SharedStore,
        owner: &User,
        name: &str,
        visibility: Visibility,
        rotation: Rotation,
    ) -> GameDefinition {
        let now = Utc::now();
        let mut def = GameDefinition {
            id: Uuid::nil(),
            owner_id: Uuid::nil(),
            name: name.to_string(),
            description: None,
            visibility,
            seed: 12_345,
            rotation,
            config: sample_game_config(),
            image_updated_at: None,
            created_at: now,
            updated_at: now,
        };
        store.write().await.create_game_definition(owner, &mut def).await.expect("seed definition");
        def
    }

    async fn record_challenge_score(store: &SharedStore, user: &User, challenge: &str) {
        let entry = ScoreEntry {
            id: Uuid::new_v4(),
            user_id: user.id,
            maze_id: None,
            challenge: Some(challenge.to_string()),
            score: 1,
            elapsed_ms: 1000,
            recorded_at: Utc::now(),
        };
        store.write().await.record_score(&entry).await.expect("record score");
    }

    async fn challenge_board_len(store: &SharedStore, challenge: &str) -> usize {
        store
            .read()
            .await
            .challenge_leaderboard(
                challenge,
                ScoreOrdering { metric: ScoreMetric::Time, direction: SortDirection::Ascending },
                100,
                0,
                false,
            )
            .await
            .expect("read board")
            .len()
    }

    #[actix_web::test]
    async fn game_definition_get_one_enforces_access_matrix() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _api_key, _login) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;

        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        let private = seed_game_definition(&store, &owner, "Private", Visibility::Private, Rotation::Static).await;
        let shared = seed_game_definition(&store, &owner, "Shared", Visibility::Shared, Rotation::Static).await;
        let public = seed_game_definition(&store, &owner, "Public", Visibility::Public, Rotation::Static).await;
        let curated = seed_game_definition(&store, &owner, "Curated", Visibility::Curated, Rotation::Static).await;
        store.write().await.grant_game_definition_access(&owner, shared.id, other.id).await.expect("grant");

        // (definition id, viewer, expected status) — owner sees all; a shared def
        // only its grantee; curated/public anyone; an admin gets no special view.
        let cases = [
            (private.id, VALID_USERNAME_1, StatusCode::OK),
            (private.id, VALID_USERNAME_2, StatusCode::NOT_FOUND),
            (private.id, VALID_ADMIN_USERNAME_1, StatusCode::NOT_FOUND),
            (shared.id, VALID_USERNAME_1, StatusCode::OK),
            (shared.id, VALID_USERNAME_2, StatusCode::OK),
            (shared.id, VALID_ADMIN_USERNAME_1, StatusCode::NOT_FOUND),
            (public.id, VALID_USERNAME_2, StatusCode::OK),
            (curated.id, VALID_USERNAME_2, StatusCode::OK),
        ];
        for (id, viewer, expected) in cases {
            let key = api_key_for(&mock_users, viewer);
            let req = create_test_get_request(&format!("/api/v1/game-definitions/{id}"), Some(key), None);
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), expected, "definition {id} viewed by {viewer}");
        }

        // An unknown id is a 404 (indistinguishable from inaccessible).
        let key = api_key_for(&mock_users, VALID_USERNAME_1);
        let req = create_test_get_request(&format!("/api/v1/game-definitions/{}", Uuid::new_v4()), Some(key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn game_definition_play_fetch_computes_subject_and_tracking() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        let static_def = seed_game_definition(&store, &owner, "Static", Visibility::Public, Rotation::Static).await;
        let daily_def = seed_game_definition(&store, &owner, "Daily", Visibility::Public, Rotation::Daily).await;

        // Static: date-less subject, config.seed == the stored seed, tracked.
        let req = create_test_get_request(&format!("/api/v1/game-definitions/{}", static_def.id), Some(key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(body["challengeKey"], serde_json::json!(format!("def:{}", static_def.id)));
        assert_eq!(body["leaderboardTracked"], serde_json::json!(true));
        assert_eq!(body["config"]["seed"], serde_json::json!(12_345u64));

        // Daily: subject carries today's UTC date and config.seed is date-mixed.
        let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
        let req = create_test_get_request(&format!("/api/v1/game-definitions/{}", daily_def.id), Some(key), None);
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(body["challengeKey"], serde_json::json!(format!("def:{}:{}", daily_def.id, today)));
        assert_ne!(body["config"]["seed"], serde_json::json!(12_345u64));
    }

    #[actix_web::test]
    async fn create_game_definition_mints_seed_and_rejects_duplicate_name() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        let body = definition_request("My Game", Visibility::Private, Rotation::Static);
        let req = create_test_post_request("/api/v1/game-definitions", Some(key), None, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let created: GameDefinition = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert!(!created.id.is_nil());
        assert_eq!(created.owner_id, owner.id);
        assert_eq!(created.visibility, Visibility::Private);

        // Same name for the same owner → 409.
        let req = create_test_post_request("/api/v1/game-definitions", Some(key), None, Some(&body));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn create_curated_definition_requires_admin() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let body = definition_request("Featured", Visibility::Curated, Rotation::Static);

        // Non-admin cannot set curated.
        let user_key = api_key_for(&mock_users, VALID_USERNAME_1);
        let req = create_test_post_request("/api/v1/game-definitions", Some(user_key), None, Some(&body));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);

        // Admin can.
        let admin_key = api_key_for(&mock_users, VALID_ADMIN_USERNAME_1);
        let req = create_test_post_request("/api/v1/game-definitions", Some(admin_key), None, Some(&body));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn publishing_a_definition_keeps_its_board() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        let def = seed_game_definition(&store, &owner, "Draft", Visibility::Private, Rotation::Static).await;
        let challenge = format!("def:{}", def.id);
        record_challenge_score(&store, &owner, &challenge).await;
        record_challenge_score(&store, &owner, &challenge).await;
        assert_eq!(challenge_board_len(&store, &challenge).await, 2);

        // Private → public with no gameplay change is a pure publish — the board
        // carries over (only a gameplay change resets it).
        let body = definition_request("Draft", Visibility::Public, Rotation::Static);
        let req = create_test_put_request(&format!("/api/v1/game-definitions/{}", def.id), Some(key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        assert_eq!(challenge_board_len(&store, &challenge).await, 2);
    }

    #[actix_web::test]
    async fn saving_a_gameplay_change_resets_the_board() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        let def = seed_game_definition(&store, &owner, "Draft", Visibility::Private, Rotation::Static).await;
        let challenge = format!("def:{}", def.id);
        record_challenge_score(&store, &owner, &challenge).await;
        assert_eq!(challenge_board_len(&store, &challenge).await, 1);

        // Changing the grid is a gameplay change → the board is reset.
        let mut body = definition_request("Draft", Visibility::Private, Rotation::Static);
        body.config = serde_json::json!({ "rows": 9, "cols": 6, "seed": 0, "levels": { "count": 2 } });
        let req = create_test_put_request(&format!("/api/v1/game-definitions/{}", def.id), Some(key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        assert_eq!(challenge_board_len(&store, &challenge).await, 0);
    }

    #[actix_web::test]
    async fn saving_a_cosmetic_change_keeps_the_board() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        let def = seed_game_definition(&store, &owner, "Draft", Visibility::Private, Rotation::Static).await;
        let challenge = format!("def:{}", def.id);
        record_challenge_score(&store, &owner, &challenge).await;

        // Only cosmetic keys change (title / status label) → the board is kept.
        let mut body = definition_request("Renamed", Visibility::Private, Rotation::Static);
        body.description = Some("A note".to_string());
        body.config = serde_json::json!({ "rows": 6, "cols": 6, "seed": 0, "levels": { "count": 2 }, "title": "Splash", "mode": "Label" });
        let req = create_test_put_request(&format!("/api/v1/game-definitions/{}", def.id), Some(key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        assert_eq!(challenge_board_len(&store, &challenge).await, 1);
    }

    #[actix_web::test]
    async fn a_private_games_board_is_owner_only() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        let def = seed_game_definition(&store, &owner, "Solo", Visibility::Private, Rotation::Static).await;
        let challenge = format!("def:{}", def.id);
        record_challenge_score(&store, &owner, &challenge).await;
        let url = format!("/api/v1/scores?challenge={challenge}");

        // The owner reads their private game's board; nobody else can.
        let req = create_test_get_request(&url, Some(owner.api_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        let req = create_test_get_request(&url, Some(other.api_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);

        // Nor can a non-owner record a run on it.
        let post = RecordScoreRequest { maze_id: None, challenge: Some(challenge.clone()), score: 5, elapsed_ms: 1000 };
        let req = create_test_post_request("/api/v1/scores", Some(other.api_key), None, Some(&post));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn deleting_a_definition_resets_its_board_and_removes_it() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        let def = seed_game_definition(&store, &owner, "Doomed", Visibility::Public, Rotation::Static).await;
        let challenge = format!("def:{}", def.id);
        record_challenge_score(&store, &owner, &challenge).await;
        assert_eq!(challenge_board_len(&store, &challenge).await, 1);

        let req = create_test_delete_request(&format!("/api/v1/game-definitions/{}", def.id), Some(key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        assert_eq!(challenge_board_len(&store, &challenge).await, 0);

        let req = create_test_get_request(&format!("/api/v1/game-definitions/{}", def.id), Some(key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn reshuffling_a_definition_changes_its_seed() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        // seed_game_definition stamps a fixed seed (12_345); reshuffle re-mints it.
        let def = seed_game_definition(&store, &owner, "Draft", Visibility::Private, Rotation::Static).await;
        assert_eq!(def.seed, 12_345);

        let req = create_test_post_request(
            &format!("/api/v1/game-definitions/{}/reshuffle", def.id),
            Some(key),
            None,
            None::<&()>,
        );
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let reshuffled: GameDefinition = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_ne!(reshuffled.seed, 12_345, "the seed must be re-minted");
        assert_eq!(reshuffled.id, def.id);
    }

    #[actix_web::test]
    async fn reshuffling_a_published_definition_resets_its_board() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        let def = seed_game_definition(&store, &owner, "Live", Visibility::Public, Rotation::Static).await;
        let challenge = format!("def:{}", def.id);
        record_challenge_score(&store, &owner, &challenge).await;
        record_challenge_score(&store, &owner, &challenge).await;
        assert_eq!(challenge_board_len(&store, &challenge).await, 2);

        let req = create_test_post_request(
            &format!("/api/v1/game-definitions/{}/reshuffle", def.id),
            Some(key),
            None,
            None::<&()>,
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        assert_eq!(challenge_board_len(&store, &challenge).await, 0, "the board is wiped on reshuffle");
    }

    #[actix_web::test]
    async fn reshuffling_another_users_definition_returns_404() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        let def = seed_game_definition(&store, &owner, "Mine", Visibility::Private, Rotation::Static).await;

        // A non-owner cannot reshuffle — reported as absent, not forbidden.
        let req = create_test_post_request(
            &format!("/api/v1/game-definitions/{}/reshuffle", def.id),
            Some(other.api_key),
            None,
            None::<&()>,
        );
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn list_game_definitions_merges_visible_sources() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);
        let admin = user_by_name(&mock_users, VALID_ADMIN_USERNAME_1);

        let my_private = seed_game_definition(&store, &me, "A my private", Visibility::Private, Rotation::Static).await;
        let my_public = seed_game_definition(&store, &me, "B my public", Visibility::Public, Rotation::Static).await;
        let others_public = seed_game_definition(&store, &other, "C others public", Visibility::Public, Rotation::Static).await;
        let shared_to_me = seed_game_definition(&store, &other, "D shared to me", Visibility::Shared, Rotation::Static).await;
        store.write().await.grant_game_definition_access(&other, shared_to_me.id, me.id).await.expect("grant");
        let others_private = seed_game_definition(&store, &other, "E others private", Visibility::Private, Rotation::Static).await;
        let curated = seed_game_definition(&store, &admin, "F curated", Visibility::Curated, Rotation::Static).await;

        let req = create_test_get_request("/api/v1/game-definitions", Some(me.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let list: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(resp).await).expect("json");
        let ids: Vec<Uuid> = list.definitions.iter().map(|d| d.id).collect();
        assert!(ids.contains(&my_private.id));
        assert!(ids.contains(&my_public.id));
        assert!(ids.contains(&others_public.id));
        assert!(ids.contains(&shared_to_me.id));
        assert!(ids.contains(&curated.id));
        assert!(!ids.contains(&others_private.id), "another user's private draft must not appear");

        // De-duplicated and ordered by name (case-insensitive).
        assert_eq!(ids.len(), 5);
        let names: Vec<String> = list.definitions.iter().map(|d| d.name.clone()).collect();
        let mut sorted = names.clone();
        sorted.sort_by_key(|n| n.to_lowercase());
        assert_eq!(names, sorted);
    }

    #[actix_web::test]
    async fn list_game_definitions_pages_the_merged_result() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);

        // Three visible definitions, which sort P1 < P2 < P3 by name.
        seed_game_definition(&store, &me, "P1", Visibility::Public, Rotation::Static).await;
        seed_game_definition(&store, &me, "P2", Visibility::Public, Rotation::Static).await;
        seed_game_definition(&store, &me, "P3", Visibility::Public, Rotation::Static).await;

        // First page of two → P1, P2, more to come.
        let req = create_test_get_request("/api/v1/game-definitions?limit=2&offset=0", Some(me.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let page: GameDefinitionListResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(page.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["P1", "P2"]);
        assert_eq!(page.limit, 2);
        assert_eq!(page.offset, 0);
        assert!(page.has_more);

        // Last page → the remaining P3, nothing beyond.
        let req = create_test_get_request("/api/v1/game-definitions?limit=2&offset=2", Some(me.api_key), None);
        let page: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["P3"]);
        assert!(!page.has_more);

        // An over-cap limit is silently capped and echoed back.
        let req = create_test_get_request("/api/v1/game-definitions?limit=1000", Some(me.api_key), None);
        let page: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.limit, 100);
        assert_eq!(page.definitions.len(), 3);
        assert!(!page.has_more);
    }

    #[actix_web::test]
    async fn list_game_definitions_scope_mine_returns_only_own_filters_and_pages() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        // Mine (any visibility, curated included) + others' visible ones that
        // scope=mine must exclude.
        seed_game_definition(&store, &me, "Alpha", Visibility::Private, Rotation::Static).await;
        seed_game_definition(&store, &me, "Beta", Visibility::Public, Rotation::Static).await;
        seed_game_definition(&store, &me, "Gamma", Visibility::Curated, Rotation::Static).await;
        let others_public = seed_game_definition(&store, &other, "Others public", Visibility::Public, Rotation::Static).await;
        let shared = seed_game_definition(&store, &other, "Shared to me", Visibility::Shared, Rotation::Static).await;
        store.write().await.grant_game_definition_access(&other, shared.id, me.id).await.expect("grant");

        // scope=mine → only my three (curated included), name-ordered; none of the others'.
        let req = create_test_get_request("/api/v1/game-definitions?scope=mine", Some(me.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let list: GameDefinitionListResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(list.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["Alpha", "Beta", "Gamma"]);
        let ids: Vec<Uuid> = list.definitions.iter().map(|d| d.id).collect();
        assert!(!ids.contains(&others_public.id));
        assert!(!ids.contains(&shared.id));

        // q filters by case-insensitive name substring within scope=mine.
        let req = create_test_get_request("/api/v1/game-definitions?scope=mine&q=ET", Some(me.api_key), None);
        let page: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["Beta"]);

        // Paging over the own set: first two, then the remainder.
        let req = create_test_get_request("/api/v1/game-definitions?scope=mine&limit=2&offset=0", Some(me.api_key), None);
        let page: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["Alpha", "Beta"]);
        assert!(page.has_more);
        let req = create_test_get_request("/api/v1/game-definitions?scope=mine&limit=2&offset=2", Some(me.api_key), None);
        let page: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["Gamma"]);
        assert!(!page.has_more);
    }

    #[actix_web::test]
    async fn list_game_definitions_scope_shared_returns_only_grants() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        // My own shared one (excluded), and others' public/curated/ungranted-shared
        // (all excluded) — only the one granted to me should appear.
        let my_own = seed_game_definition(&store, &me, "My Own", Visibility::Shared, Rotation::Static).await;
        let others_public = seed_game_definition(&store, &other, "Public", Visibility::Public, Rotation::Static).await;
        let others_curated = seed_game_definition(&store, &other, "Curated", Visibility::Curated, Rotation::Static).await;
        let shared = seed_game_definition(&store, &other, "Shared to me", Visibility::Shared, Rotation::Static).await;
        store.write().await.grant_game_definition_access(&other, shared.id, me.id).await.expect("grant");
        let ungranted = seed_game_definition(&store, &other, "Shared to others", Visibility::Shared, Rotation::Static).await;

        let req = create_test_get_request("/api/v1/game-definitions?scope=shared", Some(me.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let list: GameDefinitionListResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(list.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["Shared to me"]);
        let ids: Vec<Uuid> = list.definitions.iter().map(|d| d.id).collect();
        assert!(!ids.contains(&my_own.id));
        assert!(!ids.contains(&others_public.id));
        assert!(!ids.contains(&others_curated.id));
        assert!(!ids.contains(&ungranted.id));
    }

    #[actix_web::test]
    async fn list_game_definitions_scope_public_returns_cross_owner_and_filters() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        // My own public one (excluded — cross-owner), others' public (included) +
        // curated/private (excluded).
        let my_own = seed_game_definition(&store, &me, "My Public", Visibility::Public, Rotation::Static).await;
        seed_game_definition(&store, &other, "Public Sky", Visibility::Public, Rotation::Static).await;
        seed_game_definition(&store, &other, "Public Cave", Visibility::Public, Rotation::Static).await;
        let curated = seed_game_definition(&store, &other, "Curated", Visibility::Curated, Rotation::Static).await;

        let req = create_test_get_request("/api/v1/game-definitions?scope=public", Some(me.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let list: GameDefinitionListResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(list.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["Public Cave", "Public Sky"]);
        let ids: Vec<Uuid> = list.definitions.iter().map(|d| d.id).collect();
        assert!(!ids.contains(&my_own.id));
        assert!(!ids.contains(&curated.id));

        // q narrows the public pool case-insensitively.
        let req = create_test_get_request("/api/v1/game-definitions?scope=public&q=SKY", Some(me.api_key), None);
        let page: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["Public Sky"]);

        // An unknown sort is rejected rather than silently ignored.
        let req = create_test_get_request("/api/v1/game-definitions?scope=public&sort=bogus", Some(me.api_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn list_game_definitions_sort_newest_orders_by_creation() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        // Names run opposite to creation order, so the two sorts can't agree. The
        // store stamps `created_at` itself, so pause between seeds to guarantee
        // distinct stamps (equal ones fall back to the random-uuid tiebreak).
        seed_game_definition(&store, &other, "Alpha", Visibility::Public, Rotation::Static).await;
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        seed_game_definition(&store, &other, "Zulu", Visibility::Public, Rotation::Static).await;

        // Default (name) ordering.
        let req = create_test_get_request("/api/v1/game-definitions?scope=public", Some(me.api_key), None);
        let list: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(list.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["Alpha", "Zulu"]);

        // sort=newest reverses it (Zulu was created last).
        let req = create_test_get_request("/api/v1/game-definitions?scope=public&sort=newest", Some(me.api_key), None);
        let list: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(list.definitions.iter().map(|d| d.name.clone()).collect::<Vec<_>>(), vec!["Zulu", "Alpha"]);
    }

    #[actix_web::test]
    async fn list_game_definitions_exclude_definitions_blanks_config() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        seed_game_definition(&store, &me, "Alpha", Visibility::Private, Rotation::Static).await;

        // Default: the full opaque config is included.
        let req = create_test_get_request("/api/v1/game-definitions?scope=mine", Some(me.api_key), None);
        let list: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert!(list.definitions[0].config.get("rows").is_some(), "config included by default");

        // excludeDefinitions=true blanks the config; the light metadata stays.
        let req = create_test_get_request("/api/v1/game-definitions?scope=mine&excludeDefinitions=true", Some(me.api_key), None);
        let list: GameDefinitionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(list.definitions[0].name, "Alpha");
        assert_eq!(list.definitions[0].config, serde_json::json!({}), "config blanked when excludeDefinitions=true");
    }

    #[actix_web::test]
    async fn list_game_definitions_rejects_an_unknown_scope() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let req = create_test_get_request("/api/v1/game-definitions?scope=bogus", Some(me.api_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
    }

    /// Reads a definition's current visibility straight from the store.
    async fn definition_visibility(store: &SharedStore, id: Uuid) -> Visibility {
        store.read().await.get_game_definition(id).await.expect("definition").visibility
    }

    #[actix_web::test]
    async fn definition_shares_set_reconciles_the_list_and_is_owner_only() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);
        let def = seed_game_definition(&store, &owner, "Shared", Visibility::Shared, Rotation::Static).await;
        let url = format!("/api/v1/game-definitions/{}/shares", def.id);

        // Set the list to {other} — plus the owner's own id, which is ignored.
        let body = SetGameSharesRequest { user_ids: vec![other.id, owner.id] };
        let req = create_test_put_request(&url, Some(owner.api_key), None, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let shares: GameDefinitionSharesResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(shares.grantees, vec![GranteeSummary { id: other.id, username: VALID_USERNAME_2.to_string(), avatar_updated_at: None }]);

        // Replace with an empty list — clears it.
        let body = SetGameSharesRequest { user_ids: vec![] };
        let req = create_test_put_request(&url, Some(owner.api_key), None, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let shares: GameDefinitionSharesResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert!(shares.grantees.is_empty());

        // A non-owner cannot read or set the list.
        let req = create_test_get_request(&url, Some(other.api_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
        let body = SetGameSharesRequest { user_ids: vec![Uuid::new_v4()] };
        let req = create_test_put_request(&url, Some(other.api_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn setting_a_definitions_shares_leaves_its_tier_untouched() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        // A private game keeps its tier when a share list is set — visibility is
        // set explicitly, not inferred from the grant list.
        let def = seed_game_definition(&store, &owner, "Draft", Visibility::Private, Rotation::Static).await;
        let body = SetGameSharesRequest { user_ids: vec![other.id] };
        let req = create_test_put_request(&format!("/api/v1/game-definitions/{}/shares", def.id), Some(owner.api_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        assert_eq!(definition_visibility(&store, def.id).await, Visibility::Private);
    }

    // ****************************************************************************
    // Game-collection endpoint tests (CRUD + membership + shares + detail filter)
    // ****************************************************************************

    fn collection_request(name: &str, visibility: Visibility) -> GameCollectionRequest {
        GameCollectionRequest {
            name: name.to_string(),
            description: None,
            visibility,
            play_mode: PlayMode::Arcade,
        }
    }

    /// Seeds an empty collection straight into the store, returning it with its
    /// minted id.
    async fn seed_game_collection(
        store: &SharedStore,
        owner: &User,
        name: &str,
        visibility: Visibility,
    ) -> GameCollection {
        let now = Utc::now();
        let mut collection = GameCollection {
            meta: GameCollectionMeta {
                id: Uuid::nil(),
                owner_id: Uuid::nil(),
                name: name.to_string(),
                visibility,
                play_mode: PlayMode::Arcade,
                description: None,
                image_updated_at: None,
                created_at: now,
                updated_at: now,
            },
            items: Vec::new(),
        };
        store.write().await.create_game_collection(owner, &mut collection).await.expect("seed collection");
        collection
    }

    /// The ordered member ids of a `GameCollection` JSON response body.
    fn collection_item_ids(collection: &GameCollection) -> Vec<Uuid> {
        collection.items.iter().map(|i| i.definition_id).collect()
    }

    /// The ordered member definition ids of a collection-detail JSON body.
    fn detail_definition_ids(body: &serde_json::Value) -> Vec<String> {
        body["definitions"]
            .as_array()
            .expect("definitions array")
            .iter()
            .map(|d| d["id"].as_str().expect("id").to_string())
            .collect()
    }

    #[actix_web::test]
    async fn create_game_collection_defaults_and_rejects_duplicate_and_gates_curated() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        // Create → 201, empty, owned by the caller.
        let body = collection_request("My Set", Visibility::Private);
        let req = create_test_post_request("/api/v1/game-collections", Some(key), None, Some(&body));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let created: GameCollection = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert!(!created.meta.id.is_nil());
        assert_eq!(created.meta.owner_id, owner.id);
        assert!(created.items.is_empty());
        assert_eq!(created.meta.play_mode, PlayMode::Arcade);

        // Duplicate name for the same owner → 409.
        let req = create_test_post_request("/api/v1/game-collections", Some(key), None, Some(&body));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CONFLICT);

        // Curated requires an admin.
        let curated = collection_request("Featured", Visibility::Curated);
        let req = create_test_post_request("/api/v1/game-collections", Some(key), None, Some(&curated));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::FORBIDDEN);
        let admin_key = api_key_for(&mock_users, VALID_ADMIN_USERNAME_1);
        let req = create_test_post_request("/api/v1/game-collections", Some(admin_key), None, Some(&curated));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CREATED);
    }

    #[actix_web::test]
    async fn game_collection_play_mode_defaults_and_round_trips() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let key = owner.api_key;

        // A create body that omits playMode entirely → defaults to Arcade.
        let omitted = serde_json::json!({ "name": "No Mode", "visibility": "private" });
        let req = create_test_post_request("/api/v1/game-collections", Some(key), None, Some(&omitted));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let created: GameCollection = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(created.meta.play_mode, PlayMode::Arcade);

        // Create with campaign → round-trips on the create body and the detail GET.
        let campaign = serde_json::json!({ "name": "Campaign", "visibility": "private", "playMode": "campaign" });
        let req = create_test_post_request("/api/v1/game-collections", Some(key), None, Some(&campaign));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let created: GameCollection = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(created.meta.play_mode, PlayMode::Campaign);
        let detail: serde_json::Value = serde_json::from_slice(
            &test::read_body(
                test::call_service(
                    &app,
                    create_test_get_request(&format!("/api/v1/game-collections/{}", created.meta.id), Some(key), None),
                )
                .await,
            )
            .await,
        )
        .expect("json");
        assert_eq!(detail["playMode"], "campaign");

        // Update back to arcade → persists.
        let edit = serde_json::json!({ "name": "Campaign", "visibility": "private", "playMode": "arcade" });
        let req = create_test_put_request(&format!("/api/v1/game-collections/{}", created.meta.id), Some(key), None, &edit);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let updated: GameCollection = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(updated.meta.play_mode, PlayMode::Arcade);
    }

    #[actix_web::test]
    async fn game_collection_detail_filters_members_to_the_viewer() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        // A public collection owned by `owner` with four members, added in order.
        let collection = seed_game_collection(&store, &owner, "Mixed", Visibility::Public).await;
        let def_public = seed_game_definition(&store, &owner, "Pub", Visibility::Public, Rotation::Static).await;
        let def_owner_private = seed_game_definition(&store, &owner, "OwnerPriv", Visibility::Private, Rotation::Static).await;
        let def_other_private = seed_game_definition(&store, &other, "OtherPriv", Visibility::Private, Rotation::Static).await;
        store
            .write()
            .await
            .set_game_collection_items(
                &owner,
                collection.meta.id,
                &[def_public.id, def_owner_private.id, def_other_private.id, Uuid::new_v4()],
            )
            .await
            .expect("set members");

        // The owner sees the public member + their own private one (not the other
        // user's private, not the dangling ref), in insertion order.
        let req = create_test_get_request(&format!("/api/v1/game-collections/{}", collection.meta.id), Some(owner.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(detail_definition_ids(&body), vec![def_public.id.to_string(), def_owner_private.id.to_string()]);

        // The other user sees the public member + their own private one.
        let req = create_test_get_request(&format!("/api/v1/game-collections/{}", collection.meta.id), Some(other.api_key), None);
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(detail_definition_ids(&body), vec![def_public.id.to_string(), def_other_private.id.to_string()]);

        // A private collection is invisible to a non-owner.
        let private = seed_game_collection(&store, &owner, "Secret", Visibility::Private).await;
        let req = create_test_get_request(&format!("/api/v1/game-collections/{}", private.meta.id), Some(other.api_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn game_collection_membership_reconcile() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        let collection = seed_game_collection(&store, &owner, "Set", Visibility::Private).await;
        let d1 = seed_game_definition(&store, &owner, "D1", Visibility::Public, Rotation::Static).await;
        let d2 = seed_game_definition(&store, &owner, "D2", Visibility::Public, Rotation::Static).await;
        let d3 = seed_game_definition(&store, &owner, "D3", Visibility::Public, Rotation::Static).await;
        let url = format!("/api/v1/game-collections/{}", collection.meta.id);

        // Set the membership in one call → stored in the given order.
        let body = SetGameCollectionItemsRequest { definition_ids: vec![d1.id, d2.id, d3.id] };
        let req = create_test_put_request(&format!("{url}/items"), Some(owner.api_key), None, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let set: GameCollection = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(collection_item_ids(&set), vec![d1.id, d2.id, d3.id]);

        // Reconcile in one call: drop d1, reorder to d3, d2.
        let body = SetGameCollectionItemsRequest { definition_ids: vec![d3.id, d2.id] };
        let req = create_test_put_request(&format!("{url}/items"), Some(owner.api_key), None, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let after: GameCollection = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(collection_item_ids(&after), vec![d3.id, d2.id]);

        // A non-owner cannot mutate membership.
        let body = SetGameCollectionItemsRequest { definition_ids: vec![d1.id] };
        let req = create_test_put_request(&format!("{url}/items"), Some(other.api_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn list_game_collections_merges_and_pages() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);
        let admin = user_by_name(&mock_users, VALID_ADMIN_USERNAME_1);

        let mine = seed_game_collection(&store, &me, "A mine", Visibility::Private).await;
        let public = seed_game_collection(&store, &other, "B public", Visibility::Public).await;
        let shared = seed_game_collection(&store, &other, "C shared", Visibility::Shared).await;
        store.write().await.grant_game_collection_access(&other, shared.meta.id, me.id).await.expect("grant");
        let curated = seed_game_collection(&store, &admin, "D curated", Visibility::Curated).await;
        let others_private = seed_game_collection(&store, &other, "E others private", Visibility::Private).await;

        // Full list: mine + public + shared-with-me + curated, not the other's private.
        let req = create_test_get_request("/api/v1/game-collections", Some(me.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let list: GameCollectionListResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        let ids: Vec<Uuid> = list.collections.iter().map(|c| c.meta.id).collect();
        assert!(ids.contains(&mine.meta.id));
        assert!(ids.contains(&public.meta.id));
        assert!(ids.contains(&shared.meta.id));
        assert!(ids.contains(&curated.meta.id));
        assert!(!ids.contains(&others_private.meta.id));
        assert_eq!(ids.len(), 4);

        // First page of two (sorted A, B, C, D) → A, B, more to come.
        let req = create_test_get_request("/api/v1/game-collections?limit=2&offset=0", Some(me.api_key), None);
        let page: GameCollectionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.collections.iter().map(|c| c.meta.name.clone()).collect::<Vec<_>>(), vec!["A mine", "B public"]);
        assert_eq!(page.limit, 2);
        assert!(page.has_more);

        // Over-cap limit is capped and echoed back.
        let req = create_test_get_request("/api/v1/game-collections?limit=1000", Some(me.api_key), None);
        let page: GameCollectionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.limit, 100);
        assert!(!page.has_more);
    }

    #[actix_web::test]
    async fn list_game_collections_scope_mine_returns_only_own_and_filters() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        seed_game_collection(&store, &me, "Alpha", Visibility::Private).await;
        seed_game_collection(&store, &me, "Beta", Visibility::Public).await;
        let others_public = seed_game_collection(&store, &other, "Others public", Visibility::Public).await;
        let shared = seed_game_collection(&store, &other, "Shared to me", Visibility::Shared).await;
        store.write().await.grant_game_collection_access(&other, shared.meta.id, me.id).await.expect("grant");

        // scope=mine → only my two, name-ordered; none of the others'.
        let req = create_test_get_request("/api/v1/game-collections?scope=mine", Some(me.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let list: GameCollectionListResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(list.collections.iter().map(|c| c.meta.name.clone()).collect::<Vec<_>>(), vec!["Alpha", "Beta"]);
        let ids: Vec<Uuid> = list.collections.iter().map(|c| c.meta.id).collect();
        assert!(!ids.contains(&others_public.meta.id));
        assert!(!ids.contains(&shared.meta.id));

        // q filters case-insensitively within scope=mine.
        let req = create_test_get_request("/api/v1/game-collections?scope=mine&q=ALPHA", Some(me.api_key), None);
        let page: GameCollectionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.collections.iter().map(|c| c.meta.name.clone()).collect::<Vec<_>>(), vec!["Alpha"]);

        // Invalid scope → 400.
        let req = create_test_get_request("/api/v1/game-collections?scope=bogus", Some(me.api_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
    }

    #[actix_web::test]
    async fn list_game_collections_scope_shared_returns_only_grants() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        let my_own = seed_game_collection(&store, &me, "My Own", Visibility::Shared).await;
        let others_public = seed_game_collection(&store, &other, "Public", Visibility::Public).await;
        let shared = seed_game_collection(&store, &other, "Shared to me", Visibility::Shared).await;
        store.write().await.grant_game_collection_access(&other, shared.meta.id, me.id).await.expect("grant");
        let ungranted = seed_game_collection(&store, &other, "Shared to others", Visibility::Shared).await;

        let req = create_test_get_request("/api/v1/game-collections?scope=shared", Some(me.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let list: GameCollectionListResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(list.collections.iter().map(|c| c.meta.name.clone()).collect::<Vec<_>>(), vec!["Shared to me"]);
        let ids: Vec<Uuid> = list.collections.iter().map(|c| c.meta.id).collect();
        assert!(!ids.contains(&my_own.meta.id));
        assert!(!ids.contains(&others_public.meta.id));
        assert!(!ids.contains(&ungranted.meta.id));
    }

    #[actix_web::test]
    async fn list_game_collections_scope_public_returns_cross_owner_and_filters() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let me = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        let my_own = seed_game_collection(&store, &me, "My Public", Visibility::Public).await;
        seed_game_collection(&store, &other, "Open Sky", Visibility::Public).await;
        let private = seed_game_collection(&store, &other, "Private Set", Visibility::Private).await;

        let req = create_test_get_request("/api/v1/game-collections?scope=public", Some(me.api_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let list: GameCollectionListResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(list.collections.iter().map(|c| c.meta.name.clone()).collect::<Vec<_>>(), vec!["Open Sky"]);
        let ids: Vec<Uuid> = list.collections.iter().map(|c| c.meta.id).collect();
        assert!(!ids.contains(&my_own.meta.id));
        assert!(!ids.contains(&private.meta.id));

        let req = create_test_get_request("/api/v1/game-collections?scope=public&q=sky", Some(me.api_key), None);
        let page: GameCollectionListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.collections.iter().map(|c| c.meta.name.clone()).collect::<Vec<_>>(), vec!["Open Sky"]);
    }

    #[actix_web::test]
    async fn collection_shares_set_reconciles_the_list_and_is_owner_only() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);

        let collection = seed_game_collection(&store, &owner, "Shared", Visibility::Shared).await;
        let url = format!("/api/v1/game-collections/{}/shares", collection.meta.id);

        let body = SetGameSharesRequest { user_ids: vec![other.id, owner.id] };
        let req = create_test_put_request(&url, Some(owner.api_key), None, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let shares: GameCollectionSharesResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(shares.grantees, vec![GranteeSummary { id: other.id, username: VALID_USERNAME_2.to_string(), avatar_updated_at: None }]);

        // Empty clears it.
        let body = SetGameSharesRequest { user_ids: vec![] };
        let req = create_test_put_request(&url, Some(owner.api_key), None, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let shares: GameCollectionSharesResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert!(shares.grantees.is_empty());

        // Owner-only: a non-owner can neither read nor set.
        let req = create_test_get_request(&url, Some(other.api_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
        let body = SetGameSharesRequest { user_ids: vec![Uuid::new_v4()] };
        let req = create_test_put_request(&url, Some(other.api_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    // ****************************************************************************
    // User lookup (username prefix search) tests
    // ****************************************************************************

    async fn lookup_users_page(
        app: &impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>,
        key: Uuid,
        query: &str,
    ) -> UserLookupResponse {
        let req = create_test_get_request(&format!("/api/v1/users/lookup?{query}"), Some(key), None);
        let resp = test::call_service(app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        serde_json::from_slice(&test::read_body(resp).await).expect("json")
    }

    #[actix_web::test]
    async fn user_lookup_matches_username_prefix_case_insensitively() {
        // 3 users (user_1..user_3) + 1 admin (admin_1); caller is a non-admin.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 3, MazeContent::Empty));
        let (app, _store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let key = api_key_for(&mock_users, VALID_USERNAME_1);

        // Prefix "user" → the three user_* accounts, ordered, not the admin.
        let page = lookup_users_page(&app, key, "username=user").await;
        assert_eq!(
            page.users.iter().map(|u| u.username.clone()).collect::<Vec<_>>(),
            vec!["user_1", "user_2", "user_3"]
        );
        assert!(!page.has_more);

        // Case-insensitive.
        let upper = lookup_users_page(&app, key, "username=USER").await;
        assert_eq!(upper.users.len(), 3);

        // A different prefix matches its own set.
        let admins = lookup_users_page(&app, key, "username=admin").await;
        assert_eq!(admins.users.iter().map(|u| u.username.clone()).collect::<Vec<_>>(), vec!["admin_1"]);

        // Prefix, not substring: "ser" is inside "user_*" but matches nothing.
        assert!(lookup_users_page(&app, key, "username=ser").await.users.is_empty());

        // A blank / absent prefix never lists everyone.
        assert!(lookup_users_page(&app, key, "username=").await.users.is_empty());
        assert!(lookup_users_page(&app, key, "").await.users.is_empty());

        // Only id + username are exposed — no email/admin/avatar leak.
        let req = create_test_get_request("/api/v1/users/lookup?username=user", Some(key), None);
        let body: serde_json::Value = serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        let first = body["users"][0].as_object().expect("entry object");
        assert_eq!(first.keys().cloned().collect::<std::collections::BTreeSet<_>>(),
            ["id", "username"].iter().map(|s| s.to_string()).collect::<std::collections::BTreeSet<_>>());
    }

    #[actix_web::test]
    async fn user_lookup_pages_the_matches() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 3, MazeContent::Empty));
        let (app, _store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let key = api_key_for(&mock_users, VALID_USERNAME_1);

        // First page of two → user_1, user_2, more to come.
        let first = lookup_users_page(&app, key, "username=user&limit=2&offset=0").await;
        assert_eq!(first.users.iter().map(|u| u.username.clone()).collect::<Vec<_>>(), vec!["user_1", "user_2"]);
        assert_eq!(first.limit, 2);
        assert!(first.has_more);

        // Last page → user_3, nothing beyond.
        let last = lookup_users_page(&app, key, "username=user&limit=2&offset=2").await;
        assert_eq!(last.users.iter().map(|u| u.username.clone()).collect::<Vec<_>>(), vec!["user_3"]);
        assert!(!last.has_more);

        // Over-cap limit is capped and echoed back.
        let capped = lookup_users_page(&app, key, "username=user&limit=1000").await;
        assert_eq!(capped.limit, 100);
        assert!(!capped.has_more);
    }

    // ****************************************************************************
    // Game definition / collection image endpoint tests
    // ****************************************************************************

    /// Builds an authenticated multipart `POST` to an image endpoint.
    fn image_upload_request(url: &str, api_key: Uuid, boundary: &str, body: Vec<u8>) -> actix_http::Request {
        test::TestRequest::post()
            .uri(url)
            .insert_header(("X-API-KEY", api_key.to_string()))
            .insert_header(("Content-Type", format!("multipart/form-data; boundary={boundary}")))
            .set_payload(body)
            .to_request()
    }

    #[actix_web::test]
    async fn game_definition_image_upload_serve_and_delete() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);
        let def = seed_game_definition(&store, &owner, "Framed", Visibility::Public, Rotation::Static).await;
        let url = format!("/api/v1/game-definitions/{}/image", def.id);

        // Upload a JPEG → canonicalised, stored, marker returned.
        let jpeg = encode_test_image(10, 20, image::ImageFormat::Jpeg);
        let (body, boundary) = multipart_file_body("g.jpg", "image/jpeg", &jpeg);
        let resp = test::call_service(&app, image_upload_request(&url, owner.api_key, &boundary, body)).await;
        assert_eq!(resp.status(), StatusCode::OK);

        // A public definition's image is served to any signed-in viewer as PNG.
        let get = test::call_service(&app, create_test_get_request(&url, Some(other.api_key), None)).await;
        assert_eq!(get.status(), StatusCode::OK);
        assert_eq!(
            get.headers().get(actix_web::http::header::CONTENT_TYPE).unwrap().to_str().unwrap(),
            "image/png"
        );
        let served = test::read_body(get).await;
        assert_eq!(&served[..4], &PNG_SIGNATURE, "served bytes must be a PNG");

        // Delete → 204, then the image is gone.
        let del = test::call_service(&app, create_test_delete_request(&url, Some(owner.api_key), None)).await;
        assert_eq!(del.status(), StatusCode::NO_CONTENT);
        let gone = test::call_service(&app, create_test_get_request(&url, Some(owner.api_key), None)).await;
        assert_eq!(gone.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn game_definition_image_enforces_access_and_ownership() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);
        let def = seed_game_definition(&store, &owner, "Private", Visibility::Private, Rotation::Static).await;
        let url = format!("/api/v1/game-definitions/{}/image", def.id);
        let png = encode_test_image(8, 8, image::ImageFormat::Png);

        // Owner uploads.
        let (body, boundary) = multipart_file_body("g.png", "image/png", &png);
        assert_eq!(
            test::call_service(&app, image_upload_request(&url, owner.api_key, &boundary, body)).await.status(),
            StatusCode::OK
        );

        // A private definition's image is invisible to a non-owner…
        assert_eq!(
            test::call_service(&app, create_test_get_request(&url, Some(other.api_key), None)).await.status(),
            StatusCode::NOT_FOUND
        );
        // …and a non-owner can neither upload nor delete it.
        let (body2, boundary2) = multipart_file_body("g.png", "image/png", &png);
        assert_eq!(
            test::call_service(&app, image_upload_request(&url, other.api_key, &boundary2, body2)).await.status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            test::call_service(&app, create_test_delete_request(&url, Some(other.api_key), None)).await.status(),
            StatusCode::NOT_FOUND
        );

        // A non-image upload is rejected.
        let (bad, bad_boundary) = multipart_file_body("x.png", "image/png", b"not an image");
        assert_eq!(
            test::call_service(&app, image_upload_request(&url, owner.api_key, &bad_boundary, bad)).await.status(),
            StatusCode::BAD_REQUEST
        );
    }

    #[actix_web::test]
    async fn game_collection_image_upload_serve_and_delete() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let other = user_by_name(&mock_users, VALID_USERNAME_2);
        let col = seed_game_collection(&store, &owner, "Framed", Visibility::Public).await;
        let url = format!("/api/v1/game-collections/{}/image", col.meta.id);

        let png = encode_test_image(16, 8, image::ImageFormat::Png);
        let (body, boundary) = multipart_file_body("c.png", "image/png", &png);
        assert_eq!(
            test::call_service(&app, image_upload_request(&url, owner.api_key, &boundary, body)).await.status(),
            StatusCode::OK
        );

        let get = test::call_service(&app, create_test_get_request(&url, Some(other.api_key), None)).await;
        assert_eq!(get.status(), StatusCode::OK);
        let served = test::read_body(get).await;
        assert_eq!(&served[..4], &PNG_SIGNATURE);

        let del = test::call_service(&app, create_test_delete_request(&url, Some(owner.api_key), None)).await;
        assert_eq!(del.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            test::call_service(&app, create_test_get_request(&url, Some(owner.api_key), None)).await.status(),
            StatusCode::NOT_FOUND
        );

        // A non-owner cannot upload to someone else's collection.
        let (body2, boundary2) = multipart_file_body("c.png", "image/png", &png);
        assert_eq!(
            test::call_service(&app, image_upload_request(&url, other.api_key, &boundary2, body2)).await.status(),
            StatusCode::NOT_FOUND
        );
    }

    // ****************************************************************************
    // Paged admin user list + per-user maze cap
    // ****************************************************************************

    #[actix_web::test]
    async fn get_users_pages_the_admin_list() {
        // 1 admin + 4 users = 5 total.
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 4, MazeContent::Empty));
        let (app, _store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_ADMIN_USERNAME_1), false).await;
        let admin_key = api_key_for(&mock_users, VALID_ADMIN_USERNAME_1);

        // First page of two → more to come.
        let req = create_test_get_request("/api/v1/users?limit=2&offset=0", Some(admin_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let page: UsersListResponse = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(page.users.len(), 2);
        assert_eq!(page.limit, 2);
        assert!(page.has_more);

        // Over-cap limit is capped and returns everyone.
        let req = create_test_get_request("/api/v1/users?limit=1000", Some(admin_key), None);
        let page: UsersListResponse =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(page.limit, 100);
        assert_eq!(page.users.len(), 5);
        assert!(!page.has_more);

        // The endpoint is admin-only.
        let user_key = api_key_for(&mock_users, VALID_USERNAME_1);
        let req = create_test_get_request("/api/v1/users?limit=2", Some(user_key), None);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn create_maze_returns_409_when_user_at_maze_cap() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let user = user_by_name(&mock_users, VALID_USERNAME_1);

        // Fill the user up to the MockStore cap directly via the store.
        {
            let mut lock = store.write().await;
            let cap = lock.max_mazes_per_user().expect("mock store reports a maze cap");
            for i in 0..cap {
                let mut m = new_sized_maze(&format!("m{i}.json"), &format!("m{i}"), 3, 3);
                lock.create_maze(&user, &mut m).await.expect("seed maze under cap");
            }
        }

        // The next create via the API is refused with 409.
        let over = new_solvable_maze("over.json", "over");
        let req = create_test_post_request("/api/v1/mazes", Some(user.api_key), None, Some(&over));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn create_game_definition_returns_409_at_cap() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);

        // Fill the user up to the MockStore definition cap.
        for i in 0..MOCK_MAX_DEFINITIONS_PER_USER {
            seed_game_definition(&store, &owner, &format!("D{i}"), Visibility::Private, Rotation::Static).await;
        }
        // The next create via the API is refused with 409.
        let body = definition_request("Over", Visibility::Private, Rotation::Static);
        let req = create_test_post_request("/api/v1/game-definitions", Some(owner.api_key), None, Some(&body));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CONFLICT);
    }

    #[actix_web::test]
    async fn create_game_collection_returns_409_at_cap() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);

        for i in 0..MOCK_MAX_COLLECTIONS_PER_USER {
            seed_game_collection(&store, &owner, &format!("C{i}"), Visibility::Private).await;
        }
        let body = collection_request("Over", Visibility::Private);
        let req = create_test_post_request("/api/v1/game-collections", Some(owner.api_key), None, Some(&body));
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::CONFLICT);
    }

    // ── Admin-override on update + the featured catalogue endpoints ──────────

    /// The ordered `(kind, id)` pairs of a featured-list JSON body.
    fn featured_game_item_ids(body: &serde_json::Value) -> Vec<String> {
        body["items"]
            .as_array()
            .expect("items array")
            .iter()
            .map(|item| {
                let entity = item.get("definition").or_else(|| item.get("collection")).expect("entity");
                entity["id"].as_str().expect("id").to_string()
            })
            .collect()
    }

    #[actix_web::test]
    async fn admin_override_update_game_definition_features_and_preserves_owner() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);

        let def = seed_game_definition(&store, &owner, "Owned", Visibility::Private, Rotation::Static).await;

        // A non-owner non-admin cannot edit it — reported as absent (404).
        let stranger_key = api_key_for(&mock_users, VALID_USERNAME_2);
        let body = definition_request("Owned", Visibility::Public, Rotation::Static);
        let req = create_test_put_request(&format!("/api/v1/game-definitions/{}", def.id), Some(stranger_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);

        // An admin editing this non-featured definition they don't own, WITHOUT
        // featuring it, is denied — ownership is ignored only for Featured
        // definitions (or when featuring one).
        let admin_key = api_key_for(&mock_users, VALID_ADMIN_USERNAME_1);
        let body = definition_request("Owned", Visibility::Public, Rotation::Static);
        let req = create_test_put_request(&format!("/api/v1/game-definitions/{}", def.id), Some(admin_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);

        // An admin may feature it (body sets curated); ownership stays with the owner.
        let body = definition_request("Owned", Visibility::Curated, Rotation::Static);
        let req = create_test_put_request(&format!("/api/v1/game-definitions/{}", def.id), Some(admin_key), None, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let updated: GameDefinition = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(updated.owner_id, owner.id, "admin edit preserves ownership, no transfer");
        assert_eq!(updated.visibility, Visibility::Curated);

        // It now shows in the featured catalogue.
        let req = create_test_get_request("/api/v1/featured-game-items", Some(admin_key), None);
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(body["items"][0]["kind"], serde_json::json!("definition"));
        assert_eq!(featured_game_item_ids(&body), vec![def.id.to_string()]);

        // Now Featured, an admin may un-feature it (override applies because it is
        // currently curated) — and it drops off the catalogue.
        let body = definition_request("Owned", Visibility::Public, Rotation::Static);
        let req = create_test_put_request(&format!("/api/v1/game-definitions/{}", def.id), Some(admin_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        let req = create_test_get_request("/api/v1/featured-game-items", Some(admin_key), None);
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert!(featured_game_item_ids(&body).is_empty(), "un-featured definition leaves the catalogue");
    }

    #[actix_web::test]
    async fn admin_override_update_game_collection_features_and_preserves_owner() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);

        let col = seed_game_collection(&store, &owner, "Owned Set", Visibility::Private).await;

        // Non-owner non-admin → 404.
        let stranger_key = api_key_for(&mock_users, VALID_USERNAME_2);
        let body = collection_request("Owned Set", Visibility::Public);
        let req = create_test_put_request(&format!("/api/v1/game-collections/{}", col.meta.id), Some(stranger_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);

        // An admin editing this non-featured collection they don't own, WITHOUT
        // featuring it (visibility stays non-curated), is denied — ownership is
        // ignored only for Featured collections (or when featuring one).
        let admin_key = api_key_for(&mock_users, VALID_ADMIN_USERNAME_1);
        let body = collection_request("Owned Set", Visibility::Public);
        let req = create_test_put_request(&format!("/api/v1/game-collections/{}", col.meta.id), Some(admin_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);

        // Admin features it (body sets curated); ownership preserved.
        let body = collection_request("Owned Set", Visibility::Curated);
        let req = create_test_put_request(&format!("/api/v1/game-collections/{}", col.meta.id), Some(admin_key), None, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let updated: GameCollection = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(updated.meta.owner_id, owner.id, "admin edit preserves ownership");
        assert_eq!(updated.meta.visibility, Visibility::Curated);

        let req = create_test_get_request("/api/v1/featured-game-items", Some(admin_key), None);
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(body["items"][0]["kind"], serde_json::json!("collection"));
        assert_eq!(featured_game_item_ids(&body), vec![col.meta.id.to_string()]);

        // Now that it is Featured, an admin may also un-feature it (the override
        // applies because the collection is currently curated, even though the new
        // visibility is not) — and it drops off the featured catalogue.
        let body = collection_request("Owned Set", Visibility::Public);
        let req = create_test_put_request(&format!("/api/v1/game-collections/{}", col.meta.id), Some(admin_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::OK);
        let req = create_test_get_request("/api/v1/featured-game-items", Some(admin_key), None);
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert!(featured_game_item_ids(&body).is_empty(), "un-featured collection leaves the catalogue");
    }

    #[actix_web::test]
    async fn admin_can_edit_a_featured_collections_games_but_not_a_private_one() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 2, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let owner = user_by_name(&mock_users, VALID_USERNAME_1);
        let admin_key = api_key_for(&mock_users, VALID_ADMIN_USERNAME_1);
        let stranger_key = api_key_for(&mock_users, VALID_USERNAME_2);

        let game = seed_game_definition(&store, &owner, "G", Visibility::Public, Rotation::Static).await;
        let body = SetGameCollectionItemsRequest { definition_ids: vec![game.id] };

        // A Featured (curated) collection owned by user1: an admin (non-owner) may
        // set its games — curating the featured set — and ownership is preserved.
        let featured = seed_game_collection(&store, &owner, "Featured Set", Visibility::Curated).await;
        let req = create_test_put_request(&format!("/api/v1/game-collections/{}/items", featured.meta.id), Some(admin_key), None, &body);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let updated: GameCollection = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(updated.meta.owner_id, owner.id, "ownership preserved (no transfer)");
        assert_eq!(collection_item_ids(&updated), vec![game.id]);

        // A stranger (non-owner non-admin) still cannot.
        let req = create_test_put_request(&format!("/api/v1/game-collections/{}/items", featured.meta.id), Some(stranger_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);

        // A PRIVATE collection owned by user1: ownership is ignored only for
        // Featured collections, so even an admin cannot edit its games.
        let private = seed_game_collection(&store, &owner, "Private Set", Visibility::Private).await;
        let req = create_test_put_request(&format!("/api/v1/game-collections/{}/items", private.meta.id), Some(admin_key), None, &body);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn featured_game_items_list_is_ordered_and_paged() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_ADMIN_USERNAME_1), false).await;
        let admin = user_by_name(&mock_users, VALID_ADMIN_USERNAME_1);
        let key = admin.api_key;

        // Featured order = the order they became curated: def A, collection B, def C.
        let a = seed_game_definition(&store, &admin, "A", Visibility::Curated, Rotation::Static).await;
        let b = seed_game_collection(&store, &admin, "B", Visibility::Curated).await;
        let c = seed_game_definition(&store, &admin, "C", Visibility::Curated, Rotation::Static).await;

        // Page 1 (limit 2): A, B — more remain.
        let req = create_test_get_request("/api/v1/featured-game-items?limit=2&offset=0", Some(key), None);
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(featured_game_item_ids(&body), vec![a.id.to_string(), b.meta.id.to_string()]);
        assert_eq!(body["items"][1]["kind"], serde_json::json!("collection"));
        // Each item carries its owner's username, resolved server-side.
        assert_eq!(body["items"][0]["ownerUsername"], serde_json::json!(admin.username));
        assert_eq!(body["limit"], serde_json::json!(2));
        assert_eq!(body["hasMore"], serde_json::json!(true));

        // Page 2 (offset 2): C — none remain.
        let req = create_test_get_request("/api/v1/featured-game-items?limit=2&offset=2", Some(key), None);
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(featured_game_item_ids(&body), vec![c.id.to_string()]);
        assert_eq!(body["hasMore"], serde_json::json!(false));

        // Readable by any signed-in user (the catalogue is not per-viewer filtered).
        let user_key = api_key_for(&mock_users, VALID_USERNAME_1);
        let req = create_test_get_request("/api/v1/featured-game-items", Some(user_key), None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(body["items"].as_array().expect("items").len(), 3);
    }

    #[actix_web::test]
    async fn reorder_featured_game_items_reorders_and_requires_admin() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_ADMIN_USERNAME_1), false).await;
        let admin = user_by_name(&mock_users, VALID_ADMIN_USERNAME_1);

        let a = seed_game_definition(&store, &admin, "A", Visibility::Curated, Rotation::Static).await;
        let b = seed_game_collection(&store, &admin, "B", Visibility::Curated).await;
        let c = seed_game_definition(&store, &admin, "C", Visibility::Curated, Rotation::Static).await;

        let reorder = serde_json::json!({ "entries": [
            { "kind": "definition", "id": c.id.to_string() },
            { "kind": "collection", "id": b.meta.id.to_string() },
            { "kind": "definition", "id": a.id.to_string() },
        ]});

        // A non-admin cannot reorder (admin-gated → 401).
        let user_key = api_key_for(&mock_users, VALID_USERNAME_1);
        let req = create_test_put_request("/api/v1/featured-game-items/order", Some(user_key), None, &reorder);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::UNAUTHORIZED);

        // The admin reorders → 200, returns the catalogue in its new order.
        let req = create_test_put_request("/api/v1/featured-game-items/order", Some(admin.api_key), None, &reorder);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: serde_json::Value = serde_json::from_slice(&test::read_body(resp).await).expect("json");
        assert_eq!(
            featured_game_item_ids(&body),
            vec![c.id.to_string(), b.meta.id.to_string(), a.id.to_string()]
        );

        // A subsequent GET reflects the persisted new order.
        let req = create_test_get_request("/api/v1/featured-game-items", Some(admin.api_key), None);
        let body: serde_json::Value =
            serde_json::from_slice(&test::read_body(test::call_service(&app, req).await).await).expect("json");
        assert_eq!(
            featured_game_item_ids(&body),
            vec![c.id.to_string(), b.meta.id.to_string(), a.id.to_string()]
        );
    }

    #[actix_web::test]
    async fn reorder_featured_game_items_rejects_non_curated() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, store, mock_users, _k, _l) =
            create_test_app(&mut user_defs, Some(VALID_ADMIN_USERNAME_1), false).await;
        let admin = user_by_name(&mock_users, VALID_ADMIN_USERNAME_1);

        let curated = seed_game_definition(&store, &admin, "Curated", Visibility::Curated, Rotation::Static).await;
        let plain = seed_game_definition(&store, &admin, "Plain", Visibility::Public, Rotation::Static).await;

        // A reorder that includes a non-curated id is rejected wholesale (400).
        let reorder = serde_json::json!({ "entries": [
            { "kind": "definition", "id": curated.id.to_string() },
            { "kind": "definition", "id": plain.id.to_string() },
        ]});
        let req = create_test_put_request("/api/v1/featured-game-items/order", Some(admin.api_key), None, &reorder);
        assert_eq!(test::call_service(&app, req).await.status(), StatusCode::BAD_REQUEST);
    }
}
