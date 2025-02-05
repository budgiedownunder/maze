use crate::StoreError;
use maze::{Maze};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use utoipa::ToSchema;
use uuid::Uuid;

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
    fn create_maze(&mut self, maze: &mut Maze) -> Result<(), StoreError>;
    /// Deletes a maze from the store
    fn delete_maze(&mut self, id: &str) -> Result<(), StoreError>;
    /// Updates a maze within the store
    fn update_maze(&mut self, maze: &mut Maze) -> Result<(), StoreError>;
    /// Loads a maze from the store
    fn get_maze(&self, id: &str) -> Result<Maze, StoreError>;
    /// Locates a maze item by its name within the store
    fn find_maze_by_name(&self, name: &str) -> Result<MazeItem, StoreError>;
    /// Returns the list of maze items within the store, sorted
    /// alphabetically in ascending order
    fn get_maze_items(&self, include_definitions: bool) -> Result<Vec<MazeItem>, StoreError>;
}

/// Represents a user of the system 
#[derive(Serialize, Deserialize, ToSchema, Debug, PartialEq, Clone)]
pub struct User {
    #[schema(value_type = String)] // Treat a string during serlialization
    /// User ID
    pub id: Uuid,
    /// Username
    pub name: String,
    /// Full name 
    pub full_name: String,
    /// Email address
    pub email: String,
    /// Password (encrypted)
    pub password: String,
}

/// Represents a store for holding users
pub trait UserStore {
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
    /// Returns the list of users within the store, sorted
    /// alphabetically by username in ascending order
    fn get_users(&self) -> Result<Vec<User>, StoreError>;
}

/// Represents a store
pub trait Store: UserStore + MazeStore + Send + Sync {}
#[allow(dead_code)]
pub type SharedStore = Arc<RwLock<Box<dyn Store>>>;