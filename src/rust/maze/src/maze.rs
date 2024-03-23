use crate::solution::Solution;
use crate::solver::SolveError;
use crate::Definition;
use crate::Point;
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
        Maze {
            definition: Definition::from_vec(grid),
        }
    }
    pub fn solve(&self, start: Point, end: Point) -> Result<Solution, SolveError> {
        let s = Solver { maze: &self };
        s.solve(start, end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Path;

    #[test]
    fn can_create_new_maze_from_vector() {
        let grid: Vec<Vec<i32>> = vec![vec![-1, -1, -1], vec![-1, -1, -1]];
        let m = Maze::from_vec(grid);
        assert_eq!(m.definition.rows, 2);
        assert_eq!(m.definition.cols, 3);
    }

    #[test]
    #[should_panic(
        expected = "grid vector contains rows with different numbers of columns (expected 3 for all rows)"
    )]
    fn cannot_create_new_from_vector_with_diff_row_counts() {
        let grid: Vec<Vec<i32>> = vec![vec![-1, -1, -1], vec![-1, -1, -1, -1]];
        let _d = Definition::from_vec(grid);
    }

    #[test]
    fn can_create_new_from_definition() {
        let m = Maze::new(Definition::new(2, 3));
        assert_eq!(m.definition.rows, 2);
        assert_eq!(m.definition.cols, 3);
    }

    #[test]
    fn solve_should_fail_with_invalid_start_location() {
        let m = Maze::new(Definition::new(2, 3));
        let start = Point { row: 2, col: 0 };
        let end = Point { row: 0, col: 2 };
        let result = m.solve(start.clone(), end);
        match result {
            Ok(_) => {
                panic!("expected solve() to return Err, but it returned Ok");
            }
            Err(e) => {
                assert_eq!(e.message, format!("start location {} is invalid", start));
            }
        }
    }

    #[test]
    fn solve_should_fail_with_invalid_end_location() {
        let m = Maze::new(Definition::new(2, 3));
        let start = Point { row: 1, col: 0 };
        let end = Point { row: 0, col: 3 };
        let result = m.solve(start, end.clone());
        match result {
            Ok(_) => {
                panic!("expected solve() to return Err, but it returned Ok");
            }
            Err(e) => {
                assert_eq!(e.message, format!("end location {} is invalid", end));
            }
        }
    }

    #[test]
    fn solve_should_succeed_for_same_start_end() {
        let m = Maze::new(Definition::new(2, 3));
        let start = Point { row: 1, col: 2 };
        let end = Point { row: 1, col: 2 };
        let result = m.solve(start, end);
        match result {
            Ok(s) => {
                let expected_solution_path = Path {
                    points: vec![Point { row: 1, col: 2 }],
                };
                assert_eq!(s.path.points.len(), 1);
                assert_eq!(s.path, expected_solution_path);
            }
            Err(error) => {
                panic!(
                    "{}",
                    format!(
                        "expected solve() to succeed, but it returned the error: {}",
                        error
                    )
                );
            }
        }
    }

    #[test]
    fn solve_should_fail_as_no_solution() {
        let grid: Vec<Vec<i32>> = vec![
            vec![-1, -1, -2, -1],
            vec![-1, -1, -2, -1],
            vec![-1, -1, -2, -1],
        ];
        let m = Maze::new(Definition::from_vec(grid));
        let start = Point { row: 1, col: 0 };
        let end = Point { row: 0, col: 2 };
        let result = m.solve(start, end);
        match result {
            Ok(_) => {
                panic!("expected solve() to return Err, but it returned Ok");
            }
            Err(e) => {
                assert_eq!(e.message, "no solution found");
            }
        }
    }

    #[test]
    fn solve_should_succeed_for_maze_with_no_walls_1() {
        let m = Maze::new(Definition::new(2, 3));
        let start = Point { row: 1, col: 0 };
        let end = Point { row: 0, col: 2 };
        let result = m.solve(start, end);
        match result {
            Ok(s) => {
                let expected_solution_path = Path {
                    points: vec![
                        Point { row: 1, col: 0 },
                        Point { row: 0, col: 0 },
                        Point { row: 0, col: 1 },
                        Point { row: 0, col: 2 },
                    ],
                };
                assert_eq!(s.path.points.len(), 4);
                assert_eq!(s.path, expected_solution_path);
            }
            Err(error) => {
                panic!(
                    "{}",
                    format!(
                        "expected solve() to succeed, but it returned the error: {}",
                        error
                    )
                );
            }
        }
    }

    #[test]
    fn solve_should_succeed_for_maze_with_no_walls_2() {
        let m = Maze::new(Definition::new(2, 4));
        let start = Point { row: 1, col: 0 };
        let end = Point { row: 0, col: 3 };
        let result = m.solve(start, end);
        match result {
            Ok(s) => {
                let expected_solution_path = Path {
                    points: vec![
                        Point { row: 1, col: 0 },
                        Point { row: 0, col: 0 },
                        Point { row: 0, col: 1 },
                        Point { row: 0, col: 2 },
                        Point { row: 0, col: 3 },
                    ],
                };
                assert_eq!(s.path.points.len(), 5);
                assert_eq!(s.path, expected_solution_path);
            }
            Err(error) => {
                panic!(
                    "{}",
                    format!(
                        "expected solve() to succeed, but it returned the error: {}",
                        error
                    )
                );
            }
        }
    }

    #[test]
    fn solve_should_succeed_for_maze_with_no_walls_3() {
        let m = Maze::new(Definition::new(3, 4));
        let start = Point { row: 1, col: 1 };
        let end = Point { row: 2, col: 2 };
        let result = m.solve(start, end);
        match result {
            Ok(s) => {
                let expected_solution_path = Path {
                    points: vec![
                        Point { row: 1, col: 1 },
                        Point { row: 1, col: 2 },
                        Point { row: 2, col: 2 },
                    ],
                };
                assert_eq!(s.path.points.len(), 3);
                assert_eq!(s.path, expected_solution_path);
            }
            Err(error) => {
                panic!(
                    "{}",
                    format!(
                        "expected solve() to succeed, but it returned the error: {}",
                        error
                    )
                );
            }
        }
    }
}
