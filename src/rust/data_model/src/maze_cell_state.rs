use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
/// Represents the state of a maze cell
/// # Variants
/// - `Empty`: An empty cell that is not currently part of a solution path
/// - `Wall`: A wall
/// - `SolutionStep` - a cell that has been visited as the `value`'th step in an attempt to find the maze solution
pub enum MazeCellState {
    Empty, 
    Wall, 
    SolutionStep { value: usize }, 
}

impl fmt::Display for MazeCellState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MazeCellState::Empty => write!(f, "Empty"),
            MazeCellState::Wall => write!(f, "Wall"),
            MazeCellState::SolutionStep { value } => write!(f, "Solution Step (value = {})", value),
        }
    }
}

impl MazeCellState {
    /// Returns the step value (if any) associated with the cell state instance
    /// # Returns
    ///
    /// The step value (if variant is of type `SolutionStep`) else `None`
    pub fn step_value(&self) -> Option<usize> {
        match self {
            MazeCellState::SolutionStep { value } => Some(*value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;    

    #[test]
    fn can_clone() {
        let source = MazeCellState::Empty{};
        let clone = source.clone();
        assert_eq!(source, clone);
        let source = MazeCellState::Wall{};
        let clone = source.clone();
        assert_eq!(source, clone);
        let source = MazeCellState::SolutionStep{ value: 1};
        let clone = source.clone();
        assert_eq!(source, clone);
   }

   #[test]
   fn can_format() {
        let source = MazeCellState::Empty{};
        assert_eq!(format!("{}",source), "Empty");
        let source = MazeCellState::Wall{};
        assert_eq!(format!("{}",source), "Wall");
        let source = MazeCellState::SolutionStep{ value: 1};
        assert_eq!(format!("{}",source), "Solution Step (value = 1)");
   }
}