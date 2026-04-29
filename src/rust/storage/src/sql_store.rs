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
use crate::{validation::validate_user_fields, Error, MazeItem, Store};
use async_trait::async_trait;
use chrono::{DateTime, SecondsFormat, Utc};
use data_model::{Maze, OAuthIdentity, User, UserLogin};
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

        Ok(Self { pool, kind })
    }
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
        email: row.try_get("email").map_err(map_sqlx_err)?,
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
    email: &str,
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
    let by_email = sqlx::query(&q(
        kind,
        "SELECT id FROM users WHERE LOWER(email) = LOWER(?) AND id <> ?",
    ))
    .bind(email)
    .bind(&ignore)
    .fetch_optional(pool)
    .await
    .map_err(map_sqlx_err)?;
    if by_email.is_some() {
        return Err(Error::UserEmailExists());
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
                user.email = email.to_string();
                user.is_admin = true;
                user.password_hash = password_hash.to_string();
                self.create_user(&mut user).await?;
                Ok(user)
            }
            Err(error) => Err(error),
        }
    }

    async fn create_user(&mut self, user: &mut User) -> Result<(), Error> {
        user.id = User::new_id();
        user.api_key = User::new_api_key();
        validate_user_for_store(user)?;
        check_user_unique_fields(&self.pool, self.kind, &user.username, &user.email, Uuid::nil()).await?;

        sqlx::query(&q(
            self.kind,
            "INSERT INTO users (id, is_admin, username, full_name, email, password_hash, api_key) \
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        ))
        .bind(user.id.to_string())
        .bind(bool_to_int(user.is_admin))
        .bind(&user.username)
        .bind(&user.full_name)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.api_key.to_string())
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_err)?;

        insert_user_logins(&self.pool, self.kind, user.id, &user.logins).await?;
        insert_user_oauth_identities(&self.pool, self.kind, user.id, &user.oauth_identities).await?;
        Ok(())
    }

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

    async fn update_user(&mut self, user: &mut User) -> Result<(), Error> {
        if user.id == Uuid::nil() {
            return Err(Error::UserIdMissing());
        }
        validate_user_for_store(user)?;
        check_user_unique_fields(&self.pool, self.kind, &user.username, &user.email, user.id).await?;

        let result = sqlx::query(&q(
            self.kind,
            "UPDATE users SET is_admin = ?, username = ?, full_name = ?, email = ?, \
                              password_hash = ?, api_key = ? \
             WHERE id = ?",
        ))
        .bind(bool_to_int(user.is_admin))
        .bind(&user.username)
        .bind(&user.full_name)
        .bind(&user.email)
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

        insert_user_logins(&self.pool, self.kind, user.id, &user.logins).await?;
        insert_user_oauth_identities(&self.pool, self.kind, user.id, &user.oauth_identities).await?;
        Ok(())
    }

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

    async fn find_user_by_email(&self, email: &str) -> Result<User, Error> {
        let mut rows = sqlx::query(&q(
            self.kind,
            "SELECT * FROM users WHERE LOWER(email) = LOWER(?)",
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
}

// ─────────────────────────────────────────────────────────────────────────────
// MazeStore
// ─────────────────────────────────────────────────────────────────────────────

#[async_trait]
impl MazeStore for SqlStore {
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
}
