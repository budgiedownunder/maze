use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::Error;
use crate::MazeDefinition;

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
/// Represents a maze
pub struct Maze {
    pub id: String,
    pub name: String,
    /// MazeDefinition, containing the layout of the maze
    pub definition: MazeDefinition,
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
    /// * `definition` - Maze definition
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
    /// use data_model::MazeDefinition;
    /// use data_model::Maze;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let def = MazeDefinition::from_vec(grid);
    /// let maze = Maze::new(def);
    /// assert_eq!(maze.definition.row_count(), 2);
    /// assert_eq!(maze.definition.col_count(), 3);
    pub fn new(definition: MazeDefinition) -> Maze {
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
    /// use data_model::Maze;
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
    /// use data_model::Maze;
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
            definition: MazeDefinition::from_vec(grid),
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
    /// use data_model::Maze;
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
    pub fn to_json(&self) -> Result<String, Error> {
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
    /// use data_model::MazeDefinition;
    /// use data_model::Maze;
    /// let mut maze = Maze::new(MazeDefinition::new(0, 0));
    /// let json = r#"{"id":"maze_id","name":"maze_name", "definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#;
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
    pub fn from_json(&mut self, json: &str) -> Result<(), Error> {
        let temp: Maze = serde_json::from_str(json)?;
        *self = temp;
        Ok(())
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
        let _d = MazeDefinition::from_vec(grid);
    }

    #[test]
    fn can_create_new_from_definition() {
        let maze = Maze::new(MazeDefinition::new(2, 3));
        assert_eq!(maze.definition.row_count(), 2);
        assert_eq!(maze.definition.col_count(), 3);
    }

    #[test]
    fn can_reset_to_empty() {
        let mut maze = Maze::new(MazeDefinition::new(2, 3));
        assert_eq!(maze.definition.row_count(), 2);
        assert_eq!(maze.definition.col_count(), 3);
        assert!(!maze.definition.is_empty());
        assert!(maze.reset().definition.is_empty())
    }

    #[test]
    fn can_serialize_empty() {
        let maze = Maze::new(MazeDefinition::new(0, 0));
        let s = maze.to_json().expect("Failed to serialize");
        assert_eq!(s, r#"{"id":"","name":"","definition":{"grid":[]}}"#);
    }

    #[test]
    fn can_serialize_non_empty() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'W', ' '],
            vec![' ', ' ', 'W']
        ];
        let maze = Maze::new(MazeDefinition::from_vec(grid));
        let s = maze.to_json().expect("Failed to serialize");
        assert_eq!(
            s,
            r#"{"id":"","name":"","definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#
        );
    }

    #[test]
    fn can_deserialize_empty() {
        let mut maze = Maze::new(MazeDefinition::new(10, 10));
        let s = r#"{"id":"maze_id", "name":"maze_name","definition":{"grid":[]}}"#;
        maze.from_json(s).expect("Failed to deserialize");
        assert!(maze.definition.is_empty());
    }

    #[test]
    fn can_deserialize_non_empty() {
        let mut maze = Maze::new(MazeDefinition::new(10, 10));
        let s = r#"{"id":"maze_id", "name":"maze_name","definition":{"grid":[[" ","W"," "],[" "," ","W"]]}}"#;
        maze.from_json(s).expect("Failed to deserialize");
        assert_eq!(maze.definition.row_count(), 2);
        assert_eq!(maze.definition.col_count(), 3);
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
        let mut maze = Maze::new(MazeDefinition::new(0, 0));
        match maze.from_json(content) {
            Ok(_) => {
                panic!("Unexpectedly loaded json despite having invalid content");
            }
            Err(error) => match error {
                Error::Serialization(ref serdejson_error) => match expected_error_kind {
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
}
