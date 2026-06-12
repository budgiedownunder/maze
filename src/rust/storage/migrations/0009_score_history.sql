-- Per-completed-run score history: one row per won 3D run, serving the
-- leaderboards (per-maze, per-curated-challenge) and personal history over
-- them. "Best" is a query (`ORDER BY … LIMIT`), not a stored flag — every
-- row is a completed run, so both board orderings (fastest time / highest
-- score) read it with no outcome filter.
--
-- All schema rules from `0001_initial.sql` apply: VARCHAR(N) for every string
-- column, VARCHAR(32) for RFC 3339 timestamps, no `IF NOT EXISTS` on
-- `CREATE INDEX`. Sizing: 36 for UUIDs, 64 for the `challenge` discriminator
-- (`"<difficulty>:<seed>"`). `score` and `elapsed_ms` are BIGINT (the engine
-- score is `u64`, stored as `i64`; ms can exceed an i32).
--
-- **Dual-keyed subject.** Exactly one of `maze_id` / `challenge` is set
-- (an app-layer invariant — there is no portable cross-column CHECK under
-- SQLx-Any / MySQL):
--   * a stored **user maze** → `maze_id` (FK `mazes(id)` ON DELETE CASCADE —
--     deleting a maze removes its board), or
--   * a **curated / shared game** → `challenge` (no FK; a generated preset has
--     no stored maze row, so its board has no parent to cascade from).
-- `user_id` is the **player** (not the maze owner), so boards aggregate every
-- player of a subject.
--
-- **FKs are ON DELETE CASCADE, but as a backstop.** The store cascades in app
-- code (the delete paths `DELETE FROM score_history` explicitly, mirroring the
-- other child tables) so the behaviour is uniform across FileStore, SQLite
-- (FK enforcement is pragma-gated), PostgreSQL, and MySQL.
CREATE TABLE IF NOT EXISTS score_history (
    id            VARCHAR(36) NOT NULL PRIMARY KEY,
    user_id       VARCHAR(36) NOT NULL,
    maze_id       VARCHAR(36),
    challenge     VARCHAR(64),
    score         BIGINT      NOT NULL,
    elapsed_ms    BIGINT      NOT NULL,
    completed_at  VARCHAR(32) NOT NULL,
    CONSTRAINT fk_score_history_user
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_score_history_maze
        FOREIGN KEY (maze_id) REFERENCES mazes(id) ON DELETE CASCADE
);

CREATE INDEX idx_score_history_maze ON score_history (maze_id);
CREATE INDEX idx_score_history_challenge ON score_history (challenge);
CREATE INDEX idx_score_history_user ON score_history (user_id);
CREATE INDEX idx_score_history_completed_at ON score_history (completed_at);
