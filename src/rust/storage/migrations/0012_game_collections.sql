-- Game collections: an ordered, presentation-only grouping of game definitions
-- (leaderboards stay per-definition; a collection just frames a set for
-- browsing). A collection carries its own presentation (name / description /
-- image_updated_at marker / visibility); its members live in
-- `game_collection_items`, and its share grants in `game_collection_shares`,
-- mirroring `game_definitions` / `game_definition_shares`.
--
-- All schema rules from `0001_initial.sql` apply: VARCHAR(N) for every string
-- column, VARCHAR(32) for RFC 3339 timestamps, no `IF NOT EXISTS` on
-- `CREATE INDEX`, unique constraints inlined as table-level CONSTRAINTs. No
-- large opaque blob here (unlike `game_definitions.config`), so the MySQL
-- 65,535-byte per-row limit is not a concern.
--
-- `game_collection_items.definition_id` has **no** FK to `game_definitions`:
-- membership is presentation, a definition may be removed while still listed,
-- and the server filters a dangling reference out at display. `sort_order` is
-- INTEGER (a small list index, `u32` in the app, read back as `i32` per the
-- SQLx-Any rule). The composite PK gives per-collection membership uniqueness
-- and indexes the `collection_id` FK. Both share/item FKs are ON DELETE CASCADE
-- (deleting a collection or user drops the rows), also cascaded in app code for
-- uniform cross-backend behaviour.
CREATE TABLE IF NOT EXISTS game_collections (
    id                VARCHAR(36)   NOT NULL PRIMARY KEY,
    owner_id          VARCHAR(36)   NOT NULL,
    name              VARCHAR(255)  NOT NULL,
    description       VARCHAR(1024),
    image_updated_at  VARCHAR(32),
    visibility        VARCHAR(16)   NOT NULL,
    created_at        VARCHAR(32)   NOT NULL,
    updated_at        VARCHAR(32)   NOT NULL,
    CONSTRAINT fk_game_collections_owner
        FOREIGN KEY (owner_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT uq_game_collections_owner_name UNIQUE (owner_id, name)
);

CREATE INDEX idx_game_collections_owner ON game_collections (owner_id);
CREATE INDEX idx_game_collections_visibility ON game_collections (visibility);

CREATE TABLE IF NOT EXISTS game_collection_items (
    collection_id     VARCHAR(36) NOT NULL,
    definition_id     VARCHAR(36) NOT NULL,
    sort_order        INTEGER     NOT NULL,
    PRIMARY KEY (collection_id, definition_id),
    CONSTRAINT fk_game_collection_items_collection
        FOREIGN KEY (collection_id) REFERENCES game_collections(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS game_collection_shares (
    collection_id     VARCHAR(36) NOT NULL,
    grantee_user_id   VARCHAR(36) NOT NULL,
    PRIMARY KEY (collection_id, grantee_user_id),
    CONSTRAINT fk_game_collection_shares_collection
        FOREIGN KEY (collection_id) REFERENCES game_collections(id) ON DELETE CASCADE,
    CONSTRAINT fk_game_collection_shares_user
        FOREIGN KEY (grantee_user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX idx_game_collection_shares_grantee ON game_collection_shares (grantee_user_id);
