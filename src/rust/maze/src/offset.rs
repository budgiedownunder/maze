use std::fmt;

#[allow(dead_code)]
#[derive(Clone, Debug)] // Derive automatically as all fields implement the Clone trait
pub struct Offset {
    pub row: i32,
    pub col: i32,
}

// Implement the Display trait for the Point struct
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
