use crate::Error;
use async_trait::async_trait;
use data_model::{AuditOutcome, EmailAuditEntry, Maze, OneTimeToken, User, UserEmail};
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
    /// Soft-deletes a user. The `users` row is preserved so audit-log
    /// foreign keys remain valid, with `deleted_at` populated and the
    /// username scrambled to `deleted-<uuid>` so the original value is
    /// freed for reuse by a future signup. Related rows/fields that have no
    /// audit value are hard-deleted in the same transaction:
    /// `user_logins`, `oauth_identities`, `user_emails`, and the user's
    /// mazes. After this returns, every read path on this trait treats
    /// the user as if it never existed.
    async fn delete_user(&mut self, id: Uuid) -> Result<(), Error>;
    /// True hard-delete: removes the `users` row outright. Intended for
    /// retention / right-to-erasure flows where the soft-delete data must
    /// also be cleared.
    async fn purge_user(&mut self, id: Uuid) -> Result<(), Error>;
    /// Updates a user within the store
    async fn update_user(&mut self, user: &mut User) -> Result<(), Error>;
    /// Loads a user from the store
    async fn get_user(&self, id: Uuid) -> Result<User, Error>;
    /// Locates a user by their username within the store
    async fn find_user_by_name(&self, name: &str) -> Result<User, Error>;
    /// Locates a user by an email address within the store, returning the
    /// match only if the matching `user_emails` row is `verified = true`.
    /// Unverified rows are invisible to this lookup, preventing a
    /// session-hijack scenario where an attacker attaches an unverified
    /// address to a victim's account and redirects password resets to it.
    async fn find_user_by_verified_email(&self, email: &str) -> Result<User, Error>;
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
    /// Returns whether at least one *active* admin user exists in the
    /// store (i.e. `is_admin = true` AND `deleted_at IS NULL`). Used by
    /// startup so that a soft-deleted lone admin doesn't prevent the
    /// default admin from being recreated on next launch.
    async fn has_active_admin_user(&self) -> Result<bool, Error>;
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
    /// — preventing a session-hijacker from promoting an attacker-controlled
    /// mailbox and redirecting password resets to it.
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
/// Represents a store for holding single-use, time-bounded tokens
/// (password reset, email verification).
#[async_trait]
pub trait TokenStore {
    /// Persists a new token. The caller is responsible for assigning the
    /// `id`, `created_at`, and `expires_at` fields — typically via
    /// [`OneTimeToken::new`]. Rejects with [`Error::Other`] if a token with
    /// the same id already exists.
    async fn create_token(&mut self, token: &OneTimeToken) -> Result<(), Error>;
    /// Loads an active (non-expired, non-consumed) token by id. Expired
    /// tokens and tokens belonging to soft-deleted users are invisible to
    /// this lookup.
    async fn find_token(&self, id: Uuid) -> Result<OneTimeToken, Error>;
    /// Atomically marks the token consumed. Returns the consumed token on
    /// success. Fails with [`Error::TokenAlreadyConsumed`] when the token
    /// has already been consumed; with [`Error::TokenIdNotFound`] when no
    /// such token exists; with [`Error::TokenExpired`] when the token has
    /// passed its expiry.
    ///
    /// Implementations must enforce single-use atomically: a race of
    /// concurrent `consume_token` calls against the same id must produce
    /// exactly one winner.
    async fn consume_token(&mut self, id: Uuid) -> Result<OneTimeToken, Error>;
    /// Removes every outstanding [`TokenPurpose::EmailVerification`] token
    /// belonging to `user_id` whose `target_email` matches the supplied
    /// address (case-insensitive). Returns the number of tokens removed.
    /// Used by the verification re-send handler so re-issuing supersedes
    /// any prior token — only the most recent link works.
    async fn purge_email_verification_tokens(
        &mut self,
        user_id: Uuid,
        target_email: &str,
    ) -> Result<u64, Error>;
    /// Removes every token whose `expires_at` is in the past AND that has
    /// not been consumed. Returns the number of tokens deleted. Intended
    /// as a periodic housekeeping sweep.
    async fn purge_expired(&mut self) -> Result<u64, Error>;
}

/// Append-only audit log of every email send attempt — captures intent
/// and authorization, complementing provider-side delivery telemetry
/// (out of scope for this trait). Rows are written in two stages:
///
///   * `record_pending` synchronously inserts the row before the send
///     is attempted, returning the assigned id.
///   * `update_outcome` flips the row to `Accepted` (with
///     `provider_message_id`) or `Failed` (with `error_class`) when the
///     provider responds.
#[async_trait]
pub trait EmailAuditLog {
    /// Inserts a new audit row. The caller passes a populated
    /// [`EmailAuditEntry`] — typically built via
    /// [`EmailAuditEntry::new_pending`]. Returns the row's id on
    /// success.
    async fn record_pending(&mut self, entry: &EmailAuditEntry) -> Result<Uuid, Error>;
    /// Updates the row's outcome. `provider_message_id` populates the
    /// matching column on `Accepted`; `error_class` populates on
    /// `Failed`. Passing `Pending` is a programmer error and rejected
    /// with [`Error::Other`] — once written, a row only moves
    /// forwards.
    async fn update_outcome(
        &mut self,
        id: Uuid,
        outcome: AuditOutcome,
        provider_message_id: Option<&str>,
        error_class: Option<&str>,
    ) -> Result<(), Error>;
    /// Loads a single audit row by id.
    async fn find_audit_entry(&self, id: Uuid) -> Result<EmailAuditEntry, Error>;
    /// Returns the most recent audit rows for a user (`recipient_user_id =
    /// ?`), sorted by `created_at` descending, capped at `limit`. An
    /// implementation that returns more than `limit` rows is
    /// non-conformant.
    async fn find_recent_audit_entries_for_user(
        &self,
        user_id: Uuid,
        limit: u32,
    ) -> Result<Vec<EmailAuditEntry>, Error>;
}

// Store management
#[async_trait]
pub trait Manage {
    /// Resets the store to empty
    async fn empty(&mut self) -> Result<(), Error>;
}

/// Represents a store
pub trait Store: UserStore + MazeStore + TokenStore + EmailAuditLog + Manage + Send + Sync {}

#[allow(dead_code)]
pub type SharedStore = Arc<RwLock<Box<dyn Store>>>;
