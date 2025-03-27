use std::cmp::Ordering;
use std::io::{self};

use data_model::Maze;
use utils::LinePrinter;

use crate::Error;
use crate::maze_solution::MazeSolution;
use crate::MazePathDirection;
use crate::MazePath;
use crate::solver::Solver;

/// Represents a maze solver interface
pub trait MazeSolver {
    /// Attempts to solve the path between the start and end points defined within the maze instance
    ///
    /// # Returns
    ///
    /// A `Result` containing either the solution if successful, or an `Error` if an error occurs
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::{Maze, MazePoint};
    /// use maze::MazeSolver;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', 'W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W', ' '],
    ///    vec![' ', ' ', ' ', 'W', 'F'],
    ///    vec!['W', ' ', 'W', ' ', ' '],
    ///    vec![' ', ' ', ' ', 'W', ' '],
    ///    vec!['W', 'W', ' ', ' ', ' '],
    ///    vec!['W', 'W', ' ', 'W', ' '],
    /// ];
    /// let maze = Maze::from_vec(grid);
    /// let result = maze.solve();
    /// match result {
    ///    Ok(solution) => {
    ///       println!("Successfully solved maze, solution path => {}", solution.path);
    ///    }
    ///    Err(error) => {
    ///        panic!(
    ///            "failed to solve maze => {}",
    ///           error
    ///        );
    ///    }
    /// }
    /// ```
    fn solve(&self) -> Result<MazeSolution, Error>;
}
/// Represents a maze printer interface
pub trait MazePrinter {
    /// Print a maze instance to the given print target with the given start point, end point and solution path
    /// # Arguments
    /// * `print_target` - Print target
    /// * `start` - Start point
    /// * `end` - End point
    /// * `path` - Solution path
    ///
    /// # Returns
    ///
    /// Nothing
    ///
    /// # Examples
    ///
    /// ```
    /// use data_model::Maze;
    /// use maze::{MazePrinter, MazeSolver};
    /// use utils::StdoutLinePrinter;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', 'W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W', ' '],
    ///    vec![' ', ' ', ' ', 'W', 'F'],
    ///    vec!['W', ' ', 'W', ' ', ' '],
    ///    vec![' ', ' ', ' ', 'W', ' '],
    ///    vec!['W', 'W', ' ', ' ', ' '],
    ///    vec!['W', 'W', ' ', 'W', ' '],
    /// ];
    /// let maze = Maze::from_vec(grid);
    /// let result = maze.solve();
    /// match result {
    ///    Ok(solution) => {
    ///       println!("Successfully solved maze:");
    ///       let mut print_target = StdoutLinePrinter::new();
    ///       maze.print(&mut print_target, solution.path);
    ///    }
    ///    Err(error) => {
    ///        panic!(
    ///            "failed to solve maze => {}",
    ///           error
    ///        );
    ///    }
    /// }
    /// ```
    fn print(&self, print_target: &mut dyn LinePrinter, path: MazePath) -> Result<(), io::Error>;
}

impl MazeSolver for Maze {
    fn solve(&self) -> Result<MazeSolution, Error> {
        let s = Solver { maze: self };
        s.solve()
    }
}
impl MazePrinter for Maze {        
    fn print(&self, print_target: &mut dyn LinePrinter, path: MazePath) -> Result<(), io::Error> {
        if self.definition.row_count() == 0 || self.definition.col_count() == 0 {
            return Ok(());
        }
        let mut display_chars = self.definition.to_display_chars();
        let start = self.definition.get_start();
        let finish = self.definition.get_finish();

        for (path_idx, pt) in path.points.iter().enumerate() {
            if self.definition.is_valid(pt)
                && (start.is_none() || *pt != *(start.as_ref().unwrap()))
                && (finish.is_none() || *pt != *(finish.as_ref().unwrap()))
            {
                let mut direction = MazePathDirection::None;
                if (path_idx + 1) < path.points.len() {
                    let next_pt = &path.points[path_idx + 1];
                    if next_pt.row == pt.row {
                        direction = match pt.col.cmp(&next_pt.col) {
                            Ordering::Less => MazePathDirection::Right,
                            Ordering::Greater => MazePathDirection::Left,
                            Ordering::Equal => MazePathDirection::None,
                        };
                    } else if next_pt.col == pt.col {
                        direction = match pt.row.cmp(&next_pt.row) {
                            Ordering::Less => MazePathDirection::Down,
                            Ordering::Greater => MazePathDirection::Up,
                            Ordering::Equal => MazePathDirection::None,
                        };
                    }
                }

                display_chars[pt.row][pt.col] = direction.unicode_char();
            }
        }
        for row in display_chars.iter() {
            let row_chars: String = row.iter().collect();
            print_target.print_line(&row_chars)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data_model::{Maze, MazeDefinition, MazePoint};
    use utils::StdoutLinePrinter;

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
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let path = MazePath { points: vec![] };
        let mut print_target = StdoutLinePrinter::new();
        if let Err(error) = maze.print(&mut print_target, path) {
            panic!("Unexpected print() error: {}", error);
        }
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
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let path = MazePath {
            points: vec![
                MazePoint { row: 0, col: 1 },
                MazePoint { row: 0, col: 0 },
                MazePoint { row: 1, col: 0 },
                MazePoint { row: 2, col: 0 },
                MazePoint { row: 2, col: 1 },
                MazePoint { row: 2, col: 2 },
                MazePoint { row: 3, col: 2 },
                MazePoint { row: 4, col: 2 },
                MazePoint { row: 4, col: 1 },
                MazePoint { row: 4, col: 0 },
                MazePoint { row: 5, col: 0 },
                MazePoint { row: 6, col: 0 },
                MazePoint { row: 6, col: 1 },
                MazePoint { row: 6, col: 2 },
                MazePoint { row: 6, col: 3 },
                MazePoint { row: 6, col: 4 },
                MazePoint { row: 5, col: 4 },
                MazePoint { row: 4, col: 4 },
                MazePoint { row: 3, col: 4 },
                MazePoint { row: 2, col: 4 },
            ],
        };
        let mut print_target = StdoutLinePrinter::new();

        if let Err(error) = maze.print(&mut print_target, path) {
            panic!("Unexpected print() error: {}", error);
        }
    }

    #[test]
    fn solve_should_fail_with_missing_start_cell() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'W', ' '],
            vec![' ', ' ', 'W', ' '],
            vec![' ', 'F', 'W', ' '],
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(_) => panic_unexpected_solve_success(),
            Err(error) => assert_error_msg_eq(error, "no start cell found within maze"),
        }
    }

    #[test]
    fn solve_should_fail_with_missing_finish_cell() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W', ' '],
            vec![' ', ' ', 'W', ' '],
            vec![' ', ' ', 'W', ' '],
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(_) => panic_unexpected_solve_success(),
            Err(error) => assert_error_msg_eq(error, "no finish cell found within maze"),
        }
    }

    #[test]
    fn solve_should_succeed_for_maze_with_no_walls_1() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'F'],
            vec!['S', ' ', ' ']
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
                let expected_solution_path = MazePath {
                    points: vec![
                        MazePoint { row: 1, col: 0 },
                        MazePoint { row: 0, col: 0 },
                        MazePoint { row: 0, col: 1 },
                        MazePoint { row: 0, col: 2 },
                    ],
                };
                assert_eq!(solution.path.points.len(), 4);
                assert_eq!(solution.path, expected_solution_path);
            }
            Err(error) => panic_unexpected_solve_error(error),
        }
    }

    #[test]
    fn solve_should_succeed_for_maze_with_no_walls_2() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'F'],
            vec!['S', ' ', ' ', ' ']
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
                let expected_solution_path = MazePath {
                    points: vec![
                        MazePoint { row: 1, col: 0 },
                        MazePoint { row: 0, col: 0 },
                        MazePoint { row: 0, col: 1 },
                        MazePoint { row: 0, col: 2 },
                        MazePoint { row: 0, col: 3 },
                    ],
                };
                assert_eq!(solution.path.points.len(), 5);
                assert_eq!(solution.path, expected_solution_path);
            }
            Err(error) => panic_unexpected_solve_error(error),
        }
    }

    #[test]
    fn solve_should_succeed_for_maze_with_no_walls_3() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', ' '],
            vec![' ', 'S', ' ', ' '],
            vec![' ', ' ', 'F', ' '],
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
                let expected_solution_path = MazePath {
                    points: vec![
                        MazePoint { row: 1, col: 1 },
                        MazePoint { row: 1, col: 2 },
                        MazePoint { row: 2, col: 2 },
                    ],
                };
                assert_eq!(solution.path.points.len(), 3);
                assert_eq!(solution.path, expected_solution_path);
            }
            Err(error) => panic_unexpected_solve_error(error),
        }
    }

    #[test]
    fn solve_should_fail_as_no_solution_due_to_blocking_wall() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'W', 'F'],
            vec!['S', ' ', 'W', ' '],
            vec![' ', ' ', 'W', ' '],
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(_) => panic_unexpected_solve_success(),
            Err(error) => {
                assert_error_msg_eq(error, "no solution found");
            }
        }
    }

    #[test]
    fn solve_should_succeed_with_walls_1() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', 'W', ' ', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', 'F'],
            vec!['W', ' ', 'W', ' ', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec!['W', 'W', ' ', ' ', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
                let expected_solution_path = MazePath {
                    points: vec![
                        MazePoint { row: 0, col: 0 },
                        MazePoint { row: 1, col: 0 },
                        MazePoint { row: 2, col: 0 },
                        MazePoint { row: 2, col: 1 },
                        MazePoint { row: 3, col: 1 },
                        MazePoint { row: 4, col: 1 },
                        MazePoint { row: 4, col: 2 },
                        MazePoint { row: 5, col: 2 },
                        MazePoint { row: 5, col: 3 },
                        MazePoint { row: 5, col: 4 },
                        MazePoint { row: 4, col: 4 },
                        MazePoint { row: 3, col: 4 },
                        MazePoint { row: 2, col: 4 },
                    ],
                };
                assert_eq!(solution.path.points.len(), 13);
                assert_eq!(solution.path, expected_solution_path);
            }
            Err(error) => panic_unexpected_solve_error(error),
        }
    }

    #[test]
    fn solve_should_succeed_with_walls_2() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'S', 'W', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', 'F'],
            vec!['W', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec![' ', 'W', 'W', 'W', ' '],
            vec![' ', ' ', ' ', ' ', ' '],
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
                let expected_solution_path = MazePath {
                    points: vec![
                        MazePoint { row: 0, col: 1 },
                        MazePoint { row: 0, col: 0 },
                        MazePoint { row: 1, col: 0 },
                        MazePoint { row: 2, col: 0 },
                        MazePoint { row: 2, col: 1 },
                        MazePoint { row: 2, col: 2 },
                        MazePoint { row: 3, col: 2 },
                        MazePoint { row: 4, col: 2 },
                        MazePoint { row: 4, col: 1 },
                        MazePoint { row: 4, col: 0 },
                        MazePoint { row: 5, col: 0 },
                        MazePoint { row: 6, col: 0 },
                        MazePoint { row: 6, col: 1 },
                        MazePoint { row: 6, col: 2 },
                        MazePoint { row: 6, col: 3 },
                        MazePoint { row: 6, col: 4 },
                        MazePoint { row: 5, col: 4 },
                        MazePoint { row: 4, col: 4 },
                        MazePoint { row: 3, col: 4 },
                        MazePoint { row: 2, col: 4 },
                    ],
                };
                assert_eq!(solution.path.points.len(), 20);
                assert_eq!(solution.path, expected_solution_path);
            }
            Err(error) => panic_unexpected_solve_error(error),
        }
    }

    #[test]
    fn solve_should_succeed_with_walls_3() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'S', 'W', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', 'F'],
            vec![' ', ' ', ' ', ' ', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec![' ', 'W', 'W', 'W', ' '],
            vec![' ', ' ', ' ', ' ', ' '],
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
                let expected_solution_path = MazePath {
                    points: vec![
                        MazePoint { row: 0, col: 1 },
                        MazePoint { row: 0, col: 0 },
                        MazePoint { row: 1, col: 0 },
                        MazePoint { row: 2, col: 0 },
                        MazePoint { row: 2, col: 1 },
                        MazePoint { row: 2, col: 2 },
                        MazePoint { row: 2, col: 3 },
                        MazePoint { row: 2, col: 4 },
                        MazePoint { row: 1, col: 4 },
                    ],
                };
                assert_eq!(solution.path.points.len(), 9);
                assert_eq!(solution.path, expected_solution_path);
            }
            Err(error) => panic_unexpected_solve_error(error),
        }
    }

    fn panic_unexpected_solve_success() {
        panic!("expected solve() to return Err, but it returned Ok");
    }

    fn panic_unexpected_solve_error(error: Error) {
        panic!(
            "expected solve() to succeed but it returned the error {}",
            error
        );
    }

    fn assert_error_msg_eq(err: Error, msg: &str) {
        assert_eq!(format!("{}", err), msg);
    }
}
