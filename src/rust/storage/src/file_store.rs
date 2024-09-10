use std::fs;
use std::path::{Path as stdPath, PathBuf};

use maze::Maze;

use crate::MazeItem;
use crate::Store;
use crate::StoreError;

pub struct FileStore {}

impl FileStore {
    pub fn new() -> Self {
        FileStore {}
    }
}

impl Store for FileStore {
    fn generate_maze_id(&self, name: &str) -> String {
        format!("{}.json", name)
    }

    fn create_maze(&self, _maze: &mut Maze) -> Result<(), StoreError> {
        Err(StoreError::Other(
            "create_maze() not implemented for FileStore".to_string(),
        ))
    }

    fn delete_maze(&self, id: &str) -> Result<(), StoreError> {
        Err(StoreError::NotFound(id.to_string()))
    }

    fn update_maze(&self, maze: &mut Maze) -> Result<(), StoreError> {
        Err(StoreError::NotFound(maze.id.clone()))
    }

    fn get_maze(&self, id: &str) -> Result<Maze, StoreError> {
        Err(StoreError::NotFound(id.to_string()))
    }

    fn find_maze_by_name(&mut self, name: &str) -> Result<MazeItem, StoreError> {
        let file_id = self.generate_maze_id(name);
        let path = PathBuf::from(file_id.clone());

        if !name.is_empty() && stdPath::new(&path).exists() {
            return Ok(MazeItem {
                id: file_id,
                name: name.to_string(),
            });
        }
        Err(StoreError::NotFound(name.to_string()))
    }

    fn get_maze_items(&self) -> Result<Vec<MazeItem>, StoreError> {
        let mut items: Vec<MazeItem> = Vec::new();
        let current_dir = std::env::current_dir()?;

        for entry in fs::read_dir(current_dir)? {
            let entry = entry?;
            let path = entry.path();
            if let Some(path_str) = path.to_str() {
                if let Some(extension) = path.extension() {
                    if extension == "json" {
                        if let Some(name) = path.file_stem() {
                            if let Some(name_str) = name.to_str() {
                                items.push(MazeItem {
                                    id: path_str.to_string(),
                                    name: name_str.to_string(),
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
