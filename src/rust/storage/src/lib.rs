// Re-export modules
mod file_store;
mod store;
mod store_error;

// Re-export traits and structs
pub use store::MazeItem;
pub use store::Store;
pub use store_error::StoreError;

pub enum StoreType {
    File,
}

pub fn get_store(store_type: StoreType) -> Result<Box<dyn Store>, StoreError> {
    match store_type {
        StoreType::File => Ok(Box::new(file_store::FileStore::new())),
    }
}
