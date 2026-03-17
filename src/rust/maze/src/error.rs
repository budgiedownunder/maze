use std::{error::Error as StdError, io};
use utils::error::io_error_kind_to_string;

#[derive(Debug)]
/// Represents a maze error
pub enum Error {
    Io(std::io::Error),
    Solve(String),
    Generate(String),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::Io(ref error) => write!(f, "{}", io_error_kind_to_string(error.kind())),
            Error::Solve(ref message) => write!(f, "{}", message),
            Error::Generate(ref message) => write!(f, "{}", message),
        }
    }
}

impl StdError for Error {}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;    

    #[test]
    fn can_create_new_solve_error() {
        let msg = "This is a maze solve error";
        let err = Error::Solve(msg.to_string());
        assert_eq!(format!("{}", err), msg);
    }
}
