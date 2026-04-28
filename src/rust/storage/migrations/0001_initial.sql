-- Initial schema for the SqlStore backend.
--
-- This file is portable across PostgreSQL, MySQL, and SQLite when applied via
-- SQLx's `Any` driver. All identifiers are TEXT (UUIDs serialised as 36-char
-- strings) to avoid per-backend native UUID type differences.
--
-- Timestamp design — TEXT (RFC 3339), not TIMESTAMP. Deliberate decision:
--   * `TIMESTAMP` semantics differ across backends:
--       - PostgreSQL: native 8-byte type, no timezone (TIMESTAMPTZ has tz).
--       - MySQL:      native 4-byte type, **hard ceiling at 2038-01-19** (32-bit
--                     Unix range). MySQL `DATETIME` would fix this but is not
--                     valid PostgreSQL syntax — breaks single-file portability.
--       - SQLite:     no native type. Falls under NUMERIC affinity, in practice
--                     stored as TEXT anyway.
--   * SQLx's chrono integration encodes `DateTime<Utc>` differently per driver
--     (PG: TIMESTAMPTZ, MySQL: DATETIME, SQLite: TEXT). There is no uniform
--     "datetime column" via the `Any` driver — every column choice still
--     requires explicit conversion somewhere.
--   * All current datetime comparisons happen in Rust (`User::contains_valid_login`
--     etc.), not SQL — so we lose nothing by using TEXT.
--   * Storage cost is negligible at this project's scale (~25 B vs ~8 B per
--     timestamp; foreseeable rows < 1 M = a few MB total).
--   * Mirrors what FileStore already writes (`"expires_at": "2026-04-28T15:30:00Z"`).
--
--   Format constraint: timestamps must be written in a CONSISTENT RFC 3339
--   shape (e.g. always millisecond precision + trailing `Z`). With consistent
--   format, lexicographic order = chronological order, so SQL-side range
--   queries still work portably:
--       WHERE created_at >= ? AND created_at < ?
--       ORDER BY last_seen_at DESC
--   What does NOT work with TEXT timestamps: SQL-side date arithmetic
--   (`+ INTERVAL`), `EXTRACT`, `DATE_TRUNC`. Compute bounds in Rust instead.
--   The `SqlStore` implementation pins a single format function (Step 2.3) so
--   every write goes through one canonical serialisation path.
--
-- Maze size: `mazes.definition` uses portable plain TEXT, which on MySQL caps
-- at ~64 KB per row. Per-deployment maze cell-count caps are enforced at the
-- application layer (handlers / UI / console), not in the schema.

-- Note on `is_admin INTEGER`: stored as 0/1, not BOOLEAN. SQLx's `Any` driver
-- in 0.8 does not bridge SQLite's BOOLEAN affinity (read fails with "Any
-- driver does not support the SQLite type Bool"). INTEGER is the lowest
-- common denominator that decodes uniformly across PostgreSQL, MySQL, and
-- SQLite. The SqlStore code converts to/from `bool` at the Rust boundary.
CREATE TABLE IF NOT EXISTS users (
    id            TEXT NOT NULL PRIMARY KEY,
    is_admin      INTEGER NOT NULL DEFAULT 0,
    username      TEXT NOT NULL,
    full_name     TEXT NOT NULL DEFAULT '',
    email         TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    api_key       TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_users_username ON users (username);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_email    ON users (email);
CREATE UNIQUE INDEX IF NOT EXISTS idx_users_api_key  ON users (api_key);

CREATE TABLE IF NOT EXISTS user_logins (
    id          TEXT NOT NULL PRIMARY KEY,
    user_id     TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    expires_at  TEXT NOT NULL,
    ip_address  TEXT,
    device_info TEXT,
    CONSTRAINT fk_user_logins_user_id
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_user_logins_user_id ON user_logins (user_id);

-- OAuth identities linked to a user. A user may have multiple identities (one
-- per provider). The `(provider, provider_user_id)` pair is globally unique
-- per the OAuth/OIDC contract. Provider names are matched case-insensitively
-- in queries via `LOWER(provider) = LOWER(?)`; provider_user_id is matched
-- exactly (it is an opaque stable id from the identity provider).
--
-- Design note: `provider` is stored inline as TEXT rather than normalised into
-- a separate `oauth_providers` lookup table with an integer FK. This is a
-- deliberate trade-off:
--   * Storage savings from a lookup table are negligible at this project's
--     scale (~5 providers, foreseeable users < 100k).
--   * The trait `UserStore::find_user_by_oauth_identity(provider: &str, ...)`
--     and the application code (config.toml, OAuth connectors) are already
--     keyed on the provider *name* string. Normalising would force a
--     name→id translation on every lookup with no observable benefit.
--   * Provider configuration (enabled, OIDC settings, display name) lives in
--     `config.toml` — the appropriate home for *config* versus *data* — so a
--     `oauth_providers` table would not unify provider metadata storage.
--   * Adding a provider is currently a config change; with a lookup table it
--     would also require a schema migration (or lazy insert), adding friction
--     for no clear gain.
--   * Migration path to a normalised design later remains straightforward:
--     backfill `oauth_providers` from `SELECT DISTINCT provider`, add an FK
--     column, drop the inline column. Not a one-way door.
CREATE TABLE IF NOT EXISTS oauth_identities (
    user_id          TEXT NOT NULL,
    provider         TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    provider_email   TEXT,
    linked_at        TEXT NOT NULL,
    last_seen_at     TEXT NOT NULL,
    PRIMARY KEY (provider, provider_user_id),
    CONSTRAINT fk_oauth_identities_user_id
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_oauth_identities_user_id ON oauth_identities (user_id);

CREATE TABLE IF NOT EXISTS mazes (
    id          TEXT NOT NULL PRIMARY KEY,
    owner_id    TEXT NOT NULL,
    name        TEXT NOT NULL,
    definition  TEXT NOT NULL,
    CONSTRAINT fk_mazes_owner_id
        FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT uq_mazes_owner_name UNIQUE (owner_id, name)
);

CREATE INDEX IF NOT EXISTS idx_mazes_owner_id ON mazes (owner_id);
