-- Featured catalogue: a single admin-ordered list mixing game definitions and
-- game collections. It is a faithful projection of the `Curated` visibility
-- tier — a row is appended when an entity becomes `Curated`, and removed (with
-- the remaining `sort_order` recompacted to a dense `0..n`) when it stops being
-- `Curated` or is deleted. The store keeps this table in sync inside the same
-- transaction as the entity's visibility change, so the `Curated` flag and its
-- featured row can never disagree.
--
-- `entity_id` is **polymorphic with no FK**: it points at either
-- `game_definitions.id` or `game_collections.id` depending on `entity_kind`, so
-- a single column can't reference both tables. The reconcile hook is the only
-- writer and removes a row before its entity is deleted, so a dangling id never
-- persists; a read that does find one filters it out (like
-- `game_collection_items`). The composite PK `(entity_kind, entity_id)` keeps a
-- given entity featured at most once and indexes lookups by entity.
--
-- All schema rules from `0001_initial.sql` apply: VARCHAR(N) for the keyed
-- string columns, INTEGER `sort_order` (read back as `i32` per the SQLx-Any
-- rule), no `IF NOT EXISTS` on `CREATE INDEX`. No blob, so the MySQL
-- 65,535-byte per-row limit is not a concern.
CREATE TABLE IF NOT EXISTS featured_game_items (
    entity_kind  VARCHAR(16) NOT NULL,
    entity_id    VARCHAR(36) NOT NULL,
    sort_order   INTEGER     NOT NULL,
    PRIMARY KEY (entity_kind, entity_id)
);

CREATE INDEX idx_featured_game_items_sort_order ON featured_game_items (sort_order);
