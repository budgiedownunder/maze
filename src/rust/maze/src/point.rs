use std::fmt;

#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq)] // Derive automatically as all fields implement the Clone trait
pub struct Point {
    pub row: usize, // Zero-based index
    pub col: usize, // Zero-based index
}

// Implement the Display trait for the Point struct
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}, {}]", self.row, self.col)
    }
}

#[test]
fn should_have_expected_display_format() {
    let pt = Point { row: 1, col: 2 };
    let s = format!("{}", pt);
    assert_eq!(s, "[1, 2]");
}
