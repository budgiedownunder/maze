use serde::{Deserialize, Serialize};

use crate::CellState;
use crate::Point;
#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
pub struct Definition {
    pub grid: Vec<Vec<char>>,
}

impl Definition {
    // Public interface functions
    pub fn new(rows: usize, cols: usize) -> Self {
        Definition {
            grid: vec![vec![' '; cols]; rows],
        }
    }

    pub fn row_count(&self) -> usize {
        self.grid.len()
    }

    pub fn col_count(&self) -> usize {
        Self::first_row_col_count(&self.grid)
    }

    pub fn from_vec(grid: Vec<Vec<char>>) -> Self {
        Self::validate_grid(&grid);

        Definition { grid: grid }
    }
    pub fn to_state(&self) -> Vec<Vec<CellState>> {
        return self
            .grid
            .iter()
            .map(|inner_vec| {
                inner_vec
                    .iter()
                    .map(|value| match value {
                        'W' => CellState::Blocked,
                        ' ' => CellState::Empty,
                        _ => panic!("grid contains unsupported cell character: {}", value),
                    })
                    .collect::<Vec<CellState>>()
            })
            .collect();
    }

    pub fn is_valid(&self, pt: &Point) -> bool {
        if pt.row >= self.row_count() || pt.col >= self.col_count() {
            return false;
        }
        true
    }
    pub fn display_grid(&self) -> Vec<Vec<char>> {
        return self
            .grid
            .iter()
            .map(|inner_vec| {
                inner_vec
                    .iter()
                    .map(|value| match value {
                        'B' => '\u{2588}',
                        ' ' => '\u{2591}',
                        _ => '-',
                    })
                    .collect::<Vec<char>>()
            })
            .collect();
    }

    // Private helper functions

    fn first_row_col_count(grid: &Vec<Vec<char>>) -> usize {
        grid.get(0).map_or(0, |inner_vec| inner_vec.len())
    }

    fn validate_grid(grid: &Vec<Vec<char>>) {
        let first_row_col_count = Self::first_row_col_count(&grid);
        let same_col_counts = grid
            .iter()
            .all(|inner_vec| inner_vec.len() == first_row_col_count);
        if !same_col_counts {
            panic!("grid vector contains rows with different numbers of columns (expected {} for all rows)", first_row_col_count);
        }
        for (row_idx, row) in grid.iter().enumerate() {
            for (col_idx, &item) in row.iter().enumerate() {
                if item != ' ' && item != 'W' {
                    panic!(
                        "grid vector contains an invalid character '{}' at location [{}, {}]",
                        item, row_idx, col_idx
                    );
                }
            }
        }
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
    fn can_serialize_empty_1() {
        let d = Definition::new(0, 0);
        let s = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(s, "{\"grid\":[]}");
    }

    #[test]
    fn can_serialize_empty_2() {
        let grid: Vec<Vec<char>> = vec![];
        let d = Definition::from_vec(grid);
        let s = serde_json::to_string(&d).expect("Failed to serialize");
        assert_eq!(s, "{\"grid\":[]}");
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
        assert_eq!(s, "{\"grid\":[[\" \",\" \",\" \"],[\" \",\" \",\" \"]]}");
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
        assert_eq!(s, "{\"grid\":[[\" \",\"W\",\" \"],[\" \",\" \",\"W\"]]}");
    }
}
