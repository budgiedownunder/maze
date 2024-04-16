use std::error::Error;

#[derive(Debug)]
/// Represents a maze error
pub struct MazeError {
    /// Error message
    pub message: String,
}

impl MazeError {
    pub fn new(message: &str) -> Self {
        MazeError {
            message: message.to_string(),
        }
    }
}

impl std::fmt::Display for MazeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for MazeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_maze_error() {
        let msg = "This is a maze error";
        let e = MazeError::new(msg);
        assert_eq!(e.message, msg);
    }
}