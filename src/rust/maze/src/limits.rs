//! Maze authoring limits — the per-type caps shared by generation, the key-aware
//! solver, and the storage save-validation. Kept in one always-compiled module
//! (not behind the `generation` feature) so every layer enforces the same numbers
//! and an authored maze can never carry more of a limited object than generation
//! would place.

/// Maximum combined count of `'K'` + `'D'` cells a maze may carry. The key-aware
/// solver tracks each as a bit in a `u32` mask, so its search is exponential in
/// their sum; above this cap the solver refuses rather than degrade to a key-blind
/// walk that would misrepresent sealed mazes as playable. Generation, the React
/// Generate dialog, the React editor save flow, and the server save endpoint all
/// enforce the same cap so the solver's error path never fires for a maze produced
/// through the supported tools.
pub const MAX_TOTAL_FEATURES: usize = 16;

/// Maximum number of enemies (`'E'` cells) a maze may carry. Generation caps the
/// auto-placed count here and the server save-validation rejects authored mazes
/// that exceed it, keeping the per-tick / per-enemy cost (and per-enemy renderer
/// state) bounded.
pub const MAX_ENEMY_COUNT: usize = 8;

/// Maximum number of health pickups (`'H'` cells) a maze may carry. Mirrors
/// [`MAX_ENEMY_COUNT`] so the two knobs feel symmetric to authors.
pub const MAX_HEALTH_COUNT: usize = 8;

/// Maximum number of treasure (`'T'` cells) a maze may carry. Generation caps the
/// auto-placed count here and the server save-validation rejects authored mazes
/// that exceed it, keeping a treasure-dense maze's in-game render cost (per-chest
/// point light + sparkles) within a mobile GPU's budget.
pub const MAX_TREASURE_COUNT: usize = 12;
