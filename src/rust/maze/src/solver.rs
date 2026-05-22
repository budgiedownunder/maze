use std::collections::{HashMap, HashSet, VecDeque};

use data_model::{Maze, MazeCellState, MazePoint};
use crate::{Error, MazePath, MazePointOffset, MazeSolution};

/// Upper bound on `(#keys + #doors)` for which the key-aware search runs. The
/// search is exponential in that sum (it tracks which keys are held and which
/// doors are open as bitmasks), so beyond this a pathological maze could be
/// slow. Hand-authored mazes are nowhere near it; past the bound we fall back
/// to the lock-blind Lee solve rather than risk a long search.
const MAX_GATED_FEATURES: usize = 16;

/// Identifies a node in the key-aware search: a cell plus the history that
/// decides what's passable from here — which keys have been collected and which
/// doors have been opened (each a bitmask over the grid's indexed `K` / `D`
/// cells). Two visits to the same cell with different histories are genuinely
/// different nodes, which is what lets the search know whether a door ahead can
/// be opened. Keys-in-hand is derived, never stored:
/// `collected.count_ones() - opened.count_ones()`.
type GatedState = (usize, usize, u32, u32);

#[allow(dead_code)]
/// Represents a maze solver
pub struct Solver<'a> {
    /// Maze reference
    pub maze: &'a Maze,
}

impl Solver<'_> {
    fn is_valid(&self, pt: &MazePoint) -> bool {
        self.maze.definition.is_valid(pt)
    }

    #[allow(clippy::cast_abs_to_unsigned)]
    fn unsigned_abs_i32(value: i32) -> usize {
        value.abs() as usize
    }

    fn calc_location(&self, pt: &MazePoint, offset: &MazePointOffset) -> Result<MazePoint, Error> {
        if offset.row < 0 && Self::unsigned_abs_i32(offset.row) > pt.row {
            return Err(Error::Solve("location is out of bounds".to_string()));
        }
        if offset.col < 0 && Self::unsigned_abs_i32(offset.col) > pt.col {
            return Err(Error::Solve("location is out of bounds".to_string()));
        }
        let pt_check = {
            // Supress clippy's comparison_chain lint as "if chain"s are ok and
            // calc_location() is performance-critical during solve
            // (see: https://github.com/rust-lang/rust-clippy/issues/5354)
            #[allow(clippy::comparison_chain)]
            MazePoint {
                row: if offset.row >= 0 {
                    pt.row + offset.row as usize
                } else {
                    pt.row - (-offset.row) as usize
                },
                col: if offset.col >= 0 {
                    pt.col + offset.col as usize
                } else {
                    pt.col - (-offset.col) as usize
                },
            }
        };

        if !self.is_valid(&pt_check) {
            return Err(Error::Solve("location is out of bounds".to_string()));
        }
        Ok(pt_check)
    }

    fn get_lee_solution(
        &self,
        grid_state: &[Vec<MazeCellState>],
        start: &MazePoint,
        end: &MazePoint,
        offsets: &[MazePointOffset],
    ) -> Result<MazeSolution, Error> {
        let mut points: Vec<MazePoint> = vec![];
        if grid_state[end.row][end.col].step_value().is_none() {
            return Err(Error::Solve(
                "solution path not found (end point not processed)".to_string(),
            ));
        }
        let mut step_pt: MazePoint = end.clone();
        points.push(end.clone());
        loop {
            if let MazeCellState::SolutionStep { value } = grid_state[step_pt.row][step_pt.col] {
                let mut found_neighbour = false;
                for offset in offsets.iter() {
                    if let Ok(offset_pt) = self.calc_location(&step_pt, offset) {
                        let offset_pt_step_value =
                            grid_state[offset_pt.row][offset_pt.col].step_value();
                        if let Some(offset_pt_value) = offset_pt_step_value {
                            if step_pt == *start {
                                points.reverse();
                                return Ok(MazeSolution::new(MazePath::new(points)));
                            }
                            if offset_pt_value == value - 1 {
                                step_pt = offset_pt;
                                points.push(step_pt.clone());
                                found_neighbour = true;
                                break;
                            }
                        }
                    }
                }
                if !found_neighbour {
                    return Err(Error::Solve(format!(
                        "solution path not found (no path sequence neighbour exists for point {step_pt})"
                    )));
                }
            }
        }
    }

    // Assumes 'start' and 'end' are valid
    fn solve_lee(&self, start: &MazePoint, end: &MazePoint) -> Result<MazeSolution, Error> {
        let mut q: VecDeque<MazePoint> = VecDeque::new();
        let mut grid_state = self.maze.definition.to_state();
        let offsets = [
            MazePointOffset { row: -1, col: 0 }, // Up
            MazePointOffset { row: 0, col: -1 }, // Left
            MazePointOffset { row: 1, col: 0 },  // Down
            MazePointOffset { row: 0, col: 1 },  // Right
        ];

        q.push_back(start.clone());
        grid_state[start.row][start.col] = MazeCellState::SolutionStep { value: 0 };
        while !q.is_empty() {
            if let Some(pt) = q.pop_front() {
                if let Some(value) = grid_state[pt.row][pt.col].step_value() {
                    for offset in offsets.iter() {
                        if let Ok(offset_pt) = self.calc_location(&pt, offset) {
                            if grid_state[offset_pt.row][offset_pt.col] == MazeCellState::Empty {
                                grid_state[offset_pt.row][offset_pt.col] =
                                MazeCellState::SolutionStep { value: value + 1 };
                                if offset_pt == *end {
                                    return self.get_lee_solution(
                                        &grid_state,
                                        start,
                                        end,
                                        &offsets,
                                    );
                                }
                                q.push_back(offset_pt.clone());
                            }
                        }
                    }
                }
            }
        }

        Err(Error::Solve("no solution found".to_string()))
    }
    /// Attempts to solve the path between the start and end point defined within the maze referenced by the solver instance
    ///
    /// The returned path is **key-aware**: if the maze contains doors (`D`), the
    /// solution is the **shortest** route that actually completes the maze given
    /// key→door gating — detouring to collect the keys it needs, treating a door
    /// as passable once a key is in hand, irrespective of how many doors it ends
    /// up passing through. A maze whose finish is sealed behind a door with no
    /// reachable key returns an error, where the old lock-blind solve would have
    /// reported a (un-walkable) route. Mazes with no doors take the original
    /// shortest-path solve unchanged. Because a key route may need to backtrack
    /// (collect a key, return through a junction), the path can revisit a cell,
    /// so it is a *walk* rather than a strictly simple path.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the solution if successful, or a `Error` if an error occurs
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::{Maze, MazePoint};
    /// use maze::{MazeSolver, Solver};
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', 'W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W', ' '],
    ///    vec![' ', ' ', ' ', 'W', 'F'],
    ///    vec!['W', ' ', 'W', ' ', ' '],
    ///    vec![' ', ' ', ' ', 'W', ' '],
    ///    vec!['W', 'W', ' ', ' ', ' '],
    ///    vec!['W', 'W', ' ', 'W', ' '],
    /// ];
    /// let solver = Solver {
    ///     maze: &Maze::from_vec(grid),
    /// };
    /// let result = solver.solve();
    /// match result {
    ///    Ok(solution) => {
    ///       println!("Successfully solved maze, solution path => {}", solution.path);
    ///    }
    ///    Err(error) => {
    ///        panic!(
    ///            "failed to solve maze => {}",
    ///           error
    ///        );
    ///    }
    /// }
    /// ```
    pub fn solve(&self) -> Result<MazeSolution, Error> {
        let start = self.maze.definition.get_start();
        let finish = self.maze.definition.get_finish();
        if start.is_none() {
            return Err(Error::Solve(
                "no start cell found within maze".to_string(),
            ));
        }
        if finish.is_none() {
            return Err(Error::Solve(
                "no finish cell found within maze".to_string(),
            ));
        }
        let start_pt: MazePoint = start.unwrap();
        let finish_pt: MazePoint = finish.unwrap();
        if start_pt == finish_pt {
            let points = vec![start_pt];
            return Ok(MazeSolution::new(MazePath::new(points)));
        }
        // Doors gate the maze: the lock-blind Lee solve treats them as open, so
        // it can report a maze as solvable when the finish is actually sealed
        // behind a door with no reachable key. When the grid contains any door,
        // run the key-aware shortest-path search; otherwise the original Lee
        // solve, which is both the identical result and the fast path for the
        // common no-door maze.
        let has_door = self
            .maze
            .definition
            .grid
            .iter()
            .any(|row| row.contains(&'D'));
        if has_door {
            self.solve_with_keys_and_doors(&start_pt, &finish_pt)
        } else {
            self.solve_lee(&start_pt, &finish_pt)
        }
    }

    /// Key-aware solve over the state space `(cell, keys-collected, doors-opened)`
    /// via a breadth-first search, so the first time the finish cell is reached
    /// is by the **shortest** completing route (fewest steps). Doors are not
    /// minimised — a door is simply passable once a key is in hand. Returns that
    /// route, or `Error::Solve` if the finish can't be reached given the gating.
    ///
    /// Assumes `start` and `end` are valid and distinct (the caller guarantees
    /// this) and that the grid contains at least one door.
    fn solve_with_keys_and_doors(
        &self,
        start: &MazePoint,
        end: &MazePoint,
    ) -> Result<MazeSolution, Error> {
        let grid = &self.maze.definition.grid;

        // Index every key and door cell to a bit position in the two masks.
        let mut key_bit: HashMap<(usize, usize), u32> = HashMap::new();
        let mut door_bit: HashMap<(usize, usize), u32> = HashMap::new();
        for (r, row) in grid.iter().enumerate() {
            for (c, &ch) in row.iter().enumerate() {
                match ch {
                    'K' => {
                        let next = key_bit.len() as u32;
                        key_bit.insert((r, c), next);
                    }
                    'D' => {
                        let next = door_bit.len() as u32;
                        door_bit.insert((r, c), next);
                    }
                    _ => {}
                }
            }
        }

        // The search is exponential in (#keys + #doors). Hand-authored mazes are
        // tiny; for a pathological count, fall back to the lock-blind Lee solve
        // rather than risk a long search (see MAX_GATED_FEATURES).
        if key_bit.len() + door_bit.len() > MAX_GATED_FEATURES {
            return self.solve_lee(start, end);
        }

        // BFS: every move costs one step, so the first time a state on the
        // finish cell is dequeued it was reached by a shortest route. A state
        // carries its full history (keys collected, doors opened) so revisiting
        // a cell with a different history is a distinct node.
        let start_state: GatedState = (start.row, start.col, 0, 0);
        let mut visited: HashSet<GatedState> = HashSet::new();
        let mut prev: HashMap<GatedState, GatedState> = HashMap::new();
        let mut queue: VecDeque<GatedState> = VecDeque::new();
        visited.insert(start_state);
        queue.push_back(start_state);

        while let Some(state) = queue.pop_front() {
            let (row, col, collected, opened) = state;
            if row == end.row && col == end.col {
                return Ok(reconstruct_gated_path(&prev, start_state, state));
            }

            for (nr, nc) in neighbours(row, col, grid) {
                let (ncollected, nopened) = match grid[nr][nc] {
                    'W' => continue,
                    'K' => {
                        let bit = key_bit[&(nr, nc)];
                        (collected | (1 << bit), opened)
                    }
                    'D' => {
                        let bit = door_bit[&(nr, nc)];
                        if opened & (1 << bit) != 0 {
                            // Already open — free to re-cross.
                            (collected, opened)
                        } else {
                            let hand = collected.count_ones().saturating_sub(opened.count_ones());
                            if hand == 0 {
                                // Closed door, no key in hand — impassable.
                                continue;
                            }
                            // Spend a key to open it, then step through.
                            (collected, opened | (1 << bit))
                        }
                    }
                    // ' ', 'S', 'F' — plain passable terrain.
                    _ => (collected, opened),
                };

                let nstate: GatedState = (nr, nc, ncollected, nopened);
                if visited.insert(nstate) {
                    prev.insert(nstate, state);
                    queue.push_back(nstate);
                }
            }
        }

        Err(Error::Solve("no solution found".to_string()))
    }
}

/// In-bounds 4-neighbours of `(row, col)` in the order Up, Left, Down, Right —
/// matching the offset order the Lee solve uses.
fn neighbours(row: usize, col: usize, grid: &[Vec<char>]) -> Vec<(usize, usize)> {
    let mut out = Vec::with_capacity(4);
    if row > 0 {
        out.push((row - 1, col));
    }
    if col > 0 {
        out.push((row, col - 1));
    }
    if row + 1 < grid.len() {
        out.push((row + 1, col));
    }
    if col + 1 < grid[row].len() {
        out.push((row, col + 1));
    }
    out
}

/// Walks the `prev` chain from the finish state back to `start_state`, emitting
/// the cell at each step, then reverses to give the S→F walk. The same cell can
/// appear twice when the route backtracks to collect a key.
fn reconstruct_gated_path(
    prev: &HashMap<GatedState, GatedState>,
    start_state: GatedState,
    end_state: GatedState,
) -> MazeSolution {
    let mut points: Vec<MazePoint> = Vec::new();
    let mut cur = end_state;
    loop {
        points.push(MazePoint { row: cur.0, col: cur.1 });
        if cur == start_state {
            break;
        }
        cur = prev[&cur];
    }
    points.reverse();
    MazeSolution::new(MazePath::new(points))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(row: usize, col: usize) -> MazePoint {
        MazePoint { row, col }
    }

    fn solve_grid(grid: Vec<Vec<char>>) -> Result<MazeSolution, Error> {
        let maze = Maze::from_vec(grid);
        Solver { maze: &maze }.solve()
    }

    #[test]
    fn solvable_collects_the_key_then_opens_the_door() {
        // S · K · D · F in a straight corridor: pick up the key, spend it on the
        // door, reach the finish.
        let grid = vec![vec!['S', 'K', 'D', 'F']];
        let solution = solve_grid(grid).expect("should be solvable with the key");
        assert_eq!(
            solution.path.points,
            vec![pt(0, 0), pt(0, 1), pt(0, 2), pt(0, 3)]
        );
    }

    #[test]
    fn door_blocks_only_route_with_no_key_is_unsolvable() {
        // S · D · F with no key anywhere: the door can never be opened.
        let result = solve_grid(vec![vec!['S', 'D', 'F']]);
        match result {
            Ok(_) => panic!("expected unsolvable, got a solution"),
            Err(error) => assert_eq!(format!("{error}"), "no solution found"),
        }
    }

    #[test]
    fn one_key_but_two_doors_is_unsolvable() {
        // S · K · D · D · F: one key opens the first door, but the second is
        // then unopenable.
        let result = solve_grid(vec![vec!['S', 'K', 'D', 'D', 'F']]);
        assert!(result.is_err(), "one key cannot open two doors");
    }

    #[test]
    fn key_locked_behind_its_own_door_is_unsolvable() {
        // S · D · K · F: the only key sits behind the very door it would open.
        let result = solve_grid(vec![vec!['S', 'D', 'K', 'F']]);
        assert!(result.is_err(), "the key is unreachable behind the door");
    }

    #[test]
    fn chained_keys_and_doors_are_solvable_in_order() {
        // S · K · D · K · D · F: each door is opened by the key just before it,
        // so the maze completes by interleaving collect/open.
        let grid = vec![vec!['S', 'K', 'D', 'K', 'D', 'F']];
        let solution = solve_grid(grid).expect("chained keys/doors should solve");
        assert_eq!(
            solution.path.points,
            vec![pt(0, 0), pt(0, 1), pt(0, 2), pt(0, 3), pt(0, 4), pt(0, 5)]
        );
    }

    #[test]
    fn backtracks_through_a_junction_to_fetch_a_key() {
        // T-junction: the key is up a dead-end branch off (0,1), and the only
        // route to the finish runs down through the door from the same
        // junction. The solution must detour to the key and walk back through
        // (0,1) — so (0,1) appears twice.
        #[rustfmt::skip]
        let grid = vec![
            vec!['S', ' ', 'K'],
            vec!['W', 'D', 'W'],
            vec!['W', 'F', 'W'],
        ];
        let solution = solve_grid(grid).expect("should solve via a backtrack");
        assert_eq!(
            solution.path.points,
            vec![pt(0, 0), pt(0, 1), pt(0, 2), pt(0, 1), pt(1, 1), pt(2, 1)]
        );
    }

    #[test]
    fn prefers_shortest_route_even_if_it_opens_a_door() {
        // Two routes from S(0,0) to F(3,0): the short left column collects the
        // key at (1,0) and opens the door at (2,0) — 3 steps, 1 door; the long
        // way round the right is 9 steps and 0 doors. The solver wants the
        // shortest *walk* irrespective of doors, so it takes the short, 1-door
        // route.
        #[rustfmt::skip]
        let grid = vec![
            vec!['S', ' ', ' ', ' '],
            vec!['K', 'W', 'W', ' '],
            vec!['D', 'W', 'W', ' '],
            vec!['F', ' ', ' ', ' '],
        ];
        let solution = solve_grid(grid).expect("solvable via the short route");
        assert_eq!(
            solution.path.points,
            vec![pt(0, 0), pt(1, 0), pt(2, 0), pt(3, 0)]
        );
    }

    #[test]
    fn shortest_walk_uses_the_nearer_door_not_a_longer_detour() {
        // A user-reported maze. The finish pocket can be entered via the door at
        // (4,5) — a shorter walk that opens one more door — or via the door at
        // (6,4) — a longer way round that opens one fewer door. The solver
        // returns the shortest *walk* irrespective of door count, so it must
        // enter via (4,5) and never take the (6,4) detour.
        #[rustfmt::skip]
        let grid = vec![
            vec!['S', ' ', ' ', ' ', ' ', ' ', 'W', 'K'],
            vec![' ', ' ', ' ', 'W', 'W', ' ', ' ', ' '],
            vec!['D', 'W', 'D', 'D', 'K', 'W', 'W', 'W'],
            vec![' ', 'K', ' ', ' ', 'D', 'W', ' ', ' '],
            vec![' ', 'W', 'D', 'W', 'K', 'D', ' ', 'W'],
            vec![' ', ' ', ' ', ' ', 'W', 'K', ' ', 'D'],
            vec!['D', 'W', 'W', ' ', 'D', ' ', 'W', 'F'],
        ];
        let solution = solve_grid(grid).expect("solvable");
        let pts = &solution.path.points;
        assert_eq!(pts.first(), Some(&pt(0, 0)), "starts at S");
        assert_eq!(pts.last(), Some(&pt(6, 7)), "ends at F");
        assert_eq!(pts.len(), 30, "shortest walk is 30 cells");
        assert!(
            pts.contains(&pt(4, 5)),
            "enters the finish pocket via the nearer door (4,5)"
        );
        assert!(
            !pts.contains(&pt(6, 4)),
            "does not take the longer (6,4) detour"
        );
    }

    #[test]
    fn keys_without_doors_take_the_plain_shortest_path() {
        // No door in the grid → the key-aware search is bypassed and the
        // original lock-blind Lee solve runs (a key with no door is irrelevant
        // to reachability).
        let grid = vec![vec!['S', 'K', 'F']];
        let solution = solve_grid(grid).expect("trivially solvable");
        assert_eq!(solution.path.points, vec![pt(0, 0), pt(0, 1), pt(0, 2)]);
    }
}
