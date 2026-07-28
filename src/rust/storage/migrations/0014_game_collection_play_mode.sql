-- Adds `play_mode` to the `game_collections` table — how a collection is played
-- once opened (free-choice `arcade` or ordered `campaign`).
--
-- The column is added nullable with no literal DEFAULT (MySQL portability:
-- adding NOT NULL without a literal DEFAULT on a populated table is rejected —
-- schema rule 1 in `0001_initial.sql`). No backfill is needed: a row that
-- pre-dates this column reads back as `Option<String> = None`, and
-- `game_collection_from_row` maps an absent/unrecognised value through
-- `PlayMode::from_wire_str` to the default `Arcade` — the same "nullable column,
-- lenient parse, sensible default" spirit as `0004`/`0007`. New collections
-- written by the application always populate it.
--
-- Schema rules per `0001_initial.sql`:
--   * `VARCHAR(16)` (not bare TEXT) — the wire value is a short lowercase word
--     (`arcade` / `campaign`); VARCHAR keeps `String` decoding working under
--     SQLx-Any (rule 5).
--   * No literal `DEFAULT` (rule 1).

ALTER TABLE game_collections ADD COLUMN play_mode VARCHAR(16);
