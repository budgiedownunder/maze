use crate::Error;
use data_model::{Maze, User};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents a store for holding users
pub trait UserStore {
    /// Adds the default admin user to the store if it doesn't already exist, else returns it 
    fn init_default_admin_user(&mut self, username: &str, password_hash: &str) -> Result<User, Error>;
    /// Adds a new user to the store and sets the allocated `id` within the user object
    fn create_user(&mut self, user: &mut User) -> Result<(), Error>;
    /// Deletes a user from the store
    fn delete_user(&mut self, id: Uuid) -> Result<(), Error>;
    /// Updates a user within the store
    fn update_user(&mut self, user: &mut User) -> Result<(), Error>;
    /// Loads a user from the store
    fn get_user(&self, id: Uuid) -> Result<User, Error>;
    /// Locates a user by their username within the store
    fn find_user_by_name(&self, name: &str) -> Result<User, Error>;
    /// Locates a user by their api key within the store
    fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, Error>;
    /// Returns the list of users within the store, sorted
    /// alphabetically by username in ascending order
    fn get_users(&self) -> Result<Vec<User>, Error>;
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
    fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error>;
    /// Deletes a maze from the store
    fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), Error>;
    /// Updates a maze within the store
    fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error>;
    /// Loads a maze from the store
    fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, Error>;
    /// Locates a maze item by its name within the store
    fn find_maze_by_name(&self, owner: &User, name: &str) -> Result<MazeItem, Error>;
    /// Returns the list of maze items within the store, sorted
    /// alphabetically in ascending order
    fn get_maze_items(&self, owner: &User, include_definitions: bool) -> Result<Vec<MazeItem>, Error>;
}
// Store management
pub trait Manage {
    /// Resets the store to empty
    fn empty(&mut self) -> Result<(), Error>;
}

/// Represents a store
pub trait Store: UserStore + MazeStore + Manage + Send + Sync {}
#[allow(dead_code)]
pub type SharedStore = Arc<RwLock<Box<dyn Store>>>;