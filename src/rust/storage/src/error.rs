use data_model::Error as DataModelError;
use maze::Error as MazeError;
use std::{error::Error as StdError, io};

/// Represents a store error
#[derive(Debug)]
pub enum Error {
    UserEmailExists(),
    UserEmailInvalid(),
    UserEmailMissing(),
    UserEmailNotFound(String),
    UserEmailIsPrimary(),
    UserEmailIsLast(),
    UserEmailNotVerified(),
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
    DataModelError(DataModelError),
    Io(std::io::Error),
    SerdeJson(serde_json::Error),
    Other(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::UserEmailExists() => write!(f, "The email is already taken"),
            Error::UserEmailInvalid() => write!(f, "The email address is invalid"),
            Error::UserEmailMissing() => write!(f, "No email address provided for the user"),
            Error::UserEmailNotFound(email) => write!(f, "The email '{email}' is not registered for this user"),
            Error::UserEmailIsPrimary() => write!(f, "The primary email cannot be removed; promote another email first"),
            Error::UserEmailIsLast() => write!(f, "Cannot remove the user's only email address"),
            Error::UserEmailNotVerified() => write!(f, "An unverified email cannot be promoted to primary"),
            Error::UserIdExists(id) => write!(f, "A user with id '{id}' already exists"),
            Error::UserIdMissing() => write!(f, "No id provided for the user"),
            Error::UserIdNotFound(id) => write!(f, "A user with id '{id}' was not found"),
            Error::UserNameExists() => write!(f, "The username is already taken"),
            Error::UserNameMissing() => write!(f, "No username provided for the user"),
            Error::UserNotFound() => write!(f, "User not found"),
            Error::UserPasswordMissing() => write!(f, "No password provided for the user"),
            Error::MazeError(e) => write!(f, "Maze error: {e}"),
            Error::MazeIdMissing() => write!(f, "No id provided for the maze"),
            Error::MazeIdNotFound(id) => write!(f, "A maze with id '{id}' was not found"),
            Error::MazeIdExists(id) => write!(f, "A maze with id '{id}' already exists"),
            Error::MazeNameMissing() => write!(f, "No name provided for the maze"),
            Error::MazeNameNotFound(name) => write!(f, "A maze with the name '{name}' was not found"),
            Error::MazeNameAlreadyExists(name) => {
                write!(f, "A maze with the name '{name}' already exists")
            }
            Error::DataModelError(e) => write!(f, "Data model error: {e}"),
            Error::Io(e) => write!(f, "I/O error: {e}"),
            Error::SerdeJson(error) => write!(f, "{error}"),
            Error::Other(msg) => write!(f, "Error: {msg}"),
        }
    }
}

impl StdError for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<MazeError> for Error {
    fn from(err: MazeError) -> Error {
        Error::MazeError(err)
    }
}

impl From<DataModelError> for Error {
    fn from(err: DataModelError) -> Error {
        Error::DataModelError(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::SerdeJson(error)
    }
}

impl From<Error> for io::Error {
    fn from(err: Error) -> Self {
        io::Error::other(err.to_string())
    }
}