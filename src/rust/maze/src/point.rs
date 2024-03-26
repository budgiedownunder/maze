use serde::{Deserialize, Serialize};
use std::fmt;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub row: usize, // Zero-based index
    pub col: usize, // Zero-based index
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.row, self.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_expected_display_format() {
        let pt = Point { row: 1, col: 2 };
        let s = format!("{}", pt);
        assert_eq!(s, "[1, 2]");
    }

    #[test]
    fn should_support_serialize() {
        let my_pt = Point { row: 1, col: 2 };
        let s = serde_json::to_string(&my_pt).expect("Failed to serialize");
        assert_eq!(s, "{\"row\":1,\"col\":2}");
    }
}
