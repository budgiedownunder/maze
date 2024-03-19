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
pub struct Solver <'a>{
    pub maze: &'a Maze,
}

impl Solver <'_> {
    pub fn solve(&self) -> Result<(), SolveError> {
        Err(SolveError::new("Not implemented"))
        //Ok(())
    }
}

