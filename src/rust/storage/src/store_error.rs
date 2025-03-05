use maze::MazeError;
use std::io;

/// Represents a store error
#[derive(Debug)]
pub enum StoreError {
    UserEmailExists(),
    //UserEmailMissing(), TO DO - CHANGE TO USER EMAIL INVALID 
    UserIdExists(String),
    UserIdMissing(),
    UserIdNotFound(String),
    UserNameExists(),
    UserNameMissing(),
    UserNotFound(),
    UserPasswordMissing(),
    MazeError(MazeError),
    MazeIdMissing(),
    MazeIdNotFound(String),
    MazeIdExists(String),
    MazeNameMissing(),
    MazeNameNotFound(String),
    MazeNameAlreadyExists(String),
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
    Other(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StoreError::UserEmailExists() => write!(f, "The email is already taken"),
           // StoreError::UserEmailMissing() => write!(f, "No email provided for the user"),
            StoreError::UserIdExists(id) => write!(f, "A user with id '{}' already exists", id),
            StoreError::UserIdMissing() => write!(f, "No id provided for the user"),
            StoreError::UserIdNotFound(id) => write!(f, "A user with id '{}' was not found", id),
            StoreError::UserNameExists() => write!(f, "The username is already taken"),
            StoreError::UserNameMissing() => write!(f, "No username provided for the user"),
            StoreError::UserNotFound() => write!(f, "User not found"),
            StoreError::UserPasswordMissing() => write!(f, "No password provided for the user"),
            StoreError::MazeError(e) => write!(f, "Maze error: {}", e),
            StoreError::MazeIdMissing() => write!(f, "No id provided for the maze"),
            StoreError::MazeIdNotFound(id) => write!(f, "A maze with id '{}' was not found", id),
            StoreError::MazeIdExists(id) => write!(f, "A maze with id '{}' already exists", id),
            StoreError::MazeNameMissing() => write!(f, "No name provided for the maze"),
            StoreError::MazeNameNotFound(name) => write!(f, "A maze with the name '{}' was not found", name),
            StoreError::MazeNameAlreadyExists(name) => {
                write!(f, "A maze with the name '{}' already exists", name)
            }
            StoreError::Io(e) => write!(f, "I/O error: {}", e),
            StoreError::SerdeJson(ref error) => write!(f, "{}", error),
            StoreError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(err: std::io::Error) -> StoreError {
        StoreError::Io(err)
    }
}

impl From<MazeError> for StoreError {
    fn from(err: MazeError) -> StoreError {
        StoreError::MazeError(err)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(error: serde_json::Error) -> Self {
        StoreError::SerdeJson(error)
    }
}

impl From<StoreError> for io::Error {
    fn from(err: StoreError) -> Self {
        io::Error::new(io::ErrorKind::Other, err.to_string())
    }
}