-- Soft-delete column for the `users` table. `deleted_at IS NULL` marks
-- an active user; a populated timestamp marks a soft-deleted user that
-- the application layer treats as invisible. The supporting index
-- backs the `deleted_at IS NULL` filter applied by every find/get
-- path on the `users` table.
--
-- Schema rules per `0001_initial.sql`:
--   * `VARCHAR(32)` for the RFC 3339 millisecond UTC timestamp (text storage,
--     lex order = chronological order, portable across SQLite, PostgreSQL,
--     and MySQL via SQLx Any).
--   * No literal `DEFAULT` (TEXT/BLOB-affinity columns can't carry one
--     portably; nullable column without a default is the right shape for
--     "this user has not been deleted").
--   * No `IF NOT EXISTS` on `CREATE INDEX` (MySQL parser rejects it; SQLx
--     tracks applied migrations so the file runs at most once per
--     database).
--
-- Existing rows stay NULL.

ALTER TABLE users ADD COLUMN deleted_at VARCHAR(32);

CREATE INDEX idx_users_deleted_at ON users(deleted_at);
