use std::io::ErrorKind;
use std::{error::Error, io};

#[derive(Debug)]
/// Represents a maze error
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

fn io_error_kind_to_string(kind: ErrorKind) -> &'static str {
    match kind {
        ErrorKind::NotFound => "File or directory not found",
        ErrorKind::PermissionDenied => "Permission denied",
        ErrorKind::ConnectionRefused => "Connection refused",
        ErrorKind::ConnectionReset => "Connection reset by remote server",
        ErrorKind::ConnectionAborted => "Connection aborted (terminated) by remote server",
        ErrorKind::NotConnected => "Network operation failed because it is not connected yet",
        ErrorKind::AddrInUse => {
            "Socket address could not be bound because the address is already in use elsewhere"
        }
        ErrorKind::AddrNotAvailable => {
            "Nonexistent interface was requested or the requested address was not local"
        }
        ErrorKind::BrokenPipe => "Operation failed because pipe was closed",
        ErrorKind::AlreadyExists => "File already exists",
        ErrorKind::WouldBlock => "Operation would block",
        ErrorKind::InvalidInput => "Invalid input",
        ErrorKind::InvalidData => "Invalid data",
        ErrorKind::TimedOut => "Timeout expired, causing operation to be cancelled",
        ErrorKind::WriteZero => "Write operation could not be fully completed",
        ErrorKind::Interrupted => "Operation interrupted",
        ErrorKind::Unsupported => "Operation is unsupported on this platform",
        ErrorKind::UnexpectedEof => "Unexpected end of file",
        ErrorKind::OutOfMemory => "Insufficient memory",
        ErrorKind::Other => "Custom I/O error",
        _ => "Unknown I/O error",
    }
}

impl std::fmt::Display for MazeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            MazeError::Maze(ref message) => write!(f, "{}", message),
            MazeError::Io(ref error) => write!(f, "{}", io_error_kind_to_string(error.kind())),
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
