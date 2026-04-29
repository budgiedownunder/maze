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

use data_model::{Maze, MazeDefinition, OAuthIdentity, User, UserLogin};
use storage::{Error, Store};
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
        email: email.to_string(),
        password_hash: "argon2id$contract-test-hash".to_string(),
        api_key: Uuid::nil(),
        logins: vec![],
        oauth_identities: vec![],
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
    alice.email = "alice-new@example.com".to_string();
    store.update_user(&mut alice).await.expect("update_user");

    let loaded = store.get_user(alice.id).await.expect("get_user");
    assert_eq!(loaded.full_name, "Alice Updated");
    assert_eq!(loaded.email, "alice-new@example.com");
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

pub async fn find_user_by_email_is_case_insensitive(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "Alice@Example.COM").await;
    let by_lower = store.find_user_by_email("alice@example.com").await.expect("lower");
    let by_upper = store.find_user_by_email("ALICE@EXAMPLE.COM").await.expect("upper");
    assert_eq!(by_lower.id, alice.id);
    assert_eq!(by_upper.id, alice.id);
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

pub async fn update_maze_persists_changes(store: &mut Box<dyn Store>) {
    let alice = fixture_user(store, "alice", "alice@example.com").await;
    let mut maze = make_maze("orig-name");
    store.create_maze(&alice, &mut maze).await.expect("create_maze");

    maze.name = "renamed".to_string();
    store.update_maze(&alice, &mut maze).await.expect("update_maze");

    let loaded = store.get_maze(&alice, &maze.id).await.expect("get_maze");
    assert_eq!(loaded.name, "renamed");
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
