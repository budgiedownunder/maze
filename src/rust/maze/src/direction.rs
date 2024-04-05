/// Represents a direction relative to a location
/// # Variants
/// - `Up`: Up 
/// - `Down`: Down
/// - `Left` - Left
/// - `Right` - Right
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl Direction {
    /// Returns the unicode character associated with the given direction instance
    /// # Returns
    ///
    /// Unicode character
    pub fn unicode_char(&self) -> char {
        match self {
            Direction::Up => '\u{2191}',
            Direction::Down => '\u{2193}',
            Direction::Left => '\u{2190}',
            Direction::Right => '\u{2192}',
            Direction::None => '.',
        }
    }
}
