// Re-export modules
mod file_store;
pub mod store;
mod store_error;

// Re-export traits and structs
pub use file_store::{FileStore, FileStoreConfig};
pub use store::MazeItem;
pub use store::MazeStore;
pub use store::Store;
pub use store::SharedStore;
pub use store::User;
pub use store::UserStore;
pub use store_error::StoreError;

/// Represents the supported store configurations
pub enum StoreConfig {
    File(FileStoreConfig),
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
/// # // Make sure the store is in a suitable state prior to running the doc test   
/// # use storage::test_setup::setup;
/// # setup();
///
/// use storage::{get_store, StoreConfig, FileStoreConfig};
/// use storage::{Store, StoreError, User};
/// use maze::StdoutLinePrinter;
/// use maze::Maze;
/// use maze::Path;
///

/// let grid: Vec<Vec<char>> = vec![
///    vec!['S', ' ', 'W'],
///    vec![' ', 'F', 'W']
/// ];
/// let mut maze_to_create = Maze::from_vec(grid);
/// maze_to_create.name = "maze_1".to_string();
///
/// // Access the file store
/// let file_config = FileStoreConfig::default();
/// match get_store(StoreConfig::File(file_config)) {
///     Ok(mut store) => {
///         // Locate the owner by username
///         let find_user_result: Result<User, StoreError> = store.find_user_by_name("a_username");
///         let owner = match find_user_result {
///             Ok(user) => user,
///             Err(error) => {
///                 println!("Error fetching user: {:?}", error);
///                 return ;
///             }
///         };
/// 
///         // Create the maze within the store
///         if let Err(error) = store.create_maze(&owner, &mut maze_to_create) {
///             panic!(
///                 "failed to create maze => {}",
///                 error
///             );
///         }
///         // Now reload the maze from the store
///         match store.get_maze(&owner, &maze_to_create.id) {
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
pub fn get_store(config: StoreConfig) -> Result<Box<dyn Store>, StoreError> {
    let store = match config  
    {
        StoreConfig::File(file_config) => file_store::FileStore::new(&file_config),
    };

    Ok(Box::new(store))
}

/// Hidden module that provides setup functionality for doc tests.
#[doc(hidden)]
pub mod test_setup {
    use crate::{FileStore, FileStoreConfig, store::Manage};
    /// This function runs before every documentation test
    pub fn setup() {
        // Make sure any existing files and directories are cleared out
        let mut store = FileStore::new(&FileStoreConfig::default());
        if let Err(error) = store.empty() {
            panic!("setup() failed to empty store: {}", error);
        }
    }
}