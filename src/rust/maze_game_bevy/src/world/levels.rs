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
use maze::{GenerationAlgorithm, Generator, GeneratorOptions, MazePoint};

/// Upper bound on the number of levels a single run may stack. Caps the
/// geometry rendered at once (every level is built up front) and bounds the
/// generation cost.
pub const MAX_LEVEL_COUNT: usize = 20;

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
/// use maze_game_bevy::{generate_level_maze_jsons, LevelDifficultyChange};
///
/// let levels = generate_level_maze_jsons(
///     9, 9, 42, 0, 0, 0, 0, 2, 1, 1, 3, LevelDifficultyChange::Easier,
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

    let rows = rows as usize;
    let cols = cols as usize;
    let mut jsons = Vec::with_capacity(n);

    // The previous level's finish becomes the next level's start so the
    // transition rig is a straight vertical climb at a fixed grid cell.
    let mut next_start: Option<MazePoint> = None;

    for level_index in 0..n {
        let level_seed = mix_seed(base_seed, level_index as u64);
        let level_enemies = level_enemy_count(enemy_count as usize, n, level_index, difficulty_change);

        let start = match next_start.take() {
            Some(point) => point,
            None => {
                // Bottom level: a random start cell (higher levels inherit the
                // previous finish above).
                let mut state = mix_seed(level_seed, START_SALT);
                random_cell(rows, cols, &mut state)
            }
        };

        let (json, finish) = generate_one_level(
            rows,
            cols,
            level_seed,
            min_solution_length as usize,
            door_count as usize,
            spare_doors as usize,
            spare_keys as usize,
            level_enemies,
            health_count as usize,
            treasure_count as usize,
            &start,
        )?;
        jsons.push(json);
        next_start = Some(finish);
    }

    Ok(jsons)
}

/// Generates one level with a fixed start and a randomly chosen far finish,
/// retrying the finish + carve seed on an unsolvable carve. Returns the level
/// JSON and the finish cell (so the caller can pin the next level's start).
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
) -> Result<(String, MazePoint), String> {
    let mut last_err = String::from("no finish attempt was made");

    for attempt in 0..MAX_FINISH_ATTEMPTS {
        let mut finish_state = mix_seed(level_seed, FINISH_SALT.wrapping_add(attempt as u64));
        let finish = pick_far_finish(rows, cols, start, &mut finish_state);
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
            Ok(maze) => return Ok((grid_to_json(&maze.definition.grid), finish)),
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

/// Draws [`FINISH_CANDIDATES`] random cells and keeps the one farthest (by
/// Manhattan distance) from `start`, biasing the finish away from the start.
/// Falls back to an opposite corner if no distinct candidate was drawn (only
/// possible for pathological draws; the grid is always at least 3×3).
fn pick_far_finish(rows: usize, cols: usize, start: &MazePoint, state: &mut u64) -> MazePoint {
    let mut best: Option<MazePoint> = None;
    let mut best_distance = 0usize;
    for _ in 0..FINISH_CANDIDATES {
        let cand = random_cell(rows, cols, state);
        if cand.row == start.row && cand.col == start.col {
            continue;
        }
        let distance = start.row.abs_diff(cand.row) + start.col.abs_diff(cand.col);
        if best.is_none() || distance > best_distance {
            best_distance = distance;
            best = Some(cand);
        }
    }
    best.unwrap_or_else(|| {
        if start.row == 0 && start.col == 0 {
            MazePoint {
                row: rows - 1,
                col: cols - 1,
            }
        } else {
            MazePoint { row: 0, col: 0 }
        }
    })
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
        generate_level_maze_jsons(9, 9, seed, 0, 0, 0, 0, 2, 1, 1, 3, LevelDifficultyChange::Easier)
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
        )
        .expect("ok");
        assert_eq!(levels.len(), MAX_LEVEL_COUNT);
    }

    #[test]
    fn level_count_zero_yields_one_level() {
        let levels =
            generate_level_maze_jsons(9, 9, 7, 0, 0, 0, 0, 0, 0, 0, 0, LevelDifficultyChange::Same)
                .expect("ok");
        assert_eq!(levels.len(), 1);
    }

    #[test]
    fn single_level_matches_generate_maze_json() {
        // A clamped count of 1 must be byte-identical to the single-maze path.
        let multi =
            generate_level_maze_jsons(9, 9, 99, 0, 1, 0, 0, 2, 1, 1, 1, LevelDifficultyChange::Easier)
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
    fn impossible_min_solution_length_errors() {
        // No spine of length 1000 fits a 5×5 grid, so every attempt fails and
        // the bounded retries surface an error rather than looping forever.
        let result =
            generate_level_maze_jsons(5, 5, 1, 1000, 0, 0, 0, 0, 0, 0, 2, LevelDifficultyChange::Same);
        assert!(result.is_err());
    }
}
