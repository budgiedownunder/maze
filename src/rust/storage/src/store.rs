use crate::Error;
use async_trait::async_trait;
use data_model::{Maze, User, UserEmail};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a store for holding users
#[async_trait]
pub trait UserStore {
    /// Adds the default admin user to the store if it doesn't already exist, else returns it
    async fn init_default_admin_user(&mut self, username: &str, email: &str, password_hash: &str) -> Result<User, Error>;
    /// Adds a new user to the store and sets the allocated `id` within the user object
    async fn create_user(&mut self, user: &mut User) -> Result<(), Error>;
    /// Deletes a user from the store
    async fn delete_user(&mut self, id: Uuid) -> Result<(), Error>;
    /// Updates a user within the store
    async fn update_user(&mut self, user: &mut User) -> Result<(), Error>;
    /// Loads a user from the store
    async fn get_user(&self, id: Uuid) -> Result<User, Error>;
    /// Locates a user by their username within the store
    async fn find_user_by_name(&self, name: &str) -> Result<User, Error>;
    /// Locates a user by their email address within the store
    async fn find_user_by_email(&self, email: &str) -> Result<User, Error>;
    /// Locates a user by their api key within the store
    async fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, Error>;
    /// Locates a user by their login id within the store
    async fn find_user_by_login_id(&self, login_id: Uuid) -> Result<User, Error>;
    /// Locates a user by an OAuth identity `(provider, provider_user_id)` pair.
    /// `provider` is matched case-insensitively); `provider_user_id` is matched
    /// exactly (it is an opaque stable id from the identity provider).
    async fn find_user_by_oauth_identity(&self, provider: &str, provider_user_id: &str) -> Result<User, Error>;
    /// Returns the list of users within the store, sorted
    /// alphabetically by username in ascending order
    async fn get_users(&self) -> Result<Vec<User>, Error>;
    /// Returns the list of admin users within the store
    async fn get_admin_users(&self) -> Result<Vec<User>, Error>;
    /// Returns whether at least one user exists in the store
    async fn has_users(&self) -> Result<bool, Error>;
    /// Adds a new email row to the user. The new row is non-primary; pass
    /// `verified = true` for trusted sources (OAuth-link, admin seed) and
    /// `verified = false` for self-asserted user-typed emails. The store
    /// rejects with [`Error::UserEmailExists`] if the address is already
    /// in use by any user (mirrors the SQL `user_emails.email` UNIQUE).
    async fn add_user_email(
        &mut self,
        user_id: Uuid,
        email: &str,
        verified: bool,
    ) -> Result<UserEmail, Error>;
    /// Removes an email row from the user. Rejects with
    /// [`Error::UserEmailIsPrimary`] if it is the primary row (caller must
    /// promote another first), and with [`Error::UserEmailIsLast`] if it is
    /// the user's only email row.
    async fn remove_user_email(
        &mut self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), Error>;
    /// Promotes the named email to primary. Atomically clears `is_primary`
    /// on every other row of the user. Rejects with
    /// [`Error::UserEmailNotVerified`] if the target row is `verified = false`
    /// (the §10 linchpin: prevents a session-hijacker from redirecting
    /// password resets to an attacker-controlled mailbox).
    async fn set_primary_email(
        &mut self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), Error>;
    /// Marks the named email row verified, setting `verified_at = now()`.
    /// Idempotent: re-marking an already-verified row updates `verified_at`
    /// to the current time (matches "user re-clicked the verification link").
    async fn mark_email_verified(
        &mut self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), Error>;
}

/// Contains the identifying details for a maze item and (optionally)
/// the definition JSON
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct MazeItem {
    /// Maze ID
    pub id: String,
    /// Maze name
    pub name: String,
    /// Maze definition
    pub definition: Option<String>, // JSON
}

/// Represents a store for holding mazes and related objects
#[async_trait]
pub trait MazeStore {
    /// Adds a new maze to the store and sets the allocated `id` within the maze object
    async fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error>;
    /// Deletes a maze from the store
    async fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), Error>;
    /// Updates a maze within the store
    async fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error>;
    /// Loads a maze from the store
    async fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, Error>;
    /// Locates a maze item by its name within the store
    async fn find_maze_by_name(&self, owner: &User, name: &str) -> Result<MazeItem, Error>;
    /// Returns the list of maze items within the store, sorted
    /// alphabetically in ascending order
    async fn get_maze_items(&self, owner: &User, include_definitions: bool) -> Result<Vec<MazeItem>, Error>;
}
// Store management
#[async_trait]
pub trait Manage {
    /// Resets the store to empty
    async fn empty(&mut self) -> Result<(), Error>;
}

/// Represents a store
pub trait Store: UserStore + MazeStore + Manage + Send + Sync {}

#[allow(dead_code)]
pub type SharedStore = Arc<RwLock<Box<dyn Store>>>;
