use rand::seq::SliceRandom;
use rand::Rng;
use rand::rngs::StdRng;
use rand::SeedableRng;

use data_model::{Maze, MazeDefinition, MazePoint};

use crate::{Error, GenerationAlgorithm};

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
    /// Attempts that produce a shorter spine are discarded and retried.
    /// Defaults to `(row_count + col_count) / 2`.
    pub min_spine_length: Option<usize>,
    /// Maximum number of generation attempts before returning an error
    /// Defaults to `100`.
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
    /// [`Error::Generate`] if validation fails or all retry attempts are exhausted.
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

        let can_carve =
            |grid: &[Vec<char>], from_r: usize, from_c: usize, nr: usize, nc: usize| -> bool {
                for (dr, dc) in &OFFSETS {
                    let rr = nr as i64 + dr;
                    let cc = nc as i64 + dc;
                    if !in_bounds(rr, cc) {
                        continue;
                    }
                    let rr = rr as usize;
                    let cc = cc as usize;
                    if rr == from_r && cc == from_c {
                        continue;
                    }
                    if grid[rr][cc] != 'W' {
                        return false;
                    }
                }
                true
            };

        let mut last_err = String::new();

        for _ in 0..max_retries {
            let mut grid = vec![vec!['W'; cols]; rows];
            grid[start.row][start.col] = ' ';

            // Phase 1: random walk from start to finish (builds the spine)
            let mut current = start.clone();
            let mut spine: Vec<MazePoint> = vec![current.clone()];
            let mut stuck = false;

            loop {
                if current == finish {
                    break;
                }

                let mut neighbours: Vec<MazePoint> = vec![];
                for (dr, dc) in &OFFSETS {
                    let nr = current.row as i64 + dr;
                    let nc = current.col as i64 + dc;
                    if !in_bounds(nr, nc) {
                        continue;
                    }
                    let nr = nr as usize;
                    let nc = nc as usize;
                    if grid[nr][nc] != 'W' {
                        continue;
                    }
                    if !can_carve(&grid, current.row, current.col, nr, nc) {
                        continue;
                    }
                    neighbours.push(MazePoint { row: nr, col: nc });
                }

                if neighbours.is_empty() {
                    stuck = true;
                    break;
                }

                let next = neighbours.choose(&mut rng).unwrap().clone();
                grid[next.row][next.col] = ' ';
                spine.push(next.clone());
                current = next;
            }

            if stuck {
                last_err = "walk got stuck before reaching finish".to_string();
                continue;
            }

            if spine.len() < min_spine {
                last_err = format!(
                    "spine length {} is less than min_spine_length {}",
                    spine.len(),
                    min_spine
                );
                continue;
            }

            // Phase 2: DFS branches from (shuffled) spine cells
            let mut branch_cells = spine.clone();
            if !branch_from_finish {
                branch_cells.retain(|p| !(p.row == finish.row && p.col == finish.col));
            }
            branch_cells.shuffle(&mut rng);

            for cell in branch_cells {
                Self::dfs_branch(&mut grid, rows, cols, cell.row, cell.col, &OFFSETS, &mut rng);
            }

            // Place S and F markers
            grid[start.row][start.col] = 'S';
            grid[finish.row][finish.col] = 'F';

            return Ok(Maze::new(MazeDefinition::from_vec(grid)));
        }

        Err(Error::Generate(if last_err.is_empty() {
            "max_retries is 0, no attempts made".to_string()
        } else {
            last_err
        }))
    }

    fn dfs_branch(
        grid: &mut Vec<Vec<char>>,
        rows: usize,
        cols: usize,
        r: usize,
        c: usize,
        offsets: &[(i64, i64); 4],
        rng: &mut impl Rng,
    ) {
        // Collect valid neighbours to carve into
        let mut neighbours: Vec<(usize, usize)> = vec![];
        for (dr, dc) in offsets.iter() {
            let nr = r as i64 + dr;
            let nc = c as i64 + dc;
            if nr < 0 || nr as usize >= rows || nc < 0 || nc as usize >= cols {
                continue;
            }
            let nr = nr as usize;
            let nc = nc as usize;
            if grid[nr][nc] != 'W' {
                continue;
            }
            if Self::check_can_carve(grid, rows, cols, r, c, nr, nc, offsets) {
                neighbours.push((nr, nc));
            }
        }

        neighbours.shuffle(rng);

        for (nr, nc) in neighbours {
            // Re-check since the grid may have changed from earlier recursive calls
            if grid[nr][nc] != 'W' {
                continue;
            }
            if !Self::check_can_carve(grid, rows, cols, r, c, nr, nc, offsets) {
                continue;
            }
            grid[nr][nc] = ' ';
            Self::dfs_branch(grid, rows, cols, nr, nc, offsets, rng);
        }
    }

    fn check_can_carve(
        grid: &[Vec<char>],
        rows: usize,
        cols: usize,
        from_r: usize,
        from_c: usize,
        nr: usize,
        nc: usize,
        offsets: &[(i64, i64); 4],
    ) -> bool {
        for (dr, dc) in offsets.iter() {
            let rr = nr as i64 + dr;
            let cc = nc as i64 + dc;
            if rr < 0 || rr as usize >= rows || cc < 0 || cc as usize >= cols {
                continue;
            }
            let rr = rr as usize;
            let cc = cc as usize;
            if rr == from_r && cc == from_c {
                continue;
            }
            if grid[rr][cc] != 'W' {
                return false;
            }
        }
        true
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
            assert_eq!(
                row.len(),
                cols,
                "col count mismatch for {}x{}",
                rows,
                cols
            );
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
            .unwrap_or_else(|_| panic!("maze {}x{} should be solvable", rows, cols));

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
            },
        };
        let maze1 = make(1).generate().expect("should succeed");
        let maze2 = make(2).generate().expect("should succeed");
        assert_ne!(maze1.definition.grid, maze2.definition.grid);
    }
}
