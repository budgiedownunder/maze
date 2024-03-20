use crate::Point;

#[allow(dead_code)]
pub struct Path {
    pub points: Vec<Point>,
}

impl Path {
    pub fn new(points: Vec<Point>) -> Path {
        Path { points }
    }
}
