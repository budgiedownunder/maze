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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_maze_definition() {
        let d = Definition::new(2, 3);
        assert_eq!(d.width, 2);
        assert_eq!(d.height, 3);
    }
}