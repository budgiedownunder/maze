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
- The contract suite against FileStore (`tests/file_store_contract.rs` — 49 scenarios)
- The contract suite against SqlStore over in-memory SQLite (`tests/sql_store_contract.rs` — 49 scenarios)
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

## SqlStore schema and migrations

The SqlStore schema is defined across the migration files in [`migrations/`](./migrations/). It creates five tables:

| Table | Purpose |
|:------|:--------|
| `users` | User records with admin flag, username, full name, password hash, API key (added in `0001_initial.sql`). The `email` column was retired post-`0002_user_emails.sql` by per-backend cleanup in `SqlStore::new` (`retire_legacy_users_email_column`) — portable column-drop on a `UNIQUE NOT NULL` column isn't expressible in a single migration file across SQLite, PostgreSQL, and MySQL |
| `user_emails` | Email addresses attached to a user, with `is_primary`, `verified`, and `verified_at` (added in `0002_user_emails.sql`). Globally unique on `email`; one row per user has `is_primary = 1`, enforced in application code |
| `user_logins` | Active and expired bearer-token login sessions, FK to `users` |
| `oauth_identities` | Provider-linked identities (Google, GitHub, Facebook), FK to `users` |
| `mazes` | Maze definitions (JSON), FK to owner `users` |

Plus the standard SQLx migration tracking table `_sqlx_migrations`, created automatically.

`SqlStore::new` runs any pending migrations on startup. SQLx tracks applied migrations in `_sqlx_migrations` so subsequent runs are idempotent — the schema is set up exactly once per database.

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
