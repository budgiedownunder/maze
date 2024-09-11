use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::io::{self};

use crate::solution::Solution;
use crate::Definition;
use crate::Direction;
use crate::LinePrinter;
use crate::MazeError;
use crate::Path;
use crate::Solver;

#[allow(dead_code)]
#[derive(Serialize, Deserialize, Clone)]

/// Represents a maze
pub struct Maze {
    #[serde(skip_serializing, default)]
    pub id: String,
    pub name: String,
    /// Definition, containing the layout of the maze
    pub definition: Definition,
}

impl PartialEq for Maze {
    fn eq(&self, other: &Self) -> bool {
        if self.id != other.id {
            return false;
        }

        if let (Ok(self_json), Ok(other_json)) = (self.to_json(), other.to_json()) {
            return self_json == other_json;
        }

        false
    }
}

impl Maze {
    /// Creates a new maze instance with the given definition
    /// # Arguments
    ///
    /// * `grid` - Maze definition
    ///
    /// # Returns
    ///
    /// A new maze instance
    ///
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a start, finish and a wall in the last column
    ///
    /// ```
    /// use maze::Definition;
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let def = Definition::from_vec(grid);
    /// let maze = Maze::new(def);
    /// assert_eq!(maze.definition.row_count(), 2);
    /// assert_eq!(maze.definition.col_count(), 3);
    pub fn new(definition: Definition) -> Maze {
        Maze {
            id: "".to_string(),
            name: "".to_string(),
            definition,
        }
    }
    /// Resets a maze definition instance to empty
    ///
    /// # Returns
    ///
    /// The maze definition instance
    ///
    /// # Examples
    ///
    /// Create a definition with 2 rows and 3 columns, verify its dimensions, reset it and
    /// then confirm it is empty
    /// ```
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze = Maze::from_vec(grid);
    /// assert_eq!(maze.definition.row_count(), 2);
    /// assert_eq!(maze.definition.col_count(), 3);
    /// maze.reset();
    /// assert_eq!(maze.definition.is_empty(), true);
    /// ```
    pub fn reset(&mut self) -> &mut Self {
        self.definition.reset();
        self
    }
    /// Creates a new maze definition for the given vector of cell definition character rows, where:
    /// - `'S'`:  Represents the starting cell (limited to one).
    /// - `'F'`:  Represents the finishing cell (limited to one).
    /// - `'W'`:  Represents a wall.
    /// - `' '`:  Represents an empty cell.
    ///
    /// # Arguments
    ///
    /// * `grid` - Vector of row-column cell states
    ///
    /// # Returns
    ///
    /// A new maze instance
    ///
    ///
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a start, finish and a wall in the last column
    ///
    /// ```
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let maze = Maze::from_vec(grid);
    /// assert_eq!(maze.definition.row_count(), 2);
    /// assert_eq!(maze.definition.col_count(), 3);
    pub fn from_vec(grid: Vec<Vec<char>>) -> Self {
        Maze {
            id: "".to_string(),
            name: "".to_string(),
            definition: Definition::from_vec(grid),
        }
    }
    /// Generates the JSON string representation for the maze
    ///
    /// # Returns
    ///
    /// JSON string representing the maze definition
    ///
    ///
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a start, finish and a wall in the last column
    /// and then convert it to JSON and print it
    /// ```
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let maze = Maze::from_vec(grid);
    /// assert_eq!(maze.definition.row_count(), 2);
    /// assert_eq!(maze.definition.col_count(), 3);
    /// match maze.to_json() {
    ///     Ok(json) => {
    ///         println!("JSON: {}", json);
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to convert maze to JSON => {}",
    ///           error
    ///        );
    ///     }
    /// }
    pub fn to_json(&self) -> Result<String, MazeError> {
        Ok(serde_json::to_string(&self)?)
    }
    /// Initializes a maze instance by reading the JSON string content provided
    ///
    /// # Returns
    ///
    /// This function will return an error if the JSON could not be read
    ///
    /// # Examples
    ///
    /// Create an empty maze and then reinitialize it from a JSON string definition
    /// containing 2 rows and 3 columns  
    /// ```
    /// use maze::Definition;
    /// use maze::Maze;
    /// let mut maze = Maze::new(Definition::new(0, 0));
    /// let json = r#"{"name":"my_maze", "definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#;
    /// match maze.from_json(json) {
    ///     Ok(()) => {
    ///         println!(
    ///             "JSON successfully read into Maze => new rows = {}, new columns = {}",
    ///             maze.definition.row_count(),
    ///             maze.definition.col_count()
    ///         );
    ///     }
    ///     Err(error) => {
    ///        panic!(
    ///            "failed to read JSON into maze => {}",
    ///           error
    ///        );
    ///     }
    /// }
    pub fn from_json(&mut self, json: &str) -> Result<(), MazeError> {
        let temp: Maze = serde_json::from_str(json)?;
        *self = temp;
        Ok(())
    }
    /// Attempts to solve the path between the start and end points defined within the maze instance
    ///
    /// # Returns
    ///
    /// A `Result` containing either the solution if successful, or a `MazeError` if an error occurs
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::Maze;
    /// use maze::Point;
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
    pub fn solve(&self) -> Result<Solution, MazeError> {
        let s = Solver { maze: self };
        s.solve()
    }
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
    /// use maze::StdoutLinePrinter;
    /// use maze::Maze;
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
    pub fn print(&self, print_target: &mut dyn LinePrinter, path: Path) -> Result<(), io::Error> {
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
                let mut direction = Direction::None;
                if (path_idx + 1) < path.points.len() {
                    let next_pt = &path.points[path_idx + 1];
                    if next_pt.row == pt.row {
                        direction = match pt.col.cmp(&next_pt.col) {
                            Ordering::Less => Direction::Right,
                            Ordering::Greater => Direction::Left,
                            Ordering::Equal => Direction::None,
                        };
                    } else if next_pt.col == pt.col {
                        direction = match pt.row.cmp(&next_pt.row) {
                            Ordering::Less => Direction::Down,
                            Ordering::Greater => Direction::Up,
                            Ordering::Equal => Direction::None,
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
    use crate::Point;
    use crate::StdoutLinePrinter;

    #[test]
    fn can_create_new_maze_from_vector() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', ' ']
        ];
        let maze = Maze::from_vec(grid);
        assert_eq!(maze.definition.row_count(), 2);
        assert_eq!(maze.definition.col_count(), 3);
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
        let maze = Maze::new(Definition::new(2, 3));
        assert_eq!(maze.definition.row_count(), 2);
        assert_eq!(maze.definition.col_count(), 3);
    }

    #[test]
    fn can_reset_to_empty() {
        let mut maze = Maze::new(Definition::new(2, 3));
        assert_eq!(maze.definition.row_count(), 2);
        assert_eq!(maze.definition.col_count(), 3);
        assert!(!maze.definition.is_empty());
        assert!(maze.reset().definition.is_empty())
    }

    #[test]
    fn can_serialize_empty() {
        let maze = Maze::new(Definition::new(0, 0));
        let s = maze.to_json().expect("Failed to serialize");
        assert_eq!(s, r#"{"name":"","definition":{"grid":[]}}"#);
    }

    #[test]
    fn can_serialize_non_empty() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'W', ' '],
            vec![' ', ' ', 'W']
        ];
        let maze = Maze::new(Definition::from_vec(grid));
        let s = maze.to_json().expect("Failed to serialize");
        assert_eq!(
            s,
            r#"{"name":"","definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#
        );
    }

    #[test]
    fn can_deserialize_empty() {
        let mut maze = Maze::new(Definition::new(10, 10));
        let s = r#"{"name":"my_maze","definition":{"grid":[]}}"#;
        maze.from_json(s).expect("Failed to deserialize");
        assert!(maze.definition.is_empty());
    }

    #[test]
    fn can_deserialize_non_empty() {
        let mut maze = Maze::new(Definition::new(10, 10));
        let s = r#"{"name":"my_maze","definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#;
        maze.from_json(s).expect("Failed to deserialize");
        assert_eq!(maze.definition.row_count(), 2);
        assert_eq!(maze.definition.col_count(), 3);
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
        let maze = Maze::new(Definition::from_vec(grid));
        let path = Path { points: vec![] };
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
        let maze = Maze::new(Definition::from_vec(grid));
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
        let mut print_target = StdoutLinePrinter::new();

        if let Err(error) = maze.print(&mut print_target, path) {
            panic!("Unexpected print() error: {}", error);
        }
    }

    #[test]
    fn cannot_load_json_with_invalid_content_eof() {
        run_from_json_test_with_invalid_content("{", ExpectedSerdeErrorKind::UnexpectedEof);
    }

    #[test]
    fn cannot_load_json_with_invalid_content_syntax_1() {
        run_from_json_test_with_invalid_content("{x", ExpectedSerdeErrorKind::Syntax);
    }

    #[test]
    fn cannot_load_json_with_invalid_content_syntax_2() {
        run_from_json_test_with_invalid_content(r#"{"x"}"#, ExpectedSerdeErrorKind::Syntax);
    }

    #[test]
    fn cannot_load_json_with_invalid_content_syntax_3() {
        run_from_json_test_with_invalid_content(r#"{"x":}"#, ExpectedSerdeErrorKind::Syntax);
    }

    #[test]
    fn cannot_load_json_with_invalid_content_syntax_4() {
        run_from_json_test_with_invalid_content("}", ExpectedSerdeErrorKind::Syntax);
    }

    #[test]
    fn cannot_load_json_with_invalid_content_syntax_5() {
        run_from_json_test_with_invalid_content("{{}", ExpectedSerdeErrorKind::Syntax);
    }

    #[test]
    fn cannot_load_json_with_invalid_content_data_1() {
        run_from_json_test_with_invalid_content(
            r#"{"definition1":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#,
            ExpectedSerdeErrorKind::Data,
        );
    }

    #[test]
    fn cannot_load_json_with_invalid_content_data_2() {
        run_from_json_test_with_invalid_content(
            r#"{"definition":{"grid2":[[" ","W"," "],[" "," ","W"]]}}"#,
            ExpectedSerdeErrorKind::Data,
        );
    }

    #[test]
    fn cannot_load_json_with_invalid_content_data_3() {
        run_from_json_test_with_invalid_content(
            r#"{"definition":{"grid":"invalid data"}}"#,
            ExpectedSerdeErrorKind::Data,
        );
    }

    #[test]
    fn solve_should_fail_with_missing_start_cell() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', 'W', ' '],
            vec![' ', ' ', 'W', ' '],
            vec![' ', 'F', 'W', ' '],
        ];
        let maze = Maze::new(Definition::from_vec(grid));
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
        let maze = Maze::new(Definition::from_vec(grid));
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
        let maze = Maze::new(Definition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
                let expected_solution_path = Path {
                    points: vec![
                        Point { row: 1, col: 0 },
                        Point { row: 0, col: 0 },
                        Point { row: 0, col: 1 },
                        Point { row: 0, col: 2 },
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
        let maze = Maze::new(Definition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
                let expected_solution_path = Path {
                    points: vec![
                        Point { row: 1, col: 0 },
                        Point { row: 0, col: 0 },
                        Point { row: 0, col: 1 },
                        Point { row: 0, col: 2 },
                        Point { row: 0, col: 3 },
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
        let maze = Maze::new(Definition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
                let expected_solution_path = Path {
                    points: vec![
                        Point { row: 1, col: 1 },
                        Point { row: 1, col: 2 },
                        Point { row: 2, col: 2 },
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
        let maze = Maze::new(Definition::from_vec(grid));
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
        let maze = Maze::new(Definition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
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
        let maze = Maze::new(Definition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
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
        let maze = Maze::new(Definition::from_vec(grid));
        let result = maze.solve();
        match result {
            Ok(solution) => {
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
                assert_eq!(solution.path.points.len(), 9);
                assert_eq!(solution.path, expected_solution_path);
            }
            Err(error) => panic_unexpected_solve_error(error),
        }
    }

    // Helper functions and definitions
    enum ExpectedSerdeErrorKind {
        Data,
        Syntax,
        UnexpectedEof,
    }

    fn run_from_json_test_with_invalid_content(
        content: &str,
        expected_error_kind: ExpectedSerdeErrorKind,
    ) {
        let mut maze = Maze::new(Definition::new(0, 0));
        match maze.from_json(content) {
            Ok(_) => {
                panic!("Unexpectedly loaded json despite having invalid content");
            }
            Err(error) => match error {
                MazeError::SerdeJson(ref serdejson_error) => match expected_error_kind {
                    ExpectedSerdeErrorKind::Data => {
                        if !serdejson_error.is_data() {
                            panic!(
                                "Serde data error expected (got SerdeJson error: {})",
                                serdejson_error
                            );
                        }
                    }
                    ExpectedSerdeErrorKind::Syntax => {
                        if !serdejson_error.is_syntax() {
                            panic!(
                                "Serde syntax error expected (got SerdeJson error: {})",
                                serdejson_error
                            );
                        }
                    }
                    ExpectedSerdeErrorKind::UnexpectedEof => {
                        if !serdejson_error.is_eof() {
                            panic!(
                                "Serde unexpected EOF error expected (got SerdeJson error: {})",
                                serdejson_error
                            );
                        }
                    }
                },
                _ => panic!("Unxpected error encountered (got error: {})", error),
            },
        }
    }

    fn panic_unexpected_solve_success() {
        panic!("expected solve() to return Err, but it returned Ok");
    }

    fn panic_unexpected_solve_error(error: MazeError) {
        panic!(
            "expected solve() to succeed but it returned the error {}",
            error
        );
    }

    fn assert_error_msg_eq(err: MazeError, msg: &str) {
        assert_eq!(format!("{}", err), msg);
    }
}
