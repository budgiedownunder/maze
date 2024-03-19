// Re-export modules
mod definition;
mod maze;

// Re-export structs
pub use definition::Definition;
pub use maze::Maze;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_maze_definition() {
        let d = Definition::new(2, 3);
        assert_eq!(d.width, 2);
        assert_eq!(d.height, 3);
    }

    #[test]
    fn can_create_new_maze_from_stack_definition() {
        let m = Maze::new(Definition {
            width: 2,
            height: 3,
        });
        assert_eq!(m.definition.width, 2);
        assert_eq!(m.definition.height, 3);
    }

    #[test]
    fn can_create_new_maze_from_heap_definition() {
        let m = Maze::new(Definition::new(2, 3));
        assert_eq!(m.definition.width, 2);
        assert_eq!(m.definition.height, 3);
    }
}
