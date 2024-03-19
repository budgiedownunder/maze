// Re-export modules
mod definition;
mod maze;
mod path;
mod solution;
mod solver;
mod wall;

// Re-export structs
pub use definition::Definition;
pub use maze::Maze;
pub use path::Path;
pub use solution::Solution;
pub use solver::Solver;
pub use wall::Wall;

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
            walls: Vec::new(),
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

    #[test]
    fn maze_solve_should_fail_with_not_implemented() {
        let m = Maze::new(Definition::new(2, 3));
        let result = m.solve();
        match result {
            Ok(_) => {
                // If solve() returns Ok, fail the test
                panic!("Expected solve() to return Err, but it returned Ok");
            }
            Err(e) => {
                // Assert if "Not implemented" is not returned 
                assert_eq!(e.message, "Not implemented");
            }
        }        
    }
}
