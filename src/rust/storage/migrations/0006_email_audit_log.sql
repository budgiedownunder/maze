-- Append-only log of every email send attempt — captures *intent and
-- authorization* so we can answer "did we send X to user Y?" without
-- depending on provider-side log retention. Provider-side delivery
-- telemetry (bounces / complaints / opens) is collected separately and
-- ingested via webhook in a future phase, complementing this log
-- without replacing it.
--
-- All schema rules from `0001_initial.sql` apply: VARCHAR(N) for every
-- string column, VARCHAR(32) for RFC 3339 timestamps, no `IF NOT
-- EXISTS` on `CREATE INDEX`, no literal `DEFAULT` on TEXT/BLOB-affinity
-- columns. Sizing: 36 for UUIDs, 254 for emails (RFC 5321 max), 64 for
-- enum-like discriminators (`template_id`, `provider`), 16 for the
-- coarse outcome enum (`pending`/`accepted`/`failed`), 64 for the
-- `error_class` taxonomy, 255 for the provider message id (long enough
-- for SES/Mailgun/Postmark).
--
-- **Two-stage outcome write.** A handler inserts the row synchronously
-- with `outcome = 'pending'` *before* the send is attempted; the
-- spawned send task then UPDATEs the same row to `accepted` (with
-- `provider_message_id`) or `failed` (with `error_class`) when the
-- provider responds. A crash between the insert and the update leaves
-- the row at `pending` — operationally distinguishable from "send
-- accepted but the world ended" via the audit query.
--
-- **Never store the rendered body or any expansion containing a secret
-- token (reset link, invite link, verification link).** Every column
-- below is either an opaque id, a coarse classifier, or the recipient
-- email — none of which are credentials by themselves. `token_id` is
-- the same secret-bearing id stored in `one_time_tokens`, so it
-- inherits the same protection envelope.
--
-- **`recipient_user_id` is nullable.** Populated for every legitimate
-- send flow (the recipient user is always known before the send). The
-- column is null for *one* narrow case: a password-reset *request* for
-- an email that doesn't match any user — no send happens, but we
-- record the request itself for reconnaissance-detection / rate-limit
-- forensics.
--
-- **FK is `ON DELETE SET NULL`, not CASCADE.** Under the soft-delete
-- default the `users` row survives and the FK stays valid. Under a
-- true purge (`UserStore::purge_user`) the user row is gone — `SET
-- NULL` preserves the *fact* of the send (the email address remains
-- in `recipient_email`) without re-identifying the user. That meets
-- the "right to erasure" bar while keeping the operational audit data
-- useful.
CREATE TABLE IF NOT EXISTS email_audit_log (
    id                    VARCHAR(36)  NOT NULL PRIMARY KEY,
    created_at            VARCHAR(32)  NOT NULL,
    recipient_user_id     VARCHAR(36),
    recipient_email       VARCHAR(254) NOT NULL,
    template_id           VARCHAR(64)  NOT NULL,
    token_id              VARCHAR(36),
    triggered_by_user_id  VARCHAR(36),
    provider              VARCHAR(64)  NOT NULL,
    provider_message_id   VARCHAR(255),
    outcome               VARCHAR(16)  NOT NULL,
    error_class           VARCHAR(64),
    CONSTRAINT fk_email_audit_log_recipient_user
        FOREIGN KEY (recipient_user_id) REFERENCES users(id) ON DELETE SET NULL,
    CONSTRAINT fk_email_audit_log_triggered_by
        FOREIGN KEY (triggered_by_user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE INDEX idx_email_audit_log_recipient_user ON email_audit_log (recipient_user_id);
CREATE INDEX idx_email_audit_log_created_at ON email_audit_log (created_at);
CREATE INDEX idx_email_audit_log_token ON email_audit_log (token_id);
CREATE INDEX idx_email_audit_log_provider_message_id ON email_audit_log (provider_message_id);
