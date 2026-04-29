//! Runs the shared [`Store`] trait contract against [`FileStore`].
//!
//! Each test acquires a freshly-emptied FileStore and delegates to the
//! corresponding helper in `common::store_contract`. FileStore tests are
//! serialised via `--test-threads=1` because they share the working
//! directory's `data/` tree.

mod common;

use common::store_contract as contract;
use storage::{FileStore, FileStoreConfig, Store};

async fn fresh_store() -> Box<dyn Store> {
    let mut store: Box<dyn Store> = Box::new(FileStore::new(&FileStoreConfig::default()));
    store.empty().await.expect("fresh_store: empty");
    store
}

// ─── UserStore — create / get / round-trip ───────────────────────────────

#[tokio::test]
async fn create_user_assigns_id_and_api_key() {
    let mut s = fresh_store().await;
    contract::create_user_assigns_id_and_api_key(&mut s).await;
}

#[tokio::test]
async fn create_user_round_trips_via_get_user() {
    let mut s = fresh_store().await;
    contract::create_user_round_trips_via_get_user(&mut s).await;
}

#[tokio::test]
async fn get_user_returns_not_found_for_unknown_id() {
    let mut s = fresh_store().await;
    contract::get_user_returns_not_found_for_unknown_id(&mut s).await;
}

#[tokio::test]
async fn create_user_rejects_duplicate_username() {
    let mut s = fresh_store().await;
    contract::create_user_rejects_duplicate_username(&mut s).await;
}

#[tokio::test]
async fn create_user_rejects_username_case_collision() {
    let mut s = fresh_store().await;
    contract::create_user_rejects_username_case_collision(&mut s).await;
}

#[tokio::test]
async fn create_user_rejects_duplicate_email() {
    let mut s = fresh_store().await;
    contract::create_user_rejects_duplicate_email(&mut s).await;
}

#[tokio::test]
async fn create_user_rejects_email_case_collision() {
    let mut s = fresh_store().await;
    contract::create_user_rejects_email_case_collision(&mut s).await;
}

#[tokio::test]
async fn create_user_requires_password_or_oauth() {
    let mut s = fresh_store().await;
    contract::create_user_requires_password_or_oauth(&mut s).await;
}

#[tokio::test]
async fn create_oauth_only_user_succeeds() {
    let mut s = fresh_store().await;
    contract::create_oauth_only_user_succeeds(&mut s).await;
}

// ─── UserStore — delete ──────────────────────────────────────────────────

#[tokio::test]
async fn delete_user_removes_record() {
    let mut s = fresh_store().await;
    contract::delete_user_removes_record(&mut s).await;
}

#[tokio::test]
async fn delete_user_rejects_nil_id() {
    let mut s = fresh_store().await;
    contract::delete_user_rejects_nil_id(&mut s).await;
}

#[tokio::test]
async fn delete_user_returns_not_found_for_unknown_id() {
    let mut s = fresh_store().await;
    contract::delete_user_returns_not_found_for_unknown_id(&mut s).await;
}

#[tokio::test]
async fn delete_user_cascades_to_logins() {
    let mut s = fresh_store().await;
    contract::delete_user_cascades_to_logins(&mut s).await;
}

#[tokio::test]
async fn delete_user_cascades_to_oauth_identities() {
    let mut s = fresh_store().await;
    contract::delete_user_cascades_to_oauth_identities(&mut s).await;
}

#[tokio::test]
async fn delete_user_cascades_to_mazes() {
    let mut s = fresh_store().await;
    contract::delete_user_cascades_to_mazes(&mut s).await;
}

// ─── UserStore — update ──────────────────────────────────────────────────

#[tokio::test]
async fn update_user_persists_changes() {
    let mut s = fresh_store().await;
    contract::update_user_persists_changes(&mut s).await;
}

#[tokio::test]
async fn update_user_replaces_logins_wholesale() {
    let mut s = fresh_store().await;
    contract::update_user_replaces_logins_wholesale(&mut s).await;
}

#[tokio::test]
async fn update_user_returns_not_found_for_unknown_id() {
    let mut s = fresh_store().await;
    contract::update_user_returns_not_found_for_unknown_id(&mut s).await;
}

#[tokio::test]
async fn update_user_rejects_username_case_collision() {
    let mut s = fresh_store().await;
    contract::update_user_rejects_username_case_collision(&mut s).await;
}

// ─── UserStore — find_*_by_* ─────────────────────────────────────────────

#[tokio::test]
async fn find_user_by_name_is_case_insensitive() {
    let mut s = fresh_store().await;
    contract::find_user_by_name_is_case_insensitive(&mut s).await;
}

#[tokio::test]
async fn find_user_by_name_returns_not_found() {
    let mut s = fresh_store().await;
    contract::find_user_by_name_returns_not_found(&mut s).await;
}

#[tokio::test]
async fn find_user_by_email_is_case_insensitive() {
    let mut s = fresh_store().await;
    contract::find_user_by_email_is_case_insensitive(&mut s).await;
}

#[tokio::test]
async fn find_user_by_api_key_round_trips() {
    let mut s = fresh_store().await;
    contract::find_user_by_api_key_round_trips(&mut s).await;
}

#[tokio::test]
async fn find_user_by_api_key_returns_not_found() {
    let mut s = fresh_store().await;
    contract::find_user_by_api_key_returns_not_found(&mut s).await;
}

#[tokio::test]
async fn find_user_by_login_id_returns_active_login_owner() {
    let mut s = fresh_store().await;
    contract::find_user_by_login_id_returns_active_login_owner(&mut s).await;
}

#[tokio::test]
async fn find_user_by_oauth_identity_provider_case_insensitive() {
    let mut s = fresh_store().await;
    contract::find_user_by_oauth_identity_provider_case_insensitive(&mut s).await;
}

#[tokio::test]
async fn find_user_by_oauth_identity_supports_multiple_per_user() {
    let mut s = fresh_store().await;
    contract::find_user_by_oauth_identity_supports_multiple_per_user(&mut s).await;
}

// ─── UserStore — list operations ─────────────────────────────────────────

#[tokio::test]
async fn get_users_returns_all_sorted_by_username() {
    let mut s = fresh_store().await;
    contract::get_users_returns_all_sorted_by_username(&mut s).await;
}

#[tokio::test]
async fn get_users_empty_when_store_empty() {
    let mut s = fresh_store().await;
    contract::get_users_empty_when_store_empty(&mut s).await;
}

#[tokio::test]
async fn get_admin_users_filters_to_admins_only() {
    let mut s = fresh_store().await;
    contract::get_admin_users_filters_to_admins_only(&mut s).await;
}

// ─── UserStore — init_default_admin_user ─────────────────────────────────

#[tokio::test]
async fn init_default_admin_creates_first_time() {
    let mut s = fresh_store().await;
    contract::init_default_admin_creates_first_time(&mut s).await;
}

#[tokio::test]
async fn init_default_admin_is_idempotent() {
    let mut s = fresh_store().await;
    contract::init_default_admin_is_idempotent(&mut s).await;
}

// ─── MazeStore ───────────────────────────────────────────────────────────

#[tokio::test]
async fn create_maze_assigns_id() {
    let mut s = fresh_store().await;
    contract::create_maze_assigns_id(&mut s).await;
}

#[tokio::test]
async fn create_maze_rejects_empty_name() {
    let mut s = fresh_store().await;
    contract::create_maze_rejects_empty_name(&mut s).await;
}

#[tokio::test]
async fn create_maze_rejects_name_case_collision() {
    let mut s = fresh_store().await;
    contract::create_maze_rejects_name_case_collision(&mut s).await;
}

#[tokio::test]
async fn create_maze_allows_same_name_for_different_owners() {
    let mut s = fresh_store().await;
    contract::create_maze_allows_same_name_for_different_owners(&mut s).await;
}

#[tokio::test]
async fn delete_maze_removes_record() {
    let mut s = fresh_store().await;
    contract::delete_maze_removes_record(&mut s).await;
}

#[tokio::test]
async fn delete_maze_is_scoped_to_owner() {
    let mut s = fresh_store().await;
    contract::delete_maze_is_scoped_to_owner(&mut s).await;
}

#[tokio::test]
async fn update_maze_persists_changes() {
    let mut s = fresh_store().await;
    contract::update_maze_persists_changes(&mut s).await;
}

#[tokio::test]
async fn get_maze_is_scoped_to_owner() {
    let mut s = fresh_store().await;
    contract::get_maze_is_scoped_to_owner(&mut s).await;
}

#[tokio::test]
async fn find_maze_by_name_is_case_insensitive() {
    let mut s = fresh_store().await;
    contract::find_maze_by_name_is_case_insensitive(&mut s).await;
}

#[tokio::test]
async fn get_maze_items_lists_owners_mazes_sorted() {
    let mut s = fresh_store().await;
    contract::get_maze_items_lists_owners_mazes_sorted(&mut s).await;
}

#[tokio::test]
async fn get_maze_items_includes_definition_when_requested() {
    let mut s = fresh_store().await;
    contract::get_maze_items_includes_definition_when_requested(&mut s).await;
}

#[tokio::test]
async fn get_maze_items_is_scoped_to_owner() {
    let mut s = fresh_store().await;
    contract::get_maze_items_is_scoped_to_owner(&mut s).await;
}

// ─── Manage ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn empty_clears_all_data() {
    let mut s = fresh_store().await;
    contract::empty_clears_all_data(&mut s).await;
}
