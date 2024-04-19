use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use crate::solution::Solution;
use crate::Definition;
use crate::Direction;
use crate::MazeError;
use crate::Path;
use crate::Point;
use crate::Solver;

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]

/// Represents a maze
pub struct Maze {
    /// Definition, containing the layout of the maze
    pub definition: Definition,
}

impl Maze {
    /// Creates a new maze instance with the given definition
    /// # Arguments
    /// * `grid` - Maze definition
    ///
    /// # Returns
    ///
    /// A new maze instance
    ///
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a wall in the last column
    ///
    /// ```
    /// use maze::Definition;
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec![' ', ' ', 'W'],
    ///    vec![' ', ' ', 'W']
    /// ];
    /// let d = Definition::from_vec(grid);
    /// let m = Maze::new(d);
    /// assert_eq!(m.definition.row_count(), 2);
    /// assert_eq!(m.definition.col_count(), 3);
    pub fn new(definition: Definition) -> Maze {
        Maze { definition }
    }
    /// Creates a new maze definition for the given vector of cell definition character rows, where:
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
    /// Create a 2 row x 3 column definition with a wall in the last column
    ///
    /// ```
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec![' ', ' ', 'W'],
    ///    vec![' ', ' ', 'W']
    /// ];
    /// let m = Maze::from_vec(grid);
    /// assert_eq!(m.definition.row_count(), 2);
    /// assert_eq!(m.definition.col_count(), 3);
    pub fn from_vec(grid: Vec<Vec<char>>) -> Self {
        Maze {
            definition: Definition::from_vec(grid),
        }
    }
    /// Saves a maze definition to a file (as JSON), optionally overwriting any existing file
    ///
    /// # Arguments
    ///
    /// * `path` - file path
    /// * `overwrite` - flag indicating whether to overwrite any existing file
    ///
    /// # Returns
    ///
    /// This function will return an error if the fiel cannotbe
    ///
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a wall in the last column and then save it to the local file `my_file.json`, overwriting
    /// any existing file
    ///
    /// ```
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec![' ', ' ', 'W'],
    ///    vec![' ', ' ', 'W']
    /// ];
    /// let m = Maze::from_vec(grid);
    /// let path = "./my_maze.json";
    /// match m.save_to_file(path, true) {
    ///     Ok(_) => println!("Successfully saved to file: {}", path),
    ///     Err(err) => println!("Failed to save to file: {}", err)
    /// }
    pub fn save_to_file(&self, path: &str, overwrite: bool) -> Result<(), MazeError> {
        if !overwrite {
            if let Ok(_metadata) = fs::metadata(path) {
                return Err(MazeError::new("file path already exists"));
            }
        }
        let s = serde_json::to_string(&self)?;
        let mut file = File::create(path)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
    /// Loads a maze definition from a file
    ///
    /// # Arguments
    ///
    /// * `path` - file path
    ///
    /// # Returns
    ///
    /// A new maze instance
    ///
    ///
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a wall in the last column and then save it to the local file `my_file.json`, overwriting
    /// any existing file. Then attempt to reload that file into a second maze definition.
    ///
    /// ```
    /// use maze::Definition;
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec![' ', ' ', 'W'],
    ///    vec![' ', ' ', 'W']
    /// ];
    /// let m1 = Maze::from_vec(grid);
    /// let path = "./my_maze.json";
    /// match m1.save_to_file(path, true) {
    ///     Ok(_) => println!("Successfully saved to file: {}", path),
    ///     Err(err) => println!("Failed to save to file: {}", err)
    /// }
    /// let mut m2 = Maze::new(Definition::new(0, 0));
    /// match m2.load_from_file(path) {
    ///     Ok(_) => {
    ///         println!("Successfully loaded from file: {}", path);
    ///     }
    ///     Err(err) => println!("Failed to load from file: {} - {}", path, err)
    /// }
    pub fn load_from_file(&mut self, path: &str) -> Result<(), MazeError> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        *self = serde_json::from_str(&contents).expect("Failed to deserialize file content");
        Ok(())
    }
    /// Attempts to solve the path between a start and end point within the maze instance
    /// # Arguments
    /// * `start` - Start point
    /// * `end` - End point
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
    ///    vec![' ', 'W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W', ' '],
    ///    vec![' ', ' ', ' ', 'W', ' '],
    ///    vec!['W', ' ', 'W', ' ', ' '],
    ///    vec![' ', ' ', ' ', 'W', ' '],
    ///    vec!['W', 'W', ' ', ' ', ' '],
    ///    vec!['W', 'W', ' ', 'W', ' '],
    /// ];
    /// let m = Maze::from_vec(grid);
    /// let start = Point { row: 0, col: 0 };
    /// let end = Point { row: 2, col: 4 };
    /// let result = m.solve(start, end);
    /// match result {
    ///    Ok(solution) => {
    ///       println!("Successfully solved maze, solution path => {}", solution.path);
    ///    }
    ///    Err(error) => {
    ///        panic!(
    ///            "failed to solve maze => {}",
    ///           error.message
    ///        );
    ///    }
    /// }
    /// ```
    pub fn solve(&self, start: Point, end: Point) -> Result<Solution, MazeError> {
        let s = Solver { maze: &self };
        s.solve(start, end)
    }

    /// Print a maze instance with the given start point, end point and solution path
    /// # Arguments
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
    /// use maze::Maze;
    /// use maze::Point;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec![' ', 'W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W', ' '],
    ///    vec![' ', ' ', ' ', 'W', ' '],
    ///    vec!['W', ' ', 'W', ' ', ' '],
    ///    vec![' ', ' ', ' ', 'W', ' '],
    ///    vec!['W', 'W', ' ', ' ', ' '],
    ///    vec!['W', 'W', ' ', 'W', ' '],
    /// ];
    /// let m = Maze::from_vec(grid);
    /// let start = Point { row: 0, col: 0 };
    /// let end = Point { row: 2, col: 4 };
    /// let result = m.solve(start.clone(), end.clone());
    /// match result {
    ///    Ok(solution) => {
    ///       println!("Successfully solved maze:");
    ///       m.print(start, end, solution.path);
    ///    }
    ///    Err(error) => {
    ///        panic!(
    ///            "failed to solve maze => {}",
    ///           error.message
    ///        );
    ///    }
    /// }
    /// ```
    pub fn print(&self, start: Point, end: Point, path: Path) {
        let mut display_chars = self.definition.to_display_chars();
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

                display_chars[pt.row][pt.col] = direction.unicode_char();
            }
            path_idx += 1;
        }
        if self.definition.is_valid(&start) {
            display_chars[start.row][start.col] = 'S';
        }
        if self.definition.is_valid(&end) {
            display_chars[end.row][end.col] = 'F';
        }
        for row in display_chars.iter() {
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
    fn can_serialize_empty() {
        let m = Maze::new(Definition::new(0, 0));
        let s = serde_json::to_string(&m).expect("Failed to serialize");
        assert_eq!(s, r#"{"definition":{"grid":[]}}"#);
    }

    #[test]
    fn can_serialize_non_empty() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'W', ' '],
            vec![' ', ' ', 'W']
        ];
        let m = Maze::new(Definition::from_vec(grid));
        let s = serde_json::to_string(&m).expect("Failed to serialize");
        assert_eq!(
            s,
            r#"{"definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#
        );
    }

    #[test]
    fn can_deserialize_empty() {
        let s = r#"{"definition":{"grid":[]}}"#;
        let m: Maze = serde_json::from_str(&s).expect("Failed to deserialize");
        assert_eq!(m.definition.row_count(), 0);
        assert_eq!(m.definition.col_count(), 0);
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
    fn can_save_to_valid_file_path() {
        let grid: Vec<Vec<char>> = vec![vec![' ', ' ', 'W'], vec![' ', ' ', 'W']];
        let m = Maze::from_vec(grid);
        let path = "./maze_1.json";
        match m.save_to_file(path, true) {
            Ok(_) => println!("Successfully saved to file: {}", path),
            Err(err) => panic!("Failed to save to file: {}", err),
        }
        std::fs::remove_file(path).expect("Failed to delete file");
    }

    #[test]
    fn cannot_save_to_invalid_file_path() {
        let grid: Vec<Vec<char>> = vec![vec![' ', ' ', 'W'], vec![' ', ' ', 'W']];
        let m = Maze::from_vec(grid);
        let path = "";
        match m.save_to_file(path, true) {
            Ok(_) => panic!("Successfully saved to file: {} but did not expect to", path),
            Err(err) => {
                assert!(
                    err.message
                        .starts_with("The system cannot find the path specified"),
                    "error returned does not start with expected path not found text: `{}` was returned",
                    err.message
                );
            }
        }
    }

    #[test]
    #[should_panic(expected = "file path already exists")]
    fn cannot_save_to_existing_file_path_if_overwrite_disabled() {
        let grid: Vec<Vec<char>> = vec![vec![' ', ' ', 'W'], vec![' ', ' ', 'W']];
        let m = Maze::from_vec(grid);
        let path = "./maze_2.json";
        let mut _file = File::create(path).expect("Failed to create file");

        match m.save_to_file(path, false) {
            Ok(_) => {
                std::fs::remove_file(path).expect("Failed to delete file");
                panic!(
                    "Successfully saved to existing file: {} despite overwrite being false",
                    path
                );
            }
            Err(err) => {
                std::fs::remove_file(path).expect("Failed to delete file");
                panic!("{}", err);
            }
        }
    }

    #[test]
    fn can_save_to_existing_file_path_if_overwrite_enabled() {
        let grid: Vec<Vec<char>> = vec![vec![' ', ' ', 'W'], vec![' ', ' ', 'W']];
        let m = Maze::from_vec(grid);
        let path = "./maze_3.json";
        let mut _file = File::create(path).expect("Failed to create file");

        match m.save_to_file(path, true) {
            Ok(_) => {
                std::fs::remove_file(path).expect("Failed to delete file");
            }
            Err(err) => {
                std::fs::remove_file(path).expect("Failed to delete file");
                panic!("{}", err);
            }
        }
    }

    #[test]
    fn can_load_from_valid_file_path() {
        let grid: Vec<Vec<char>> = vec![vec![' ', ' ', 'W'], vec![' ', ' ', 'W']];
        let m1 = Maze::from_vec(grid);
        let path = "./maze_4.json";
        match m1.save_to_file(path, true) {
            Ok(_) => {}
            Err(err) => panic!("Failed to save to file: {}", err),
        }
        let mut m2 = Maze::new(Definition::new(0, 0));
        match m2.load_from_file(path) {
            Ok(_) => {
                assert_eq!(m2.definition.row_count(), m1.definition.row_count());
                assert_eq!(m2.definition.col_count(), m1.definition.col_count());
            }
            Err(err) => panic!("Failed to load from: {} - {}", path, err),
        }
        std::fs::remove_file(path).expect("Failed to delete file");
    }

    #[test]
    fn cannot_load_from_invalid_file_path() {
        let path = "./maze_does_not_exist.json";
        let mut m = Maze::new(Definition::new(0, 0));
        match m.load_from_file(path) {
            Ok(_) => panic!("File should not exist"),
            Err(err) => {
                assert!(
                    err.message
                        .starts_with("The system cannot find the file specified"),
                    "error returned does not start with expected file not found text: `{}` was returned",
                    err.message
                );
            }
        }
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
