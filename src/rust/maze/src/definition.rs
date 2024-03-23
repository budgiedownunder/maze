#[allow(dead_code)]
pub struct Definition {
    pub rows: usize,
    pub cols: usize,
    pub grid: Vec<Vec<i32>>,
}

impl Definition {
    pub fn new(rows: usize, cols: usize) -> Self {
        Definition {
            rows,
            cols,
            grid: vec![vec![-1; cols]; rows],
        }
    }
    pub fn from_vec(grid: Vec<Vec<i32>>) -> Self {
        let first_row_cols = grid.get(0).map_or(0, |inner_vec| inner_vec.len());
        let same_col_counts = grid
            .iter()
            .all(|inner_vec| inner_vec.len() == first_row_cols);
        if !same_col_counts {
            panic!("grid vector contains rows with different numbers of columns (expected {} for all rows)", first_row_cols);
        }
        Definition {
            rows: grid.len(),
            cols: first_row_cols,
            grid: grid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_from_dimensions() {
        let d = Definition::new(2, 3);
        assert_eq!(d.rows, 2);
        assert_eq!(d.cols, 3);
    }

    #[test]
    fn can_create_new_from_vector() {
        let grid: Vec<Vec<i32>> = vec![vec![-1, -1, -1], vec![-1, -1, -1]];
        let d = Definition::from_vec(grid);
        assert_eq!(d.rows, 2);
        assert_eq!(d.cols, 3);
    }

    #[test]
    #[should_panic(
        expected = "grid vector contains rows with different numbers of columns (expected 3 for all rows)"
    )]
    fn cannot_create_new_from_vector_with_diff_row_counts() {
        let grid: Vec<Vec<i32>> = vec![vec![-1, -1, -1], vec![-1, -1, -1, -1]];
        let _d = Definition::from_vec(grid);
    }
}
