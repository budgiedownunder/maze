-- Single-use, time-bounded tokens used by the password-reset, email-
-- verification, and invitation flows.
--
-- Schema rules from `0001_initial.sql` apply verbatim here: VARCHAR(N)
-- everywhere a string is stored, INTEGER for booleans (none on this
-- table), VARCHAR(32) for RFC 3339 timestamps, no `IF NOT EXISTS` on
-- `CREATE INDEX`, no literal `DEFAULT` on TEXT/BLOB-affinity columns.
--
-- Sizing follows the rest of the schema: 36 for UUIDs, 32 for RFC 3339
-- timestamps, 254 for emails (RFC 5321 max), 32 for the purpose
-- discriminator (room for "email_verification" plus headroom).
--
-- `purpose` is a free-form string at the schema level — the application
-- maps it to the `TokenPurpose` enum (`password_reset` |
-- `email_verification`). Application code is the single source of truth
-- for the variant set; portable enum types across PostgreSQL/MySQL/SQLite
-- via SQLx Any aren't expressible in one migration file.
--
-- `consumed_at` is nullable: a populated timestamp marks the token as
-- consumed. Single-use enforcement happens via
--   UPDATE one_time_tokens SET consumed_at = ?
--    WHERE id = ? AND consumed_at IS NULL
-- — the WHERE-NULL guard makes consumption race-free across backends.
--
-- `target_email` is nullable; populated only for `email_verification`
-- tokens (the specific user_emails row to flip on consumption). Stored
-- inline rather than as a FK to user_emails because user_emails has a
-- composite primary key and tokens often reference an address that the
-- user is in the process of adding/changing — keeping the email as
-- a string here decouples the token row's lifetime from any specific
-- user_emails row.
--
-- `ON DELETE CASCADE` to users — when a user is hard-deleted (via
-- `purge_user`), their pending tokens go with them. Soft-delete is
-- handled at the application layer (UserStore::delete_user explicitly
-- DELETEs from this table), since soft-delete updates `users.deleted_at`
-- rather than removing the row.
CREATE TABLE IF NOT EXISTS one_time_tokens (
    id            VARCHAR(36)  NOT NULL PRIMARY KEY,
    user_id       VARCHAR(36)  NOT NULL,
    purpose       VARCHAR(32)  NOT NULL,
    target_email  VARCHAR(254),
    created_at    VARCHAR(32)  NOT NULL,
    expires_at    VARCHAR(32)  NOT NULL,
    consumed_at   VARCHAR(32),
    CONSTRAINT fk_one_time_tokens_user_id
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_one_time_tokens_user_id ON one_time_tokens (user_id);
CREATE INDEX idx_one_time_tokens_expires_at ON one_time_tokens (expires_at);
