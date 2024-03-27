use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, PartialEq)]
pub enum CellState {
    Empty,
    Blocked,
    Step { value: usize },
}

impl Clone for CellState {
    fn clone(&self) -> Self {
        match self {
            CellState::Empty => CellState::Empty,
            CellState::Blocked => CellState::Blocked,
            CellState::Step { value } => CellState::Step { value: *value },
        }
    }
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellState::Empty => write!(f, "Empty"),
            CellState::Blocked => write!(f, "Blocked"),
            CellState::Step { value } => write!(f, "Step (value = {})", value),
        }
    }
}

impl CellState {
    pub fn step_value(&self) -> Option<usize> {
        match self {
            CellState::Step { value } => Some(*value),
            _ => None,
        }
    }
}
