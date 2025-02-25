use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path as StdPath, PathBuf};
use uuid::Uuid;

use maze::Maze;
use utils::file::delete_file;

use crate::MazeItem;
use crate::store::MazeStore;
use crate::store::UserStore;
use crate::Store;
use crate::StoreError;
use crate::User;

/// A file store that implements the [`Store`] trait
///
/// Maze objects are stored on disk as files named `<name>.json` (in the working directory), with the `id`
/// of the object assumed to be the file name
pub struct FileStore {}

impl FileStore {
    /// Creates a new file store instance
    ///
    /// # Returns
    ///
    /// A new file store instance if successful
    ///
    /// # Examples
    ///
    /// Try to create a new maze within a file store
    ///
    /// ```
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::FileStore;
    /// use maze::Maze;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_create = Maze::from_vec(grid);
    /// maze_to_create.name = "maze_1".to_string();
    ///
    /// // Create the file store
    /// let mut store = FileStore::new();
    ///
    /// // Create a maze within the file store
    /// match store.create_maze(&mut maze_to_create) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created maze in the file store with id = {}",
    ///             maze_to_create.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create maze => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    pub fn new() -> Self {
        FileStore {}
    }

    fn make_maze_id(&self, name: &str) -> String {
        format!("{}.json", name)
    }

    fn maze_exists(&self, path: &str) -> bool {
        let path = PathBuf::from(path);
        StdPath::new(&path).exists()
    }

    fn save_maze_to_file(
        &self,
        maze: &mut Maze,
        path: &str,
        overwrite: bool,
    ) -> Result<(), StoreError> {
        let os_path = PathBuf::from(path);

        let full_path = if os_path.is_absolute() {
            os_path.clone()
        } else {
            env::current_dir()?.join(&os_path)
        };

        let normalized_path = full_path
            .strip_prefix(r"\\?\")
            .unwrap_or(&full_path)
            .to_path_buf();

        maze.id = normalized_path.to_string_lossy().to_string();

        if !overwrite {
            if StdPath::new(&os_path).exists() {
                return Err(StoreError::IdAlreadyExists(path.to_string()));
            }
        }
        
        let s = maze.to_json()?;
        let mut file = File::create(path)?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}

impl Default for FileStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MazeStore for FileStore {
    /// Creates a new maze within the file store instance
    ///
    /// # Examples
    ///
    /// Try to create a new maze within a file store
    ///
    /// ```
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::FileStore;
    /// use maze::Maze;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_create = Maze::from_vec(grid);
    /// maze_to_create.name = "maze_1".to_string();
    ///
    /// // Create the file store
    /// let mut store = FileStore::new();
    ///
    /// // Create maze within the file store
    /// match store.create_maze(&mut maze_to_create) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created maze in the file store with id = {}",
    ///             maze_to_create.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create maze => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn create_maze(&mut self, maze: &mut Maze) -> Result<(), StoreError> {
        if maze.name.is_empty() {
            return Err(StoreError::NameMissing());
        }
        let file_id = self.make_maze_id(&maze.name);
        if self.maze_exists(&file_id) {
            return Err(StoreError::IdAlreadyExists(file_id.to_string()));
        }
        self.save_maze_to_file(maze, &file_id, false)?;
        Ok(())
    }
    /// Deletes an existing maze from within the file store instance
    ///
    /// # Examples
    ///
    /// Try to delete an existing maze from within a file store
    ///
    /// ```
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::FileStore;
    /// use maze::Maze;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new();
    ///
    /// // Delete maze from within the file store
    /// let id = "maze_1.json".to_string();
    ///
    /// match store.delete_maze(&id) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully delete maze from the file store",
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to delete maze with id {} => {}",
    ///             id,
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn delete_maze(&mut self, id: &str) -> Result<(), StoreError> {
        if id.is_empty() {
            return Err(StoreError::IdMissing());
        }
        if !self.maze_exists(id) {
            return Err(StoreError::IdNotFound(id.to_string()));
        }
        delete_file(id);
        Ok(())
    }
    /// Updates an existing maze within the file store instance
    ///
    /// # Examples
    ///
    /// Try to update an existing maze within a file store with new content
    ///
    /// ```
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::FileStore;
    /// use maze::Maze;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_update = Maze::from_vec(grid);
    /// maze_to_update.name = "maze_1".to_string();
    /// maze_to_update.id = "maze_1".to_string();
    ///
    /// // Create the file store
    /// let mut store = FileStore::new();
    ///
    /// // Update maze within the file store
    /// match store.update_maze(&mut maze_to_update) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully updated maze in the file store with id = {}",
    ///             maze_to_update.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to update maze => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn update_maze(&mut self, maze: &mut Maze) -> Result<(), StoreError> {
        if maze.id.is_empty() {
            return Err(StoreError::IdMissing());
        }
        if !self.maze_exists(&maze.id) {
            return Err(StoreError::IdNotFound(maze.id.to_string()));
        }
        self.save_maze_to_file(maze, &maze.id.clone(), true)?;
        Ok(())
    }
    /// Loads a maze from within the file store instance
    ///
    /// # Returns
    ///
    /// The maze instance if successful
    ///
    /// # Examples
    ///
    /// Try to create and then reload a maze from within a file store and, if successful, print it
    ///
    /// ```
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::FileStore;
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
    /// // Create the file store
    /// let mut store = FileStore::new();
    ///
    /// // Create the maze within the store
    /// if let Err(error) = store.create_maze(&mut maze_to_create) {
    ///     println!(
    ///         "Failed to create maze => {}",
    ///         error
    ///     );
    ///     return;
    /// }
    /// 
    /// // Now reload the maze from the store
    /// match store.get_maze(&maze_to_create.id) {
    ///     Ok(loaded_maze) => {
    ///         println!("Successfully loaded maze:");
    ///         let mut print_target = StdoutLinePrinter::new();
    ///         let empty_path = Path { points: vec![] };
    ///         loaded_maze.print(&mut print_target, empty_path);
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to load maze with id '{}' => {}",
    ///             maze_to_create.id,
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn get_maze(&self, id: &str) -> Result<Maze, StoreError> {
        if !self.maze_exists(id) {
            return Err(StoreError::IdNotFound(id.to_string()));
        }
        let file = File::open(id)?;
        let reader = BufReader::new(file);
        match serde_json::from_reader::<BufReader<File>, Maze>(reader) {
            Ok(mut maze) => {
                maze.id = id.to_string();
                Ok(maze)
            }
            Err(error) => Err(StoreError::from(error)),
        }
    }
    /// Locates a maze item by name from within the file store instance
    ///
    /// # Returns
    ///
    /// The maze item if successful
    ///
    /// # Examples
    ///
    /// Try to find the maze item with name `my_maze` from within a file store and, if successful,
    /// print its details
    ///
    /// ```
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::FileStore;
    ///
    /// // Create the file store
    /// let store = FileStore::new();
    ///
    /// let id = "my_maze".to_string();
    ///
    /// // Attempt to find the maze item
    /// match store.find_maze_by_name(&id) {
    ///     Ok(maze_item) => {
    ///         println!("Successfully found maze item => id = {}, name = {}",
    ///             maze_item.id,
    ///             maze_item.name
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to find maze item with id '{}' => {}",
    ///             id,
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn find_maze_by_name(&self, name: &str) -> Result<MazeItem, StoreError> {
        let file_id = self.make_maze_id(name);
        let path = PathBuf::from(file_id.clone());

        if !name.is_empty() && StdPath::new(&path).exists() {
            return Ok(MazeItem {
                id: file_id,
                name: name.to_string(),
                definition: None
            });
        }
        Err(StoreError::NameNotFound(name.to_string()))
    }
    /// Returns the list of maze items within the file store instance, sorted
    /// alphabetically in ascending order, optionally including the
    /// maze definitions as a JSON string
    ///
    /// # Returns
    ///
    /// The maze items if successful
    ///
    /// # Examples
    ///
    /// Try to load the maze items within a file store and, if successful,
    /// print the number of items found
    ///
    /// ```
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::FileStore;
    ///
    /// // Create the file store
    /// let store = FileStore::new();
    ///
    /// // Attempt to load the maze items along with their definitions
    /// match store.get_maze_items(true) {
    ///     Ok(maze_items) => {
    ///         println!("Successfully loaded {} maze items",
    ///             maze_items.len()
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to load maze items=> {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn get_maze_items(&self, include_definitions: bool) -> Result<Vec<MazeItem>, StoreError> {
        let mut items: Vec<MazeItem> = Vec::new();
        let current_dir = std::env::current_dir()?;

        let mut paths: Vec<_> = fs::read_dir(current_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<_, std::io::Error>>()?;

        paths.sort();

        for path in paths {
            if let Some(path_str) = path.to_str() {
                if let Some(extension) = path.extension() {
                    if extension == "json" {
                        if let Some(name) = path.file_stem() {
                            if let Some(name_str) = name.to_str() {
                                let mut name_use = name_str.to_string();
                                let mut definition:Option<String> = None;
                                match self.get_maze(&path_str) {
                                    Ok(maze_loaded) => {
                                        if include_definitions {
                                            definition = Some(serde_json::to_string(&maze_loaded).expect("Failed to serialize"));
                                        }
                                        if maze_loaded.name != "" {
                                            name_use = maze_loaded.name.to_string();
                                        }
                                    }
                                    Err(_) => {},
                                }    
                    
                                items.push(MazeItem {
                                    id: path_str.to_string(),
                                    name: name_use,
                                    definition: definition,
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(items)
    }
}

impl UserStore for FileStore {
    /// Adds a new user to the store and sets the allocated `id` within the user object
    fn create_user(&mut self, _user: &mut User) -> Result<(), StoreError> {
        Err(StoreError::Other("create_user() not implemented for FileStore".to_string()))
    }
    /// Deletes a user from the store
    fn delete_user(&mut self, _id: Uuid) -> Result<(), StoreError> {
        Err(StoreError::Other("deletee_user() not implemented for FileStore".to_string()))
    }
    /// Updates a user within the store
    fn update_user(&mut self, _user: &mut User) -> Result<(), StoreError> {
        Err(StoreError::Other("update_user() not implemented for FileStore".to_string()))
    }
    /// Loads a user from the store
    fn get_user(&self, _id: Uuid) -> Result<User, StoreError> {
        Err(StoreError::Other("get_user() not implemented for FileStore".to_string()))
    }
    /// Locates a user by their username within the store
    fn find_user_by_name(&self, _name: &str) -> Result<User, StoreError> {
        Err(StoreError::Other("find_user_by_name() not implemented for FileStore".to_string()))
    }
    /// Returns the list of users within the store, sorted
    /// alphabetically by username in ascending order
    fn get_users(&self) -> Result<Vec<User>, StoreError> {
        Err(StoreError::Other("get_users() not implemented for FileStore".to_string()))
    }
    
}    

impl Store for FileStore {}

#[cfg(test)]
mod tests {
    //use crate::file_store::FileStore;
    use super::*;
    use utils::file::delete_files_with_ext;

    #[test]
    fn can_save_maze_to_valid_file_path() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);

        delete_file(&path);

        match store.save_maze_to_file(&mut maze, &path, true) {
            Ok(_) => {
                delete_file(&path);
            }
            Err(error) => panic!("Failed to save to file: {}", error),
        }
    }

    #[test]
    fn cannot_save_maze_to_invalid_file_path() {
        let store = FileStore::new();
        let (_, mut maze) = init_test_maze(&store, "maze", true, true);
        let path = "";

        match store.save_maze_to_file(&mut maze, path, true) {
            Ok(_) => panic!("Successfully saved to file: {} but did not expect to", path),
            Err(error) => assert_io_err_not_found(error),
        }
    }

    #[test]
    #[should_panic(expected = "Item with id './maze.json' already exists")]
    fn cannot_save_maze_to_existing_file_path_if_overwrite_disabled() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.save_maze_to_file(&mut maze, &path, false) {
            Ok(_) => {
                delete_file(&path);
                panic!(
                    "Successfully saved to existing file: {} despite overwrite being false",
                    path
                );
            }
            Err(error) => {
                delete_file(&path);
                panic!("{}", error);
            }
        }
    }
    #[test]
    fn can_save_maze_to_existing_file_path_if_overwrite_enabled() {
        let store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", false, true);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.save_maze_to_file(&mut maze, &path, true) {
            Ok(_) => {
                delete_file(&path);
            }
            Err(error) => {
                delete_file(&path);
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_create_maze_that_does_not_exist() {
        let mut store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", false, true);

        delete_file(&path);

        match store.create_maze(&mut maze) {
            Ok(_) => delete_file(&path),
            Err(error) => panic!("Failed to create maze: {}", error),
        }
    }

    #[test]
    #[should_panic(expected = "No name provided")]
    fn cannot_create_maze_with_empty_name() {
        let mut store = FileStore::new();
        let (_, mut maze) = init_test_maze(&store, "maze", false, false);
        match store.create_maze(&mut maze) {
            Ok(_) => panic!("Successfully saved unnamed maze but did not expect to"),
            Err(error) => panic!("{}", error),
        }
    }

    #[test]
    #[should_panic(expected = "Item with id 'maze.json' already exists")]
    fn cannot_create_maze_that_exists() {
        let mut store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", false, true);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.create_maze(&mut maze) {
            Ok(_) => {
                delete_file(&path);
                panic!(
                    "Successfully created maze when file: {} existed, when should not have",
                    path
                );
            }
            Err(error) => {
                delete_file(&path);
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_update_existing_maze() {
        let mut store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.update_maze(&mut maze) {
            Ok(_) => {
                delete_file(&path);
            }
            Err(error) => {
                delete_file(&path);
                panic!("{}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "Item with id 'maze.json' not found")]
    fn cannot_update_non_existant_maze() {
        let mut store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);

        delete_file(&path);

        match store.update_maze(&mut maze) {
            Ok(_) => {
                delete_file(&path);
                panic!(
                    "Successfully updated maze when file: {} did not exist",
                    path
                );
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "No id provided")]
    fn cannot_update_maze_with_no_id() {
        let mut store = FileStore::new();
        let (_, mut maze) = init_test_maze(&store, "maze", false, true);

        match store.update_maze(&mut maze) {
            Ok(_) => {
                panic!("Successfully updated maze when maze had no id");
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_delete_maze() {
        let mut store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);

        delete_file(&path);

        match store.save_maze_to_file(&mut maze, &path, true) {
            Ok(_) => {
                if store.maze_exists(&maze.id) {
                    match store.delete_maze(&maze.id) {
                        Ok(_) => {
                            if store.maze_exists(&maze.id) {
                                panic!("Maze file {} still exists after maze delete", maze.id);
                            }
                        }
                        Err(error) => panic!("Failed to delete maze: {}", error),
                    }
                } else {
                    panic!("Saved maze file not found prior to maze delete test");
                }
            }
            Err(error) => panic!("Failed to save maze to file: {}", error),
        }
    }

    #[test]
    #[should_panic(expected = "No id provided")]
    fn cannot_delete_maze_with_empty_id() {
        let mut store = FileStore::new();

        match store.delete_maze("") {
            Ok(()) => {
                panic!("delete_maze() suceeded for blank id");
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_get_maze_that_exists() {
        let mut store = FileStore::new();
        let (path, mut maze) = init_test_maze(&store, "maze", true, true);

        delete_file(&path);

        match store.create_maze(&mut maze) {
            Ok(_) => match store.get_maze(&maze.id) {
                Ok(maze_loaded) => {
                    delete_file(&path);
                    if maze_loaded != maze {
                        panic!("Loaded maze content is different to maze content saved");
                    }
                }
                Err(error) => {
                    delete_file(&path);
                    panic!("Failed to load saved maze: {}", error);
                }
            },
            Err(error) => {
                panic!("Failed to create maze: {}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "Item with id 'missing.json' not found")]
    fn cannot_get_maze_that_does_not_exist() {
        let store = FileStore::new();
        let path = "./missing.json";

        delete_file(path);

        match store.get_maze("missing.json") {
            Ok(_) => {
                panic!("Succesfully loaded maze content when file is missing");
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn maze_item_list_should_be_empty() {
        let store = FileStore::new();

        let _ = delete_files_with_ext(".", "json");

        match store.get_maze_items(false) {
            Ok(items) => {
                if !items.is_empty() {
                    panic!("Maze item list is not empty ({} items found)", items.len());
                }
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn maze_item_list_should_not_be_empty() {
        let mut store = FileStore::new();

        let _ = delete_files_with_ext(".", "json");

        let (_, mut maze_1) = init_test_maze(&store, "maze_1", false, true);
        match store.create_maze(&mut maze_1) {
            Ok(_) => {}
            Err(error) => panic!("Failed to create maze {}: {}", maze_1.name, error),
        }

        let (_, mut maze_2) = init_test_maze(&store, "maze_2", false, true);
        match store.create_maze(&mut maze_2) {
            Ok(_) => {}
            Err(error) => panic!("Failed to create maze {}: {}", maze_2.name, error),
        }

        match store.get_maze_items(false) {
            Ok(items) => {
                if items.len() != 2 {
                    panic!("Maze item list does not contain the expected number of items (2 expected, {} found)", items.len());
                }
                check_maze_item(&items, 0, "maze_1", true);
                check_maze_item(&items, 1, "maze_2", true);
            }
            Err(error) => {
                panic!("{}", error);
            }
        }

        match store.get_maze_items(true) {
            Ok(items) => {
                if items.len() != 2 {
                    panic!("Maze item list does not contain the expected number of items (2 expected, {} found)", items.len());
                }
                check_maze_item(&items, 0, "maze_1", false);
                check_maze_item(&items, 1, "maze_2", false);
            }
            Err(error) => {
                panic!("{}", error);
            }
        }

        fn check_maze_item(items: &[MazeItem], idx: usize, expected: &str, no_definition_expected: bool) {
            if items[idx].name != expected {
                panic!(
                    "Item at index {} contains unexpected value (expected = {}, found: {})",
                    idx, expected, items[idx].name
                );
            }
            if items[idx].definition.is_none() != no_definition_expected {
                panic!(
                    "Item at index {} contains an unexpected definition value  (expected_none = {}, is_none = {})",
                    idx,
                    no_definition_expected,
                    items[idx].definition.is_none()
                );
            }
        }
    }

    fn init_test_maze(
        store: &FileStore,
        name: &str,
        set_id: bool,
        set_name: bool,
    ) -> (String, Maze) {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W'],
            vec!['F', ' ', 'W']
        ];
        let mut maze = Maze::from_vec(grid);
        if set_name {
            maze.name = name.to_string();
        }
        if set_id {
            maze.id = store.make_maze_id(name);
        }
        let path = format!("./{}.json", name);
        (path, maze)
    }

    fn assert_io_err_not_found(error: StoreError) {
        match error {
            StoreError::Io(io_error) => match io_error.kind() {
                std::io::ErrorKind::NotFound => {}
                _ => panic!("io::ErrorKind::NotFound error expected (got: {})", io_error),
            },
            _ => panic!("io::ErrorKind::NotFound error expected (got: {})", error),
        }
    }
}
