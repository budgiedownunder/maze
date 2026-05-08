//! Runs the shared [`Store`] trait contract against [`SqlStore`].
//!
//! Each test acquires a fresh `SqlStore` and delegates to the corresponding
//! helper in `common::store_contract`. Backend selection follows the
//! `DATABASE_URL` env var, defaulting to in-memory SQLite when it is unset:
//!
//!   * unset / empty                           → `sqlite::memory:`
//!   * `postgres://…` or `postgresql://…`      → PostgreSQL
//!   * `mysql://…`                             → MySQL
//!
//! For PostgreSQL/MySQL each test re-uses the same database; `store.empty()`
//! between scenarios resets state. Run with `--test-threads=1` to avoid races
//! against a shared database; SQLite-in-memory is per-connection and parallel-
//! safe but the same flag is harmless there.

#![cfg(feature = "sql-store")]

mod common;

use common::store_contract as contract;
use storage::{SqlStore, SqlStoreConfig, Store};

async fn fresh_store() -> Box<dyn Store> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite::memory:".to_string());
    // SQLite `:memory:` is per-connection: a multi-connection pool gives each
    // connection its own DB. Pin to one connection in that case so contract
    // helpers see a consistent state across awaits. File-backed SQLite,
    // postgres, and mysql all use the configured pool size.
    let max_connections = if url.contains(":memory:") { 1 } else { 5 };

    let mut store: Box<dyn Store> = Box::new(
        SqlStore::new(SqlStoreConfig {
            url,
            max_connections,
            auto_create_database: true,
            ..SqlStoreConfig::default()
        })
        .await
        .expect("fresh_store: SqlStore::new"),
    );
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
async fn find_user_by_verified_email_is_case_insensitive() {
    let mut s = fresh_store().await;
    contract::find_user_by_verified_email_is_case_insensitive(&mut s).await;
}

#[tokio::test]
async fn find_user_by_verified_email_skips_unverified_rows() {
    let mut s = fresh_store().await;
    contract::find_user_by_verified_email_skips_unverified_rows(&mut s).await;
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
async fn find_user_by_oauth_identity_strict_matching() {
    let mut s = fresh_store().await;
    contract::find_user_by_oauth_identity_strict_matching(&mut s).await;
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
async fn has_users_round_trips() {
    let mut s = fresh_store().await;
    contract::has_users_round_trips(&mut s).await;
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

// ─── UserStore — email management ────────────────────────────────────────

#[tokio::test]
async fn add_user_email_appends_a_non_primary_row() {
    let mut s = fresh_store().await;
    contract::add_user_email_appends_a_non_primary_row(&mut s).await;
}

#[tokio::test]
async fn add_user_email_with_verified_true_records_verified_at() {
    let mut s = fresh_store().await;
    contract::add_user_email_with_verified_true_records_verified_at(&mut s).await;
}

#[tokio::test]
async fn add_user_email_rejects_invalid_format() {
    let mut s = fresh_store().await;
    contract::add_user_email_rejects_invalid_format(&mut s).await;
}

#[tokio::test]
async fn add_user_email_rejects_empty() {
    let mut s = fresh_store().await;
    contract::add_user_email_rejects_empty(&mut s).await;
}

#[tokio::test]
async fn add_user_email_rejects_duplicate_across_users() {
    let mut s = fresh_store().await;
    contract::add_user_email_rejects_duplicate_across_users(&mut s).await;
}

#[tokio::test]
async fn add_user_email_rejects_duplicate_on_same_user() {
    let mut s = fresh_store().await;
    contract::add_user_email_rejects_duplicate_on_same_user(&mut s).await;
}

#[tokio::test]
async fn add_user_email_rejects_unknown_user() {
    let mut s = fresh_store().await;
    contract::add_user_email_rejects_unknown_user(&mut s).await;
}

#[tokio::test]
async fn remove_user_email_drops_a_non_primary_row() {
    let mut s = fresh_store().await;
    contract::remove_user_email_drops_a_non_primary_row(&mut s).await;
}

#[tokio::test]
async fn remove_user_email_refuses_the_only_email() {
    let mut s = fresh_store().await;
    contract::remove_user_email_refuses_the_only_email(&mut s).await;
}

#[tokio::test]
async fn remove_user_email_refuses_the_primary() {
    let mut s = fresh_store().await;
    contract::remove_user_email_refuses_the_primary(&mut s).await;
}

#[tokio::test]
async fn remove_user_email_returns_not_found_for_unknown_address() {
    let mut s = fresh_store().await;
    contract::remove_user_email_returns_not_found_for_unknown_address(&mut s).await;
}

#[tokio::test]
async fn set_primary_email_clears_other_primaries() {
    let mut s = fresh_store().await;
    contract::set_primary_email_clears_other_primaries(&mut s).await;
}

#[tokio::test]
async fn set_primary_email_rejects_unverified_target() {
    let mut s = fresh_store().await;
    contract::set_primary_email_rejects_unverified_target(&mut s).await;
}

#[tokio::test]
async fn set_primary_email_returns_not_found_for_unknown_address() {
    let mut s = fresh_store().await;
    contract::set_primary_email_returns_not_found_for_unknown_address(&mut s).await;
}

#[tokio::test]
async fn mark_email_verified_promotes_unverified_row() {
    let mut s = fresh_store().await;
    contract::mark_email_verified_promotes_unverified_row(&mut s).await;
}

#[tokio::test]
async fn mark_email_verified_returns_not_found_for_unknown_address() {
    let mut s = fresh_store().await;
    contract::mark_email_verified_returns_not_found_for_unknown_address(&mut s).await;
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
async fn update_maze_returns_not_found_for_unknown_id() {
    let mut s = fresh_store().await;
    contract::update_maze_returns_not_found_for_unknown_id(&mut s).await;
}

#[tokio::test]
async fn update_maze_rejects_empty_id() {
    let mut s = fresh_store().await;
    contract::update_maze_rejects_empty_id(&mut s).await;
}

#[tokio::test]
async fn delete_maze_rejects_empty_id() {
    let mut s = fresh_store().await;
    contract::delete_maze_rejects_empty_id(&mut s).await;
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

// ─── UserStore — soft-delete behaviour ───────────────────────────────────

#[tokio::test]
async fn delete_user_soft_deletes_and_scrambles_username() {
    let mut s = fresh_store().await;
    contract::delete_user_soft_deletes_and_scrambles_username(&mut s).await;
}

#[tokio::test]
async fn delete_user_frees_email_for_reuse() {
    let mut s = fresh_store().await;
    contract::delete_user_frees_email_for_reuse(&mut s).await;
}

#[tokio::test]
async fn delete_user_is_idempotent_per_row() {
    let mut s = fresh_store().await;
    contract::delete_user_is_idempotent_per_row(&mut s).await;
}

#[tokio::test]
async fn get_user_filters_soft_deleted() {
    let mut s = fresh_store().await;
    contract::get_user_filters_soft_deleted(&mut s).await;
}

#[tokio::test]
async fn find_user_by_name_filters_soft_deleted() {
    let mut s = fresh_store().await;
    contract::find_user_by_name_filters_soft_deleted(&mut s).await;
}

#[tokio::test]
async fn find_user_by_verified_email_filters_soft_deleted() {
    let mut s = fresh_store().await;
    contract::find_user_by_verified_email_filters_soft_deleted(&mut s).await;
}

#[tokio::test]
async fn find_user_by_api_key_filters_soft_deleted() {
    let mut s = fresh_store().await;
    contract::find_user_by_api_key_filters_soft_deleted(&mut s).await;
}

#[tokio::test]
async fn find_user_by_login_id_filters_soft_deleted() {
    let mut s = fresh_store().await;
    contract::find_user_by_login_id_filters_soft_deleted(&mut s).await;
}

#[tokio::test]
async fn find_user_by_oauth_identity_filters_soft_deleted() {
    let mut s = fresh_store().await;
    contract::find_user_by_oauth_identity_filters_soft_deleted(&mut s).await;
}

#[tokio::test]
async fn get_users_filters_soft_deleted() {
    let mut s = fresh_store().await;
    contract::get_users_filters_soft_deleted(&mut s).await;
}

#[tokio::test]
async fn get_admin_users_filters_soft_deleted() {
    let mut s = fresh_store().await;
    contract::get_admin_users_filters_soft_deleted(&mut s).await;
}

#[tokio::test]
async fn has_users_filters_soft_deleted() {
    let mut s = fresh_store().await;
    contract::has_users_filters_soft_deleted(&mut s).await;
}

// ─── UserStore — purge_user ──────────────────────────────────────────────

#[tokio::test]
async fn purge_user_truly_removes_row() {
    let mut s = fresh_store().await;
    contract::purge_user_truly_removes_row(&mut s).await;
}

#[tokio::test]
async fn purge_user_works_on_active_user() {
    let mut s = fresh_store().await;
    contract::purge_user_works_on_active_user(&mut s).await;
}

#[tokio::test]
async fn purge_user_rejects_nil_id() {
    let mut s = fresh_store().await;
    contract::purge_user_rejects_nil_id(&mut s).await;
}

#[tokio::test]
async fn purge_user_returns_not_found_for_unknown_id() {
    let mut s = fresh_store().await;
    contract::purge_user_returns_not_found_for_unknown_id(&mut s).await;
}

// ─── UserStore — has_active_admin_user ───────────────────────────────────

#[tokio::test]
async fn has_active_admin_user_returns_true_when_active_admin_exists() {
    let mut s = fresh_store().await;
    contract::has_active_admin_user_returns_true_when_active_admin_exists(&mut s).await;
}

#[tokio::test]
async fn has_active_admin_user_returns_false_when_only_admin_is_soft_deleted() {
    let mut s = fresh_store().await;
    contract::has_active_admin_user_returns_false_when_only_admin_is_soft_deleted(&mut s).await;
}

#[tokio::test]
async fn has_active_admin_user_returns_false_when_no_users_exist() {
    let mut s = fresh_store().await;
    contract::has_active_admin_user_returns_false_when_no_users_exist(&mut s).await;
}

#[tokio::test]
async fn has_active_admin_user_ignores_non_admin_users() {
    let mut s = fresh_store().await;
    contract::has_active_admin_user_ignores_non_admin_users(&mut s).await;
}

// ─── TokenStore ──────────────────────────────────────────────────────────

#[tokio::test]
async fn create_token_round_trips_via_find() {
    let mut s = fresh_store().await;
    contract::create_token_round_trips_via_find(&mut s).await;
}

#[tokio::test]
async fn create_token_preserves_target_email_for_verification() {
    let mut s = fresh_store().await;
    contract::create_token_preserves_target_email_for_verification(&mut s).await;
}

#[tokio::test]
async fn create_token_rejects_duplicate_id() {
    let mut s = fresh_store().await;
    contract::create_token_rejects_duplicate_id(&mut s).await;
}

#[tokio::test]
async fn find_token_returns_not_found_for_unknown_id() {
    let mut s = fresh_store().await;
    contract::find_token_returns_not_found_for_unknown_id(&mut s).await;
}

#[tokio::test]
async fn find_token_filters_expired_tokens() {
    let mut s = fresh_store().await;
    contract::find_token_filters_expired_tokens(&mut s).await;
}

#[tokio::test]
async fn consume_token_marks_consumed_at() {
    let mut s = fresh_store().await;
    contract::consume_token_marks_consumed_at(&mut s).await;
}

#[tokio::test]
async fn consume_token_twice_rejects_second_call() {
    let mut s = fresh_store().await;
    contract::consume_token_twice_rejects_second_call(&mut s).await;
}

#[tokio::test]
async fn consume_token_returns_not_found_for_unknown_id() {
    let mut s = fresh_store().await;
    contract::consume_token_returns_not_found_for_unknown_id(&mut s).await;
}

#[tokio::test]
async fn consume_token_rejects_expired_token() {
    let mut s = fresh_store().await;
    contract::consume_token_rejects_expired_token(&mut s).await;
}

#[tokio::test]
async fn delete_user_cascades_to_one_time_tokens() {
    let mut s = fresh_store().await;
    contract::delete_user_cascades_to_one_time_tokens(&mut s).await;
}

#[tokio::test]
async fn purge_expired_removes_only_expired_unconsumed_rows() {
    let mut s = fresh_store().await;
    contract::purge_expired_removes_only_expired_unconsumed_rows(&mut s).await;
}

#[tokio::test]
async fn consume_token_concurrent_race_has_exactly_one_winner() {
    let s = fresh_store().await;
    contract::consume_token_concurrent_race_has_exactly_one_winner(s).await;
}

// ─── EmailAuditLog ───────────────────────────────────────────────────────

#[tokio::test]
async fn record_pending_returns_id_and_inserts_pending_row() {
    let mut s = fresh_store().await;
    contract::record_pending_returns_id_and_inserts_pending_row(&mut s).await;
}

#[tokio::test]
async fn record_pending_rejects_duplicate_id() {
    let mut s = fresh_store().await;
    contract::record_pending_rejects_duplicate_id(&mut s).await;
}

#[tokio::test]
async fn find_audit_entry_returns_not_found_for_unknown_id() {
    let mut s = fresh_store().await;
    contract::find_audit_entry_returns_not_found_for_unknown_id(&mut s).await;
}

#[tokio::test]
async fn update_outcome_to_accepted_populates_provider_message_id() {
    let mut s = fresh_store().await;
    contract::update_outcome_to_accepted_populates_provider_message_id(&mut s).await;
}

#[tokio::test]
async fn update_outcome_to_failed_populates_error_class() {
    let mut s = fresh_store().await;
    contract::update_outcome_to_failed_populates_error_class(&mut s).await;
}

#[tokio::test]
async fn update_outcome_to_failed_populates_error_message() {
    let mut s = fresh_store().await;
    contract::update_outcome_to_failed_populates_error_message(&mut s).await;
}

#[tokio::test]
async fn update_outcome_rejects_pending_target() {
    let mut s = fresh_store().await;
    contract::update_outcome_rejects_pending_target(&mut s).await;
}

#[tokio::test]
async fn update_outcome_returns_not_found_for_unknown_id() {
    let mut s = fresh_store().await;
    contract::update_outcome_returns_not_found_for_unknown_id(&mut s).await;
}

#[tokio::test]
async fn find_recent_audit_entries_returns_user_rows_descending_capped_at_limit() {
    let mut s = fresh_store().await;
    contract::find_recent_audit_entries_returns_user_rows_descending_capped_at_limit(&mut s).await;
}

#[tokio::test]
async fn find_recent_audit_entries_is_empty_when_user_has_none() {
    let mut s = fresh_store().await;
    contract::find_recent_audit_entries_is_empty_when_user_has_none(&mut s).await;
}

#[tokio::test]
async fn audit_log_supports_anti_enumeration_null_recipient() {
    let mut s = fresh_store().await;
    contract::audit_log_supports_anti_enumeration_null_recipient(&mut s).await;
}

#[tokio::test]
async fn audit_log_survives_soft_delete_pointing_at_user() {
    let mut s = fresh_store().await;
    contract::audit_log_survives_soft_delete_pointing_at_user(&mut s).await;
}

#[tokio::test]
async fn audit_log_clears_recipient_user_id_under_purge() {
    let mut s = fresh_store().await;
    contract::audit_log_clears_recipient_user_id_under_purge(&mut s).await;
}

#[tokio::test]
async fn audit_log_clears_triggered_by_user_id_under_purge() {
    let mut s = fresh_store().await;
    contract::audit_log_clears_triggered_by_user_id_under_purge(&mut s).await;
}

// ─── Manage ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn empty_clears_all_data() {
    let mut s = fresh_store().await;
    contract::empty_clears_all_data(&mut s).await;
}
