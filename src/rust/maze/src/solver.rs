use crate::solution::Solution;
use crate::Maze;

use std::error::Error;

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
    fn is_valid_location(&self, row_idx: usize, col_idx: usize) -> bool {
        if row_idx >= self.maze.definition.rows || col_idx >= self.maze.definition.cols {
            return false;
        }
        true
    }

    pub fn solve(
        &self,
        start_row: usize,
        start_col: usize,
        end_row: usize,
        end_col: usize,
    ) -> Result<Solution, SolveError> {
        if !self.is_valid_location(start_row, start_col) {
            return Err(SolveError::new(
                format!("Start location [{}, {}] is invalid", start_row, start_col).as_str(),
            ));
        }
        if !self.is_valid_location(end_row, end_col) {
            return Err(SolveError::new(
                format!("End location [{}, {}] is invalid", end_row, end_col,).as_str(),
            ));
        }

        Err(SolveError::new("Not implemented"))
        //Ok(())
    }
}
