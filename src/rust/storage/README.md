# `storage` Crate

## Introduction

The `storage` crate is written in `Rust` and exposes structs, traits and functions for storing data objects (users, mazes, game definitions, game collections, OAuth identities, login tokens).

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
- Doc tests

plus the shared trait-contract suite against both FileStore and in-memory SQLite.

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
  email_audit_log/
    <entry-id>.json          one row per email send attempt (intent + outcome)
  featured_game_items.json   single ordered array of {kind, id} — the featured catalogue (present only when ≥1 item is curated)
  game_collections/
    <collection-id>/         one folder per collection
      collection.json        the collection (name / visibility + ordered items)
      shares.json            grantee-uuid list (present only when shared)
      image.png              collection image (present only when set)
  game_definitions/
    <definition-id>/         one folder per definition
      definition.json        the game definition (name / visibility + opaque config + seed)
      shares.json            grantee-uuid list (present only when shared)
      image.png              game image (present only when set)
  one_time_tokens/
    <token-id>.json          single-use, time-bounded token (password reset / invite / email verification)
  score_history/
    <entry-id>.json          one row per completed 3D run (score + elapsed time)
  users/
    <uuid>/
      avatar.png             avatar image (present only when set)
      mazes/
        <maze-id>.json
      user.json              user record (multi-email shape)
```

`FileStore::new` runs two startup passes against `data_dir` in order:

1. **`migrate_users_dir`** — a one-shot, idempotent rewrite of any pre-multi-email `user.json` files into the current shape. New-shape files parse straight through and are left alone; legacy single-email files are rewritten and the original kept alongside as `user.json.bak`. Runs unconditionally on every startup.

2. **`apply_pending_migrations`** — the schema-versioned migration framework. Reads `<data_dir>/.schema_version` (defaulting to `0` if absent), runs every registered migration with a higher version in order, and writes the new version atomically **after each successful migration** so a failure mid-batch leaves the schema at the last successful step (not at zero).

The pre-migration version also feeds `Manage::was_freshly_created()`: a store constructed over a version-`0` data dir (FileStore) or an unmigrated database — no rows in SQLx's `_sqlx_migrations` (SqlStore) — reports `true`, distinguishing a brand-new store from a reopened one. Callers use it to run one-time bootstrap seeding only on a genuinely fresh store, so deleted seed content is not resurrected on the next restart.

The migration registry lives in `src/file_store_migration.rs`:

| Version | Effect |
|:--------|:-------|
| 1, 2    | No-ops. Align the FileStore counter with the SQL `0001_initial.sql` and `0002_user_emails.sql` migrations already applied to existing deployments. |
| 3       | `migrate_0003_user_emails_verified_reset` — for every non-admin user, sets `verified = false, verified_at = None` on each email **not** matched by an `oauth_identities[*].provider_email` for that user. Admin users are skipped wholesale. Counterpart to the SQL `0003_user_emails_verified_reset.sql` migration described below. |
| 4       | No-op. The matching SQL migration adds a `users.deleted_at` column. The FileStore data shape is updated by `#[serde(default, skip_serializing_if = "Option::is_none")]` on the new `User.deleted_at` field — existing `user.json` files round-trip without rewriting; new files written after version-4-applied include the field only when populated. |
| 5       | `migrate_0005_create_one_time_tokens_dir` — creates `<data_dir>/one_time_tokens/`. Each token is one file `<token-id>.json`; the FileStore `TokenStore` impl reads/writes via tempfile + rename. |
| 6       | `migrate_0006_create_email_audit_log_dir` — creates `<data_dir>/email_audit_log/`. One file per audit row keyed by id; the FileStore `EmailAuditLog` impl reads/writes via tempfile + rename. `purge_user` walks the directory and clears `recipient_user_id` / `triggered_by_user_id` on rows referencing the purged user — the FileStore counterpart to the SQL `ON DELETE SET NULL` FK behaviour. |
| 7       | No-op. The matching SQL migration adds an `error_message VARCHAR(2000)` column to `email_audit_log` for verbose upstream-error capture. The FileStore data shape is updated by `#[serde(default, skip_serializing_if = "Option::is_none")]` on the new `EmailAuditEntry.error_message` field — existing audit-row JSON files round-trip without rewriting; new files written after version-7-applied include the field only when populated. Both stores truncate oversize values at write time via `data_model::truncate_email_audit_error_message` so a verbose upstream body never fails the audit write. |
| 8       | `migrate_0008_user_timestamps` — walks every `users/<uuid>/user.json` and writes the migration-run timestamp (captured once at the top of the migration function) into `created_at` for any file that lacks it. `last_sign_in_at` is backfilled to the most recent `logins[*].created_at` (the most accurate evidence of the user's last sign-in); files with no logins keep `last_sign_in_at` absent so the welcome-banner trigger `User::is_first_sign_in()` fires correctly on their first actual sign-in. `User.created_at` is non-nullable in the application struct so callers never have to handle the absent case. New users created from this migration onwards carry the real `Utc::now()` set at user creation. |
| 9       | `migrate_0009_create_score_history_dir` — creates `<data_dir>/score_history/`. One file per completed run keyed by id; the FileStore `ScoreStore` impl reads/writes via tempfile + rename and serves the leaderboards / personal history by scanning the directory. The SQL `ON DELETE CASCADE` on `score_history` is mirrored here as explicit deletes: `delete_user` / `purge_user` remove the user's own rows plus the boards of the user's deleted mazes, and `delete_maze` removes that maze's board. |
| 11      | `migrate_0011_create_game_definitions_dir` — creates the `<data_dir>/game_definitions/` parent directory used by the FileStore `GameStore` impl. Each definition owns an `<id>/` sub-folder (`definition.json` + optional `shares.json` / `image.png`), created lazily on write. **Version 10 is skipped** — the SQL `0010_user_avatars` migration has no FileStore directory counterpart (avatars ride each user's dir as `avatar.png`). `delete_user` removes the user's own definition folders, strips the user from every remaining definition's grantee list, and clears the `def:<id>` board(s) of the removed definitions. |
| 12      | `migrate_0012_create_game_collections_dir` — creates the `<data_dir>/game_collections/` parent directory used by the FileStore `GameStore` collection impl. Each collection owns an `<id>/` sub-folder (`collection.json` + optional `shares.json` / `image.png`), created lazily on write. **`delete_user` and `purge_user`** both remove the user's own collection folders and strip the user from every remaining collection's grantee list — game content sits at the data-dir root rather than under `users/<id>/`, so removing the user directory doesn't reach it, and a purge may run on a still-active user with no prior soft-delete. |
| 13      | No-op. Aligns the FileStore counter with the SQL `0013_featured_game_items.sql` migration, which creates the `featured_game_items` table. The FileStore keeps the featured catalogue in a single root file `featured_game_items.json` (an ordered `[{kind, id}]` array, the index being the `sort_order`), created lazily on the first feature — there is no directory to pre-create. The featured list is a faithful projection of the `Curated` visibility tier, maintained by the create/update/delete paths (append on curate, remove + recompact on un-curate or delete) behind the store write-lock. |
| 14      | No-op. The matching SQL migration adds a nullable `play_mode` column to `game_collections`. The FileStore data shape is updated by `#[serde(default)]` on the new `GameCollection.play_mode` field — existing `collection.json` files round-trip without rewriting (an absent value loads as the default `Arcade`). |

Behaviour properties:

- **Idempotent**: re-running a migration on already-migrated data has no effect (each migration's logic is a deterministic transform; `mutated` flags suppress unnecessary file rewrites).
- **Atomic per file**: every `user.json` rewrite uses tempfile + rename.
- **No silent downgrade**: a `.schema_version` value higher than the registry's max version returns a clear error rather than re-running migrations against a newer schema.
- **Schema version persists across restarts**: an existing deployment that's already at the current version sees the second-pass framework as a near-zero-cost check (read the file, compare, exit).

## SqlStore schema and migrations

The SqlStore schema is defined across the migration files in [`migrations/`](./migrations/) plus the per-backend blob tables created in `SqlStore::new`. It creates seventeen tables:

| Table | Purpose |
|:------|:--------|
| `email_audit_log` | Append-only log of every email send attempt (added in `0006_email_audit_log.sql`). Two FKs to `users` — `recipient_user_id` and `triggered_by_user_id` — both `ON DELETE SET NULL` so the audit history survives a hard-delete (`purge_user`) without re-identifying the user. Soft-delete leaves the FK untouched. |
| `featured_game_items` | The admin-ordered featured catalogue — one ordered list mixing definitions and collections (added in `0013_featured_game_items.sql`). Composite PK `(entity_kind, entity_id)`, `sort_order` INTEGER, index on `sort_order`. `entity_id` is **polymorphic with no FK** (it points at either `game_definitions` or `game_collections` per `entity_kind`), cleaned app-side like `game_collection_items.definition_id`. A faithful projection of the `Curated` visibility tier: `GameStore`'s create/update/delete paths append a row (`sort_order` = max+1, derived in-SQL) when an entity becomes `Curated` and remove it + recompact `sort_order` to a dense `0..n` when it stops being `Curated` or is deleted — all inside the same transaction as the entity's visibility change, so the `curated` flag and its featured row commit atomically. `reorder_featured_game_items` rewrites the order in one transaction; `list_featured_game_items` hydrates the curated defs + collections in `sort_order`. `reconcile_featured_game_items` is a startup catch-up that **appends** any curated def/collection missing from the table (name-ordered, defs first; idempotent) — the reconcile hook keeps the table in sync going forward, but content curated *before* the table existed (or dropped by a bulk reorder while still curated) needs this backfill. |
| `game_collection_images` | One row per collection holding its image bytes (`image_data` BLOB) keyed by `collection_id` (PK + FK `game_collections`, `ON DELETE CASCADE`). Pairs with the `game_collections.image_updated_at` marker (the "has an image" signal + cache-buster). **Not created by a migration file** — same per-backend BLOB rationale as `user_avatars`; created in `SqlStore::new` via `create_game_image_table`. |
| `game_collection_items` | The ordered members of a collection (added in `0012_game_collections.sql`). Composite PK `(collection_id, definition_id)`, `sort_order` INTEGER. FK `collection_id` `ON DELETE CASCADE`; **no FK on `definition_id`** — membership is presentation, so a reference the server can't resolve (e.g. a member the viewer may not see) is simply filtered out at display. Because there is no FK, nothing removes these rows automatically, so **every path that removes a game definition prunes it from the collections listing it** (re-compacting the survivors' `sort_order`) — `delete_game_definition`, and `delete_user` / `purge_user` for the departing user's games — keeping a collection's item count from outrunning the members it can show. |
| `game_collection_shares` | The grantee list for a `Shared` collection (added in `0012_game_collections.sql`). Composite PK `(collection_id, grantee_user_id)`; two FKs `ON DELETE CASCADE`. Also cascaded in app code (`delete_user` / `delete_game_collection`). |
| `game_collections` | An ordered, presentation-only grouping of game definitions (added in `0012_game_collections.sql`), FK to owner `users` `ON DELETE CASCADE`. Carries name / `description` / `image_updated_at` marker / `visibility` / `play_mode` (`arcade` \| `campaign`, added nullable in `0014_game_collection_play_mode.sql`) + timestamps; `UNIQUE(owner_id, name)`. Members live in `game_collection_items`, share grants in `game_collection_shares`. No leaderboard of its own — boards are per-definition. |
| `game_definition_images` | One row per definition holding its image bytes (`image_data` BLOB) keyed by `definition_id` (PK + FK `game_definitions`, `ON DELETE CASCADE`). Pairs with the `game_definitions.image_updated_at` marker. **Not created by a migration file** — same per-backend BLOB rationale as `user_avatars`; created in `SqlStore::new` via `create_game_image_table` (the shared helper both image tables use). |
| `game_definition_shares` | The grantee list for a `Shared` game definition (added in `0011_game_definitions.sql`). Composite PK `(definition_id, grantee_user_id)`; two FKs `ON DELETE CASCADE` (to `game_definitions` and `users`). Also cascaded in app code (`delete_user` / `delete_game_definition`) for uniform cross-backend behaviour. |
| `game_definitions` | Parametric 3D game definitions (added in `0011_game_definitions.sql`), FK to owner `users` `ON DELETE CASCADE`. First-class columns the store queries on (`owner_id`, `name`, `visibility`) + the model's presentation/marker fields (`description`, `image_updated_at`) + `seed` (BIGINT), `rotation`, and the opaque `config VARCHAR(4000)`. `UNIQUE(owner_id, name)`. No maze grid — the game is regenerated from `seed`; the image *bytes* are not stored here (only the `image_updated_at` marker), matching `users.avatar_updated_at`. `config` is only 4000 (not 16000): the config is ~1–2 KB of scalar knobs and this table's many columns must sum under MySQL's per-row limit. |
| `mazes` | Maze definitions (JSON), FK to owner `users`. The `definition` column holds the whole serialised `Maze`, which may carry an optional `game_settings` object. |
| `oauth_identities` | Provider-linked identities (Google, GitHub, Facebook), FK to `users` |
| `one_time_tokens` | Single-use, time-bounded tokens for password-reset / invite / email-verification flows (added in `0005_one_time_tokens.sql`). FK to `users` with `ON DELETE CASCADE`. Single-use enforcement is application-driven via `UPDATE ... WHERE consumed_at IS NULL`. |
| `score_history` | One row per completed 3D run (added in `0009_score_history.sql`); serves the leaderboards (per-maze, per-curated-challenge) and personal history. Dual-keyed subject — exactly one of `maze_id` (FK `mazes`, `ON DELETE CASCADE`) or `challenge` (a `"<difficulty>:<seed>"` string, no FK) — plus `user_id` (the *player*, FK `users`, `ON DELETE CASCADE`). Both FK cascades are also enforced in app code (the delete paths `DELETE FROM score_history` explicitly, mirroring FileStore). |
| `user_avatars` | One row per user holding the avatar image bytes (`image_data` BLOB) keyed by `user_id` (PK + FK `users`, `ON DELETE CASCADE`). The companion marker `users.avatar_updated_at` (added in `0010_user_avatars.sql`) is both the "has an avatar" signal and the cache-buster. **Not created by a migration file** — its binary column type has no portable spelling across the three backends (PostgreSQL `BYTEA`, MySQL `LONGBLOB`, SQLite `BLOB`), so it is created per-backend in `create_user_avatars_table` (`sql_store.rs`), run from `SqlStore::new` after the portable migrations — the same pattern `retire_legacy_users_email_column` uses. The avatar *value* round-trips uniformly through SQLx-Any (`Vec<u8>` ⇄ blob on every driver); only the table DDL is per-backend. The FK cascade is also issued explicitly in `empty`, matching the `score_history` backstop. The leaderboard reads (`maze_leaderboard` / `challenge_leaderboard`) resolve each player's `avatar_updated_at` alongside `username` via the same `users` lookup, so `ScoreboardEntry` carries it and a board row can show the player's avatar or the placeholder without an extra round-trip. |
| `user_emails` | Email addresses attached to a user — `email`, `is_primary`, `verified`, `verified_at` (added in `0002_user_emails.sql`). Globally unique on `email`; one row per user has `is_primary = 1`, enforced in application code |
| `user_logins` | Active and expired bearer-token login sessions, FK to `users` |
| `users` | User records with admin flag, username, full name, password hash, API key (added in `0001_initial.sql`), a nullable `deleted_at` soft-delete marker (added in `0004_users_soft_delete.sql`), and `created_at` / `last_sign_in_at` timestamps (added in `0008_user_timestamps.sql`; both nullable in the column but `created_at` is non-nullable in the application struct via the migration's epoch backfill). The `email` column was retired post-`0002_user_emails.sql` by per-backend cleanup in `SqlStore::new` (`retire_legacy_users_email_column`) — portable column-drop on a `UNIQUE NOT NULL` column isn't expressible in a single migration file across SQLite, PostgreSQL, and MySQL |

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
| `0007_email_audit_log_error_message.sql` | Adds a nullable `error_message VARCHAR(2000)` column to `email_audit_log` for free-form diagnostic detail captured alongside `error_class` when a send fails (e.g. an Azure AD `AADSTS70011` body for token-mint failures, or the SMTP enhanced status response for SMTP send failures). `error_class` remains the stable, low-cardinality dashboard signal; `error_message` is the human-readable why. VARCHAR (not bare TEXT) per the rule in [`0001_initial.sql`](./migrations/0001_initial.sql) — SQLx-Any classifies MySQL TEXT as BLOB and breaks `Option<String>` decoding. Sized at 2000 (~8 KB at utf8mb4): well above the AAD JSON / SMTP responses this column actually stores, but small enough to leave ~24 KB of row-size headroom for future columns. The store layer truncates oversize values at write time via `data_model::truncate_email_audit_error_message` (with the `…[truncated]` marker), so an unusually verbose upstream body never fails the audit write — the audit row is only useful if it always lands. |
| `0008_user_timestamps.sql` | Adds `users.created_at VARCHAR(32)` (RFC 3339 timestamp) and `users.last_sign_in_at VARCHAR(32)`. Both are added nullable for portability (MySQL rejects `ALTER TABLE ... ADD COLUMN ... NOT NULL` without a literal `DEFAULT` on a populated table). The backfill UPDATEs aren't in the static SQL file — they live in `backfill_user_timestamps_if_null` in `sql_store.rs` and run from `SqlStore::new` after `sqlx::migrate!()`. `created_at` is unconditionally backfilled with `Utc::now()` captured at startup (non-null required by the app, no more accurate value available). `last_sign_in_at` is backfilled to the timestamp of the user's most recent login row — the most accurate evidence we have of when they last signed in. The backfill iterates `(user_id, MAX(created_at))` from `user_logins` in Rust and issues one parameterised UPDATE per user rather than a single `UPDATE … (correlated subquery) WHERE …`; the correlated form is rejected by PostgreSQL with `syntax error at or near "WHERE"` despite being accepted by SQLite and MySQL. Users with no `user_logins` row stay at NULL so the welcome-banner trigger fires correctly on their first actual sign-in. The application's `User.created_at` is non-nullable; new users carry the real `Utc::now()` set at creation. `User.last_sign_in_at` is `Option<DateTime<Utc>>`; the welcome-banner trigger is `User::is_first_sign_in()` = `last_sign_in_at.is_none() && logins.is_empty()` (captured before the handler flips either field). |
| `0009_score_history.sql` | Creates the `score_history` table (`id`, `user_id`, `maze_id`, `challenge`, `score BIGINT`, `elapsed_ms BIGINT`, `recorded_at`) plus four lookup indexes (`maze_id`, `challenge`, `user_id`, `recorded_at`). FKs `user_id` → `users(id)` and `maze_id` → `mazes(id)`, both `ON DELETE CASCADE`. The trait `ScoreStore` ([`store.rs`](./src/store.rs)) provides `record_score` (one row per won run), the paged board/history reads (`maze_leaderboard`, `challenge_leaderboard`, `user_history` — each `limit` + `offset`), and `completed_challenges(user_id, &[challenge])` — the subset of the given challenges the user has scored on, in one `SELECT DISTINCT … WHERE user_id = ? AND challenge IN (…)` (FileStore scans the user's rows), used to derive campaign progress without paging the whole history. Board ordering is a metric (time / score) plus a direction; the secondary metric + `recorded_at` / `id` tie-breaks stay fixed. The two board reads take an `include_usernames` flag and return `ScoreboardEntry { entry, username }` — the backend owns username resolution (SqlStore joins `users` in the board query; FileStore reads the player files), so callers never N+1. `user_history` carries no username (every row is the caller). `score` / `elapsed_ms` are `BIGINT` (the engine score is `u64`, stored as `i64`). The cascade is also done in app code (the delete paths `DELETE FROM score_history` explicitly — the FK is a backstop, uniform across SQLite/PostgreSQL/MySQL/FileStore). |
| `0010_user_avatars.sql` | Adds `users.avatar_updated_at VARCHAR(32)` (nullable RFC 3339 timestamp) — the "has an avatar" signal + cache-buster, kept off the hot path so loading a user never drags the image bytes. The companion `user_avatars` table (the `image_data` BLOB) is deliberately not created in this file as its binary column type has no portable spelling across the three backends (PostgreSQL `BYTEA` / MySQL `LONGBLOB` / SQLite `BLOB`).
| `0011_game_definitions.sql` | Creates `game_definitions` (id, owner_id FK→`users` CASCADE, name, description, image_updated_at, visibility, seed BIGINT, rotation, config VARCHAR(4000), created_at, updated_at; `UNIQUE(owner_id, name)` + indexes on owner_id / visibility) and `game_definition_shares` (composite PK `(definition_id, grantee_user_id)`, both FKs CASCADE, index on grantee_user_id). All-VARCHAR so a single portable file applies to every backend — the image *bytes* are not a column here (only the `image_updated_at` marker); they live in the separate per-backend `game_definition_images` BLOB table, created in `SqlStore::new` like `user_avatars`. The trait `GameStore` ([`store.rs`](./src/store.rs)) sits on top: owner-scoped create/update/delete/grant/revoke, `get_game_definition` by id, `get_game_definitions_for_owner`, and the composed **`get_visible_game_definitions(viewer, limit, offset)`** — the server's list feed, which evaluates the `owner ∨ public/curated ∨ EXISTS share` predicate + `ORDER BY LOWER(name), id` + `LIMIT/OFFSET` in one query (the OR-predicate returns each row once, so no dedup). `config` is byte-capped at `MAX_GAME_DEFINITION_CONFIG_BYTES` (the column width) on write, mirroring `mazes.definition`. |
| `0012_game_collections.sql` | Creates `game_collections` (id, owner_id FK→`users` CASCADE, name, description, image_updated_at, visibility, timestamps; `UNIQUE(owner_id, name)` + indexes on owner_id / visibility), `game_collection_items` (composite PK `(collection_id, definition_id)`, `sort_order` INTEGER, FK `collection_id` CASCADE, **no FK on `definition_id`**), and `game_collection_shares` (composite PK `(collection_id, grantee_user_id)`, both FKs CASCADE, index on grantee_user_id). All-VARCHAR (no blob). `GameStore`'s collection methods sit on top: owner-scoped collection CRUD + membership reconcile (`set_game_collection_items` — replaces the whole ordered list in one operation) + share grant/revoke, `get_game_collections_for_owner`, and the composed paged `get_visible_game_collections(viewer, limit, offset)` (same predicate/paging shape as `get_visible_game_definitions`). |
| `0013_featured_game_items.sql` | Creates `featured_game_items` (composite PK `(entity_kind, entity_id)`, `sort_order` INTEGER, index on `sort_order`; **no FK on `entity_id`** — it is polymorphic across `game_definitions` / `game_collections`). All-VARCHAR keyed columns, no blob. Holds the admin-ordered featured catalogue as a faithful projection of the `Curated` tier. Every mutation derives `sort_order` **in-SQL** inside a single transaction (append via `INSERT … SELECT COALESCE(MAX(sort_order),-1)+1` wrapped in a derived table so MySQL materialises it; recompact via a single `ROW_NUMBER()` renumber `UPDATE` whose window-function derived table is likewise materialised on every backend), never read into app code and written back, so two concurrent transactions can't collide. `GameStore`'s create/update/delete keep it in sync; `reorder_featured_game_items` / `list_featured_game_items` drive it. |
| `0014_game_collection_play_mode.sql` | Adds `game_collections.play_mode VARCHAR(16)` (nullable) — how a collection is played once opened (`arcade` free-choice or `campaign` ordered). Added nullable with no literal `DEFAULT` (schema rule 1); no backfill — a pre-column row reads back as `NULL`, which `game_collection_from_row` maps through `PlayMode::from_wire_str` to the default `Arcade` (the same nullable-column-lenient-default spirit as `0004`/`0007`). New collections always populate it. |

### Soft-delete behaviour

`UserStore::delete_user(id)` performs a **soft-delete**: the `users` row is kept (so audit-log foreign keys stay valid) with `deleted_at` populated and `username` rewritten to `deleted-<uuid>` to free the original handle for reuse. Related rows that have no audit value are hard-deleted in the same call: `user_logins`, `oauth_identities`, `user_emails`, and the user's `mazes`. After the call, every read path (`get_user`, `get_users`, `get_admin_users`, `has_users`, `find_user_by_name`, `find_user_by_verified_email`, `find_user_by_api_key`, `find_user_by_login_id`, `find_user_by_oauth_identity`) treats the user as if it never existed by applying a `deleted_at IS NULL` filter.

Two additional methods round out the surface:

- `UserStore::purge_user(id)` — true hard-delete of the `users` row. Intended for retention / right-to-erasure flows where the soft-deleted row must also be cleared. Reachable on either an active or already-soft-deleted user.
- `UserStore::has_active_admin_user()` — `is_admin = true AND deleted_at IS NULL`. Used by startup so a soft-deleted lone admin doesn't prevent the default admin from being recreated on next launch.

The username scramble form `deleted-<uuid>` is 44 chars, fitting comfortably within the `VARCHAR(64)` cap on `users.username` regardless of the original username's length, and works identically on FileStore (where the scramble is written directly to `user.json`).

### Schema portability rules

*Validated against MySQL 8.4 (Docker `mysql:8` image, which currently resolves to 8.4.x) and SQLx 0.8. The rules are fragile to upgrades on either side — re-validate against the contract suite (`tests/sql_store_contract.rs`) when bumping either version.*

The schema is written to MySQL's strict subset so the same file applies cleanly across SQLite, PostgreSQL, and MySQL. Six MySQL-specific rules govern its shape (full rationale inline in the migration file):

1. **No literal `DEFAULT` on TEXT/BLOB columns.** MySQL error 1101. Defaults that *must* be supplied are emitted by application code on every INSERT.
2. **No bare TEXT in keyed columns.** Primary keys, unique indexes, and foreign keys all use `VARCHAR(N)`. MySQL error 1170 otherwise.
3. **No `IF NOT EXISTS` on `CREATE [UNIQUE] INDEX`.** MySQL error 1064. Unique constraints are inlined as column-level `UNIQUE`; non-unique helper indexes use plain `CREATE INDEX` (SQLx tracks applied migrations, so re-runs are not a concern).
4. **`is_admin` is `INTEGER` and read as `i32`.** SQLx 0.8's `Any` decoder for MySQL doesn't auto-widen INT4 to `i64`. PostgreSQL happens to auto-widen but MySQL doesn't, so we read as `i32` for portability.
5. **Every string column is `VARCHAR(N)`, not `TEXT`.** SQLx-Any classifies MySQL TEXT as `BLOB` (TEXT and BLOB share the wire type), breaking `String` decoding.
6. **A table's `VARCHAR` widths must sum under MySQL's 65,535-byte per-row limit.** Each `VARCHAR(N)` counts its full `N × 4` (utf8mb4) byte width toward the row, whatever is actually stored. One wide column is fine (`mazes.definition VARCHAR(16000)` = 64,000 bytes only just fits with `mazes`'s three other small columns), but a many-column table must budget: `game_definitions` sizes `config` at `VARCHAR(4000)` for exactly this reason. PostgreSQL/SQLite impose no equivalent limit, so a too-wide row surfaces only against MySQL — as error 1118 / "Row size too large", which SQLx reports as a partially-applied migration.

PostgreSQL and SQLite accept all six rules transparently. **New migrations must follow the same rules** — adding a column or table that violates them will surface only when the migration runs against MySQL.

**Binary (BLOB) columns have no portable DDL spelling.** PostgreSQL has only `BYTEA`, MySQL only `BLOB`/`LONGBLOB` (no `BYTEA`), and SQLite takes `BLOB` — there is no keyword common to all three, so a binary column *cannot* be declared in a static migration file applied verbatim to every backend. To overcome this, we create tables with blob columns per-backend in `SqlStore::new` (`create_user_avatars_table` for `user_avatars`, and the shared `create_game_image_table` for `game_definition_images` / `game_collection_images`), choosing the column type by `self.kind`. This is a DDL-only constraint: the binary *value* binds and reads back uniformly through SQLx-Any (`Vec<u8>`/`&[u8]` ⇄ `AnyValueKind::Blob`, mapped to the native `BYTEA`/`BLOB`/`BLOB` encode+decode on each driver), so the read/write code stays backend-agnostic.

### Placeholder translation

SQLx 0.8's `Any` driver does **not** auto-translate `?` placeholders to PostgreSQL's `$1, $2, …` form for raw `sqlx::query("...")` strings — that translation only happens through the compile-time `query!` / `query_as!` macros. `SqlStore` detects the backend at startup (`SqlBackend::from_url`) and runs a small `q(kind, sql)` helper that rewrites `?` to `$N` only for PostgreSQL. SQLite and MySQL accept `?` natively and pass through unchanged. This is invisible to callers — every query in `sql_store.rs` is wrapped in `q(...)`.

## Maze cell-count caps

Each `MazeStore` impl reports the maximum number of cells (`rows × cols`) it will accept on `create_maze` / `update_maze` via the trait method `max_maze_cells() -> Option<usize>`. `None` means the store imposes no cap. The cap is a property of the *storage backend*, not of the maze domain — the `maze` and `data_model` crates have no notion of size limits and remain unbounded.

| Backend       | `max_maze_cells()` | Why |
|:--------------|:-------------------|:----|
| `FileStore`   | `Some(10_000)`     | The filesystem imposes no row-size limit, so `10,000` (eg. a 100 x 100 grid) is chosen as a practical cap. |
| `SqlStore`    | `Some(3_600)`      | Bound by the `mazes.definition VARCHAR(16000)` column in [`migrations/0001_initial.sql`](./migrations/0001_initial.sql). With the existing JSON serialisation (`4·N·M + 2·N + 10` chars for an N-row × M-col grid) the 16,000-char column maxes out around 62×62 cells; 60×60 = 3,600 sits inside that with a margin. The same cap applies across SQLite, PostgreSQL, and MySQL — SQLite would ignore the column length declaration, but enforcing the cap uniformly avoids dev-vs-prod divergence when the same data set is later loaded under MySQL or PostgreSQL. |
| trait default | `None`             | Suits stub implementations and any future store with no practical size limit. Production stores override. |

The cell-count cap assumes plain single-character cells. Per-cell **entity overrides** (an enemy/health/key/door cell serialised as `[{"type":"E",…}]` rather than the bare `"E"`) can inflate individual cells well beyond that, so a maze can sit under the cell-count cap yet still overflow the column. `SqlStore` therefore also enforces an authoritative **byte cap** — `SqlStore::MAX_MAZE_DEFINITION_BYTES = 16_000`, matching the `mazes.definition VARCHAR(16000)` column — on the exact serialised string about to be written. An over-cap maze is refused with `Error::MazeDefinitionTooLarge { bytes, max }` (surfaced by the server as HTTP 422) before the database sees it, rather than being silently truncated. `FileStore` keeps only the cell-count cap (its JSON files have no column-width limit). An optional per-maze `game_settings` object (the 3D launch environment) rides the same serialised `Maze` blob, so it likewise counts toward this byte cap — negligible in practice, being a small fixed object relative to the grid.

Independently of the *size* caps above, both stores also cap the **number of items one user may own**, to keep the per-user list reads (`get_maze_items` / `get_game_definitions_for_owner` / `get_game_collections_for_owner`) bounded. Each is a single **product** limit shared across backends (unlike the backend-specific size caps), reported by a `max_*_per_user() -> Option<usize>` trait method and enforced on create (an over-cap save is refused with a dedicated `Error` variant, surfaced by the server as HTTP **409**):

| Entity | Constant | Cap | Reported by | Enforced in | Error |
|:---|:---|:--:|:---|:---|:---|
| Mazes | `MAX_MAZES_PER_USER` | 500 | `MazeStore::max_mazes_per_user` | `create_maze` | `MazeCountLimitReached` |
| Game definitions | `MAX_DEFINITIONS_PER_USER` | 500 | `GameStore::max_definitions_per_user` | `create_game_definition` | `GameDefinitionCountLimitReached` |
| Game collections | `MAX_COLLECTIONS_PER_USER` | 100 | `GameStore::max_collections_per_user` | `create_game_collection` | `GameCollectionCountLimitReached` |


## Maze object-count caps

Independently of the storage-size caps above, both stores enforce the `maze` crate's **per-type object caps** on `create_maze` / `update_maze`, so an authored maze can never carry more of a limited object than generation would place: at most `maze::MAX_ENEMY_COUNT` enemies, `maze::MAX_HEALTH_COUNT` health pickups and `maze::MAX_TREASURE_COUNT` treasure (`validate_maze_object_counts`), plus the combined `maze::MAX_TOTAL_FEATURES` keys + doors (`validate_maze_feature_count`). These are maze-domain limits — shared with the generator and the key-aware solver — not backend properties, so they hold identically across every store. An over-cap save is refused with `Error::MazeHasTooManyObjects { kind, count, max }` or `Error::MazeHasTooManyFeatures { … }` (surfaced by the server as HTTP 422) rather than persisted, keeping a hand-painted maze's in-game object count within the same budget generation respects (an unbounded treasure pile, say, would overwhelm a mobile GPU's per-chest lights and sparkles at play time).


## Architecture note: one impl over `AnyPool`

`SqlStore` is a single struct over `sqlx::AnyPool` rather than three per-backend implementations (`PgSqlStore`/`MySqlSqlStore`/`SqliteSqlStore`). The strict-subset schema removes essentially all runtime divergence between backends, so the only place per-backend logic is needed is the placeholder translator (`q(kind, sql)`, ~10 lines). If a future feature genuinely needs backend-specific SQL (e.g. native upsert syntax, full-text search), the pattern is a local `match self.kind` block at the one call site rather than a new type — keeping the trade-off proportional to the divergence.
