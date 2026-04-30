-- Multi-email schema: normalise email out of `users` into a separate
-- `user_emails` table. Each row is `(user_id, email, is_primary, verified,
-- verified_at)`; exactly one row per user is `is_primary = 1` (enforced
-- in application code — see `storage::store::UserStore` impls).
--
-- All design rules from `0001_initial.sql` apply here verbatim:
--   * `VARCHAR(N)` for every text column (TEXT in indexed positions blows
--     up on MySQL).
--   * `INTEGER` for booleans (SQLite `BOOLEAN` decoder gap in SQLx Any).
--   * `VARCHAR(32)` for RFC 3339 timestamps (text storage, lex == chrono
--     when written through one canonical formatter).
--   * No `IF NOT EXISTS` on `CREATE INDEX` (MySQL parser rejects it).
--   * No literal `DEFAULT` on TEXT/BLOB-affinity columns (MySQL 8.0.13
--     allows expression defaults but the syntax isn't portable).
--
-- Sizing follows `0001_initial.sql`:
--   * 36   for UUIDs (user_id)
--   * 254  for emails (RFC 5321 max)
--   * 32   for RFC 3339 timestamps
--
-- `is_primary` invariant: the database does NOT enforce one-primary-per-user.
-- A partial unique index (`UNIQUE ... WHERE is_primary = 1`) is the obvious
-- mechanism but isn't portable across SQLx Any (PostgreSQL and SQLite support
-- it; MySQL does not until 8.0 generated columns + functional indexes, which
-- aren't reachable through the Any driver). The invariant is upheld by every
-- write path in `UserStore` instead.
--
-- Email uniqueness across all users mirrors today's `users.email UNIQUE`
-- constraint.
--
-- The original `users.email` column is **not** dropped in this migration:
-- portable column-drop on a `UNIQUE NOT NULL` column requires per-backend
-- DDL gymnastics (SQLite refuses with "cannot drop UNIQUE column"; PG and
-- MySQL each need different sequences to drop the implicit constraint
-- before the column). The application keeps `users.email` populated as
-- the primary email's address (write-through denormalisation) so the old
-- column remains valid for any tool that still queries it. A future
-- migration with per-backend SQL can retire `users.email` cleanly.
CREATE TABLE IF NOT EXISTS user_emails (
    user_id      VARCHAR(36)  NOT NULL,
    email        VARCHAR(254) NOT NULL,
    is_primary   INTEGER      NOT NULL DEFAULT 0,
    verified     INTEGER      NOT NULL DEFAULT 1,
    verified_at  VARCHAR(32),
    PRIMARY KEY (user_id, email),
    CONSTRAINT fk_user_emails_user_id
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT uq_user_emails_email UNIQUE (email)
);

CREATE INDEX idx_user_emails_user_primary ON user_emails (user_id, is_primary);

-- Migrate existing data: every user gets their current email moved to
-- user_emails as primary + verified. `verified_at` is left NULL — no
-- meaningful "when was this verified" instant exists for migrated rows.
-- Pre-existing logins are evidence enough that ownership is real.
INSERT INTO user_emails (user_id, email, is_primary, verified, verified_at)
SELECT id, email, 1, 1, NULL FROM users;
