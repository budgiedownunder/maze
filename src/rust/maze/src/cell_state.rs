use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Represents the state of a maze point cell
/// # Variants
/// - `Empty`: An empty cell that is not currently part of a solution path
/// - `Wall`: A wall
/// - `SolutionStep` - a cell that has been visited as the `value`'th step in an attempt to find the maze solution
pub enum CellState {
    Empty, 
    Wall, 
    SolutionStep { value: usize }, 
}

impl fmt::Display for CellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellState::Empty => write!(f, "Empty"),
            CellState::Wall => write!(f, "Wall"),
            CellState::SolutionStep { value } => write!(f, "Solution Step (value = {})", value),
        }
    }
}

impl CellState {
    /// Returns the step value (if any) associated with the cell state instance
    /// # Returns
    ///
    /// The step value (if variant is of type `SolutionStep`) else `None`
    pub fn step_value(&self) -> Option<usize> {
        match self {
            CellState::SolutionStep { value } => Some(*value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_clone() {
        let source = CellState::Empty{};
        let clone = source.clone();
        assert_eq!(source, clone);
        let source = CellState::Wall{};
        let clone = source.clone();
        assert_eq!(source, clone);
        let source = CellState::SolutionStep{ value: 1};
        let clone = source.clone();
        assert_eq!(source, clone);
   }

   #[test]
   fn can_format() {
        let source = CellState::Empty{};
        assert_eq!(format!("{}",source), "Empty");
        let source = CellState::Wall{};
        assert_eq!(format!("{}",source), "Wall");
        let source = CellState::SolutionStep{ value: 1};
        assert_eq!(format!("{}",source), "Solution Step (value = 1)");
   }
}