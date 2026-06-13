//! Backend-agnostic [`Store`] trait contract.
//!
//! Each helper is a self-contained scenario that takes a freshly-empty
//! `&mut Box<dyn Store>` and panics on any contract violation. Wrapped from
//! `tests/file_store_contract.rs` (FileStore) and `tests/sql_store_contract.rs`
//! (SqlStore over SQLite/PostgreSQL/MySQL via `DATABASE_URL`).
//!
//! Per-test isolation is the caller's responsibility: hand each helper a store
//! that has just had `.empty().await` called.

#![allow(dead_code)] // Some helpers may not yet be wired into every backend's runner.

use chrono::{Duration, SubsecRound, Utc};
use data_model::{
    AuditOutcome, EMAIL_AUDIT_ERROR_MESSAGE_MAX_CHARS, ERROR_MESSAGE_TRUNCATION_MARKER,
    EmailAuditEntry, Maze, MazeDefinition, OAuthIdentity, OneTimeToken, TokenPurpose, User,
    UserEmail, UserLogin,
};
use storage::{Error, ScoreEntry, ScoreMetric, ScoreOrdering, ScoreboardEntry, SortDirection, Store};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────
// Data builders (pure — no DB I/O)
// ─────────────────────────────────────────────────────────────────────────

/// Builds a User with a password hash and no logins/oauth identities. The
/// `id` and `api_key` fields are nil — `create_user` overwrites them.
pub fn make_user(username: &str, email: &str) -> User {
    User {
        id: Uuid::nil(),
        is_admin: false,
        username: username.to_string(),
        full_name: String::new(),
        emails: vec![UserEmail::new_primary_verified(email)],
        password_hash: "argon2id$contract-test-hash".to_string(),
        api_key: Uuid::nil(),
        logins: vec![],
        oauth_identities: vec![],
        deleted_at: None,
        // Truncate to millisecond precision to match the storage round-trip.
        // `datetime_to_sql` writes RFC 3339 with millis; comparing against a
        // `Utc::now()` that carries sub-millisecond precision would diff after
        // the round-trip even though the stored value is correct.
        created_at: chrono::Utc::now().trunc_subsecs(3),
        last_sign_in_at: None,
    }
}

pub fn make_admin(username: &str, email: &str) -> User {
    let mut u = make_user(username, email);
    u.is_admin = true;
    u
}

/// Builds an OAuth-only User: empty password hash + one OAuth identity.
pub fn make_oauth_user(username: &str, email: &str, provider: &str, sub: &str) -> User {
    let mut u = make_user(username, email);
    u.password_hash = String::new();
    u.oauth_identities
        .push(OAuthIdentity::new(provider.to_string(), sub.to_string(), None));
    u
}

pub fn make_maze(name: &str) -> Maze {
    let mut m = Maze::new(MazeDefinition::new(3, 3));
    m.name = name.to_string();
    m
}

// ─────────────────────────────────────────────────────────────────────────
// Store seeders (write to store, return the created entity)
// ─────────────────────────────────────────────────────────────────────────

pub async fn fixture_user(store: &mut Box<dyn Store>, username: &str, email: &str) -> User {
    let mut user = make_user(username, email);
    store.create_user(&mut user).await.expect("fixture_user");
    user
}

pub async fn fixture_admin(store: &mut Box<dyn Store>, username: &str, email: &str) -> User {
    let mut user = make_admin(username, email);
    store.create_user(&mut user).await.expect("fixture_admin");
    user
}

pub async fn fixture_two_users(store: &mut Box<dyn Store>) -> (User, User) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let bob = fixture_user(store, "bob", "bob@example.com").await;
    (alice, bob)
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — create / get / round-trip
// ─────────────────────────────────────────────────────────────────────────

pub async fn create_user_assigns_id_and_api_key(store: &mut Box<dyn Store>) {
    let mut u = make_user("alice", "alice@example.com");
    assert_eq!(u.id, Uuid::nil(), "test pre-condition: id starts nil");
    assert_eq!(u.api_key, Uuid::nil(), "test pre-condition: api_key starts nil");

    store.create_user(&mut u).await.expect("create_user");

    assert_ne!(u.id, Uuid::nil(), "create_user must assign a non-nil id");
    assert_ne!(u.api_key, Uuid::nil(), "create_user must assign a non-nil api_key");
}

pub async fn create_user_round_trips_via_get_user(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let loaded = store.get_user(alice.id).await.expect("get_user");
    assert_eq!(loaded, alice, "round-trip user must equal what was created");
}

pub async fn get_user_returns_not_found_for_unknown_id(store: &mut Box<dyn Store>) {
    let id = Uuid::new_v4();
    let err = store.get_user(id).await.expect_err("expected UserIdNotFound");
    assert!(
        matches!(err, Error::UserIdNotFound(ref s) if s == &id.to_string()),
        "expected Error::UserIdNotFound({id}), got {err:?}"
    );
}

pub async fn create_user_rejects_duplicate_username(store: &mut Box<dyn Store>) {
    let _ = fixture_user(store, "alice", "alice@example.com").await;
    let mut clash = make_user("alice", "alice2@example.com");
    let err = store.create_user(&mut clash).await.expect_err("expected name conflict");
    assert!(matches!(err, Error::UserNameExists()), "got {err:?}");
}

pub async fn create_user_rejects_username_case_collision(store: &mut Box<dyn Store>) {
    let _ = fixture_user(store, "alice", "alice@example.com").await;
    let mut clash = make_user("ALICE", "alice2@example.com");
    let err = store.create_user(&mut clash).await.expect_err("expected name conflict");
    assert!(matches!(err, Error::UserNameExists()), "got {err:?}");
}

pub async fn create_user_rejects_duplicate_email(store: &mut Box<dyn Store>) {
    let _ = fixture_user(store, "alice", "alice@example.com").await;
    let mut clash = make_user("bob", "alice@example.com");
    let err = store.create_user(&mut clash).await.expect_err("expected email conflict");
    assert!(matches!(err, Error::UserEmailExists()), "got {err:?}");
}

pub async fn create_user_rejects_email_case_collision(store: &mut Box<dyn Store>) {
    let _ = fixture_user(store, "alice", "alice@example.com").await;
    let mut clash = make_user("bob", "ALICE@EXAMPLE.COM");
    let err = store.create_user(&mut clash).await.expect_err("expected email conflict");
    assert!(matches!(err, Error::UserEmailExists()), "got {err:?}");
}

pub async fn create_user_requires_password_or_oauth(store: &mut Box<dyn Store>) {
    // No password and no oauth identity → must be rejected.
    let mut u = make_user("alice", "alice@example.com");
    u.password_hash = String::new();
    let err = store.create_user(&mut u).await.expect_err("expected password-missing");
    assert!(matches!(err, Error::UserPasswordMissing()), "got {err:?}");
}

pub async fn create_oauth_only_user_succeeds(store: &mut Box<dyn Store>) {
    let mut u = make_oauth_user("alice", "alice@example.com", "google", "google-sub-1");
    store.create_user(&mut u).await.expect("oauth-only user create");

    let loaded = store
        .find_user_by_oauth_identity("google", "google-sub-1")
        .await
        .expect("find_user_by_oauth_identity");
    assert_eq!(loaded.id, u.id);
    assert_eq!(loaded.oauth_identities.len(), 1);
    assert!(loaded.password_hash.is_empty());
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — delete
// ─────────────────────────────────────────────────────────────────────────

pub async fn delete_user_removes_record(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.delete_user(alice.id).await.expect("delete_user");
    let err = store.get_user(alice.id).await.expect_err("user should be gone");
    assert!(matches!(err, Error::UserIdNotFound(_)), "got {err:?}");
}

pub async fn delete_user_rejects_nil_id(store: &mut Box<dyn Store>) {
    let err = store.delete_user(Uuid::nil()).await.expect_err("nil id should fail");
    assert!(matches!(err, Error::UserIdMissing()), "got {err:?}");
}

pub async fn delete_user_returns_not_found_for_unknown_id(store: &mut Box<dyn Store>) {
    let id = Uuid::new_v4();
    let err = store.delete_user(id).await.expect_err("expected UserIdNotFound");
    assert!(
        matches!(err, Error::UserIdNotFound(ref s) if s == &id.to_string()),
        "got {err:?}"
    );
}

pub async fn delete_user_cascades_to_logins(store: &mut Box<dyn Store>) {
    let mut alice = fixture_user(store, "alice", "alice@example.com").await;
    let login = UserLogin::new(24, None, None);
    let login_id = login.id;
    alice.logins.push(login);
    store.update_user(&mut alice).await.expect("update_user");

    // Sanity: login is reachable
    let _ = store
        .find_user_by_login_id(login_id)
        .await
        .expect("login should resolve before delete");

    store.delete_user(alice.id).await.expect("delete_user");

    let err = store
        .find_user_by_login_id(login_id)
        .await
        .expect_err("login should be gone after user delete");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn delete_user_cascades_to_oauth_identities(store: &mut Box<dyn Store>) {
    let mut alice = make_oauth_user("alice", "alice@example.com", "google", "sub-alice");
    store.create_user(&mut alice).await.expect("create_user");

    store.delete_user(alice.id).await.expect("delete_user");

    let err = store
        .find_user_by_oauth_identity("google", "sub-alice")
        .await
        .expect_err("oauth identity should be gone");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn delete_user_cascades_to_mazes(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut maze = make_maze("alice-maze");
    store.create_maze(&alice, &mut maze).await.expect("create_maze");
    let maze_id = maze.id.clone();

    store.delete_user(alice.id).await.expect("delete_user");

    // Reconstruct an owner shell with the same id to attempt the lookup; the
    // user is gone but the API takes a `&User` so we forge one.
    let ghost = User { id: alice.id, ..make_user("alice", "alice@example.com") };
    let err = store
        .get_maze(&ghost, &maze_id)
        .await
        .expect_err("maze should be gone after owner delete");
    assert!(matches!(err, Error::MazeIdNotFound(_)), "got {err:?}");
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — update
// ─────────────────────────────────────────────────────────────────────────

pub async fn update_user_persists_changes(store: &mut Box<dyn Store>) {
    let mut alice = fixture_user(store, "alice", "alice@example.com").await;
    alice.full_name = "Alice Updated".to_string();
    alice.set_primary_email_address("alice-new@example.com");
    store.update_user(&mut alice).await.expect("update_user");

    let loaded = store.get_user(alice.id).await.expect("get_user");
    assert_eq!(loaded.full_name, "Alice Updated");
    assert_eq!(loaded.email(), "alice-new@example.com");
}

pub async fn update_user_replaces_logins_wholesale(store: &mut Box<dyn Store>) {
    let mut alice = fixture_user(store, "alice", "alice@example.com").await;

    alice.logins.push(UserLogin::new(24, None, None));
    alice.logins.push(UserLogin::new(48, None, None));
    store.update_user(&mut alice).await.expect("update_user (add logins)");
    let two = store.get_user(alice.id).await.expect("get_user");
    assert_eq!(two.logins.len(), 2, "should have 2 logins after first update");

    let mut second_pass = two.clone();
    second_pass.logins.clear();
    second_pass.logins.push(UserLogin::new(12, None, None));
    store.update_user(&mut second_pass).await.expect("update_user (replace)");

    let one = store.get_user(alice.id).await.expect("get_user");
    assert_eq!(one.logins.len(), 1, "second update must replace, not append");
}

pub async fn update_user_returns_not_found_for_unknown_id(store: &mut Box<dyn Store>) {
    let mut ghost = make_user("ghost", "ghost@example.com");
    ghost.id = Uuid::new_v4();
    let err = store.update_user(&mut ghost).await.expect_err("expected UserIdNotFound");
    assert!(matches!(err, Error::UserIdNotFound(_)), "got {err:?}");
}

pub async fn update_user_rejects_username_case_collision(store: &mut Box<dyn Store>) {
    let _ = fixture_user(store, "alice", "alice@example.com").await;
    let mut bob = fixture_user(store, "bob", "bob@example.com").await;

    bob.username = "ALICE".to_string();
    let err = store.update_user(&mut bob).await.expect_err("expected name collision");
    assert!(matches!(err, Error::UserNameExists()), "got {err:?}");
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — find_*_by_*
// ─────────────────────────────────────────────────────────────────────────

pub async fn find_user_by_name_is_case_insensitive(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "Alice", "alice@example.com").await;
    let by_lower = store.find_user_by_name("alice").await.expect("lower");
    let by_upper = store.find_user_by_name("ALICE").await.expect("upper");
    assert_eq!(by_lower.id, alice.id);
    assert_eq!(by_upper.id, alice.id);
}

pub async fn find_user_by_name_returns_not_found(store: &mut Box<dyn Store>) {
    let err = store.find_user_by_name("nobody").await.expect_err("expected UserNotFound");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn find_user_by_verified_email_is_case_insensitive(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "Alice@Example.COM").await;
    let by_lower = store.find_user_by_verified_email("alice@example.com").await.expect("lower");
    let by_upper = store.find_user_by_verified_email("ALICE@EXAMPLE.COM").await.expect("upper");
    assert_eq!(by_lower.id, alice.id);
    assert_eq!(by_upper.id, alice.id);
}

pub async fn find_user_by_verified_email_skips_unverified_rows(store: &mut Box<dyn Store>) {
    // A `user_emails` row with `verified = false` must be invisible to
    // `find_user_by_verified_email`, even when the row exists and the
    // address matches case-insensitively.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store
        .add_user_email(alice.id, "alice2@example.com", false)
        .await
        .expect("add unverified email");

    // The unverified row exists on alice but is invisible to the lookup.
    let err = store
        .find_user_by_verified_email("alice2@example.com")
        .await
        .expect_err("unverified row must not be returned");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");

    // Sanity: the verified primary is still findable.
    let by_primary = store
        .find_user_by_verified_email("alice@example.com")
        .await
        .expect("primary verified email is findable");
    assert_eq!(by_primary.id, alice.id);

    // After verification the previously-invisible row becomes findable.
    store
        .mark_email_verified(alice.id, "alice2@example.com")
        .await
        .expect("mark_email_verified");
    let by_secondary = store
        .find_user_by_verified_email("alice2@example.com")
        .await
        .expect("once verified, the row is visible");
    assert_eq!(by_secondary.id, alice.id);
}

pub async fn find_user_by_api_key_round_trips(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let by_key = store.find_user_by_api_key(alice.api_key).await.expect("api_key");
    assert_eq!(by_key.id, alice.id);
}

pub async fn find_user_by_api_key_returns_not_found(store: &mut Box<dyn Store>) {
    let err = store
        .find_user_by_api_key(Uuid::new_v4())
        .await
        .expect_err("expected UserNotFound");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn find_user_by_login_id_returns_active_login_owner(store: &mut Box<dyn Store>) {
    let mut alice = fixture_user(store, "alice", "alice@example.com").await;
    let login = UserLogin::new(24, Some("127.0.0.1".to_string()), None);
    let login_id = login.id;
    alice.logins.push(login);
    store.update_user(&mut alice).await.expect("update_user");

    let owner = store.find_user_by_login_id(login_id).await.expect("active login");
    assert_eq!(owner.id, alice.id);
}

pub async fn find_user_by_oauth_identity_provider_case_insensitive(store: &mut Box<dyn Store>) {
    let mut alice = make_oauth_user("alice", "alice@example.com", "google", "sub-alice");
    store.create_user(&mut alice).await.expect("create_user");

    let lower = store
        .find_user_by_oauth_identity("google", "sub-alice")
        .await
        .expect("lower");
    let mixed = store
        .find_user_by_oauth_identity("Google", "sub-alice")
        .await
        .expect("mixed case");
    assert_eq!(lower.id, alice.id);
    assert_eq!(mixed.id, alice.id);
}

pub async fn find_user_by_oauth_identity_strict_matching(store: &mut Box<dyn Store>) {
    let mut alice = make_oauth_user("alice", "alice@example.com", "google", "sub-alice");
    store.create_user(&mut alice).await.expect("create_user");

    // provider_user_id is matched exactly (case-sensitive) per OAuth/OIDC
    // spec — `sub` is opaque and case-significant. PG and SQLite use
    // case-sensitive collations by default; MySQL needs an explicit
    // `COLLATE utf8mb4_bin` patch on the column (applied in `SqlStore::new`
    // post-migration since the COLLATE syntax isn't portable through the
    // single migration file).
    assert!(
        store
            .find_user_by_oauth_identity("google", "SUB-ALICE")
            .await
            .is_err(),
        "provider_user_id must be case-sensitive"
    );

    // Wrong provider for a known sub must not match.
    assert!(
        store
            .find_user_by_oauth_identity("github", "sub-alice")
            .await
            .is_err(),
        "wrong provider must not match a known sub"
    );

    // Unknown identity returns UserNotFound (not Other).
    let err = store
        .find_user_by_oauth_identity("google", "no-such-sub")
        .await
        .expect_err("unknown identity must error");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn find_user_by_oauth_identity_supports_multiple_per_user(store: &mut Box<dyn Store>) {
    let mut alice = fixture_user(store, "alice", "alice@example.com").await;
    alice.oauth_identities.push(OAuthIdentity::new(
        "google".to_string(),
        "sub-alice-google".to_string(),
        None,
    ));
    alice.oauth_identities.push(OAuthIdentity::new(
        "github".to_string(),
        "sub-alice-github".to_string(),
        None,
    ));
    store.update_user(&mut alice).await.expect("update_user");

    let via_google = store
        .find_user_by_oauth_identity("google", "sub-alice-google")
        .await
        .expect("google");
    let via_github = store
        .find_user_by_oauth_identity("github", "sub-alice-github")
        .await
        .expect("github");
    assert_eq!(via_google.id, alice.id);
    assert_eq!(via_github.id, alice.id);
    assert_eq!(via_google.oauth_identities.len(), 2);
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — list operations
// ─────────────────────────────────────────────────────────────────────────

pub async fn get_users_returns_all_sorted_by_username(store: &mut Box<dyn Store>) {
    // Insert in reverse order to prove sorting happens on read.
    let _ = fixture_user(store, "charlie", "charlie@example.com").await;
    let _ = fixture_user(store, "bob", "bob@example.com").await;
    let _ = fixture_user(store, "alice", "alice@example.com").await;

    let users = store.get_users().await.expect("get_users");
    let names: Vec<&str> = users.iter().map(|u| u.username.as_str()).collect();
    assert_eq!(names, vec!["alice", "bob", "charlie"], "must sort by username");
}

pub async fn get_users_empty_when_store_empty(store: &mut Box<dyn Store>) {
    let users = store.get_users().await.expect("get_users");
    assert!(users.is_empty(), "got {} users on empty store", users.len());
}

pub async fn has_users_round_trips(store: &mut Box<dyn Store>) {
    assert!(
        !store.has_users().await.expect("has_users on empty store"),
        "has_users must return false on an empty store"
    );
    let _ = fixture_user(store, "alice", "alice@example.com").await;
    assert!(
        store.has_users().await.expect("has_users on populated store"),
        "has_users must return true after a user is created"
    );
}

pub async fn get_admin_users_filters_to_admins_only(store: &mut Box<dyn Store>) {
    let _ = fixture_user(store, "alice", "alice@example.com").await;
    let _ = fixture_admin(store, "root", "root@example.com").await;
    let _ = fixture_user(store, "bob", "bob@example.com").await;

    let admins = store.get_admin_users().await.expect("get_admin_users");
    assert_eq!(admins.len(), 1);
    assert_eq!(admins[0].username, "root");
    assert!(admins[0].is_admin);
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — init_default_admin_user
// ─────────────────────────────────────────────────────────────────────────

pub async fn init_default_admin_creates_first_time(store: &mut Box<dyn Store>) {
    let admin = store
        .init_default_admin_user("admin", "admin@example.com", "argon2id$bootstrap")
        .await
        .expect("init_default_admin_user");
    assert_eq!(admin.username, "admin");
    assert!(admin.is_admin);

    let users = store.get_users().await.expect("get_users");
    assert_eq!(users.len(), 1);
}

pub async fn init_default_admin_is_idempotent(store: &mut Box<dyn Store>) {
    let first = store
        .init_default_admin_user("admin", "admin@example.com", "argon2id$bootstrap")
        .await
        .expect("first call");
    let second = store
        .init_default_admin_user("admin", "admin@example.com", "argon2id$bootstrap")
        .await
        .expect("second call must not error");
    assert_eq!(first.id, second.id, "second call must return the existing admin");

    let users = store.get_users().await.expect("get_users");
    assert_eq!(users.len(), 1, "no duplicate admin should be created");
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — email management (add/remove/set_primary/mark_verified)
// ─────────────────────────────────────────────────────────────────────────

pub async fn add_user_email_appends_a_non_primary_row(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let row = store
        .add_user_email(alice.id, "alice2@example.com", false)
        .await
        .expect("add_user_email");
    assert_eq!(row.email, "alice2@example.com");
    assert!(!row.is_primary, "newly added row must not be primary");
    assert!(!row.verified, "verified flag must reflect the requested value");

    let loaded = store.get_user(alice.id).await.expect("get_user");
    assert_eq!(loaded.emails.len(), 2);
    assert_eq!(loaded.primary_email().expect("primary").email, "alice@example.com");
}

pub async fn add_user_email_with_verified_true_records_verified_at(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let row = store
        .add_user_email(alice.id, "alice-verified@example.com", true)
        .await
        .expect("add_user_email");
    assert!(row.verified);
    assert!(row.verified_at.is_some(), "verified=true must set verified_at");
}

pub async fn add_user_email_rejects_invalid_format(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let err = store
        .add_user_email(alice.id, "not-an-email", false)
        .await
        .expect_err("expected EmailInvalid");
    assert!(matches!(err, Error::UserEmailInvalid()), "got {err:?}");
}

pub async fn add_user_email_rejects_empty(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let err = store
        .add_user_email(alice.id, "", false)
        .await
        .expect_err("expected EmailMissing");
    assert!(
        matches!(err, Error::UserEmailMissing() | Error::UserEmailInvalid()),
        "got {err:?}"
    );
}

pub async fn add_user_email_rejects_duplicate_across_users(store: &mut Box<dyn Store>) {
    let _alice = fixture_user(store, "alice", "alice@example.com").await;
    let bob = fixture_user(store, "bob", "bob@example.com").await;

    let err = store
        .add_user_email(bob.id, "alice@example.com", false)
        .await
        .expect_err("expected EmailExists across users");
    assert!(matches!(err, Error::UserEmailExists()), "got {err:?}");
}

pub async fn add_user_email_rejects_duplicate_on_same_user(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let err = store
        .add_user_email(alice.id, "alice@example.com", false)
        .await
        .expect_err("expected EmailExists on same user");
    assert!(matches!(err, Error::UserEmailExists()), "got {err:?}");
}

pub async fn add_user_email_rejects_unknown_user(store: &mut Box<dyn Store>) {
    let id = Uuid::new_v4();
    let err = store
        .add_user_email(id, "ghost@example.com", false)
        .await
        .expect_err("expected UserIdNotFound");
    assert!(
        matches!(err, Error::UserIdNotFound(ref s) if s == &id.to_string()),
        "got {err:?}"
    );
}

pub async fn remove_user_email_drops_a_non_primary_row(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store
        .add_user_email(alice.id, "alice2@example.com", true)
        .await
        .expect("add_user_email");

    store
        .remove_user_email(alice.id, "alice2@example.com")
        .await
        .expect("remove_user_email");

    let loaded = store.get_user(alice.id).await.expect("get_user");
    assert_eq!(loaded.emails.len(), 1);
    assert_eq!(loaded.email(), "alice@example.com");
}

pub async fn remove_user_email_refuses_the_only_email(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let err = store
        .remove_user_email(alice.id, "alice@example.com")
        .await
        .expect_err("expected EmailIsLast");
    assert!(matches!(err, Error::UserEmailIsLast()), "got {err:?}");
}

pub async fn remove_user_email_refuses_the_primary(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store
        .add_user_email(alice.id, "alice2@example.com", true)
        .await
        .expect("add_user_email");
    let err = store
        .remove_user_email(alice.id, "alice@example.com")
        .await
        .expect_err("expected EmailIsPrimary");
    assert!(matches!(err, Error::UserEmailIsPrimary()), "got {err:?}");
}

pub async fn remove_user_email_returns_not_found_for_unknown_address(
    store: &mut Box<dyn Store>,
) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let err = store
        .remove_user_email(alice.id, "nope@example.com")
        .await
        .expect_err("expected EmailNotFound");
    assert!(matches!(err, Error::UserEmailNotFound(_)), "got {err:?}");
}

pub async fn remove_user_email_drops_matching_oauth_identities(store: &mut Box<dyn Store>) {
    // An OAuth identity bound to a since-removed email must NOT survive
    // the removal — otherwise the OAuth provider could still authenticate
    // the user via branch 1 of `account::resolve` (which matches by
    // `(provider, provider_user_id)`, not by current email).
    let mut alice = make_user("alice", "alice@example.com");
    alice.oauth_identities.push(OAuthIdentity::new(
        "google".to_string(),
        "google-sub-alice".to_string(),
        Some("alice@example.com".to_string()),
    ));
    store.create_user(&mut alice).await.expect("create_user");
    // Need a second email so removing the primary's match doesn't trip
    // `UserEmailIsLast` — we'll add and promote a secondary, then remove
    // the original.
    store
        .add_user_email(alice.id, "alice2@example.com", true)
        .await
        .expect("add secondary");
    store
        .set_primary_email(alice.id, "alice2@example.com")
        .await
        .expect("promote secondary");

    store
        .remove_user_email(alice.id, "alice@example.com")
        .await
        .expect("remove_user_email");

    let loaded = store.get_user(alice.id).await.expect("get_user");
    assert!(
        loaded.oauth_identities.is_empty(),
        "OAuth identity tied to removed email must be dropped, got {:?}",
        loaded.oauth_identities
    );
}

pub async fn remove_user_email_preserves_unrelated_oauth_identities(store: &mut Box<dyn Store>) {
    // OAuth identities whose `provider_email` does NOT match the removed
    // address (or is `None`) must be preserved.
    let mut alice = make_user("alice", "alice@example.com");
    alice.oauth_identities.push(OAuthIdentity::new(
        "google".to_string(),
        "google-sub-alice".to_string(),
        Some("alice@example.com".to_string()),
    ));
    alice.oauth_identities.push(OAuthIdentity::new(
        "github".to_string(),
        "github-sub-alice".to_string(),
        Some("alt@example.com".to_string()),
    ));
    store.create_user(&mut alice).await.expect("create_user");
    store
        .add_user_email(alice.id, "alt@example.com", true)
        .await
        .expect("add secondary");

    store
        .remove_user_email(alice.id, "alt@example.com")
        .await
        .expect("remove alt@example.com");

    let loaded = store.get_user(alice.id).await.expect("get_user");
    // The github identity bound to the removed `alt@example.com` is gone.
    // The google identity bound to `alice@example.com` survives.
    assert_eq!(loaded.oauth_identities.len(), 1);
    assert_eq!(loaded.oauth_identities[0].provider, "google");
    assert_eq!(
        loaded.oauth_identities[0].provider_email.as_deref(),
        Some("alice@example.com")
    );
}

pub async fn set_primary_email_clears_other_primaries(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store
        .add_user_email(alice.id, "alice2@example.com", true)
        .await
        .expect("add_user_email");

    store
        .set_primary_email(alice.id, "alice2@example.com")
        .await
        .expect("set_primary_email");

    let loaded = store.get_user(alice.id).await.expect("get_user");
    let primary_count = loaded.emails.iter().filter(|r| r.is_primary).count();
    assert_eq!(primary_count, 1, "exactly one primary must remain");
    assert_eq!(loaded.email(), "alice2@example.com");
}

pub async fn set_primary_email_rejects_unverified_target(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store
        .add_user_email(alice.id, "alice2@example.com", false)
        .await
        .expect("add_user_email (unverified)");
    let err = store
        .set_primary_email(alice.id, "alice2@example.com")
        .await
        .expect_err("expected EmailNotVerified");
    assert!(matches!(err, Error::UserEmailNotVerified()), "got {err:?}");

    // Original primary must be unchanged.
    let loaded = store.get_user(alice.id).await.expect("get_user");
    assert_eq!(loaded.email(), "alice@example.com");
}

pub async fn set_primary_email_returns_not_found_for_unknown_address(
    store: &mut Box<dyn Store>,
) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let err = store
        .set_primary_email(alice.id, "nope@example.com")
        .await
        .expect_err("expected EmailNotFound");
    assert!(matches!(err, Error::UserEmailNotFound(_)), "got {err:?}");
}

pub async fn mark_email_verified_promotes_unverified_row(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store
        .add_user_email(alice.id, "alice2@example.com", false)
        .await
        .expect("add_user_email (unverified)");

    store
        .mark_email_verified(alice.id, "alice2@example.com")
        .await
        .expect("mark_email_verified");

    let loaded = store.get_user(alice.id).await.expect("get_user");
    let row = loaded
        .emails
        .iter()
        .find(|r| r.email == "alice2@example.com")
        .expect("row");
    assert!(row.verified, "verified must be true after mark");
    assert!(row.verified_at.is_some(), "verified_at must be populated");
}

pub async fn mark_email_verified_returns_not_found_for_unknown_address(
    store: &mut Box<dyn Store>,
) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let err = store
        .mark_email_verified(alice.id, "nope@example.com")
        .await
        .expect_err("expected EmailNotFound");
    assert!(matches!(err, Error::UserEmailNotFound(_)), "got {err:?}");
}

// ─────────────────────────────────────────────────────────────────────────
// MazeStore
// ─────────────────────────────────────────────────────────────────────────

pub async fn create_maze_assigns_id(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut maze = make_maze("first-maze");
    assert!(maze.id.is_empty(), "test pre-condition: id starts empty");

    store.create_maze(&alice, &mut maze).await.expect("create_maze");
    assert!(!maze.id.is_empty(), "create_maze must assign an id");
}

pub async fn create_maze_rejects_empty_name(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut maze = Maze::new(MazeDefinition::new(3, 3)); // no name set
    let err = store.create_maze(&alice, &mut maze).await.expect_err("empty name");
    assert!(matches!(err, Error::MazeNameMissing()), "got {err:?}");
}

pub async fn create_maze_rejects_name_case_collision(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut first = make_maze("Treasure");
    store.create_maze(&alice, &mut first).await.expect("create_maze first");

    let mut clash = make_maze("TREASURE");
    let err = store
        .create_maze(&alice, &mut clash)
        .await
        .expect_err("case-collision");
    // Backends differ on the variant they raise: FileStore uses `MazeIdExists`
    // (its id is derived from the filename); SqlStore uses
    // `MazeNameAlreadyExists` (id is a UUID independent of name). Both
    // satisfy the contract — the duplicate-name create must be rejected.
    assert!(
        matches!(err, Error::MazeNameAlreadyExists(_) | Error::MazeIdExists(_)),
        "got {err:?}"
    );

    // The owner ends up with exactly one maze.
    let items = store.get_maze_items(&alice, false).await.expect("get_maze_items");
    assert_eq!(items.len(), 1);
}

pub async fn create_maze_allows_same_name_for_different_owners(store: &mut Box<dyn Store>) {
    let (alice, bob) = fixture_two_users(store).await;
    let mut alice_maze = make_maze("Treasure");
    let mut bob_maze = make_maze("Treasure");
    store.create_maze(&alice, &mut alice_maze).await.expect("alice create");
    store.create_maze(&bob, &mut bob_maze).await.expect("bob create");

    // Each owner sees exactly one Treasure of their own. We don't compare
    // ids across owners — FileStore derives the id from the filename so the
    // ids may collide string-wise even though the storage is partitioned by
    // owner directory. SqlStore assigns independent UUIDs.
    let alice_items = store.get_maze_items(&alice, false).await.expect("alice items");
    let bob_items = store.get_maze_items(&bob, false).await.expect("bob items");
    assert_eq!(alice_items.len(), 1);
    assert_eq!(bob_items.len(), 1);
    assert_eq!(alice_items[0].name, "Treasure");
    assert_eq!(bob_items[0].name, "Treasure");
}

pub async fn delete_maze_removes_record(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut maze = make_maze("maze-1");
    store.create_maze(&alice, &mut maze).await.expect("create_maze");
    store.delete_maze(&alice, &maze.id).await.expect("delete_maze");
    let err = store.get_maze(&alice, &maze.id).await.expect_err("should be gone");
    assert!(matches!(err, Error::MazeIdNotFound(_)), "got {err:?}");
}

pub async fn delete_maze_is_scoped_to_owner(store: &mut Box<dyn Store>) {
    let (alice, bob) = fixture_two_users(store).await;
    let mut alice_maze = make_maze("alice-only");
    store.create_maze(&alice, &mut alice_maze).await.expect("alice create");

    // Bob attempts to delete Alice's maze by id — must fail with NotFound (owner-scoped).
    let err = store
        .delete_maze(&bob, &alice_maze.id)
        .await
        .expect_err("bob must not be able to delete alice's maze");
    assert!(matches!(err, Error::MazeIdNotFound(_)), "got {err:?}");

    // Alice's maze still exists.
    let still_there = store.get_maze(&alice, &alice_maze.id).await.expect("still there");
    assert_eq!(still_there.name, "alice-only");
}

pub async fn update_maze_returns_not_found_for_unknown_id(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut ghost = make_maze("ghost-maze");
    ghost.id = "no-such-id".to_string(); // non-empty so we get past the empty-id guard
    let err = store
        .update_maze(&alice, &mut ghost)
        .await
        .expect_err("expected MazeIdNotFound");
    assert!(matches!(err, Error::MazeIdNotFound(_)), "got {err:?}");
}

pub async fn update_maze_rejects_empty_id(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut maze = make_maze("some-maze");
    maze.id = String::new();
    let err = store
        .update_maze(&alice, &mut maze)
        .await
        .expect_err("expected MazeIdMissing");
    assert!(matches!(err, Error::MazeIdMissing()), "got {err:?}");
}

pub async fn delete_maze_rejects_empty_id(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let err = store
        .delete_maze(&alice, "")
        .await
        .expect_err("expected MazeIdMissing");
    assert!(matches!(err, Error::MazeIdMissing()), "got {err:?}");
}

pub async fn update_maze_persists_changes(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut maze = make_maze("orig-name");
    store.create_maze(&alice, &mut maze).await.expect("create_maze");

    maze.name = "renamed".to_string();
    store.update_maze(&alice, &mut maze).await.expect("update_maze");

    let loaded = store.get_maze(&alice, &maze.id).await.expect("get_maze");
    assert_eq!(loaded.name, "renamed");
}

pub async fn create_maze_round_trips_game_settings(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;

    // A maze carrying opaque game settings round-trips them unchanged.
    let mut with_settings = make_maze("with-settings");
    with_settings.game_settings = Some(serde_json::json!({
        "skyType": "dungeon",
        "wallType": "lava",
        "timerSeconds": 90
    }));
    store
        .create_maze(&alice, &mut with_settings)
        .await
        .expect("create_maze with settings");
    let loaded = store
        .get_maze(&alice, &with_settings.id)
        .await
        .expect("get_maze with settings");
    assert_eq!(loaded.game_settings, with_settings.game_settings);

    // A maze with no settings round-trips as None.
    let mut without = make_maze("no-settings");
    store
        .create_maze(&alice, &mut without)
        .await
        .expect("create_maze no settings");
    let loaded_without = store
        .get_maze(&alice, &without.id)
        .await
        .expect("get_maze no settings");
    assert!(loaded_without.game_settings.is_none());
}

pub async fn get_maze_is_scoped_to_owner(store: &mut Box<dyn Store>) {
    let (alice, bob) = fixture_two_users(store).await;
    let mut alice_maze = make_maze("private");
    store.create_maze(&alice, &mut alice_maze).await.expect("create_maze");

    let err = store
        .get_maze(&bob, &alice_maze.id)
        .await
        .expect_err("bob must not see alice's maze");
    assert!(matches!(err, Error::MazeIdNotFound(_)), "got {err:?}");
}

pub async fn find_maze_by_name_is_case_insensitive(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut maze = make_maze("Treasure");
    store.create_maze(&alice, &mut maze).await.expect("create_maze");

    // Lookup with a different case must succeed and round-trip back to the
    // same maze via get_maze. We don't compare returned id/name strings
    // directly — backends differ in whether the id is derived from the
    // filename (FileStore) or an independent UUID (SqlStore), and the
    // returned `name` casing varies similarly.
    let item = store
        .find_maze_by_name(&alice, "TREASURE")
        .await
        .expect("uppercase lookup");
    let loaded = store
        .get_maze(&alice, &item.id)
        .await
        .expect("found item id must round-trip via get_maze");
    assert_eq!(
        loaded.name.to_lowercase(),
        "treasure",
        "round-trip maze name mismatch"
    );
}

pub async fn get_maze_items_lists_owners_mazes_sorted(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    for n in ["charlie", "alpha", "bravo"] {
        let mut m = make_maze(n);
        store.create_maze(&alice, &mut m).await.expect("create_maze");
    }
    let items = store
        .get_maze_items(&alice, false)
        .await
        .expect("get_maze_items");
    let names: Vec<&str> = items.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["alpha", "bravo", "charlie"]);
    // include_definitions = false → all definition fields must be None
    assert!(items.iter().all(|m| m.definition.is_none()));
}

pub async fn get_maze_items_includes_definition_when_requested(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut m = make_maze("with-def");
    store.create_maze(&alice, &mut m).await.expect("create_maze");

    let items = store
        .get_maze_items(&alice, true)
        .await
        .expect("get_maze_items");
    assert_eq!(items.len(), 1);
    assert!(items[0].definition.is_some(), "definition must be populated");
}

pub async fn get_maze_items_is_scoped_to_owner(store: &mut Box<dyn Store>) {
    let (alice, bob) = fixture_two_users(store).await;
    let mut alice_maze = make_maze("alice-maze");
    store.create_maze(&alice, &mut alice_maze).await.expect("alice create");

    let bob_items = store.get_maze_items(&bob, false).await.expect("bob items");
    assert!(bob_items.is_empty(), "bob must see none of alice's mazes");
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — soft-delete behaviour
// ─────────────────────────────────────────────────────────────────────────

pub async fn delete_user_soft_deletes_and_scrambles_username(store: &mut Box<dyn Store>) {
    // After a soft-delete the original username is freed (scrambled to
    // `deleted-<uuid>`), so a brand-new account can claim the same handle.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.delete_user(alice.id).await.expect("soft-delete");
    let mut reborn = make_user("alice", "alice2@example.com");
    store.create_user(&mut reborn).await.expect("username freed for reuse");
    assert_ne!(reborn.id, alice.id, "rebirth must be a brand-new row");
}

pub async fn delete_user_frees_email_for_reuse(store: &mut Box<dyn Store>) {
    // Email rows are hard-deleted on cascade so the address is freed for the
    // next signup.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.delete_user(alice.id).await.expect("soft-delete");
    let mut reborn = make_user("bob", "alice@example.com");
    store.create_user(&mut reborn).await.expect("email freed for reuse");
    assert_ne!(reborn.id, alice.id);
}

pub async fn delete_user_is_idempotent_per_row(store: &mut Box<dyn Store>) {
    // Calling delete_user twice on the same row must surface the second
    // attempt as UserIdNotFound — guard against double-soft-deleting.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.delete_user(alice.id).await.expect("first delete");
    let err = store
        .delete_user(alice.id)
        .await
        .expect_err("second delete must fail");
    assert!(matches!(err, Error::UserIdNotFound(_)), "got {err:?}");
}

pub async fn get_user_filters_soft_deleted(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.delete_user(alice.id).await.expect("soft-delete");
    let err = store
        .get_user(alice.id)
        .await
        .expect_err("soft-deleted user must be invisible to get_user");
    assert!(matches!(err, Error::UserIdNotFound(_)), "got {err:?}");
}

pub async fn find_user_by_name_filters_soft_deleted(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.delete_user(alice.id).await.expect("soft-delete");
    let err = store
        .find_user_by_name("alice")
        .await
        .expect_err("soft-deleted user must be invisible to find_user_by_name");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn find_user_by_verified_email_filters_soft_deleted(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.delete_user(alice.id).await.expect("soft-delete");
    let err = store
        .find_user_by_verified_email("alice@example.com")
        .await
        .expect_err("soft-deleted user must be invisible to find_user_by_verified_email");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn find_user_by_api_key_filters_soft_deleted(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let api_key = alice.api_key;
    store.delete_user(alice.id).await.expect("soft-delete");
    let err = store
        .find_user_by_api_key(api_key)
        .await
        .expect_err("soft-deleted user must be invisible to find_user_by_api_key");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn find_user_by_login_id_filters_soft_deleted(store: &mut Box<dyn Store>) {
    let mut alice = fixture_user(store, "alice", "alice@example.com").await;
    let login = UserLogin::new(24, None, None);
    let login_id = login.id;
    alice.logins.push(login);
    store.update_user(&mut alice).await.expect("update_user");
    store.delete_user(alice.id).await.expect("soft-delete");
    let err = store
        .find_user_by_login_id(login_id)
        .await
        .expect_err("soft-deleted user must be invisible to find_user_by_login_id");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn find_user_by_oauth_identity_filters_soft_deleted(store: &mut Box<dyn Store>) {
    let mut alice = make_oauth_user("alice", "alice@example.com", "google", "sub-alice");
    store.create_user(&mut alice).await.expect("create_user");
    store.delete_user(alice.id).await.expect("soft-delete");
    let err = store
        .find_user_by_oauth_identity("google", "sub-alice")
        .await
        .expect_err("soft-deleted user must be invisible to find_user_by_oauth_identity");
    assert!(matches!(err, Error::UserNotFound()), "got {err:?}");
}

pub async fn get_users_filters_soft_deleted(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let _bob = fixture_user(store, "bob", "bob@example.com").await;
    store.delete_user(alice.id).await.expect("soft-delete");
    let users = store.get_users().await.expect("get_users");
    let names: Vec<&str> = users.iter().map(|u| u.username.as_str()).collect();
    assert_eq!(names, vec!["bob"], "soft-deleted alice must not appear");
}

pub async fn get_admin_users_filters_soft_deleted(store: &mut Box<dyn Store>) {
    let admin = fixture_admin(store, "root", "root@example.com").await;
    let _other_admin = fixture_admin(store, "second", "second@example.com").await;
    store.delete_user(admin.id).await.expect("soft-delete");
    let admins = store.get_admin_users().await.expect("get_admin_users");
    let names: Vec<&str> = admins.iter().map(|u| u.username.as_str()).collect();
    assert_eq!(names, vec!["second"], "soft-deleted root must not appear");
}

pub async fn has_users_filters_soft_deleted(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.delete_user(alice.id).await.expect("soft-delete");
    assert!(
        !store.has_users().await.expect("has_users"),
        "soft-deleted lone user must not register as present"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — purge_user
// ─────────────────────────────────────────────────────────────────────────

pub async fn purge_user_truly_removes_row(store: &mut Box<dyn Store>) {
    // Soft-delete first, then purge — the contract-visible signal is that a
    // second purge against the same id surfaces UserIdNotFound (the row is
    // really gone, not just hidden behind the soft-delete read filter).
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.delete_user(alice.id).await.expect("soft-delete");
    store.purge_user(alice.id).await.expect("purge_user");
    let err = store
        .purge_user(alice.id)
        .await
        .expect_err("second purge must fail — row is gone");
    assert!(matches!(err, Error::UserIdNotFound(_)), "got {err:?}");
    let err = store
        .delete_user(alice.id)
        .await
        .expect_err("soft-delete after purge must fail — row is gone");
    assert!(matches!(err, Error::UserIdNotFound(_)), "got {err:?}");
}

pub async fn purge_user_works_on_active_user(store: &mut Box<dyn Store>) {
    // No prior soft-delete required — purge_user accepts an active row too.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store.purge_user(alice.id).await.expect("purge_user (active)");
    let err = store
        .get_user(alice.id)
        .await
        .expect_err("user must be gone after purge");
    assert!(matches!(err, Error::UserIdNotFound(_)), "got {err:?}");
}

pub async fn purge_user_rejects_nil_id(store: &mut Box<dyn Store>) {
    let err = store
        .purge_user(Uuid::nil())
        .await
        .expect_err("nil id must be rejected");
    assert!(matches!(err, Error::UserIdMissing()), "got {err:?}");
}

pub async fn purge_user_returns_not_found_for_unknown_id(store: &mut Box<dyn Store>) {
    let id = Uuid::new_v4();
    let err = store
        .purge_user(id)
        .await
        .expect_err("unknown id must surface NotFound");
    assert!(
        matches!(err, Error::UserIdNotFound(ref s) if s == &id.to_string()),
        "got {err:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// UserStore — has_active_admin_user
// ─────────────────────────────────────────────────────────────────────────

pub async fn has_active_admin_user_returns_true_when_active_admin_exists(
    store: &mut Box<dyn Store>,
) {
    let _ = fixture_admin(store, "root", "root@example.com").await;
    assert!(
        store
            .has_active_admin_user()
            .await
            .expect("has_active_admin_user")
    );
}

pub async fn has_active_admin_user_returns_false_when_only_admin_is_soft_deleted(
    store: &mut Box<dyn Store>,
) {
    let admin = fixture_admin(store, "root", "root@example.com").await;
    store.delete_user(admin.id).await.expect("soft-delete");
    assert!(
        !store
            .has_active_admin_user()
            .await
            .expect("has_active_admin_user"),
        "soft-deleted lone admin must not register as active"
    );
}

pub async fn has_active_admin_user_returns_false_when_no_users_exist(store: &mut Box<dyn Store>) {
    assert!(
        !store
            .has_active_admin_user()
            .await
            .expect("has_active_admin_user")
    );
}

pub async fn has_active_admin_user_ignores_non_admin_users(store: &mut Box<dyn Store>) {
    let _ = fixture_user(store, "alice", "alice@example.com").await;
    assert!(
        !store
            .has_active_admin_user()
            .await
            .expect("has_active_admin_user"),
        "non-admin user must not satisfy has_active_admin_user"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// TokenStore — create / find / consume / purge_expired
// ─────────────────────────────────────────────────────────────────────────

fn make_password_reset_token(user_id: Uuid) -> OneTimeToken {
    OneTimeToken::new(user_id, TokenPurpose::PasswordReset, None, 1)
}

fn make_email_verification_token(user_id: Uuid, target_email: &str) -> OneTimeToken {
    OneTimeToken::new(
        user_id,
        TokenPurpose::EmailVerification,
        Some(target_email.to_string()),
        24,
    )
}

/// Builds a token whose `expires_at` is already in the past. Timestamps are
/// truncated to millisecond precision to match the storage layer's canonical
/// shape, so round-trip equality holds across both backends.
fn make_expired_token(user_id: Uuid) -> OneTimeToken {
    let now = Utc::now().trunc_subsecs(3);
    OneTimeToken {
        id: Uuid::new_v4(),
        user_id,
        purpose: TokenPurpose::PasswordReset,
        target_email: None,
        created_at: now - Duration::hours(2),
        expires_at: now - Duration::hours(1),
        consumed_at: None,
    }
}

pub async fn create_token_round_trips_via_find(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let token = make_password_reset_token(alice.id);
    store.create_token(&token).await.expect("create_token");
    let loaded = store.find_token(token.id).await.expect("find_token");
    assert_eq!(loaded, token);
}

pub async fn create_token_preserves_target_email_for_verification(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let token = make_email_verification_token(alice.id, "alice2@example.com");
    store.create_token(&token).await.expect("create_token");
    let loaded = store.find_token(token.id).await.expect("find_token");
    assert_eq!(loaded.target_email.as_deref(), Some("alice2@example.com"));
    assert_eq!(loaded.purpose, TokenPurpose::EmailVerification);
}

pub async fn create_token_rejects_duplicate_id(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let token = make_password_reset_token(alice.id);
    store.create_token(&token).await.expect("first create_token");
    let err = store
        .create_token(&token)
        .await
        .expect_err("duplicate id must be rejected");
    assert!(matches!(err, Error::TokenIdExists(_)), "got {err:?}");
}

pub async fn find_token_returns_not_found_for_unknown_id(store: &mut Box<dyn Store>) {
    let id = Uuid::new_v4();
    let err = store
        .find_token(id)
        .await
        .expect_err("unknown id must surface NotFound");
    assert!(
        matches!(err, Error::TokenIdNotFound(ref s) if s == &id.to_string()),
        "got {err:?}"
    );
}

pub async fn find_token_filters_expired_tokens(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let expired = make_expired_token(alice.id);
    store.create_token(&expired).await.expect("create_token");
    let err = store
        .find_token(expired.id)
        .await
        .expect_err("expired token must be invisible to find_token");
    assert!(matches!(err, Error::TokenIdNotFound(_)), "got {err:?}");
}

pub async fn consume_token_marks_consumed_at(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let token = make_password_reset_token(alice.id);
    store.create_token(&token).await.expect("create_token");
    let consumed = store
        .consume_token(token.id)
        .await
        .expect("consume_token");
    assert_eq!(consumed.id, token.id);
    assert!(consumed.consumed_at.is_some(), "consumed_at must be populated");
}

pub async fn consume_token_twice_rejects_second_call(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let token = make_password_reset_token(alice.id);
    store.create_token(&token).await.expect("create_token");
    store
        .consume_token(token.id)
        .await
        .expect("first consume succeeds");
    let err = store
        .consume_token(token.id)
        .await
        .expect_err("second consume must fail");
    assert!(
        matches!(err, Error::TokenAlreadyConsumed()),
        "expected TokenAlreadyConsumed, got {err:?}"
    );
}

pub async fn consume_token_returns_not_found_for_unknown_id(store: &mut Box<dyn Store>) {
    let id = Uuid::new_v4();
    let err = store
        .consume_token(id)
        .await
        .expect_err("unknown id must surface NotFound");
    assert!(matches!(err, Error::TokenIdNotFound(_)), "got {err:?}");
}

pub async fn consume_token_rejects_expired_token(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let expired = make_expired_token(alice.id);
    store.create_token(&expired).await.expect("create_token");
    let err = store
        .consume_token(expired.id)
        .await
        .expect_err("expired token must not consume");
    assert!(matches!(err, Error::TokenExpired()), "got {err:?}");
}

pub async fn delete_user_cascades_to_one_time_tokens(store: &mut Box<dyn Store>) {
    // Soft-deleting a user must clear their pending tokens — leaving live
    // reset/invite tokens alive is a phishing vector.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let token = make_password_reset_token(alice.id);
    store.create_token(&token).await.expect("create_token");

    store.delete_user(alice.id).await.expect("soft-delete");

    let err = store
        .find_token(token.id)
        .await
        .expect_err("token must be gone after the owner is soft-deleted");
    assert!(matches!(err, Error::TokenIdNotFound(_)), "got {err:?}");
}

pub async fn purge_expired_removes_only_expired_unconsumed_rows(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;

    // Three tokens: an active one, an expired-unconsumed one, and an
    // expired-but-consumed one (which must NOT be purged — its row is
    // historical evidence that the token was used).
    let active = make_password_reset_token(alice.id);
    let expired_unconsumed = make_expired_token(alice.id);
    let expired_consumed = OneTimeToken {
        consumed_at: Some((Utc::now() - Duration::hours(1)).trunc_subsecs(3)),
        ..make_expired_token(alice.id)
    };

    store.create_token(&active).await.expect("create active");
    store
        .create_token(&expired_unconsumed)
        .await
        .expect("create expired-unconsumed");
    store
        .create_token(&expired_consumed)
        .await
        .expect("create expired-consumed");

    let purged = store.purge_expired().await.expect("purge_expired");
    assert_eq!(purged, 1, "exactly the expired-unconsumed row must be purged");

    // Active token still findable.
    store
        .find_token(active.id)
        .await
        .expect("active token must survive purge_expired");

    // Expired-unconsumed token gone.
    let err = store
        .find_token(expired_unconsumed.id)
        .await
        .expect_err("expired-unconsumed token must be gone");
    assert!(matches!(err, Error::TokenIdNotFound(_)), "got {err:?}");

    // A second purge run is a clean no-op.
    let again = store.purge_expired().await.expect("purge_expired idempotent");
    assert_eq!(again, 0, "no further rows to purge");
}

/// 8 concurrent tasks race to consume the same token. Exactly one must
/// win; the other seven must surface `TokenAlreadyConsumed`. Pins the
/// race-free single-use semantics of `consume_token`.
pub async fn consume_token_concurrent_race_has_exactly_one_winner(store: Box<dyn Store>) {
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Wrap the box in an `Arc<Mutex<...>>` so multiple tasks can each take
    // an exclusive lock when they call `consume_token`. The lock simulates
    // the per-connection serialisation that the real store provides via
    // its connection pool — without it, the FileStore impl would have
    // multiple tasks holding their own &mut and trample each other.
    let alice_id = {
        // Limited scope: the mutable borrow ends before the Arc is built.
        let mut store = store;
        let alice = fixture_user(&mut store, "alice", "alice@example.com").await;
        let token = make_password_reset_token(alice.id);
        store.create_token(&token).await.expect("create_token");
        let token_id = token.id;
        let user_id = alice.id;

        let shared = Arc::new(Mutex::new(store));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let shared = shared.clone();
            handles.push(tokio::spawn(async move {
                let mut guard = shared.lock().await;
                guard.consume_token(token_id).await
            }));
        }
        let mut wins = 0;
        let mut losses = 0;
        for h in handles {
            match h.await.expect("join task") {
                Ok(consumed) => {
                    assert_eq!(consumed.id, token_id);
                    assert!(consumed.consumed_at.is_some());
                    wins += 1;
                }
                Err(Error::TokenAlreadyConsumed()) => losses += 1,
                Err(other) => panic!("unexpected error from concurrent consume: {other:?}"),
            }
        }
        assert_eq!(wins, 1, "exactly one task must win the consume race");
        assert_eq!(losses, 7, "the other seven tasks must lose with TokenAlreadyConsumed");
        user_id
    };
    // Silence the unused-variable warning if the test ever needs alice_id later.
    let _ = alice_id;
}

// ─────────────────────────────────────────────────────────────────────────
// EmailAuditLog
// ─────────────────────────────────────────────────────────────────────────

fn pending_entry_for(user_id: Option<Uuid>, email: &str, template_id: &str) -> EmailAuditEntry {
    EmailAuditEntry::new_pending(
        user_id,
        email,
        template_id,
        None,
        None,
        "stub",
    )
}

pub async fn record_pending_returns_id_and_inserts_pending_row(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    let id = store.record_pending(&entry).await.expect("record_pending");
    assert_eq!(id, entry.id);

    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    assert_eq!(loaded.outcome, AuditOutcome::Pending);
    assert!(loaded.provider_message_id.is_none());
    assert!(loaded.error_class.is_none());
    assert_eq!(loaded, entry);
}

pub async fn record_pending_rejects_duplicate_id(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    store.record_pending(&entry).await.expect("first record_pending");
    let err = store
        .record_pending(&entry)
        .await
        .expect_err("duplicate id must be rejected");
    assert!(matches!(err, Error::AuditEntryIdExists(_)), "got {err:?}");
}

pub async fn find_audit_entry_returns_not_found_for_unknown_id(store: &mut Box<dyn Store>) {
    let id = Uuid::new_v4();
    let err = store
        .find_audit_entry(id)
        .await
        .expect_err("unknown id must surface NotFound");
    assert!(
        matches!(err, Error::AuditEntryIdNotFound(ref s) if s == &id.to_string()),
        "got {err:?}"
    );
}

pub async fn update_outcome_to_accepted_populates_provider_message_id(
    store: &mut Box<dyn Store>,
) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    let id = store.record_pending(&entry).await.expect("record_pending");

    store
        .update_outcome(id, AuditOutcome::Accepted, Some("provider-123"), None, None)
        .await
        .expect("update_outcome");

    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    assert_eq!(loaded.outcome, AuditOutcome::Accepted);
    assert_eq!(loaded.provider_message_id.as_deref(), Some("provider-123"));
    assert!(loaded.error_class.is_none());
    assert!(loaded.error_message.is_none());
}

pub async fn update_outcome_to_failed_populates_error_class(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    let id = store.record_pending(&entry).await.expect("record_pending");

    store
        .update_outcome(id, AuditOutcome::Failed, None, Some("provider_unavailable"), None)
        .await
        .expect("update_outcome");

    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    assert_eq!(loaded.outcome, AuditOutcome::Failed);
    assert!(loaded.provider_message_id.is_none());
    assert_eq!(loaded.error_class.as_deref(), Some("provider_unavailable"));
    assert!(loaded.error_message.is_none());
}

pub async fn update_outcome_to_failed_populates_error_message(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    let id = store.record_pending(&entry).await.expect("record_pending");

    let detail = "provider HTTP error: status 400: AADSTS70011: scope is not valid";
    store
        .update_outcome(
            id,
            AuditOutcome::Failed,
            None,
            Some("provider_4xx"),
            Some(detail),
        )
        .await
        .expect("update_outcome");

    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    assert_eq!(loaded.outcome, AuditOutcome::Failed);
    assert_eq!(loaded.error_class.as_deref(), Some("provider_4xx"));
    assert_eq!(loaded.error_message.as_deref(), Some(detail));
}

pub async fn update_outcome_truncates_oversize_error_message(store: &mut Box<dyn Store>) {
    // Verifies the audit-write path doesn't fail on a verbose upstream
    // body (which would hit MySQL's 65,535-byte VARCHAR check or PG's
    // varchar(N) overflow). The store layer truncates with a marker so
    // the audit row always lands.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    let id = store.record_pending(&entry).await.expect("record_pending");

    let huge = "x".repeat(EMAIL_AUDIT_ERROR_MESSAGE_MAX_CHARS * 3);
    store
        .update_outcome(
            id,
            AuditOutcome::Failed,
            None,
            Some("provider_4xx"),
            Some(&huge),
        )
        .await
        .expect("update_outcome must succeed even on oversize body");

    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    let stored = loaded.error_message.expect("error_message present");
    assert_eq!(stored.chars().count(), EMAIL_AUDIT_ERROR_MESSAGE_MAX_CHARS);
    assert!(stored.ends_with(ERROR_MESSAGE_TRUNCATION_MARKER));
}

pub async fn record_pending_truncates_oversize_error_message(store: &mut Box<dyn Store>) {
    // Same protection as `update_outcome_truncates_oversize_error_message`
    // but on the synchronous-insert side: an audit row constructed with an
    // oversize `error_message` (e.g. from a future caller that sets it
    // directly on the entry rather than via update_outcome) still lands.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    entry.error_message = Some("y".repeat(EMAIL_AUDIT_ERROR_MESSAGE_MAX_CHARS * 3));
    let id = store
        .record_pending(&entry)
        .await
        .expect("record_pending must succeed even on oversize body");

    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    let stored = loaded.error_message.expect("error_message present");
    assert_eq!(stored.chars().count(), EMAIL_AUDIT_ERROR_MESSAGE_MAX_CHARS);
    assert!(stored.ends_with(ERROR_MESSAGE_TRUNCATION_MARKER));
}

pub async fn update_outcome_rejects_pending_target(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    let id = store.record_pending(&entry).await.expect("record_pending");

    let err = store
        .update_outcome(id, AuditOutcome::Pending, None, None, None)
        .await
        .expect_err("must not allow re-targeting pending");
    assert!(matches!(err, Error::Other(_)), "got {err:?}");
}

pub async fn update_outcome_returns_not_found_for_unknown_id(store: &mut Box<dyn Store>) {
    let id = Uuid::new_v4();
    let err = store
        .update_outcome(id, AuditOutcome::Accepted, Some("p"), None, None)
        .await
        .expect_err("unknown id must fail");
    assert!(matches!(err, Error::AuditEntryIdNotFound(_)), "got {err:?}");
}

pub async fn find_recent_audit_entries_returns_user_rows_descending_capped_at_limit(
    store: &mut Box<dyn Store>,
) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let bob = fixture_user(store, "bob", "bob@example.com").await;

    // Three rows for alice with strictly increasing created_at, plus one
    // for bob that must NOT appear in alice's results.
    let now = Utc::now().trunc_subsecs(3);
    let mut rows = Vec::new();
    for (offset_secs, template) in [(-30, "a"), (-20, "b"), (-10, "c")] {
        let mut e = pending_entry_for(Some(alice.id), "alice@example.com", template);
        e.created_at = now + Duration::seconds(offset_secs);
        rows.push(e);
    }
    for entry in &rows {
        store.record_pending(entry).await.expect("record_pending");
    }
    let mut bob_entry = pending_entry_for(Some(bob.id), "bob@example.com", "z");
    bob_entry.created_at = now;
    store
        .record_pending(&bob_entry)
        .await
        .expect("record_pending bob");

    // Latest 2 alice rows by created_at descending.
    let recent = store
        .find_recent_audit_entries_for_user(alice.id, 2)
        .await
        .expect("find_recent_audit_entries_for_user");
    assert_eq!(recent.len(), 2, "limit must cap result size");
    assert_eq!(recent[0].template_id, "c", "newest first");
    assert_eq!(recent[1].template_id, "b");
    assert!(recent[0].created_at >= recent[1].created_at);

    // limit larger than the row count just returns them all.
    let all = store
        .find_recent_audit_entries_for_user(alice.id, 100)
        .await
        .expect("find_recent_audit_entries_for_user (large limit)");
    assert_eq!(all.len(), 3);
    assert!(all.iter().all(|e| e.recipient_user_id == Some(alice.id)));
}

pub async fn find_recent_audit_entries_is_empty_when_user_has_none(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let recent = store
        .find_recent_audit_entries_for_user(alice.id, 10)
        .await
        .expect("find_recent_audit_entries_for_user");
    assert!(recent.is_empty());
}

pub async fn audit_log_supports_anti_enumeration_null_recipient(store: &mut Box<dyn Store>) {
    // Reset request for a non-existent email — no send happens but the
    // audit row is still recorded with `recipient_user_id = NULL`.
    let entry = pending_entry_for(None, "ghost@example.com", "password_reset");
    let id = store.record_pending(&entry).await.expect("record_pending");
    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    assert!(loaded.recipient_user_id.is_none());
    assert_eq!(loaded.recipient_email, "ghost@example.com");
    assert_eq!(loaded.outcome, AuditOutcome::Pending);
}

pub async fn audit_log_survives_soft_delete_pointing_at_user(store: &mut Box<dyn Store>) {
    // Soft-delete keeps the users row alive (only `deleted_at` is set);
    // the FK still resolves and the audit row continues to point at the
    // soft-deleted user. The whole purpose of soft-delete is to keep
    // audit-log FKs valid.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    let id = store.record_pending(&entry).await.expect("record_pending");

    store.delete_user(alice.id).await.expect("soft-delete");

    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    assert_eq!(
        loaded.recipient_user_id,
        Some(alice.id),
        "audit row must still reference the soft-deleted user"
    );
}

pub async fn audit_log_clears_recipient_user_id_under_purge(store: &mut Box<dyn Store>) {
    // Hard-delete via `purge_user` is the right-to-erasure path: the FK
    // must SET NULL so the audit row's *fact* survives without
    // re-identifying the user.
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "password_reset",
    );
    let id = store.record_pending(&entry).await.expect("record_pending");

    store.purge_user(alice.id).await.expect("purge_user");

    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    assert!(
        loaded.recipient_user_id.is_none(),
        "purge_user must NULL the recipient_user_id FK"
    );
    assert_eq!(
        loaded.recipient_email, "alice@example.com",
        "the recipient email survives as the audit anchor"
    );
}

pub async fn audit_log_clears_triggered_by_user_id_under_purge(store: &mut Box<dyn Store>) {
    // Same SET NULL behaviour for `triggered_by_user_id` — covers the
    // admin-invite path where the trigger user is the admin and the
    // recipient is someone else.
    let admin = fixture_admin(store, "admin", "admin@example.com").await;
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut entry = pending_entry_for(
        Some(alice.id),
        "alice@example.com",
        "invitation",
    );
    entry.triggered_by_user_id = Some(admin.id);
    let id = store.record_pending(&entry).await.expect("record_pending");

    store.purge_user(admin.id).await.expect("purge_user");

    let loaded = store.find_audit_entry(id).await.expect("find_audit_entry");
    assert!(
        loaded.triggered_by_user_id.is_none(),
        "purge of trigger user must NULL the triggered_by_user_id FK"
    );
    assert_eq!(
        loaded.recipient_user_id,
        Some(alice.id),
        "recipient FK is unrelated and must be unaffected"
    );
}

// ─────────────────────────────────────────────────────────────────────────
// Manage
// ─────────────────────────────────────────────────────────────────────────

pub async fn empty_clears_all_data(store: &mut Box<dyn Store>) {
    let alice = fixture_admin(store, "alice", "alice@example.com").await;
    let mut maze = make_maze("alice-maze");
    store.create_maze(&alice, &mut maze).await.expect("create_maze");

    store.empty().await.expect("empty");

    // After empty(), users + their cascaded mazes are gone. We don't query
    // get_maze_items for the deleted user — FileStore reasonably errors
    // when the user's mazes directory no longer exists. The user-list
    // assertion is sufficient: no users → no mazes (mazes are owned).
    assert!(store.get_users().await.expect("get_users").is_empty());
}

// ─────────────────────────────────────────────────────────────────────────
// ScoreStore
// ─────────────────────────────────────────────────────────────────────────

fn score_entry(
    user_id: Uuid,
    maze_id: Option<&str>,
    challenge: Option<&str>,
    score: u64,
    elapsed_ms: u64,
) -> ScoreEntry {
    ScoreEntry {
        id: Uuid::new_v4(),
        user_id,
        maze_id: maze_id.map(str::to_string),
        challenge: challenge.map(str::to_string),
        score,
        elapsed_ms,
        // Millisecond precision so the value round-trips identically through the
        // SQL backends (which store RFC 3339 to millis) and FileStore.
        recorded_at: Utc::now().trunc_subsecs(3),
    }
}

const FASTEST: ScoreOrdering = ScoreOrdering {
    metric: ScoreMetric::Time,
    direction: SortDirection::Ascending,
};
const SLOWEST: ScoreOrdering = ScoreOrdering {
    metric: ScoreMetric::Time,
    direction: SortDirection::Descending,
};
const HIGHEST: ScoreOrdering = ScoreOrdering {
    metric: ScoreMetric::Score,
    direction: SortDirection::Descending,
};
const LOWEST: ScoreOrdering = ScoreOrdering {
    metric: ScoreMetric::Score,
    direction: SortDirection::Ascending,
};

// Seeds a maze owned by `owner` and returns its assigned id.
async fn fixture_maze(store: &mut Box<dyn Store>, owner: &User, name: &str) -> String {
    let mut maze = make_maze(name);
    store.create_maze(owner, &mut maze).await.expect("fixture_maze");
    maze.id
}

pub async fn score_record_round_trips_for_both_subjects(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let maze_id = fixture_maze(store, &alice, "board-maze").await;

    let on_maze = score_entry(alice.id, Some(&maze_id), None, 5, 4_200);
    let on_challenge = score_entry(alice.id, None, Some("hard:7"), 3, 9_100);
    let maze_row = store.record_score(&on_maze).await.expect("record maze score");
    let challenge_row = store
        .record_score(&on_challenge)
        .await
        .expect("record challenge score");
    assert_eq!(maze_row, on_maze.id);
    assert_eq!(challenge_row, on_challenge.id);

    let board = store
        .maze_leaderboard(&maze_id, HIGHEST, 10, 0, false)
        .await
        .expect("maze_leaderboard");
    assert_eq!(board, vec![ScoreboardEntry { entry: on_maze.clone(), username: None }]);

    let challenge_board = store
        .challenge_leaderboard("hard:7", HIGHEST, 10, 0, false)
        .await
        .expect("challenge_leaderboard");
    assert_eq!(challenge_board, vec![ScoreboardEntry { entry: on_challenge.clone(), username: None }]);

    // Personal history aggregates a player's runs across both subjects.
    let history = store
        .user_history(alice.id, 10, 0)
        .await
        .expect("user_history");
    assert_eq!(history.len(), 2);
}

pub async fn score_record_rejects_invalid_subject(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    // Both subjects set.
    let both = score_entry(alice.id, Some("m1"), Some("c:1"), 1, 100);
    assert!(store.record_score(&both).await.is_err());
    // Neither subject set.
    let neither = score_entry(alice.id, None, None, 1, 100);
    assert!(store.record_score(&neither).await.is_err());
}

pub async fn score_maze_leaderboard_orders_by_metric_and_direction(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let maze_id = fixture_maze(store, &alice, "board-maze").await;
    // (score, elapsed_ms): (10, 5000), (2, 1000), (6, 3000).
    for (score, ms) in [(10u64, 5_000u64), (2, 1_000), (6, 3_000)] {
        store
            .record_score(&score_entry(alice.id, Some(&maze_id), None, score, ms))
            .await
            .expect("record_score");
    }
    let elapsed =
        |rows: Vec<ScoreboardEntry>| rows.iter().map(|e| e.entry.elapsed_ms).collect::<Vec<_>>();
    let scores = |rows: Vec<ScoreboardEntry>| rows.iter().map(|e| e.entry.score).collect::<Vec<_>>();

    let fastest = store.maze_leaderboard(&maze_id, FASTEST, 10, 0, false).await.expect("fastest");
    assert_eq!(elapsed(fastest), vec![1_000, 3_000, 5_000]);
    let slowest = store.maze_leaderboard(&maze_id, SLOWEST, 10, 0, false).await.expect("slowest");
    assert_eq!(elapsed(slowest), vec![5_000, 3_000, 1_000]);
    let highest = store.maze_leaderboard(&maze_id, HIGHEST, 10, 0, false).await.expect("highest");
    assert_eq!(scores(highest), vec![10, 6, 2]);
    let lowest = store.maze_leaderboard(&maze_id, LOWEST, 10, 0, false).await.expect("lowest");
    assert_eq!(scores(lowest), vec![2, 6, 10]);
}

pub async fn score_challenge_leaderboard_orders_and_pages(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    for score in [10u64, 2, 6, 8] {
        store
            .record_score(&score_entry(alice.id, None, Some("c:1"), score, 1_000))
            .await
            .expect("record_score");
    }
    // Highest first: 10, 8, 6, 2. Page of 2 from offset 0, then offset 2.
    let page1 = store
        .challenge_leaderboard("c:1", HIGHEST, 2, 0, false)
        .await
        .expect("page1");
    assert_eq!(page1.iter().map(|e| e.entry.score).collect::<Vec<_>>(), vec![10, 8]);
    let page2 = store
        .challenge_leaderboard("c:1", HIGHEST, 2, 2, false)
        .await
        .expect("page2");
    assert_eq!(page2.iter().map(|e| e.entry.score).collect::<Vec<_>>(), vec![6, 2]);
    // Offset past the end yields an empty page.
    let page3 = store
        .challenge_leaderboard("c:1", HIGHEST, 2, 4, false)
        .await
        .expect("page3");
    assert!(page3.is_empty());
}

pub async fn score_user_history_is_recent_first_and_pages(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let base = Utc::now().trunc_subsecs(3);
    // Three runs at increasing completion times — newest must come first.
    for secs in [0i64, 1, 2] {
        let mut e = score_entry(alice.id, None, Some("c:1"), 1, 1_000);
        e.recorded_at = base + Duration::seconds(secs);
        store.record_score(&e).await.expect("record_score");
    }
    let all = store
        .user_history(alice.id, 10, 0)
        .await
        .expect("user_history");
    let ts: Vec<_> = all.iter().map(|e| e.recorded_at).collect();
    assert!(ts[0] > ts[1] && ts[1] > ts[2], "must be most-recent first");
    // Paging: one row per page.
    let first = store.user_history(alice.id, 1, 0).await.expect("first");
    let second = store.user_history(alice.id, 1, 1).await.expect("second");
    assert_eq!(first[0].recorded_at, ts[0]);
    assert_eq!(second[0].recorded_at, ts[1]);
}

pub async fn score_boards_are_empty_for_unknown_subject(store: &mut Box<dyn Store>) {
    assert!(store
        .maze_leaderboard("does-not-exist", FASTEST, 10, 0, false)
        .await
        .expect("maze_leaderboard")
        .is_empty());
    assert!(store
        .challenge_leaderboard("nope:0", HIGHEST, 10, 0, false)
        .await
        .expect("challenge_leaderboard")
        .is_empty());
    assert!(store
        .user_history(Uuid::new_v4(), 10, 0)
        .await
        .expect("user_history")
        .is_empty());
}

pub async fn score_delete_user_cascades_player_rows(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    store
        .record_score(&score_entry(alice.id, None, Some("c:1"), 1, 100))
        .await
        .expect("record_score");
    assert_eq!(store.user_history(alice.id, 10, 0).await.unwrap().len(), 1);
    store.delete_user(alice.id).await.expect("delete_user");
    assert!(store.user_history(alice.id, 10, 0).await.unwrap().is_empty());
}

pub async fn score_delete_maze_cascades_its_board_not_challenge_rows(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let maze_id = fixture_maze(store, &alice, "board-maze").await;
    store
        .record_score(&score_entry(alice.id, Some(&maze_id), None, 5, 100))
        .await
        .expect("record maze score");
    store
        .record_score(&score_entry(alice.id, None, Some("c:9"), 3, 100))
        .await
        .expect("record challenge score");

    store.delete_maze(&alice, &maze_id).await.expect("delete_maze");

    assert!(store
        .maze_leaderboard(&maze_id, HIGHEST, 10, 0, false)
        .await
        .unwrap()
        .is_empty());
    // The curated challenge row has no maze parent — it survives.
    assert_eq!(
        store.challenge_leaderboard("c:9", HIGHEST, 10, 0, false).await.unwrap().len(),
        1
    );
}

pub async fn score_delete_user_cascades_boards_of_owned_mazes(store: &mut Box<dyn Store>) {
    let (alice, bob) = fixture_two_users(store).await;
    // Alice owns the maze; Bob plays it (boards aggregate every player).
    let maze_id = fixture_maze(store, &alice, "alice-maze").await;
    store
        .record_score(&score_entry(bob.id, Some(&maze_id), None, 7, 2_000))
        .await
        .expect("record bob's run on alice's maze");
    assert_eq!(store.maze_leaderboard(&maze_id, HIGHEST, 10, 0, false).await.unwrap().len(), 1);

    // Deleting Alice deletes her maze, and thus its board — including Bob's run.
    store.delete_user(alice.id).await.expect("delete_user");
    assert!(store
        .maze_leaderboard(&maze_id, HIGHEST, 10, 0, false)
        .await
        .unwrap()
        .is_empty());
    assert!(store.user_history(bob.id, 10, 0).await.unwrap().is_empty());
}

/// `include_usernames` resolves each player's name on a board; omitting it
/// leaves `username` unset — regardless of backend (SqlStore joins `users`,
/// FileStore reads the player files).
pub async fn score_leaderboard_includes_usernames_when_requested(store: &mut Box<dyn Store>) {
    let (alice, bob) = fixture_two_users(store).await;
    // Both players post a run on the same curated challenge board.
    store
        .record_score(&score_entry(alice.id, None, Some("c:1"), 5, 1_000))
        .await
        .expect("alice run");
    store
        .record_score(&score_entry(bob.id, None, Some("c:1"), 9, 2_000))
        .await
        .expect("bob run");

    // include_usernames = true → both names resolved.
    let named = store
        .challenge_leaderboard("c:1", HIGHEST, 10, 0, true)
        .await
        .expect("named board");
    assert_eq!(named.len(), 2);
    let mut names: Vec<String> = named.iter().filter_map(|e| e.username.clone()).collect();
    names.sort();
    assert_eq!(names, vec![alice.username.clone(), bob.username.clone()]);

    // include_usernames = false → no names resolved.
    let anon = store
        .challenge_leaderboard("c:1", HIGHEST, 10, 0, false)
        .await
        .expect("anon board");
    assert!(anon.iter().all(|e| e.username.is_none()));
}
