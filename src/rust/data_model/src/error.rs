use std::error::Error as StdError;

#[derive(Debug)]
/// Represents a data model error
pub enum Error {
    Serialization(serde_json::Error),
    Validation(String),
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serialization(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Serialization(ref error) => write!(f, "{}", error),
            Error::Validation(ref message) => write!(f, "{}", message),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Serialization(err) => Some(err),
            Error::Validation(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_create_new_validation_error() {
        let msg = "This is a validation error";
        let err = Error::Validation(msg.to_string());
        assert_eq!(format!("{}", err), msg);
    }
}