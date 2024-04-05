use std::fmt;

#[allow(dead_code)]
#[derive(Clone, Debug)]
/// Represents an offset between cell points within a maze instance
pub struct Offset {
    /// Row offset
    pub row: i32,
    /// Column offset
    pub col: i32,
}

impl fmt::Display for Offset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.row, self.col)
    }
}

#[test]
fn should_have_expected_display_format() {
    let o = Offset { row: -1, col: 0 };
    let s = format!("{}", o);
    assert_eq!(s, "[-1, 0]");
}
