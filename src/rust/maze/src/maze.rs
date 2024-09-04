use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs::File;
use std::io::{self, BufReader, Write};
use std::path::{Path as StdPath, PathBuf};

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
    /// Definition, containing the layout of the maze
    pub definition: Definition,
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
    /// let d = Definition::from_vec(grid);
    /// let m = Maze::new(d);
    /// assert_eq!(m.definition.row_count(), 2);
    /// assert_eq!(m.definition.col_count(), 3);
    pub fn new(definition: Definition) -> Maze {
        Maze { definition }
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
    /// let mut m = Maze::from_vec(grid);
    /// assert_eq!(m.definition.row_count(), 2);
    /// assert_eq!(m.definition.col_count(), 3);
    /// m.reset();
    /// assert_eq!(m.definition.is_empty(), true);
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
    /// let m = Maze::from_vec(grid);
    /// assert_eq!(m.definition.row_count(), 2);
    /// assert_eq!(m.definition.col_count(), 3);
    pub fn from_vec(grid: Vec<Vec<char>>) -> Self {
        Maze {
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
    /// let m = Maze::from_vec(grid);
    /// assert_eq!(m.definition.row_count(), 2);
    /// assert_eq!(m.definition.col_count(), 3);
    /// match m.to_json() {
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
    /// let mut m = Maze::new(Definition::new(0, 0));
    /// let json = r#"{"definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#;
    /// match m.from_json(json) {
    ///     Ok(()) => {
    ///         println!(
    ///             "JSON successfully read into Maze => new rows = {}, new columns = {}",
    ///             m.definition.row_count(),
    ///             m.definition.col_count()
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
    /// Saves a maze definition to a file (as JSON), optionally overwriting any existing file
    ///
    /// # Arguments
    ///
    /// * `path` - file path
    /// * `overwrite` - flag indicating whether to overwrite any existing file
    ///
    /// # Returns
    ///
    /// This function will return an error if the definition cannot be saved
    ///
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a start, finish and a wall in the last column and
    /// then save it to the local file `my_file.json`, overwriting any existing file
    ///
    /// ```
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let m = Maze::from_vec(grid);
    /// let path = "./my_maze.json";
    /// match m.save_to_file(path, true) {
    ///     Ok(_) => println!("Successfully saved to file: {}", path),
    ///     Err(error) => println!("Failed to save to file: {}", error)
    /// }
    pub fn save_to_file(&self, path: &str, overwrite: bool) -> Result<(), MazeError> {
        if !overwrite {
            let os_path = PathBuf::from(path);
            if StdPath::new(&os_path).exists() {
                return Err(MazeError::new("file path already exists".to_string()));
            }
        }
        let s = self.to_json()?;
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
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a start, finish and a wall in the last column and then
    /// save it to the local file `my_file.json`, overwriting any existing file. Then attempt to reload
    /// that file into a second maze definition.
    ///
    /// ```
    /// use maze::Definition;
    /// use maze::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let m1 = Maze::from_vec(grid);
    /// let path = "./my_maze.json";
    /// match m1.save_to_file(path, true) {
    ///     Ok(_) => println!("Successfully saved to file: {}", path),
    ///     Err(error) => println!("Failed to save to file: {}", error)
    /// }
    /// let mut m2 = Maze::new(Definition::new(0, 0));
    /// match m2.load_from_file(path) {
    ///     Ok(_) => {
    ///         println!("Successfully loaded from file: {}", path);
    ///     }
    ///     Err(error) => println!("Failed to load from file: {} - {}", path, error)
    /// }
    pub fn load_from_file(&mut self, path: &str) -> Result<(), MazeError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        match serde_json::from_reader(reader) {
            Ok(result) => {
                *self = result;
                Ok(())
            }
            Err(err) => Err(MazeError::from(err)),
        }
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
    /// let m = Maze::from_vec(grid);
    /// let result = m.solve();
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
    /// let m = Maze::from_vec(grid);
    /// let result = m.solve();
    /// match result {
    ///    Ok(solution) => {
    ///       println!("Successfully solved maze:");
    ///       let mut print_target = StdoutLinePrinter::new();
    ///       m.print(&mut print_target, solution.path);
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
    fn can_reset_to_empty() {
        let mut m = Maze::new(Definition::new(2, 3));
        assert_eq!(m.definition.row_count(), 2);
        assert_eq!(m.definition.col_count(), 3);
        assert!(!m.definition.is_empty());
        assert!(m.reset().definition.is_empty())
    }

    #[test]
    fn can_serialize_empty() {
        let m = Maze::new(Definition::new(0, 0));
        let s = m.to_json().expect("Failed to serialize");
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
        let s = m.to_json().expect("Failed to serialize");
        assert_eq!(
            s,
            r#"{"definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#
        );
    }

    #[test]
    fn can_deserialize_empty() {
        let mut m = Maze::new(Definition::new(10, 10));
        let s = r#"{"definition":{"grid":[]}}"#;
        m.from_json(s).expect("Failed to deserialize");
        assert!(m.definition.is_empty());
    }

    #[test]
    fn can_deserialize_non_empty() {
        let mut m = Maze::new(Definition::new(10, 10));
        let s = r#"{"definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#;
        m.from_json(s).expect("Failed to deserialize");
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
        let path = Path { points: vec![] };
        let mut print_target = StdoutLinePrinter::new();
        if let Err(error) = m.print(&mut print_target, path) {
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
        let m = Maze::new(Definition::from_vec(grid));
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

        if let Err(error) = m.print(&mut print_target, path) {
            panic!("Unexpected print() error: {}", error);
        }
    }

    #[test]
    fn can_save_to_valid_file_path() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W'], 
            vec!['F', ' ', 'W']
        ];
        let m = Maze::from_vec(grid);
        let path = "./maze_1.json";
        match m.save_to_file(path, true) {
            Ok(_) => println!("Successfully saved to file: {}", path),
            Err(error) => panic!("Failed to save to file: {}", error),
        }
        delete_test_file(path);
    }

    #[test]
    fn cannot_save_to_invalid_file_path() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W'], 
            vec!['F', ' ', 'W']
        ];
        let m = Maze::from_vec(grid);
        let path = "";
        match m.save_to_file(path, true) {
            Ok(_) => panic!("Successfully saved to file: {} but did not expect to", path),
            Err(error) => assert_io_err_not_found(error),
        }
    }

    #[test]
    #[should_panic(expected = "file path already exists")]
    fn cannot_save_to_existing_file_path_if_overwrite_disabled() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W'], 
            vec!['F', ' ', 'W']
        ];
        let m = Maze::from_vec(grid);
        let path = "./maze_2.json";
        let mut _file = File::create(path).expect("Failed to create file");

        match m.save_to_file(path, false) {
            Ok(_) => {
                std::fs::remove_file(path).expect("Failed to delete test file");
                panic!(
                    "Successfully saved to existing file: {} despite overwrite being false",
                    path
                );
            }
            Err(error) => {
                std::fs::remove_file(path).expect("Failed to delete test file");
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_save_to_existing_file_path_if_overwrite_enabled() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W'],
            vec!['F', ' ', 'W']
        ];
        let m = Maze::from_vec(grid);
        let path = "./maze_3.json";
        let mut _file = File::create(path).expect("Failed to create file");

        match m.save_to_file(path, true) {
            Ok(_) => {
                std::fs::remove_file(path).expect("Failed to delete test file");
            }
            Err(error) => {
                std::fs::remove_file(path).expect("Failed to delete test file");
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_load_from_valid_file_path() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W'], 
            vec!['F', ' ', 'W']
        ];
        let m1 = Maze::from_vec(grid);
        let path = "./maze_4.json";
        match m1.save_to_file(path, true) {
            Ok(_) => {}
            Err(error) => panic!("Failed to save to file: {}", error),
        }
        let mut m2 = Maze::new(Definition::new(0, 0));
        match m2.load_from_file(path) {
            Ok(_) => {
                assert_eq!(m2.definition.row_count(), m1.definition.row_count());
                assert_eq!(m2.definition.col_count(), m1.definition.col_count());
            }
            Err(error) => panic!("Failed to load from: {} - {}", path, error),
        }
        std::fs::remove_file(path).expect("Failed to delete test file");
    }

    #[test]
    fn cannot_load_from_invalid_file_path() {
        let path = "./maze_does_not_exist.json";
        let mut m = Maze::new(Definition::new(0, 0));
        match m.load_from_file(path) {
            Ok(_) => panic!("File should not exist"),
            Err(error) => assert_io_err_not_found(error),
        }
    }

    #[test]
    fn cannot_load_file_with_invalid_content_eof() {
        run_load_file_test_with_invalid_content(
            "./maze_file_with_invalid_content_eof.json",
            "{",
            ExpectedSerdeErrorKind::UnexpectedEof,
        );
    }

    #[test]
    fn cannot_load_file_with_invalid_content_syntax_1() {
        run_load_file_test_with_invalid_content(
            "./maze_file_with_invalid_content_syntax_1.json",
            "{x",
            ExpectedSerdeErrorKind::Syntax,
        );
    }

    #[test]
    fn cannot_load_file_with_invalid_content_syntax_2() {
        run_load_file_test_with_invalid_content(
            "./maze_file_with_invalid_content_syntax_2.json",
            r#"{"x"}"#,
            ExpectedSerdeErrorKind::Syntax,
        );
    }

    #[test]
    fn cannot_load_file_with_invalid_content_syntax_3() {
        run_load_file_test_with_invalid_content(
            "./maze_file_with_invalid_content_syntax_3.json",
            r#"{"x":}"#,
            ExpectedSerdeErrorKind::Syntax,
        );
    }

    #[test]
    fn cannot_load_file_with_invalid_content_syntax_4() {
        run_load_file_test_with_invalid_content(
            "./maze_file_with_invalid_content_syntax_4.json",
            "}",
            ExpectedSerdeErrorKind::Syntax,
        );
    }

    #[test]
    fn cannot_load_file_with_invalid_content_syntax_5() {
        run_load_file_test_with_invalid_content(
            "./maze_file_with_invalid_content_syntax_5.json",
            "{{}",
            ExpectedSerdeErrorKind::Syntax,
        );
    }

    #[test]
    fn cannot_load_file_with_invalid_content_data_1() {
        run_load_file_test_with_invalid_content(
            "./maze_file_with_invalid_content_data_1.json",
            r#"{"definition1":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#,
            ExpectedSerdeErrorKind::Data,
        );
    }

    #[test]
    fn cannot_load_file_with_invalid_content_data_2() {
        run_load_file_test_with_invalid_content(
            "./maze_file_with_invalid_content_data_2.json",
            r#"{"definition":{"grid2":[[" ","W"," "],[" "," ","W"]]}}"#,
            ExpectedSerdeErrorKind::Data,
        );
    }

    #[test]
    fn cannot_load_file_with_invalid_content_data_3() {
        run_load_file_test_with_invalid_content(
            "./maze_file_with_invalid_content_data_3.json",
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
        let m = Maze::new(Definition::from_vec(grid));
        let result = m.solve();
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
        let m = Maze::new(Definition::from_vec(grid));
        let result = m.solve();
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
        let m = Maze::new(Definition::from_vec(grid));
        let result = m.solve();
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
        let m = Maze::new(Definition::from_vec(grid));
        let result = m.solve();
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
        let m = Maze::new(Definition::from_vec(grid));
        let result = m.solve();
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
        let m = Maze::new(Definition::from_vec(grid));
        let result = m.solve();
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
        let m = Maze::new(Definition::from_vec(grid));
        let result = m.solve();
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
        let m = Maze::new(Definition::from_vec(grid));
        let result = m.solve();
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
        let m = Maze::new(Definition::from_vec(grid));
        let result = m.solve();
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
            Err(error) => panic_unexpected_solve_error(error),
        }
    }

    // Helper functions and definitions
    fn delete_test_file(path: &str) {
        std::fs::remove_file(path)
            .unwrap_or_else(|_| panic!("Failed to delete test file: {}", path));
    }

    enum ExpectedSerdeErrorKind {
        Data,
        Syntax,
        UnexpectedEof,
    }

    fn run_load_file_test_with_invalid_content(
        file_path: &str,
        content: &str,
        expected_error_kind: ExpectedSerdeErrorKind,
    ) {
        let mut file = File::create(file_path).expect("Failed to create test file");
        match file.write_all(content.as_bytes()) {
            Ok(_) => {
                let mut m = Maze::new(Definition::new(0, 0));
                match m.load_from_file(file_path) {
                    Ok(_) => {
                        delete_test_file(file_path);
                        panic!("Unexpectedly loaded file despite having invalid content");
                    }
                    Err(error) => {
                        delete_test_file(file_path);
                        match error {
                            MazeError::SerdeJson(ref serdejson_error) => {
                                match expected_error_kind {
                                    ExpectedSerdeErrorKind::Data => {
                                        if !serdejson_error.is_data() {
                                            panic!("Serde data error expected (got SerdeJson error: {})", serdejson_error);
                                        }
                                    }
                                    ExpectedSerdeErrorKind::Syntax => {
                                        if !serdejson_error.is_syntax() {
                                            panic!("Serde syntax error expected (got SerdeJson error: {})", serdejson_error);
                                        }
                                    }
                                    ExpectedSerdeErrorKind::UnexpectedEof => {
                                        if !serdejson_error.is_eof() {
                                            panic!("Serde unexpected EOF error expected (got SerdeJson error: {})", serdejson_error);
                                        }
                                    }
                                }
                            }
                            _ => panic!("Unxpected error encountered (got error: {})", error),
                        }
                    }
                }
            }
            Err(error) => panic!("Failed to create invalid maze file: {}", error),
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

    fn assert_io_err_not_found(error: MazeError) {
        match error {
            MazeError::Io(io_error) => match io_error.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => panic!("io::ErrorKind::NotFound error expected (got: {})", io_error),
            },
            _ => panic!("io::ErrorKind::NotFound error expected (got: {})", error),
        }
    }

    fn assert_error_msg_eq(err: MazeError, msg: &str) {
        assert_eq!(format!("{}", err), msg);
    }
}
