use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};
use uuid::Uuid;

use maze::Maze;
use utils::file::{delete_dir, delete_file, dir_exists, file_exists};

use crate::store::Manage;
use crate::store::MazeStore;
use crate::store::UserStore;
use crate::MazeItem;
use crate::Store;
use crate::StoreError;
use crate::User;

/// File store configuration settings
#[derive(Debug, Clone)]
pub struct FileStoreConfig {
    /// The directory under which data is stored (default = "data", under the working directory)
    pub data_dir: String,
}

impl FileStoreConfig {
    pub fn default() -> Self {
        FileStoreConfig {
            data_dir: "data".to_string(),
        }
    }
}

/// A file store that implements the [`Store`] trait
///
/// Maze objects are stored on disk as files named `<name>.json` (in the working directory), with the `id`
/// of the object assumed to be the file name
pub struct FileStore {
    /// Configuration settings
    config: FileStoreConfig,
    /// Full path to the root data directory
    data_dir: String,
    /// Full path to the root users directory
    users_dir: String,
}

// Private trait used for accessing struct fields
trait FieldAccess {
    fn get_string_field(&self, field_name: &str) -> Option<String>;
}

// Private FieldAccess implementation for User  
impl FieldAccess for User {
    fn get_string_field(&self, field_name: &str) -> Option<String> {
        match field_name {
            "username" => Some(self.username.clone()),
            "full_name" => Some(self.full_name.clone()),
            "email" => Some(self.email.clone()),
            "password_hash" => Some(self.password_hash.clone()),
            _ => None,
        }
    }
}        

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
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::{FileStore, FileStoreConfig};
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
    /// let mut store = FileStore::new(&FileStoreConfig::default());
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
    pub fn new(config: &FileStoreConfig) -> Self {
        let mut store = FileStore {
            config: config.clone(),
            data_dir: "".to_string(),
            users_dir: "".to_string(),
        };

        match store.init() {
            Ok(_) => store,
            Err(error) => panic!("Failed to initialize file store: {}", error),
        }
    }

    // Initializes the file store
    fn init(&mut self) -> Result<(), StoreError> {
        self.data_dir = Self::make_data_dir(&self.config.data_dir)?;
        self.users_dir = self.make_users_dir()?;
        Ok(())
    }

    // Creates the data directory within the file store
    fn make_data_dir(data_dir: &str) -> Result<String, StoreError> {
        let os_path = PathBuf::from(data_dir);

        let path = if os_path.is_absolute() {
            os_path.clone()
        } else {
            env::current_dir()?.join(&os_path)
        };

        let normalized_path = path
            .strip_prefix(r"\\?\")
            .unwrap_or(&path)
            .to_path_buf();

        let dir_path: String = normalized_path
            .to_string_lossy()
            .replace('/', &MAIN_SEPARATOR.to_string());

        match fs::create_dir_all(normalized_path) {
            Ok(_) => Ok(dir_path),
            Err(error) => Err(StoreError::Other(format!(
                "Failed to create FileStore data directory: {} - {}",
                dir_path, error
            ))),
        }
    }

    // Creates a data sub-directory within the file store
    fn make_data_sub_dir(&self, sub_dir: &str) -> Result<String, StoreError> {
        let path = PathBuf::from(self.data_dir.clone()).join(sub_dir);

        let dir_path: String = path
            .to_string_lossy()
            .to_string();

        match fs::create_dir_all(path) {
            Ok(_) => Ok(dir_path),
            Err(error) => Err(StoreError::Other(format!(
                "Failed to create FileStore sub directory {}: - {}",
                dir_path, error
            ))),
        }
    }    

    // Creates the users directory within the file store
    fn make_users_dir(&self) -> Result<String, StoreError> {
        self.make_data_sub_dir("users")
    }    

    // Creates a user directory within the file store
    fn make_user_dir(&self, id:Uuid) -> Result<String, StoreError> {
        let path = PathBuf::from(self.users_dir.clone()).join(id.to_string());
        let dir_path: String = path
            .to_string_lossy()
            .to_string();

        match fs::create_dir_all(path) {
            Ok(_) => Ok(dir_path),
            Err(error) => Err(StoreError::Other(format!(
                "Failed to create user sub directory {}: - {}",
                dir_path, error
            ))),
        }
    }    

    // Returns a new user id for use in the file store
    fn new_user_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    // Returns a new user API key for use in the file store
    fn new_user_api_key(&self) -> Uuid {
        Uuid::new_v4()
    }

    // Returns the directory path for a given user
    fn user_dir_path(&self, id: Uuid) -> String {
        Path::new(&self.users_dir)
            .join(id.to_string())
            .to_string_lossy()
            .to_string()
    }

    // Returns the file path for a given user
    fn user_file_path(&self, id: Uuid) -> String {
        Path::new(&self.user_dir_path(id))
            .join("user.json")
            .to_string_lossy()
            .to_string()
    }

    // Returns whether a given user exists
    fn user_exists(&self, id: Uuid) -> bool {
        file_exists(&self.user_file_path(id))
    }

    // Returns whether a given user directory exists
    fn user_dir_exists(&self, id: Uuid) -> bool {
        dir_exists(&self.user_dir_path(id))
    }

    // Writes the file associated whether a given user
    fn write_user_file(
        &self,
        user: &User,
        overwrite: bool,
    ) -> Result<(), StoreError> {

        if !overwrite {
            if self.user_exists(user.id) {
                return Err(StoreError::UserIdExists(user.id.to_string()));
            }
        }

        if !self.user_dir_exists(user.id) {
            self.make_user_dir(user.id)?;
        }

        let s = user.to_json()?;
        let mut file = File::create(&self.user_file_path(user.id))?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }

    // Validate user content
    fn validate_user(&self, user: &User, ignore_id: Uuid) -> Result<(), StoreError> {
        if user.id == Uuid::nil() {
            return Err(StoreError::UserIdMissing());
        }
        if user.username.is_empty() {
            return Err(StoreError::UserNameMissing());
        }
        if user.email.is_empty() {
            return Err(StoreError::UserEmailMissing());
        }
        if user.password_hash.is_empty() {
            return Err(StoreError::UserPasswordMissing());
        }
        if self.user_name_exists(&user.username, ignore_id) {
            return Err(StoreError::UserNameExists());
        }
        if self.user_email_exists(&user.email, ignore_id) {
            return Err(StoreError::UserEmailExists());
        }
        Ok(())
    }

    // Read a user definition
    fn read_user(&self, id: Uuid) -> Result<User, StoreError> {
        if !self.user_exists(id) {
            return Err(StoreError::UserIdNotFound(id.to_string()));
        }
        let path = self.user_file_path(id);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        match serde_json::from_reader::<BufReader<File>, User>(reader) {
            Ok(user) => Ok(user),
            Err(error) => Err(StoreError::from(error)),
        }
    }   

    // Locate the first user with a given string field value
    fn find_user_by_string_field(&self, field_name: &str, search_value: &str, ignore_id: Uuid) -> Result<User, StoreError> {
        let ids = self.get_user_ids()?;
        for id in ids {
            if id != ignore_id {
                let user = self.get_user(id)?;
                if let Some(user_value) = user.get_string_field(field_name) {
                    if user_value == search_value {
                        return Ok(user);
                    } 
                } 
            }
        }
        Err(StoreError::UserNotFound())
    }

    // Checks whether a given username exists in the file store
    fn user_name_exists(&self, name: &str, ignore_id: Uuid) -> bool {
        self.find_user_by_string_field("username", name, ignore_id).is_ok()
    }    

    // Checks whether a given user email exists in the file store
    fn user_email_exists(&self, email: &str, ignore_id: Uuid) -> bool {
        self.find_user_by_string_field("email", email, ignore_id).is_ok()
    } 

    // Returns the list of user ids associated with the file store
    fn get_user_ids(&self) -> Result<Vec<Uuid>, StoreError> {
        let ids: Vec<Uuid> = fs::read_dir(&self.users_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.path().is_dir() {
                        e.file_name()
                        .to_str()
                        .and_then(|name| Uuid::parse_str(name).ok())
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(ids)
    }

    // Returns the maze id for a given maze name
    fn make_maze_id(&self, name: &str) -> String {
        format!("{}.json", name)
    }

    // Returns the mazes directory
    pub fn get_mazes_dir(&self) -> String {
        self.data_dir.clone()
    }

    // Returns the maze file path for a given maze id
    fn maze_path(&self, id: &str) -> String {
        Path::new(&self.data_dir)
            .join(id)
            .to_string_lossy()
            .to_string()
    }

    // Checks whether a given maze file exists
    fn maze_exists(&self, id: &str) -> bool {
        file_exists(&self.maze_path(id))
    }

    // Wriets a maze file
    fn write_maze_file(
        &self,
        maze: &mut Maze,
        id: &str,
        overwrite: bool,
    ) -> Result<(), StoreError> {
        maze.id = id.to_string();

        if !overwrite {
            if self.maze_exists(id) {
                return Err(StoreError::MazeIdExists(id.to_string()));
            }
        }

        let s = maze.to_json()?;
        let mut file = File::create(self.maze_path(id))?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}

impl Default for FileStore {
    fn default() -> Self {
        Self::new(&FileStoreConfig::default())
    }
}

impl UserStore for FileStore {
    /// Adds a new user to the store and sets the allocated `id` within the user object
    ///
    /// # Examples
    ///
    /// Try to create a new user within a file store
    ///
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::UserStore;
    /// use crate::storage::{Store, User};
    /// use storage::{FileStore, FileStoreConfig};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    /// 
    /// // Create the user definition  
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "jsmith@company.com".to_string(),
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    /// };
    /// 
    /// // Create the user within the file store
    /// match store.create_user(&mut user) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn create_user(&mut self, user: &mut User) -> Result<(), StoreError> {
        user.id = self.new_user_id();
        user.api_key = self.new_user_api_key();
        self.validate_user(user, Uuid::nil())?;
        self.write_user_file(&user, false)?;
        Ok(())
    }
    /// Deletes a user from the store
    ///
    /// # Examples
    ///
    /// Try to create and then delete a user within a file store
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::UserStore;
    /// use crate::storage::{Store, User};
    /// use storage::{FileStore, FileStoreConfig};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    /// 
    /// // Create the user definition  
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "jsmith@company.com".to_string(),
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    /// };
    /// 
    /// // Create the user within the file store
    /// match store.create_user(&mut user) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         match store.delete_user(user.id) {
    ///             Ok(_) => {
    ///                 println!("Successfully deleted user from the file store");
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to delete user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         } 
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn delete_user(&mut self, id: Uuid) -> Result<(), StoreError> {
        if id.is_nil() {
            return Err(StoreError::UserIdMissing());
        }
        if !self.user_dir_exists(id) {
            return Err(StoreError::UserIdNotFound(id.to_string()));
        }
        delete_dir(&self.user_dir_path(id));

        if self.user_dir_exists(id) {
            panic!("User directory {} still exists XXX", id);
        }

        Ok(())        
    }
    /// Updates a user within the store
    ///
    /// # Examples
    ///
    /// Try to create and then update a user within a file store
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::UserStore;
    /// use crate::storage::{Store, User};
    /// use storage::{FileStore, FileStoreConfig};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    /// 
    /// // Create the user definition  
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "jsmith@company.com".to_string(),
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    /// };
    /// 
    /// // Create the user within the file store
    /// match store.create_user(&mut user) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Change the user full name
    ///         user.full_name = "John Henry Smith".to_string();
    ///         match store.update_user(&mut user) {
    ///             Ok(_) => {
    ///                 println!("Successfully update user within the file store");
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to update user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         } 
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn update_user(&mut self, user: &mut User) -> Result<(), StoreError> {
        if user.id == Uuid::nil() {
            return Err(StoreError::UserIdMissing());
        }
        if !self.user_exists(user.id) {
            return Err(StoreError::UserIdNotFound(user.id.to_string()));
        }
        self.validate_user(user, user.id)?;
        self.write_user_file(&user, true)?;
        Ok(())
    }
    /// Loads a user from the store
    ///
    /// # Examples
    ///
    /// Try to create and then load a user from within a file store
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::UserStore;
    /// use crate::storage::{Store, User};
    /// use storage::{FileStore, FileStoreConfig};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    /// 
    /// // Create the user definition  
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "jsmith@company.com".to_string(),
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    /// };
    /// 
    /// // Create the user within the file store
    /// match store.create_user(&mut user) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to load it again and display the results
    ///         match store.get_user(user.id) {
    ///             Ok(user_loaded) => {
    ///                 println!("Successfully loaded user from within the file store => {:?}", user_loaded);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to load user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         } 
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn get_user(&self, id: Uuid) -> Result<User, StoreError> {
        self.read_user(id)
    }
    /// Locates a user by their username within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user from within a file store
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::UserStore;
    /// use crate::storage::{Store, User};
    /// use storage::{FileStore, FileStoreConfig};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    /// 
    /// // Create the user definition  
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "jsmith@company.com".to_string(),
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    /// };
    /// 
    /// // Create the user within the file store
    /// match store.create_user(&mut user) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by username and display the results
    ///         match store.find_user_by_name(&user.username) {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the file store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         } 
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn find_user_by_name(&self, name: &str) -> Result<User, StoreError> {
        self.find_user_by_string_field("username", name, Uuid::nil())
    }
    /// Returns the list of users within the store, sorted
    /// alphabetically by username in ascending order
    ///
    /// # Examples
    ///
    /// Try to create a user within a file store and then load the list of registered users and display their count
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::UserStore;
    /// use crate::storage::{Store, User};
    /// use storage::{FileStore, FileStoreConfig};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    /// 
    /// // Create the user definition  
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "jsmith@company.com".to_string(),
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    /// };
    /// 
    /// // Create the user within the file store
    /// match store.create_user(&mut user) {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to load the user list and display the results
    ///         match store.get_users() {
    ///             Ok(users_found) => {
    ///                 println!("Successfully loaded {} users from within the file store", users_found.len());
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to load users => {}",
    ///                      error
    ///                 );
    ///             }
    ///         } 
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// ```
    fn get_users(&self) -> Result<Vec<User>, StoreError> {
        let ids = self.get_user_ids()?;
        let mut users:Vec<User> = Vec::new();
        for id in ids {
            users.push(self.get_user(id)?);
        }
        users.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(users)
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
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::{FileStore, FileStoreConfig};
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
    /// let mut store = FileStore::new(&FileStoreConfig::default());
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
            return Err(StoreError::MazeNameMissing());
        }
        let id = self.make_maze_id(&maze.name);
        self.write_maze_file(maze, &id, false)?;
        Ok(())
    }
    /// Deletes an existing maze from within the file store instance
    ///
    /// # Examples
    ///
    /// Try to delete an existing maze from within a file store
    ///
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::{FileStore, FileStoreConfig};
    /// use maze::Maze;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
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
            return Err(StoreError::MazeIdMissing());
        }
        if !self.maze_exists(id) {
            return Err(StoreError::MazeIdNotFound(id.to_string()));
        }
        delete_file(&self.maze_path(id));
        Ok(())
    }
    /// Updates an existing maze within the file store instance
    ///
    /// # Examples
    ///
    /// Try to update an existing maze within a file store with new content
    ///
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::{FileStore, FileStoreConfig};
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
    /// let mut store = FileStore::new(&FileStoreConfig::default());
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
            return Err(StoreError::MazeIdMissing());
        }
        if !self.maze_exists(&maze.id) {
            return Err(StoreError::MazeIdNotFound(maze.id.to_string()));
        }
        self.write_maze_file(maze, &maze.id.clone(), true)?;
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
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::{FileStore, FileStoreConfig};
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
    /// let mut store = FileStore::new(&FileStoreConfig::default());
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
            return Err(StoreError::MazeIdNotFound(id.to_string()));
        }
        let path = self.maze_path(id);
        let file = File::open(path)?;
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
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::{FileStore, FileStoreConfig};
    ///
    /// // Create the file store
    /// let store = FileStore::new(&FileStoreConfig::default());
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
        let id = self.make_maze_id(name);
        if !name.is_empty() && self.maze_exists(&id) {
            return Ok(MazeItem {
                id: id,
                name: name.to_string(),
                definition: None,
            });
        }
        Err(StoreError::MazeNameNotFound(name.to_string()))
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
    /// # // Make sure the store is in a suitable state prior to running the doc test   
    /// # use storage::test_setup::setup;
    /// # setup();
    ///
    /// use crate::storage::store::MazeStore;
    /// use crate::storage::Store;
    /// use storage::{FileStore, FileStoreConfig};
    ///
    /// // Create the file store
    /// let store = FileStore::new(&FileStoreConfig::default());
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
        let mazes_dir = self.get_mazes_dir();

        let mut paths: Vec<_> = fs::read_dir(mazes_dir)?
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
                                let mut definition: Option<String> = None;
                                match self.get_maze(&path_str) {
                                    Ok(maze_loaded) => {
                                        if include_definitions {
                                            definition = Some(
                                                serde_json::to_string(&maze_loaded)
                                                    .expect("Failed to serialize"),
                                            );
                                        }
                                        if maze_loaded.name != "" {
                                            name_use = maze_loaded.name.to_string();
                                        }
                                    }
                                    Err(_) => {}
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

impl Manage for FileStore {
    fn empty(&mut self) -> Result<(), StoreError> {
        let root_path = Path::new(&self.data_dir);
        if root_path.is_dir() {
            if let Err(error) = fs::remove_dir_all(root_path) {
                return Err(StoreError::Other(format!(
                    "Failed to delete root data directory: {} - {}",
                    self.data_dir, error
                )));
            }
        }
        if let Err(error) = self.init() {
            return Err(StoreError::Other(format!(
                "Failed to reinitialize FileStore: {}",
                error
            )));
        }
        Ok(())
    }
}

impl Store for FileStore {}

#[cfg(test)]
mod tests {
    use super::*;
    //****************************************************************
    // Utility functions
    //****************************************************************
    // Create a new, empty store
    fn new_store() -> FileStore {
        let mut store = FileStore::new(&FileStoreConfig::default());
        if let Err(error) = store.empty() {
            panic!("new_store() failed to empty content: {}", error);
        }
        store
    }

    // Initialize a User struct
    fn init_test_user(
        store: &FileStore,
        is_admin: bool,
        username: &str,
        full_name: &str,
        email: &str,
        password_hash: &str,
    ) -> User {
        User {
            id: store.new_user_id(),
            is_admin: is_admin,
            username: username.to_string(),
            full_name: full_name.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            api_key: store.new_user_api_key(),
        }
    }

    // Create a user in the file store
    fn create_user(
        store: &mut FileStore,
        is_admin: bool,
        username: &str,
        full_name: &str,
        email: &str,
        password_hash: &str,
    ) -> User {
        let mut user = init_test_user(&store, is_admin, username, full_name, email, password_hash);

        if let Err(error) = store.create_user(&mut user) {
            panic!( "{}", error);
        }
        user
    }    

    // Initialize a Maze struct
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
        let id = store.make_maze_id(name);
        if set_id {
            maze.id = id.clone();
        }
        (id, maze)
    }    
    //****************************************************************
    // User tests
    //****************************************************************
    #[test]
    #[should_panic(expected = "No username provided for the user")]
    fn cannot_create_user_without_name() {
        let mut store = new_store();
        let user = create_user(&mut store, false, "", "", "", "");
        panic!("Successfully created user {:?} but did not expect to", user);
    }

    #[test]
    #[should_panic(expected = "No email provided for the user")]
    fn cannot_create_user_without_email() {
        let mut store = new_store();
        let user = create_user(&mut store, false, "test", "", "", "");
        panic!("Successfully created user {:?} but did not expect to", user);
    }

    #[test]
    #[should_panic(expected = "No password provided for the user")]
    fn cannot_create_user_without_password() {
        let mut store = new_store();
        let user = create_user(&mut store, false, "test", "", "test@company.com", "");
        panic!("Successfully created user {:?} but did not expect to", user);
    }

    #[test]
    fn can_create_user() {
        let mut store = new_store();
        let _ = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
    }

    #[test]
    #[should_panic(expected = "The username is already taken")]
    fn cannot_create_user_with_existing_name() {
        let mut store = new_store();
        let _ = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        let _ = create_user(&mut store, false, "test", "", "test@company.com2", "password_hash2");
    }

    #[test]
    #[should_panic(expected = "The email is already taken")]
    fn cannot_create_user_with_existing_email() {
        let mut store = new_store();
        let _ = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        let _ = create_user(&mut store, false, "test2", "", "test@company.com", "password_hash2");
    }

    #[test]
    fn can_get_user_that_exists() {
        let mut store = new_store();
        let user = create_user(&mut store, false, "test2", "", "test@company.com", "password_hash2");

        match store.get_user(user.id) {
            Ok(user_loaded) => {
                if user_loaded != user {
                    panic!("Loaded user content is different to user content saved");
                }
            }
            Err(error) => {
                panic!("Failed to load saved user: {}", error);
            }
        }
    }

    #[test]
    fn cannot_get_user_that_does_not_exist() {
        let store = new_store();
        let id = store.new_user_id();
        match store.get_user(id) {
            Ok(_) => { 
                panic!("Loaded user content for id '{}' when did not expected to", id);
            }
            Err(error) => {
                let err_msg = error.to_string();
                let err_msg_expected = StoreError::UserIdNotFound(id.to_string()).to_string();
                if err_msg != err_msg_expected {
                    panic!("Unexpected error returned: '{}', expected: '{}'", err_msg, err_msg_expected);
                }
            }
        }
    }

    #[test]
    fn can_delete_user_that_exists() {
        let mut store = new_store();
        let user = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");

        match store.delete_user(user.id) {
            Ok(_) => {
                if store.user_dir_exists(user.id) {
                    panic!("User directory {} still exists", user.id);
                }
            }, 
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "No id provided for the user")]
    fn cannot_delete_user_with_empty_id() {
        let mut store = new_store();

        match store.delete_user(Uuid::nil()) {
            Ok(()) => {
                panic!("delete_user() suceeded for nil id");
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }    

    #[test]
    fn cannot_delete_user_that_does_not_exist() {
        let mut store = new_store();
        let id = store.new_user_id();

        match store.delete_user(id) {
            Ok(()) => {
                panic!("delete_user() suceeded for id {}", id);
            }
            Err(error) => {
                let err_msg = error.to_string();
                let err_msg_expected = StoreError::UserIdNotFound(id.to_string()).to_string();
                if err_msg != err_msg_expected {
                    panic!("Unexpected error returned: '{}', expected: '{}'", err_msg, err_msg_expected);
                }
            }
        }
    } 

    #[test]
    fn can_update_existing_user() {
        let mut store = new_store();
        let mut user = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        if let Err(error) = store.update_user(&mut user) {
            panic!("Failed to update user: '{}'", error.to_string());
        }
    }

    #[test]
    fn cannot_update_user_that_does_not_exist() {
        let mut store = new_store();
        let mut user = init_test_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        user.id = store.new_user_id();
        if let Err(error) = store.update_user(&mut user) {
            let err_msg = error.to_string();
            let err_msg_expected = StoreError::UserIdNotFound(user.id.to_string()).to_string();
            if err_msg != err_msg_expected {
                panic!("Unexpected error returned: '{}', expected: '{}'", err_msg, err_msg_expected);
            }
        }
    }

    #[test]
    #[should_panic(expected = "No id provided for the user")]
    fn cannot_update_existing_user_without_id() {
        let mut store = new_store();
        let mut user = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        user.id = Uuid::nil();
        if let Err(error) = store.update_user(&mut user) {
            panic!("{}'", error.to_string());
        }
    }

    #[test]
    #[should_panic(expected = "No username provided for the user")]
    fn cannot_update_existing_user_without_name() {
        let mut store = new_store();
        let mut user = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        user.username = "".to_string();
        if let Err(error) = store.update_user(&mut user) {
            panic!("{}", error.to_string());
        }
    }

    #[test]
    #[should_panic(expected = "No email provided for the user")]
    fn cannot_update_existing_user_without_email() {
        let mut store = new_store();
        let mut user = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        user.email = "".to_string();
        if let Err(error) = store.update_user(&mut user) {
            panic!("{}", error.to_string());
        }
    }

    #[test]
    #[should_panic(expected = "No password provided for the user")]
    fn cannot_update_existing_user_without_password() {
        let mut store = new_store();
        let mut user = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        user.password_hash = "".to_string();
        if let Err(error) = store.update_user(&mut user) {
            panic!("{}", error.to_string());
        }
    }

    #[test]
    #[should_panic(expected = "The username is already taken")]
    fn cannot_update_user_with_existing_name() {
        let mut store = new_store();
        let user = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        let mut user2 = create_user(&mut store, false, "test2", "", "test2@company.com", "password_hash2");
        user2.username = user.username;
        if let Err(error) = store.update_user(&mut user2) {
            panic!("{}", error.to_string());
        }
    }

    #[test]
    #[should_panic(expected = "The email is already taken")]
    fn cannot_update_user_with_existing_email() {
        let mut store = new_store();
        let user = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        let mut user2 = create_user(&mut store, false, "test2", "", "test2@company.com", "password_hash2");
        user2.email = user.email;
        if let Err(error) = store.update_user(&mut user2) {
            panic!("{}", error.to_string());
        }
    }

    #[test]
    fn can_find_existing_user_by_name() {
        let mut store = new_store();
        let _ = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        if let Err(error) = store.find_user_by_name("test") {
            panic!("Failed to find user by name: {}", error.to_string());
        }
    }

    #[test]
    #[should_panic(expected = "User not found")]
    fn cannot_find_non_existent_user_by_name() {
        let mut store = new_store();
        let _ = create_user(&mut store, false, "test", "", "test@company.com", "password_hash");
        if let Err(error) = store.find_user_by_name("unknown") {
            panic!("{}", error.to_string());
        }
    }

    #[test]
    fn can_get_user_list() {
        let mut store = new_store();
        let _ = create_user(&mut store, false, "test4", "", "test4@company.com", "password_hash");
        let _ = create_user(&mut store, false, "test1", "", "test1@company.com", "password_hash");
        let _ = create_user(&mut store, false, "test2", "", "test2@company.com", "password_hash");
        let _ = create_user(&mut store, false, "test3", "", "test3@company.com", "password_hash");
        match store.get_users() {
            Ok(users) => {
                let count_expected = 4;
                if users.len() != count_expected {
                    panic!("Expected {} users but got {}", count_expected, users.len());
                }
                let expected_names = vec!["test1".to_string(), "test2".to_string(), "test3".to_string(), "test4".to_string()];
                let names: Vec<String> = users.iter().map(|p| p.username.clone()).collect();
                if names != expected_names {
                    panic!("Expected usernames: {:?} but got: {:?}", expected_names, names);
                }
            },
            Err(error) => panic!("{}", error.to_string()),
        }
    }

    #[test]
    fn can_get_empty_user_list_when_there_are_no_users() {
        let store = new_store();
        if let Err(error) = store.get_users() {
            panic!("{}", error.to_string());
        }
    }
    //****************************************************************
    // Maze tests
    //****************************************************************
    #[test]
    fn can_save_maze_to_valid_file_path() {
        let store = new_store();
        let (id, mut maze) = init_test_maze(&store, "maze", true, true);

        match store.write_maze_file(&mut maze, &id, true) {
            Ok(_) => {},
            Err(error) => panic!("Failed to save to file: {}", error),
        }
    }

    #[test]
    #[should_panic(expected = "A maze with id 'maze.json' already exists")]
    fn cannot_save_maze_to_existing_file_path_if_overwrite_disabled() {
        let store = new_store();
        let (id, mut maze) = init_test_maze(&store, "maze", true, true);
        let path = store.maze_path(&id);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.write_maze_file(&mut maze, &id, false) {
            Ok(_) => {
                panic!(
                    "Successfully saved to existing file: {} despite overwrite being false",
                    path
                );
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }
    #[test]
    fn can_save_maze_to_existing_file_path_if_overwrite_enabled() {
        let store = new_store();
        let (id, mut maze) = init_test_maze(&store, "maze", false, true);
        let path = store.maze_path(&id);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.write_maze_file(&mut maze, &id, true) {
            Ok(_) => {}
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_create_maze_that_does_not_exist() {
        let mut store = new_store();
        let (_id, mut maze) = init_test_maze(&store, "maze", false, true);

        match store.create_maze(&mut maze) {
            Ok(_) => {},
            Err(error) => panic!("Failed to create maze: {}", error),
        }
    }

    #[test]
    #[should_panic(expected = "No name provided")]
    fn cannot_create_maze_with_empty_name() {
        let mut store = new_store();
        let (_, mut maze) = init_test_maze(&store, "maze", false, false);

        match store.create_maze(&mut maze) {
            Ok(_) => panic!("Successfully saved unnamed maze but did not expect to"),
            Err(error) => panic!("{}", error),
        }
    }

    #[test]
    #[should_panic(expected = "A maze with id 'maze.json' already exists")]
    fn cannot_create_maze_that_exists() {
        let mut store = new_store();
        let (id, mut maze) = init_test_maze(&store, "maze", false, true);
        let path = store.maze_path(&id);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.create_maze(&mut maze) {
            Ok(_) => {
                panic!(
                    "Successfully created maze when file: {} existed, when should not have",
                    path
                );
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    fn can_update_existing_maze() {
        let mut store = new_store();
        let (id, mut maze) = init_test_maze(&store, "maze", true, true);
        let path = store.maze_path(&id);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.update_maze(&mut maze) {
            Ok(_) => {},
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "A maze with id 'maze.json' was not found")]
    fn cannot_update_non_existent_maze() {
        let mut store = new_store();
        let (id, mut maze) = init_test_maze(&store, "maze", true, true);

        match store.update_maze(&mut maze) {
            Ok(_) => {
                panic!("Successfully updated maze when file: {} did not exist", id);
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "No id provided for the maze")]
    fn cannot_update_maze_with_no_id() {
        let mut store = new_store();
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
        let mut store = new_store();
        let (id, mut maze) = init_test_maze(&store, "maze", true, true);

        match store.write_maze_file(&mut maze, &id, true) {
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
    #[should_panic(expected = "No id provided for the maze")]
    fn cannot_delete_maze_with_empty_id() {
        let mut store = new_store();

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
        let mut store = new_store();
        let (_id, mut maze) = init_test_maze(&store, "maze", true, true);

        match store.create_maze(&mut maze) {
            Ok(_) => match store.get_maze(&maze.id) {
                Ok(maze_loaded) => {
                    if maze_loaded != maze {
                        panic!("Loaded maze content is different to maze content saved");
                    }
                }
                Err(error) => {
                    panic!("Failed to load saved maze: {}", error);
                }
            },
            Err(error) => {
                panic!("Failed to create maze: {}", error);
            }
        }
    }

    #[test]
    #[should_panic(expected = "A maze with id 'missing.json' was not found")]
    fn cannot_get_maze_that_does_not_exist() {
        let store = new_store();
        let id = "missing.json";

        match store.get_maze(id) {
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
        let store = new_store();

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
        let mut store = new_store();

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

        fn check_maze_item(
            items: &[MazeItem],
            idx: usize,
            expected_name: &str,
            no_definition_expected: bool,
        ) {
            if items[idx].name != expected_name {
                panic!(
                    "Item at index {} contains unexpected value (expected = {}, found: {})",
                    idx, expected_name, items[idx].name
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
}
