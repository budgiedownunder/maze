use std::fs;
use std::fs::File;
use std::path::{Path as StdPath, PathBuf};
use std::io::{/*self, BufReader,*/ Write};

use maze::Maze;

use crate::MazeItem;
use crate::Store;
use crate::StoreError;

pub struct FileStore {}

impl FileStore {
    pub fn new() -> Self {
        FileStore {}
    }

    fn save_maze_to_file(&self, maze: &mut Maze, path: &str, overwrite: bool) -> Result<(), StoreError> {
        if !overwrite {
            let os_path = PathBuf::from(path);
            if StdPath::new(&os_path).exists() {
                return Err(StoreError::AlreadyExists(path.to_string()));
            }
        }
        let s = maze.to_json()?;
        let mut file = File::create(path)?;
        file.write_all(s.as_bytes())?;
        maze.id = path.to_string();
        Ok(())
    }
}

impl Store for FileStore {
    fn generate_maze_id(&self, name: &str) -> String {
        format!("{}.json", name)
    }

    fn create_maze(&self, maze: &mut Maze) -> Result<(), StoreError> {
        let file_id = self.generate_maze_id(&maze.name);
        if let Err(err) = self.save_maze_to_file(maze, &file_id, false) {
            return Err(StoreError::from(err));
        }
        Ok(())
    }

    fn delete_maze(&self, id: &str) -> Result<(), StoreError> {
        Err(StoreError::NotFound(id.to_string()))
    }

    fn update_maze(&self, maze: &mut Maze) -> Result<(), StoreError> {
        let file_id = self.generate_maze_id(&maze.name);
        if let Err(err) = self.save_maze_to_file(maze, &file_id, true) {
            return Err(StoreError::from(err));
        }
        Ok(())
    }

    fn get_maze(&self, id: &str) -> Result<Maze, StoreError> {
        Err(StoreError::NotFound(id.to_string()))
    }

    fn find_maze_by_name(&mut self, name: &str) -> Result<MazeItem, StoreError> {
        let file_id = self.generate_maze_id(name);
        let path = PathBuf::from(file_id.clone());

        if !name.is_empty() && StdPath::new(&path).exists() {
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
