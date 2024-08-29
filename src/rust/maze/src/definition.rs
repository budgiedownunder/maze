use serde::{de, Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

use crate::CellState;
use crate::MazeError;
use crate::Point;
#[allow(dead_code)]
#[derive(Serialize, Clone)]
/// Represents a maze definition
pub struct Definition {
    // 2-d grid (rows x columns) of characters describing the maze layout, where
    // - `'W'`:  Represents a wall.
    // - `' '`:  Represents an empty cell.
    grid: Vec<Vec<char>>,
}

impl<'de> Deserialize<'de> for Definition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: HashMap<String, Vec<Vec<char>>> = Deserialize::deserialize(deserializer)?;

        for key in map.keys() {
            if key != "grid" {
                return Err(serde::de::Error::unknown_field(key, &["grid"]));
            }
        }

        let grid = match map.get("grid") {
            Some(inner_vecs) => inner_vecs.clone(),
            None => {
                return Err(serde::de::Error::missing_field("grid"));
            }
        };

        for row in &grid {
            for ch in row {
                if *ch != 'W' && *ch != ' ' {
                    return Err(serde::de::Error::invalid_value(
                        serde::de::Unexpected::Char(*ch),
                        &"valid characters are 'W' or ' '",
                    ));
                }
            }
        }

        if let Some(err) = Self::validate_grid(&grid) {
            return Err(de::Error::custom(err.to_string()));
        }

        Ok(Definition { grid })
    }
}

impl Definition {
    // Public interface functions

    /// Creates a maze definition instance with the given number of rows x columns empty cells
    ///
    /// # Arguments
    /// * `row_count` - Number of rows
    /// * `col_count` - Number of columns
    ///
    /// # Returns
    ///
    /// A new maze definition instance
    ///
    /// # Examples
    ///
    /// Create a definition with 3 rows and 4 columns and then verify its dimensions
    /// ```
    /// use maze::Definition;
    /// let d = Definition::new(3, 4);
    /// assert_eq!(d.row_count(), 3);
    /// assert_eq!(d.col_count(), 4);
    /// ```
    pub fn new(row_count: usize, col_count: usize) -> Self {
        Definition {
            grid: Self::alloc_empty_rows(row_count, col_count),
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
    /// Create a definition with 3 rows and 4 columns, verify its dimensions, reset it and
    /// then confirm it is empty
    /// ```
    /// use maze::Definition;
    /// let mut d = Definition::new(3, 4);
    /// assert_eq!(d.row_count(), 3);
    /// assert_eq!(d.col_count(), 4);
    /// assert_eq!(d.reset().is_empty(), true);
    /// ```
    pub fn reset(&mut self) -> &mut Self {
        self.grid = vec![];
        self
    }
    /// Returns the number of rows associated with the definition instance
    ///
    /// # Returns
    ///
    /// Number of rows
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::Definition;
    /// let d = Definition::new(3, 4);
    /// assert_eq!(d.row_count(), 3);
    /// ```
    pub fn row_count(&self) -> usize {
        self.grid.len()
    }
    /// Returns the number of columns associated with the definition instance
    ///
    /// # Returns
    ///
    /// Number of columns
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::Definition;
    /// let d = Definition::new(3, 4);
    /// assert_eq!(d.col_count(), 4);
    /// ```
    pub fn col_count(&self) -> usize {
        Self::first_row_col_count(&self.grid)
    }
    /// Checks whether the definition instance is empty
    ///
    /// # Returns
    ///
    /// Boolean
    ///
    /// # Examples
    ///
    /// ```
    /// use maze::Definition;
    /// let d = Definition::new(3, 4);
    /// assert_eq!(d.is_empty(), false);
    /// ```
    pub fn is_empty(&self) -> bool {
        self.row_count() == 0
    }
    /// Verifies whether the definition instance is empty, returning an error if it is
    ///
    /// # Returns
    ///
    /// This function will return an error if the definition is empty
    ///  
    /// # Examples
    ///
    /// Create an empty maze definition and then verify it
    ///
    /// ```
    /// use maze::Definition;
    /// let d = Definition::new(0, 0);
    /// match d.verify_not_empty() {
    ///     Err(e) => println!("Verification failed: {}", e.to_string()),
    ///     Ok(()) => println!("Definition is not empty"),
    /// }
    /// ```
    pub fn verify_not_empty(&self) -> Result<(), MazeError> {
        if self.is_empty() {
            return Err(MazeError::new("definition is empty".to_string()));
        }
        Ok(())
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
    /// A new definition instance
    ///
    /// # Examples
    ///
    /// Create a 2 row x 3 column definition with a wall in the last column
    ///
    /// ```
    /// use maze::Definition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec![' ', ' ', 'W'],
    ///    vec![' ', ' ', 'W']
    /// ];
    /// let d = Definition::from_vec(grid);
    /// assert_eq!(d.row_count(), 2);
    /// assert_eq!(d.col_count(), 3);
    /// ```
    pub fn from_vec(grid: Vec<Vec<char>>) -> Self {
        if let Some(err) = Self::validate_grid(&grid) {
            panic!("{}", err.to_string());
        }
        Definition { grid }
    }
    /// Converts the definition instance to a vector of row cell states
    ///
    /// # Returns
    ///
    /// A vector of row-column cell states
    ///
    /// # Examples
    ///
    /// Create a maze definition with 3 rows and 4 columns, convert it to a row-column state vector and then confirm that
    /// the number of rows in the state vector is the same as the number of rows in the definition (3).
    ///
    /// ```
    /// use maze::Definition;
    /// let d = Definition::new(3, 4);
    /// let state = d.to_state();
    /// assert_eq!(state.len(), d.row_count());
    /// assert_eq!(state.len(), 3);
    /// ```
    pub fn to_state(&self) -> Vec<Vec<CellState>> {
        return self
            .grid
            .iter()
            .map(|inner_vec| {
                inner_vec
                    .iter()
                    .map(|value| match value {
                        'W' => CellState::Wall,
                        ' ' => CellState::Empty,
                        _ => panic!(
                            "internal error - grid contains unsupported cell character: {}",
                            value
                        ),
                    })
                    .collect::<Vec<CellState>>()
            })
            .collect();
    }
    /// Checks that a point is valid for the definition instance
    ///
    /// # Arguments
    ///
    /// * `pt` - Point to validate
    ///
    /// # Returns
    ///
    /// Boolean
    ///
    /// # Examples
    ///
    /// Create a maze definition with 3 rows and 4 columns and confirm that `[2,1]` is valid, but that `[3,1]` is not
    ///
    /// ```
    /// use maze::Definition;
    /// use maze::Point;
    /// let d = Definition::new(3, 4);
    /// assert_eq!(d.is_valid( &Point {row: 2, col: 1}), true);
    /// assert_eq!(d.is_valid( &Point {row: 3, col: 1}), false);
    /// ```
    pub fn is_valid(&self, pt: &Point) -> bool {
        if pt.row >= self.row_count() || pt.col >= self.col_count() {
            return false;
        }
        true
    }
    /// Converts the definition instance to a vector of display characters
    ///
    /// # Returns
    ///
    /// Vector containing the rows of display characters
    ///
    /// # Examples
    ///
    /// Create a maze definition with 3 rows and 4 columns and print it
    ///
    /// ```
    /// use maze::Definition;
    /// let d = Definition::new(3, 4);
    /// println!("{:?}", d.to_display_chars());
    /// ```
    pub fn to_display_chars(&self) -> Vec<Vec<char>> {
        return self
            .grid
            .iter()
            .map(|inner_vec| {
                inner_vec
                    .iter()
                    .map(|value| match value {
                        'W' => '\u{2588}',
                        ' ' => '\u{2591}',
                        _ => '-',
                    })
                    .collect::<Vec<char>>()
            })
            .collect();
    }
    /// Deletes one or more consecutive columns from the definition instance
    ///
    /// # Arguments
    ///
    /// * `start_col` - Start column index (zero-based)
    /// * `count` - Number of columns to delete
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the definition is empty
    /// - If the target columns are out of range
    ///
    /// # Examples
    ///
    /// Create a maze definition with 2 rows and 4 columns, with a wall at the end of each row, delete the second and third columns and print the result
    ///
    /// ```
    /// use maze::Definition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec![' ', ' ', ' ', 'W'],
    ///    vec![' ', ' ', ' ', 'W']
    /// ];
    /// let mut d = Definition::from_vec(grid);
    /// d.delete_cols(1,2).expect("delete_cols() failed");
    /// println!("{:?}", d.to_display_chars());
    /// ```
    pub fn delete_cols(&mut self, start_col: usize, count: usize) -> Result<(), MazeError> {
        self.verify_not_empty()?;
        if start_col >= self.col_count() {
            return Err(MazeError::new(format!(
                "invalid 'start_col' index ({})",
                start_col
            )));
        }
        for row in &mut self.grid {
            row.drain(start_col..(start_col + count));
        }
        Ok(())
    }
    /// Inserts one or more empty columns into the definition instance
    ///
    /// # Arguments
    ///
    /// * `start_col` - Start column index (zero-based)
    /// * `count` - Number of columns to insert
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the definition is empty
    /// - If the target columns are out of range
    ///  
    /// # Examples
    ///
    /// Create a maze definition with 2 rows and 4 columns, with a wall at the end of each row, insert 2 columns at the start of each row and print the result
    ///
    /// ```
    /// use maze::Definition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec![' ', ' ', ' ', 'W'],
    ///    vec![' ', ' ', ' ', 'W']
    /// ];
    /// let mut d = Definition::from_vec(grid);
    /// d.insert_cols(0,2).expect("insert_cols() failed");
    /// println!("{:?}", d.to_display_chars());
    /// ```
    pub fn insert_cols(&mut self, start_col: usize, count: usize) -> Result<(), MazeError> {
        self.verify_not_empty()?;
        if start_col > self.col_count() {
            return Err(MazeError::new(format!(
                "invalid 'start_col' index ({})",
                start_col
            )));
        }
        for row in &mut self.grid {
            row.splice(start_col..start_col, vec![' '; count]);
        }
        Ok(())
    }
    /// Deletes one or more consecutive rows from the definition instance
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `count` - Number of rows to delete
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the definition is empty
    /// - If the target rows are out of range
    ///
    /// # Examples
    ///
    /// Create a maze definition with 5 rows and 4 columns, delete the first and and second rows and print the result
    ///
    /// ```
    /// use maze::Definition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W'],
    ///    vec![' ', ' ', 'W', 'W'],
    ///    vec!['W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W']
    /// ];
    /// let mut d = Definition::from_vec(grid);
    /// d.delete_rows(1,2).expect("delete_rows() failed");
    /// println!("{:?}", d.to_display_chars());
    /// ```
    pub fn delete_rows(&mut self, start_row: usize, count: usize) -> Result<(), MazeError> {
        self.verify_not_empty()?;
        if start_row >= self.row_count() {
            return Err(MazeError::new(format!(
                "invalid 'start_row' index ({})",
                start_row
            )));
        }
        self.grid.drain(start_row..(start_row + count));
        Ok(())
    }
    /// Inserts one or more empty rows into the definition instance
    ///
    /// # Arguments
    ///
    /// * `start_row` - Start row index (zero-based)
    /// * `count` - Number of rows to insert
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target rows are out of range
    ///
    /// # Examples
    ///
    /// Create a maze definition with 5 rows and 4 columns, insert 2 rows after the fourth row and print the result
    ///
    /// ```
    /// use maze::Definition;
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W'],
    ///    vec![' ', ' ', 'W', 'W'],
    ///    vec!['W', ' ', ' ', 'W'],
    ///    vec![' ', 'W', ' ', 'W']
    /// ];
    /// let mut d = Definition::from_vec(grid);
    /// d.insert_rows(3,2).expect("insert_rows() failed");
    /// println!("{:?}", d.to_display_chars());
    /// ```
    pub fn insert_rows(&mut self, start_row: usize, count: usize) -> Result<(), MazeError> {
        if start_row > self.row_count() {
            return Err(MazeError::new(format!(
                "invalid 'start_row' index ({})",
                start_row
            )));
        }
        if count > 0 {
            let empty_rows = Self::alloc_empty_rows(count, self.col_count());
            self.grid.splice(start_row..start_row, empty_rows);
        }
        Ok(())
    }
    /// Modify the value of each cell in a given region of the definition instance
    /// # Arguments
    ///
    /// * `from` - Starting point of cell region to modify
    /// * `to` - Ending point of cell region to modify
    /// * `value` - Value to set. Must be either `'W'` (wall) or `' '` (empty).
    ///
    /// # Returns
    ///
    /// This function will return an error in the following situations:
    /// - If the target points are out of range
    /// - if the character value is invalid
    ///
    /// # Examples
    ///
    /// Create a maze definition with 5 rows and 4 columns, then set the central region (1,1) to (3, 2) to be a wall and then print it
    ///
    ///
    /// ```
    /// use maze::CellState;
    /// use maze::Definition;
    /// use maze::Point;
    /// let mut d = Definition::new(5, 4);
    /// let from = Point { row: 1, col: 1, };
    /// let to = Point { row: 3, col: 2, };
    /// d.set_value( from, to, 'W').expect("set_value() failed");
    /// println!("{:?}", d.to_display_chars());
    /// ```
    pub fn set_value(&mut self, from: Point, to: Point, value: char) -> Result<(), MazeError> {
        if !self.is_valid(&from) {
            return Err(MazeError::new(format!("invalid 'from' point {}", from)));
        }
        if !self.is_valid(&to) {
            return Err(MazeError::new(format!("invalid 'to' point {}", to)));
        }
        match value {
            'W' | ' ' => {
                let top_row = from.row.min(to.row);
                let bottom_row = from.row.max(to.row);
                let left_col = from.col.min(to.col);
                let right_col = from.col.max(to.col);
                for row_idx in top_row..(bottom_row + 1) {
                    for col_idx in left_col..(right_col + 1) {
                        self.grid[row_idx][col_idx] = value;
                    }
                }
            }
            _ => return Err(MazeError::new(format!("invalid 'value' ('{}')", value))),
        }
        Ok(())
    }

    // Private helper functions

    fn first_row_col_count(grid: &[Vec<char>]) -> usize {
        grid.first().map_or(0, |inner_vec| inner_vec.len())
    }

    fn validate_grid(grid: &[Vec<char>]) -> Option<MazeError> {
        let first_row_col_count = Self::first_row_col_count(grid);
        let same_col_counts = grid
            .iter()
            .all(|inner_vec| inner_vec.len() == first_row_col_count);
        if !same_col_counts {
            let msg = format!("grid vector contains rows with different numbers of columns (expected {} for all rows)", first_row_col_count).clone();
            return Some(MazeError::new(msg));
        }
        for (row_idx, row) in grid.iter().enumerate() {
            for (col_idx, &item) in row.iter().enumerate() {
                if item != ' ' && item != 'W' {
                    let msg = format!(
                        "grid vector contains an invalid character '{}' at location {}",
                        item,
                        Point {
                            row: row_idx,
                            col: col_idx
                        }
                    );
                    return Some(MazeError::new(msg));
                }
            }
        }
        None
    }

    fn alloc_empty_rows(row_count: usize, col_count: usize) -> Vec<Vec<char>> {
        vec![vec![' '; col_count]; row_count]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_empty_from_dimensions() {
        let d = Definition::new(0, 0);
        assert_eq!(d.row_count(), 0);
        assert_eq!(d.col_count(), 0);
    }

    #[test]
    fn can_create_new_from_dimensions() {
        let d = Definition::new(2, 3);
        assert_eq!(d.row_count(), 2);
        assert_eq!(d.col_count(), 3);
    }

    #[test]
    fn can_reset_to_empty() {
        let mut d = Definition::new(2, 3);
        assert_eq!(d.row_count(), 2);
        assert_eq!(d.col_count(), 3);
        assert_eq!(d.is_empty(), false);
        assert_eq!(d.reset().is_empty(), true)
    }

    #[test]
    fn can_create_empty_from_vector() {
        let grid: Vec<Vec<char>> = vec![];
        let d = Definition::from_vec(grid);
        assert_eq!(d.row_count(), 0);
        assert_eq!(d.col_count(), 0);
    }

    #[test]
    fn can_create_new_from_vector() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', ' ']
        ];
        let d = Definition::from_vec(grid);
        assert_eq!(d.row_count(), 2);
        assert_eq!(d.col_count(), 3);
    }

    #[test]
    #[should_panic(expected = "grid vector contains an invalid character 'X' at location [1, 2]")]
    fn cannot_create_new_from_vector_with_invalid_char() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', 'X']
        ];
        let _d = Definition::from_vec(grid);
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
    fn can_confirm_empty() {
        let d = Definition::new(0, 0);
        assert_eq!(d.is_empty(), true);
    }

    #[test]
    fn can_confirm_not_empty() {
        let d = Definition::new(1, 1);
        assert_eq!(d.is_empty(), false);
    }

    #[test]
    #[should_panic(expected = "definition is empty")]
    fn confirm_verify_not_empty_detects_empty() {
        let d = Definition::new(0, 0);
        if let Err(e) = d.verify_not_empty() {
            panic!("{}", e.to_string());
        }
        panic!("verify_not_empty() did not return an error");
    }

    #[test]
    fn confirm_verify_not_empty_ignores_non_empty() {
        let d = Definition::new(1, 1);
        if let Err(e) = d.verify_not_empty() {
            panic!("{}", e.to_string());
        }
    }

    #[test]
    fn can_serialize_empty_1() {
        let d = Definition::new(0, 0);
        let s = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(s, r#"{"grid":[]}"#);
    }

    #[test]
    fn can_serialize_empty_2() {
        let grid: Vec<Vec<char>> = vec![];
        let d = Definition::from_vec(grid);
        let s = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(s, r#"{"grid":[]}"#);
    }

    #[test]
    fn can_serialize_non_empty_1() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' '],
            vec![' ', ' ', ' ']
        ];
        let d = Definition::from_vec(grid);
        let s = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(s, r#"{"grid":[[" "," "," "],[" "," "," "]]}"#);
    }

    #[test]
    fn can_serialize_non_empty_2() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'W', ' '],
            vec![' ', ' ', 'W']
        ];
        let d = Definition::from_vec(grid);
        let s = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(s, r#"{"grid":[[" ","W"," "],[" "," ","W"]]}"#);
    }

    #[test]
    fn can_deserialize_empty() {
        let s = r#"{"grid":[]}"#;
        let d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
        assert_eq!(d.row_count(), 0);
        assert_eq!(d.col_count(), 0);
    }

    #[test]
    fn can_deserialize_non_empty() {
        let s = r#"{"grid":[[" ","W"," "],[" "," ","W"]]}"#;
        let d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
        assert_eq!(d.row_count(), 2);
        assert_eq!(d.col_count(), 3);
        let grid: Vec<Vec<char>> = vec![vec![' ', 'W', ' '], vec![' ', ' ', 'W']];
        assert_eq!(d.grid, grid);
    }

    #[test]
    #[should_panic(expected = "EOF while parsing an object")]
    fn cannot_deserialize_bad_json_format_incomplete_object() {
        let s = "{";
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "expected value")]
    fn cannot_deserialize_bad_json_format_no_open_object() {
        let s = "}";
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "expected value")]
    fn cannot_deserialize_bad_json_format_missing_field_value() {
        let s = r#"{"grid":}"#;
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "EOF while parsing a string")]
    fn cannot_deserialize_bad_json_format_field_name_not_closed() {
        let s = r#"{"grid:}"#;
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "key must be a string")]
    fn cannot_deserialize_bad_json_format_field_name_not_quoted() {
        let s = "{grid:}";
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = r#"invalid type: string \"a\", expected a sequence"#)]
    fn cannot_deserialize_json_with_non_vec_grid_value() {
        let s = r#"{"grid":"a"}"#;
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "missing field `grid`")]
    fn cannot_deserialize_json_missing_grid_field() {
        let s = "{}";
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "unknown field `grid2`")]
    fn cannot_deserialize_json_with_invalid_field_name() {
        let s = r#"{"grid2":[[" ","W"," "],[" "," ","W"]]}"#;
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(
        expected = "invalid value: character `X`, expected valid characters are 'W' or ' '"
    )]
    fn cannot_deserialize_bad_json_invalid_char_1() {
        let s = r#"{"grid":[[" ","X"," "],[" "," ","W"]]}"#;
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = r#"invalid value: string \"XX\", expected a character"#)]
    fn cannot_deserialize_bad_json_invalid_char_2() {
        let s = r#"{"grid":[[" ","XX"," "],[" "," ","W"]]}"#;
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(
        expected = "grid vector contains rows with different numbers of columns (expected 3 for all rows)"
    )]
    fn cannot_deserialize_bad_json_with_different_col_counts() {
        let s = r#"{"grid":[[" "," "," "],[" "," "]]}"#;
        let _d: Definition = serde_json::from_str(&s).expect("Failed to deserialize");
    }

    #[test]
    #[should_panic(expected = "definition is empty")]
    fn cannot_delete_cols_if_empty() {
        let mut d = Definition::new(0, 0);
        d.delete_cols(0, 1).expect("delete_cols() failed");
    }

    #[test]
    fn can_delete_valid_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.delete_cols(1, 2).expect("delete_cols() failed");
        assert_eq!(d.col_count(), 2);
    }

    #[test]
    #[should_panic(expected = "invalid 'start_col' index (4)")]
    fn cannot_delete_invalid_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.delete_cols(4, 2).expect("delete_cols() failed");
    }

    #[test]
    #[should_panic(expected = "definition is empty")]
    fn cannot_insert_cols_if_empty() {
        let mut d = Definition::new(0, 0);
        d.insert_cols(0, 1).expect("insert_cols() failed");
        assert_empty_cols(&d, 0, 1);
    }

    #[test]
    fn can_insert_valid_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', 'W', ' ', 'W'],
            vec![' ', 'W', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.insert_cols(1, 2).expect("insert_cols() failed");
        assert_eq!(d.col_count(), 6);
        assert_empty_cols(&d, 1, 2);
    }

    #[test]
    fn can_insert_no_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.insert_cols(1, 0).expect("insert_cols() failed");
        assert_eq!(d.col_count(), 4);
    }

    #[test]
    #[should_panic(expected = "invalid 'start_col' index (5)")]
    fn cannot_insert_invalid_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.insert_cols(5, 2).expect("insert_cols() failed");
    }

    #[test]
    fn can_append_cols() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.insert_cols(4, 2).expect("insert_cols() failed");
        assert_eq!(d.col_count(), 6);
        assert_empty_cols(&d, 4, 5);
    }

    #[test]
    #[should_panic(expected = "definition is empty")]
    fn cannot_delete_rows_if_empty() {
        let mut d = Definition::new(0, 0);
        d.delete_rows(0, 1).expect("delete_rows() failed");
    }

    #[test]
    fn can_delete_valid_rows_1() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.delete_rows(0, 2).expect("delete_rows() failed");
        assert_eq!(d.row_count(), 1);
        assert_eq!(d.col_count(), 4);
    }

    #[test]
    fn can_delete_valid_rows_2() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.delete_rows(0, 3).expect("delete_rows() failed");
        assert_eq!(d.row_count(), 0);
        assert_eq!(d.col_count(), 0);
    }

    #[test]
    #[should_panic(expected = "invalid 'start_row' index (2)")]
    fn cannot_delete_invalid_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.delete_rows(2, 1).expect("delete_rows() failed");
    }

    #[test]
    fn can_insert_valid_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.insert_rows(1, 3).expect("insert_rows() failed");
        assert_eq!(d.row_count(), 5);
        assert_empty_rows(&d, 1, 3);
    }

    #[test]
    fn can_insert_no_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.insert_rows(1, 0).expect("insert_rows() failed");
        assert_eq!(d.row_count(), 2);
    }

    #[test]
    #[should_panic(expected = "invalid 'start_row' index (3)")]
    fn cannot_insert_invalid_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.insert_rows(3, 2).expect("insert_rows() failed");
    }

    #[test]
    fn can_append_rows() {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![' ', ' ', ' ', 'W'],
            vec![' ', ' ', ' ', 'W']
        ];
        let mut d = Definition::from_vec(grid);
        d.insert_rows(2, 2).expect("insert_rows() failed");
        assert_eq!(d.row_count(), 4);
        assert_empty_rows(&d, 2, 3);
    }

    #[test]
    fn can_set_value_valid_range() {
        let mut d = Definition::new(5, 4);
        let from = Point { row: 1, col: 1 };
        let to = Point { row: 3, col: 2 };
        d.set_value(from.clone(), to.clone(), 'W')
            .expect("set_value() failed");
        assert_cell_value(&d, from.clone(), to.clone(), 'W');
    }

    #[test]
    #[should_panic(expected = "invalid 'from' point [6, 1]")]
    fn cannot_set_value_invalid_from() {
        let mut d = Definition::new(5, 4);
        let from = Point { row: 6, col: 1 };
        let to = Point { row: 2, col: 2 };
        d.set_value(from, to, 'W').expect("set_value() failed");
    }

    #[test]
    #[should_panic(expected = "invalid 'to' point [6, 2]")]
    fn cannot_set_value_invalid_to() {
        let mut d = Definition::new(5, 4);
        let from = Point { row: 1, col: 1 };
        let to = Point { row: 6, col: 2 };
        d.set_value(from, to, 'W').expect("set_value() failed");
    }

    #[test]
    #[should_panic(expected = "invalid 'value' ('X')")]
    fn cannot_set_value_invalid_value() {
        let mut d = Definition::new(5, 4);
        let from = Point { row: 1, col: 1 };
        let to = Point { row: 3, col: 2 };
        d.set_value(from, to, 'X').expect("set_value() failed");
    }

    // Private test helper functions
    fn assert_empty_cols(d: &Definition, start_col: usize, end_col: usize) {
        let row_count = d.row_count();
        for row_idx in 0..row_count {
            for col_idx in start_col..(end_col + 1) {
                assert_eq!(d.grid[row_idx][col_idx], ' ');
            }
        }
    }

    fn assert_empty_rows(d: &Definition, start_row: usize, end_row: usize) {
        let col_count = d.col_count();
        for row_idx in start_row..(end_row + 1) {
            for col_idx in 0..col_count {
                assert_eq!(d.grid[row_idx][col_idx], ' ');
            }
        }
    }

    fn assert_cell_value(d: &Definition, from: Point, to: Point, expected: char) {
        let top_row = from.row.min(to.row);
        let bottom_row = from.row.max(to.row);
        let left_col = from.col.min(to.col);
        let right_col = from.col.max(to.col);
        for row_idx in top_row..(bottom_row + 1) {
            for col_idx in left_col..(right_col + 1) {
                if d.grid[row_idx][col_idx] != expected {
                    panic!(
                        "grid contains unexpected value: '{}' - expected: '{}' (row: {}, col: {})",
                        d.grid[row_idx][col_idx], expected, row_idx, col_idx
                    );
                }
            }
        }
    }
}
