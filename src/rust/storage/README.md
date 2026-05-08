# `storage` Crate

## Introduction

The `storage` crate is written in `Rust` and exposes structs, traits and functions for storing data objects (users, mazes, OAuth identities, login tokens).

Two backends are available, both implementing the same `Store` trait:

| Backend | Feature flag | When to use |
|:--------|:-------------|:------------|
| `FileStore` | (default — no flag) | Local dev, single-instance, no infrastructure beyond a writable directory. JSON files on disk. |
| `SqlStore`  | `sql-store` | SQLite, PostgreSQL, or MySQL via SQLx's `Any` driver. One implementation, all three engines, runtime selection via the connection URL. |

The choice between FileStore and SqlStore is a runtime config decision — both backends ship in the same binary when the application is built with `sql-store` enabled. See [`maze_web_server/README.md`](../maze_web_server/README.md) for the application-level configuration.

## Getting Started

### Build

FileStore only (default):
```
cargo build
```

FileStore + SqlStore (all three SQL engines together):
```
cargo build --features sql-store
```

The `sql-store` feature pulls in [`sqlx`](https://crates.io/crates/sqlx) v0.8 with the `sqlite`, `postgres`, and `mysql` drivers and the migration runner. There are no per-database sub-features — enabling `sql-store` unlocks all three. Driver selection happens at runtime via the connection URL passed to `SqlStore::new`, not at compile time.

### Testing

The full suite runs the same trait-contract scenarios against every backend.

#### Default — FileStore + SqlStore against in-memory SQLite

From within the `storage` directory:
```
cargo test --features sql-store
```

This runs:
- FileStore inline unit tests
- SqlStore inline unit tests (datetime helpers — gated by `sql-store`)
- Validation tests
- The contract suite against FileStore (`tests/file_store_contract.rs` — 113 scenarios)
- The contract suite against SqlStore over in-memory SQLite (`tests/sql_store_contract.rs` — 113 scenarios)
- Doc tests

Tests run in parallel — every FileStore test is rooted at its own `tempfile::TempDir`, and every SqlStore test creates its own in-memory SQLite, so there's no shared state to serialise around.

#### SqlStore against PostgreSQL

The contract suite runs against any backend SQLx supports when `DATABASE_URL` is set. For PostgreSQL via Docker:

```bash
# One-off setup
docker run --name maze-postgres -e POSTGRES_PASSWORD=pw -p 5432:5432 -d postgres:16
docker exec -it maze-postgres psql -U postgres -c "CREATE DATABASE maze_test;"

# Run the contract suite
DATABASE_URL=postgres://postgres:pw@localhost:5432/maze_test \
    cargo test --features sql-store --test sql_store_contract -- --test-threads=1
```

`--test sql_store_contract` scopes the run to just the SqlStore integration test binary; FileStore + unit/doc tests are skipped (they don't depend on `DATABASE_URL`). `--test-threads=1` here is needed only because PostgreSQL/MySQL backends share a single test database — the contract suite calls `store.empty()` between scenarios to keep them isolated.

#### SqlStore against MySQL

```bash
# One-off setup
docker run --name maze-mysql -e MYSQL_ROOT_PASSWORD=pw -p 3306:3306 -d mysql:8
docker exec -it maze-mysql mysql -uroot -ppw \
  -e "CREATE DATABASE maze_test CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;"

# Run the contract suite
DATABASE_URL=mysql://root:pw@localhost:3306/maze_test \
    cargo test --features sql-store --test sql_store_contract -- --test-threads=1
```

#### Smoke example

A standalone end-to-end exerciser is provided as a runnable example:

```
cargo run --features sql-store --example sql_store_smoke
```

Defaults to in-memory SQLite. Set `DATABASE_URL` to point at PostgreSQL or MySQL to exercise those backends. Creates two users (one with a password, one OAuth-only), creates a maze, exercises lookups and the cascade-delete path, and prints progress per step.

### Linting

The workspace-level `cargo clippy --all-targets` checks the `storage` crate with its default features only. The `sql-store`-gated tests and examples (e.g. `sql_store_smoke`, `verify_migration`) are skipped because they declare `required-features = ["sql-store"]`. Cover them with an explicit invocation:

```
cargo clippy --features sql-store --all-targets
```

Expected: zero errors, zero warnings.

### Benchmarking
No benchmarking tests are currently implemented for the crate.

### Generating Documentation
```
cargo doc --features sql-store --open
```

## FileStore data layout and migrations

`FileStore` writes one JSON file per record under `data_dir/`:

```
data_dir/
  .schema_version            single integer, written atomically (tempfile + rename)
  users/
    <uuid>/
      user.json              user record (multi-email shape)
      mazes/
        <maze-id>.json
  one_time_tokens/
    <token-id>.json          single-use, time-bounded token (password reset / invite / email verification)
  email_audit_log/
    <entry-id>.json          one row per email send attempt (intent + outcome)
```

`FileStore::new` runs two startup passes against `data_dir` in order:

1. **`migrate_users_dir`** — a one-shot, idempotent rewrite of any pre-multi-email `user.json` files into the current shape. New-shape files parse straight through and are left alone; legacy single-email files are rewritten and the original kept alongside as `user.json.bak`. Runs unconditionally on every startup.

2. **`apply_pending_migrations`** — the schema-versioned migration framework. Reads `<data_dir>/.schema_version` (defaulting to `0` if absent), runs every registered migration with a higher version in order, and writes the new version atomically **after each successful migration** so a failure mid-batch leaves the schema at the last successful step (not at zero).

The migration registry lives in `src/file_store_migration.rs`:

| Version | Effect |
|:--------|:-------|
| 1, 2    | No-ops. Align the FileStore counter with the SQL `0001_initial.sql` and `0002_user_emails.sql` migrations already applied to existing deployments. |
| 3       | `migrate_0003_user_emails_verified_reset` — for every non-admin user, sets `verified = false, verified_at = None` on each email **not** matched by an `oauth_identities[*].provider_email` for that user. Admin users are skipped wholesale. Counterpart to the SQL `0003_user_emails_verified_reset.sql` migration described below. |
| 4       | No-op. The matching SQL migration adds a `users.deleted_at` column. The FileStore data shape is updated by `#[serde(default, skip_serializing_if = "Option::is_none")]` on the new `User.deleted_at` field — existing `user.json` files round-trip without rewriting; new files written after version-4-applied include the field only when populated. |
| 5       | `migrate_0005_create_one_time_tokens_dir` — creates `<data_dir>/one_time_tokens/`. Each token is one file `<token-id>.json`; the FileStore `TokenStore` impl reads/writes via tempfile + rename. |
| 6       | `migrate_0006_create_email_audit_log_dir` — creates `<data_dir>/email_audit_log/`. One file per audit row keyed by id; the FileStore `EmailAuditLog` impl reads/writes via tempfile + rename. `purge_user` walks the directory and clears `recipient_user_id` / `triggered_by_user_id` on rows referencing the purged user — the FileStore counterpart to the SQL `ON DELETE SET NULL` FK behaviour. |
| 7       | No-op. The matching SQL migration adds an `error_message TEXT` column to `email_audit_log` for verbose upstream-error capture. The FileStore data shape is updated by `#[serde(default, skip_serializing_if = "Option::is_none")]` on the new `EmailAuditEntry.error_message` field — existing audit-row JSON files round-trip without rewriting; new files written after version-7-applied include the field only when populated. |

Behaviour properties:

- **Idempotent**: re-running a migration on already-migrated data has no effect (each migration's logic is a deterministic transform; `mutated` flags suppress unnecessary file rewrites).
- **Atomic per file**: every `user.json` rewrite uses tempfile + rename.
- **No silent downgrade**: a `.schema_version` value higher than the registry's max version returns a clear error rather than re-running migrations against a newer schema.
- **Schema version persists across restarts**: an existing deployment that's already at the current version sees the second-pass framework as a near-zero-cost check (read the file, compare, exit).

## SqlStore schema and migrations

The SqlStore schema is defined across the migration files in [`migrations/`](./migrations/). It creates seven tables:

| Table | Purpose |
|:------|:--------|
| `users` | User records with admin flag, username, full name, password hash, API key (added in `0001_initial.sql`), plus a nullable `deleted_at` soft-delete marker (added in `0004_users_soft_delete.sql`). The `email` column was retired post-`0002_user_emails.sql` by per-backend cleanup in `SqlStore::new` (`retire_legacy_users_email_column`) — portable column-drop on a `UNIQUE NOT NULL` column isn't expressible in a single migration file across SQLite, PostgreSQL, and MySQL |
| `user_emails` | Email addresses attached to a user — `email`, `is_primary`, `verified`, `verified_at` (added in `0002_user_emails.sql`). Globally unique on `email`; one row per user has `is_primary = 1`, enforced in application code |
| `user_logins` | Active and expired bearer-token login sessions, FK to `users` |
| `oauth_identities` | Provider-linked identities (Google, GitHub, Facebook), FK to `users` |
| `mazes` | Maze definitions (JSON), FK to owner `users` |
| `one_time_tokens` | Single-use, time-bounded tokens for password-reset / invite / email-verification flows (added in `0005_one_time_tokens.sql`). FK to `users` with `ON DELETE CASCADE`. Single-use enforcement is application-driven via `UPDATE ... WHERE consumed_at IS NULL`. |
| `email_audit_log` | Append-only log of every email send attempt (added in `0006_email_audit_log.sql`). Two FKs to `users` — `recipient_user_id` and `triggered_by_user_id` — both `ON DELETE SET NULL` so the audit history survives a hard-delete (`purge_user`) without re-identifying the user. Soft-delete leaves the FK untouched. |

Plus the standard SQLx migration tracking table `_sqlx_migrations`, created automatically.

`SqlStore::new` runs any pending migrations on startup. SQLx tracks applied migrations in `_sqlx_migrations` so subsequent runs are idempotent — the schema is set up exactly once per database.

The migration files in [`migrations/`](./migrations/):

| File | Effect |
|:-----|:-------|
| `0001_initial.sql` | Creates `users`, `user_logins`, `oauth_identities`, `mazes`. The `users.email UNIQUE NOT NULL` column from this migration is later retired by `retire_legacy_users_email_column` in `SqlStore::new` (post-`0002`). |
| `0002_user_emails.sql` | Creates `user_emails` and seeds it from each user's `users.email`. Each seeded row is `is_primary = 1, verified = 1, verified_at = NULL`. |
| `0003_user_emails_verified_reset.sql` | One-sweep flip from "verified by default" to "verification required". Sets `verified = 0, verified_at = NULL` on every `user_emails` row whose owning user is not an admin AND no matching `oauth_identities.provider_email` row exists for that (user, email) pair. |
| `0004_users_soft_delete.sql` | Adds `users.deleted_at VARCHAR(32)` (RFC 3339 timestamp, nullable) and the supporting `idx_users_deleted_at` index. `deleted_at IS NULL` marks an active user; a populated timestamp marks a soft-deleted user. The trait surface enforces the filter — see "Soft-delete behaviour" below. |
| `0005_one_time_tokens.sql` | Creates the `one_time_tokens` table (`id`, `user_id`, `purpose`, `target_email`, `created_at`, `expires_at`, `consumed_at`) plus `idx_one_time_tokens_user_id` and `idx_one_time_tokens_expires_at`. FK to `users(id)` with `ON DELETE CASCADE`. The trait `TokenStore` ([`store.rs`](./src/store.rs)) sits on top: `create_token`, `find_token` (filters expired), `consume_token` (race-free single-use via `UPDATE ... WHERE consumed_at IS NULL`), `purge_email_verification_tokens` (used by the verification re-send handler so re-issuing supersedes any prior token), `purge_expired`. |
| `0006_email_audit_log.sql` | Creates the `email_audit_log` table (`id`, `created_at`, `recipient_user_id`, `recipient_email`, `template_id`, `token_id`, `triggered_by_user_id`, `triggered_by_ip`, `provider`, `provider_message_id`, `outcome`, `error_class`) plus four lookup indexes. Two FKs to `users(id)` — `recipient_user_id` and `triggered_by_user_id`, both `ON DELETE SET NULL`. The trait `EmailAuditLog` ([`store.rs`](./src/store.rs)) provides `record_pending` (synchronous insert before the send), `update_outcome` (asynchronous flip to `accepted`/`failed`), `find_audit_entry`, and `find_recent_audit_entries_for_user`. The body of every send and any expansion containing a secret token is *deliberately not stored* — the log records intent and authorization, not credentials. |
| `0007_email_audit_log_error_message.sql` | Adds a nullable `error_message TEXT` column to `email_audit_log` for free-form diagnostic detail captured alongside `error_class` when a send fails (e.g. an Azure AD `AADSTS70011` body for token-mint failures, or the SMTP enhanced status response for SMTP send failures). `error_class` remains the stable, low-cardinality dashboard signal; `error_message` is the human-readable why. TEXT (not VARCHAR) because upstream bodies are unbounded — see [`0001_initial.sql`](./migrations/0001_initial.sql) for the no-literal-DEFAULT-on-TEXT rule that informs the column shape. |

### Soft-delete behaviour

`UserStore::delete_user(id)` performs a **soft-delete**: the `users` row is kept (so audit-log foreign keys stay valid) with `deleted_at` populated and `username` rewritten to `deleted-<uuid>` to free the original handle for reuse. Related rows that have no audit value are hard-deleted in the same call: `user_logins`, `oauth_identities`, `user_emails`, and the user's `mazes`. After the call, every read path (`get_user`, `get_users`, `get_admin_users`, `has_users`, `find_user_by_name`, `find_user_by_verified_email`, `find_user_by_api_key`, `find_user_by_login_id`, `find_user_by_oauth_identity`) treats the user as if it never existed by applying a `deleted_at IS NULL` filter.

Two additional methods round out the surface:

- `UserStore::purge_user(id)` — true hard-delete of the `users` row. Intended for retention / right-to-erasure flows where the soft-deleted row must also be cleared. Reachable on either an active or already-soft-deleted user.
- `UserStore::has_active_admin_user()` — `is_admin = true AND deleted_at IS NULL`. Used by startup so a soft-deleted lone admin doesn't prevent the default admin from being recreated on next launch.

The username scramble form `deleted-<uuid>` is 44 chars, fitting comfortably within the `VARCHAR(64)` cap on `users.username` regardless of the original username's length, and works identically on FileStore (where the scramble is written directly to `user.json`).

### Schema portability rules

*Validated against MySQL 8.4 (Docker `mysql:8` image, which currently resolves to 8.4.x) and SQLx 0.8. The rules are fragile to upgrades on either side — re-validate against the contract suite (`tests/sql_store_contract.rs`) when bumping either version.*

The schema is written to MySQL's strict subset so the same file applies cleanly across SQLite, PostgreSQL, and MySQL. Five MySQL-specific rules govern its shape (full rationale inline in the migration file):

1. **No literal `DEFAULT` on TEXT/BLOB columns.** MySQL error 1101. Defaults that *must* be supplied are emitted by application code on every INSERT.
2. **No bare TEXT in keyed columns.** Primary keys, unique indexes, and foreign keys all use `VARCHAR(N)`. MySQL error 1170 otherwise.
3. **No `IF NOT EXISTS` on `CREATE [UNIQUE] INDEX`.** MySQL error 1064. Unique constraints are inlined as column-level `UNIQUE`; non-unique helper indexes use plain `CREATE INDEX` (SQLx tracks applied migrations, so re-runs are not a concern).
4. **`is_admin` is `INTEGER` and read as `i32`.** SQLx 0.8's `Any` decoder for MySQL doesn't auto-widen INT4 to `i64`. PostgreSQL happens to auto-widen but MySQL doesn't, so we read as `i32` for portability.
5. **Every string column is `VARCHAR(N)`, not `TEXT`.** SQLx-Any classifies MySQL TEXT as `BLOB` (TEXT and BLOB share the wire type), breaking `String` decoding.

PostgreSQL and SQLite accept all five rules transparently. **New migrations must follow the same rules** — adding a column or table that violates them will surface only when the migration runs against MySQL.

### Placeholder translation

SQLx 0.8's `Any` driver does **not** auto-translate `?` placeholders to PostgreSQL's `$1, $2, …` form for raw `sqlx::query("...")` strings — that translation only happens through the compile-time `query!` / `query_as!` macros. `SqlStore` detects the backend at startup (`SqlBackend::from_url`) and runs a small `q(kind, sql)` helper that rewrites `?` to `$N` only for PostgreSQL. SQLite and MySQL accept `?` natively and pass through unchanged. This is invisible to callers — every query in `sql_store.rs` is wrapped in `q(...)`.

## Architecture note: one impl over `AnyPool`

`SqlStore` is a single struct over `sqlx::AnyPool` rather than three per-backend implementations (`PgSqlStore`/`MySqlSqlStore`/`SqliteSqlStore`). The strict-subset schema removes essentially all runtime divergence between backends, so the only place per-backend logic is needed is the placeholder translator (`q(kind, sql)`, ~10 lines). If a future feature genuinely needs backend-specific SQL (e.g. native upsert syntax, full-text search), the pattern is a local `match self.kind` block at the one call site rather than a new type — keeping the trade-off proportional to the divergence.
