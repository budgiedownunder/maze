use std::{error::Error, io};

#[derive(Debug)]
/// Represents an error
pub enum MazeError {
    Maze(String),
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
}

impl MazeError {
    pub fn new(message: String) -> Self {
        MazeError::Maze(message)
    }
}

impl From<serde_json::Error> for MazeError {
    fn from(error: serde_json::Error) -> Self {
        MazeError::SerdeJson(error)
    }
}

impl From<io::Error> for MazeError {
    fn from(error: io::Error) -> Self {
        MazeError::Io(error)
    }
}

impl std::fmt::Display for MazeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MazeError::Maze(ref message) => write!(f, "{}", message),
            MazeError::Io(ref error) => write!(f, "{}", error),
            MazeError::SerdeJson(ref error) => write!(f, "{}", error),
        }
    }
}

impl Error for MazeError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_maze_error() {
        let msg = "This is a custom maze error";
        let err = MazeError::new(msg.to_string());
        assert_eq!(format!("{}", err), msg);
    }
}
