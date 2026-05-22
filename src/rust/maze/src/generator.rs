use rand::seq::SliceRandom;
use rand::rngs::StdRng;
use rand::SeedableRng;

use data_model::{Maze, MazeDefinition, MazePoint};

use crate::{Error, GenerationAlgorithm, Solver};

/// Options that control how a maze is generated.
///
/// All `Option` fields fall back to a documented default when `None`.
#[derive(serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub struct GeneratorOptions {
    /// Number of rows in the generated maze. Must be ≥ 3.
    pub row_count: usize,
    /// Number of columns in the generated maze. Must be ≥ 3.
    pub col_count: usize,
    /// Algorithm used to generate the maze.
    pub algorithm: GenerationAlgorithm,
    /// Start cell. Defaults to `(0, 0)`.
    pub start: Option<MazePoint>,
    /// Finish cell. Defaults to `(row_count - 1, col_count - 1)`.
    pub finish: Option<MazePoint>,
    /// Minimum number of cells on the spine (the direct start to finish path).
    /// If no path of this length exists given the grid geometry, generation returns an error.
    /// Defaults to `(row_count + col_count) / 2`.
    pub min_spine_length: Option<usize>,
    /// Maximum number of generation attempts before returning an error. Each attempt uses a fresh
    /// RNG draw and may fail if finish is unreachable in that Depth-First Search (DFS) pass or the spine is shorter
    /// than `min_spine_length`. Passing `Some(0)` returns an error immediately without attempting
    /// generation. Defaults to `100`.
    pub max_retries: Option<usize>,
    /// Whether branches may grow out of the finish cell.
    /// When `false` (the default) the finish cell is excluded from branching,
    /// keeping it as an unambiguous dead end with exactly one passage leading to it.
    pub branch_from_finish: Option<bool>,
    /// Optional random number generator seed for deterministic generation.
    /// When `Some(seed)`, a seeded pseudo-random number generator is used — repeated calls with the same seed
    /// produce identical mazes. When `None` (the default), the OS-seeded thread random number generator
    /// is used as before.
    #[serde(default)]
    pub seed: Option<u64>,
    /// Number of doors (each paired with one key) to auto-place into the
    /// generated maze. Doors are placed on the start→finish solution path at
    /// 1-wide choke points, with each door's key hidden in a reachable dead-end
    /// before it, so the maze stays solvable (verified with the key-aware
    /// solver). The count is clamped to what the maze can hold and to a small
    /// ceiling that keeps the key-aware solver in verifying range. `None` or
    /// `Some(0)` (the default) places nothing — the lock-free maze as before.
    #[serde(default)]
    pub door_count: Option<usize>,
}

/// Generates a maze from a set of [`GeneratorOptions`].
///
/// # Examples
///
/// ```
/// use data_model::MazePoint;
/// use maze::{Generator, GeneratorOptions, GenerationAlgorithm, MazeSolver, Solver};
///
/// let gen = Generator {
///     options: GeneratorOptions {
///         row_count: 11,
///         col_count: 11,
///         algorithm: GenerationAlgorithm::RecursiveBacktracking,
///         start: None,
///         finish: None,
///         min_spine_length: None,
///         max_retries: None,
///         branch_from_finish: None,
///         seed: None,
///         door_count: None,
///     },
/// };
/// let maze = gen.generate().expect("generation should succeed");
/// assert_eq!(maze.definition.row_count(), 11);
/// assert_eq!(maze.definition.col_count(), 11);
/// Solver { maze: &maze }.solve().expect("generated maze should be solvable");
/// ```
pub struct Generator {
    pub options: GeneratorOptions,
}

impl Generator {
    /// Generates a maze according to [`GeneratorOptions`].
    ///
    /// # Returns
    ///
    /// A `Result` containing the generated [`Maze`] if successful, or an
    /// [`Error::Generate`] if validation fails or all `max_retries` attempts are exhausted
    /// without producing a maze that satisfies `min_spine_length`.
    pub fn generate(&self) -> Result<Maze, Error> {
        self.validate()?;
        match self.options.algorithm {
            GenerationAlgorithm::RecursiveBacktracking => self.generate_recursive_backtracking(),
        }
    }

    fn validate(&self) -> Result<(), Error> {
        let opts = &self.options;
        if opts.row_count < 3 {
            return Err(Error::Generate("row_count must be at least 3".to_string()));
        }
        if opts.col_count < 3 {
            return Err(Error::Generate("col_count must be at least 3".to_string()));
        }
        let start = opts
            .start
            .clone()
            .unwrap_or(MazePoint { row: 0, col: 0 });
        let finish = opts.finish.clone().unwrap_or(MazePoint {
            row: opts.row_count - 1,
            col: opts.col_count - 1,
        });
        if start.row >= opts.row_count || start.col >= opts.col_count {
            return Err(Error::Generate("start is out of bounds".to_string()));
        }
        if finish.row >= opts.row_count || finish.col >= opts.col_count {
            return Err(Error::Generate("finish is out of bounds".to_string()));
        }
        if start == finish {
            return Err(Error::Generate(
                "start and finish must be different cells".to_string(),
            ));
        }
        Ok(())
    }

    fn generate_recursive_backtracking(&self) -> Result<Maze, Error> {
        let opts = &self.options;
        let rows = opts.row_count;
        let cols = opts.col_count;
        let start = opts
            .start
            .clone()
            .unwrap_or(MazePoint { row: 0, col: 0 });
        let finish = opts.finish.clone().unwrap_or(MazePoint {
            row: rows - 1,
            col: cols - 1,
        });
        let min_spine = opts.min_spine_length.unwrap_or((rows + cols) / 2);
        let max_retries = opts.max_retries.unwrap_or(100);
        let branch_from_finish = opts.branch_from_finish.unwrap_or(false);
        let door_count = opts.door_count.unwrap_or(0);

        let seed_val: u64 = match opts.seed {
            Some(s) => s,
            None => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    use rand::RngCore;
                    let mut buf = [0u8; 8];
                    rand::thread_rng().fill_bytes(&mut buf);
                    u64::from_le_bytes(buf)
                }
                #[cfg(target_arch = "wasm32")]
                { unreachable!("seed must be provided on wasm32") }
            }
        };
        let mut rng = StdRng::seed_from_u64(seed_val);

        const OFFSETS: [(i64, i64); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

        let in_bounds = |r: i64, c: i64| -> bool {
            r >= 0 && (r as usize) < rows && c >= 0 && (c as usize) < cols
        };

        // Returns true if carving from (from_r, from_c) into (nr, nc) would not create a cycle.
        // A cycle would occur if (nr, nc) is already adjacent to any carved cell other than
        // (from_r, from_c). In this grid-based representation, adjacency of two passable cells
        // IS a passage — so we must ensure the new cell only touches its single parent.
        let can_carve =
            |grid: &[Vec<char>], from_r: usize, from_c: usize, nr: usize, nc: usize| -> bool {
                for (dr, dc) in &OFFSETS {
                    let rr = nr as i64 + dr;
                    let cc = nc as i64 + dc;
                    if !in_bounds(rr, cc) {
                        continue;
                    }
                    let (rr, cc) = (rr as usize, cc as usize);
                    if rr == from_r && cc == from_c {
                        continue;
                    }
                    if grid[rr][cc] != 'W' {
                        return false;
                    }
                }
                true
            };

        // Collects unvisited ('W') neighbours of `cell` that pass can_carve, shuffled.
        let collect_neighbors =
            |grid: &[Vec<char>], cell: &MazePoint, rng: &mut StdRng| -> Vec<MazePoint> {
                let mut result: Vec<MazePoint> = Vec::new();
                for (dr, dc) in &OFFSETS {
                    let nr = cell.row as i64 + dr;
                    let nc = cell.col as i64 + dc;
                    if !in_bounds(nr, nc) {
                        continue;
                    }
                    let (nr, nc) = (nr as usize, nc as usize);
                    if grid[nr][nc] != 'W' {
                        continue;
                    }
                    if !can_carve(grid, cell.row, cell.col, nr, nc) {
                        continue;
                    }
                    result.push(MazePoint { row: nr, col: nc });
                }
                result.shuffle(rng);
                result
            };

        // max_retries == 0 is a special sentinel that means "don't attempt at all".
        if max_retries == 0 {
            return Err(Error::Generate("max_retries is 0, no attempts made".to_string()));
        }

        // A single iterative Depth-First Search (DFS) from start carves the full maze in one pass.
        // can_carve ensures each new cell touches only its DFS parent, which guarantees
        // the result is a spanning tree (perfect maze: exactly one path between any two cells).
        // Cells are never un-carved, so each cell is visited at most once and generation
        // is always O(n) regardless of grid size or RNG seed.
        //
        // After generation the maze is solved to check the spine length. Two retry conditions:
        //   1. finish stayed 'W' — can_carve can block all paths to finish when its neighbours
        //      all get carved before the DFS reaches it; retry with the next RNG draw.
        //   2. spine shorter than min_spine — retry until the DFS produces a long enough path.
        //
        // branch_from_finish: when false, finish is carved but not pushed onto the DFS stack,
        // keeping it as an unambiguous dead end with exactly one inbound passage.

        let mut last_err = format!(
            "solution length is less than minimum solution length {min_spine}"
        );

        for _ in 0..max_retries {
            let mut grid = vec![vec!['W'; cols]; rows];
            grid[start.row][start.col] = ' ';

            let init_neighbors = collect_neighbors(&grid, &start, &mut rng);
            // Stack frame: (from_row, from_col, remaining_neighbors).
            // from_row/from_col are stored as usize (Copy) so they can be read before the
            // mutable pop() without conflicting borrows.
            let mut stack: Vec<(usize, usize, Vec<MazePoint>)> =
                vec![(start.row, start.col, init_neighbors)];

            while let Some(frame) = stack.last_mut() {
                let (from_row, from_col) = (frame.0, frame.1);
                match frame.2.pop() {
                    Some(next) => {
                        // Re-check: grid may have changed since this frame was pushed.
                        if grid[next.row][next.col] != 'W' {
                            continue;
                        }
                        if !can_carve(&grid, from_row, from_col, next.row, next.col) {
                            continue;
                        }
                        grid[next.row][next.col] = ' ';
                        if !(next.row == finish.row
                            && next.col == finish.col
                            && !branch_from_finish)
                        {
                            let nbrs = collect_neighbors(&grid, &next, &mut rng);
                            stack.push((next.row, next.col, nbrs));
                        }
                    }
                    None => {
                        stack.pop();
                    }
                }
            }

            // If finish was never carved (can_carve blocked all paths to it), retry.
            if grid[finish.row][finish.col] == 'W' {
                continue;
            }

            grid[start.row][start.col] = 'S';
            grid[finish.row][finish.col] = 'F';

            let maze = Maze::new(MazeDefinition::from_vec(grid));
            match (Solver { maze: &maze }).solve() {
                Ok(solution) if solution.path.points.len() >= min_spine => {
                    if door_count == 0 {
                        return Ok(maze);
                    }
                    // Auto-place keys/doors onto the solved spine, then confirm the
                    // result is still solvable with the key-aware solver. On the rare
                    // failure, fall through to retry with a fresh carve.
                    let mut placed = maze.definition.grid.clone();
                    place_keys_and_doors(&mut placed, &solution.path.points, door_count);
                    let placed_maze = Maze::new(MazeDefinition::from_vec(placed));
                    match (Solver { maze: &placed_maze }).solve() {
                        Ok(sol2) if sol2.path.points.len() >= min_spine => {
                            return Ok(placed_maze)
                        }
                        _ => {
                            last_err =
                                "key/door placement did not yield a solvable maze".to_string();
                        }
                    }
                }
                Ok(solution) => {
                    last_err = format!(
                        "solution length {} is less than minimum solution length {}",
                        solution.path.points.len(),
                        min_spine,
                    );
                }
                Err(_) => {
                    last_err = "maze is not solvable".to_string();
                }
            }
        }

        Err(Error::Generate(last_err))
    }
}

/// Maximum number of doors auto-placed at generation. Keeps
/// `keys + doors = 2 * doors ≤ 16` — the key-aware solver's `MAX_GATED_FEATURES`
/// bound — so the placement-validation solve stays in true key-aware mode rather
/// than falling back to the lock-blind solve.
const MAX_AUTO_DOORS: usize = 8;

/// Count of non-wall 4-neighbours of `(r, c)`.
fn open_degree(grid: &[Vec<char>], r: usize, c: usize) -> usize {
    let rows = grid.len();
    let cols = grid[r].len();
    let mut d = 0;
    if r > 0 && grid[r - 1][c] != 'W' {
        d += 1;
    }
    if r + 1 < rows && grid[r + 1][c] != 'W' {
        d += 1;
    }
    if c > 0 && grid[r][c - 1] != 'W' {
        d += 1;
    }
    if c + 1 < cols && grid[r][c + 1] != 'W' {
        d += 1;
    }
    d
}

/// Passable (non-wall) 4-neighbours of `(r, c)`.
fn passable_neighbours(grid: &[Vec<char>], r: usize, c: usize) -> Vec<(usize, usize)> {
    let rows = grid.len();
    let cols = grid[r].len();
    let mut out = Vec::with_capacity(4);
    if r > 0 && grid[r - 1][c] != 'W' {
        out.push((r - 1, c));
    }
    if c > 0 && grid[r][c - 1] != 'W' {
        out.push((r, c - 1));
    }
    if r + 1 < rows && grid[r + 1][c] != 'W' {
        out.push((r + 1, c));
    }
    if c + 1 < cols && grid[r][c + 1] != 'W' {
        out.push((r, c + 1));
    }
    out
}

/// Walks down a branch (a subtree hanging off the spine) from `start`, never
/// stepping back to `came_from`, and returns a dead-end leaf cell. The maze is a
/// perfect maze (a tree), so following any child each step terminates at a leaf.
fn branch_dead_end(
    grid: &[Vec<char>],
    start: (usize, usize),
    came_from: (usize, usize),
) -> (usize, usize) {
    let mut prev = came_from;
    let mut cur = start;
    loop {
        match passable_neighbours(grid, cur.0, cur.1)
            .into_iter()
            .find(|&n| n != prev)
        {
            Some(n) => {
                prev = cur;
                cur = n;
            }
            None => return cur,
        }
    }
}

/// Finds a cell to hold a key within the spine segment spanning spine indices
/// `[lo, hi]`: prefers a hidden dead-end in a branch hanging off a segment spine
/// cell, else falls back to a segment spine cell itself (always passable and
/// neither start/finish nor a door). Returns `None` only for an empty segment,
/// which the door spacing prevents.
fn find_key_cell(
    grid: &[Vec<char>],
    spine: &[MazePoint],
    lo: usize,
    hi: usize,
) -> Option<(usize, usize)> {
    if lo > hi {
        return None;
    }
    // Prefer a dead-end down a branch off the segment, so the key isn't simply
    // sitting on the through-route.
    for j in lo..=hi {
        let s = &spine[j];
        let prev = if j > 0 {
            Some((spine[j - 1].row, spine[j - 1].col))
        } else {
            None
        };
        let next = spine.get(j + 1).map(|p| (p.row, p.col));
        for nb in passable_neighbours(grid, s.row, s.col) {
            if Some(nb) == prev || Some(nb) == next {
                continue; // stay off the spine
            }
            return Some(branch_dead_end(grid, nb, (s.row, s.col)));
        }
    }
    // Fallback: a spine cell in the segment.
    Some((spine[lo].row, spine[lo].col))
}

/// Places up to `requested` doors (each with one preceding key) onto the maze's
/// start→finish `spine`, mutating `grid` in place. Doors land on interior 1-wide
/// spine cells (so each gates the through-route without sealing a side branch),
/// spaced out, and each door's key goes in the spine segment before it. The
/// count is clamped to the available choke points and to [`MAX_AUTO_DOORS`]; the
/// caller validates the result with the key-aware solver.
fn place_keys_and_doors(grid: &mut [Vec<char>], spine: &[MazePoint], requested: usize) {
    let m = spine.len();
    if requested == 0 || m < 5 {
        return;
    }
    // Door candidates: interior spine cells (excluding start, finish and their
    // immediate neighbours) that are 1-wide corridors — degree 2, so their only
    // open neighbours are the two adjacent spine cells. A door there gates the
    // spine without cutting off a branch.
    let candidates: Vec<usize> = (2..m - 2)
        .filter(|&j| open_degree(grid, spine[j].row, spine[j].col) == 2)
        .collect();
    let n = requested.min(candidates.len()).min(MAX_AUTO_DOORS);
    if n == 0 {
        return;
    }
    // Spread the doors across the candidates, keeping a spine gap ≥ 2 between
    // chosen doors so each leaves room for the next door's key segment.
    let mut doors: Vec<usize> = Vec::with_capacity(n);
    for k in 0..n {
        let idx = if n == 1 {
            candidates.len() / 2
        } else {
            k * (candidates.len() - 1) / (n - 1)
        };
        let cand = candidates[idx];
        if doors.iter().all(|&d| cand.abs_diff(d) >= 2) {
            doors.push(cand);
        }
    }
    doors.sort_unstable();
    doors.dedup();
    if doors.is_empty() {
        return;
    }
    for &j in &doors {
        grid[spine[j].row][spine[j].col] = 'D';
    }
    // One key per door, in the segment of the spine before it. Keys are
    // interchangeable, so collecting one per segment leaves the player with
    // exactly one in hand at each door.
    let mut prev_boundary = 0usize; // spine index of the previous door (0 = start)
    for &dj in &doors {
        if let Some((r, c)) = find_key_cell(grid, spine, prev_boundary + 1, dj - 1) {
            grid[r][c] = 'K';
        }
        prev_boundary = dj;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{GenerationAlgorithm, Solver};
    use pretty_assertions::assert_eq;

    fn make_generator(rows: usize, cols: usize) -> Generator {
        Generator {
            options: GeneratorOptions {
                row_count: rows,
                col_count: cols,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: None,
                finish: None,
                min_spine_length: None,
                max_retries: None,
                branch_from_finish: None,
                seed: None,
                door_count: None,
            },
        }
    }

    // --- Validation tests ---

    #[test]
    fn row_count_less_than_3_returns_error() {
        let gen = make_generator(2, 5);
        assert!(matches!(gen.generate(), Err(Error::Generate(_))));
    }

    #[test]
    fn col_count_less_than_3_returns_error() {
        let gen = make_generator(5, 2);
        assert!(matches!(gen.generate(), Err(Error::Generate(_))));
    }

    #[test]
    fn start_out_of_bounds_returns_error() {
        let gen = Generator {
            options: GeneratorOptions {
                row_count: 5,
                col_count: 5,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: Some(MazePoint { row: 10, col: 0 }),
                finish: None,
                min_spine_length: None,
                max_retries: None,
                branch_from_finish: None,
                seed: None,
                door_count: None,
            },
        };
        assert!(matches!(gen.generate(), Err(Error::Generate(_))));
    }

    #[test]
    fn finish_out_of_bounds_returns_error() {
        let gen = Generator {
            options: GeneratorOptions {
                row_count: 5,
                col_count: 5,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: None,
                finish: Some(MazePoint { row: 0, col: 10 }),
                min_spine_length: None,
                max_retries: None,
                branch_from_finish: None,
                seed: None,
                door_count: None,
            },
        };
        assert!(matches!(gen.generate(), Err(Error::Generate(_))));
    }

    #[test]
    fn start_equals_finish_returns_error() {
        let gen = Generator {
            options: GeneratorOptions {
                row_count: 5,
                col_count: 5,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: Some(MazePoint { row: 2, col: 2 }),
                finish: Some(MazePoint { row: 2, col: 2 }),
                min_spine_length: None,
                max_retries: None,
                branch_from_finish: None,
                seed: None,
                door_count: None,
            },
        };
        assert!(matches!(gen.generate(), Err(Error::Generate(_))));
    }

    // --- Structural correctness tests ---

    fn assert_structural_correctness(rows: usize, cols: usize) {
        let maze = make_generator(rows, cols)
            .generate()
            .expect("generation should succeed");

        let grid = &maze.definition.grid;

        // Correct dimensions
        assert_eq!(grid.len(), rows, "row count mismatch for {}x{}", rows, cols);
        for row in grid {
            assert_eq!(row.len(), cols, "col count mismatch for {}x{}", rows, cols);
        }

        // Exactly one S and one F
        let s_count = grid.iter().flatten().filter(|&&c| c == 'S').count();
        let f_count = grid.iter().flatten().filter(|&&c| c == 'F').count();
        assert_eq!(s_count, 1, "S count should be 1 for {}x{}", rows, cols);
        assert_eq!(f_count, 1, "F count should be 1 for {}x{}", rows, cols);

        // Solvable (S and F are connected)
        let solver = Solver { maze: &maze };
        solver
            .solve()
            .unwrap_or_else(|_| panic!("maze {rows}x{cols} should be solvable"));

        // Perfect maze property: passable_count - 1 == adjacent_passable_pair_count
        let is_passable = |c: char| c != 'W';
        let passable_count = grid.iter().flatten().filter(|&&c| is_passable(c)).count();

        let mut adjacent_pairs = 0usize;
        for r in 0..rows {
            for c in 0..cols {
                if is_passable(grid[r][c]) {
                    if r + 1 < rows && is_passable(grid[r + 1][c]) {
                        adjacent_pairs += 1;
                    }
                    if c + 1 < cols && is_passable(grid[r][c + 1]) {
                        adjacent_pairs += 1;
                    }
                }
            }
        }
        assert_eq!(
            passable_count - 1,
            adjacent_pairs,
            "perfect maze property failed for {}x{}: {} passable cells, {} adjacent pairs",
            rows,
            cols,
            passable_count,
            adjacent_pairs
        );
    }

    #[test]
    fn structural_correctness_3x3() {
        assert_structural_correctness(3, 3);
    }

    #[test]
    fn structural_correctness_5x7() {
        assert_structural_correctness(5, 7);
    }

    #[test]
    fn structural_correctness_11x11() {
        assert_structural_correctness(11, 11);
    }

    #[test]
    fn structural_correctness_21x31() {
        assert_structural_correctness(21, 31);
    }

    #[test]
    fn structural_correctness_51x51() {
        assert_structural_correctness(51, 51);
    }

    // --- Custom start/finish placement ---

    #[test]
    fn default_start_is_at_0_0() {
        let maze = make_generator(7, 7).generate().expect("should succeed");
        assert_eq!(maze.definition.grid[0][0], 'S');
    }

    #[test]
    fn default_finish_is_at_last_cell() {
        let maze = make_generator(7, 7).generate().expect("should succeed");
        assert_eq!(maze.definition.grid[6][6], 'F');
    }

    #[test]
    fn custom_start_and_finish_land_at_specified_coordinates() {
        let gen = Generator {
            options: GeneratorOptions {
                row_count: 9,
                col_count: 9,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: Some(MazePoint { row: 0, col: 4 }),
                finish: Some(MazePoint { row: 8, col: 4 }),
                min_spine_length: None,
                max_retries: None,
                branch_from_finish: None,
                seed: None,
                door_count: None,
            },
        };
        let maze = gen.generate().expect("should succeed");
        assert_eq!(maze.definition.grid[0][4], 'S');
        assert_eq!(maze.definition.grid[8][4], 'F');
    }

    // --- Spine length ---

    #[test]
    fn solution_path_length_meets_min_spine_length() {
        let min_spine = 10usize;
        let gen = Generator {
            options: GeneratorOptions {
                row_count: 11,
                col_count: 11,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: None,
                finish: None,
                min_spine_length: Some(min_spine),
                max_retries: None,
                branch_from_finish: None,
                seed: None,
                door_count: None,
            },
        };
        let maze = gen.generate().expect("should succeed");
        let solution = Solver { maze: &maze }.solve().expect("should be solvable");
        assert!(
            solution.path.points.len() >= min_spine,
            "solution path length {} should be >= min_spine_length {}",
            solution.path.points.len(),
            min_spine
        );
    }

    // --- Options ---

    #[test]
    fn impossible_min_spine_length_exhausts_retries_and_errors() {
        let gen = Generator {
            options: GeneratorOptions {
                row_count: 3,
                col_count: 3,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: None,
                finish: None,
                min_spine_length: Some(1000),
                max_retries: Some(5),
                branch_from_finish: None,
                seed: None,
                door_count: None,
            },
        };
        assert!(matches!(gen.generate(), Err(Error::Generate(_))));
    }

    #[test]
    fn max_retries_zero_returns_error_immediately() {
        let gen = Generator {
            options: GeneratorOptions {
                row_count: 5,
                col_count: 5,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: None,
                finish: None,
                min_spine_length: None,
                max_retries: Some(0),
                branch_from_finish: None,
                seed: None,
                door_count: None,
            },
        };
        assert!(matches!(gen.generate(), Err(Error::Generate(_))));
    }

    // --- Seeded generation ---

    #[test]
    fn seeded_generation_is_deterministic() {
        let make = || Generator {
            options: GeneratorOptions {
                row_count: 11,
                col_count: 11,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: None,
                finish: None,
                min_spine_length: None,
                max_retries: None,
                branch_from_finish: None,
                seed: Some(42),
                door_count: None,
            },
        };
        let maze1 = make().generate().expect("should succeed");
        let maze2 = make().generate().expect("should succeed");
        assert_eq!(maze1.definition.grid, maze2.definition.grid);
    }

    #[test]
    fn different_seeds_produce_different_mazes() {
        let make = |seed: u64| Generator {
            options: GeneratorOptions {
                row_count: 11,
                col_count: 11,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: None,
                finish: None,
                min_spine_length: None,
                max_retries: None,
                branch_from_finish: None,
                seed: Some(seed),
                door_count: None,
            },
        };
        let maze1 = make(1).generate().expect("should succeed");
        let maze2 = make(2).generate().expect("should succeed");
        assert_ne!(maze1.definition.grid, maze2.definition.grid);
    }

    // --- Automatic key/door placement ---

    fn make_with_doors(rows: usize, cols: usize, seed: u64, doors: usize) -> Generator {
        Generator {
            options: GeneratorOptions {
                row_count: rows,
                col_count: cols,
                algorithm: GenerationAlgorithm::RecursiveBacktracking,
                start: None,
                finish: None,
                min_spine_length: None,
                max_retries: None,
                branch_from_finish: None,
                seed: Some(seed),
                door_count: Some(doors),
            },
        }
    }

    fn count_char(grid: &[Vec<char>], ch: char) -> usize {
        grid.iter().flatten().filter(|&&c| c == ch).count()
    }

    #[test]
    fn door_count_zero_places_no_keys_or_doors() {
        let maze = make_with_doors(15, 15, 7, 0)
            .generate()
            .expect("should succeed");
        assert_eq!(count_char(&maze.definition.grid, 'D'), 0);
        assert_eq!(count_char(&maze.definition.grid, 'K'), 0);
    }

    #[test]
    fn auto_placed_maze_has_matching_keys_and_doors_and_is_solvable() {
        let maze = make_with_doors(15, 15, 7, 3)
            .generate()
            .expect("should succeed");
        let doors = count_char(&maze.definition.grid, 'D');
        let keys = count_char(&maze.definition.grid, 'K');
        assert!(doors >= 1, "expected at least one door, got {doors}");
        assert!(doors <= 3, "should not exceed the requested 3, got {doors}");
        assert_eq!(keys, doors, "one key is placed per door");
        // The key-aware solver must find a completing route.
        Solver { maze: &maze }
            .solve()
            .expect("auto-placed maze must be key-aware solvable");
        // Still exactly one start and one finish.
        assert_eq!(count_char(&maze.definition.grid, 'S'), 1);
        assert_eq!(count_char(&maze.definition.grid, 'F'), 1);
    }

    #[test]
    fn auto_placed_doors_lie_on_the_solution_spine() {
        // Removing the doors/keys (back to ' ') and solving lock-blind gives the
        // spine; every placed door must sit on it (placement only puts doors on
        // the spine).
        let maze = make_with_doors(15, 15, 11, 3)
            .generate()
            .expect("should succeed");
        let grid = &maze.definition.grid;
        let mut bare = grid.clone();
        for row in bare.iter_mut() {
            for cell in row.iter_mut() {
                if *cell == 'D' || *cell == 'K' {
                    *cell = ' ';
                }
            }
        }
        let bare_maze = Maze::new(MazeDefinition::from_vec(bare));
        let spine: std::collections::HashSet<(usize, usize)> = Solver { maze: &bare_maze }
            .solve()
            .expect("bare maze solvable")
            .path
            .points
            .iter()
            .map(|p| (p.row, p.col))
            .collect();
        for (r, row) in grid.iter().enumerate() {
            for (c, &cell) in row.iter().enumerate() {
                if cell == 'D' {
                    assert!(spine.contains(&(r, c)), "door at ({r},{c}) is off the spine");
                }
            }
        }
    }

    #[test]
    fn door_count_is_clamped_on_a_small_maze_and_stays_solvable() {
        // A 5x5 maze can't hold 8 doors; placement clamps and the result is
        // still solvable with matching keys/doors.
        let maze = make_with_doors(5, 5, 3, 8)
            .generate()
            .expect("should succeed");
        let doors = count_char(&maze.definition.grid, 'D');
        assert!(doors <= 8);
        assert_eq!(count_char(&maze.definition.grid, 'K'), doors);
        Solver { maze: &maze }.solve().expect("must stay solvable");
    }

    #[test]
    fn door_count_never_exceeds_the_auto_cap() {
        // Even with a huge request, no more than MAX_AUTO_DOORS are placed (keeps
        // the key-aware solver within MAX_GATED_FEATURES for validation).
        let maze = make_with_doors(31, 31, 99, 50)
            .generate()
            .expect("should succeed");
        assert!(count_char(&maze.definition.grid, 'D') <= MAX_AUTO_DOORS);
    }

    #[test]
    fn auto_placement_is_deterministic_for_a_fixed_seed() {
        let a = make_with_doors(15, 15, 123, 3)
            .generate()
            .expect("should succeed");
        let b = make_with_doors(15, 15, 123, 3)
            .generate()
            .expect("should succeed");
        assert_eq!(a.definition.grid, b.definition.grid);
    }
}
