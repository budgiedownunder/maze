use std::fmt;

use crate::Point;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
/// Represents a path composed of a sequence of points
pub struct Path {
    /// Vector of successive points within the path 
    pub points: Vec<Point>,
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = write!(f, "Points: {} =>", self.points.len());
        for pt in self.points.iter() {
            result = write!(f, "\n> {}", pt);
        }
        result
    }
}

impl Path {
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
    /// use maze::Path;
    /// use maze::Point;
    /// let points = vec![
    ///   Point { row: 0, col: 1 },
    ///   Point { row: 0, col: 0 },
    ///   Point { row: 1, col: 0 },
    /// ];

    /// let p = Path::new(points);
    /// assert_eq!(p.points.len(), 3);
    /// ``` 
    pub fn new(points: Vec<Point>) -> Path {
        Path { points }
    }
}
