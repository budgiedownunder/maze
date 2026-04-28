//! End-to-end smoke test for `SqlStore` against any SQLx-supported backend.
//!
//! Defaults to SQLite in-memory; honours `DATABASE_URL` for postgres/mysql.
//! Exercises create/read/update/delete on users, mazes, and OAuth identities,
//! plus login lookup and admin filtering.
//!
//! Usage:
//!     cargo run --example sql_store_smoke -p storage --features sql-store
//!
//!     DATABASE_URL=postgres://postgres:pw@localhost/postgres \
//!         cargo run --example sql_store_smoke -p storage --features sql-store

use data_model::{Maze, MazeDefinition, OAuthIdentity, User, UserLogin};
use storage::store::{Manage, MazeStore, UserStore};
use storage::{SqlStore, SqlStoreConfig};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
    println!("Connecting: {url}");

    // SQLite's `:memory:` is per-connection, so a multi-connection pool gives
    // each connection its own DB. For the smoke test we pin to one connection
    // when running against an in-memory SQLite. File-based SQLite, postgres,
    // and mysql all use the configured pool size.
    let max_connections = if url.contains(":memory:") { 1 } else { 5 };

    let mut store = SqlStore::new(SqlStoreConfig {
        url,
        max_connections,
        auto_create_database: true,
    })
    .await?;

    // Start clean so reruns against the same DB work.
    store.empty().await?;

    // ── Users ────────────────────────────────────────────────────────────
    let mut alice = User::default();
    alice.username = "alice".into();
    alice.email = "alice@example.com".into();
    alice.password_hash = "argon2id$dummyhash".into();
    alice.is_admin = true;
    store.create_user(&mut alice).await?;
    println!("Created admin user '{}' with id {}", alice.username, alice.id);

    let mut bob = User::default();
    bob.username = "bob".into();
    bob.email = "bob@example.com".into();
    bob.oauth_identities.push(OAuthIdentity::new(
        "google".into(),
        "google-sub-bob".into(),
        Some("bob@gmail.com".into()),
    ));
    store.create_user(&mut bob).await?;
    println!("Created OAuth-only user '{}' with id {}", bob.username, bob.id);

    // ── Look-ups ─────────────────────────────────────────────────────────
    let alice_loaded = store.find_user_by_name("ALICE").await?; // case-insensitive
    assert_eq!(alice_loaded.id, alice.id);
    println!("find_user_by_name OK (case-insensitive)");

    let bob_via_oauth = store
        .find_user_by_oauth_identity("Google", "google-sub-bob")
        .await?;
    assert_eq!(bob_via_oauth.id, bob.id);
    assert_eq!(bob_via_oauth.oauth_identities.len(), 1);
    println!("find_user_by_oauth_identity OK (provider case-insensitive, identities loaded)");

    let admins = store.get_admin_users().await?;
    assert_eq!(admins.len(), 1);
    assert_eq!(admins[0].id, alice.id);
    println!("get_admin_users OK");

    // ── Logins ───────────────────────────────────────────────────────────
    let login = UserLogin::new(24, Some("127.0.0.1".into()), Some("smoke-test".into()));
    let login_id = login.id;
    alice.logins.push(login);
    store.update_user(&mut alice).await?;
    let alice_via_login = store.find_user_by_login_id(login_id).await?;
    assert_eq!(alice_via_login.id, alice.id);
    assert_eq!(alice_via_login.logins.len(), 1);
    println!("update_user + find_user_by_login_id OK");

    // ── Mazes ────────────────────────────────────────────────────────────
    let mut maze = Maze::new(MazeDefinition::new(3, 3));
    maze.name = "smoke-test-maze".into();
    store.create_maze(&alice, &mut maze).await?;
    let maze_id = maze.id.clone();
    let loaded = store.get_maze(&alice, &maze_id).await?;
    assert_eq!(loaded.name, "smoke-test-maze");
    println!("create_maze + get_maze OK (id = {maze_id})");

    let items = store.get_maze_items(&alice, false).await?;
    assert_eq!(items.len(), 1);
    println!("get_maze_items OK ({} item)", items.len());

    // ── Cascade delete ───────────────────────────────────────────────────
    let bob_id = bob.id;
    store.delete_user(bob_id).await?;
    assert!(matches!(
        store.find_user_by_oauth_identity("google", "google-sub-bob").await,
        Err(storage::Error::UserNotFound())
    ));
    println!("delete_user cascades to oauth_identities OK");

    // ── Final cleanup ────────────────────────────────────────────────────
    store.empty().await?;
    let after = store.get_users().await?;
    assert!(after.is_empty());
    println!("Manage::empty OK");

    println!("\nAll SqlStore smoke checks passed.");
    Ok(())
}
