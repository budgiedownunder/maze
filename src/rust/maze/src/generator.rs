use rand::seq::SliceRandom;
use rand::rngs::StdRng;
use rand::Rng;
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
    /// Number of **decoy** doors to scatter onto off-spine branches **after**
    /// the maze passes the key-aware solvability check. A decoy is visually
    /// indistinguishable from a real path door — opening one burns a key the
    /// player might have needed for a real door on the spine, and (when the
    /// spare budget is exhausted) strands them. Doors are preferred at corridor
    /// cells (open-degree 2) so they look like they actually gate something;
    /// candidates adjacent to an existing `'K'` are skipped so a decoy isn't
    /// telegraphed by an obvious nearby key. Clamped to [`MAX_AUTO_DOORS`] and
    /// to the available off-spine cells. `None` or `Some(0)` (the default)
    /// places no decoys; the maze is unchanged from the `door_count`-only
    /// result. Placement does **not** re-validate the maze with `solve()` —
    /// decoys sit on side branches and never block the spine, so solvability
    /// is preserved by construction.
    #[serde(default)]
    pub spare_doors: Option<usize>,
    /// Number of **spare keys** to scatter onto off-spine branches **after**
    /// the maze passes the key-aware solvability check. Spare keys give the
    /// player a budget to spend on decoys before they run the risk of
    /// stranding themselves. Candidates skip cells adjacent to any `'D'`
    /// (whether a real path door or a decoy) so the spare doesn't immediately
    /// reveal which adjacent door is real. Clamped to the available off-spine
    /// cells. `None` or `Some(0)` (the default) places no spares.
    #[serde(default)]
    pub spare_keys: Option<usize>,
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
///         spare_doors: None,
///         spare_keys: None,
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
        let spare_doors = opts.spare_doors.unwrap_or(0);
        let spare_keys = opts.spare_keys.unwrap_or(0);

        let total_features = 2 * door_count + spare_doors + spare_keys;
        if total_features > crate::MAX_TOTAL_FEATURES {
            return Err(Error::Generate(format!(
                "requested keys + doors ({total_features}) exceeds the cap ({}): \
                 2 * door_count ({door_count}) + spare_doors ({spare_doors}) + spare_keys ({spare_keys})",
                crate::MAX_TOTAL_FEATURES,
            )));
        }

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

        // max_retries == 0 is a special sentinel that means "don't attempt at all".
        if max_retries == 0 {
            return Err(Error::Generate("max_retries is 0, no attempts made".to_string()));
        }

        // Each attempt carves a fresh perfect maze with the growing-tree algorithm
        // (see `carve`), then solves it to check the spine length. Two retry
        // conditions:
        //   1. finish stayed 'W' — `can_carve` can block all paths to finish when
        //      its neighbours all get carved first; retry with the next RNG draw.
        //   2. spine shorter than min_spine — retry until a long enough path forms.

        let mut last_err = format!(
            "solution length is less than minimum solution length {min_spine}"
        );

        for _ in 0..max_retries {
            let mut grid = carve(
                rows,
                cols,
                &start,
                &finish,
                branch_from_finish,
                RIVER_FACTOR,
                &mut rng,
            );

            // If finish was never carved (can_carve blocked all paths to it), retry.
            if grid[finish.row][finish.col] == 'W' {
                continue;
            }

            grid[start.row][start.col] = 'S';
            grid[finish.row][finish.col] = 'F';

            let maze = Maze::new(MazeDefinition::from_vec(grid));
            match (Solver { maze: &maze }).solve() {
                Ok(solution) if solution.path.points.len() >= min_spine => {
                    let spine_points = &solution.path.points;
                    // Stage 1 — auto-place real keys/doors on the spine and
                    // confirm key-aware solvability. On the rare failure, fall
                    // through to retry with a fresh carve.
                    let mut working = maze.definition.grid.clone();
                    if door_count > 0 {
                        place_keys_and_doors(&mut working, spine_points, door_count, &mut rng);
                        let probe = Maze::new(MazeDefinition::from_vec(working.clone()));
                        match (Solver { maze: &probe }).solve() {
                            Ok(sol2) if sol2.path.points.len() >= min_spine => { /* ok */ }
                            _ => {
                                last_err =
                                    "key/door placement did not yield a solvable maze"
                                        .to_string();
                                continue;
                            }
                        }
                    }
                    // Stage 2 — overlay decoy doors + spare keys onto off-spine
                    // branches. Solvability is preserved by construction:
                    // decoys never sit on the spine and spare keys only loosen
                    // the player's budget, so no second `solve()` is needed.
                    if spare_doors > 0 || spare_keys > 0 {
                        place_spare_keys_and_doors(
                            &mut working,
                            spine_points,
                            spare_doors,
                            spare_keys,
                            &mut rng,
                        );
                    }
                    return Ok(Maze::new(MazeDefinition::from_vec(working)));
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

/// River factor for the growing-tree carve: the probability of extending the
/// **most-recently-carved** cell (a depth-first "river") rather than a **random**
/// active cell (Prim's-style branching). `1.0` is pure recursive backtracking
/// (long winding corridors, dead-end branches skewed toward the finish); `0.0`
/// is pure Prim's (very bushy, short spines). The value below is tuned to flatten
/// the branch-length distribution along the spine while keeping spines long
/// enough for typical `min_spine_length`s and the wall fill close to the
/// recursive-backtracking baseline (~39%).
const RIVER_FACTOR: f64 = 0.8;

/// 4-neighbour offsets (up, down, left, right) used by the carve.
const CARVE_OFFSETS: [(i64, i64); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];

/// Carves a perfect maze (a spanning tree) into a fresh `rows`×`cols` grid using
/// the **growing-tree** algorithm. An active list of carved cells is grown: each
/// step picks one — the newest with probability `river_factor`, otherwise a
/// random active cell — and carves a random carve-able unvisited neighbour;
/// a cell is dropped once it has none left. Picking newest reproduces recursive
/// backtracking; picking random reproduces Prim's; a mix tunes branchiness
/// (see [`RIVER_FACTOR`]).
///
/// `can_carve` admits a neighbour only if it touches no carved cell but its
/// parent, so adjacent passable cells are always tree edges — no loops. Returns
/// the grid as `' '` (passable) / `'W'` (wall): the start is carved and the
/// finish is carved unless it was never reached; neither is marked `S`/`F`.
fn carve(
    rows: usize,
    cols: usize,
    start: &MazePoint,
    finish: &MazePoint,
    branch_from_finish: bool,
    river_factor: f64,
    rng: &mut StdRng,
) -> Vec<Vec<char>> {
    let in_bounds = |r: i64, c: i64| r >= 0 && (r as usize) < rows && c >= 0 && (c as usize) < cols;
    // (nr, nc) may be carved from (fr, fc) only if it touches no carved cell but (fr, fc).
    let can_carve = |grid: &[Vec<char>], fr: usize, fc: usize, nr: usize, nc: usize| -> bool {
        for (dr, dc) in &CARVE_OFFSETS {
            let (rr, cc) = (nr as i64 + dr, nc as i64 + dc);
            if !in_bounds(rr, cc) {
                continue;
            }
            let (rr, cc) = (rr as usize, cc as usize);
            if (rr, cc) != (fr, fc) && grid[rr][cc] != 'W' {
                return false;
            }
        }
        true
    };
    // Carve-able unvisited neighbours of `cell`, shuffled.
    let carveable = |grid: &[Vec<char>], cell: &MazePoint, rng: &mut StdRng| -> Vec<MazePoint> {
        let mut out: Vec<MazePoint> = Vec::new();
        for (dr, dc) in &CARVE_OFFSETS {
            let (nr, nc) = (cell.row as i64 + dr, cell.col as i64 + dc);
            if !in_bounds(nr, nc) {
                continue;
            }
            let (nr, nc) = (nr as usize, nc as usize);
            if grid[nr][nc] == 'W' && can_carve(grid, cell.row, cell.col, nr, nc) {
                out.push(MazePoint { row: nr, col: nc });
            }
        }
        out.shuffle(rng);
        out
    };

    let mut grid = vec![vec!['W'; cols]; rows];
    grid[start.row][start.col] = ' ';
    let mut active: Vec<MazePoint> = vec![start.clone()];
    while !active.is_empty() {
        let idx = if active.len() == 1 || rng.gen_bool(river_factor) {
            active.len() - 1 // newest — depth-first "river"
        } else {
            rng.gen_range(0..active.len()) // random — Prim's-style branching
        };
        let cell = active[idx].clone();
        if let Some(next) = carveable(&grid, &cell, rng).into_iter().next() {
            grid[next.row][next.col] = ' ';
            // Keep the finish a clean dead-end unless branching from it is allowed.
            if branch_from_finish || (next.row, next.col) != (finish.row, finish.col) {
                active.push(next);
            }
        } else {
            active.swap_remove(idx);
        }
    }
    grid
}

/// Maximum number of doors auto-placed at generation. Keeps
/// `keys + doors = 2 * doors ≤ 16` — the key-aware solver's `MAX_TOTAL_FEATURES`
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

/// Iterative DFS over a branch (a subtree hanging off the spine), entered at
/// `start` from `came_from`, returning the **deepest** dead-end leaf and its
/// depth (cells walked from `start`, so a single-cell branch has depth 0). The
/// maze is a perfect maze (a tree), so the branch is a finite subtree of leaves.
fn deepest_leaf(
    grid: &[Vec<char>],
    start: (usize, usize),
    came_from: (usize, usize),
) -> ((usize, usize), usize) {
    let mut best = (start, 0usize);
    let mut stack = vec![(start, came_from, 0usize)];
    while let Some((cur, prev, depth)) = stack.pop() {
        let mut has_child = false;
        for n in passable_neighbours(grid, cur.0, cur.1) {
            if n == prev {
                continue;
            }
            has_child = true;
            stack.push((n, cur, depth + 1));
        }
        if !has_child && depth >= best.1 {
            best = (cur, depth);
        }
    }
    best
}

/// Finds the key cell for the spine segment spanning spine indices `[lo, hi]`:
/// the **deepest** dead-end across all branches hanging off the segment's spine
/// cells, so the key is tucked as far off the through-route as the segment
/// allows. Falls back to a segment spine cell when the segment has no branch.
/// Returns `None` only for an empty segment, which the door spacing prevents.
fn find_key_cell(
    grid: &[Vec<char>],
    spine: &[MazePoint],
    lo: usize,
    hi: usize,
) -> Option<(usize, usize)> {
    if lo > hi {
        return None;
    }
    let mut best: Option<((usize, usize), usize)> = None;
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
            let (cell, depth) = deepest_leaf(grid, nb, (s.row, s.col));
            let length = depth + 1; // include the branch's root cell
            if best.is_none_or(|(_, b)| length > b) {
                best = Some((cell, length));
            }
        }
    }
    if let Some((cell, _)) = best {
        return Some(cell);
    }
    // Fallback: a spine cell in the segment (always passable, not start/finish/door).
    Some((spine[lo].row, spine[lo].col))
}

/// How many cells past a junction a door may be placed. A door is positioned a
/// random `1..=MAX_DOOR_OFFSET` cells *ahead* of its junction (toward the
/// finish) rather than on it, so the junction — and its branch — stays in the
/// segment before the door, available to hold the door's key.
const MAX_DOOR_OFFSET: usize = 3;

/// Returns every off-spine cell that lies on the unique path between a real
/// `'K'` and the spine — the **key tributaries**. A decoy door placed on any
/// of these cells would seal the key (and the spine door it unlocks) behind
/// an unopenable barrier, since the player has no other keys at maze start.
///
/// Walks from each `'K'` outward via BFS, recording a parent pointer at each
/// step, and stops at the first spine cell encountered. The maze is a
/// perfect maze (a spanning tree), so the K→spine path is unique; tracing
/// parent pointers back from that spine cell to the K yields exactly the
/// cells the player must cross. Spine cells themselves are excluded from
/// the returned set — they're already off-limits to decoy placement by the
/// off-spine filter — but the K cell itself is included for symmetry
/// (it's already `'K'`, so the `' '`-only filter excludes it anyway).
fn key_tributary_cells(
    grid: &[Vec<char>],
    spine: &std::collections::HashSet<(usize, usize)>,
) -> std::collections::HashSet<(usize, usize)> {
    use std::collections::{HashMap, HashSet, VecDeque};
    let mut tributary: HashSet<(usize, usize)> = HashSet::new();
    let rows = grid.len();
    let cols = if rows > 0 { grid[0].len() } else { 0 };
    for r in 0..rows {
        for c in 0..cols {
            if grid[r][c] != 'K' || spine.contains(&(r, c)) {
                continue;
            }
            let mut parent: HashMap<(usize, usize), (usize, usize)> = HashMap::new();
            let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
            parent.insert((r, c), (r, c)); // self-parent sentinel
            queue.push_back((r, c));
            while let Some(cell) = queue.pop_front() {
                if spine.contains(&cell) {
                    // Walk parent pointers from this spine cell back to K,
                    // marking each non-spine cell along the way.
                    let mut cur = cell;
                    while parent[&cur] != cur {
                        if !spine.contains(&cur) {
                            tributary.insert(cur);
                        }
                        cur = parent[&cur];
                    }
                    tributary.insert((r, c));
                    break;
                }
                for n in passable_neighbours(grid, cell.0, cell.1) {
                    if let std::collections::hash_map::Entry::Vacant(e) = parent.entry(n) {
                        e.insert(cell);
                        queue.push_back(n);
                    }
                }
            }
        }
    }
    tributary
}

/// Selects up to `requested` (clamped to [`MAX_AUTO_DOORS`]) interior spine cells
/// to host doors, **evenly spread along the spine with a small random jitter**:
/// for `n` doors, target slot centres are placed at `i·span/(n+1)` for
/// `i = 1..=n` across the interior range, each jittered by ± a fraction of the
/// slot. The actual anchor is the nearest **junction** (open-degree ≥ 3) within a
/// half-slot window of the target — so the junction's branch stays in the segment
/// before the door for the key — falling back to the target itself when no
/// junction is close enough. The door then sits a random
/// `1..=`[`MAX_DOOR_OFFSET`] cells *ahead* of the anchor. Doors are kept ≥ 2 spine
/// cells apart (room for each key segment); any candidate that collides is
/// dropped. Returned ascending.
fn select_doors(
    grid: &[Vec<char>],
    spine: &[MazePoint],
    requested: usize,
    rng: &mut StdRng,
) -> Vec<usize> {
    let m = spine.len();
    let cap = requested.min(MAX_AUTO_DOORS);
    if cap == 0 || m < 5 {
        return Vec::new();
    }
    let lo = 2usize;
    let hi = m - 3; // highest interior index (m-2 is the finish's neighbour)
    if hi < lo {
        return Vec::new();
    }
    let span = hi - lo + 1; // number of interior anchor positions
    let degree = |j: usize| open_degree(grid, spine[j].row, spine[j].col);

    let slot = span / (cap + 1); // ~spacing between consecutive target centres
    let jitter_radius = slot / 3; // ± a third of the slot keeps doors inside their slot
    let search_radius = (slot / 2).max(1); // how far from the target we'll look for a junction

    let mut doors: Vec<usize> = Vec::new();
    for i in 1..=cap {
        let target_center = lo + (i * span) / (cap + 1);
        let jitter: isize = if jitter_radius > 0 {
            rng.gen_range(0..=2 * jitter_radius) as isize - jitter_radius as isize
        } else {
            0
        };
        let target = (target_center as isize + jitter)
            .clamp(lo as isize, hi as isize) as usize;

        // Nearest junction within `search_radius` cells of the target — prefer the
        // exact target, then expand outward symmetrically.
        let mut anchor: Option<usize> = None;
        for r in 0..=search_radius {
            let mut candidates: Vec<usize> = Vec::with_capacity(2);
            if r == 0 {
                candidates.push(target);
            } else {
                if target + r <= hi {
                    candidates.push(target + r);
                }
                if target >= lo + r {
                    candidates.push(target - r);
                }
            }
            for c in candidates {
                if degree(c) >= 3 {
                    anchor = Some(c);
                    break;
                }
            }
            if anchor.is_some() {
                break;
            }
        }
        // Fall back to a corridor cell at the target if no junction is in range.
        let anchor = anchor.unwrap_or(target);

        // Door sits a random few cells ahead of the anchor (clamped to interior).
        let offset = rng.gen_range(1..=MAX_DOOR_OFFSET);
        let door = (anchor + offset).min(hi);
        if door > anchor && doors.iter().all(|&d| door.abs_diff(d) >= 2) {
            doors.push(door);
        }
    }
    doors.sort_unstable();
    doors.dedup();
    doors
}

/// Places up to `requested` doors (each with one preceding key) onto the maze's
/// start→finish `spine`, mutating `grid` in place. Doors are chosen by
/// [`select_doors`] (evenly spread along the spine with a small random jitter,
/// snapping to the nearest junction and sitting a few cells ahead of it); each
/// door's key is hidden at the deepest dead-end of a branch in the spine segment
/// before it (see [`find_key_cell`]) — typically the anchoring junction's own
/// branch. The count is clamped to the available cells and to [`MAX_AUTO_DOORS`];
/// the caller validates the result with the key-aware solver.
fn place_keys_and_doors(
    grid: &mut [Vec<char>],
    spine: &[MazePoint],
    requested: usize,
    rng: &mut StdRng,
) {
    if requested == 0 || spine.len() < 5 {
        return;
    }
    let doors = select_doors(grid, spine, requested, rng);
    if doors.is_empty() {
        return;
    }
    for &j in &doors {
        grid[spine[j].row][spine[j].col] = 'D';
    }
    // One key per door, in the spine segment before it. Keys are interchangeable,
    // so collecting one per segment leaves the player exactly one in hand at each
    // door.
    let mut prev_boundary = 0usize; // spine index of the previous door (0 = start)
    for &dj in &doors {
        if let Some((r, c)) = find_key_cell(grid, spine, prev_boundary + 1, dj - 1) {
            grid[r][c] = 'K';
        }
        prev_boundary = dj;
    }
}

/// Overlays decoy doors + spare keys onto **off-spine** branches of a maze
/// that has already passed the key-aware solvability check (see
/// [`place_keys_and_doors`] for stage 1). Mutates `grid` in place.
///
/// Decoy doors go on `' '` corridor cells (open-degree 2) preferentially —
/// they read as gates with something behind them — falling back to dead-end
/// leaves (open-degree 1) if corridors are exhausted; junction cells
/// (open-degree ≥ 3) are excluded because a door at a fork looks unnatural
/// compared to the spine doors (which sit a few cells past their junction).
/// Candidates adjacent to an existing `'K'` are skipped so the decoy isn't
/// telegraphed; candidates adjacent to an already-placed `'D'` are also
/// skipped to avoid door clumping. Critically, **key-tributary cells**
/// (off-spine cells on the unique path between a real `'K'` and the spine)
/// are excluded — a decoy on such a cell would seal the key (and the spine
/// door it unlocks) behind an unopenable barrier.
///
/// Spare keys then go on the remaining off-spine `' '` cells, skipping any
/// cell adjacent to a `'D'` — so the spare key doesn't accidentally identify
/// a nearby door as real and undercut the bait.
///
/// Both counts are clamped to the cells the candidate filters yield;
/// `requested_doors` is additionally clamped to [`MAX_AUTO_DOORS`]. Solvability
/// is preserved by construction (decoys never sit on the spine or on a
/// key tributary; spare keys only loosen the player's key budget), so
/// the caller does **not** re-run `solve()`.
fn place_spare_keys_and_doors(
    grid: &mut [Vec<char>],
    spine: &[MazePoint],
    requested_doors: usize,
    requested_keys: usize,
    rng: &mut StdRng,
) {
    if requested_doors == 0 && requested_keys == 0 {
        return;
    }

    use std::collections::HashSet;
    let spine_set: HashSet<(usize, usize)> = spine.iter().map(|p| (p.row, p.col)).collect();

    let collect_off_spine_empty = |grid: &[Vec<char>]| -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for (r, row) in grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                if ch == ' ' && !spine_set.contains(&(r, c)) {
                    out.push((r, c));
                }
            }
        }
        out
    };

    let is_adjacent_to = |grid: &[Vec<char>], r: usize, c: usize, target: char| -> bool {
        passable_neighbours(grid, r, c)
            .into_iter()
            .any(|(nr, nc)| grid[nr][nc] == target)
    };

    // ── Stage A — decoy doors ─────────────────────────────────────────────────
    if requested_doors > 0 {
        let door_cap = requested_doors.min(MAX_AUTO_DOORS);
        let off_spine = collect_off_spine_empty(grid);
        // Cells on the unique path from each real K to the spine are
        // off-limits — a decoy here would seal the key.
        let tributary = key_tributary_cells(grid, &spine_set);

        // Partition by open-degree: corridor cells preferred over leaves;
        // junction cells (degree ≥ 3) excluded.
        let mut corridors: Vec<(usize, usize)> = Vec::new();
        let mut leaves: Vec<(usize, usize)> = Vec::new();
        for cell in off_spine {
            if tributary.contains(&cell) {
                continue;
            }
            match open_degree(grid, cell.0, cell.1) {
                2 => corridors.push(cell),
                1 => leaves.push(cell),
                _ => { /* skip — junction or isolated */ }
            }
        }
        corridors.shuffle(rng);
        leaves.shuffle(rng);

        let mut placed: usize = 0;
        for cell in corridors.into_iter().chain(leaves) {
            if placed >= door_cap {
                break;
            }
            // Skip adjacency to keys (telegraphed bait) and to other doors
            // (clumping reads poorly).
            if is_adjacent_to(grid, cell.0, cell.1, 'K')
                || is_adjacent_to(grid, cell.0, cell.1, 'D')
            {
                continue;
            }
            grid[cell.0][cell.1] = 'D';
            placed += 1;
        }
    }

    // ── Stage B — spare keys ──────────────────────────────────────────────────
    if requested_keys > 0 {
        // Re-enumerate: the previous stage consumed some cells.
        let mut off_spine = collect_off_spine_empty(grid);
        off_spine.shuffle(rng);

        let mut placed: usize = 0;
        for cell in off_spine {
            if placed >= requested_keys {
                break;
            }
            if is_adjacent_to(grid, cell.0, cell.1, 'D') {
                continue;
            }
            grid[cell.0][cell.1] = 'K';
            placed += 1;
        }
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
                spare_doors: None,
                spare_keys: None,
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
    fn huge_door_count_is_rejected_at_the_entry_point() {
        // door_count alone contributes `2 * door_count` to the K+D budget (one
        // K and one D each); anything past `MAX_AUTO_DOORS` (= 8) is already
        // past the cap and gets refused up front rather than silently
        // clamped.
        let result = make_with_doors(31, 31, 99, 50).generate();
        assert!(result.is_err(), "huge door_count must error, not clamp");
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

    // --- Spare-key / spare-door overlay (decoys + safety budget) ---

    fn make_with_doors_and_spares(
        rows: usize,
        cols: usize,
        seed: u64,
        doors: usize,
        spare_doors: Option<usize>,
        spare_keys: Option<usize>,
    ) -> Generator {
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
                spare_doors,
                spare_keys,
            },
        }
    }

    /// Lock-blind spine cells for a maze with K/D temporarily stripped.
    fn spine_cells_of(grid: &[Vec<char>]) -> std::collections::HashSet<(usize, usize)> {
        let mut bare = grid.to_vec();
        for row in bare.iter_mut() {
            for cell in row.iter_mut() {
                if *cell == 'D' || *cell == 'K' {
                    *cell = ' ';
                }
            }
        }
        let bare_maze = Maze::new(MazeDefinition::from_vec(bare));
        Solver { maze: &bare_maze }
            .solve()
            .expect("bare maze solvable")
            .path
            .points
            .iter()
            .map(|p| (p.row, p.col))
            .collect()
    }

    #[test]
    fn no_spares_produces_byte_identical_grid_to_unspecified_spares() {
        // Defaulted (None, None) spares must consume no rng draws, so the
        // generated grid is byte-identical to the request with spares
        // disabled. This guards downstream determinism: existing tests + UI
        // consumers that don't yet pass spare fields keep their current
        // output.
        let a = make_with_doors(15, 15, 7, 3)
            .generate()
            .expect("baseline succeeds");
        let b = make_with_doors_and_spares(15, 15, 7, 3, None, None)
            .generate()
            .expect("explicit-None spares succeeds");
        let c = make_with_doors_and_spares(15, 15, 7, 3, Some(0), Some(0))
            .generate()
            .expect("zero spares succeeds");
        assert_eq!(a.definition.grid, b.definition.grid);
        assert_eq!(a.definition.grid, c.definition.grid);
    }

    #[test]
    fn spare_doors_sit_off_spine_in_corridor_or_leaf_cells() {
        let maze = make_with_doors_and_spares(21, 21, 17, 3, Some(4), None)
            .generate()
            .expect("should succeed");
        let grid = &maze.definition.grid;
        let spine = spine_cells_of(grid);
        // Walk every D cell. The real path doors are on the spine; spare
        // (decoy) doors must be OFF the spine, and on cells of open-degree 1
        // or 2 (junctions are excluded as door candidates).
        let mut spare_door_count = 0;
        for (r, row) in grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                if ch != 'D' || spine.contains(&(r, c)) {
                    continue;
                }
                spare_door_count += 1;
                let deg = open_degree(grid, r, c);
                assert!(
                    deg == 1 || deg == 2,
                    "spare door at ({r},{c}) has open-degree {deg}; expected 1 or 2"
                );
            }
        }
        assert!(spare_door_count > 0, "expected at least one spare door");
    }

    #[test]
    fn spare_doors_are_not_adjacent_to_an_existing_key() {
        // Telegraph-prevention: a decoy door right next to a real key would
        // make the bait too obvious.
        let maze = make_with_doors_and_spares(21, 21, 19, 3, Some(4), None)
            .generate()
            .expect("should succeed");
        let grid = &maze.definition.grid;
        let spine = spine_cells_of(grid);
        for (r, row) in grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                if ch != 'D' || spine.contains(&(r, c)) {
                    continue;
                }
                for (nr, nc) in passable_neighbours(grid, r, c) {
                    assert_ne!(
                        grid[nr][nc], 'K',
                        "spare door at ({r},{c}) is adjacent to a key at ({nr},{nc})"
                    );
                }
            }
        }
    }

    #[test]
    fn spare_keys_sit_off_spine_and_not_adjacent_to_any_door() {
        // Real keys (from `place_keys_and_doors`) sit at deepest dead-ends of
        // the segment before each spine door — those are placed by stage 1,
        // before spares. After spares, the OFF-SPINE 'K' cells include both
        // real keys (which were placed by stage 1) and spare keys (which were
        // placed by stage 2). We just verify the spare invariant on every
        // off-spine 'K' the spare stage *could have* added — i.e. the
        // adjacency rule holds globally.
        let maze = make_with_doors_and_spares(21, 21, 23, 2, None, Some(5))
            .generate()
            .expect("should succeed");
        let grid = &maze.definition.grid;
        let spine = spine_cells_of(grid);
        // No K cell on the spine (real keys go in branches, spare keys go
        // off-spine).
        for (r, row) in grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                if ch == 'K' {
                    assert!(
                        !spine.contains(&(r, c)),
                        "key at ({r},{c}) sits on the spine"
                    );
                }
            }
        }
        // Spare keys' adjacency rule: no off-spine K is next to a D. (Real
        // keys also obey this in the typical case — they're at deepest
        // dead-ends, far from doors — so the global check is safe here.)
        for (r, row) in grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                if ch != 'K' {
                    continue;
                }
                for (nr, nc) in passable_neighbours(grid, r, c) {
                    assert_ne!(
                        grid[nr][nc], 'D',
                        "key at ({r},{c}) is adjacent to a door at ({nr},{nc}) — \
                         spare placement should have skipped this candidate"
                    );
                }
            }
        }
    }

    #[test]
    fn repro_spare_doors_can_seal_off_real_keys() {
        // Tests that `solve()` does not fail because a decoy door is placed 
        // on an off-spine branch that contains a real key — sealing the key
        // (and hence the spine door it unlocks) behind an unopenable
        // decoy.
        let mut failures: Vec<u64> = Vec::new();
        for seed in 0u64..200 {
            let gen = make_with_doors_and_spares(15, 15, seed, 3, Some(3), None);
            let maze = match gen.generate() {
                Ok(m) => m,
                Err(_) => continue, // a generation-time error is a separate failure mode
            };
            if (Solver { maze: &maze }).solve().is_err() {
                failures.push(seed);
            }
        }
        assert!(
            failures.is_empty(),
            "{} of 200 seeds produced an unsolvable maze: {failures:?}",
            failures.len()
        );
    }

    #[test]
    fn repro_spare_doors_with_spare_keys_does_not_seal_off_real_keys() {
        // Sibling to the above — same scenario but with spare_keys=2
        // mixed in. Spare keys land in stage B (after decoys) and can sit
        // anywhere off-spine; the decoy-tributary exclusion still has to
        // hold even when there are spare keys in the maze.
        let mut failures: Vec<u64> = Vec::new();
        for seed in 0u64..200 {
            let gen = make_with_doors_and_spares(15, 15, seed, 3, Some(3), Some(2));
            let maze = match gen.generate() {
                Ok(m) => m,
                Err(_) => continue,
            };
            if (Solver { maze: &maze }).solve().is_err() {
                failures.push(seed);
            }
        }
        assert!(
            failures.is_empty(),
            "{} of 200 seeds produced an unsolvable maze: {failures:?}",
            failures.len()
        );
    }

    #[test]
    fn solvability_sweep_across_spare_configs() {
        // Broader cover for the spare-keys + decoy-doors path tests
        // and asserts every generated maze is solvable.
        //
        // Smaller grids run with more seeds (cheap; surface lots of
        // topology shapes); larger grids run with fewer seeds but each
        // generation exercises far more branches and tributaries, so
        // they're better at flushing rare placement edge cases. Configs
        // exercise:
        //   - several grid sizes from 10×10 up to 50×50
        //   - decoys with NO real doors (pure-bait mazes)
        //   - spare keys with NO decoys (player gets a safety budget)
        //   - the K+D = 16 cap (boundary case)
        //   - heavy decoys + heavy spare-keys mix (just under the cap)
        //   - large mazes with the feature count near the cap, since
        //     bigger topology + more features = more placement
        //     permutations the tributary check has to be right for
        // Per-config failure lists are reported separately so a future
        // regression at one config is obvious.
        struct Cfg {
            rows: usize,
            cols: usize,
            doors: usize,
            spare_doors: Option<usize>,
            spare_keys: Option<usize>,
            seeds: u64,
        }
        let cfgs = [
            Cfg { rows: 10, cols: 10, doors: 2, spare_doors: Some(2), spare_keys: Some(2), seeds: 100 },
            Cfg { rows: 15, cols: 15, doors: 3, spare_doors: Some(3), spare_keys: Some(2), seeds: 100 },
            Cfg { rows: 21, cols: 21, doors: 4, spare_doors: Some(2), spare_keys: Some(4), seeds: 100 },
            Cfg { rows: 15, cols: 15, doors: 0, spare_doors: Some(3), spare_keys: Some(3), seeds: 100 },
            Cfg { rows: 10, cols: 10, doors: 2, spare_doors: Some(0), spare_keys: Some(4), seeds: 100 },
            Cfg { rows: 15, cols: 15, doors: 8, spare_doors: Some(0), spare_keys: Some(0), seeds: 100 },
            Cfg { rows: 15, cols: 15, doors: 1, spare_doors: Some(6), spare_keys: Some(8), seeds: 100 },
            // Larger grids — richer topology means more candidate branches
            // and tributaries, so more chances for a placement edge case.
            // Fewer seeds per config to keep total runtime bounded.
            Cfg { rows: 30, cols: 30, doors: 3, spare_doors: Some(3), spare_keys: Some(2), seeds: 50 },
            Cfg { rows: 30, cols: 30, doors: 4, spare_doors: Some(4), spare_keys: Some(4), seeds: 50 },
            Cfg { rows: 40, cols: 40, doors: 4, spare_doors: Some(4), spare_keys: Some(0), seeds: 25 },
            Cfg { rows: 50, cols: 50, doors: 4, spare_doors: Some(4), spare_keys: Some(4), seeds: 25 },
        ];
        let mut total_failures: usize = 0;
        let mut per_cfg_report: Vec<String> = Vec::new();
        for cfg in &cfgs {
            let mut failures: Vec<u64> = Vec::new();
            for seed in 0u64..cfg.seeds {
                let gen = make_with_doors_and_spares(
                    cfg.rows, cfg.cols, seed, cfg.doors, cfg.spare_doors, cfg.spare_keys,
                );
                let maze = match gen.generate() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                if (Solver { maze: &maze }).solve().is_err() {
                    failures.push(seed);
                }
            }
            if !failures.is_empty() {
                total_failures += failures.len();
                per_cfg_report.push(format!(
                    "({}x{}, doors={}, spare_doors={:?}, spare_keys={:?}): {} fails — {failures:?}",
                    cfg.rows, cfg.cols, cfg.doors, cfg.spare_doors, cfg.spare_keys, failures.len(),
                ));
            }
        }
        assert_eq!(
            total_failures, 0,
            "{} unsolvable generations across the sweep:\n  {}",
            total_failures,
            per_cfg_report.join("\n  ")
        );
    }

    #[test]
    fn generated_maze_never_exceeds_total_key_door_cap() {
        // Structural invariant: regardless of (door_count, spare_doors,
        // spare_keys) — provided the request itself passes the up-front
        // total-features check — the generated maze must contain at most
        // `MAX_TOTAL_FEATURES` (= 16) total K+D cells. Above 16 the
        // key-aware solver falls back to lock-blind Lee, which would
        // misrepresent strand reachability and let unsolvable mazes
        // through; the cap is the keystone of the gameplay invariant.
        let cfgs = [
            (15, 15, 3, Some(3), Some(2)),  // 6+3+2=11
            (21, 21, 4, Some(2), Some(4)),  // 8+2+4=14
            (15, 15, 8, Some(0), Some(0)),  // 16+0+0=16 (boundary)
            (15, 15, 1, Some(6), Some(8)),  // 2+6+8=16 (boundary)
        ];
        for (rows, cols, doors, sd, sk) in cfgs {
            for seed in 0u64..40 {
                let gen = make_with_doors_and_spares(rows, cols, seed, doors, sd, sk);
                let maze = match gen.generate() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let grid = &maze.definition.grid;
                let k_count = count_char(grid, 'K');
                let d_count = count_char(grid, 'D');
                assert!(
                    k_count + d_count <= crate::MAX_TOTAL_FEATURES,
                    "({rows}x{cols}, doors={doors}, sd={sd:?}, sk={sk:?}, seed={seed}): \
                     K={k_count} + D={d_count} = {} exceeds cap ({})",
                    k_count + d_count,
                    crate::MAX_TOTAL_FEATURES,
                );
            }
        }
    }

    #[test]
    fn spare_doors_clamp_to_max_auto_doors_ceiling() {
        // Over-request spare doors (right up to the K+D budget); the placement
        // must clamp to MAX_AUTO_DOORS independent of however many real doors
        // were placed.
        let maze = make_with_doors_and_spares(25, 25, 31, 0, Some(crate::MAX_TOTAL_FEATURES), None)
            .generate()
            .expect("should succeed");
        let grid = &maze.definition.grid;
        let total_doors = count_char(grid, 'D');
        assert!(
            total_doors <= MAX_AUTO_DOORS,
            "spare-door placement must clamp to MAX_AUTO_DOORS, got {total_doors}"
        );
    }

    #[test]
    fn generate_errors_when_total_features_exceeds_cap() {
        // 2 * door_count + spare_doors + spare_keys > MAX_TOTAL_FEATURES must
        // be rejected at the entry point so over-cap requests never silently
        // get partially clamped into a smaller-than-asked maze. Choose a
        // request that just exceeds the cap: door_count=8 (16) + spare_keys=1
        // = 17. Same shape as the Gen 1 bug that motivated the cap.
        let result = make_with_doors_and_spares(21, 21, 7, 8, None, Some(1)).generate();
        match result {
            Ok(_) => panic!("expected over-cap error, got Ok"),
            Err(e) => {
                let msg = format!("{e}");
                assert!(
                    msg.contains("exceeds the cap"),
                    "expected over-cap error, got: {msg}"
                );
            }
        }
    }

    #[test]
    fn spare_placement_is_deterministic_for_a_fixed_seed() {
        let a = make_with_doors_and_spares(21, 21, 99, 3, Some(3), Some(3))
            .generate()
            .expect("should succeed");
        let b = make_with_doors_and_spares(21, 21, 99, 3, Some(3), Some(3))
            .generate()
            .expect("should succeed");
        assert_eq!(a.definition.grid, b.definition.grid);
    }

    #[test]
    fn spare_placement_preserves_solvability() {
        // After overlaying spares, the maze must still solve. With
        // keys+doors potentially > MAX_TOTAL_FEATURES the solver falls back
        // to lock-blind — either way `solve()` returns `Ok` because the spine
        // is open and unaltered by the spare overlay.
        let maze = make_with_doors_and_spares(21, 21, 41, 3, Some(4), Some(4))
            .generate()
            .expect("should succeed");
        Solver { maze: &maze }
            .solve()
            .expect("overlaid maze must remain solvable");
        // Exactly one start and one finish; no decoy clobbered them.
        assert_eq!(count_char(&maze.definition.grid, 'S'), 1);
        assert_eq!(count_char(&maze.definition.grid, 'F'), 1);
    }

    #[test]
    fn spare_keys_only_places_some_off_spine_keys() {
        // No spare doors, just spare keys: a generous safety budget for any
        // future decoys (or simply a no-op buffer in a door-free maze).
        let maze = make_with_doors_and_spares(15, 15, 53, 0, None, Some(3))
            .generate()
            .expect("should succeed");
        let grid = &maze.definition.grid;
        let spine = spine_cells_of(grid);
        let mut off_spine_keys = 0;
        for (r, row) in grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                if ch == 'K' && !spine.contains(&(r, c)) {
                    off_spine_keys += 1;
                }
            }
        }
        assert!(
            (1..=3).contains(&off_spine_keys),
            "expected 1..=3 spare keys, got {off_spine_keys}"
        );
        assert_eq!(count_char(grid, 'D'), 0, "no doors when door_count = 0");
    }

    #[test]
    fn spare_doors_only_places_some_off_spine_doors() {
        // No spare keys, just decoys. With door_count=2 the maze has real
        // path doors too, but the spare doors must be off the spine.
        let maze = make_with_doors_and_spares(21, 21, 61, 2, Some(3), None)
            .generate()
            .expect("should succeed");
        let grid = &maze.definition.grid;
        let spine = spine_cells_of(grid);
        let mut off_spine_doors = 0;
        for (r, row) in grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                if ch == 'D' && !spine.contains(&(r, c)) {
                    off_spine_doors += 1;
                }
            }
        }
        assert!(
            (1..=3).contains(&off_spine_doors),
            "expected 1..=3 spare doors, got {off_spine_doors}"
        );
    }

    #[test]
    fn places_door_ahead_of_a_junction_with_the_key_in_the_junction_branch() {
        // Spine runs along row 0 (S..F). One junction at (0,2) has a 2-deep
        // branch down to (2,2); the rest of the spine is plain corridor. With one
        // door requested, it is placed a random few cells AHEAD of the junction
        // (a corridor cell in cols 3–5, never on the junction itself), and the
        // key goes into the junction's branch — its deepest dead-end (2,2) —
        // which only stays reachable because the door sits past it.
        #[rustfmt::skip]
        let grid = vec![
            vec!['S', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', 'F'],
            vec!['W', 'W', ' ', 'W', 'W', 'W', 'W', 'W', 'W', 'W'],
            vec!['W', 'W', ' ', 'W', 'W', 'W', 'W', 'W', 'W', 'W'],
        ];
        let maze = Maze::from_vec(grid.clone());
        let spine = Solver { maze: &maze }
            .solve()
            .expect("bare maze solvable")
            .path
            .points;
        let mut placed = grid.clone();
        let mut rng = StdRng::seed_from_u64(1);
        place_keys_and_doors(&mut placed, &spine, 1, &mut rng);

        assert_eq!(count_char(&placed, 'D'), 1);
        assert_eq!(count_char(&placed, 'K'), 1);
        // The key is in the junction's branch, at its deepest dead-end.
        assert_eq!(placed[2][2], 'K');
        // The door is NOT on the junction — it sits ahead of it, in the corridor
        // (exact column depends on the even-spread target plus jitter and offset).
        assert_ne!(placed[0][2], 'D');
        assert_eq!(
            (3..=7).filter(|&c| placed[0][c] == 'D').count(),
            1,
            "door should sit on the spine ahead of the junction"
        );

        // The placed maze is key-aware solvable.
        let placed_maze = Maze::from_vec(placed);
        Solver { maze: &placed_maze }
            .solve()
            .expect("placed maze must be solvable");
    }
}
