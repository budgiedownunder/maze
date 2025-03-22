use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

use data_model::MazePoint;

#[allow(dead_code)]
#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema)]
/// Represents a maze path composed of a sequence of maze points
pub struct MazePath {
    /// Vector of successive points within the path 
    pub points: Vec<MazePoint>,
}

impl fmt::Display for MazePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = write!(f, "Points: {} =>", self.points.len());
        for pt in self.points.iter() {
            result = write!(f, "\n> {}", pt);
        }
        result
    }
}

impl MazePath {
    /// Creates a new path instance for the given set of points
    /// # Arguments
    /// * `points` - Sequence of points defining the path
    /// 
    /// # Returns
    /// 
    /// A new path
    /// 
    /// # Examples
    ///
    /// ```
    /// use data_model::MazePoint;
    /// use maze::MazePath;
    /// let points = vec![
    ///   MazePoint { row: 0, col: 1 },
    ///   MazePoint { row: 0, col: 0 },
    ///   MazePoint { row: 1, col: 0 },
    /// ];

    /// let p = MazePath::new(points);
    /// assert_eq!(p.points.len(), 3);
    /// ``` 
    pub fn new(points: Vec<MazePoint>) -> MazePath {
        MazePath { points }
    }
}
