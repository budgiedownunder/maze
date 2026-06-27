//! Multi-level generation for the stacked 3D difficulty games.
//!
//! A multi-level run is a chain of independently generated single-level
//! mazes. Each level's finish cell is the entry point to the next level, so
//! the chain is built bottom-to-top with three rules:
//!
//! * the per-level RNG seed is derived from the run's base seed + the level
//!   index, so a given (base seed, level count) always produces the same set;
//! * each level's finish is placed at a random cell far from its start, and
//!   every level above the bottom inherits the previous level's finish as its
//!   own start — so the transition is a clean vertical move at a fixed cell;
//! * the per-level enemy count follows the difficulty curve
//!   ([`LevelDifficultyChange`]) — the footprint and the health / treasure
//!   counts stay uniform across levels.

use super::{generate_maze_json, grid_to_json};
use crate::state::LayeredAlignment;
use maze::{GenerationAlgorithm, Generator, GeneratorOptions, MazePoint};

/// Upper bound on the number of levels a single run may stack. Caps the
/// geometry rendered at once (every level is built up front) and bounds the
/// generation cost.
pub const MAX_LEVEL_COUNT: usize = 20;

/// Target smallest footprint a tapered axis shrinks.
/// The effective per-axis minimum is bumped to share the base's parity (see
/// [`taper_min`]) so `base − dim` stays even.
const MIN_TAPER_DIM: usize = 3;

/// The smallest size a tapered axis of `base` shrinks to: [`MIN_TAPER_DIM`],
/// bumped up by one where needed so it shares `base`'s parity. Keeping
/// `base − dim` **even** is what lets an upper level's cells sit centred exactly
/// on a lower level's cell-centres (a half-cell offset would never align), so the
/// world-XZ landing maths ([`aligned_landing`]) — the exact inverse of the
/// renderer's [`crate::world::LevelPlacement`] offset — stays integer.
fn taper_min(base: usize) -> usize {
    let min = MIN_TAPER_DIM.min(base);
    if (base - min).is_multiple_of(2) { min } else { min + 1 }
}

/// Level `i`'s size along one axis (0 = bottom = `base`), interpolated down to
/// [`taper_min`] across the run's `n` levels in parity-preserving steps of 2 so
/// the open-sky stack opens up. Monotonic non-increasing in `i`;
/// returns `base` unchanged when no taper is possible (`n <= 1` or already at the
/// minimum).
fn taper_dim(base: usize, n: usize, i: usize) -> usize {
    if n <= 1 {
        return base;
    }
    let total_steps = (base - taper_min(base)) / 2;
    if total_steps == 0 {
        return base;
    }
    // Rounded distribution of the available 2-cell decrements across the levels:
    // step(0) = 0, step(n-1) = total_steps.
    let step = (total_steps * i + (n - 1) / 2) / (n - 1);
    base - 2 * step
}

/// How a run's difficulty changes from the bottom level to the top. The
/// player always climbs upward; this selects whether the climb gets harder,
/// easier, or stays level. The enemy count is the lever — it is interpolated
/// across the levels while the footprint and the health / treasure counts
/// stay uniform.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum LevelDifficultyChange {
    /// Every level is equally hard (same counts; only the maze layout differs).
    Same,
    /// Hardest at the bottom, easing as the player climbs (top is easiest).
    #[default]
    Easier,
    /// Easiest at the bottom, intensifying as the player climbs (top is hardest).
    Harder,
}

impl LevelDifficultyChange {
    /// Parses a wire string into a [`LevelDifficultyChange`]. Unknown values fall
    /// back to [`LevelDifficultyChange::Easier`] — the same forgiving policy as the
    /// other config enums, so a stale client or a typo still yields a playable run.
    pub fn from_wire_str(s: &str) -> Self {
        match s.to_ascii_lowercase().as_str() {
            "same" => Self::Same,
            "harder" => Self::Harder,
            _ => Self::Easier,
        }
    }
}

/// Number of candidate finish cells drawn per attempt; the farthest from the
/// start is kept, biasing the finish away from the start so the level has a
/// substantial path and the generator's own solvability retries rarely fail.
const FINISH_CANDIDATES: usize = 8;

/// Bounded outer retries: if a chosen start/finish pair can't be carved into a
/// solvable maze (the generator exhausts its own internal retries), a fresh
/// finish + carve seed is drawn up to this many times before giving up.
const MAX_FINISH_ATTEMPTS: usize = 24;

/// Generates the maze JSON for each level of a stacked multi-level run, bottom
/// level first.
///
/// The leading argument list mirrors [`generate_maze_json`] one-for-one (the
/// per-level generator knobs), followed by the run-level `level_count` and the
/// difficulty curve. `level_count` is clamped to `[1, MAX_LEVEL_COUNT]`; a
/// clamped count of 1 returns exactly what [`generate_maze_json`] would (the
/// generator's default corner start/finish), so a single-level run is
/// byte-identical to a non-multi-level game.
///
/// Deterministic: the same `base_seed` + `level_count` (+ identical other
/// arguments) always yields the same set of levels.
///
/// # Errors
///
/// Returns `Err` if a level cannot be generated into a solvable maze after the
/// bounded finish/seed retries (e.g. an impossible `min_solution_length` for
/// the grid).
///
/// # Examples
///
/// ```
/// use maze_game_bevy::{generate_level_maze_jsons, LayeredAlignment, LevelDifficultyChange};
///
/// let levels = generate_level_maze_jsons(
///     9, 9, 42, 0, 0, 0, 0, 2, 1, 1, 3, LevelDifficultyChange::Easier,
///     false, LayeredAlignment::Edge,
/// )
/// .expect("generates three chained levels");
/// assert_eq!(levels.len(), 3);
/// ```
#[allow(clippy::too_many_arguments)]
pub fn generate_level_maze_jsons(
    rows: u32,
    cols: u32,
    base_seed: u64,
    min_solution_length: u32,
    door_count: u32,
    spare_doors: u32,
    spare_keys: u32,
    enemy_count: u32,
    health_count: u32,
    treasure_count: u32,
    level_count: u32,
    difficulty_change: LevelDifficultyChange,
    // Whether to taper the upper levels' footprints so the open-sky stack reads as
    // see-through (decision 14). `false` for an enclosed/roofed stack — every level
    // keeps the full `rows × cols` footprint, and the chaining below collapses to
    // the previous "next start = previous finish at the same cell" behaviour.
    taper: bool,
    // How a smaller upper level sits over the level below — the same alignment the
    // renderer's `LevelPlacement` uses, so a ladder's landing cell agrees.
    alignment: LayeredAlignment,
) -> Result<Vec<String>, String> {
    let n = (level_count.max(1) as usize).min(MAX_LEVEL_COUNT);

    // A single level uses the generator's default corner start/finish, so a
    // 1-level run renders exactly as a non-multi-level game does today.
    if n == 1 {
        return Ok(vec![generate_maze_json(
            rows,
            cols,
            base_seed,
            min_solution_length,
            door_count,
            spare_doors,
            spare_keys,
            enemy_count,
            health_count,
            treasure_count,
        )?]);
    }

    let base_rows = rows as usize;
    let base_cols = cols as usize;
    let base_min = base_rows.min(base_cols).max(1);
    let mut jsons = Vec::with_capacity(n);

    // The next level's start: the previous level's finish, mapped down to the cell
    // directly *below* it in world space (the smaller upper grid is centred/edged
    // over the lower one). `None` until the bottom level fixes its own start.
    let mut next_start: Option<MazePoint> = None;

    for level_index in 0..n {
        let level_seed = mix_seed(base_seed, level_index as u64);
        let level_enemies = level_enemy_count(enemy_count as usize, n, level_index, difficulty_change);

        // This level's footprint and the next level's (smaller) footprint. With
        // `taper` off both stay at the base size, so every helper below reduces to
        // the uniform behaviour.
        let dims = level_dims(base_rows, base_cols, n, level_index, taper);
        let dims_next = (level_index + 1 < n)
            .then(|| level_dims(base_rows, base_cols, n, level_index + 1, taper));

        // Shrink the required spine with the footprint so a small upper maze stays
        // generatable (a large `min_solution_length` won't fit a 3×3).
        let min_sol = min_solution_length as usize * dims.0.min(dims.1) / base_min;

        let start = match next_start.take() {
            Some(point) => point,
            None => {
                // Bottom level: a random start cell (higher levels inherit the
                // previous finish above).
                let mut state = mix_seed(level_seed, START_SALT);
                random_cell(dims.0, dims.1, &mut state)
            }
        };

        let (json, landing) = generate_one_level(
            dims.0,
            dims.1,
            level_seed,
            min_sol,
            door_count as usize,
            spare_doors as usize,
            spare_keys as usize,
            level_enemies,
            health_count as usize,
            treasure_count as usize,
            &start,
            dims_next,
            alignment,
        )?;
        jsons.push(json);

        // Pin the next level's start. A ladder finish maps to the aligned landing
        // cell directly above it; a finish with no valid landing (too near the
        // edge of the smaller upper grid) gets a free start, and the renderer draws
        // that finish as a portal.
        next_start = dims_next.map(|dn| {
            landing.unwrap_or_else(|| {
                let mut state = mix_seed(level_seed, PORTAL_START_SALT);
                random_cell(dn.0, dn.1, &mut state)
            })
        });
    }

    Ok(jsons)
}

/// This level's `(rows, cols)` footprint: the base size when `taper` is off,
/// otherwise [`taper_dim`] applied per axis.
fn level_dims(base_rows: usize, base_cols: usize, n: usize, i: usize, taper: bool) -> (usize, usize) {
    if taper {
        (taper_dim(base_rows, n, i), taper_dim(base_cols, n, i))
    } else {
        (base_rows, base_cols)
    }
}

/// The cell in the next (smaller, offset) level that sits directly **below** a
/// finish at `finish` in this level — the inverse of the renderer's
/// [`crate::world::LevelPlacement`] X/Z offset. `Centre` insets the finish by half
/// the per-axis size difference; `Edge` leaves it unchanged. `None` when the
/// inset pushes the landing outside the smaller grid (no cell above → not a valid
/// ladder landing).
fn aligned_landing(
    alignment: LayeredAlignment,
    finish: &MazePoint,
    dims: (usize, usize),
    dims_next: (usize, usize),
) -> Option<MazePoint> {
    let (row_inset, col_inset) = match alignment {
        LayeredAlignment::Edge => (0, 0),
        LayeredAlignment::Centre => (
            (dims.0 - dims_next.0) / 2,
            (dims.1 - dims_next.1) / 2,
        ),
    };
    let row = finish.row.checked_sub(row_inset)?;
    let col = finish.col.checked_sub(col_inset)?;
    (row < dims_next.0 && col < dims_next.1).then_some(MazePoint { row, col })
}

/// Generates one level with a fixed start and an alignment-biased far finish,
/// retrying the finish + carve seed on an unsolvable carve. Returns the level
/// JSON and the **next level's landing cell** — the cell directly above the
/// chosen finish (so the caller pins the next start there for a ladder), or
/// `None` when the finish doesn't align over the smaller upper grid (the next
/// start is then free and the finish renders as a portal). `dims_next` is `None`
/// for the top level.
#[allow(clippy::too_many_arguments)]
fn generate_one_level(
    rows: usize,
    cols: usize,
    level_seed: u64,
    min_solution_length: usize,
    door_count: usize,
    spare_doors: usize,
    spare_keys: usize,
    enemy_count: usize,
    health_count: usize,
    treasure_count: usize,
    start: &MazePoint,
    dims_next: Option<(usize, usize)>,
    alignment: LayeredAlignment,
) -> Result<(String, Option<MazePoint>), String> {
    let mut last_err = String::from("no finish attempt was made");

    for attempt in 0..MAX_FINISH_ATTEMPTS {
        let mut finish_state = mix_seed(level_seed, FINISH_SALT.wrapping_add(attempt as u64));
        let (finish, landing) =
            pick_biased_finish(rows, cols, dims_next, alignment, start, &mut finish_state);
        let carve_seed = mix_seed(level_seed, attempt as u64);

        let options = GeneratorOptions {
            row_count: rows,
            col_count: cols,
            algorithm: GenerationAlgorithm::RecursiveBacktracking,
            start: Some(start.clone()),
            finish: Some(finish.clone()),
            min_spine_length: Some(min_solution_length),
            max_retries: None,
            branch_from_finish: None,
            seed: Some(carve_seed),
            door_count: Some(door_count),
            spare_doors: Some(spare_doors),
            spare_keys: Some(spare_keys),
            enemy_count: Some(enemy_count),
            health_count: Some(health_count),
            treasure_count: Some(treasure_count),
        };

        match (Generator { options }).generate() {
            Ok(maze) => return Ok((grid_to_json(&maze.definition.grid), landing)),
            Err(err) => last_err = err.to_string(),
        }
    }

    Err(format!(
        "could not generate a solvable level after {MAX_FINISH_ATTEMPTS} finish attempts: {last_err}"
    ))
}

/// Per-level enemy count along the difficulty curve. Level 0 is the bottom of
/// the stack. With a single level the base count is used unchanged.
fn level_enemy_count(
    base: usize,
    level_count: usize,
    level_index: usize,
    change: LevelDifficultyChange,
) -> usize {
    if level_count <= 1 {
        return base;
    }
    let last = level_count - 1;
    match change {
        LevelDifficultyChange::Same => base,
        // Linear from `base` at the bottom (index 0) to 0 at the top.
        LevelDifficultyChange::Easier => (base * (last - level_index) + last / 2) / last,
        // Linear from 0 at the bottom to `base` at the top.
        LevelDifficultyChange::Harder => (base * level_index + last / 2) / last,
    }
}

/// Picks a finish far from `start`, **biased toward one that aligns** over the
/// next (smaller, offset) level so the transition can be a ladder. Among the
/// [`FINISH_CANDIDATES`] draws it keeps the farthest candidate whose
/// [`aligned_landing`] is valid; failing that (or for the top level, where
/// `dims_next` is `None`), the farthest candidate overall — with no landing, so
/// the renderer knows to draws a non-ladder there. Returns the finish and the next level's
/// landing/start cell.
///
/// With no taper (`dims_next == Some(this level's size)`) every candidate aligns
/// (zero inset, landing == candidate), so this reduces to "the farthest finish,
/// landing at the same cell".
fn pick_biased_finish(
    rows: usize,
    cols: usize,
    dims_next: Option<(usize, usize)>,
    alignment: LayeredAlignment,
    start: &MazePoint,
    state: &mut u64,
) -> (MazePoint, Option<MazePoint>) {
    let mut best_aligned: Option<(MazePoint, MazePoint, usize)> = None;
    let mut best_any: Option<(MazePoint, usize)> = None;
    for _ in 0..FINISH_CANDIDATES {
        let cand = random_cell(rows, cols, state);
        if cand.row == start.row && cand.col == start.col {
            continue;
        }
        let distance = start.row.abs_diff(cand.row) + start.col.abs_diff(cand.col);
        if best_any.as_ref().is_none_or(|(_, d)| distance > *d) {
            best_any = Some((cand.clone(), distance));
        }
        if let Some(dn) = dims_next {
            if let Some(landing) = aligned_landing(alignment, &cand, (rows, cols), dn) {
                if best_aligned.as_ref().is_none_or(|(_, _, d)| distance > *d) {
                    best_aligned = Some((cand, landing, distance));
                }
            }
        }
    }
    if let Some((finish, landing, _)) = best_aligned {
        return (finish, Some(landing));
    }
    let finish = best_any
        .map(|(f, _)| f)
        .unwrap_or_else(|| fallback_finish(rows, cols, start));
    (finish, None)
}

/// Opposite-corner fallback when no distinct finish candidate was drawn (only a
/// pathological all-equal-to-start draw; the grid is always at least 3×3).
fn fallback_finish(rows: usize, cols: usize, start: &MazePoint) -> MazePoint {
    if start.row == 0 && start.col == 0 {
        MazePoint { row: rows - 1, col: cols - 1 }
    } else {
        MazePoint { row: 0, col: 0 }
    }
}

/// Picks a uniformly random cell within the grid using two `splitmix64` draws.
fn random_cell(rows: usize, cols: usize, state: &mut u64) -> MazePoint {
    let row = (splitmix64(state) % rows as u64) as usize;
    let col = (splitmix64(state) % cols as u64) as usize;
    MazePoint { row, col }
}

/// Salt mixed into the bottom level's start-cell RNG so it doesn't share a
/// stream with the carve seed.
const START_SALT: u64 = 0x5061_7468_5374_6172; // "PathStar"
/// Salt mixed into the per-attempt finish-cell RNG, kept separate from the
/// carve seed stream.
const FINISH_SALT: u64 = 0x4669_6E69_7368_2121; // "Finish!!"
/// Salt for the free start a tapered level gets when the level below's finish has
/// no aligned landing (a portal transition) — distinct from the carve / finish
/// streams, so it's deterministic but independent. Never reached when `taper` is
/// off (every finish then aligns).
const PORTAL_START_SALT: u64 = 0x506F_7274_616C_2121; // "Portal!!"

/// Combines a base seed with a counter into a well-distributed seed via one
/// `splitmix64` step. Deterministic — the whole multi-level chain is
/// reproducible from the run's base seed.
fn mix_seed(base: u64, counter: u64) -> u64 {
    let mut state = base ^ counter.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    splitmix64(&mut state)
}

/// A `splitmix64` step: advances `state` and returns a scrambled draw. Chosen
/// over pulling in an RNG dependency because the only randomness needed here is
/// deterministic cell selection, and a fixed algorithm guarantees identical
/// output on native and wasm for a given seed.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct GridJson {
        grid: Vec<Vec<char>>,
    }

    fn parse_grid(json: &str) -> Vec<Vec<char>> {
        serde_json::from_str::<GridJson>(json)
            .expect("level JSON parses")
            .grid
    }

    fn find_cell(json: &str, target: char) -> Option<(usize, usize)> {
        parse_grid(json).into_iter().enumerate().find_map(|(r, row)| {
            row.into_iter()
                .position(|c| c == target)
                .map(|col| (r, col))
        })
    }

    fn three_easier_levels(seed: u64) -> Result<Vec<String>, String> {
        generate_level_maze_jsons(
            9, 9, seed, 0, 0, 0, 0, 2, 1, 1, 3, LevelDifficultyChange::Easier, false,
            LayeredAlignment::Edge,
        )
    }

    fn grid_dims(json: &str) -> (usize, usize) {
        let grid = parse_grid(json);
        (grid.len(), grid.first().map_or(0, |row| row.len()))
    }

    #[test]
    fn deterministic_for_same_seed() {
        let a = three_easier_levels(1234).expect("ok");
        let b = three_easier_levels(1234).expect("ok");
        assert_eq!(a, b, "same base seed must yield identical levels");
    }

    #[test]
    fn level_count_clamped_to_max() {
        let levels = generate_level_maze_jsons(
            9,
            9,
            7,
            0,
            0,
            0,
            0,
            1,
            1,
            1,
            (MAX_LEVEL_COUNT as u32) + 5,
            LevelDifficultyChange::Same,
            false,
            LayeredAlignment::Edge,
        )
        .expect("ok");
        assert_eq!(levels.len(), MAX_LEVEL_COUNT);
    }

    #[test]
    fn level_count_zero_yields_one_level() {
        let levels = generate_level_maze_jsons(
            9, 9, 7, 0, 0, 0, 0, 0, 0, 0, 0, LevelDifficultyChange::Same, false,
            LayeredAlignment::Edge,
        )
        .expect("ok");
        assert_eq!(levels.len(), 1);
    }

    #[test]
    fn single_level_matches_generate_maze_json() {
        // A clamped count of 1 must be byte-identical to the single-maze path.
        let multi = generate_level_maze_jsons(
            9, 9, 99, 0, 1, 0, 0, 2, 1, 1, 1, LevelDifficultyChange::Easier, true,
            LayeredAlignment::Centre,
        )
        .expect("ok");
        let single = generate_maze_json(9, 9, 99, 0, 1, 0, 0, 2, 1, 1).expect("ok");
        assert_eq!(multi, vec![single]);
    }

    #[test]
    fn each_level_start_pins_to_previous_finish() {
        let levels = three_easier_levels(2024).expect("ok");
        assert_eq!(levels.len(), 3);
        for pair in levels.windows(2) {
            let prev_finish = find_cell(&pair[0], 'F').expect("a finish on every level");
            let next_start = find_cell(&pair[1], 'S').expect("a start on every level");
            assert_eq!(
                prev_finish, next_start,
                "the next level's start must sit above the previous level's finish",
            );
        }
    }

    #[test]
    fn enemy_count_curve_same_is_uniform() {
        let counts: Vec<usize> =
            (0..5).map(|i| level_enemy_count(4, 5, i, LevelDifficultyChange::Same)).collect();
        assert_eq!(counts, vec![4, 4, 4, 4, 4]);
    }

    #[test]
    fn enemy_count_curve_easier_decreases_to_zero() {
        let counts: Vec<usize> =
            (0..5).map(|i| level_enemy_count(4, 5, i, LevelDifficultyChange::Easier)).collect();
        // Bottom (index 0) hardest, top easiest.
        assert_eq!(counts, vec![4, 3, 2, 1, 0]);
        assert!(counts.windows(2).all(|w| w[0] >= w[1]), "non-increasing");
    }

    #[test]
    fn enemy_count_curve_harder_increases_to_base() {
        let counts: Vec<usize> =
            (0..5).map(|i| level_enemy_count(4, 5, i, LevelDifficultyChange::Harder)).collect();
        // Bottom easiest, top hardest.
        assert_eq!(counts, vec![0, 1, 2, 3, 4]);
        assert!(counts.windows(2).all(|w| w[0] <= w[1]), "non-decreasing");
    }

    #[test]
    fn taper_min_shares_the_base_parity() {
        assert_eq!(taper_min(9), 3); // odd base → odd min
        assert_eq!(taper_min(10), 4); // even base → bumped to even
        assert_eq!(taper_min(3), 3);
        assert_eq!(taper_min(4), 4);
        assert_eq!(taper_min(5), 3);
    }

    #[test]
    fn taper_dim_tapers_monotonically_and_preserves_parity() {
        let odd: Vec<usize> = (0..4).map(|i| taper_dim(9, 4, i)).collect();
        assert_eq!(odd, vec![9, 7, 5, 3]);
        let even: Vec<usize> = (0..4).map(|i| taper_dim(10, 4, i)).collect();
        assert_eq!(even, vec![10, 8, 6, 4]);
        // Many levels over a modest base: monotonic, clamped at the min, never
        // below, and (base − dim) stays even throughout (so cells keep aligning).
        let many: Vec<usize> = (0..20).map(|i| taper_dim(9, 20, i)).collect();
        assert!(many.windows(2).all(|w| w[0] >= w[1]), "non-increasing: {many:?}");
        assert!(many.iter().all(|&d| d >= 3 && (9 - d).is_multiple_of(2)));
        assert_eq!(*many.last().unwrap(), 3);
    }

    #[test]
    fn aligned_landing_inverts_the_centre_offset_and_rejects_out_of_range() {
        let dims = (7, 7);
        let next = (5, 5);
        let landing = |a, r, c| {
            aligned_landing(a, &MazePoint { row: r, col: c }, dims, next).map(|p| (p.row, p.col))
        };
        // Centre insets by half the size difference (1 here).
        assert_eq!(landing(LayeredAlignment::Centre, 4, 4), Some((3, 3)));
        // A finish at the edge maps outside the smaller grid → no landing (portal).
        assert_eq!(landing(LayeredAlignment::Centre, 0, 0), None);
        assert_eq!(landing(LayeredAlignment::Centre, 6, 6), None);
        // Edge keeps the same cell, valid only inside the smaller corner.
        assert_eq!(landing(LayeredAlignment::Edge, 3, 3), Some((3, 3)));
        assert_eq!(landing(LayeredAlignment::Edge, 5, 5), None);
    }

    #[test]
    fn aligned_landing_sits_directly_under_the_finish_in_world_space() {
        use crate::world::{LevelPlacement, CELL_SIZE};
        // The landing must map to the SAME world XZ the renderer puts the finish at
        // — that's what makes the ladder climb straight up.
        let (base, dims, next) = ((9, 9), (7, 7), (5, 5));
        let finish = MazePoint { row: 4, col: 4 };
        let landing = aligned_landing(LayeredAlignment::Centre, &finish, dims, next)
            .expect("a centred finish has a landing");
        let pi =
            LevelPlacement::for_level(1, dims.0, dims.1, base.0, base.1, LayeredAlignment::Centre, 0.0);
        let pn =
            LevelPlacement::for_level(2, next.0, next.1, base.0, base.1, LayeredAlignment::Centre, 0.0);
        let xz = |p: &LevelPlacement, r: usize, c: usize| {
            (p.world_x(c as f32 * CELL_SIZE + 1.0), p.world_z(r as f32 * CELL_SIZE + 1.0))
        };
        assert_eq!(xz(&pi, finish.row, finish.col), xz(&pn, landing.row, landing.col));
    }

    #[test]
    fn tapered_run_footprints_taper_monotonically() {
        let levels = generate_level_maze_jsons(
            9, 9, 2024, 0, 0, 0, 0, 0, 0, 0, 4, LevelDifficultyChange::Same, true,
            LayeredAlignment::Centre,
        )
        .expect("ok");
        let dims: Vec<(usize, usize)> = levels.iter().map(|j| grid_dims(j)).collect();
        assert_eq!(dims.first().copied(), Some((9, 9)), "bottom keeps the base footprint");
        assert!(
            dims.windows(2).all(|w| w[0].0 >= w[1].0 && w[0].1 >= w[1].1),
            "footprints taper monotonically: {dims:?}",
        );
        assert!(dims.last().unwrap().0 < 9, "the top level is strictly smaller than the base");
    }

    #[test]
    fn impossible_min_solution_length_errors() {
        // No spine of length 1000 fits a 5×5 grid, so every attempt fails and
        // the bounded retries surface an error rather than looping forever.
        let result = generate_level_maze_jsons(
            5, 5, 1, 1000, 0, 0, 0, 0, 0, 0, 2, LevelDifficultyChange::Same, false,
            LayeredAlignment::Edge,
        );
        assert!(result.is_err());
    }
}
