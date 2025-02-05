// Re-export modules
mod file_store;
pub mod store;
mod store_error;

// Re-export traits and structs
pub use file_store::FileStore;
pub use store::MazeItem;
pub use store::MazeStore;
pub use store::Store;
pub use store::SharedStore;
pub use store_error::StoreError;

/// Represents the supported store types
pub enum StoreType {
    File,
}

/// Creates and returns a store of the given type
///
/// # Returns
///
/// A new store instance if successful
///
/// # Examples
///
/// Try to create and then reload a maze from within a file store and, if successful, print it
///
/// ```
/// use storage::{get_store, StoreType};
/// use maze::StdoutLinePrinter;
/// use maze::Maze;
/// use maze::Path;
///

/// # // Ensure the maze file does not exist, prior to running the doc test   
/// # use utils::file::delete_file;
/// # delete_file("./maze_1.json");

/// let grid: Vec<Vec<char>> = vec![
///    vec!['S', ' ', 'W'],
///    vec![' ', 'F', 'W']
/// ];
/// let mut maze_to_create = Maze::from_vec(grid);
/// maze_to_create.name = "maze_1".to_string();
///
/// // Access the file store
/// match get_store(StoreType::File) {
///     Ok(mut store) => {
///         // Create the maze within the store
///         if let Err(error) = store.create_maze(&mut maze_to_create) {
///             panic!(
///                 "failed to create maze => {}",
///                 error
///             );
///         }
///         // Now reload the maze from the store
///         match store.get_maze(&maze_to_create.id) {
///             Ok(loaded_maze) => {
///                 println!("Successfully loaded maze:");
///                 let mut print_target = StdoutLinePrinter::new();
///                 let empty_path = Path { points: vec![] };
///                 loaded_maze.print(&mut print_target, empty_path);
///             }
///             Err(error) => {
///                 panic!(
///                     "failed to load maze with id '{}' => {}",
///                     maze_to_create.id,
///                     error
///                 );
///             }
///         }
///     }
///     Err(error) => {
///         panic!(
///             "failed to access file store => {}",
///             error
///         );
///     }
/// }
/// ```
pub fn get_store(store_type: StoreType) -> Result<Box<dyn Store>, StoreError> {
    match store_type {
        StoreType::File => Ok(Box::new(file_store::FileStore::new())),
    }
}
