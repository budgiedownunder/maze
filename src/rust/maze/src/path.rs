use std::fmt;

use crate::Point;

#[allow(dead_code)]
#[derive(Debug, PartialEq)]
pub struct Path {
    pub points: Vec<Point>,
}

// Implement the Display trait for the Path struct
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
    pub fn new(points: Vec<Point>) -> Path {
        Path { points }
    }
}
