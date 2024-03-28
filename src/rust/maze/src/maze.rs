use crate::solution::Solution;
use crate::solver::SolveError;
use crate::Definition;
use crate::Direction;
use crate::Path;
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
    pub fn from_vec(grid: Vec<Vec<char>>) -> Self {
        Maze {
            definition: Definition::from_vec(grid),
        }
    }
    pub fn solve(&self, start: Point, end: Point) -> Result<Solution, SolveError> {
        let s = Solver { maze: &self };
        s.solve(start, end)
    }

    pub fn print(&self, start: Point, end: Point, path: Path) {
        let mut base = self.definition.display_grid();
        let mut path_idx = 0;

        for pt in &path.points {
            if self.definition.is_valid(pt) && *pt != start && *pt != end {
                let mut direction = Direction::None;
                if (path_idx + 1) < path.points.len() {
                    let next_pt = &path.points[path_idx + 1];
                    if next_pt.row == pt.row {
                        direction = if pt.col < next_pt.col {
                            Direction::Right
                        } else if pt.col > next_pt.col {
                            Direction::Left
                        } else {
                            Direction::None
                        };
                    } else if next_pt.col == pt.col {
                        direction = if pt.row < next_pt.row {
                            Direction::Down
                        } else if pt.row > next_pt.row {
                            Direction::Up
                        } else {
                            Direction::None
                        };
                    }
                }

                base[pt.row][pt.col] = direction.unicode_char();
            }
            path_idx += 1;
        }
        if self.definition.is_valid(&start) {
            base[start.row][start.col] = 'S';
        }
        if self.definition.is_valid(&end) {
            base[end.row][end.col] = 'F';
        }
        for row in base.iter() {
            println!();
            for col in row {
                print!("{}", col);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_maze_from_vector() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', ' ']
        ];
        let m = Maze::from_vec(grid);
        assert_eq!(m.definition.row_count(), 2);
        assert_eq!(m.definition.col_count(), 3);
    }

    #[test]
    #[should_panic(
        expected = "grid vector contains rows with different numbers of columns (expected 3 for all rows)"
    )]
    fn cannot_create_new_from_vector_with_diff_row_counts() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', ' ', ' ']
        ];
        let _d = Definition::from_vec(grid);
    }

    #[test]
    fn can_create_new_from_definition() {
        let m = Maze::new(Definition::new(2, 3));
        assert_eq!(m.definition.row_count(), 2);
        assert_eq!(m.definition.col_count(), 3);
    }

    #[test]
    fn can_print_maze_with_empty_solution_path() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'W', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec![' ', 'W', 'W', 'W', ' '],
            vec![' ', ' ', ' ', ' ', ' '],
        ];
        let m = Maze::new(Definition::from_vec(grid));
        let start = Point { row: 0, col: 1 };
        let end = Point { row: 2, col: 4 };
        let path = Path { points: vec![] };
        m.print(start, end, path);
    }

    #[test]
    fn can_print_maze_with_solution_path() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'W', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec![' ', 'W', 'W', 'W', ' '],
            vec![' ', ' ', ' ', ' ', ' '],
        ];
        let m = Maze::new(Definition::from_vec(grid));
        let start = Point { row: 0, col: 1 };
        let end = Point { row: 2, col: 4 };
        let path = Path {
            points: vec![
                Point { row: 0, col: 1 },
                Point { row: 0, col: 0 },
                Point { row: 1, col: 0 },
                Point { row: 2, col: 0 },
                Point { row: 2, col: 1 },
                Point { row: 2, col: 2 },
                Point { row: 3, col: 2 },
                Point { row: 4, col: 2 },
                Point { row: 4, col: 1 },
                Point { row: 4, col: 0 },
                Point { row: 5, col: 0 },
                Point { row: 6, col: 0 },
                Point { row: 6, col: 1 },
                Point { row: 6, col: 2 },
                Point { row: 6, col: 3 },
                Point { row: 6, col: 4 },
                Point { row: 5, col: 4 },
                Point { row: 4, col: 4 },
                Point { row: 3, col: 4 },
                Point { row: 2, col: 4 },
            ],
        };
        m.print(start, end, path);
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

    #[test]
    fn solve_should_fail_as_no_solution_due_to_blocking_wall() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'W', ' '],
            vec![' ', ' ', 'W', ' '],
            vec![' ', ' ', 'W', ' '],
        ];
        let m = Maze::new(Definition::from_vec(grid));
        let start = Point { row: 1, col: 0 };
        let end = Point { row: 0, col: 2 };
        let result = m.solve(start, end);
        match result {
            Ok(_) => {
                panic!("expected solve() to return Err, but it returned Ok");
            }
            Err(error) => {
                assert_eq!(error.message, "no solution found");
            }
        }
    }

    #[test]
    fn solve_should_succeed_with_walls_1() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'W', ' ', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec!['W', ' ', 'W', ' ', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec!['W', 'W', ' ', ' ', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
        ];
        let m = Maze::new(Definition::from_vec(grid));
        let start = Point { row: 0, col: 0 };
        let end = Point { row: 2, col: 4 };
        let result = m.solve(start, end);
        match result {
            Ok(s) => {
                let expected_solution_path = Path {
                    points: vec![
                        Point { row: 0, col: 0 },
                        Point { row: 1, col: 0 },
                        Point { row: 2, col: 0 },
                        Point { row: 2, col: 1 },
                        Point { row: 3, col: 1 },
                        Point { row: 4, col: 1 },
                        Point { row: 4, col: 2 },
                        Point { row: 5, col: 2 },
                        Point { row: 5, col: 3 },
                        Point { row: 5, col: 4 },
                        Point { row: 4, col: 4 },
                        Point { row: 3, col: 4 },
                        Point { row: 2, col: 4 },
                    ],
                };
                assert_eq!(s.path.points.len(), 13);
                assert_eq!(s.path, expected_solution_path);
            }
            Err(error) => {
                panic!(
                    "expected solve() to succeed but it returned the error {}",
                    error.message
                );
            }
        }
    }

    #[test]
    fn solve_should_succeed_with_walls_2() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'W', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec![' ', 'W', 'W', 'W', ' '],
            vec![' ', ' ', ' ', ' ', ' '],
        ];
        let m = Maze::new(Definition::from_vec(grid));
        let start = Point { row: 0, col: 1 };
        let end = Point { row: 2, col: 4 };
        let result = m.solve(start, end);
        match result {
            Ok(s) => {
                let expected_solution_path = Path {
                    points: vec![
                        Point { row: 0, col: 1 },
                        Point { row: 0, col: 0 },
                        Point { row: 1, col: 0 },
                        Point { row: 2, col: 0 },
                        Point { row: 2, col: 1 },
                        Point { row: 2, col: 2 },
                        Point { row: 3, col: 2 },
                        Point { row: 4, col: 2 },
                        Point { row: 4, col: 1 },
                        Point { row: 4, col: 0 },
                        Point { row: 5, col: 0 },
                        Point { row: 6, col: 0 },
                        Point { row: 6, col: 1 },
                        Point { row: 6, col: 2 },
                        Point { row: 6, col: 3 },
                        Point { row: 6, col: 4 },
                        Point { row: 5, col: 4 },
                        Point { row: 4, col: 4 },
                        Point { row: 3, col: 4 },
                        Point { row: 2, col: 4 },
                    ],
                };
                assert_eq!(s.path.points.len(), 20);
                assert_eq!(s.path, expected_solution_path);
            }
            Err(error) => {
                panic!(
                    "expected solve() to succeed but it returned the error {}",
                    error.message
                );
            }
        }
    }

    #[test]
    fn solve_should_succeed_with_walls_3() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'W', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', ' ', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec![' ', 'W', 'W', 'W', ' '],
            vec![' ', ' ', ' ', ' ', ' '],
        ];
        let m = Maze::new(Definition::from_vec(grid));
        let start = Point { row: 0, col: 1 };
        let end = Point { row: 1, col: 4 };
        let result = m.solve(start, end);
        match result {
            Ok(s) => {
                let expected_solution_path = Path {
                    points: vec![
                        Point { row: 0, col: 1 },
                        Point { row: 0, col: 0 },
                        Point { row: 1, col: 0 },
                        Point { row: 2, col: 0 },
                        Point { row: 2, col: 1 },
                        Point { row: 2, col: 2 },
                        Point { row: 2, col: 3 },
                        Point { row: 2, col: 4 },
                        Point { row: 1, col: 4 },
                    ],
                };
                assert_eq!(s.path.points.len(), 9);
                assert_eq!(s.path, expected_solution_path);
            }
            Err(error) => {
                panic!(
                    "expected solve() to succeed but it returned the error {}",
                    error.message
                );
            }
        }
    }
}
