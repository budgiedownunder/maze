
#[derive(Debug)]
pub enum StoreError {
    NotFound(String),
    AlreadyExists(String),
    IoError(std::io::Error),
    Other(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            StoreError::NotFound(id_or_name) => write!(f, "Item '{}' not found", id_or_name),
            StoreError::AlreadyExists(name) => write!(f, "Item '{}' already exists", name),
            StoreError::IoError(e) => write!(f, "I/O error: {}", e),
            StoreError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for StoreError {}

impl From<std::io::Error> for StoreError {
    fn from(err: std::io::Error) -> StoreError {
        StoreError::IoError(err)
    }
}