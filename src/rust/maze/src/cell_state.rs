use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum CellState {
    Empty,
    Blocked,
    SolutionStep { value: usize },
}

impl Clone for CellState {
    fn clone(&self) -> Self {
        match self {
            CellState::Empty => CellState::Empty,
            CellState::Blocked => CellState::Blocked,
            CellState::SolutionStep { value } => CellState::SolutionStep { value: *value },
        }
    }
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellState::Empty => write!(f, "Empty"),
            CellState::Blocked => write!(f, "Blocked"),
            CellState::SolutionStep { value } => write!(f, "Path Step (value = {})", value),
        }
    }
}

impl CellState {
    pub fn step_value(&self) -> Option<usize> {
        match self {
            CellState::SolutionStep { value } => Some(*value),
            _ => None,
        }
    }
}
