use std::fmt;

#[allow(dead_code)]
#[derive(Clone, Debug)]
/// Represents an offset between maze points
pub struct MazePointOffset {
    /// Row offset
    pub row: i32,
    /// Column offset
    pub col: i32,
}

impl fmt::Display for MazePointOffset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.row, self.col)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;    

    #[test]
    fn should_have_expected_display_format() {
        let o = MazePointOffset { row: -1, col: 0 };
        let s = format!("{}", o);
        assert_eq!(s, "[-1, 0]");
    }
}