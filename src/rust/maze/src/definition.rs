use crate::Wall;

#[allow(dead_code)]
pub struct Definition {
    pub width: usize,
    pub height: usize,
    pub walls:Vec<Wall>,
}

impl Definition {
    pub fn new(width: usize, height: usize) -> Self {
        Definition { width, height, walls: Vec::new(), }
    }
}