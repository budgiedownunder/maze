use crate::StoreError;
use maze::{Maze};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a store for holding users
pub trait UserStore {
    /// Adds the default admin user to the store if it doesn't already exist, else returns it 
    fn init_default_admin_user(&mut self, username: &str, password_hash: &str) -> Result<User, StoreError>;
    /// Adds a new user to the store and sets the allocated `id` within the user object
    fn create_user(&mut self, user: &mut User) -> Result<(), StoreError>;
    /// Deletes a user from the store
    fn delete_user(&mut self, id: Uuid) -> Result<(), StoreError>;
    /// Updates a user within the store
    fn update_user(&mut self, user: &mut User) -> Result<(), StoreError>;
    /// Loads a user from the store
    fn get_user(&self, id: Uuid) -> Result<User, StoreError>;
    /// Locates a user by their username within the store
    fn find_user_by_name(&self, name: &str) -> Result<User, StoreError>;
    /// Locates a user by their api key within the store
    fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, StoreError>;
    /// Returns the list of users within the store, sorted
    /// alphabetically by username in ascending order
    fn get_users(&self) -> Result<Vec<User>, StoreError>;
}
/// Contains the identifying details for a maze item and (optionally)
/// the definition JSON
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct MazeItem {
    /// Maze ID
    pub id: String,
    /// Maze name
    pub name: String,
    /// Maze definition
    pub definition: Option<String>, // JSON
}

/// Represents a store for holding mazes and related objects
pub trait MazeStore {
    /// Adds a new maze to the store and sets the allocated `id` within the maze object
    fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError>;
    /// Deletes a maze from the store
    fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), StoreError>;
    /// Updates a maze within the store
    fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError>;
    /// Loads a maze from the store
    fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, StoreError>;
    /// Locates a maze item by its name within the store
    fn find_maze_by_name(&self, owner: &User, name: &str) -> Result<MazeItem, StoreError>;
    /// Returns the list of maze items within the store, sorted
    /// alphabetically in ascending order
    fn get_maze_items(&self, owner: &User, include_definitions: bool) -> Result<Vec<MazeItem>, StoreError>;
}

/// Represents a user of the system 
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct User {
    #[schema(value_type = String)] // Treat as string during serlialization
    /// User ID
    pub id: Uuid,
    /// Is administrator?
    pub is_admin: bool,
    /// Username
    pub username: String,
    /// Full name 
    pub full_name: String,
    /// Email address
    pub email: String,
    /// Password hash (encrypted)
    pub password_hash: String,
    #[schema(value_type = String)] // Treat as string during serlialization
    /// API key
    pub api_key: Uuid,
}

impl User {
    /// Creates a new user id
    pub fn new_id() -> Uuid {
        Uuid::new_v4()
    }

    /// Creates a new API key
    pub fn new_api_key() -> Uuid {
        Uuid::new_v4()
    }

    /// Returns a User instance initialized with the defautl values
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> User {
        User {
            id: Uuid::nil(),
            is_admin: false,
            username: "".to_string(),
            full_name: "".to_string(),
            email: "".to_string(),
            password_hash: "".to_string(),
            api_key: Uuid::nil(),
        }
    }

    /// Converts the instance to a JSON string
    pub fn to_json(&self) -> Result<String, StoreError> {
        Ok(serde_json::to_string(&self)?)
    }
}

// Store management
pub trait Manage {
    /// Resets the store to empty
    fn empty(&mut self) -> Result<(), StoreError>;
}

/// Represents a store
pub trait Store: UserStore + MazeStore + Manage + Send + Sync {}
#[allow(dead_code)]
pub type SharedStore = Arc<RwLock<Box<dyn Store>>>;