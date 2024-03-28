use std::collections::VecDeque;
use std::error::Error;

use crate::CellState;
use crate::Maze;
use crate::Offset;
use crate::Path;
use crate::Point;
use crate::Solution;

#[derive(Debug)]
pub struct SolveError {
    pub message: String,
}

impl SolveError {
    fn new(message: &str) -> Self {
        SolveError {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for SolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for SolveError {}

#[allow(dead_code)]
pub struct Solver<'a> {
    pub maze: &'a Maze,
}

impl Solver<'_> {
    fn is_valid(&self, pt: &Point) -> bool {
        self.maze.definition.is_valid(pt)
    }

    fn calc_location(&self, pt: &Point, offset: &Offset) -> Result<Point, SolveError> {
        if offset.row < 0 && (offset.row.abs() as usize) > pt.row {
            return Err(SolveError::new("location is out of bounds"));
        }
        if offset.col < 0 && (offset.col.abs() as usize) > pt.col {
            return Err(SolveError::new("location is out of bounds"));
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
            return Err(SolveError::new("location is out of bounds"));
        }
        Ok(pt_check)
    }

    fn get_lee_solution(
        &self,
        grid_state: &Vec<Vec<CellState>>,
        start: &Point,
        end: &Point,
        offsets: &[Offset],
    ) -> Result<Solution, SolveError> {
        let mut points: Vec<Point> = vec![];
        match grid_state[end.row][end.col].step_value() {
            None => {
                return Err(SolveError::new(
                    "solution path not found (end point not processed)",
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
                        return Err(SolveError::new(format!("solution path not found (no path sequence neighbour exists for point {})", step_pt).as_str()));
                    }
                }
                _ => (),
            }
        }
    }

    // Assumes 'start' and 'end' are valid
    fn solve_lee(&self, start: &Point, end: &Point) -> Result<Solution, SolveError> {
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
                                    Ok(offset_pt) => match grid_state[offset_pt.row][offset_pt.col] {
                                        CellState::Empty => {
                                            grid_state[offset_pt.row][offset_pt.col] =
                                                CellState::SolutionStep { value: value + 1 };
                                            if offset_pt == *end {
                                                return self
                                                    .get_lee_solution(&grid_state, start, end, &offsets);
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

        Err(SolveError::new("no solution found"))
    }

    pub fn solve(&self, start: Point, end: Point) -> Result<Solution, SolveError> {
        if !self.is_valid(&start) {
            return Err(SolveError::new(
                format!("start location {} is invalid", start).as_str(),
            ));
        }
        if !self.is_valid(&end) {
            return Err(SolveError::new(
                format!("end location {} is invalid", end,).as_str(),
            ));
        }
        if start == end {
            let points = vec![start.clone()];

            return Ok(Solution::new(Path::new(points)));
        }
        self.solve_lee(&start, &end)
    }
}
