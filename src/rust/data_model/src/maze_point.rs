use serde::{Deserialize, Serialize};
use std::fmt;
use utoipa::ToSchema;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, ToSchema)]
/// Represents a point within a maze
pub struct MazePoint {
    /// Row index (zero-based)
    pub row: usize,
    /// Column index (zero-based)
    pub col: usize,
}

impl fmt::Display for MazePoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.row, self.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_have_expected_display_format() {
        let pt = MazePoint { row: 1, col: 2 };
        let s = format!("{}", pt);
        assert_eq!(s, "[1, 2]");
    }

    #[test]
    fn should_support_serialize() {
        let my_pt = MazePoint { row: 1, col: 2 };
        let s = serde_json::to_string(&my_pt).expect("Failed to serialize");
        assert_eq!(s, r#"{"row":1,"col":2}"#);
    }
}
