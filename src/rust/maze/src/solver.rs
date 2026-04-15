use std::collections::VecDeque;

use data_model::{Maze, MazeCellState, MazePoint};
use crate::{Error, MazePath, MazePointOffset, MazeSolution};

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
        self.solve_lee(&start_pt, &finish_pt)
    }
}
