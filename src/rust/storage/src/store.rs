use crate::Error;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
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
    /// Locates a user by an email address within the store, **regardless
    /// of verification state**. Use only when the verification state of
    /// the row is being inspected by the caller for a downstream
    /// decision. Authentication / session paths must use
    /// [`UserStore::find_user_by_verified_email`] instead — that variant
    /// gates on `verified = true` to prevent attaching to attacker-
    /// controlled rows.
    ///
    /// Existing callers:
    /// - OAuth squat-reclaim (`maze_web_server::oauth::account::resolve`):
    ///   when branch 3 would create a user but the email is already held
    ///   by an unverified, no-OAuth squatted record, the reclaim path
    ///   inspects the existing user's emails + identities here before
    ///   purging it.
    async fn find_user_by_email_any_state(&self, email: &str) -> Result<User, Error>;
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
    /// Removes an email row from the user, and atomically removes any of
    /// the user's `oauth_identities` rows whose `provider_email` matches
    /// the removed address (case-insensitive). The invariant the
    /// secondary cleanup maintains is "an OAuth identity row implies the
    /// user still owns the email the provider linked through" — without
    /// it, an OAuth identity bound to a since-removed email would let
    /// the OAuth provider still authenticate the user (branch 1 of
    /// `account::resolve` matches by `(provider, provider_user_id)`, not
    /// by email).
    ///
    /// Rejects with [`Error::UserEmailIsPrimary`] if it is the primary
    /// row (caller must promote another first), and with
    /// [`Error::UserEmailIsLast`] if it is the user's only email row.
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
    /// Stores (or replaces) the user's avatar image. `png_bytes` is the
    /// canonical avatar the caller has already produced — the store treats it
    /// as opaque and performs no decoding, re-encoding, or format validation.
    /// The server canonicalises every upload to a 256×256 PNG before calling
    /// this, so a stored avatar is always a PNG and no content-type is
    /// persisted. Stamps [`data_model::User::avatar_updated_at`] so the
    /// "has an avatar" signal and the cache-buster move in lock-step with the
    /// bytes. Rejects with [`Error::UserIdNotFound`] when no active user has
    /// the given id.
    async fn set_user_avatar(&mut self, id: Uuid, png_bytes: Vec<u8>) -> Result<(), Error>;
    /// Loads the user's avatar bytes, or `None` when the user has no avatar
    /// (never set, or since cleared, or no such user). A stored avatar is
    /// always a PNG; the caller owns the `Content-Type` on the way out.
    async fn get_user_avatar(&self, id: Uuid) -> Result<Option<Vec<u8>>, Error>;
    /// Removes the user's avatar if present and clears
    /// [`data_model::User::avatar_updated_at`]. Idempotent — clearing a user
    /// that has no avatar (or no such user) is a successful no-op.
    async fn clear_user_avatar(&mut self, id: Uuid) -> Result<(), Error>;
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
    /// Returns the maximum number of cells (`rows × cols`) the store will
    /// accept on a `create_maze` / `update_maze` call, or `None` when the
    /// store imposes no cap. The cap is a property of the storage backend
    /// (row size on a SQL column, runtime cost on a file store), not of
    /// the maze itself — implementations report the value they actually
    /// enforce on writes. Callers use this to surface the limit to clients
    /// and to validate ahead of an actual write.
    fn max_maze_cells(&self) -> Option<usize> {
        None
    }
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
    /// Removes every outstanding [`data_model::TokenPurpose::EmailVerification`] token
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
    /// matching column on `Accepted`; `error_class` and `error_message`
    /// populate on `Failed`. `error_class` is the stable, low-cardinality
    /// dashboard signal; `error_message` carries the upstream diagnostic
    /// detail (e.g. an Azure AD `AADSTS70011` body or an SMTP enhanced
    /// status response). Passing `Pending` is a programmer error and
    /// rejected with [`Error::Other`] — once written, a row only moves
    /// forwards.
    async fn update_outcome(
        &mut self,
        id: Uuid,
        outcome: AuditOutcome,
        provider_message_id: Option<&str>,
        error_class: Option<&str>,
        error_message: Option<&str>,
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

/// One completed-run score record. A run's subject is one of two — exactly
/// one of `maze_id` / `challenge` is set (an app-layer invariant; there is no
/// portable cross-column CHECK under SQLx-Any / MySQL):
///
///   * a stored **user maze** → `maze_id` (FK `mazes(id)`), or
///   * a **curated / shared game** → `challenge` = `"<difficulty>:<seed>"`.
///
/// `user_id` is the **player** (not the maze owner), so boards aggregate every
/// player of a subject. `score` / `elapsed_ms` are `u64` here (matching the
/// engine + the game-result wire) and stored as `i64` / `BIGINT`.
///
/// No `ToSchema` derive — the typed `Uuid` fields would require utoipa's
/// `uuid`/`chrono` features; the OpenAPI wire shape is defined by the server
/// layer (which owns the response DTO).
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ScoreEntry {
    /// Row id.
    pub id: Uuid,
    /// The player who recorded the run.
    pub user_id: Uuid,
    /// The stored maze played, or `None` for a curated/shared game.
    pub maze_id: Option<String>,
    /// The curated/shared game played (`"<difficulty>:<seed>"`), or `None` for
    /// a user maze.
    pub challenge: Option<String>,
    /// Final score at completion.
    pub score: u64,
    /// Elapsed run time in milliseconds.
    pub elapsed_ms: u64,
    /// When the run was recorded (server-stamped at record time).
    pub recorded_at: DateTime<Utc>,
}

/// The metric a leaderboard ranks by.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoreMetric {
    /// Completion time (`elapsed_ms`).
    Time,
    /// Final score.
    Score,
}

/// Sort direction for a leaderboard's primary metric.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    /// Smallest first.
    Ascending,
    /// Largest first.
    Descending,
}

/// Ordering for a leaderboard query: the primary `metric` sorted in
/// `direction`, with a fixed sensible tie-break (the *other* metric — faster /
/// higher among equal primaries) followed by `recorded_at` / `id` as
/// deterministic final keys. Only the primary direction follows `direction`
/// (normal table-sort behaviour — the tie-breaks stay fixed so a UI column
/// toggle reads naturally and pagination stays deterministic). The clauses are
/// built from fixed column names — never user input.
///
/// "Best first" depends on the metric: `Time` + `Ascending` (fastest first)
/// and `Score` + `Descending` (highest first) are the two canonical board
/// views; the opposite directions surface the worst runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreOrdering {
    /// Which metric to rank by.
    pub metric: ScoreMetric,
    /// The primary metric's sort direction.
    pub direction: SortDirection,
}

/// Per-completed-run score history: records a won run and serves the
/// leaderboards (per-maze, per-curated-challenge) and personal history over
/// them. One row per completed run — "best" is a query, not a stored flag.
/// The board/history reads are **paged** (`limit` + `offset`); callers cap
/// `limit` to a sane maximum.
/// A leaderboard row: a recorded run plus the player's `username` when the
/// caller asked for it (`include_usernames`). `username` is `None` when names
/// weren't requested, or when the player can't be resolved. The username is
/// resolved by the storage layer (each backend picks its strategy — e.g.
/// SqlStore joins `users` in the board query) rather than by callers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoreboardEntry {
    /// The recorded run.
    pub entry: ScoreEntry,
    /// The player's username, when resolved.
    pub username: Option<String>,
    /// The player's [`User::avatar_updated_at`] marker, resolved from the same
    /// lookup that resolves `username` (the SqlStore board JOIN / the FileStore
    /// player-file read). `Some(ts)` means the player has an avatar and the
    /// value is its cache-buster; `None` means no avatar (or names/avatars
    /// weren't requested, or the player couldn't be resolved). Lets a board row
    /// decide between rendering the player's image and the generic placeholder
    /// without a per-row round-trip.
    pub avatar_updated_at: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait ScoreStore {
    /// Records a completed run. The caller supplies a fully-populated
    /// [`ScoreEntry`]. Rejects with [`Error::Other`] when the subject invariant
    /// is violated (neither or both of `maze_id` / `challenge` set). Returns the
    /// row id on success.
    async fn record_score(&mut self, entry: &ScoreEntry) -> Result<Uuid, Error>;
    /// A page of the leaderboard for a user maze, ranked by `ordering`. When
    /// `include_usernames` is set, each row carries the player's username
    /// (resolved by the backend); otherwise `username` is `None`.
    async fn maze_leaderboard(
        &self,
        maze_id: &str,
        ordering: ScoreOrdering,
        limit: u32,
        offset: u32,
        include_usernames: bool,
    ) -> Result<Vec<ScoreboardEntry>, Error>;
    /// A page of the leaderboard for a curated/shared challenge, ranked by
    /// `ordering`. `include_usernames` behaves as for [`maze_leaderboard`].
    async fn challenge_leaderboard(
        &self,
        challenge: &str,
        ordering: ScoreOrdering,
        limit: u32,
        offset: u32,
        include_usernames: bool,
    ) -> Result<Vec<ScoreboardEntry>, Error>;
    /// A page of a player's own run history, most recent first
    /// (`recorded_at` descending, `id` descending). No usernames — every row is
    /// the caller.
    async fn user_history(
        &self,
        user_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ScoreEntry>, Error>;
    /// Deletes every score recorded against a user maze, resetting its
    /// leaderboard to empty. Returns the number of rows removed (0 if the board
    /// was already empty). Authorization (maze ownership) is the caller's
    /// responsibility — this clears unconditionally by subject.
    async fn clear_maze_scores(&mut self, maze_id: &str) -> Result<u64, Error>;
    /// Deletes every score recorded against a curated/shared challenge, resetting
    /// its leaderboard to empty. Returns the number of rows removed. Authorization
    /// (admin) is the caller's responsibility — this clears unconditionally by
    /// subject.
    async fn clear_challenge_scores(&mut self, challenge: &str) -> Result<u64, Error>;
}

/// Enforces the dual-keyed subject invariant for a [`ScoreEntry`]: exactly one
/// of `maze_id` / `challenge` must be set. Shared by every [`ScoreStore`]
/// backend's `record_score` (there is no portable cross-column CHECK).
pub(crate) fn validate_score_subject(entry: &ScoreEntry) -> Result<(), Error> {
    // `is_some() == is_some()` is true when both are set or both are unset.
    if entry.maze_id.is_some() == entry.challenge.is_some() {
        return Err(Error::Other(
            "score entry must set exactly one of maze_id / challenge".to_string(),
        ));
    }
    Ok(())
}

// Store management
#[async_trait]
pub trait Manage {
    /// Resets the store to empty
    async fn empty(&mut self) -> Result<(), Error>;
}

/// Represents a store
pub trait Store: UserStore + MazeStore + TokenStore + EmailAuditLog + ScoreStore + Manage + Send + Sync {}

#[allow(dead_code)]
pub type SharedStore = Arc<RwLock<Box<dyn Store>>>;

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal stub implementing only the required trait methods so that the
    // default `max_maze_cells` body is the one under test.
    struct NoCapStub;

    #[async_trait]
    impl MazeStore for NoCapStub {
        async fn create_maze(&mut self, _owner: &User, _maze: &mut Maze) -> Result<(), Error> {
            unimplemented!()
        }
        async fn delete_maze(&mut self, _owner: &User, _id: &str) -> Result<(), Error> {
            unimplemented!()
        }
        async fn update_maze(&mut self, _owner: &User, _maze: &mut Maze) -> Result<(), Error> {
            unimplemented!()
        }
        async fn get_maze(&self, _owner: &User, _id: &str) -> Result<Maze, Error> {
            unimplemented!()
        }
        async fn find_maze_by_name(
            &self,
            _owner: &User,
            _name: &str,
        ) -> Result<MazeItem, Error> {
            unimplemented!()
        }
        async fn get_maze_items(
            &self,
            _owner: &User,
            _include_definitions: bool,
        ) -> Result<Vec<MazeItem>, Error> {
            unimplemented!()
        }
    }

    #[test]
    fn maze_store_max_maze_cells_default_is_none() {
        let stub = NoCapStub;
        assert!(stub.max_maze_cells().is_none());
    }
}
