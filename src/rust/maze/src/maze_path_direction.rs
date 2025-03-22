/// Represents a direction relative to a location
/// # Variants
/// - `Up`: Up 
/// - `Down`: Down
/// - `Left` - Left
/// - `Right` - Right
pub enum MazePathDirection {
    Up,
    Down,
    Left,
    Right,
    None,
}

impl MazePathDirection {
    /// Returns the unicode character associated with the given direction instance
    /// # Returns
    ///
    /// Unicode character
    pub fn unicode_char(&self) -> char {
        match self {
            MazePathDirection::Up => '\u{2191}',
            MazePathDirection::Down => '\u{2193}',
            MazePathDirection::Left => '\u{2190}',
            MazePathDirection::Right => '\u{2192}',
            MazePathDirection::None => '.',
        }
    }
}
