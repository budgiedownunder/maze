use std::collections::VecDeque;

use crate::CellState;
use crate::Maze;
use crate::MazeError;
use crate::Offset;
use crate::Path;
use crate::Point;
use crate::Solution;

#[allow(dead_code)]
/// Represents a maze solver
pub struct Solver<'a> {
    /// Maze reference
    pub maze: &'a Maze,
}

impl Solver<'_> {
    fn is_valid(&self, pt: &Point) -> bool {
        self.maze.definition.is_valid(pt)
    }

    fn calc_location(&self, pt: &Point, offset: &Offset) -> Result<Point, MazeError> {
        if offset.row < 0 && (offset.row.abs() as usize) > pt.row {
            return Err(MazeError::new("location is out of bounds".to_string()));
        }
        if offset.col < 0 && (offset.col.abs() as usize) > pt.col {
            return Err(MazeError::new("location is out of bounds".to_string()));
        }
        let pt_check = Point {
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
        };

        if !self.is_valid(&pt_check) {
            return Err(MazeError::new("location is out of bounds".to_string()));
        }
        Ok(pt_check)
    }

    fn get_lee_solution(
        &self,
        grid_state: &Vec<Vec<CellState>>,
        start: &Point,
        end: &Point,
        offsets: &[Offset],
    ) -> Result<Solution, MazeError> {
        let mut points: Vec<Point> = vec![];
        match grid_state[end.row][end.col].step_value() {
            None => {
                return Err(MazeError::new(
                    "solution path not found (end point not processed)".to_string(),
                ));
            }
            _ => {}
        }
        let mut step_pt: Point = end.clone();
        points.push(end.clone());
        loop {
            match grid_state[step_pt.row][step_pt.col] {
                CellState::SolutionStep { value } => {
                    let mut found_neighbour = false;
                    for offset in offsets.iter() {
                        match self.calc_location(&step_pt, &offset) {
                            Ok(offset_pt) => {
                                let offset_pt_step_value =
                                    grid_state[offset_pt.row][offset_pt.col].step_value();
                                match offset_pt_step_value {
                                    Some(offset_pt_value) => {
                                        if step_pt == *start {
                                            points.reverse();
                                            return Ok(Solution::new(Path::new(points)));
                                        }
                                        if offset_pt_value == value - 1 {
                                            step_pt = offset_pt;
                                            points.push(step_pt.clone());
                                            found_neighbour = true;
                                            break;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                            Err(_) => {} // Skip
                        }
                    }
                    if !found_neighbour {
                        return Err(MazeError::new(format!("solution path not found (no path sequence neighbour exists for point {})", step_pt)));
                    }
                }
                _ => (),
            }
        }
    }

    // Assumes 'start' and 'end' are valid
    fn solve_lee(&self, start: &Point, end: &Point) -> Result<Solution, MazeError> {
        let mut q: VecDeque<Point> = VecDeque::new();
        let mut grid_state = self.maze.definition.to_state();
        let offsets = [
            Offset { row: -1, col: 0 }, // Up
            Offset { row: 0, col: -1 }, // Left
            Offset { row: 1, col: 0 },  // Down
            Offset { row: 0, col: 1 },  // Right
        ];

        q.push_back(start.clone());
        grid_state[start.row][start.col] = CellState::SolutionStep { value: 0 };
        while q.len() > 0 {
            let opt_pt = q.pop_front();
            match opt_pt {
                Some(pt) => {
                    let pt_step_value = grid_state[pt.row][pt.col].step_value();
                    match pt_step_value {
                        Some(value) => {
                            for offset in offsets.iter() {
                                match self.calc_location(&pt, &offset) {
                                    Ok(offset_pt) => match grid_state[offset_pt.row][offset_pt.col]
                                    {
                                        CellState::Empty => {
                                            grid_state[offset_pt.row][offset_pt.col] =
                                                CellState::SolutionStep { value: value + 1 };
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
                                        _ => (),
                                    },
                                    Err(_) => {} // Skip
                                }
                            }
                        }
                        None => (),
                    }
                }
                None => {}
            }
        }

        Err(MazeError::new("no solution found".to_string()))
    }

    /// Attempts to solve the path between a start and end point within the maze referenced by the solver instance
    /// # Arguments
    /// * `start` - Start point
    /// * `end` - End point
    ///
    /// # Returns
    ///
    /// A `Result` containing either the solution if successful, or a `MazeError` if an error occurs
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::Maze;
    /// use maze::Point;
    /// use maze::Solver;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec![' ', 'W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W', ' '],
    ///    vec![' ', ' ', ' ', 'W', ' '],
    ///    vec!['W', ' ', 'W', ' ', ' '],
    ///    vec![' ', ' ', ' ', 'W', ' '],
    ///    vec!['W', 'W', ' ', ' ', ' '],
    ///    vec!['W', 'W', ' ', 'W', ' '],
    /// ];
    /// let solver = Solver {
    ///     maze: &Maze::from_vec(grid),
    /// };
    /// let start = Point { row: 0, col: 0 };
    /// let end = Point { row: 2, col: 4 };
    /// let result = solver.solve(start, end);
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
    pub fn solve(&self, start: Point, end: Point) -> Result<Solution, MazeError> {
        if !self.is_valid(&start) {
            return Err(MazeError::new(format!(
                "start location {} is invalid",
                start
            )));
        }
        if !self.is_valid(&end) {
            return Err(MazeError::new(format!("end location {} is invalid", end)));
        }
        if start == end {
            let points = vec![start.clone()];

            return Ok(Solution::new(Path::new(points)));
        }
        self.solve_lee(&start, &end)
    }
}
