use crate::solution::Solution;
use crate::solver::SolveError;
use crate::Definition;
use crate::Solver;

#[allow(dead_code)]
pub struct Maze {
    pub definition: Definition,
}

impl Maze {
    pub fn new(definition: Definition) -> Maze {
        Maze { definition }
    }
    pub fn from_vec(grid: Vec<Vec<i32>>) -> Self {
        Maze { definition: Definition::from_vec(grid) }
    }
    pub fn solve(&self) -> Result<Solution, SolveError> {
        let s = Solver { maze: &self };
        s.solve()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_maze_from_vector() {
        let grid: Vec<Vec<i32>> = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
        ];
        let m = Maze::from_vec(grid);
        assert_eq!(m.definition.rows, 2);
        assert_eq!(m.definition.cols, 3);
    }

    #[test]
    #[should_panic(expected = "Grid vector contains rows with different numbers of columns (expected 3 for all rows)")]
    fn cannot_create_new_maze_definition_from_vector_with_diff_row_counts() {
        let grid: Vec<Vec<i32>> = vec![
            vec![1, 2, 3],
            vec![4, 5, 6, 7],
        ];
        let _d = Definition::from_vec(grid);
    }

    #[test]
    fn can_create_new_maze_from_definition() {
        let m = Maze::new(Definition::new(2, 3));
        assert_eq!(m.definition.rows, 2);
        assert_eq!(m.definition.cols, 3);
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
