use maze::MazeError;

/// Represents a store error
#[derive(Debug)]
pub enum StoreError {
    IdMissing(),
    IdNotFound(String),
    IdAlreadyExists(String),
    NameMissing(),
    NameNotFound(String),
    NameAlreadyExists(String),
    Io(std::io::Error),
    MazeError(MazeError),
    SerdeJson(serde_json::Error),
    Other(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StoreError::IdMissing() => write!(f, "No id provided"),
            StoreError::IdNotFound(id) => write!(f, "Item with id '{}' not found", id),
            StoreError::IdAlreadyExists(id) => write!(f, "Item with id '{}' already exists", id),
            StoreError::NameMissing() => write!(f, "No name provided"),
            StoreError::NameNotFound(name) => write!(f, "Item with name '{}' not found", name),
            StoreError::NameAlreadyExists(name) => {
                write!(f, "An item with name '{}' already exists", name)
            }
            StoreError::Io(e) => write!(f, "I/O error: {}", e),
            StoreError::MazeError(e) => write!(f, "Maze error: {}", e),
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
