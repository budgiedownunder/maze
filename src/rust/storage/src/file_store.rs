use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};
use async_trait::async_trait;
use unicase::UniCase;
use uuid::Uuid;

use data_model::{Maze, User};
use utils::file::{delete_dir, delete_file, dir_exists, file_exists};

use crate::store::{Manage, MazeStore, UserStore};
use crate::{validation::validate_user_fields, Error, MazeItem, Store};

/// File store configuration settings
#[derive(Debug, Clone)]
pub struct FileStoreConfig {
    /// The directory under which data is stored (default = "data", under the working directory)
    pub data_dir: String,
}

impl FileStoreConfig {
    #[allow(clippy::should_implement_trait)]
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
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
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Create a maze within the file store
    /// match store.create_maze(&owner, &mut maze_to_create).await {
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
    /// # });
    /// ```
    pub fn new(config: &FileStoreConfig) -> Self {
        let mut store = FileStore {
            config: config.clone(),
            data_dir: "".to_string(),
            users_dir: "".to_string(),
        };

        match store.init() {
            Ok(_) => store,
            Err(error) => panic!("Failed to initialize file store: {error}"),
        }
    }

    // Initializes the file store
    fn init(&mut self) -> Result<(), Error> {
        self.data_dir = Self::make_data_dir(&self.config.data_dir)?;
        self.users_dir = self.make_users_dir()?;
        Ok(())
    }

    fn make_dir(dir: &str) -> Result<String, Error> {
        let path = PathBuf::from(dir);
        let normalized_path = path.strip_prefix(r"\\?\").unwrap_or(&path).to_path_buf();

        match fs::create_dir_all(normalized_path) {
            Ok(_) => Ok(dir.to_string()),
            Err(error) => Err(Error::Other(format!(
                "Failed to create directory: {dir} - {error}"
            ))),
        }
    }

    // Creates the data directory within the file store
    fn make_data_dir(data_dir: &str) -> Result<String, Error> {
        let os_path = PathBuf::from(data_dir);

        let path = if os_path.is_absolute() {
            os_path.clone()
        } else {
            env::current_dir()?.join(&os_path)
        };

        let normalized_path = path.strip_prefix(r"\\?\").unwrap_or(&path).to_path_buf();

        let dir_path: String = normalized_path
            .to_string_lossy()
            .replace('/', MAIN_SEPARATOR_STR);

        Self::make_dir(&dir_path)
    }

    // Creates a data sub-directory within the file store
    fn make_data_sub_dir(&self, sub_dir: &str) -> Result<String, Error> {
        let path = PathBuf::from(self.data_dir.clone()).join(sub_dir);
        let dir_path: String = path.to_string_lossy().to_string();
        Self::make_dir(&dir_path)
    }

    // Creates the users directory within the file store
    fn make_users_dir(&self) -> Result<String, Error> {
        self.make_data_sub_dir("users")
    }

    fn get_user_sub_dir_path(&self, id: Uuid, sub_dir: &str) -> String {
        PathBuf::from(self.user_dir_path(id))
            .join(sub_dir)
            .to_string_lossy()
            .to_string()
    }

    fn make_user_sub_dir(&self, id: Uuid, sub_dir: &str) -> Result<String, Error> {
        Self::make_dir(&self.get_user_sub_dir_path(id, sub_dir))
    }

    // Creates a user directory within the file store
    fn make_user_dir(&self, id: Uuid) -> Result<String, Error> {
        Self::make_dir(&self.user_dir_path(id))
    }

    // Returns the directory path for a given user id
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
    fn write_user_file(&self, user: &User, overwrite: bool) -> Result<(), Error> {
        if !overwrite && self.user_exists(user.id) {
            return Err(Error::UserIdExists(user.id.to_string()));
        }

        if !self.user_dir_exists(user.id) {
            self.make_user_dir(user.id)?;
        }

        let s = user.to_json()?;
        let mut file = File::create(self.user_file_path(user.id))?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }

    // Validate user content
    fn validate_user(&self, user: &User, ignore_id: Uuid) -> Result<(), Error> {
        validate_user_fields(user)?;
        if self.user_name_exists(&user.username, ignore_id) {
            return Err(Error::UserNameExists());
        }
        if self.user_email_exists(&user.email, ignore_id) {
            return Err(Error::UserEmailExists());
        }
        Ok(())
    }

    // Read a user definition
    fn read_user(&self, id: Uuid) -> Result<User, Error> {
        if !self.user_exists(id) {
            return Err(Error::UserIdNotFound(id.to_string()));
        }
        let path = self.user_file_path(id);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        match serde_json::from_reader::<BufReader<File>, User>(reader) {
            Ok(user) => Ok(user),
            Err(error) => Err(Error::from(error)),
        }
    }

    // Locate the first user with a given string field value
    fn find_user_by_string_field(
        &self,
        field_name: &str,
        search_value: &str,
        ignore_id: Uuid,
    ) -> Result<User, Error> {
        let ids = self.get_user_ids()?;
        let search = UniCase::new(search_value);
        for id in ids {
            if id != ignore_id {
                if let Some(user) = self.load_user_if_present(id)? {
                    if let Some(user_value) = user.get_string_field(field_name) {
                        if UniCase::new(user_value) == search {
                            return Ok(user);
                        }
                    }
                }
            }
        }
        Err(Error::UserNotFound())
    }

    // Checks whether a given username exists in the file store
    fn user_name_exists(&self, name: &str, ignore_id: Uuid) -> bool {
        self.find_user_by_string_field("username", name, ignore_id)
            .is_ok()
    }

    // Checks whether a given user email exists in the file store
    fn user_email_exists(&self, email: &str, ignore_id: Uuid) -> bool {
        self.find_user_by_string_field("email", email, ignore_id)
            .is_ok()
    }

    // Loads a user by id, returning None (with a warning) if the user directory exists
    // but user.json is missing or unreadable, and Err for all other failures.
    fn load_user_if_present(&self, id: Uuid) -> Result<Option<User>, Error> {
        match self.read_user(id) {
            Ok(user) => Ok(Some(user)),
            Err(Error::UserIdNotFound(_)) => {
                log::warn!("Skipping user directory '{id}': user.json is missing or unreadable");
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }

    // Returns the list of user ids associated with the file store
    fn get_user_ids(&self) -> Result<Vec<Uuid>, Error> {
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
        format!("{name}.json")
    }

    // Returns the mazes directory for a given owner
    pub fn get_mazes_dir(&self, owner: &User) -> String {
        Path::new(&self.user_dir_path(owner.id))
            .join("mazes")
            .to_string_lossy()
            .to_string()
    }

    // Creates the mazes directory within the file store for a given owner
    fn make_user_mazes_dir(&self, owner: &User) -> Result<String, Error> {
        self.make_user_sub_dir(owner.id, "mazes")
    }

    // Returns whether a given mazes directory exists
    fn user_mazes_dir_exists(&self, owner: &User) -> bool {
        dir_exists(&self.get_mazes_dir(owner))
    }

    // Returns the maze file path for a given maze id
    fn maze_path(&self, owner: &User, id: &str) -> String {
        Path::new(&self.get_mazes_dir(owner))
            .join(id)
            .to_string_lossy()
            .to_string()
    }

    // Checks whether a given maze file exists
    fn maze_exists(&self, owner: &User, id: &str) -> bool {
        file_exists(&self.maze_path(owner, id))
    }

    // Returns the actual on-disk filename of any maze whose name matches
    // `name` case-insensitively for `owner`, or None if no such maze
    // exists.
    //
    // Used by `find_maze_by_name` and `create_maze` so that case-insensitive
    // matching is enforced in code rather than via filesystem semantics —
    // NTFS and APFS-default are case-insensitive, ext4 is not, so relying
    // on the filesystem makes behaviour OS-dependent.
    fn find_maze_filename_ci(&self, owner: &User, name: &str) -> Option<String> {
        if name.is_empty() {
            return None;
        }
        let target = UniCase::new(self.make_maze_id(name));
        let mazes_dir = self.get_mazes_dir(owner);
        let entries = std::fs::read_dir(&mazes_dir).ok()?;
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str() {
                if UniCase::new(filename.to_string()) == target {
                    return Some(filename.to_string());
                }
            }
        }
        None
    }

    // Wriets a maze file
    fn write_maze_file(
        &self,
        owner: &User,
        maze: &mut Maze,
        id: &str,
        overwrite: bool,
    ) -> Result<(), Error> {
        maze.id = id.to_string();

        if !self.user_mazes_dir_exists(owner) {
            self.make_user_mazes_dir(owner)?;
        }

        if !overwrite && self.maze_exists(owner, id) {
            return Err(Error::MazeIdExists(id.to_string()));
        }

        let s = maze.to_json()?;
        let mut file = File::create(self.maze_path(owner, id))?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}

impl Default for FileStore {
    fn default() -> Self {
        Self::new(&FileStoreConfig::default())
    }
}

#[async_trait]
impl UserStore for FileStore {
    /// Adds the default admin user to the store if it doesn't already exist, else returns it
    ///
    /// # Examples
    ///
    /// Try to create a new user within a file store
    ///
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test
    /// # use storage::test_setup::setup;
    /// # setup();
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    ///
    /// // Create the default admin user within the file store if needed
    /// match store.init_default_admin_user("admin", "admin@maze.local", "my_password_hash").await {
    ///     Ok(user) => {
    ///         println!(
    ///             "Successfully intiialized default admin user with id {} in the file store",
    ///             user.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to initialized default admin user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn init_default_admin_user(
        &mut self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, Error> {
        match self.find_user_by_name(username).await {
            Ok(user) => Ok(user),
            Err(error) => match error {
                Error::UserNotFound() => {
                    let mut user = User::default();
                    user.username = username.to_string();
                    user.email = email.to_string();
                    user.is_admin = true;
                    user.password_hash = password_hash.to_string();
                    self.create_user(&mut user).await?;
                    Ok(user)
                }
                _ => Err(error),
            },
        }
    }
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
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
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
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
    /// # });
    /// ```
    async fn create_user(&mut self, user: &mut User) -> Result<(), Error> {
        user.id = User::new_id();
        user.api_key = User::new_api_key();
        self.validate_user(user, Uuid::nil())?;
        self.write_user_file(user, false)?;
        self.make_user_mazes_dir(user)?;
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
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
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         match store.delete_user(user.id).await {
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
    /// # });
    /// ```
    async fn delete_user(&mut self, id: Uuid) -> Result<(), Error> {
        if id.is_nil() {
            return Err(Error::UserIdMissing());
        }
        if !self.user_dir_exists(id) {
            return Err(Error::UserIdNotFound(id.to_string()));
        }
        delete_dir(&self.user_dir_path(id));

        if self.user_dir_exists(id) {
            panic!("User directory {id} still exists XXX");
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
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
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Change the user full name
    ///         user.full_name = "John Henry Smith".to_string();
    ///         match store.update_user(&mut user).await {
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
    /// # });
    /// ```
    async fn update_user(&mut self, user: &mut User) -> Result<(), Error> {
        if user.id == Uuid::nil() {
            return Err(Error::UserIdMissing());
        }
        if !self.user_exists(user.id) {
            return Err(Error::UserIdNotFound(user.id.to_string()));
        }
        self.validate_user(user, user.id)?;
        self.write_user_file(user, true)?;
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
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
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to load it again and display the results
    ///         match store.get_user(user.id).await {
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
    /// # });
    /// ```
    async fn get_user(&self, id: Uuid) -> Result<User, Error> {
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
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
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by username and display the results
    ///         match store.find_user_by_name(&user.username).await {
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
    /// # });
    /// ```
    async fn find_user_by_name(&self, name: &str) -> Result<User, Error> {
        self.find_user_by_string_field("username", name, Uuid::nil())
    }
    /// Locates a user by their email address within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user from within a file store by email
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test
    /// # use storage::test_setup::setup;
    /// # setup();
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
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
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by email and display the results
    ///         match store.find_user_by_email(&user.email).await {
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
    /// # });
    /// ```
    async fn find_user_by_email(&self, email: &str) -> Result<User, Error> {
        self.find_user_by_string_field("email", email, Uuid::nil())
    }
    /// Locates a user by their api key within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user by its api key from within a file store
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test
    /// # use storage::test_setup::setup;
    /// # setup();
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
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
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by username and display the results
    ///         match store.find_user_by_api_key(user.api_key).await {
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
    /// # });
    /// ```
    async fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, Error> {
        let ids = self.get_user_ids()?;
        for id in ids {
            if let Some(user) = self.load_user_if_present(id)? {
                if user.api_key == api_key {
                    return Ok(user);
                }
            }
        }
        Err(Error::UserNotFound())
    }
    /// Locates a user by their login id within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user by its login id within a file store
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test
    /// # use storage::test_setup::setup;
    /// # setup();
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserLogin};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    /// 
    /// // Create the login tokens
    /// let login = UserLogin::new(24, Some("123.456.789.012".to_string()), Some("Device info string".to_string()));
    /// let search_login_id = login.id; 
    /// let logins = vec![login];
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
    ///     logins,
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again using the login id and display the results
    ///         match store.find_user_by_login_id(search_login_id).await {
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
    /// # });
    /// ```
    async fn find_user_by_login_id(&self, login_id: Uuid) -> Result<User, Error>{
        let ids = self.get_user_ids()?;
        for id in ids {
            if let Some(user) = self.load_user_if_present(id)? {
                if user.contains_valid_login(login_id) {
                    return Ok(user);
                }
            }
        }
        Err(Error::UserNotFound())
    }
    /// Locates a user by an OAuth identity `(provider, provider_user_id)` pair.
    /// `provider` is matched case-insensitively (canonical providers are stored
    /// lowercase: "google", "github"); `provider_user_id` is matched exactly (it
    /// is an opaque stable id from the identity provider).
    ///
    /// # Examples
    ///
    /// Try to create a user with a linked Google identity and then locate it by
    /// its OAuth identity within a file store
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test
    /// # use storage::test_setup::setup;
    /// # setup();
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{OAuthIdentity, User};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    ///
    /// // Create the user definition with a linked Google identity
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "jsmith@company.com".to_string(),
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![OAuthIdentity::new(
    ///         "google".to_string(),
    ///         "google-sub-jsmith".to_string(),
    ///         Some("jsmith@company.com".to_string()),
    ///     )],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by its OAuth identity and display the results
    ///         match store.find_user_by_oauth_identity("google", "google-sub-jsmith").await {
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
    /// # });
    /// ```
    async fn find_user_by_oauth_identity(&self, provider: &str, provider_user_id: &str) -> Result<User, Error> {
        let ids = self.get_user_ids()?;
        for id in ids {
            if let Some(user) = self.load_user_if_present(id)? {
                if user.oauth_identities.iter().any(|identity| {
                    identity.provider.eq_ignore_ascii_case(provider)
                        && identity.provider_user_id == provider_user_id
                }) {
                    return Ok(user);
                }
            }
        }
        Err(Error::UserNotFound())
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
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
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to load the user list and display the results
    ///         match store.get_users().await {
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
    /// # });
    /// ```
    async fn get_users(&self) -> Result<Vec<User>, Error> {
        let ids = self.get_user_ids()?;
        let mut users: Vec<User> = Vec::new();
        for id in ids {
            if let Some(user) = self.load_user_if_present(id)? {
                users.push(user);
            }
        }
        users.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(users)
    }

    /// Returns the list of admin users within the store
    ///
    /// # Examples
    ///
    /// Try to create an admin user within a file store and then load the list of admin users and display their count
    /// ```
    /// # // Make sure the store is in a suitable state prior to running the doc test
    /// # use storage::test_setup::setup;
    /// # setup();
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    ///
    /// // Create the admin user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: true,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     email: "jsmith@company.com".to_string(),
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    /// };
    ///
    /// // Create the admin user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created admin user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to load the admin user list and display the results
    ///         match store.get_admin_users().await {
    ///             Ok(admins_found) => {
    ///                 println!("Successfully loaded {} admin users from within the file store", admins_found.len());
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to load admin users => {}",
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
    /// # });
    /// ```
    async fn get_admin_users(&self) -> Result<Vec<User>, Error> {
        let ids = self.get_user_ids()?;
        let mut admins: Vec<User> = Vec::new();
        for id in ids {
            if let Some(user) = self.load_user_if_present(id)? {
                if user.is_admin {
                    admins.push(user);
                }
            }
        }
        Ok(admins)
    }
}

#[async_trait]
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
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
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Create maze within the file store
    /// match store.create_maze(&owner, &mut maze_to_create).await {
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
    /// # });
    /// ```
    async fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error> {
        if maze.name.is_empty() {
            return Err(Error::MazeNameMissing());
        }
        // Reject case-insensitive name collision before writing — the
        // `write_maze_file` overwrite check uses `Path::exists`, which
        // is case-insensitive on NTFS/APFS but case-sensitive on ext4.
        // Without this guard, "Treasure" and "TREASURE" can both be
        // created on Linux but only one on Windows.
        if let Some(existing) = self.find_maze_filename_ci(owner, &maze.name) {
            return Err(Error::MazeIdExists(existing));
        }
        let id = self.make_maze_id(&maze.name);
        self.write_maze_file(owner, maze, &id, false)?;
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let mut store = FileStore::new(&FileStoreConfig::default());
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Delete maze from within the file store
    /// let id = "maze_1.json".to_string();
    ///
    /// match store.delete_maze(&owner, &id).await {
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
    /// # });
    /// ```
    async fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), Error> {
        if id.is_empty() {
            return Err(Error::MazeIdMissing());
        }
        if !self.maze_exists(owner, id) {
            return Err(Error::MazeIdNotFound(id.to_string()));
        }
        delete_file(&self.maze_path(owner, id));
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
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
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Update maze within the file store
    /// match store.update_maze(&owner, &mut maze_to_update).await {
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
    /// # });
    /// ```
    async fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error> {
        if maze.id.is_empty() {
            return Err(Error::MazeIdMissing());
        }
        if !self.maze_exists(owner, &maze.id) {
            return Err(Error::MazeIdNotFound(maze.id.to_string()));
        }
        self.write_maze_file(owner, maze, &maze.id.clone(), true)?;
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use maze::{MazePath, MazePrinter};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error,  UserStore};
    /// use utils::StdoutLinePrinter;
    /// use uuid::Uuid;
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
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Create the maze within the store
    /// if let Err(error) = store.create_maze(&owner, &mut maze_to_create).await {
    ///     println!(
    ///         "Failed to create maze => {}",
    ///         error
    ///     );
    ///     return;
    /// }
    ///
    /// // Now reload the maze from the store
    /// match store.get_maze(&owner, &maze_to_create.id).await {
    ///     Ok(loaded_maze) => {
    ///         println!("Successfully loaded maze:");
    ///         let mut print_target = StdoutLinePrinter::new();
    ///         let empty_path = MazePath { points: vec![] };
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
    /// # });
    /// ```
    async fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, Error> {
        if !self.maze_exists(owner, id) {
            return Err(Error::MazeIdNotFound(id.to_string()));
        }
        let path = self.maze_path(owner, id);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        match serde_json::from_reader::<BufReader<File>, Maze>(reader) {
            Ok(mut maze) => {
                maze.id = id.to_string();
                Ok(maze)
            }
            Err(error) => Err(Error::from(error)),
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let store = FileStore::new(&FileStoreConfig::default());
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// let id = "my_maze".to_string();
    ///
    /// // Attempt to find the maze item
    /// match store.find_maze_by_name(&owner, &id).await {
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
    /// # });
    /// ```
    async fn find_maze_by_name(&self, owner: &User, name: &str) -> Result<MazeItem, Error> {
        // Case-insensitive lookup, implemented in code rather than via
        // filesystem semantics — see `find_maze_filename_ci` for rationale.
        match self.find_maze_filename_ci(owner, name) {
            Some(id) => Ok(MazeItem {
                id,
                name: name.to_string(),
                definition: None,
            }),
            None => Err(Error::MazeNameNotFound(name.to_string())),
        }
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
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::User;
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let store = FileStore::new(&FileStoreConfig::default());
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Attempt to load the maze items along with their definitions
    /// match store.get_maze_items(&owner, true).await {
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
    /// # });
    /// ```
    async fn get_maze_items(
        &self,
        owner: &User,
        include_definitions: bool,
    ) -> Result<Vec<MazeItem>, Error> {
        let mut items: Vec<MazeItem> = Vec::new();
        let mazes_dir = self.get_mazes_dir(owner);

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
                                if let Ok(maze_loaded) = self.get_maze(owner, path_str).await {
                                    if include_definitions {
                                        definition = Some(
                                            serde_json::to_string(&maze_loaded)
                                                .expect("Failed to serialize"),
                                        );
                                    }
                                    if !maze_loaded.name.is_empty() {
                                        name_use = maze_loaded.name.to_string();
                                    }
                                }

                                items.push(MazeItem {
                                    id: path_str.to_string(),
                                    name: name_use,
                                    definition,
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

#[async_trait]
impl Manage for FileStore {
    async fn empty(&mut self) -> Result<(), Error> {
        let root_path = Path::new(&self.data_dir);
        if root_path.is_dir() {
            if let Err(error) = fs::remove_dir_all(root_path) {
                return Err(Error::Other(format!(
                    "Failed to delete root data directory: {} - {}",
                    self.data_dir, error
                )));
            }
        }
        if let Err(error) = self.init() {
            return Err(Error::Other(format!(
                "Failed to reinitialize FileStore: {error}"
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
    async fn new_store() -> FileStore {
        let mut store = FileStore::new(&FileStoreConfig::default());
        if let Err(error) = store.empty().await {
            panic!("new_store() failed to empty content: {error}");
        }
        store
    }

    // Initialize a User struct
    fn init_test_user(
        is_admin: bool,
        username: &str,
        full_name: &str,
        email: &str,
        password_hash: &str,
    ) -> User {
        User {
            id: User::new_id(),
            is_admin,
            username: username.to_string(),
            full_name: full_name.to_string(),
            email: email.to_string(),
            password_hash: password_hash.to_string(),
            api_key: User::new_api_key(),
            logins: vec![],
            oauth_identities: vec![],
        }
    }

    // Create a user in the file store
    async fn create_user(
        store: &mut FileStore,
        is_admin: bool,
        username: &str,
        full_name: &str,
        email: &str,
        password_hash: &str,
    ) -> User {
        let mut user = init_test_user(is_admin, username, full_name, email, password_hash);

        if let Err(error) = store.create_user(&mut user).await {
            panic!("{}", error);
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
    // FileStore-specific tests
    //
    // The bulk of FileStore behaviour — user/maze CRUD, find/list, error
    // semantics — is now exercised through the backend-agnostic
    // `Store` trait contract suite in `tests/file_store_contract.rs`. Only
    // tests that depend on private FileStore symbols (`maze_path`,
    // `users_dir`, `write_maze_file`) or on filesystem-level edge cases
    // (orphaned user directories, pre-existing on-disk files) remain here.
    //****************************************************************

    // ─── Orphaned-directory recovery ──────────────────────────────────

    #[tokio::test]
    async fn get_users_skips_orphaned_user_directory() {
        let mut store = new_store().await;
        let _ = create_user(&mut store, false, "valid", "", "valid@company.com", "hash").await;
        let orphan_id = Uuid::new_v4();
        std::fs::create_dir_all(std::path::Path::new(&store.users_dir).join(orphan_id.to_string()))
            .expect("failed to create orphan directory");
        let users = store.get_users().await.expect("get_users should succeed despite orphaned directory");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "valid");
    }

    #[tokio::test]
    async fn find_user_by_email_skips_orphaned_user_directory() {
        let mut store = new_store().await;
        let _ = create_user(&mut store, false, "valid", "", "valid@company.com", "hash").await;
        let orphan_id = Uuid::new_v4();
        std::fs::create_dir_all(std::path::Path::new(&store.users_dir).join(orphan_id.to_string()))
            .expect("failed to create orphan directory");
        store.find_user_by_email("valid@company.com").await.expect("find_user_by_email should succeed despite orphaned directory");
    }

    #[tokio::test]
    async fn find_user_by_oauth_identity_skips_orphaned_user_directory() {
        use data_model::OAuthIdentity;
        let mut store = new_store().await;
        let mut alice = init_test_user(false, "valid", "", "valid@company.com", "hash");
        alice.oauth_identities.push(OAuthIdentity::new(
            "google".to_string(),
            "sub-1".to_string(),
            Some("valid@company.com".to_string()),
        ));
        store.create_user(&mut alice).await.expect("create user");
        let orphan_id = Uuid::new_v4();
        std::fs::create_dir_all(std::path::Path::new(&store.users_dir).join(orphan_id.to_string()))
            .expect("failed to create orphan directory");
        store.find_user_by_oauth_identity("google", "sub-1").await
            .expect("find_user_by_oauth_identity should succeed despite orphaned directory");
    }

    #[tokio::test]
    async fn find_user_by_login_id_skips_orphaned_user_directory() {
        let mut store = new_store().await;
        let mut user = init_test_user(false, "valid", "", "valid@company.com", "hash");
        let login = data_model::UserLogin::new(24, None, None);
        let login_id = login.id;
        user.logins.push(login);
        store.create_user(&mut user).await.expect("failed to create user");
        let orphan_id = Uuid::new_v4();
        std::fs::create_dir_all(std::path::Path::new(&store.users_dir).join(orphan_id.to_string()))
            .expect("failed to create orphan directory");
        store.find_user_by_login_id(login_id).await.expect("find_user_by_login_id should succeed despite orphaned directory");
    }

    // ─── Private `write_maze_file` overwrite-flag behaviour ──────────

    #[tokio::test]
    async fn can_save_maze_to_valid_file_path() {
        let mut store = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "test",
            "",
            "test@company.com",
            "password_hash",
        ).await;
        let (id, mut maze) = init_test_maze(&store, "maze", true, true);

        match store.write_maze_file(&owner, &mut maze, &id, true) {
            Ok(_) => {}
            Err(error) => panic!("Failed to save to file: {error}"),
        }
    }

    #[tokio::test]
    #[should_panic(expected = "A maze with id 'maze.json' already exists")]
    async fn cannot_save_maze_to_existing_file_path_if_overwrite_disabled() {
        let mut store = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "test",
            "",
            "test@company.com",
            "password_hash",
        ).await;
        let (id, mut maze) = init_test_maze(&store, "maze", true, true);
        let path = store.maze_path(&owner, &id);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.write_maze_file(&owner, &mut maze, &id, false) {
            Ok(_) => {
                panic!(
                    "Successfully saved to existing file: {path} despite overwrite being false"
                );
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[tokio::test]
    async fn can_save_maze_to_existing_file_path_if_overwrite_enabled() {
        let mut store = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "test",
            "",
            "test@company.com",
            "password_hash",
        ).await;
        let (id, mut maze) = init_test_maze(&store, "maze", false, true);
        let path = store.maze_path(&owner, &id);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.write_maze_file(&owner, &mut maze, &id, true) {
            Ok(_) => {}
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    // ─── Pre-existing on-disk maze file (orphan-file detection) ──────

    #[tokio::test]
    #[should_panic(expected = "A maze with id 'maze.json' already exists")]
    async fn cannot_create_maze_that_exists() {
        let mut store = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "test",
            "",
            "test@company.com",
            "password_hash",
        ).await;
        let (id, mut maze) = init_test_maze(&store, "maze", false, true);
        let path = store.maze_path(&owner, &id);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.create_maze(&owner, &mut maze).await {
            Ok(_) => {
                panic!(
                    "Successfully created maze when file: {path} existed, when should not have"
                );
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }
}
