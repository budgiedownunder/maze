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

use crate::store::{Manage, MazeStore, UserStore};
use crate::{
    validation::{validate_email_format, validate_user_fields},
    Error, MazeItem, Store,
};
use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, Utc};
use data_model::{Maze, OAuthIdentity, User, UserEmail, UserLogin};
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

        Ok(Self { pool, kind })
    }
}

/// Per-backend retirement of `users.email`. Runs every startup; naturally
/// idempotent. See `SqlStore::new` for context.
async fn retire_legacy_users_email_column(
    pool: &AnyPool,
    kind: SqlBackend,
) -> Result<(), Error> {
    match kind {
        SqlBackend::Postgres => {
            // Dropping the column also drops the implicit `users_email_key`
            // UNIQUE constraint and its supporting index in PG.
            sqlx::query("ALTER TABLE users DROP COLUMN IF EXISTS email")
                .execute(pool)
                .await
                .map_err(map_sqlx_err)?;
        }
        SqlBackend::MySql => {
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
            .map_err(map_sqlx_err)?
            .is_some();
            if has_index {
                sqlx::query("ALTER TABLE users DROP INDEX email")
                    .execute(pool)
                    .await
                    .map_err(map_sqlx_err)?;
            }
            let has_column = sqlx::query(
                "SELECT 1 FROM INFORMATION_SCHEMA.COLUMNS \
                 WHERE TABLE_SCHEMA = DATABASE() \
                   AND TABLE_NAME = 'users' \
                   AND COLUMN_NAME = 'email'",
            )
            .fetch_optional(pool)
            .await
            .map_err(map_sqlx_err)?
            .is_some();
            if has_column {
                sqlx::query("ALTER TABLE users DROP COLUMN email")
                    .execute(pool)
                    .await
                    .map_err(map_sqlx_err)?;
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
            let mut conn = pool.acquire().await.map_err(map_sqlx_err)?;

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
            .map_err(map_sqlx_err)?
            .is_some();
            let users_new_exists = sqlx::query(
                "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'users_new'",
            )
            .fetch_optional(&mut *conn)
            .await
            .map_err(map_sqlx_err)?
            .is_some();

            if users_new_exists && !users_exists {
                // Recover: a previous SqlStore::new dropped `users` but
                // failed before renaming `users_new` (e.g. older code that
                // ran the rebuild across pool connections). Just complete
                // the rename.
                sqlx::query("ALTER TABLE users_new RENAME TO users")
                    .execute(&mut *conn)
                    .await
                    .map_err(map_sqlx_err)?;
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
                .map_err(map_sqlx_err)?
                .is_some();
            if !has_email_column {
                return Ok(());
            }

            // Drop any stale `users_new` left behind by a previous aborted
            // attempt before starting fresh — guarantees the CREATE below
            // doesn't collide.
            if users_new_exists {
                sqlx::query("DROP TABLE users_new")
                    .execute(&mut *conn)
                    .await
                    .map_err(map_sqlx_err)?;
            }

            // Disable FK enforcement for the duration of the rebuild —
            // user_logins / oauth_identities / mazes / user_emails all
            // reference `users(id)` and would error mid-rebuild. SQLite
            // resolves FKs by name so the references survive the rename.
            sqlx::query("PRAGMA foreign_keys = OFF")
                .execute(&mut *conn)
                .await
                .map_err(map_sqlx_err)?;
            sqlx::query(
                "CREATE TABLE users_new (\
                    id            VARCHAR(36)  NOT NULL PRIMARY KEY,\
                    is_admin      INTEGER      NOT NULL DEFAULT 0,\
                    username      VARCHAR(64)  NOT NULL UNIQUE,\
                    full_name     VARCHAR(255) NOT NULL,\
                    password_hash VARCHAR(255) NOT NULL,\
                    api_key       VARCHAR(36)  NOT NULL UNIQUE\
                )",
            )
            .execute(&mut *conn)
            .await
            .map_err(map_sqlx_err)?;
            sqlx::query(
                "INSERT INTO users_new (id, is_admin, username, full_name, password_hash, api_key) \
                 SELECT id, is_admin, username, full_name, password_hash, api_key FROM users",
            )
            .execute(&mut *conn)
            .await
            .map_err(map_sqlx_err)?;
            sqlx::query("DROP TABLE users")
                .execute(&mut *conn)
                .await
                .map_err(map_sqlx_err)?;
            sqlx::query("ALTER TABLE users_new RENAME TO users")
                .execute(&mut *conn)
                .await
                .map_err(map_sqlx_err)?;
            sqlx::query("PRAGMA foreign_keys = ON")
                .execute(&mut *conn)
                .await
                .map_err(map_sqlx_err)?;
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
            "INSERT INTO users (id, is_admin, username, full_name, password_hash, api_key) \
             VALUES (?, ?, ?, ?, ?, ?)",
        ))
        .bind(user.id.to_string())
        .bind(bool_to_int(user.is_admin))
        .bind(&user.username)
        .bind(&user.full_name)
        .bind(&user.password_hash)
        .bind(user.api_key.to_string())
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
                              password_hash = ?, api_key = ? \
             WHERE id = ?",
        ))
        .bind(bool_to_int(user.is_admin))
        .bind(&user.username)
        .bind(&user.full_name)
        .bind(&user.password_hash)
        .bind(user.api_key.to_string())
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
        let row = sqlx::query(&q(self.kind, "SELECT * FROM users WHERE id = ?"))
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
            "SELECT * FROM users WHERE LOWER(username) = LOWER(?)",
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
             WHERE LOWER(ue.email) = LOWER(?) AND ue.verified <> 0",
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
        let mut rows = sqlx::query(&q(self.kind, "SELECT * FROM users WHERE api_key = ?"))
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
             WHERE l.id = ? AND l.expires_at > ?",
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
             WHERE LOWER(oi.provider) = LOWER(?) AND oi.provider_user_id = ?",
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
        let rows = sqlx::query(&q(self.kind, "SELECT * FROM users ORDER BY username"))
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
            "SELECT * FROM users WHERE is_admin <> 0 ORDER BY username",
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
        let row = sqlx::query(&q(self.kind, "SELECT 1 FROM users LIMIT 1"))
            .fetch_optional(&self.pool)
            .await
            .map_err(map_sqlx_err)?;
        Ok(row.is_some())
    }

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
        Ok(())
    }

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
        let definition_json = serde_json::to_string(&maze)?;
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
        for sql in [
            "DELETE FROM user_logins",
            "DELETE FROM oauth_identities",
            "DELETE FROM mazes",
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
}
