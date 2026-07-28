-- Parametric 3D game definitions and their per-definition share grants.
--
-- A game definition stores no maze grid — the game is regenerated from its
-- `seed` — so it carries only a small opaque `config` blob plus first-class
-- columns the store queries on (owner, name, visibility) and the model's own
-- presentation/marker fields (description, image_updated_at). `config` is
-- byte-capped in app code at `MAX_GAME_DEFINITION_CONFIG_BYTES` (= the 4000
-- column width) on create/update, mirroring the `mazes.definition` guard.
--
-- All schema rules from `0001_initial.sql` apply: VARCHAR(N) for every string
-- column, VARCHAR(32) for RFC 3339 timestamps, no `IF NOT EXISTS` on
-- `CREATE INDEX`, unique constraints inlined as table-level CONSTRAINTs. Sizing:
-- 36 for UUIDs, 16 for the `visibility` / `rotation` enum strings. `seed` is
-- BIGINT (a `u64` stored as `i64`, round-tripping via the bit pattern). `config`
-- is only 4000 (not the 16000 the `mazes.definition` grid needs): the config is
-- ~1–2 KB of scalar knobs, and this table's many columns must sum under MySQL's
-- 65,535-byte per-row limit (a VARCHAR(N) counts its full N×4 utf8mb4 width).
-- `image_updated_at` is the marker only (the "has an image" signal + cache
-- buster); the image *bytes* live in a separate per-backend BLOB table added
-- later, exactly as `user_avatars` is to `users.avatar_updated_at`.
--
-- `game_definition_shares` is the grantee list for a `Shared` definition. Both
-- FKs are ON DELETE CASCADE (a deleted definition or user drops its grants),
-- but the store also cascades in app code so the behaviour is uniform across
-- FileStore, SQLite (FK enforcement is pragma-gated), PostgreSQL, and MySQL.
CREATE TABLE IF NOT EXISTS game_definitions (
    id                VARCHAR(36)    NOT NULL PRIMARY KEY,
    owner_id          VARCHAR(36)    NOT NULL,
    name              VARCHAR(255)   NOT NULL,
    description       VARCHAR(1024),
    image_updated_at  VARCHAR(32),
    visibility        VARCHAR(16)    NOT NULL,
    seed              BIGINT         NOT NULL,
    rotation          VARCHAR(16)    NOT NULL,
    config            VARCHAR(4000)  NOT NULL,
    created_at        VARCHAR(32)    NOT NULL,
    updated_at        VARCHAR(32)    NOT NULL,
    CONSTRAINT fk_game_definitions_owner
        FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT uq_game_definitions_owner_name UNIQUE (owner_id, name)
);

CREATE INDEX idx_game_definitions_owner ON game_definitions (owner_id);
CREATE INDEX idx_game_definitions_visibility ON game_definitions (visibility);

CREATE TABLE IF NOT EXISTS game_definition_shares (
    definition_id     VARCHAR(36) NOT NULL,
    grantee_user_id   VARCHAR(36) NOT NULL,
    PRIMARY KEY (definition_id, grantee_user_id),
    CONSTRAINT fk_game_definition_shares_definition
        FOREIGN KEY (definition_id) REFERENCES game_definitions(id) ON DELETE CASCADE,
    CONSTRAINT fk_game_definition_shares_user
        FOREIGN KEY (grantee_user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_game_definition_shares_grantee ON game_definition_shares (grantee_user_id);
