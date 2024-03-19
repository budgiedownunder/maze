#[allow(dead_code)]
pub struct Definition {
    pub width: usize,
    pub height: usize,
}

impl Definition {
    pub fn new(width: usize, height: usize) -> Self {
        Definition { width, height }
    }
}