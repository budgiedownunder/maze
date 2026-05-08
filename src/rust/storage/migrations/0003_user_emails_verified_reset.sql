-- One-sweep flip from "every email verified by default" to
-- "verification required" — the policy switch this plan introduces.
--
-- For every existing `user_emails` row, set `verified = 0, verified_at = NULL`
-- UNLESS one of two carve-outs applies:
--   * The owning user is an admin (`users.is_admin = 1`).
--   * The owning user has an OAuth identity whose `provider_email` matches
--     this email exactly. `oauth_identities.provider_email` is the most
--     recent verified email observed from the provider — its presence is
--     itself evidence the provider attests to the user's ownership of
--     that address (see `data_model::OAuthIdentity` doc comment).
--
-- Idempotent: re-running on already-flipped data produces the same result
-- (admin/OAuth-matched rows are untouched; others stay at verified = 0).
--
-- Schema notes per `0001_initial.sql`:
--   * `verified` is INTEGER (0/1, not BOOLEAN — SQLx Any driver constraint).
--   * `verified_at` is VARCHAR(32) RFC 3339 timestamp; NULL on reset.
--   * `oauth_identities` does not carry an explicit `email_verified` flag;
--     `provider_email IS NOT NULL` already encodes that, so the carve-out
--     matches on the address alone.
UPDATE user_emails
SET verified = 0, verified_at = NULL
WHERE user_id NOT IN (SELECT id FROM users WHERE is_admin = 1)
  AND NOT EXISTS (
        SELECT 1 FROM oauth_identities oi
         WHERE oi.user_id = user_emails.user_id
           AND oi.provider_email = user_emails.email
      );
