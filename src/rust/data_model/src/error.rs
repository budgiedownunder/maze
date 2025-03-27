use std::error::Error as StdError;

#[derive(Debug)]
/// Represents a user validation error
pub enum UserValidationError {
    EmailInvalid,
    IdMissing,
    UsernameMissing,
    PasswordMissing
}

#[derive(Debug)]
/// Represents a data model error
pub enum Error {
    MazeValidation(String),
    Serialization(serde_json::Error),
    UserValidation(UserValidationError),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serialization(error)
    }
}

impl std::fmt::Display for UserValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            UserValidationError::EmailInvalid => write!(f, "Invalid email address"),
            UserValidationError::IdMissing => write!(f, "No id provided for the user"),
            UserValidationError::UsernameMissing => write!(f, "No username provided for the user"),
            UserValidationError::PasswordMissing => write!(f, "No password provided for the user"),
        }
    }
}


impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::MazeValidation(ref message) => write!(f, "{}", message),
            Error::Serialization(ref error) => write!(f, "{}", error),
            Error::UserValidation(ref error) => write!(f, "{}", error),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::MazeValidation(_) => None,
            Error::Serialization(err) => Some(err),
            Error::UserValidation(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_maze_validation_error() {
        let msg = "This is a maze validation error";
        let err = Error::MazeValidation(msg.to_string());
        assert_eq!(format!("{}", err), msg);
    }

    #[test]
    fn can_create_new_user_validation_error() {
        let expected = "Invalid email address";
        let err = Error::UserValidation(UserValidationError::EmailInvalid);
        assert_eq!(format!("{}", err), expected);
    }
}