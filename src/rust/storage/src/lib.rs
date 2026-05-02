// Re-export modules
mod error;
mod file_store;
mod file_store_migration;
#[cfg(feature = "sql-store")]
mod sql_store;
pub mod store;
pub mod validation;

// Re-export traits and structs
pub use error::Error;
pub use file_store::{FileStore, FileStoreConfig};
#[cfg(feature = "sql-store")]
pub use sql_store::{SqlStore, SqlStoreConfig};
pub use store::Manage;
pub use store::MazeItem;
pub use store::MazeStore;
pub use store::Store;
pub use store::SharedStore;
pub use store::UserStore;

/// Represents the supported store configurations
pub enum StoreConfig {
    File(FileStoreConfig),
    #[cfg(feature = "sql-store")]
    Sql(SqlStoreConfig),
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
/// # tokio_test::block_on(async {
///
/// use data_model::{Maze, User};
/// use maze::{MazePath, MazePrinter};
/// use storage::{FileStoreConfig, get_store, Store,  StoreConfig, Error};
/// use utils::StdoutLinePrinter;
///
/// let grid: Vec<Vec<char>> = vec![
///    vec!['S', ' ', 'W'],
///    vec![' ', 'F', 'W']
/// ];
/// let mut maze_to_create = Maze::from_vec(grid);
/// maze_to_create.name = "maze_1".to_string();
///
/// // Access the file store
/// let temp = tempfile::tempdir().unwrap();
/// let file_config = FileStoreConfig {
///     data_dir: temp.path().to_string_lossy().to_string(),
/// };
/// match get_store(StoreConfig::File(file_config)).await {
///     Ok(mut store) => {
///         // Locate the owner by username
///         let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
///         let owner = match find_user_result {
///             Ok(user) => user,
///             Err(error) => {
///                 println!("Error fetching user: {:?}", error);
///                 return ;
///             }
///         };
///
///         // Create the maze within the store
///         if let Err(error) = store.create_maze(&owner, &mut maze_to_create).await {
///             panic!(
///                 "failed to create maze => {}",
///                 error
///             );
///         }
///         // Now reload the maze from the store
///         match store.get_maze(&owner, &maze_to_create.id).await {
///             Ok(loaded_maze) => {
///                 println!("Successfully loaded maze:");
///                 let mut print_target = StdoutLinePrinter::new();
///                 let empty_path = MazePath { points: vec![] };
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
/// # });
/// ```
pub async fn get_store(config: StoreConfig) -> Result<Box<dyn Store>, Error> {
    match config {
        StoreConfig::File(file_config) => Ok(Box::new(file_store::FileStore::new(&file_config))),
        #[cfg(feature = "sql-store")]
        StoreConfig::Sql(sql_config) => Ok(Box::new(sql_store::SqlStore::new(sql_config).await?)),
    }
}

