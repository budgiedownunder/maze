use crate::MazePath;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a maze solution
#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct MazeSolution {
    /// Solution path
    pub path: MazePath,
}

impl MazeSolution {
    /// Creates a maze solution instance with the given solution path
    /// # Arguments
    /// * `path` - Solution path
    /// 
    /// # Returns
    /// 
    /// A new solution instance
    /// 
    /// # Examples
    ///
    /// ```
    /// use data_model::MazePoint;
    /// use maze::{MazePath, MazeSolution};
    /// let path = MazePath {
    ///   points: vec![
    ///     MazePoint { row: 0, col: 1 },
    ///     MazePoint { row: 0, col: 0 },
    ///     MazePoint { row: 1, col: 0 },
    ///   ],
    /// };
    /// let s = MazeSolution::new(path);
    /// assert_eq!(s.path.points.len(), 3);
    /// ```
    pub fn new(path: MazePath) -> MazeSolution {
        MazeSolution { path }
    }
}
