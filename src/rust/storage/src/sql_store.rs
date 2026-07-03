//! SQL-backed [`Store`] implementation.
//!
//! Supports SQLite, PostgreSQL, and MySQL through SQLx's [`Any`](sqlx::Any)
//! driver. Selection happens at runtime via the connection URL — there are no
//! per-backend sub-features. The schema lives in `migrations/0001_initial.sql`
//! and is applied automatically by [`SqlStore::new`].
//!
//! Timestamps are written to TEXT columns in a single canonical RFC 3339
//! shape (millisecond precision, trailing `Z`) so that lexicographic order
//! matches chronological order across all three backends — keeping SQL-side
//! range queries (`WHERE expires_at < ?`, `ORDER BY last_seen_at DESC`)
//! portable. See `migrations/0001_initial.sql` for the full design rationale.

use crate::store::{
    EmailAuditLog, GameStore, Manage, MazeStore, ScoreEntry, ScoreMetric, ScoreOrdering,
    ScoreStore, ScoreboardEntry, SortDirection, TokenStore, UserStore,
};
use crate::{
    validation::{validate_email_format, validate_game_definition_config_size, validate_maze_cell_count, validate_maze_definition_size, validate_maze_feature_count, validate_maze_object_counts, validate_user_fields},
    Error, MazeItem, Store, MAX_GAME_DEFINITION_CONFIG_BYTES,
};
use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, SubsecRound, Utc};
use data_model::{
    AuditOutcome, EmailAuditEntry, GameDefinition, Maze, OAuthIdentity, OneTimeToken, Rotation,
    TokenPurpose, User, UserEmail, UserLogin, Visibility, truncate_email_audit_error_message,
};
use sqlx::any::{install_default_drivers, AnyPoolOptions, AnyRow};
use sqlx::migrate::MigrateDatabase;
use sqlx::{AnyPool, Row};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Backend detection + placeholder translation
// ─────────────────────────────────────────────────────────────────────────────

/// Concrete backend behind an `AnyPool`. SQLx 0.8's `Any` driver intentionally
/// does not translate `?` placeholders to `$N` for PostgreSQL when raw
/// `sqlx::query("...")` strings are used — that translation only happens via
/// the compile-time `query!` / `query_as!` macros. We therefore detect the
/// backend up front and translate placeholders ourselves only for PostgreSQL.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SqlBackend {
    Sqlite,
    Postgres,
    MySql,
}

impl SqlBackend {
    fn from_url(url: &str) -> Result<Self, Error> {
        let lower = url.to_ascii_lowercase();
        if lower.starts_with("sqlite:") {
            Ok(SqlBackend::Sqlite)
        } else if lower.starts_with("postgres:") || lower.starts_with("postgresql:") {
            Ok(SqlBackend::Postgres)
        } else if lower.starts_with("mysql:") {
            Ok(SqlBackend::MySql)
        } else {
            Err(Error::Other(format!(
                "unsupported sqlx URL scheme: {url} (expected sqlite:, postgres:, or mysql:)"
            )))
        }
    }
}

/// Returns the SQL string adapted to the target backend's placeholder style.
///
/// SQLite and MySQL accept `?` placeholders natively; the input is returned
/// untouched. PostgreSQL requires `$1, $2, ...`, so for that backend the SQL
/// is walked once and each `?` outside a string literal is rewritten in
/// order. The walker handles doubled `''` escapes inside literals so a
/// literal containing `?` is left alone.
fn q(kind: SqlBackend, sql: &str) -> String {
    if kind != SqlBackend::Postgres {
        return sql.to_string();
    }
    let mut out = String::with_capacity(sql.len() + 8);
    let mut counter = 1usize;
    let mut in_str = false;
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\'' => {
                out.push(c);
                if in_str && chars.peek() == Some(&'\'') {
                    out.push(chars.next().unwrap());
                } else {
                    in_str = !in_str;
                }
            }
            '?' if !in_str => {
                out.push('$');
                out.push_str(&counter.to_string());
                counter += 1;
            }
            _ => out.push(c),
        }
    }
    out
}

// ─────────────────────────────────────────────────────────────────────────────
// Datetime format helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Canonical SQL serialisation for `DateTime<Utc>`.
///
/// Always millisecond precision + trailing `Z` so every row uses the same
/// fixed-width shape. Mixing precisions would break the lex == chrono ordering
/// invariant the schema relies on.
fn datetime_to_sql(dt: DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// Reverse of [`datetime_to_sql`]. Accepts any RFC 3339 input.
fn datetime_from_sql(s: &str) -> Result<DateTime<Utc>, Error> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| Error::Other(format!("invalid timestamp '{s}': {e}")))
}

fn map_sqlx_err(e: sqlx::Error) -> Error {
    Error::Other(format!("sqlx: {e}"))
}

fn parse_uuid(field: &str, raw: &str) -> Result<Uuid, Error> {
    Uuid::parse_str(raw).map_err(|e| Error::Other(format!("invalid {field} '{raw}': {e}")))
}

/// Surfaces a "should be impossible" multi-row result loudly rather than
/// silently picking a winner.
///
/// The four `find_*` queries that match case-insensitively (`LOWER(...)`)
/// rely on the application layer (`check_user_unique_fields`) to keep the
/// underlying case-sensitive UNIQUE indexes free of equivalent-but-cased
/// duplicates. If that invariant is ever broken (for example by a direct DB
/// edit, a future code path that bypasses validation, or a constraint that
/// drifts in a future migration), this turns silent corruption into a loud
/// runtime error with enough context to investigate.
fn integrity_violation(detail: &str) -> Error {
    log::error!("storage integrity violation: {detail}");
    Error::Other(format!("storage integrity violation: {detail}"))
}

// ─────────────────────────────────────────────────────────────────────────────
// Limits
// ─────────────────────────────────────────────────────────────────────────────

/// Cell-count ceiling enforced by [`SqlStore`] on `create_maze` and
/// `update_maze`. Derived from `mazes.definition VARCHAR(16000)` in
/// `migrations/0001_initial.sql`: with the existing serialisation
/// (`4·N·M + 2·N + 10` chars for an N-row × M-col grid) the 16,000-char
/// column maxes out at ~3,844 cells, so 3,600 sits inside that with a
/// margin while still allowing a 60×60 square.
///
/// Applied uniformly across all SQL drivers — SQLite ignores the column
/// length declaration, but enforcing the cap uniformly avoids dev-vs-prod
/// divergence when the same data set is later loaded under MySQL or
/// PostgreSQL, both of which do enforce VARCHAR length.
pub const MAX_MAZE_CELLS: usize = 3_600;

/// Byte ceiling enforced by [`SqlStore`] on the serialised maze written to the
/// `mazes.definition VARCHAR(16000)` column. The cell-count cap above assumes
/// plain single-character cells (`4·N·M + …` chars); per-cell entity overrides
/// inflate individual cells well beyond that, so a maze can be under the
/// cell-count cap yet still overflow the column. This byte cap is the
/// authoritative storage guard — it is checked against the exact string about
/// to be written, so an over-cap maze is refused with
/// [`Error::MazeDefinitionTooLarge`] rather than truncated by the database.
/// Matches the column width; applied uniformly across drivers (SQLite ignores
/// `VARCHAR` length, but enforcing it avoids dev-vs-prod divergence under
/// MySQL / PostgreSQL).
pub const MAX_MAZE_DEFINITION_BYTES: usize = 16_000;

// ─────────────────────────────────────────────────────────────────────────────
// Configuration
// ─────────────────────────────────────────────────────────────────────────────

/// Configuration for [`SqlStore`].
#[derive(Debug, Clone)]
pub struct SqlStoreConfig {
    /// SQLx-style connection URL: `sqlite::memory:`, `sqlite:path/to.db`,
    /// `postgres://user:pass@host/db`, or `mysql://user:pass@host/db`.
    pub url: String,
    /// Maximum pool connections.
    pub max_connections: u32,
    /// If true and the target database does not exist, create it before
    /// running migrations. Requires `CREATEDB` privilege on PostgreSQL or
    /// server-level `CREATE` on MySQL; for SQLite this just creates the file.
    /// Default: `false` (cloud deployments expect the DB to be pre-provisioned).
    pub auto_create_database: bool,
    /// Idle-connection timeout, in seconds. Pool connections that sit idle
    /// longer than this are dropped — important for cloud databases that
    /// kill idle TCP sockets.
    pub idle_timeout_secs: u64,
    /// Pool-acquisition timeout, in seconds. Bounds both the initial
    /// connect inside [`SqlStore::new`] and `pool.acquire()` calls thereafter
    /// — `AnyPoolOptions` does not split the two.
    pub acquire_timeout_secs: u64,
}

impl Default for SqlStoreConfig {
    fn default() -> Self {
        Self {
            url: "sqlite::memory:".to_string(),
            max_connections: 5,
            auto_create_database: false,
            idle_timeout_secs: 600,
            acquire_timeout_secs: 30,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Store
// ─────────────────────────────────────────────────────────────────────────────

/// SQL-backed [`Store`]. See module docs.
pub struct SqlStore {
    pool: AnyPool,
    kind: SqlBackend,
}

impl SqlStore {
    /// Connects to the configured database, optionally creates it, and runs
    /// any pending migrations. Subsequent calls against an already-migrated
    /// database are idempotent — SQLx tracks applied migrations in its own
    /// `_sqlx_migrations` table.
    ///
    /// # Returns
    ///
    /// A new [`SqlStore`] connected to the configured database with all
    /// migrations applied. Errors if the URL scheme is unsupported, the
    /// database is unreachable, the optional `auto_create_database` step
    /// fails, or a migration fails to apply.
    ///
    /// # Examples
    ///
    /// Create an in-memory SQLite store, run migrations, and verify the
    /// schema is queryable
    /// ```
    /// # tokio_test::block_on(async {
    /// use storage::{SqlStore, SqlStoreConfig, UserStore};
    ///
    /// let store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     // SQLite `:memory:` is per-connection; pin to one connection so
    ///     // every query sees the same database.
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // The store has just been migrated — no users yet.
    /// assert!(!store.has_users().await.expect("has_users"));
    /// # });
    /// ```
    ///
    /// For PostgreSQL or MySQL, set `url` to a `postgres://…` or `mysql://…`
    /// connection string. Runnable starter configurations for every backend
    /// are checked in alongside `maze_web_server` — see
    /// `config.example.sqlite.toml`, `config.example.postgres.toml`,
    /// `config.example.postgres-cloud.toml`, and `config.example.mysql.toml`.
    pub async fn new(config: SqlStoreConfig) -> Result<Self, Error> {
        install_default_drivers();

        let kind = SqlBackend::from_url(&config.url)?;

        if config.auto_create_database
            && !sqlx::Any::database_exists(&config.url)
                .await
                .map_err(map_sqlx_err)?
        {
            sqlx::Any::create_database(&config.url)
                .await
                .map_err(map_sqlx_err)?;
        }

        let pool = AnyPoolOptions::new()
            .max_connections(config.max_connections)
            .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_secs))
            .idle_timeout(Some(std::time::Duration::from_secs(config.idle_timeout_secs)))
            .connect(&config.url)
            .await
            .map_err(map_sqlx_err)?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .map_err(|e| Error::Other(format!("migration failed: {e}")))?;

        // MySQL post-migration patch: force `oauth_identities.provider_user_id`
        // to a case-sensitive collation. The default `utf8mb4_unicode_ci` makes
        // string comparisons case-insensitive, but the OAuth/OIDC `sub` claim
        // is opaque and case-significant per spec. PG and SQLite already use
        // case-sensitive defaults — no equivalent needed there. The schema
        // can't carry `COLLATE utf8mb4_bin` directly because it's MySQL-only
        // syntax that PG/SQLite reject, so it lives here, run after the
        // portable migration. ALTER TABLE on an already-utf8mb4_bin column is
        // an INPLACE metadata-only op, fast and idempotent.
        if kind == SqlBackend::MySql {
            sqlx::query(
                "ALTER TABLE oauth_identities \
                 MODIFY provider_user_id VARCHAR(255) COLLATE utf8mb4_bin NOT NULL",
            )
            .execute(&pool)
            .await
            .map_err(map_sqlx_err)?;
        }

        // Retire the legacy `users.email` column. Migration 0002 normalised
        // email out of `users` into the new `user_emails` table but did not
        // drop the original column there because the portable migration
        // dialect can't express it: SQLite refuses to `DROP COLUMN` on a
        // UNIQUE-bearing column at all, and PG / MySQL each need their own
        // syntax to drop the implicit constraint first. We therefore retire
        // it here per-backend, after the portable migrations have run.
        //
        // Idempotency strategy: each branch is naturally idempotent — runs
        // unconditionally on every startup but is a no-op once `users.email`
        // is gone. PG and MySQL get this for free via `IF EXISTS` clauses;
        // SQLite probes `PRAGMA table_info` and short-circuits when the
        // column is already absent. No `_sqlx_migrations` version gate
        // needed (matches the COLLATE pattern above).
        retire_legacy_users_email_column(&pool, kind).await?;

        // Backfill `users.created_at` / `users.last_sign_in_at` for any
        // pre-v8 row that still has them NULL. The migration-run timestamp
        // is captured here at startup (rather than baked into the static
        // SQL of `0008_user_timestamps.sql`) so the value reflects when the
        // upgrade actually happened. Idempotent — once every row has a
        // value the UPDATEs match zero rows.
        backfill_user_timestamps_if_null(&pool, kind).await?;

        // Create the `user_avatars` table. Done here per-backend rather than
        // in a static migration because its binary column type is the one
        // piece of the schema with no portable spelling across all three
        // backends — see `create_user_avatars_table`.
        create_user_avatars_table(&pool, kind).await?;

        Ok(Self { pool, kind })
    }
}

/// Creates the `user_avatars` table if it does not already exist.
///
/// Lives here rather than in a static `migrations/` file because the binary
/// `image_data` column has no portable spelling: PostgreSQL has only `BYTEA`,
/// MySQL only `BLOB`/`LONGBLOB`, and SQLite takes `BLOB` — a single
/// `sqlx::migrate!` SQL string applied verbatim to every backend can't
/// satisfy all three. The avatar *value* round-trips uniformly through
/// SQLx-Any (`Vec<u8>` ⇄ `AnyValueKind::Blob` on every driver), so only the
/// DDL needs per-backend dispatch — the read/write code in
/// `set_user_avatar` / `get_user_avatar` / `clear_user_avatar` is shared.
/// This mirrors the per-backend approach `retire_legacy_users_email_column`
/// takes for DDL the portable migration dialect can't express.
///
/// `image_data` holds the canonical avatar PNG (one row per user, keyed by
/// `user_id`). The FK `ON DELETE CASCADE` makes a hard-delete of the owning
/// user remove the avatar row; the cascade is also issued explicitly in
/// `empty`, matching the `score_history` backstop. Idempotent via
/// `CREATE TABLE IF NOT EXISTS` (SQLx tracks no version for it, so it simply
/// no-ops once the table exists).
async fn create_user_avatars_table(pool: &AnyPool, kind: SqlBackend) -> Result<(), Error> {
    let blob_type = match kind {
        SqlBackend::Postgres => "BYTEA",
        SqlBackend::MySql => "LONGBLOB",
        SqlBackend::Sqlite => "BLOB",
    };
    let sql = format!(
        "CREATE TABLE IF NOT EXISTS user_avatars (\
             user_id VARCHAR(36) NOT NULL PRIMARY KEY, \
             image_data {blob_type} NOT NULL, \
             CONSTRAINT fk_user_avatars_user_id \
                 FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE\
         )"
    );
    sqlx::query(&sql)
        .execute(pool)
        .await
        .map_err(|e| Error::Other(format!("create_user_avatars_table failed: sqlx: {e}")))?;
    Ok(())
}

/// Backfills `users.created_at` and `users.last_sign_in_at` for any row
/// where they are still NULL. Companion to `0008_user_timestamps.sql`,
/// which adds the columns nullable; the application's `User` struct treats
/// `created_at` as non-nullable so this backfill must complete before any
/// read goes through `user_from_row`.
///
/// `created_at` is unconditionally backfilled to the migration-run
/// timestamp captured here at startup — non-nullable in the app, so every
/// pre-existing row needs a value, and we don't have a more accurate
/// value to substitute.
///
/// `last_sign_in_at` is backfilled to the timestamp of each user's most
/// recent login row — the most accurate evidence we have of when they
/// last signed in. Users with no `user_logins` row stay at NULL so the
/// welcome-banner trigger `User::is_first_sign_in()` (=
/// `last_sign_in_at.is_none() && logins.is_empty()`) correctly fires on
/// their first actual sign-in.
///
/// Implementation note: the `last_sign_in_at` step deliberately avoids a
/// single `UPDATE … (correlated subquery) WHERE …` statement, which
/// PostgreSQL rejects with `syntax error at or near "WHERE"` in some
/// versions even though it's accepted by SQLite and MySQL. Iterating
/// `(user_id, MAX(created_at))` rows in Rust and issuing one parameterised
/// `UPDATE` per user is portable across all three backends and gives a
/// clear error message naming the failing step if anything goes wrong.
async fn backfill_user_timestamps_if_null(
    pool: &AnyPool,
    kind: SqlBackend,
) -> Result<(), Error> {
    log::info!(
        "SqlStore: backfilling users.created_at to migration-run timestamp \
         for any pre-v8 row that's still NULL"
    );
    sqlx::query(&q(
        kind,
        "UPDATE users SET created_at = ? WHERE created_at IS NULL",
    ))
        .bind(datetime_to_sql(Utc::now()))
        .execute(pool)
        .await
        .map_err(|e| {
            Error::Other(format!(
                "backfill_user_timestamps_if_null: \
                 UPDATE users.created_at failed: sqlx: {e}"
            ))
        })?;

    log::info!(
        "SqlStore: backfilling users.last_sign_in_at from MAX(user_logins.created_at) \
         for any pre-v8 row that's still NULL"
    );
    let rows = sqlx::query(
        "SELECT user_id, MAX(created_at) AS max_created_at \
         FROM user_logins GROUP BY user_id",
    )
        .fetch_all(pool)
        .await
        .map_err(|e| {
            Error::Other(format!(
                "backfill_user_timestamps_if_null: \
                 SELECT MAX(user_logins.created_at) per user failed: sqlx: {e}"
            ))
        })?;
    for row in rows {
        let user_id: String = row.try_get("user_id").map_err(map_sqlx_err)?;
        let max_created_at: String =
            row.try_get("max_created_at").map_err(map_sqlx_err)?;
        sqlx::query(&q(
            kind,
            "UPDATE users SET last_sign_in_at = ? \
             WHERE id = ? AND last_sign_in_at IS NULL",
        ))
            .bind(&max_created_at)
            .bind(&user_id)
            .execute(pool)
            .await
            .map_err(|e| {
                Error::Other(format!(
                    "backfill_user_timestamps_if_null: \
                     UPDATE users.last_sign_in_at for user_id={user_id} failed: sqlx: {e}"
                ))
            })?;
    }
    Ok(())
}

/// Per-backend retirement of `users.email`. Runs every startup; naturally
/// idempotent. See `SqlStore::new` for context. Each `sqlx::query` below
/// is wrapped with a `map_err` that prefixes the error with the
/// backend-specific step name so a Postgres/MySQL/SQLite-specific failure
/// is identifiable from the log alone.
async fn retire_legacy_users_email_column(
    pool: &AnyPool,
    kind: SqlBackend,
) -> Result<(), Error> {
    /// Helper to wrap an sqlx error with the function name and a step
    /// description. Mirrors the pattern in
    /// `backfill_user_timestamps_if_null`.
    fn err(step: &str, e: sqlx::Error) -> Error {
        Error::Other(format!(
            "retire_legacy_users_email_column: {step} failed: sqlx: {e}"
        ))
    }
    match kind {
        SqlBackend::Postgres => {
            log::info!(
                "SqlStore: retiring legacy users.email column \
                 (PostgreSQL DROP COLUMN IF EXISTS)"
            );
            // Dropping the column also drops the implicit `users_email_key`
            // UNIQUE constraint and its supporting index in PG.
            sqlx::query("ALTER TABLE users DROP COLUMN IF EXISTS email")
                .execute(pool)
                .await
                .map_err(|e| err("ALTER TABLE users DROP COLUMN email", e))?;
        }
        SqlBackend::MySql => {
            log::info!(
                "SqlStore: retiring legacy users.email column (MySQL probe + drop)"
            );
            // The UNIQUE on `email` creates an index named after the column
            // by convention. We can't rely on `IF EXISTS` here:
            //   * `ALTER TABLE … DROP INDEX IF EXISTS …` is rejected by MySQL
            //     entirely (error 1064) — IF EXISTS isn't accepted on the
            //     ALTER TABLE form of DROP INDEX even in 8.0+.
            //   * `ALTER TABLE … DROP COLUMN IF EXISTS …` only landed in MySQL
            //     8.0.29 (Apr 2022); earlier 8.0.x rejects it the same way.
            // Probe INFORMATION_SCHEMA first instead — works on any 5.7+ /
            // 8.x server we'll meet.
            let has_index = sqlx::query(
                "SELECT 1 FROM INFORMATION_SCHEMA.STATISTICS \
                 WHERE TABLE_SCHEMA = DATABASE() \
                   AND TABLE_NAME = 'users' \
                   AND INDEX_NAME = 'email'",
            )
            .fetch_optional(pool)
            .await
            .map_err(|e| err("probe INFORMATION_SCHEMA.STATISTICS for users.email index", e))?
            .is_some();
            if has_index {
                sqlx::query("ALTER TABLE users DROP INDEX email")
                    .execute(pool)
                    .await
                    .map_err(|e| err("ALTER TABLE users DROP INDEX email", e))?;
            }
            let has_column = sqlx::query(
                "SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS \
                 WHERE TABLE_SCHEMA = DATABASE() \
                   AND TABLE_NAME = 'users' \
                   AND COLUMN_NAME = 'email'",
            )
            .fetch_optional(pool)
            .await
            .map_err(|e| err("probe INFORMATION_SCHEMA.COLUMNS for users.email column", e))?
            .is_some();
            if has_column {
                sqlx::query("ALTER TABLE users DROP COLUMN email")
                    .execute(pool)
                    .await
                    .map_err(|e| err("ALTER TABLE users DROP COLUMN email", e))?;
            }
        }
        SqlBackend::Sqlite => {
            // SQLite forbids `DROP COLUMN` on a UNIQUE-bearing column and
            // forbids dropping the implicit `sqlite_autoindex_users_*` index
            // — the only path is a full table rebuild.
            //
            // Critical: every statement below must run on the **same**
            // pooled connection. SQLite caches the schema per connection;
            // splitting `DROP TABLE users` and `ALTER TABLE users_new RENAME
            // TO users` across two pool connections leaves the renaming
            // connection still seeing `users` in its cached view and the
            // rename fails with "there is already another table or index
            // with this name: users". `pool.acquire()` pins one connection
            // for the whole rebuild.
            let mut conn = pool
                .acquire()
                .await
                .map_err(|e| err("acquire dedicated connection for SQLite rebuild", e))?;

            // Probe what state the schema is in so we can pick the right
            // path. Three states matter:
            //   * `users` has `email` column        → full rebuild needed
            //   * `users_new` exists, `users` does not → recover from a
            //                                            previous aborted
            //                                            rebuild via rename
            //   * everything else                   → no-op (already retired)
            let users_exists = sqlx::query(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'users'",
            )
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| err("probe sqlite_master for users table", e))?
            .is_some();
            let users_new_exists = sqlx::query(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'users_new'",
            )
            .fetch_optional(&mut *conn)
            .await
            .map_err(|e| err("probe sqlite_master for users_new table", e))?
            .is_some();

            if users_new_exists && !users_exists {
                // Recover: a previous SqlStore::new dropped `users` but
                // failed before renaming `users_new` (e.g. older code that
                // ran the rebuild across pool connections). Just complete
                // the rename.
                log::info!(
                    "SqlStore: detected half-applied users.email retirement \
                     (users_new present, users absent); completing the rename"
                );
                sqlx::query("ALTER TABLE users_new RENAME TO users")
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| err("recovery rename users_new -> users", e))?;
                log::info!(
                    "SqlStore: completed half-applied users.email retirement \
                     by renaming users_new to users"
                );
                return Ok(());
            }

            let has_email_column = users_exists
                && sqlx::query(
                    "SELECT 1 FROM pragma_table_info('users') WHERE name = 'email'",
                )
                .fetch_optional(&mut *conn)
                .await
                .map_err(|e| err("probe pragma_table_info for users.email column", e))?
                .is_some();
            if !has_email_column {
                return Ok(());
            }

            log::info!(
                "SqlStore: retiring legacy users.email column \
                 (SQLite full table rebuild)"
            );
            // Drop any stale `users_new` left behind by a previous aborted
            // attempt before starting fresh — guarantees the CREATE below
            // doesn't collide.
            if users_new_exists {
                log::info!(
                    "SqlStore: dropping stale users_new left by a previous aborted rebuild"
                );
                sqlx::query("DROP TABLE users_new")
                    .execute(&mut *conn)
                    .await
                    .map_err(|e| err("DROP TABLE users_new (stale)", e))?;
            }

            // Disable FK enforcement for the duration of the rebuild —
            // user_logins / oauth_identities / mazes / user_emails all
            // reference `users(id)` and would error mid-rebuild. SQLite
            // resolves FKs by name so the references survive the rename.
            sqlx::query("PRAGMA foreign_keys = OFF")
                .execute(&mut *conn)
                .await
                .map_err(|e| err("PRAGMA foreign_keys = OFF", e))?;
            sqlx::query(
                "CREATE TABLE users_new (\
                    id              VARCHAR(36)  NOT NULL PRIMARY KEY,\
                    is_admin        INTEGER      NOT NULL DEFAULT 0,\
                    username        VARCHAR(64)  NOT NULL UNIQUE,\
                    full_name       VARCHAR(255) NOT NULL,\
                    password_hash   VARCHAR(255) NOT NULL,\
                    api_key         VARCHAR(36)  NOT NULL UNIQUE,\
                    deleted_at        VARCHAR(32),\
                    created_at        VARCHAR(32),\
                    last_sign_in_at   VARCHAR(32),\
                    avatar_updated_at VARCHAR(32)\
                )",
            )
            .execute(&mut *conn)
            .await
            .map_err(|e| err("CREATE TABLE users_new", e))?;
            sqlx::query(
                "INSERT INTO users_new (id, is_admin, username, full_name, password_hash, api_key, deleted_at, created_at, last_sign_in_at, avatar_updated_at) \
                 SELECT id, is_admin, username, full_name, password_hash, api_key, deleted_at, created_at, last_sign_in_at, avatar_updated_at FROM users",
            )
            .execute(&mut *conn)
            .await
            .map_err(|e| err("INSERT INTO users_new SELECT FROM users", e))?;
            sqlx::query("DROP TABLE users")
                .execute(&mut *conn)
                .await
                .map_err(|e| err("DROP TABLE users (legacy)", e))?;
            sqlx::query("ALTER TABLE users_new RENAME TO users")
                .execute(&mut *conn)
                .await
                .map_err(|e| err("ALTER TABLE users_new RENAME TO users", e))?;
            // The supporting index on the rebuilt `users` table.
            // CREATE INDEX is not idempotent across all backends, but
            // because we just DROPped the old `users` table and renamed
            // `users_new` into place, no `idx_users_deleted_at` exists
            // on the new table — emit it here.
            sqlx::query("CREATE INDEX idx_users_deleted_at ON users(deleted_at)")
                .execute(&mut *conn)
                .await
                .map_err(|e| err("CREATE INDEX idx_users_deleted_at", e))?;
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&mut *conn)
                .await
                .map_err(|e| err("PRAGMA foreign_keys = ON", e))?;
            log::info!(
                "SqlStore: retired legacy users.email column (SQLite table rebuild)"
            );
        }
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Row → struct helpers
// ─────────────────────────────────────────────────────────────────────────────

async fn fetch_user_logins(
    pool: &AnyPool,
    kind: SqlBackend,
    user_id: Uuid,
) -> Result<Vec<UserLogin>, Error> {
    let rows = sqlx::query(&q(
        kind,
        "SELECT id, created_at, expires_at, ip_address, device_info \
         FROM user_logins WHERE user_id = ? ORDER BY created_at",
    ))
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_err)?;

    let mut logins = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.try_get("id").map_err(map_sqlx_err)?;
        let created_at: String = row.try_get("created_at").map_err(map_sqlx_err)?;
        let expires_at: String = row.try_get("expires_at").map_err(map_sqlx_err)?;
        let ip_address: Option<String> = row.try_get("ip_address").map_err(map_sqlx_err)?;
        let device_info: Option<String> = row.try_get("device_info").map_err(map_sqlx_err)?;
        logins.push(UserLogin {
            id: parse_uuid("login id", &id)?,
            created_at: datetime_from_sql(&created_at)?,
            expires_at: datetime_from_sql(&expires_at)?,
            ip_address,
            device_info,
        });
    }
    Ok(logins)
}

async fn fetch_user_emails(
    pool: &AnyPool,
    kind: SqlBackend,
    user_id: Uuid,
) -> Result<Vec<UserEmail>, Error> {
    // Order by primary-first, then alphabetically — keeps the primary at the
    // front of every loaded user, which `User::primary_email()` finds via
    // `iter().find(...)` in O(1) for the common case.
    let rows = sqlx::query(&q(
        kind,
        "SELECT email, is_primary, verified, verified_at \
         FROM user_emails WHERE user_id = ? ORDER BY is_primary DESC, email",
    ))
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_err)?;

    let mut emails = Vec::with_capacity(rows.len());
    for row in rows {
        let is_primary_raw: i32 = row.try_get("is_primary").map_err(map_sqlx_err)?;
        let verified_raw: i32 = row.try_get("verified").map_err(map_sqlx_err)?;
        let verified_at_raw: Option<String> =
            row.try_get("verified_at").map_err(map_sqlx_err)?;
        let verified_at = match verified_at_raw {
            Some(s) => Some(datetime_from_sql(&s)?),
            None => None,
        };
        emails.push(UserEmail {
            email: row.try_get("email").map_err(map_sqlx_err)?,
            is_primary: int_to_bool(is_primary_raw),
            verified: int_to_bool(verified_raw),
            verified_at,
        });
    }
    Ok(emails)
}

async fn fetch_user_oauth_identities(
    pool: &AnyPool,
    kind: SqlBackend,
    user_id: Uuid,
) -> Result<Vec<OAuthIdentity>, Error> {
    let rows = sqlx::query(&q(
        kind,
        "SELECT provider, provider_user_id, provider_email, linked_at, last_seen_at \
         FROM oauth_identities WHERE user_id = ? ORDER BY linked_at",
    ))
    .bind(user_id.to_string())
    .fetch_all(pool)
    .await
    .map_err(map_sqlx_err)?;

    let mut identities = Vec::with_capacity(rows.len());
    for row in rows {
        identities.push(OAuthIdentity {
            provider: row.try_get("provider").map_err(map_sqlx_err)?,
            provider_user_id: row.try_get("provider_user_id").map_err(map_sqlx_err)?,
            provider_email: row.try_get("provider_email").map_err(map_sqlx_err)?,
            linked_at: datetime_from_sql(&row.try_get::<String, _>("linked_at").map_err(map_sqlx_err)?)?,
            last_seen_at: datetime_from_sql(&row.try_get::<String, _>("last_seen_at").map_err(map_sqlx_err)?)?,
        });
    }
    Ok(identities)
}

/// `is_admin` is stored as INTEGER (0/1) — see migration note. Read and write
/// it as i32 (matches INTEGER natively across postgres/mysql/sqlite); SQLx
/// 0.8's Any decoder for postgres happens to auto-widen INT4 to i64 but
/// MySQL's does not, so i64 here would fail row decoding on MySQL.
fn int_to_bool(v: i32) -> bool {
    v != 0
}

fn bool_to_int(v: bool) -> i32 {
    if v {
        1
    } else {
        0
    }
}

async fn user_from_row(pool: &AnyPool, kind: SqlBackend, row: &AnyRow) -> Result<User, Error> {
    let id_str: String = row.try_get("id").map_err(map_sqlx_err)?;
    let id = parse_uuid("user id", &id_str)?;
    let api_key_str: String = row.try_get("api_key").map_err(map_sqlx_err)?;
    let api_key = parse_uuid("api_key", &api_key_str)?;
    let is_admin_raw: i32 = row.try_get("is_admin").map_err(map_sqlx_err)?;
    let deleted_at_str: Option<String> = row.try_get("deleted_at").map_err(map_sqlx_err)?;
    let deleted_at = match deleted_at_str {
        Some(s) => Some(datetime_from_sql(&s)?),
        None => None,
    };
    let created_at_str: String = row.try_get("created_at").map_err(map_sqlx_err)?;
    let created_at = datetime_from_sql(&created_at_str)?;
    let last_sign_in_at_str: Option<String> = row.try_get("last_sign_in_at").map_err(map_sqlx_err)?;
    let last_sign_in_at = match last_sign_in_at_str {
        Some(s) => Some(datetime_from_sql(&s)?),
        None => None,
    };
    let avatar_updated_at_str: Option<String> = row.try_get("avatar_updated_at").map_err(map_sqlx_err)?;
    let avatar_updated_at = match avatar_updated_at_str {
        Some(s) => Some(datetime_from_sql(&s)?),
        None => None,
    };
    Ok(User {
        id,
        is_admin: int_to_bool(is_admin_raw),
        username: row.try_get("username").map_err(map_sqlx_err)?,
        full_name: row.try_get("full_name").map_err(map_sqlx_err)?,
        emails: fetch_user_emails(pool, kind, id).await?,
        password_hash: row.try_get("password_hash").map_err(map_sqlx_err)?,
        api_key,
        logins: fetch_user_logins(pool, kind, id).await?,
        oauth_identities: fetch_user_oauth_identities(pool, kind, id).await?,
        deleted_at,
        created_at,
        last_sign_in_at,
        avatar_updated_at,
    })
}

async fn maze_from_row(row: &AnyRow) -> Result<Maze, Error> {
    let id: String = row.try_get("id").map_err(map_sqlx_err)?;
    let name: String = row.try_get("name").map_err(map_sqlx_err)?;
    let definition_json: String = row.try_get("definition").map_err(map_sqlx_err)?;
    let mut maze: Maze = serde_json::from_str(&definition_json)?;
    maze.id = id;
    maze.name = name;
    Ok(maze)
}

/// Deserialises a `game_definitions` row into a [`GameDefinition`]. `seed` comes
/// back as `i64` (BIGINT) and widens to the struct's `u64` via the bit pattern;
/// the enum columns parse leniently; timestamps go through [`datetime_from_sql`].
fn game_definition_from_row(row: &AnyRow) -> Result<GameDefinition, Error> {
    let id = parse_uuid("game definition id", &row.try_get::<String, _>("id").map_err(map_sqlx_err)?)?;
    let owner_id = parse_uuid(
        "game definition owner_id",
        &row.try_get::<String, _>("owner_id").map_err(map_sqlx_err)?,
    )?;
    let name: String = row.try_get("name").map_err(map_sqlx_err)?;
    let description: Option<String> = row.try_get("description").map_err(map_sqlx_err)?;
    let image_updated_at = match row
        .try_get::<Option<String>, _>("image_updated_at")
        .map_err(map_sqlx_err)?
    {
        Some(s) => Some(datetime_from_sql(&s)?),
        None => None,
    };
    let visibility = Visibility::from_wire_str(&row.try_get::<String, _>("visibility").map_err(map_sqlx_err)?);
    let seed: i64 = row.try_get("seed").map_err(map_sqlx_err)?;
    let rotation = Rotation::from_wire_str(&row.try_get::<String, _>("rotation").map_err(map_sqlx_err)?);
    let config: serde_json::Value =
        serde_json::from_str(&row.try_get::<String, _>("config").map_err(map_sqlx_err)?)?;
    let created_at = datetime_from_sql(&row.try_get::<String, _>("created_at").map_err(map_sqlx_err)?)?;
    let updated_at = datetime_from_sql(&row.try_get::<String, _>("updated_at").map_err(map_sqlx_err)?)?;
    Ok(GameDefinition {
        id,
        owner_id,
        name,
        description,
        image_updated_at,
        visibility,
        seed: seed as u64,
        rotation,
        config,
        created_at,
        updated_at,
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// User-row helpers (write)
// ─────────────────────────────────────────────────────────────────────────────

async fn check_user_unique_fields(
    pool: &AnyPool,
    kind: SqlBackend,
    username: &str,
    emails: &[UserEmail],
    ignore_id: Uuid,
) -> Result<(), Error> {
    let ignore = ignore_id.to_string();
    let by_name = sqlx::query(&q(
        kind,
        "SELECT id FROM users WHERE LOWER(username) = LOWER(?) AND id <> ?",
    ))
    .bind(username)
    .bind(&ignore)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_err)?;
    if by_name.is_some() {
        return Err(Error::UserNameExists());
    }
    // Check every email row against the global UNIQUE on user_emails.email.
    for row in emails {
        let by_email = sqlx::query(&q(
            kind,
            "SELECT user_id FROM user_emails \
             WHERE LOWER(email) = LOWER(?) AND user_id <> ?",
        ))
        .bind(&row.email)
        .bind(&ignore)
        .fetch_optional(pool)
        .await
        .map_err(map_sqlx_err)?;
        if by_email.is_some() {
            return Err(Error::UserEmailExists());
        }
    }
    Ok(())
}

fn validate_user_for_store(user: &User) -> Result<(), Error> {
    validate_user_fields(user)?;
    // OAuth-only users carry an empty password_hash. Only require a hash when
    // no OAuth identity is attached. Mirrors FileStore validation.
    if user.password_hash.is_empty() && user.oauth_identities.is_empty() {
        return Err(Error::UserPasswordMissing());
    }
    Ok(())
}

async fn insert_user_emails(
    pool: &AnyPool,
    kind: SqlBackend,
    user_id: Uuid,
    emails: &[UserEmail],
) -> Result<(), Error> {
    for row in emails {
        sqlx::query(&q(
            kind,
            "INSERT INTO user_emails (user_id, email, is_primary, verified, verified_at) \
             VALUES (?, ?, ?, ?, ?)",
        ))
        .bind(user_id.to_string())
        .bind(&row.email)
        .bind(bool_to_int(row.is_primary))
        .bind(bool_to_int(row.verified))
        .bind(row.verified_at.map(datetime_to_sql))
        .execute(pool)
        .await
        .map_err(map_sqlx_err)?;
    }
    Ok(())
}

async fn insert_user_logins(
    pool: &AnyPool,
    kind: SqlBackend,
    user_id: Uuid,
    logins: &[UserLogin],
) -> Result<(), Error> {
    for login in logins {
        sqlx::query(&q(
            kind,
            "INSERT INTO user_logins (id, user_id, created_at, expires_at, ip_address, device_info) \
             VALUES (?, ?, ?, ?, ?, ?)",
        ))
        .bind(login.id.to_string())
        .bind(user_id.to_string())
        .bind(datetime_to_sql(login.created_at))
        .bind(datetime_to_sql(login.expires_at))
        .bind(login.ip_address.clone())
        .bind(login.device_info.clone())
        .execute(pool)
        .await
        .map_err(map_sqlx_err)?;
    }
    Ok(())
}

async fn insert_user_oauth_identities(
    pool: &AnyPool,
    kind: SqlBackend,
    user_id: Uuid,
    identities: &[OAuthIdentity],
) -> Result<(), Error> {
    for identity in identities {
        sqlx::query(&q(
            kind,
            "INSERT INTO oauth_identities \
                 (user_id, provider, provider_user_id, provider_email, linked_at, last_seen_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        ))
        .bind(user_id.to_string())
        .bind(&identity.provider)
        .bind(&identity.provider_user_id)
        .bind(identity.provider_email.clone())
        .bind(datetime_to_sql(identity.linked_at))
        .bind(datetime_to_sql(identity.last_seen_at))
        .execute(pool)
        .await
        .map_err(map_sqlx_err)?;
    }
    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// UserStore
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl UserStore for SqlStore {
    /// Adds the default admin user to the store if it doesn't already exist, else returns it
    ///
    /// # Examples
    ///
    /// Try to create a new user within an in-memory SQLite-backed store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the default admin user within the SQL store if needed
    /// match store.init_default_admin_user("admin", "admin@maze.local", "my_password_hash").await {
    ///     Ok(user) => {
    ///         println!(
    ///             "Successfully initialised default admin user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to initialise default admin user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn init_default_admin_user(
        &mut self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, Error> {
        match self.find_user_by_name(username).await {
            Ok(user) => Ok(user),
            Err(Error::UserNotFound()) => {
                let mut user = User::default();
                user.username = username.to_string();
                user.set_primary_email_address(email);
                user.is_admin = true;
                user.password_hash = password_hash.to_string();
                self.create_user(&mut user).await?;
                Ok(user)
            }
            Err(error) => Err(error),
        }
    }

    /// Adds a new user to the store and sets the allocated `id` within the user object
    ///
    /// # Examples
    ///
    /// Try to create a new user within an in-memory SQLite-backed store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn create_user(&mut self, user: &mut User) -> Result<(), Error> {
        user.id = User::new_id();
        user.api_key = User::new_api_key();
        validate_user_for_store(user)?;
        check_user_unique_fields(&self.pool, self.kind, &user.username, &user.emails, Uuid::nil()).await?;

        sqlx::query(&q(
            self.kind,
            "INSERT INTO users (id, is_admin, username, full_name, password_hash, api_key, created_at, last_sign_in_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        ))
        .bind(user.id.to_string())
        .bind(bool_to_int(user.is_admin))
        .bind(&user.username)
        .bind(&user.full_name)
        .bind(&user.password_hash)
        .bind(user.api_key.to_string())
        .bind(datetime_to_sql(user.created_at))
        .bind(user.last_sign_in_at.map(datetime_to_sql))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        insert_user_emails(&self.pool, self.kind, user.id, &user.emails).await?;
        insert_user_logins(&self.pool, self.kind, user.id, &user.logins).await?;
        insert_user_oauth_identities(&self.pool, self.kind, user.id, &user.oauth_identities).await?;
        Ok(())
    }

    /// Deletes a user from the store
    ///
    /// # Examples
    ///
    /// Try to create and then delete a user within an in-memory SQLite-backed store
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         match store.delete_user(user.id).await {
    ///             Ok(_) => {
    ///                 println!("Successfully deleted user from the SQL store");
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to delete user => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn delete_user(&mut self, id: Uuid) -> Result<(), Error> {
        if id.is_nil() {
            return Err(Error::UserIdMissing());
        }
        let now = datetime_to_sql(Utc::now());
        let scrambled = format!("deleted-{id}");
        let result = sqlx::query(&q(
            self.kind,
            "UPDATE users SET deleted_at = ?, username = ? \
             WHERE id = ? AND deleted_at IS NULL",
        ))
        .bind(&now)
        .bind(&scrambled)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(id.to_string()));
        }
        // Hard-delete cascaded data that has no audit value: pending sessions,
        // OAuth identities (frees `(provider, provider_user_id)` for reuse),
        // email rows (frees the address for reuse), and the user's mazes.
        // The audit log row, when added, intentionally survives.
        sqlx::query(&q(self.kind, "DELETE FROM user_logins WHERE user_id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        sqlx::query(&q(self.kind, "DELETE FROM oauth_identities WHERE user_id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        sqlx::query(&q(self.kind, "DELETE FROM user_emails WHERE user_id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        sqlx::query(&q(self.kind, "DELETE FROM one_time_tokens WHERE user_id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        // The user's own score history, plus the boards of the mazes about to be
        // deleted below (other players' runs on those mazes). Runs before the
        // `mazes` delete so the subquery still sees them. FK cascade is a
        // backstop; we delete explicitly so the behaviour is uniform across
        // backends (SQLite FK enforcement is pragma-gated).
        sqlx::query(&q(
            self.kind,
            "DELETE FROM score_history \
             WHERE user_id = ? OR maze_id IN (SELECT id FROM mazes WHERE owner_id = ?)",
        ))
        .bind(id.to_string())
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        sqlx::query(&q(self.kind, "DELETE FROM mazes WHERE owner_id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        // Clear the boards of the user's game definitions (other players' runs
        // on them), keyed by the `def:<id>` challenge subject. The challenge is
        // a string, not a FK column, so it can't be swept with a subquery —
        // gather the owned ids and prefix-clear each.
        let owned_def_rows =
            sqlx::query(&q(self.kind, "SELECT id FROM game_definitions WHERE owner_id = ?"))
                .bind(id.to_string())
                .fetch_all(&self.pool)
                .await
                .map_err(map_sqlx_err)?;
        for row in &owned_def_rows {
            let def_id: String = row.try_get("id").map_err(map_sqlx_err)?;
            self.clear_challenge_scores_prefix(&format!("def:{def_id}")).await?;
        }
        // Strip the user from every remaining definition's grantee list, then
        // delete the user's own definitions + their shares. FK cascades are a
        // backstop; we delete explicitly for uniform cross-backend behaviour.
        sqlx::query(&q(
            self.kind,
            "DELETE FROM game_definition_shares WHERE grantee_user_id = ?",
        ))
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        sqlx::query(&q(
            self.kind,
            "DELETE FROM game_definition_shares \
             WHERE definition_id IN (SELECT id FROM game_definitions WHERE owner_id = ?)",
        ))
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        sqlx::query(&q(self.kind, "DELETE FROM game_definitions WHERE owner_id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(())
    }

    /// True hard-delete: removes the `users` row outright. Intended for
    /// retention / right-to-erasure flows where the soft-deleted row must
    /// also be cleared. `ON DELETE CASCADE` on `user_logins`,
    /// `oauth_identities`, `user_emails`, and `mazes` clears every
    /// related row in the same transaction. Reachable on either an active
    /// or already-soft-deleted user.
    ///
    /// # Examples
    ///
    /// Soft-delete a user, then purge them so the row is truly gone
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// store.create_user(&mut user).await.expect("create_user");
    /// store.delete_user(user.id).await.expect("soft-delete");
    /// match store.purge_user(user.id).await {
    ///     Ok(_) => println!("User purged from the SQL store"),
    ///     Err(error) => println!("Failed to purge user => {}", error),
    /// }
    /// # });
    /// ```
    async fn purge_user(&mut self, id: Uuid) -> Result<(), Error> {
        if id.is_nil() {
            return Err(Error::UserIdMissing());
        }
        // True hard-delete. ON DELETE CASCADE on user_logins, oauth_identities,
        // user_emails, mazes, and score_history clears every related row.
        // Reachable in two
        // legitimate cases: (1) the row is already soft-deleted and operations
        // is purging it; (2) right-to-erasure called directly on an active
        // user. The trait does not require a prior soft-delete, so the
        // `deleted_at IS NULL` filter is intentionally omitted here.
        let result = sqlx::query(&q(self.kind, "DELETE FROM users WHERE id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Updates a user within the store
    ///
    /// # Examples
    ///
    /// Try to create and then update a user within an in-memory SQLite-backed store
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         // Change the user full name
    ///         user.full_name = "John Henry Smith".to_string();
    ///         match store.update_user(&mut user).await {
    ///             Ok(_) => {
    ///                 println!("Successfully updated user within the SQL store");
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to update user => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn update_user(&mut self, user: &mut User) -> Result<(), Error> {
        if user.id == Uuid::nil() {
            return Err(Error::UserIdMissing());
        }
        validate_user_for_store(user)?;
        check_user_unique_fields(&self.pool, self.kind, &user.username, &user.emails, user.id).await?;

        let result = sqlx::query(&q(
            self.kind,
            "UPDATE users SET is_admin = ?, username = ?, full_name = ?, \
                              password_hash = ?, api_key = ?, last_sign_in_at = ? \
             WHERE id = ?",
        ))
        .bind(bool_to_int(user.is_admin))
        .bind(&user.username)
        .bind(&user.full_name)
        .bind(&user.password_hash)
        .bind(user.api_key.to_string())
        .bind(user.last_sign_in_at.map(datetime_to_sql))
        .bind(user.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(user.id.to_string()));
        }

        // Replace child collections wholesale — matches the load-modify-save
        // semantics callers use against the trait. Far simpler than diffing.
        sqlx::query(&q(self.kind, "DELETE FROM user_emails WHERE user_id = ?"))
            .bind(user.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        sqlx::query(&q(self.kind, "DELETE FROM user_logins WHERE user_id = ?"))
            .bind(user.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        sqlx::query(&q(self.kind, "DELETE FROM oauth_identities WHERE user_id = ?"))
            .bind(user.id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;

        insert_user_emails(&self.pool, self.kind, user.id, &user.emails).await?;
        insert_user_logins(&self.pool, self.kind, user.id, &user.logins).await?;
        insert_user_oauth_identities(&self.pool, self.kind, user.id, &user.oauth_identities).await?;
        Ok(())
    }

    /// Loads a user from the store
    ///
    /// # Examples
    ///
    /// Try to create and then load a user from within an in-memory SQLite-backed store
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         // Now attempt to load it again and display the results
    ///         match store.get_user(user.id).await {
    ///             Ok(user_loaded) => {
    ///                 println!("Successfully loaded user from within the SQL store => {:?}", user_loaded);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to load user => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_user(&self, id: Uuid) -> Result<User, Error> {
        let row = sqlx::query(&q(
            self.kind,
            "SELECT * FROM users WHERE id = ? AND deleted_at IS NULL",
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match row {
            Some(row) => user_from_row(&self.pool, self.kind, &row).await,
            None => Err(Error::UserIdNotFound(id.to_string())),
        }
    }

    /// Locates a user by their username within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user from within an in-memory SQLite-backed store
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by username and display the results
    ///         match store.find_user_by_name(&user.username).await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the SQL store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_name(&self, name: &str) -> Result<User, Error> {
        let mut rows = sqlx::query(&q(
            self.kind,
            "SELECT * FROM users WHERE LOWER(username) = LOWER(?) AND deleted_at IS NULL",
        ))
        .bind(name)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match rows.len() {
            0 => Err(Error::UserNotFound()),
            1 => user_from_row(&self.pool, self.kind, &rows.pop().expect("len==1")).await,
            n => Err(integrity_violation(&format!(
                "{n} users match username '{name}' case-insensitively"
            ))),
        }
    }

    /// Locates a user by an email address within the store, returning the
    /// match only if the matching `user_emails` row is `verified = true`.
    /// Unverified rows are invisible to this lookup. See the trait
    /// doc-comment for the security rationale.
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user from within an in-memory SQLite-backed store by email
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by email and display the results
    ///         match store.find_user_by_verified_email(user.email()).await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the SQL store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_verified_email(&self, email: &str) -> Result<User, Error> {
        let mut rows = sqlx::query(&q(
            self.kind,
            "SELECT u.* FROM users u \
             JOIN user_emails ue ON ue.user_id = u.id \
             WHERE LOWER(ue.email) = LOWER(?) AND ue.verified <> 0 \
               AND u.deleted_at IS NULL",
        ))
        .bind(email)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match rows.len() {
            0 => Err(Error::UserNotFound()),
            1 => user_from_row(&self.pool, self.kind, &rows.pop().expect("len==1")).await,
            n => Err(integrity_violation(&format!(
                "{n} users match verified email '{email}' case-insensitively"
            ))),
        }
    }

    /// Locates a user by an email address regardless of verification state.
    /// Same SQL shape as [`Self::find_user_by_verified_email`] minus the
    /// `ue.verified <> 0` filter. The `user_emails.email` UNIQUE constraint
    /// guarantees at most one match in healthy state; the multi-row guard
    /// is here for parity and to fail loudly if a future migration ever
    /// weakens that constraint.
    ///
    /// See the trait doc-comment for usage rules — auth code must use
    /// [`Self::find_user_by_verified_email`] instead.
    async fn find_user_by_email_any_state(&self, email: &str) -> Result<User, Error> {
        let mut rows = sqlx::query(&q(
            self.kind,
            "SELECT u.* FROM users u \
             JOIN user_emails ue ON ue.user_id = u.id \
             WHERE LOWER(ue.email) = LOWER(?) \
               AND u.deleted_at IS NULL",
        ))
        .bind(email)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match rows.len() {
            0 => Err(Error::UserNotFound()),
            1 => user_from_row(&self.pool, self.kind, &rows.pop().expect("len==1")).await,
            n => Err(integrity_violation(&format!(
                "{n} users match email '{email}' case-insensitively"
            ))),
        }
    }

    /// Locates a user by their api key within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user by its api key from within an in-memory SQLite-backed store
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by api key and display the results
    ///         match store.find_user_by_api_key(user.api_key).await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the SQL store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, Error> {
        // `users.api_key` is enforced UNIQUE at the schema level so this can
        // return at most one row by construction. The multi-row guard is here
        // for parity with the rest of the `find_user_by_*` family and to fail
        // loudly if a future migration ever weakens the unique index.
        let mut rows = sqlx::query(&q(
            self.kind,
            "SELECT * FROM users WHERE api_key = ? AND deleted_at IS NULL",
        ))
        .bind(api_key.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match rows.len() {
            0 => Err(Error::UserNotFound()),
            1 => user_from_row(&self.pool, self.kind, &rows.pop().expect("len==1")).await,
            n => Err(integrity_violation(&format!(
                "{n} users match api_key {api_key}"
            ))),
        }
    }

    /// Locates a user by their login id within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user by its login id within an in-memory SQLite-backed store
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail, UserLogin};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the login token
    /// let login = UserLogin::new(24, Some("123.456.789.012".to_string()), Some("Device info string".to_string()));
    /// let search_login_id = login.id;
    /// let logins = vec![login];
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins,
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again using the login id and display the results
    ///         match store.find_user_by_login_id(search_login_id).await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the SQL store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_login_id(&self, login_id: Uuid) -> Result<User, Error> {
        // Strictly speaking, `user_logins.id` is the table's PRIMARY KEY so this
        // can return at most one row by construction. We still use the
        // multi-row guard for parity with the other find_user_by_* methods —
        // a future migration that drops the PK or a direct DB edit would
        // otherwise silently pick a row.
        let now = datetime_to_sql(Utc::now());
        let mut rows = sqlx::query(&q(
            self.kind,
            "SELECT u.* FROM users u \
             JOIN user_logins l ON l.user_id = u.id \
             WHERE l.id = ? AND l.expires_at > ? AND u.deleted_at IS NULL",
        ))
        .bind(login_id.to_string())
        .bind(now)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match rows.len() {
            0 => Err(Error::UserNotFound()),
            1 => user_from_row(&self.pool, self.kind, &rows.pop().expect("len==1")).await,
            n => Err(integrity_violation(&format!(
                "{n} users match login_id {login_id}"
            ))),
        }
    }

    /// Locates a user by an OAuth identity `(provider, provider_user_id)` pair.
    /// `provider` is matched case-insensitively (canonical providers are stored
    /// lowercase: "google", "github"); `provider_user_id` is matched exactly (it
    /// is an opaque stable id from the identity provider).
    ///
    /// # Examples
    ///
    /// Try to create a user with a linked Google identity and then locate it by
    /// its OAuth identity within an in-memory SQLite-backed store
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{OAuthIdentity, User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition with a linked Google identity
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![OAuthIdentity::new(
    ///         "google".to_string(),
    ///         "google-sub-jsmith".to_string(),
    ///         Some("jsmith@company.com".to_string()),
    ///     )],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by its OAuth identity and display the results
    ///         match store.find_user_by_oauth_identity("google", "google-sub-jsmith").await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the SQL store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_oauth_identity(
        &self,
        provider: &str,
        provider_user_id: &str,
    ) -> Result<User, Error> {
        let mut rows = sqlx::query(&q(
            self.kind,
            "SELECT u.* FROM users u \
             JOIN oauth_identities oi ON oi.user_id = u.id \
             WHERE LOWER(oi.provider) = LOWER(?) AND oi.provider_user_id = ? \
               AND u.deleted_at IS NULL",
        ))
        .bind(provider)
        .bind(provider_user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match rows.len() {
            0 => Err(Error::UserNotFound()),
            1 => user_from_row(&self.pool, self.kind, &rows.pop().expect("len==1")).await,
            n => Err(integrity_violation(&format!(
                "{n} users match oauth identity ({provider}, {provider_user_id})"
            ))),
        }
    }

    /// Returns the list of users within the store, sorted
    /// alphabetically by username in ascending order
    ///
    /// # Examples
    ///
    /// Try to create a user within an in-memory SQLite-backed store and then load the list of registered users and display their count
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         // Now attempt to load the user list and display the results
    ///         match store.get_users().await {
    ///             Ok(users_found) => {
    ///                 println!("Successfully loaded {} users from within the SQL store", users_found.len());
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to load users => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_users(&self) -> Result<Vec<User>, Error> {
        let rows = sqlx::query(&q(
            self.kind,
            "SELECT * FROM users WHERE deleted_at IS NULL ORDER BY username",
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        let mut users = Vec::with_capacity(rows.len());
        for row in &rows {
            users.push(user_from_row(&self.pool, self.kind, row).await?);
        }
        Ok(users)
    }

    /// Returns the list of admin users within the store
    ///
    /// # Examples
    ///
    /// Try to create an admin user within an in-memory SQLite-backed store and then load the list of admin users and display their count
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Create the admin user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: true,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the admin user within the SQL store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created admin user with id {} in the SQL store",
    ///             user.id
    ///         );
    ///         // Now attempt to load the admin user list and display the results
    ///         match store.get_admin_users().await {
    ///             Ok(admins_found) => {
    ///                 println!("Successfully loaded {} admin users from within the SQL store", admins_found.len());
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to load admin users => {}",
    ///                     error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_admin_users(&self) -> Result<Vec<User>, Error> {
        let rows = sqlx::query(&q(
            self.kind,
            "SELECT * FROM users WHERE is_admin <> 0 AND deleted_at IS NULL \
             ORDER BY username",
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        let mut users = Vec::with_capacity(rows.len());
        for row in &rows {
            users.push(user_from_row(&self.pool, self.kind, row).await?);
        }
        Ok(users)
    }

    /// Returns whether at least one user exists in the SQL store.
    ///
    /// Implemented as a `SELECT 1 FROM users LIMIT 1` existence probe so the
    /// engine can return on the first row it sees (index-only on the PK in
    /// practice). Far cheaper than `get_users()` which would hydrate every
    /// user plus their logins and oauth_identities.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if any user is present, `Ok(false)` if the store is empty.
    ///
    /// # Examples
    ///
    /// Check whether the store has any users before deciding to seed a
    /// default admin account
    /// ```
    /// # tokio_test::block_on(async {
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// match store.has_users().await {
    ///     Ok(true) => println!("Store already has users — skip bootstrap"),
    ///     Ok(false) => println!("Store is empty — seed a default admin"),
    ///     Err(error) => println!("Failed to check store: {}", error),
    /// }
    /// # });
    /// ```
    async fn has_users(&self) -> Result<bool, Error> {
        let row = sqlx::query(&q(
            self.kind,
            "SELECT 1 FROM users WHERE deleted_at IS NULL LIMIT 1",
        ))
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(row.is_some())
    }

    /// Returns whether at least one *active* admin user exists in the SQL
    /// store (`is_admin <> 0` AND `deleted_at IS NULL`).
    ///
    /// Implemented as a `SELECT 1 ... LIMIT 1` existence probe so the
    /// engine can return on the first matching row (covered by the
    /// `idx_users_deleted_at` index). Used by startup so a soft-deleted
    /// lone admin doesn't prevent the default admin from being recreated
    /// on next launch.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if at least one active admin user exists, `Ok(false)`
    /// otherwise (no users, no admins, or every admin has been soft-deleted).
    ///
    /// # Examples
    ///
    /// Probe the store before deciding whether to seed a default admin
    /// ```
    /// # tokio_test::block_on(async {
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// match store.has_active_admin_user().await {
    ///     Ok(true) => println!("Active admin already present — skip bootstrap"),
    ///     Ok(false) => println!("No active admin — seed a default admin"),
    ///     Err(error) => println!("Failed to check store: {}", error),
    /// }
    /// # });
    /// ```
    async fn has_active_admin_user(&self) -> Result<bool, Error> {
        let row = sqlx::query(&q(
            self.kind,
            "SELECT 1 FROM users WHERE is_admin <> 0 AND deleted_at IS NULL LIMIT 1",
        ))
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(row.is_some())
    }

    /// Adds a non-primary email row to the user. See the `UserStore`
    /// trait doc-comment for the full contract; pass `verified = true`
    /// for trusted sources (OAuth-link, admin seed) and `verified = false`
    /// for self-asserted user-typed emails.
    ///
    /// # Examples
    ///
    /// Add a secondary unverified email to an existing user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// let row = store
    ///     .add_user_email(user.id, "alice2@example.com", false)
    ///     .await
    ///     .expect("add secondary");
    /// assert!(!row.verified);
    /// # });
    /// ```
    async fn add_user_email(
        &mut self,
        user_id: Uuid,
        email: &str,
        verified: bool,
    ) -> Result<UserEmail, Error> {
        // Confirm the user exists; surfaces a clean UserIdNotFound if not.
        let _ = self.get_user(user_id).await?;
        validate_email_format(email)?;
        // Reject if any user already owns this address.
        let conflict = sqlx::query(&q(
            self.kind,
            "SELECT 1 FROM user_emails WHERE LOWER(email) = LOWER(?)",
        ))
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if conflict.is_some() {
            return Err(Error::UserEmailExists());
        }
        let verified_at = if verified {
            Some(canonical_now_millis())
        } else {
            None
        };
        sqlx::query(&q(
            self.kind,
            "INSERT INTO user_emails (user_id, email, is_primary, verified, verified_at) \
             VALUES (?, ?, ?, ?, ?)",
        ))
        .bind(user_id.to_string())
        .bind(email)
        .bind(bool_to_int(false)) // never primary on add
        .bind(bool_to_int(verified))
        .bind(verified_at.map(datetime_to_sql))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(UserEmail {
            email: email.to_string(),
            is_primary: false,
            verified,
            verified_at,
        })
    }

    /// Removes a non-primary, non-last email row from the user. See the
    /// trait doc-comment for the rejection rules.
    ///
    /// # Examples
    ///
    /// Add a secondary email then remove it
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// store.add_user_email(user.id, "alice2@example.com", true).await.expect("add");
    /// store.remove_user_email(user.id, "alice2@example.com").await.expect("remove");
    /// # });
    /// ```
    async fn remove_user_email(
        &mut self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), Error> {
        let row = fetch_user_email_row(&self.pool, self.kind, user_id, email).await?;
        let is_primary: i32 = row.try_get("is_primary").map_err(map_sqlx_err)?;

        // Count rows so we can refuse to remove the user's only email.
        let total: i64 = sqlx::query(&q(
            self.kind,
            "SELECT COUNT(*) AS c FROM user_emails WHERE user_id = ?",
        ))
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(map_sqlx_err)?
        .try_get("c")
        .map_err(map_sqlx_err)?;
        if total <= 1 {
            return Err(Error::UserEmailIsLast());
        }
        if int_to_bool(is_primary) {
            return Err(Error::UserEmailIsPrimary());
        }

        sqlx::query(&q(
            self.kind,
            "DELETE FROM user_emails \
             WHERE user_id = ? AND LOWER(email) = LOWER(?)",
        ))
        .bind(user_id.to_string())
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        // Drop any OAuth identity rows whose `provider_email` matches the
        // removed address. See the trait doc for the invariant this
        // upholds — otherwise an OAuth provider could still authenticate
        // the user via branch 1 of `account::resolve` (which matches by
        // `(provider, provider_user_id)`, not by current email).
        sqlx::query(&q(
            self.kind,
            "DELETE FROM oauth_identities \
             WHERE user_id = ? AND LOWER(provider_email) = LOWER(?)",
        ))
        .bind(user_id.to_string())
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    /// Promotes the named email row to primary. The target must already
    /// be `verified = true`; promoting an unverified row is rejected to
    /// stop a session-hijacker from redirecting password resets to an
    /// attacker-controlled mailbox.
    ///
    /// # Examples
    ///
    /// Promote a verified secondary to primary
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// store.add_user_email(user.id, "alice2@example.com", true).await.expect("add");
    /// store.set_primary_email(user.id, "alice2@example.com").await.expect("promote");
    /// # });
    /// ```
    async fn set_primary_email(
        &mut self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), Error> {
        let row = fetch_user_email_row(&self.pool, self.kind, user_id, email).await?;
        let verified: i32 = row.try_get("verified").map_err(map_sqlx_err)?;
        if !int_to_bool(verified) {
            return Err(Error::UserEmailNotVerified());
        }

        // Atomically clear is_primary on every row of the user, then set it
        // on the target. A transaction ensures the "exactly one primary"
        // invariant isn't observable as broken mid-flight.
        let mut tx = self.pool.begin().await.map_err(map_sqlx_err)?;
        sqlx::query(&q(
            self.kind,
            "UPDATE user_emails SET is_primary = 0 WHERE user_id = ?",
        ))
        .bind(user_id.to_string())
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;
        sqlx::query(&q(
            self.kind,
            "UPDATE user_emails SET is_primary = 1 \
             WHERE user_id = ? AND LOWER(email) = LOWER(?)",
        ))
        .bind(user_id.to_string())
        .bind(email)
        .execute(&mut *tx)
        .await
        .map_err(map_sqlx_err)?;
        tx.commit().await.map_err(map_sqlx_err)?;
        Ok(())
    }

    /// Marks the named email row verified, refreshing `verified_at` to
    /// the current time. Idempotent — re-marking an already-verified row
    /// just updates the timestamp.
    ///
    /// # Examples
    ///
    /// Add an unverified secondary and then verify it
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// store.add_user_email(user.id, "alice2@example.com", false).await.expect("add");
    /// store.mark_email_verified(user.id, "alice2@example.com").await.expect("mark verified");
    /// # });
    /// ```
    async fn mark_email_verified(
        &mut self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), Error> {
        let result = sqlx::query(&q(
            self.kind,
            "UPDATE user_emails SET verified = 1, verified_at = ? \
             WHERE user_id = ? AND LOWER(email) = LOWER(?)",
        ))
        .bind(datetime_to_sql(canonical_now_millis()))
        .bind(user_id.to_string())
        .bind(email)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(Error::UserEmailNotFound(email.to_string()));
        }
        Ok(())
    }
    /// Stores (or replaces) the user's avatar PNG in `user_avatars` and stamps
    /// `users.avatar_updated_at`. The bytes are bound as a blob and stored
    /// verbatim (the caller has canonicalised them to a PNG). Rejects with
    /// [`Error::UserIdNotFound`] when no active user has the given id.
    ///
    /// # Examples
    ///
    /// Create a user, set an avatar, and read it back
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    ///
    /// store.set_user_avatar(user.id, vec![0x89, 0x50, 0x4E, 0x47]).await.unwrap();
    /// assert_eq!(
    ///     store.get_user_avatar(user.id).await.unwrap(),
    ///     Some(vec![0x89, 0x50, 0x4E, 0x47])
    /// );
    /// assert!(store.get_user(user.id).await.unwrap().avatar_updated_at.is_some());
    /// # });
    /// ```
    async fn set_user_avatar(&mut self, id: Uuid, png_bytes: Vec<u8>) -> Result<(), Error> {
        // Stamp the marker first. The `deleted_at IS NULL` filter doubles as
        // the existence/active check — zero rows affected means no such active
        // user, so we stop before touching `user_avatars` (whose FK would
        // reject the orphan anyway).
        let result = sqlx::query(&q(
            self.kind,
            "UPDATE users SET avatar_updated_at = ? WHERE id = ? AND deleted_at IS NULL",
        ))
        .bind(datetime_to_sql(canonical_now_millis()))
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(Error::UserIdNotFound(id.to_string()));
        }
        // Replace any existing avatar wholesale (DELETE + INSERT) — portable
        // across all three backends without per-backend upsert syntax, and the
        // same load-modify-save shape `update_user` uses for child rows.
        sqlx::query(&q(self.kind, "DELETE FROM user_avatars WHERE user_id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        sqlx::query(&q(
            self.kind,
            "INSERT INTO user_avatars (user_id, image_data) VALUES (?, ?)",
        ))
        .bind(id.to_string())
        .bind(png_bytes)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }
    /// Loads the user's avatar bytes, or `None` when the user has no avatar
    /// row (never set, since cleared, or no such user).
    ///
    /// # Examples
    ///
    /// A freshly-created user has no avatar
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    ///
    /// assert_eq!(store.get_user_avatar(user.id).await.unwrap(), None);
    /// # });
    /// ```
    async fn get_user_avatar(&self, id: Uuid) -> Result<Option<Vec<u8>>, Error> {
        let row = sqlx::query(&q(
            self.kind,
            "SELECT image_data FROM user_avatars WHERE user_id = ?",
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match row {
            Some(row) => Ok(Some(
                row.try_get::<Vec<u8>, _>("image_data").map_err(map_sqlx_err)?,
            )),
            None => Ok(None),
        }
    }
    /// Removes the user's avatar row if present and clears
    /// `users.avatar_updated_at`. Idempotent — clearing a user with no avatar
    /// (or no such user) is a successful no-op.
    ///
    /// # Examples
    ///
    /// Setting then clearing leaves no avatar behind
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    /// store.set_user_avatar(user.id, vec![1, 2, 3]).await.unwrap();
    ///
    /// store.clear_user_avatar(user.id).await.unwrap();
    /// assert_eq!(store.get_user_avatar(user.id).await.unwrap(), None);
    /// assert!(store.get_user(user.id).await.unwrap().avatar_updated_at.is_none());
    /// # });
    /// ```
    async fn clear_user_avatar(&mut self, id: Uuid) -> Result<(), Error> {
        // Both statements are harmless no-ops when there's no avatar / no such
        // user, so clearing is always Ok (idempotent).
        sqlx::query(&q(self.kind, "DELETE FROM user_avatars WHERE user_id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        sqlx::query(&q(
            self.kind,
            "UPDATE users SET avatar_updated_at = NULL WHERE id = ?",
        ))
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }
}

/// Returns the current UTC time truncated to millisecond precision so it
/// round-trips losslessly through the `VARCHAR(32)` RFC 3339 columns.
/// Mirrors `UserEmail::new_primary_verified` in `data_model`.
fn canonical_now_millis() -> DateTime<Utc> {
    use chrono::SubsecRound;
    Utc::now().trunc_subsecs(3)
}

/// Fetches a single `user_emails` row identified by `(user_id, email)` —
/// the email match is case-insensitive — and returns
/// `Error::UserEmailNotFound` if no row exists. Centralises the lookup
/// that every email-mutating `UserStore` method runs to validate the
/// target row before deciding the action.
async fn fetch_user_email_row(
    pool: &AnyPool,
    kind: SqlBackend,
    user_id: Uuid,
    email: &str,
) -> Result<AnyRow, Error> {
    sqlx::query(&q(
        kind,
        "SELECT email, is_primary, verified, verified_at FROM user_emails \
         WHERE user_id = ? AND LOWER(email) = LOWER(?)",
    ))
    .bind(user_id.to_string())
    .bind(email)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_err)?
    .ok_or_else(|| Error::UserEmailNotFound(email.to_string()))
}

// ─────────────────────────────────────────────────────────────────────────────
// MazeStore
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl MazeStore for SqlStore {
    /// Returns the cell-count ceiling enforced by this SQL store on
    /// create/update — see [`crate::MAX_MAZE_CELLS`].
    ///
    /// # Examples
    ///
    /// Read the cap from a fresh in-memory SQLite store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use storage::{SqlStore, SqlStoreConfig, MazeStore};
    ///
    /// let store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// assert_eq!(store.max_maze_cells(), Some(3_600));
    /// # });
    /// ```
    fn max_maze_cells(&self) -> Option<usize> {
        Some(MAX_MAZE_CELLS)
    }
    /// Creates a new maze within the SQL store instance
    ///
    /// # Examples
    ///
    /// Try to create a new maze within an in-memory SQLite-backed store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{Maze, User};
    /// use storage::{SqlStore, SqlStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_create = Maze::from_vec(grid);
    /// maze_to_create.name = "maze_1".to_string();
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Create maze within the SQL store
    /// match store.create_maze(&owner, &mut maze_to_create).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created maze in the SQL store with id = {}",
    ///             maze_to_create.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create maze => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error> {
        if maze.name.is_empty() {
            return Err(Error::MazeNameMissing());
        }

        validate_maze_cell_count(
            maze.definition.row_count(),
            maze.definition.col_count(),
            MAX_MAZE_CELLS,
        )?;
        validate_maze_feature_count(&maze.definition.grid, maze::MAX_TOTAL_FEATURES)?;
        validate_maze_object_counts(&maze.definition.grid)?;

        let existing = sqlx::query(&q(
            self.kind,
            "SELECT id FROM mazes WHERE owner_id = ? AND LOWER(name) = LOWER(?)",
        ))
        .bind(owner.id.to_string())
        .bind(&maze.name)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if existing.is_some() {
            return Err(Error::MazeNameAlreadyExists(maze.name.clone()));
        }

        maze.id = Uuid::new_v4().to_string();
        let definition_json = serde_json::to_string(&maze)?;
        validate_maze_definition_size(definition_json.len(), MAX_MAZE_DEFINITION_BYTES)?;

        sqlx::query(&q(
            self.kind,
            "INSERT INTO mazes (id, owner_id, name, definition) VALUES (?, ?, ?, ?)",
        ))
        .bind(&maze.id)
        .bind(owner.id.to_string())
        .bind(&maze.name)
        .bind(&definition_json)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    /// Deletes an existing maze from within the SQL store instance
    ///
    /// # Examples
    ///
    /// Try to delete an existing maze from within an in-memory SQLite-backed store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{Maze, User};
    /// use storage::{SqlStore, SqlStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Delete maze from within the SQL store
    /// let id = "some-maze-id".to_string();
    ///
    /// match store.delete_maze(&owner, &id).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully deleted maze from the SQL store",
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to delete maze with id {} => {}",
    ///             id,
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), Error> {
        if id.is_empty() {
            return Err(Error::MazeIdMissing());
        }
        let result = sqlx::query(&q(
            self.kind,
            "DELETE FROM mazes WHERE owner_id = ? AND id = ?",
        ))
        .bind(owner.id.to_string())
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(Error::MazeIdNotFound(id.to_string()));
        }
        // Drop the maze's leaderboard now that the maze is gone (FK cascade is a
        // backstop; deleting explicitly keeps the behaviour uniform across
        // backends). Idempotent if the FK already cascaded the rows.
        sqlx::query(&q(self.kind, "DELETE FROM score_history WHERE maze_id = ?"))
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(())
    }

    /// Updates an existing maze within the SQL store instance
    ///
    /// # Examples
    ///
    /// Try to update an existing maze within an in-memory SQLite-backed store with new content
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{Maze, User};
    /// use storage::{SqlStore, SqlStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_update = Maze::from_vec(grid);
    /// maze_to_update.name = "maze_1".to_string();
    /// maze_to_update.id = "some-maze-id".to_string();
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Update maze within the SQL store
    /// match store.update_maze(&owner, &mut maze_to_update).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully updated maze in the SQL store with id = {}",
    ///             maze_to_update.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to update maze => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error> {
        if maze.id.is_empty() {
            return Err(Error::MazeIdMissing());
        }
        validate_maze_cell_count(
            maze.definition.row_count(),
            maze.definition.col_count(),
            MAX_MAZE_CELLS,
        )?;
        validate_maze_feature_count(&maze.definition.grid, maze::MAX_TOTAL_FEATURES)?;
        validate_maze_object_counts(&maze.definition.grid)?;
        let definition_json = serde_json::to_string(&maze)?;
        validate_maze_definition_size(definition_json.len(), MAX_MAZE_DEFINITION_BYTES)?;
        let result = sqlx::query(&q(
            self.kind,
            "UPDATE mazes SET name = ?, definition = ? WHERE owner_id = ? AND id = ?",
        ))
        .bind(&maze.name)
        .bind(&definition_json)
        .bind(owner.id.to_string())
        .bind(&maze.id)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(Error::MazeIdNotFound(maze.id.clone()));
        }
        Ok(())
    }

    /// Loads a maze from within the SQL store instance
    ///
    /// # Returns
    ///
    /// The maze instance if successful
    ///
    /// # Examples
    ///
    /// Try to create and then reload a maze from within an in-memory SQLite-backed store and, if successful, print it
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{Maze, User};
    /// use maze::{MazePath, MazePrinter};
    /// use storage::{SqlStore, SqlStoreConfig, MazeStore, Store, Error, UserStore};
    /// use utils::StdoutLinePrinter;
    /// use uuid::Uuid;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_create = Maze::from_vec(grid);
    /// maze_to_create.name = "maze_1".to_string();
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Create the maze within the store
    /// if let Err(error) = store.create_maze(&owner, &mut maze_to_create).await {
    ///     println!(
    ///         "Failed to create maze => {}",
    ///         error
    ///     );
    ///     return;
    /// }
    ///
    /// // Now reload the maze from the store
    /// match store.get_maze(&owner, &maze_to_create.id).await {
    ///     Ok(loaded_maze) => {
    ///         println!("Successfully loaded maze:");
    ///         let mut print_target = StdoutLinePrinter::new();
    ///         let empty_path = MazePath { points: vec![] };
    ///         loaded_maze.print(&mut print_target, empty_path);
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to load maze with id '{}' => {}",
    ///             maze_to_create.id,
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, Error> {
        let row = sqlx::query(&q(
            self.kind,
            "SELECT id, name, definition FROM mazes WHERE owner_id = ? AND id = ?",
        ))
        .bind(owner.id.to_string())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match row {
            Some(row) => maze_from_row(&row).await,
            None => Err(Error::MazeIdNotFound(id.to_string())),
        }
    }

    /// Locates a maze item by name from within the SQL store instance
    ///
    /// # Returns
    ///
    /// The maze item if successful
    ///
    /// # Examples
    ///
    /// Try to find the maze item with name `my_maze` from within an in-memory SQLite-backed store and, if successful, print its details
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// let name = "my_maze".to_string();
    ///
    /// // Attempt to find the maze item
    /// match store.find_maze_by_name(&owner, &name).await {
    ///     Ok(maze_item) => {
    ///         println!("Successfully found maze item => id = {}, name = {}",
    ///             maze_item.id,
    ///             maze_item.name
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to find maze item with name '{}' => {}",
    ///             name,
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_maze_by_name(&self, owner: &User, name: &str) -> Result<MazeItem, Error> {
        if name.is_empty() {
            return Err(Error::MazeNameNotFound(name.to_string()));
        }
        let rows = sqlx::query(&q(
            self.kind,
            "SELECT id, name FROM mazes WHERE owner_id = ? AND LOWER(name) = LOWER(?)",
        ))
        .bind(owner.id.to_string())
        .bind(name)
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match rows.len() {
            0 => Err(Error::MazeNameNotFound(name.to_string())),
            1 => {
                let row = &rows[0];
                Ok(MazeItem {
                    id: row.try_get("id").map_err(map_sqlx_err)?,
                    name: row.try_get("name").map_err(map_sqlx_err)?,
                    definition: None,
                })
            }
            n => Err(integrity_violation(&format!(
                "{n} mazes match name '{name}' case-insensitively for owner {}",
                owner.id
            ))),
        }
    }

    /// Returns the list of maze items within the SQL store instance, sorted
    /// alphabetically in ascending order, optionally including the
    /// maze definitions as a JSON string
    ///
    /// # Returns
    ///
    /// The maze items if successful
    ///
    /// # Examples
    ///
    /// Try to load the maze items within an in-memory SQLite-backed store and, if successful, print the number of items found
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Attempt to load the maze items along with their definitions
    /// match store.get_maze_items(&owner, true).await {
    ///     Ok(maze_items) => {
    ///         println!("Successfully loaded {} maze items",
    ///             maze_items.len()
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to load maze items => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_maze_items(
        &self,
        owner: &User,
        include_definitions: bool,
    ) -> Result<Vec<MazeItem>, Error> {
        let rows = sqlx::query(&q(
            self.kind,
            "SELECT id, name, definition FROM mazes WHERE owner_id = ? ORDER BY name",
        ))
        .bind(owner.id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let id: String = row.try_get("id").map_err(map_sqlx_err)?;
            let name: String = row.try_get("name").map_err(map_sqlx_err)?;
            let definition: Option<String> = if include_definitions {
                Some(row.try_get("definition").map_err(map_sqlx_err)?)
            } else {
                None
            };
            items.push(MazeItem { id, name, definition });
        }
        Ok(items)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Manage
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl Manage for SqlStore {
    /// Resets the SQL store to its initial empty state by deleting all rows
    /// from every application table (`user_logins`, `oauth_identities`,
    /// `mazes`, and `users`) in foreign-key-safe order.
    ///
    /// Intended for tests and scripted bootstrap flows. **Destructive** —
    /// every user, login, OAuth identity, and maze is removed. The schema
    /// itself (and SQLx's `_sqlx_migrations` tracking table) is preserved
    /// so subsequent restarts skip the migration step.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(...)` if any of the underlying DELETE
    /// statements fail.
    ///
    /// # Examples
    ///
    /// Empty an in-memory SQLite-backed store before running a test scenario
    /// ```
    /// # tokio_test::block_on(async {
    /// use storage::{SqlStore, SqlStoreConfig, Manage, Store};
    ///
    /// // Create the SQL store (in-memory SQLite for the example)
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // Wipe any existing content
    /// if let Err(error) = store.empty().await {
    ///     panic!("Failed to empty the store: {}", error);
    /// }
    /// # });
    /// ```
    async fn empty(&mut self) -> Result<(), Error> {
        // Delete in FK-safe order (children first). A single TRUNCATE-equivalent
        // would be faster but TRUNCATE syntax differs across backends; portable
        // DELETEs are fine for the test/reset use case.
        //
        // `email_audit_log` is cleared first because it FKs into `users`
        // with `ON DELETE SET NULL` — clearing it explicitly avoids the
        // SET NULL fanning out across every audit row when users go.
        for sql in [
            "DELETE FROM email_audit_log",
            "DELETE FROM score_history",
            "DELETE FROM user_logins",
            "DELETE FROM oauth_identities",
            "DELETE FROM one_time_tokens",
            "DELETE FROM mazes",
            "DELETE FROM user_avatars",
            "DELETE FROM users",
        ] {
            sqlx::query(sql)
                .execute(&self.pool)
                .await
                .map_err(map_sqlx_err)?;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TokenStore
// ─────────────────────────────────────────────────────────────────────────────

/// Maps a [`TokenPurpose`] to the lowercase string written into the
/// `purpose` column. Mirrors the snake_case wire form used by the
/// FileStore JSON files so both backends agree.
fn token_purpose_to_sql(purpose: TokenPurpose) -> &'static str {
    match purpose {
        TokenPurpose::PasswordReset => "password_reset",
        TokenPurpose::EmailVerification => "email_verification",
    }
}

/// Reverse of [`token_purpose_to_sql`]. Unknown values surface a loud
/// `Error::Other` rather than silently defaulting — a stored row with an
/// unknown discriminator is a data-corruption signal, not something to
/// paper over.
fn token_purpose_from_sql(raw: &str) -> Result<TokenPurpose, Error> {
    match raw {
        "password_reset" => Ok(TokenPurpose::PasswordReset),
        "email_verification" => Ok(TokenPurpose::EmailVerification),
        other => Err(integrity_violation(&format!(
            "unknown token purpose '{other}' in one_time_tokens"
        ))),
    }
}

async fn token_from_row(row: &AnyRow) -> Result<OneTimeToken, Error> {
    let id_str: String = row.try_get("id").map_err(map_sqlx_err)?;
    let id = parse_uuid("token id", &id_str)?;
    let user_id_str: String = row.try_get("user_id").map_err(map_sqlx_err)?;
    let user_id = parse_uuid("token user_id", &user_id_str)?;
    let purpose_raw: String = row.try_get("purpose").map_err(map_sqlx_err)?;
    let purpose = token_purpose_from_sql(&purpose_raw)?;
    let target_email: Option<String> = row.try_get("target_email").map_err(map_sqlx_err)?;
    let created_at_str: String = row.try_get("created_at").map_err(map_sqlx_err)?;
    let created_at = datetime_from_sql(&created_at_str)?;
    let expires_at_str: String = row.try_get("expires_at").map_err(map_sqlx_err)?;
    let expires_at = datetime_from_sql(&expires_at_str)?;
    let consumed_at_str: Option<String> = row.try_get("consumed_at").map_err(map_sqlx_err)?;
    let consumed_at = match consumed_at_str {
        Some(s) => Some(datetime_from_sql(&s)?),
        None => None,
    };
    Ok(OneTimeToken {
        id,
        user_id,
        purpose,
        target_email,
        created_at,
        expires_at,
        consumed_at,
    })
}

#[async_trait]
impl TokenStore for SqlStore {
    /// Persists a one-time token. The caller is responsible for
    /// assigning the `id` and timestamps — typically via
    /// [`OneTimeToken::new`]. Rejects with [`Error::TokenIdExists`] on a
    /// duplicate id.
    ///
    /// # Examples
    ///
    /// Issue a password-reset token for a freshly-created user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{OneTimeToken, TokenPurpose, User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, TokenStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// let token = OneTimeToken::new(user.id, TokenPurpose::PasswordReset, None, 1);
    /// store.create_token(&token).await.expect("create_token");
    /// # });
    /// ```
    async fn create_token(&mut self, token: &OneTimeToken) -> Result<(), Error> {
        if token.id.is_nil() {
            return Err(Error::Other("token id must not be nil".to_string()));
        }
        if token.user_id.is_nil() {
            return Err(Error::UserIdMissing());
        }
        // Surface a clean error if the id is already taken — gives callers
        // a deterministic signal rather than the raw sqlx unique-violation.
        let existing = sqlx::query(&q(
            self.kind,
            "SELECT 1 FROM one_time_tokens WHERE id = ?",
        ))
        .bind(token.id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if existing.is_some() {
            return Err(Error::TokenIdExists(token.id.to_string()));
        }
        sqlx::query(&q(
            self.kind,
            "INSERT INTO one_time_tokens \
                 (id, user_id, purpose, target_email, created_at, expires_at, consumed_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        ))
        .bind(token.id.to_string())
        .bind(token.user_id.to_string())
        .bind(token_purpose_to_sql(token.purpose))
        .bind(token.target_email.as_deref())
        .bind(datetime_to_sql(token.created_at))
        .bind(datetime_to_sql(token.expires_at))
        .bind(token.consumed_at.map(datetime_to_sql))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    /// Loads an active (non-expired, non-consumed) token by id. Returns
    /// `Err(TokenIdNotFound)` for unknown ids and for tokens past their
    /// `expires_at`.
    ///
    /// # Examples
    ///
    /// Round-trip a token through `create_token` + `find_token`
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{OneTimeToken, TokenPurpose, User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, TokenStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// let token = OneTimeToken::new(user.id, TokenPurpose::PasswordReset, None, 1);
    /// store.create_token(&token).await.expect("create_token");
    /// let loaded = store.find_token(token.id).await.expect("find_token");
    /// assert_eq!(loaded.user_id, user.id);
    /// # });
    /// ```
    async fn find_token(&self, id: Uuid) -> Result<OneTimeToken, Error> {
        // Filter expired tokens at the storage layer so handlers can treat
        // "find_token Ok" as "active and consumable". Soft-deleted users
        // are filtered via the FK + the application-level cascade in
        // delete_user — once the row is hard-deleted, the token is gone.
        let now = datetime_to_sql(Utc::now());
        let row = sqlx::query(&q(
            self.kind,
            "SELECT * FROM one_time_tokens WHERE id = ? AND expires_at > ?",
        ))
        .bind(id.to_string())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match row {
            Some(row) => token_from_row(&row).await,
            None => Err(Error::TokenIdNotFound(id.to_string())),
        }
    }

    /// Atomically marks the token consumed via
    /// `UPDATE ... WHERE consumed_at IS NULL` so concurrent calls
    /// against the same id produce exactly one winner. Race losses,
    /// expired tokens, and unknown ids are distinguished by a
    /// follow-up read on the slow path.
    ///
    /// # Examples
    ///
    /// Single-use enforcement: the second consume call fails
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{OneTimeToken, TokenPurpose, User, UserEmail};
    /// use storage::{Error, SqlStore, SqlStoreConfig, Store, TokenStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// let token = OneTimeToken::new(user.id, TokenPurpose::PasswordReset, None, 1);
    /// store.create_token(&token).await.expect("create_token");
    /// store.consume_token(token.id).await.expect("first consume");
    /// assert!(matches!(
    ///     store.consume_token(token.id).await,
    ///     Err(Error::TokenAlreadyConsumed())
    /// ));
    /// # });
    /// ```
    async fn consume_token(&mut self, id: Uuid) -> Result<OneTimeToken, Error> {
        // Race-free single-use enforcement: the UPDATE only matches when
        // the token is unconsumed. A losing concurrent UPDATE matches zero
        // rows and surfaces TokenAlreadyConsumed (or NotFound / Expired).
        let now_dt = Utc::now();
        let now_sql = datetime_to_sql(now_dt);
        let result = sqlx::query(&q(
            self.kind,
            "UPDATE one_time_tokens \
             SET consumed_at = ? \
             WHERE id = ? AND consumed_at IS NULL AND expires_at > ?",
        ))
        .bind(&now_sql)
        .bind(id.to_string())
        .bind(&now_sql)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            // Distinguish the failure modes by re-reading the row. This
            // second probe runs only on the race-loss / expiry / missing
            // path, so it doesn't tax the happy path.
            let row = sqlx::query(&q(
                self.kind,
                "SELECT * FROM one_time_tokens WHERE id = ?",
            ))
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
            return match row {
                None => Err(Error::TokenIdNotFound(id.to_string())),
                Some(row) => {
                    let token = token_from_row(&row).await?;
                    if token.consumed_at.is_some() {
                        Err(Error::TokenAlreadyConsumed())
                    } else {
                        Err(Error::TokenExpired())
                    }
                }
            };
        }
        // Re-read to return the updated row as the trait contract requires.
        let row = sqlx::query(&q(self.kind, "SELECT * FROM one_time_tokens WHERE id = ?"))
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?
            .ok_or_else(|| Error::TokenIdNotFound(id.to_string()))?;
        token_from_row(&row).await
    }

    /// Removes every outstanding [`TokenPurpose::EmailVerification`]
    /// token belonging to `user_id` whose `target_email` matches the
    /// supplied address (case-insensitive). Used by the verification
    /// re-send handler so re-issuing supersedes prior tokens.
    ///
    /// # Examples
    ///
    /// Two verification tokens issued for the same address — purging
    /// removes both
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{OneTimeToken, TokenPurpose, User, UserEmail};
    /// use storage::{SqlStore, SqlStoreConfig, Store, TokenStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// for _ in 0..2 {
    ///     let t = OneTimeToken::new(
    ///         user.id, TokenPurpose::EmailVerification,
    ///         Some("alice@example.com".into()), 24,
    ///     );
    ///     store.create_token(&t).await.expect("create_token");
    /// }
    /// let purged = store
    ///     .purge_email_verification_tokens(user.id, "alice@example.com")
    ///     .await
    ///     .expect("purge");
    /// assert_eq!(purged, 2);
    /// # });
    /// ```
    async fn purge_email_verification_tokens(
        &mut self,
        user_id: Uuid,
        target_email: &str,
    ) -> Result<u64, Error> {
        let result = sqlx::query(&q(
            self.kind,
            "DELETE FROM one_time_tokens \
             WHERE user_id = ? AND purpose = ? AND LOWER(target_email) = LOWER(?)",
        ))
        .bind(user_id.to_string())
        .bind(token_purpose_to_sql(TokenPurpose::EmailVerification))
        .bind(target_email)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(result.rows_affected())
    }

    /// Removes every token whose `expires_at` is in the past AND that
    /// has not been consumed. Returns the number of rows deleted.
    /// Intended as a periodic housekeeping sweep.
    ///
    /// # Examples
    ///
    /// Purging a fresh store is a no-op
    /// ```
    /// # tokio_test::block_on(async {
    /// use storage::{SqlStore, SqlStoreConfig, Store, TokenStore};
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// assert_eq!(store.purge_expired().await.expect("purge"), 0);
    /// # });
    /// ```
    async fn purge_expired(&mut self) -> Result<u64, Error> {
        let now = datetime_to_sql(Utc::now());
        let result = sqlx::query(&q(
            self.kind,
            "DELETE FROM one_time_tokens WHERE expires_at <= ? AND consumed_at IS NULL",
        ))
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(result.rows_affected())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EmailAuditLog
// ─────────────────────────────────────────────────────────────────────────────

fn audit_outcome_to_sql(outcome: AuditOutcome) -> &'static str {
    match outcome {
        AuditOutcome::Pending => "pending",
        AuditOutcome::Accepted => "accepted",
        AuditOutcome::Failed => "failed",
    }
}

fn audit_outcome_from_sql(raw: &str) -> Result<AuditOutcome, Error> {
    match raw {
        "pending" => Ok(AuditOutcome::Pending),
        "accepted" => Ok(AuditOutcome::Accepted),
        "failed" => Ok(AuditOutcome::Failed),
        other => Err(integrity_violation(&format!(
            "unknown audit outcome '{other}' in email_audit_log"
        ))),
    }
}

async fn audit_entry_from_row(row: &AnyRow) -> Result<EmailAuditEntry, Error> {
    let id_str: String = row.try_get("id").map_err(map_sqlx_err)?;
    let id = parse_uuid("audit id", &id_str)?;
    let created_at_str: String = row.try_get("created_at").map_err(map_sqlx_err)?;
    let created_at = datetime_from_sql(&created_at_str)?;
    let recipient_user_id_str: Option<String> =
        row.try_get("recipient_user_id").map_err(map_sqlx_err)?;
    let recipient_user_id = match recipient_user_id_str {
        Some(s) => Some(parse_uuid("audit recipient_user_id", &s)?),
        None => None,
    };
    let recipient_email: String = row.try_get("recipient_email").map_err(map_sqlx_err)?;
    let template_id: String = row.try_get("template_id").map_err(map_sqlx_err)?;
    let token_id_str: Option<String> = row.try_get("token_id").map_err(map_sqlx_err)?;
    let token_id = match token_id_str {
        Some(s) => Some(parse_uuid("audit token_id", &s)?),
        None => None,
    };
    let triggered_by_user_id_str: Option<String> =
        row.try_get("triggered_by_user_id").map_err(map_sqlx_err)?;
    let triggered_by_user_id = match triggered_by_user_id_str {
        Some(s) => Some(parse_uuid("audit triggered_by_user_id", &s)?),
        None => None,
    };
    let provider: String = row.try_get("provider").map_err(map_sqlx_err)?;
    let provider_message_id: Option<String> =
        row.try_get("provider_message_id").map_err(map_sqlx_err)?;
    let outcome_raw: String = row.try_get("outcome").map_err(map_sqlx_err)?;
    let outcome = audit_outcome_from_sql(&outcome_raw)?;
    let error_class: Option<String> = row.try_get("error_class").map_err(map_sqlx_err)?;
    let error_message: Option<String> = row.try_get("error_message").map_err(map_sqlx_err)?;
    Ok(EmailAuditEntry {
        id,
        created_at,
        recipient_user_id,
        recipient_email,
        template_id,
        token_id,
        triggered_by_user_id,
        provider,
        provider_message_id,
        outcome,
        error_class,
        error_message,
    })
}

#[async_trait]
impl EmailAuditLog for SqlStore {
    /// Inserts a new audit row synchronously, before the actual send is
    /// attempted. Caller builds the entry via
    /// [`EmailAuditEntry::new_pending`]; this method just persists it.
    /// Returns the assigned id on success.
    ///
    /// # Examples
    ///
    /// Record a pending password-reset send
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::EmailAuditEntry;
    /// use storage::{EmailAuditLog, SqlStore, SqlStoreConfig, Store};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let entry = EmailAuditEntry::new_pending(
    ///     None, "alice@example.com", "password_reset",
    ///     None, None, "stub",
    /// );
    /// let id = store.record_pending(&entry).await.expect("record_pending");
    /// assert_eq!(id, entry.id);
    /// # });
    /// ```
    async fn record_pending(&mut self, entry: &EmailAuditEntry) -> Result<Uuid, Error> {
        if entry.id.is_nil() {
            return Err(Error::Other(
                "audit entry id must not be nil".to_string(),
            ));
        }
        // Surface a clean duplicate-id error rather than the raw sqlx
        // unique-violation. Callers building rows via
        // `EmailAuditEntry::new_pending` won't hit this; it guards
        // against accidental re-record.
        let existing = sqlx::query(&q(
            self.kind,
            "SELECT 1 FROM email_audit_log WHERE id = ?",
        ))
        .bind(entry.id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if existing.is_some() {
            return Err(Error::AuditEntryIdExists(entry.id.to_string()));
        }
        let truncated_error_message = entry
            .error_message
            .as_deref()
            .map(truncate_email_audit_error_message);
        sqlx::query(&q(
            self.kind,
            "INSERT INTO email_audit_log \
                 (id, created_at, recipient_user_id, recipient_email, template_id, \
                  token_id, triggered_by_user_id, provider, \
                  provider_message_id, outcome, error_class, error_message) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ))
        .bind(entry.id.to_string())
        .bind(datetime_to_sql(entry.created_at))
        .bind(entry.recipient_user_id.map(|id| id.to_string()))
        .bind(&entry.recipient_email)
        .bind(&entry.template_id)
        .bind(entry.token_id.map(|id| id.to_string()))
        .bind(entry.triggered_by_user_id.map(|id| id.to_string()))
        .bind(&entry.provider)
        .bind(entry.provider_message_id.as_deref())
        .bind(audit_outcome_to_sql(entry.outcome))
        .bind(entry.error_class.as_deref())
        .bind(truncated_error_message.as_deref())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(entry.id)
    }

    /// Flips a previously-recorded `pending` row to `accepted` (with
    /// `provider_message_id`) or `failed` (with `error_class` and
    /// `error_message`). Once written, an audit row only moves forward —
    /// passing `AuditOutcome::Pending` is rejected.
    ///
    /// # Examples
    ///
    /// Mark the audit row as accepted after the provider responds
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{AuditOutcome, EmailAuditEntry};
    /// use storage::{EmailAuditLog, SqlStore, SqlStoreConfig, Store};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let entry = EmailAuditEntry::new_pending(
    ///     None, "alice@example.com", "password_reset",
    ///     None, None, "stub",
    /// );
    /// store.record_pending(&entry).await.expect("record_pending");
    /// store
    ///     .update_outcome(entry.id, AuditOutcome::Accepted, Some("provider-123"), None, None)
    ///     .await
    ///     .expect("update_outcome");
    /// # });
    /// ```
    async fn update_outcome(
        &mut self,
        id: Uuid,
        outcome: AuditOutcome,
        provider_message_id: Option<&str>,
        error_class: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), Error> {
        if matches!(outcome, AuditOutcome::Pending) {
            return Err(Error::Other(
                "update_outcome cannot move a row back to pending".to_string(),
            ));
        }
        let truncated_error_message = error_message.map(truncate_email_audit_error_message);
        let result = sqlx::query(&q(
            self.kind,
            "UPDATE email_audit_log \
             SET outcome = ?, provider_message_id = ?, error_class = ?, error_message = ? \
             WHERE id = ?",
        ))
        .bind(audit_outcome_to_sql(outcome))
        .bind(provider_message_id)
        .bind(error_class)
        .bind(truncated_error_message.as_deref())
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if result.rows_affected() == 0 {
            return Err(Error::AuditEntryIdNotFound(id.to_string()));
        }
        Ok(())
    }

    /// Loads a single audit row by id. Returns
    /// `Err(AuditEntryIdNotFound)` for unknown ids.
    ///
    /// # Examples
    ///
    /// Load back a recorded row
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::EmailAuditEntry;
    /// use storage::{EmailAuditLog, SqlStore, SqlStoreConfig, Store};
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// let entry = EmailAuditEntry::new_pending(
    ///     None, "alice@example.com", "password_reset",
    ///     None, None, "stub",
    /// );
    /// store.record_pending(&entry).await.expect("record_pending");
    /// let loaded = store.find_audit_entry(entry.id).await.expect("find");
    /// assert_eq!(loaded.recipient_email, "alice@example.com");
    /// # });
    /// ```
    async fn find_audit_entry(&self, id: Uuid) -> Result<EmailAuditEntry, Error> {
        let row = sqlx::query(&q(
            self.kind,
            "SELECT * FROM email_audit_log WHERE id = ?",
        ))
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        match row {
            Some(row) => audit_entry_from_row(&row).await,
            None => Err(Error::AuditEntryIdNotFound(id.to_string())),
        }
    }

    /// Returns the `limit` most recent audit rows for a user
    /// (`recipient_user_id = user_id`), sorted by `created_at`
    /// descending with `id` as a deterministic tie-breaker.
    ///
    /// # Examples
    ///
    /// Read back the most recent two audit entries for a user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::EmailAuditEntry;
    /// use storage::{EmailAuditLog, SqlStore, SqlStoreConfig, Store, UserStore};
    /// use data_model::{User, UserEmail};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    /// // Recipient must exist for the FK; create the user first.
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// for template in ["password_reset", "email_verification"] {
    ///     let e = EmailAuditEntry::new_pending(
    ///         Some(user.id), "alice@example.com", template,
    ///         None, None, "stub",
    ///     );
    ///     store.record_pending(&e).await.expect("record_pending");
    /// }
    /// let recent = store
    ///     .find_recent_audit_entries_for_user(user.id, 5)
    ///     .await
    ///     .expect("find_recent");
    /// assert_eq!(recent.len(), 2);
    /// # });
    /// ```
    async fn find_recent_audit_entries_for_user(
        &self,
        user_id: Uuid,
        limit: u32,
    ) -> Result<Vec<EmailAuditEntry>, Error> {
        let rows = sqlx::query(&q(
            self.kind,
            "SELECT * FROM email_audit_log \
             WHERE recipient_user_id = ? \
             ORDER BY created_at DESC, id DESC LIMIT ?",
        ))
        .bind(user_id.to_string())
        .bind(i64::from(limit))
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        let mut entries = Vec::with_capacity(rows.len());
        for row in &rows {
            entries.push(audit_entry_from_row(row).await?);
        }
        Ok(entries)
    }
}

/// The `ORDER BY` clause for a leaderboard ordering. The primary metric takes
/// the requested direction; the secondary (the other metric) and the
/// `recorded_at` / `id` final keys are fixed. Built from fixed column names
/// (never user input), so it is safe to interpolate into the query.
fn score_order_by_clause(ordering: ScoreOrdering) -> String {
    let primary = match ordering.direction {
        SortDirection::Ascending => "ASC",
        SortDirection::Descending => "DESC",
    };
    // Columns are `s.`-qualified: the board queries alias `score_history` as
    // `s`, and the `id` tiebreaker is otherwise ambiguous when the optional
    // `LEFT JOIN users u` is present.
    match ordering.metric {
        ScoreMetric::Time => {
            format!("s.elapsed_ms {primary}, s.score DESC, s.recorded_at ASC, s.id ASC")
        }
        ScoreMetric::Score => {
            format!("s.score {primary}, s.elapsed_ms ASC, s.recorded_at ASC, s.id ASC")
        }
    }
}

/// Builds the board SELECT for a single `WHERE` column (`maze_id` or
/// `challenge`). When `include_usernames` is set, joins `users` so each row
/// carries the player's `username` in one round-trip; otherwise selects the
/// score columns alone.
fn score_board_sql(where_col: &str, ordering: ScoreOrdering, include_usernames: bool) -> String {
    let order = score_order_by_clause(ordering);
    if include_usernames {
        format!(
            "SELECT s.*, u.username, u.avatar_updated_at FROM score_history s \
             LEFT JOIN users u ON u.id = s.user_id \
             WHERE s.{where_col} = ? ORDER BY {order} LIMIT ? OFFSET ?"
        )
    } else {
        format!(
            "SELECT s.* FROM score_history s \
             WHERE s.{where_col} = ? ORDER BY {order} LIMIT ? OFFSET ?"
        )
    }
}

/// Deserialises a `score_history` row into a [`ScoreEntry`]. `score` /
/// `elapsed_ms` come back as `i64` (BIGINT) and widen to the struct's `u64`.
fn score_entry_from_row(row: &AnyRow) -> Result<ScoreEntry, Error> {
    let id_str: String = row.try_get("id").map_err(map_sqlx_err)?;
    let id = parse_uuid("score id", &id_str)?;
    let user_id_str: String = row.try_get("user_id").map_err(map_sqlx_err)?;
    let user_id = parse_uuid("score user_id", &user_id_str)?;
    let maze_id: Option<String> = row.try_get("maze_id").map_err(map_sqlx_err)?;
    let challenge: Option<String> = row.try_get("challenge").map_err(map_sqlx_err)?;
    let score: i64 = row.try_get("score").map_err(map_sqlx_err)?;
    let elapsed_ms: i64 = row.try_get("elapsed_ms").map_err(map_sqlx_err)?;
    let recorded_at_str: String = row.try_get("recorded_at").map_err(map_sqlx_err)?;
    let recorded_at = datetime_from_sql(&recorded_at_str)?;
    Ok(ScoreEntry {
        id,
        user_id,
        maze_id,
        challenge,
        score: score as u64,
        elapsed_ms: elapsed_ms as u64,
        recorded_at,
    })
}

/// Deserialises a board row into a [`ScoreboardEntry`]. Reads the joined
/// `username` column only when `with_username` is set (it is absent from the
/// SELECT otherwise).
fn scoreboard_entry_from_row(row: &AnyRow, with_username: bool) -> Result<ScoreboardEntry, Error> {
    let entry = score_entry_from_row(row)?;
    // Both columns ride the same `LEFT JOIN users`, so they're present together
    // or absent together — gated by the one `with_username` flag.
    let (username, avatar_updated_at) = if with_username {
        let username = row.try_get::<Option<String>, _>("username").map_err(map_sqlx_err)?;
        let avatar_str = row
            .try_get::<Option<String>, _>("avatar_updated_at")
            .map_err(map_sqlx_err)?;
        let avatar_updated_at = match avatar_str {
            Some(s) => Some(datetime_from_sql(&s)?),
            None => None,
        };
        (username, avatar_updated_at)
    } else {
        (None, None)
    };
    Ok(ScoreboardEntry {
        entry,
        username,
        avatar_updated_at,
    })
}

#[async_trait]
impl ScoreStore for SqlStore {
    /// Inserts a completed-run row. Enforces the subject invariant (exactly one
    /// of `maze_id` / `challenge`) before the write.
    ///
    /// # Examples
    ///
    /// Record a curated-game run and read it back from the challenge board
    /// ```
    /// # tokio_test::block_on(async {
    /// use storage::{
    ///     ScoreEntry, ScoreMetric, ScoreOrdering, ScoreStore, SortDirection,
    ///     SqlStore, SqlStoreConfig, UserStore,
    /// };
    /// use data_model::{User, UserEmail};
    /// use uuid::Uuid;
    ///
    /// let mut store = SqlStore::new(SqlStoreConfig {
    ///     url: "sqlite::memory:".to_string(),
    ///     max_connections: 1,
    ///     auto_create_database: true,
    ///     ..SqlStoreConfig::default()
    /// })
    /// .await
    /// .expect("create in-memory SqlStore");
    ///
    /// // The player must exist for the user_id FK.
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    ///
    /// let entry = ScoreEntry {
    ///     id: Uuid::new_v4(), user_id: user.id,
    ///     maze_id: None, challenge: Some("hard:42".to_string()),
    ///     score: 5, elapsed_ms: 83_456, recorded_at: chrono::Utc::now(),
    /// };
    /// store.record_score(&entry).await.expect("record_score");
    ///
    /// let highest = ScoreOrdering {
    ///     metric: ScoreMetric::Score,
    ///     direction: SortDirection::Descending,
    /// };
    /// let board = store
    ///     .challenge_leaderboard("hard:42", highest, 10, 0, true)
    ///     .await
    ///     .expect("challenge_leaderboard");
    /// assert_eq!(board.len(), 1);
    /// assert_eq!(board[0].entry.score, 5);
    /// assert_eq!(board[0].username.as_deref(), Some("alice"));
    /// # });
    /// ```
    async fn record_score(&mut self, entry: &ScoreEntry) -> Result<Uuid, Error> {
        if entry.id.is_nil() {
            return Err(Error::Other("score entry id must not be nil".to_string()));
        }
        crate::store::validate_score_subject(entry)?;
        sqlx::query(&q(
            self.kind,
            "INSERT INTO score_history \
                 (id, user_id, maze_id, challenge, score, elapsed_ms, recorded_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        ))
        .bind(entry.id.to_string())
        .bind(entry.user_id.to_string())
        .bind(entry.maze_id.as_deref())
        .bind(entry.challenge.as_deref())
        .bind(entry.score as i64)
        .bind(entry.elapsed_ms as i64)
        .bind(datetime_to_sql(entry.recorded_at))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(entry.id)
    }

    async fn maze_leaderboard(
        &self,
        maze_id: &str,
        ordering: ScoreOrdering,
        limit: u32,
        offset: u32,
        include_usernames: bool,
    ) -> Result<Vec<ScoreboardEntry>, Error> {
        let sql = score_board_sql("maze_id", ordering, include_usernames);
        let rows = sqlx::query(&q(self.kind, &sql))
            .bind(maze_id)
            .bind(i64::from(limit))
            .bind(i64::from(offset))
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        rows.iter().map(|r| scoreboard_entry_from_row(r, include_usernames)).collect()
    }

    async fn challenge_leaderboard(
        &self,
        challenge: &str,
        ordering: ScoreOrdering,
        limit: u32,
        offset: u32,
        include_usernames: bool,
    ) -> Result<Vec<ScoreboardEntry>, Error> {
        let sql = score_board_sql("challenge", ordering, include_usernames);
        let rows = sqlx::query(&q(self.kind, &sql))
            .bind(challenge)
            .bind(i64::from(limit))
            .bind(i64::from(offset))
            .fetch_all(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        rows.iter().map(|r| scoreboard_entry_from_row(r, include_usernames)).collect()
    }

    async fn user_history(
        &self,
        user_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ScoreEntry>, Error> {
        let rows = sqlx::query(&q(
            self.kind,
            "SELECT * FROM score_history WHERE user_id = ? \
             ORDER BY recorded_at DESC, id DESC LIMIT ? OFFSET ?",
        ))
        .bind(user_id.to_string())
        .bind(i64::from(limit))
        .bind(i64::from(offset))
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        rows.iter().map(score_entry_from_row).collect()
    }

    async fn clear_maze_scores(&mut self, maze_id: &str) -> Result<u64, Error> {
        let result = sqlx::query(&q(self.kind, "DELETE FROM score_history WHERE maze_id = ?"))
            .bind(maze_id)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(result.rows_affected())
    }

    async fn clear_challenge_scores(&mut self, challenge: &str) -> Result<u64, Error> {
        let result = sqlx::query(&q(self.kind, "DELETE FROM score_history WHERE challenge = ?"))
            .bind(challenge)
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(result.rows_affected())
    }

    async fn clear_challenge_scores_prefix(&mut self, prefix: &str) -> Result<u64, Error> {
        // `prefix` is always `def:<uuid>` (no LIKE metacharacters), so the
        // `prefix:%` pattern is safe without escaping. Matches the static
        // `"def:<id>"` board and every daily `"def:<id>:<date>"` board.
        let result = sqlx::query(&q(
            self.kind,
            "DELETE FROM score_history WHERE challenge = ? OR challenge LIKE ?",
        ))
        .bind(prefix)
        .bind(format!("{prefix}:%"))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(result.rows_affected())
    }
}

impl SqlStore {
    /// Runs a `SELECT … FROM game_definitions …` query with the given ordered
    /// string binds and maps each row to a [`GameDefinition`]. Shared by the
    /// owner / curated / public / shared-with list reads.
    async fn query_game_definitions(
        &self,
        sql: &str,
        binds: &[String],
    ) -> Result<Vec<GameDefinition>, Error> {
        let translated = q(self.kind, sql);
        let mut query = sqlx::query(&translated);
        for bind in binds {
            query = query.bind(bind.as_str());
        }
        let rows = query.fetch_all(&self.pool).await.map_err(map_sqlx_err)?;
        rows.iter().map(game_definition_from_row).collect()
    }
}

#[async_trait]
impl GameStore for SqlStore {
    async fn create_game_definition(
        &mut self,
        owner: &User,
        definition: &mut GameDefinition,
    ) -> Result<(), Error> {
        if definition.name.trim().is_empty() {
            return Err(Error::GameDefinitionNameMissing());
        }
        let config_json = serde_json::to_string(&definition.config)?;
        validate_game_definition_config_size(config_json.len(), MAX_GAME_DEFINITION_CONFIG_BYTES)?;

        let existing = sqlx::query(&q(
            self.kind,
            "SELECT id FROM game_definitions WHERE owner_id = ? AND LOWER(name) = LOWER(?)",
        ))
        .bind(owner.id.to_string())
        .bind(&definition.name)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if existing.is_some() {
            return Err(Error::GameDefinitionNameAlreadyExists(definition.name.clone()));
        }

        definition.owner_id = owner.id;
        if definition.id.is_nil() {
            definition.id = Uuid::new_v4();
        }
        let now = Utc::now().trunc_subsecs(3);
        definition.created_at = now;
        definition.updated_at = now;

        sqlx::query(&q(
            self.kind,
            "INSERT INTO game_definitions \
             (id, owner_id, name, description, image_updated_at, visibility, seed, rotation, config, created_at, updated_at) \
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        ))
        .bind(definition.id.to_string())
        .bind(definition.owner_id.to_string())
        .bind(&definition.name)
        .bind(definition.description.clone())
        .bind(definition.image_updated_at.map(datetime_to_sql))
        .bind(definition.visibility.as_wire_str())
        .bind(definition.seed as i64)
        .bind(definition.rotation.as_wire_str())
        .bind(&config_json)
        .bind(datetime_to_sql(definition.created_at))
        .bind(datetime_to_sql(definition.updated_at))
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn get_game_definition(&self, id: Uuid) -> Result<GameDefinition, Error> {
        let row = sqlx::query(&q(self.kind, "SELECT * FROM game_definitions WHERE id = ?"))
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        match row {
            Some(row) => game_definition_from_row(&row),
            None => Err(Error::GameDefinitionIdNotFound(id.to_string())),
        }
    }

    async fn update_game_definition(
        &mut self,
        owner: &User,
        definition: &mut GameDefinition,
    ) -> Result<(), Error> {
        let existing = self.get_game_definition(definition.id).await?;
        if existing.owner_id != owner.id {
            return Err(Error::GameDefinitionIdNotFound(definition.id.to_string()));
        }
        if definition.name.trim().is_empty() {
            return Err(Error::GameDefinitionNameMissing());
        }
        let config_json = serde_json::to_string(&definition.config)?;
        validate_game_definition_config_size(config_json.len(), MAX_GAME_DEFINITION_CONFIG_BYTES)?;

        let clash = sqlx::query(&q(
            self.kind,
            "SELECT id FROM game_definitions WHERE owner_id = ? AND LOWER(name) = LOWER(?) AND id <> ?",
        ))
        .bind(owner.id.to_string())
        .bind(&definition.name)
        .bind(definition.id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if clash.is_some() {
            return Err(Error::GameDefinitionNameAlreadyExists(definition.name.clone()));
        }

        definition.owner_id = owner.id;
        definition.created_at = existing.created_at;
        definition.updated_at = Utc::now().trunc_subsecs(3);

        sqlx::query(&q(
            self.kind,
            "UPDATE game_definitions SET name = ?, description = ?, image_updated_at = ?, \
             visibility = ?, seed = ?, rotation = ?, config = ?, updated_at = ? WHERE id = ?",
        ))
        .bind(&definition.name)
        .bind(definition.description.clone())
        .bind(definition.image_updated_at.map(datetime_to_sql))
        .bind(definition.visibility.as_wire_str())
        .bind(definition.seed as i64)
        .bind(definition.rotation.as_wire_str())
        .bind(&config_json)
        .bind(datetime_to_sql(definition.updated_at))
        .bind(definition.id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn delete_game_definition(&mut self, owner: &User, id: Uuid) -> Result<(), Error> {
        let existing = self.get_game_definition(id).await?;
        if existing.owner_id != owner.id {
            return Err(Error::GameDefinitionIdNotFound(id.to_string()));
        }
        // Shares cascade via the FK, but delete explicitly for uniform
        // behaviour across backends (SQLite FK enforcement is pragma-gated).
        sqlx::query(&q(
            self.kind,
            "DELETE FROM game_definition_shares WHERE definition_id = ?",
        ))
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        sqlx::query(&q(self.kind, "DELETE FROM game_definitions WHERE id = ?"))
            .bind(id.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn grant_definition_access(
        &mut self,
        owner: &User,
        id: Uuid,
        grantee: Uuid,
    ) -> Result<(), Error> {
        let existing = self.get_game_definition(id).await?;
        if existing.owner_id != owner.id {
            return Err(Error::GameDefinitionIdNotFound(id.to_string()));
        }
        // Idempotent: skip when already present (the composite PK would reject a
        // duplicate insert).
        let present = sqlx::query(&q(
            self.kind,
            "SELECT 1 AS present FROM game_definition_shares WHERE definition_id = ? AND grantee_user_id = ?",
        ))
        .bind(id.to_string())
        .bind(grantee.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        if present.is_none() {
            sqlx::query(&q(
                self.kind,
                "INSERT INTO game_definition_shares (definition_id, grantee_user_id) VALUES (?, ?)",
            ))
            .bind(id.to_string())
            .bind(grantee.to_string())
            .execute(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        }
        Ok(())
    }

    async fn revoke_definition_access(
        &mut self,
        owner: &User,
        id: Uuid,
        grantee: Uuid,
    ) -> Result<(), Error> {
        let existing = self.get_game_definition(id).await?;
        if existing.owner_id != owner.id {
            return Err(Error::GameDefinitionIdNotFound(id.to_string()));
        }
        sqlx::query(&q(
            self.kind,
            "DELETE FROM game_definition_shares WHERE definition_id = ? AND grantee_user_id = ?",
        ))
        .bind(id.to_string())
        .bind(grantee.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    async fn get_definitions_for_owner(&self, owner: &User) -> Result<Vec<GameDefinition>, Error> {
        self.query_game_definitions(
            "SELECT * FROM game_definitions WHERE owner_id = ? ORDER BY LOWER(name) ASC",
            &[owner.id.to_string()],
        )
        .await
    }

    async fn get_curated_definitions(&self) -> Result<Vec<GameDefinition>, Error> {
        self.query_game_definitions(
            "SELECT * FROM game_definitions WHERE visibility = ? ORDER BY LOWER(name) ASC",
            &[Visibility::Curated.as_wire_str().to_string()],
        )
        .await
    }

    async fn get_public_definitions(&self) -> Result<Vec<GameDefinition>, Error> {
        self.query_game_definitions(
            "SELECT * FROM game_definitions WHERE visibility = ? ORDER BY LOWER(name) ASC",
            &[Visibility::Public.as_wire_str().to_string()],
        )
        .await
    }

    async fn get_definitions_shared_with(
        &self,
        user: Uuid,
    ) -> Result<Vec<GameDefinition>, Error> {
        self.query_game_definitions(
            "SELECT gd.* FROM game_definitions gd \
             JOIN game_definition_shares s ON s.definition_id = gd.id \
             WHERE s.grantee_user_id = ? ORDER BY LOWER(gd.name) ASC",
            &[user.to_string()],
        )
        .await
    }

    async fn get_definition_grantees(&self, id: Uuid) -> Result<Vec<Uuid>, Error> {
        let rows = sqlx::query(&q(
            self.kind,
            "SELECT grantee_user_id FROM game_definition_shares WHERE definition_id = ? ORDER BY grantee_user_id ASC",
        ))
        .bind(id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_err)?;
        rows.iter()
            .map(|row| {
                let s: String = row.try_get("grantee_user_id").map_err(map_sqlx_err)?;
                parse_uuid("grantee_user_id", &s)
            })
            .collect()
    }
}

impl Store for SqlStore {}

// ─────────────────────────────────────────────────────────────────────────────
// Tests — datetime helpers only. Full SqlStore tests land in Step 4.1.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn datetime_format_is_fixed_width_rfc3339_with_z() {
        let dt = Utc.with_ymd_and_hms(2026, 4, 28, 15, 30, 45).unwrap();
        let s = datetime_to_sql(dt);
        assert_eq!(s, "2026-04-28T15:30:45.000Z");
        assert_eq!(s.len(), 24);
    }

    #[test]
    fn datetime_round_trips_through_format() {
        let dt = Utc.with_ymd_and_hms(2026, 4, 28, 15, 30, 45).unwrap();
        let round_tripped = datetime_from_sql(&datetime_to_sql(dt)).unwrap();
        assert_eq!(round_tripped, dt);
    }

    #[test]
    fn lexicographic_order_matches_chronological_order() {
        // The schema relies on this property to support portable SQL-side
        // range queries. Verify a handful of close-spaced timestamps order
        // the same way as strings as they do as DateTime values.
        let dts = vec![
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 1).unwrap(),
            Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap(),
            Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2099, 12, 31, 23, 59, 59).unwrap(),
        ];
        let mut as_strings: Vec<String> = dts.iter().copied().map(datetime_to_sql).collect();
        as_strings.sort();
        let parsed_back: Vec<DateTime<Utc>> = as_strings
            .iter()
            .map(|s| datetime_from_sql(s).unwrap())
            .collect();
        assert_eq!(parsed_back, dts);
    }

    #[test]
    fn datetime_from_sql_rejects_bad_input() {
        assert!(datetime_from_sql("not a timestamp").is_err());
        assert!(datetime_from_sql("").is_err());
    }

    // ── score_history smoke tests (in-memory SQLite). The full cross-backend
    //    contract suite lives in tests/. ──────────────────────────────────────

    async fn mem_store_with_user() -> (SqlStore, data_model::User) {
        let mut store = SqlStore::new(SqlStoreConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            auto_create_database: true,
            ..SqlStoreConfig::default()
        })
        .await
        .expect("in-memory SqlStore");
        let mut user = data_model::User {
            id: Uuid::nil(),
            is_admin: false,
            username: "alice".into(),
            full_name: "Alice".into(),
            emails: vec![data_model::UserEmail::new_primary_verified("alice@example.com")],
            password_hash: "hash".into(),
            api_key: Uuid::nil(),
            logins: vec![],
            oauth_identities: vec![],
            deleted_at: None,
            created_at: Utc::now(),
            last_sign_in_at: None,
            avatar_updated_at: None,
        };
        store.create_user(&mut user).await.expect("create_user");
        (store, user)
    }

    fn challenge_score(user_id: Uuid, challenge: &str, score: u64, elapsed_ms: u64) -> ScoreEntry {
        ScoreEntry {
            id: Uuid::new_v4(),
            user_id,
            maze_id: None,
            challenge: Some(challenge.to_string()),
            score,
            elapsed_ms,
            recorded_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn challenge_leaderboard_orders_and_pages() {
        let (mut store, user) = mem_store_with_user().await;
        // Three runs: (score, elapsed_ms) = (10, 5000), (2, 1000), (6, 3000).
        store.record_score(&challenge_score(user.id, "hard:1", 10, 5000)).await.unwrap();
        store.record_score(&challenge_score(user.id, "hard:1", 2, 1000)).await.unwrap();
        store.record_score(&challenge_score(user.id, "hard:1", 6, 3000)).await.unwrap();

        let fastest = ScoreOrdering {
            metric: ScoreMetric::Time,
            direction: SortDirection::Ascending,
        };
        let slowest = ScoreOrdering {
            metric: ScoreMetric::Time,
            direction: SortDirection::Descending,
        };
        let highest = ScoreOrdering {
            metric: ScoreMetric::Score,
            direction: SortDirection::Descending,
        };

        let fast = store.challenge_leaderboard("hard:1", fastest, 10, 0, false).await.unwrap();
        assert_eq!(fast.iter().map(|e| e.entry.elapsed_ms).collect::<Vec<_>>(), vec![1000, 3000, 5000]);

        // Reversed direction surfaces the slowest first.
        let slow = store.challenge_leaderboard("hard:1", slowest, 10, 0, false).await.unwrap();
        assert_eq!(slow.iter().map(|e| e.entry.elapsed_ms).collect::<Vec<_>>(), vec![5000, 3000, 1000]);

        let high = store.challenge_leaderboard("hard:1", highest, 10, 0, false).await.unwrap();
        assert_eq!(high.iter().map(|e| e.entry.score).collect::<Vec<_>>(), vec![10, 6, 2]);
        // No usernames requested → none resolved.
        assert!(high.iter().all(|e| e.username.is_none()));

        // Paging: limit 1, offset 1 of fastest → the middle (3000 ms) run.
        let page = store.challenge_leaderboard("hard:1", fastest, 1, 1, false).await.unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].entry.elapsed_ms, 3000);

        // include_usernames=true joins the player's name in.
        let named = store.challenge_leaderboard("hard:1", highest, 10, 0, true).await.unwrap();
        assert!(named.iter().all(|e| e.username.as_deref() == Some("alice")));
    }

    #[tokio::test]
    async fn record_score_enforces_the_subject_invariant() {
        let (mut store, user) = mem_store_with_user().await;
        let mut both = challenge_score(user.id, "easy:1", 1, 100);
        both.maze_id = Some("m1".to_string()); // both subjects set → rejected
        assert!(store.record_score(&both).await.is_err());
        let mut neither = challenge_score(user.id, "easy:1", 1, 100);
        neither.challenge = None; // neither subject set → rejected
        assert!(store.record_score(&neither).await.is_err());
    }

    #[tokio::test]
    async fn delete_user_cascades_score_history() {
        let (mut store, user) = mem_store_with_user().await;
        store.record_score(&challenge_score(user.id, "easy:1", 1, 100)).await.unwrap();
        assert_eq!(store.user_history(user.id, 10, 0).await.unwrap().len(), 1);
        store.delete_user(user.id).await.unwrap();
        assert_eq!(store.user_history(user.id, 10, 0).await.unwrap().len(), 0);
    }

    #[tokio::test]
    async fn legacy_users_email_column_retire_is_idempotent() {
        // File-based SQLite so a second `SqlStore::new` against the same URL
        // re-opens the same database (in-memory `:memory:` is per-connection).
        // First open: migrations run, then `retire_legacy_users_email_column`
        // rebuilds `users` to drop the legacy column. Second open: same
        // function runs again, observes `email` is already gone via
        // `PRAGMA table_info('users')`, and short-circuits — no rebuild.
        //
        // `max_connections = 5` is deliberate: it matches the real
        // `SqlStoreConfig::default()` shape and exercises the multi-connection
        // case. SQLite caches the schema per connection; if the rebuild
        // statements were issued through `pool.execute(...)` instead of a
        // single acquired connection, `DROP TABLE users` could be on one
        // pool connection and `ALTER TABLE users_new RENAME TO users` on
        // another — the renaming connection would still see `users` in its
        // schema cache and fail with "there is already another table or
        // index with this name: users". This test would catch that
        // regression.
        let temp = tempfile::tempdir().expect("tempdir");
        let db_path = temp.path().join("retire-idempotent.db");
        let url = format!("sqlite:{}", db_path.to_string_lossy());
        let cfg = SqlStoreConfig {
            url: url.clone(),
            max_connections: 5,
            auto_create_database: true,
            ..SqlStoreConfig::default()
        };

        // First open — runs the rebuild for real.
        let store1 = SqlStore::new(cfg.clone()).await.expect("first open");
        // Sanity: `email` is gone after the first open.
        let still_has_email: Option<_> = sqlx::query(
            "SELECT 1 FROM pragma_table_info('users') WHERE name = 'email'",
        )
        .fetch_optional(&store1.pool)
        .await
        .expect("pragma probe");
        assert!(
            still_has_email.is_none(),
            "users.email must be gone after first SqlStore::new"
        );
        // Drop the first store so its connection is released before the second open.
        drop(store1);

        // Second open against the same file — should be a clean no-op.
        let store2 = SqlStore::new(cfg).await.expect("second open");
        let still_has_email: Option<_> = sqlx::query(
            "SELECT 1 FROM pragma_table_info('users') WHERE name = 'email'",
        )
        .fetch_optional(&store2.pool)
        .await
        .expect("pragma probe (second)");
        assert!(
            still_has_email.is_none(),
            "users.email must remain gone on subsequent SqlStore::new calls"
        );
    }

    /// SQL of `migrations/0003_user_emails_verified_reset.sql` embedded at
    /// compile time so the test runs the exact statement `sqlx::migrate!`
    /// applies — no copy-paste drift between this assertion and the
    /// production migration.
    const MIGRATION_0003_SQL: &str =
        include_str!("../migrations/0003_user_emails_verified_reset.sql");

    /// Pre/post snapshot of migration 0003 against a seeded fixture:
    /// admin user, OAuth-verified user, two plain users — all with
    /// `verified = 1`. After re-applying the migration SQL: admin and
    /// OAuth-matched rows stay verified; the two plain rows flip to
    /// `verified = 0, verified_at = NULL`.
    ///
    /// `SqlStore::new` runs migrations 0001/0002/0003 against an empty
    /// schema (so 0003 is a no-op there); the test then seeds with
    /// `verified = 1` rows and re-applies 0003's SQL, exercising the
    /// transformation against realistic state.
    #[tokio::test]
    async fn migration_0003_resets_verified_except_admin_and_oauth_matched() {
        let temp = tempfile::tempdir().expect("tempdir");
        let db_path = temp.path().join("verified-reset-test.db");
        let url = format!("sqlite:{}", db_path.to_string_lossy());
        let store = SqlStore::new(SqlStoreConfig {
            url,
            max_connections: 1,
            auto_create_database: true,
            ..SqlStoreConfig::default()
        })
        .await
        .expect("SqlStore::new");

        let admin_id = Uuid::new_v4().to_string();
        let oauth_id = Uuid::new_v4().to_string();
        let plain1_id = Uuid::new_v4().to_string();
        let plain2_id = Uuid::new_v4().to_string();

        // Insert four users; verified = 1 on every email.
        for (id, is_admin, username, email) in &[
            (&admin_id, 1, "admin", "admin@example.com"),
            (&oauth_id, 0, "alice", "alice@gmail.com"),
            (&plain1_id, 0, "bob", "bob@example.com"),
            (&plain2_id, 0, "carol", "carol@example.com"),
        ] {
            sqlx::query(
                "INSERT INTO users (id, is_admin, username, full_name, password_hash, api_key, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(*id)
            .bind(*is_admin)
            .bind(*username)
            .bind(*username)
            .bind("hash")
            .bind(Uuid::new_v4().to_string())
            .bind("2026-01-01T00:00:00.000Z")
            .execute(&store.pool)
            .await
            .expect("insert user");
            sqlx::query(
                "INSERT INTO user_emails (user_id, email, is_primary, verified, verified_at) \
                 VALUES (?, ?, 1, 1, ?)",
            )
            .bind(*id)
            .bind(*email)
            .bind("2026-01-01T00:00:00.000Z")
            .execute(&store.pool)
            .await
            .expect("insert user_emails");
        }
        // OAuth identity for the OAuth user; provider_email matches the
        // user's email, so the carve-out applies.
        sqlx::query(
            "INSERT INTO oauth_identities \
             (user_id, provider, provider_user_id, provider_email, linked_at, last_seen_at) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(&oauth_id)
        .bind("google")
        .bind("google-sub-1")
        .bind("alice@gmail.com")
        .bind("2026-01-01T00:00:00.000Z")
        .bind("2026-01-01T00:00:00.000Z")
        .execute(&store.pool)
        .await
        .expect("insert oauth_identities");

        // Re-apply migration 0003 SQL against the now-seeded fixture.
        sqlx::query(MIGRATION_0003_SQL)
            .execute(&store.pool)
            .await
            .expect("re-run migration 0003");

        // Verify each user's verified flag.
        for (id, expected_verified) in &[
            (&admin_id, 1i64),
            (&oauth_id, 1),
            (&plain1_id, 0),
            (&plain2_id, 0),
        ] {
            let row: (i64, Option<String>) = sqlx::query_as(
                "SELECT verified, verified_at FROM user_emails WHERE user_id = ?",
            )
            .bind(*id)
            .fetch_one(&store.pool)
            .await
            .expect("fetch user_email");
            assert_eq!(
                row.0, *expected_verified,
                "user {id}: verified = {} expected {expected_verified}",
                row.0
            );
            if *expected_verified == 0 {
                assert!(
                    row.1.is_none(),
                    "user {id}: verified_at should be NULL after reset, got {:?}",
                    row.1
                );
            } else {
                assert!(
                    row.1.is_some(),
                    "user {id}: verified_at should be preserved, got NULL"
                );
            }
        }
    }

    /// Idempotency for the SQL migration: re-running on already-flipped
    /// data leaves the rows untouched (the WHERE clause's NOT IN /
    /// NOT EXISTS conditions still match, but `verified` is already 0,
    /// so the UPDATE writes the same value — no functional change).
    #[tokio::test]
    async fn migration_0003_is_idempotent_when_re_run() {
        let temp = tempfile::tempdir().expect("tempdir");
        let db_path = temp.path().join("verified-reset-idempotent.db");
        let url = format!("sqlite:{}", db_path.to_string_lossy());
        let store = SqlStore::new(SqlStoreConfig {
            url,
            max_connections: 1,
            auto_create_database: true,
            ..SqlStoreConfig::default()
        })
        .await
        .expect("SqlStore::new");

        let id = Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO users (id, is_admin, username, full_name, password_hash, api_key, created_at) \
             VALUES (?, 0, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind("dave")
        .bind("dave")
        .bind("hash")
        .bind(Uuid::new_v4().to_string())
        .bind("2026-01-01T00:00:00.000Z")
        .execute(&store.pool)
        .await
        .expect("insert user");
        sqlx::query(
            "INSERT INTO user_emails (user_id, email, is_primary, verified, verified_at) \
             VALUES (?, ?, 1, 1, ?)",
        )
        .bind(&id)
        .bind("dave@example.com")
        .bind("2026-01-01T00:00:00.000Z")
        .execute(&store.pool)
        .await
        .expect("insert user_emails");

        // First re-application flips the row.
        sqlx::query(MIGRATION_0003_SQL)
            .execute(&store.pool)
            .await
            .expect("first re-run");
        let (verified, verified_at): (i64, Option<String>) = sqlx::query_as(
            "SELECT verified, verified_at FROM user_emails WHERE user_id = ?",
        )
        .bind(&id)
        .fetch_one(&store.pool)
        .await
        .expect("fetch after first run");
        assert_eq!(verified, 0);
        assert!(verified_at.is_none());

        // Second re-application is a no-op semantically.
        sqlx::query(MIGRATION_0003_SQL)
            .execute(&store.pool)
            .await
            .expect("second re-run");
        let (verified, verified_at): (i64, Option<String>) = sqlx::query_as(
            "SELECT verified, verified_at FROM user_emails WHERE user_id = ?",
        )
        .bind(&id)
        .fetch_one(&store.pool)
        .await
        .expect("fetch after second run");
        assert_eq!(verified, 0);
        assert!(verified_at.is_none());
    }

    /// Empty `user_emails` table → migration succeeds without error.
    /// Already exercised implicitly by `SqlStore::new` (which runs 0003
    /// against an empty schema), but a focused test makes the contract
    /// explicit.
    #[tokio::test]
    async fn migration_0003_succeeds_on_empty_user_emails_table() {
        let temp = tempfile::tempdir().expect("tempdir");
        let db_path = temp.path().join("verified-reset-empty.db");
        let url = format!("sqlite:{}", db_path.to_string_lossy());
        let store = SqlStore::new(SqlStoreConfig {
            url,
            max_connections: 1,
            auto_create_database: true,
            ..SqlStoreConfig::default()
        })
        .await
        .expect("SqlStore::new");
        // `SqlStore::new` already ran 0003; a manual re-run on an empty
        // table must also succeed cleanly.
        sqlx::query(MIGRATION_0003_SQL)
            .execute(&store.pool)
            .await
            .expect("re-run on empty table");
    }

    // ─────────────────────────────────────────────────────────────────────
    // max_maze_cells cap enforcement
    // ─────────────────────────────────────────────────────────────────────

    async fn new_sqlite_store() -> SqlStore {
        SqlStore::new(SqlStoreConfig {
            url: "sqlite::memory:".to_string(),
            max_connections: 1,
            auto_create_database: true,
            ..SqlStoreConfig::default()
        })
        .await
        .expect("create in-memory SqlStore")
    }

    async fn seed_owner(store: &mut SqlStore) -> User {
        let mut user = User {
            id: User::new_id(),
            is_admin: false,
            username: "owner".to_string(),
            full_name: "Maze Owner".to_string(),
            emails: vec![UserEmail::new_primary_verified("owner@example.com")],
            password_hash: "hash".to_string(),
            api_key: User::new_api_key(),
            logins: vec![],
            oauth_identities: vec![],
            deleted_at: None,
            created_at: chrono::Utc::now(),
            last_sign_in_at: None,
            avatar_updated_at: None,
        };
        store.create_user(&mut user).await.expect("seed owner");
        user
    }

    fn make_maze(name: &str, rows: usize, cols: usize) -> Maze {
        use data_model::MazeDefinition;
        let mut maze = Maze::new(MazeDefinition::new(rows, cols));
        maze.name = name.to_string();
        maze
    }

    #[tokio::test]
    async fn sql_store_max_maze_cells_returns_cap() {
        let store = new_sqlite_store().await;
        assert_eq!(store.max_maze_cells(), Some(MAX_MAZE_CELLS));
    }

    #[tokio::test]
    async fn sql_store_create_maze_accepts_at_cap() {
        let mut store = new_sqlite_store().await;
        let owner = seed_owner(&mut store).await;
        // 60 × 60 = 3,600 = MAX_MAZE_CELLS
        let mut maze = make_maze("at-cap", 60, 60);
        store
            .create_maze(&owner, &mut maze)
            .await
            .expect("at-cap create succeeds");
        assert!(!maze.id.is_empty());
    }

    #[tokio::test]
    async fn sql_store_create_maze_accepts_just_under_cap() {
        let mut store = new_sqlite_store().await;
        let owner = seed_owner(&mut store).await;
        // 60 × 59 = 3,540 < 3,600
        let mut maze = make_maze("under-cap", 60, 59);
        store
            .create_maze(&owner, &mut maze)
            .await
            .expect("under-cap create succeeds");
    }

    #[tokio::test]
    async fn sql_store_create_maze_rejects_over_cap() {
        let mut store = new_sqlite_store().await;
        let owner = seed_owner(&mut store).await;
        // 61 × 60 = 3,660 > 3,600
        let mut maze = make_maze("over-cap", 61, 60);
        let err = store
            .create_maze(&owner, &mut maze)
            .await
            .expect_err("over-cap create should fail");
        match err {
            Error::MazeHasTooManyCells { rows, cols, max } => {
                assert_eq!(rows, 61);
                assert_eq!(cols, 60);
                assert_eq!(max, MAX_MAZE_CELLS);
            }
            other => panic!("expected MazeHasTooManyCells, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn sql_store_create_maze_rejects_oversized_definition_from_overrides() {
        use data_model::{CellEntity, MazeDefinition, WallOverride, WallType};
        let mut store = new_sqlite_store().await;
        let owner = seed_owner(&mut store).await;
        // 30 × 30 = 900 cells — comfortably under the 3,600-cell cap — but
        // filling the cells with wall overrides inflates the serialised
        // definition far past the 16,000-byte column. This is exactly the case
        // the cell-count cap can't catch: the byte guard must reject it. Walls
        // carry no per-type count cap (unlike enemies / health / treasure /
        // keys / doors), so this isolates the byte guard as the only check that
        // can reject the maze.
        let mut grid = vec![vec!['W'; 30]; 30];
        grid[0][0] = 'S';
        grid[29][29] = 'F';
        let mut definition = MazeDefinition::from_vec(grid);
        for r in 0..30 {
            for c in 0..30 {
                if definition.grid[r][c] == 'W' {
                    definition.cell_entities.insert(
                        (r, c),
                        vec![CellEntity::Wall(WallOverride {
                            wall_type: Some(WallType::IronFence),
                        })],
                    );
                }
            }
        }
        let mut maze = Maze::new(definition);
        maze.name = "oversized-overrides".to_string();
        let err = store
            .create_maze(&owner, &mut maze)
            .await
            .expect_err("oversized definition should fail");
        match err {
            Error::MazeDefinitionTooLarge { bytes, max } => {
                assert!(bytes > max, "bytes {bytes} should exceed max {max}");
                assert_eq!(max, MAX_MAZE_DEFINITION_BYTES);
            }
            other => panic!("expected MazeDefinitionTooLarge, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn sql_store_update_maze_rejects_over_cap() {
        use data_model::MazeDefinition;
        let mut store = new_sqlite_store().await;
        let owner = seed_owner(&mut store).await;
        // Seed at half cap, then try to update to over cap.
        let mut maze = make_maze("resize-me", 50, 50);
        store
            .create_maze(&owner, &mut maze)
            .await
            .expect("seed create");
        maze.definition = MazeDefinition::new(70, 60); // 4,200 cells
        let err = store
            .update_maze(&owner, &mut maze)
            .await
            .expect_err("over-cap update should fail");
        match err {
            Error::MazeHasTooManyCells { rows, cols, max } => {
                assert_eq!(rows, 70);
                assert_eq!(cols, 60);
                assert_eq!(max, MAX_MAZE_CELLS);
            }
            other => panic!("expected MazeHasTooManyCells, got {other:?}"),
        }
    }
}
