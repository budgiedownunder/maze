use crate::Path;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Represents a maze solution
#[derive(Debug, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct Solution {
    /// Solution path
    pub path: Path,
}

impl Solution {
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
    /// use maze::Path;
    /// use maze::Point;
    /// use maze::Solution;
    /// let path = Path {
    ///   points: vec![
    ///     Point { row: 0, col: 1 },
    ///     Point { row: 0, col: 0 },
    ///     Point { row: 1, col: 0 },
    ///   ],
    /// };
    /// let s = Solution::new(path);
    /// assert_eq!(s.path.points.len(), 3);
    /// ```
    pub fn new(path: Path) -> Solution {
        Solution { path }
    }
}
