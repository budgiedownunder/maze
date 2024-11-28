use maze::{Maze};
use storage::{MazeItem, Store, SharedStore, StoreError};
use actix_web::{get, delete, web, HttpResponse, Error};
use std::sync::{RwLockReadGuard, RwLockWriteGuard, RwLock, Arc};

fn get_store_read_lock<'a>(
    store: &'a web::Data<Arc<RwLock<Box<dyn Store>>>>,
) -> Result<RwLockReadGuard<'a, Box<dyn Store>>, Error> {
    store.read().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire store read lock")
    })
}

fn get_store_write_lock<'a>(
    store: &'a web::Data<Arc<RwLock<Box<dyn Store>>>>,
) -> Result<RwLockWriteGuard<'a, Box<dyn Store>>, Error> {
    store.write().map_err(|_| {
        actix_web::error::ErrorInternalServerError("Failed to acquire store write lock")
    })
}

fn get_mazes_fetch_internal_error(err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error fetching maze items: {}", err))
}

fn get_maze_not_found_error(id: &str) -> Error {
    actix_web::error::ErrorNotFound(format!("Maze with id '{}' not found", id))
}

fn get_maze_fetch_internal_error(id: &str, err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error fetching maze item with id '{}': {}", id, err))
}

// get_maze_list
#[utoipa::path(
    get,
    path = "/api/v1/mazes",
    responses(
        (status = 200, description = "Maze definitions", body=[MazeItem])
    ),
    tags = ["v1"]
)]
#[get("/mazes")]
pub async fn get_maze_list(store: web::Data<SharedStore>) -> Result<HttpResponse, Error> {
    let store_lock = get_store_read_lock(&store)?;
    let stored_items = store_lock.get_maze_items().map_err(|err| {
        get_mazes_fetch_internal_error(&err)
    })?;
    Ok(HttpResponse::Ok().json(stored_items))    
}

// get_maze
#[utoipa::path(
    get,
    path = "/api/v1/mazes/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to retrieve")
    ),
    responses(
        (status = 200, description = "Maze retrieved successfully", body = Maze),
        (status = 404, description = "Maze not found")
    ),
    tags = ["v1"]
)]
#[get("/mazes/{id}")]
pub async fn get_maze(
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
) -> Result<HttpResponse, Error> {
    let store_lock = get_store_read_lock(&store)?;
    let id = path.into_inner();

    match store_lock.get_maze(&id) {
        Ok(maze) => Ok(HttpResponse::Ok().json(maze)),
        Err(err) => {
            match err {
               StoreError::IdNotFound(id) => Err(get_maze_not_found_error(&id)),
                _ => Err(get_maze_fetch_internal_error(&id, &err))
            }    
        }
    }
}

// delete_maze
#[utoipa::path(
    delete,
    path = "/api/v1/mazes/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to delete")
    ),
    responses(
        (status = 200, description = "Maze deleted successfully", body = Maze),
        (status = 404, description = "Maze not found")
    ),
    tags = ["v1"]
)]
#[delete("/mazes/{id}")]
pub async fn delete_maze(
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store)?;
    let id = path.into_inner();

    match store_lock.delete_maze(&id) {
        Ok(()) => Ok(HttpResponse::Ok().body(format!("maze with id '{}' deleted", id))),
        Err(err) => {
            match err {
                    StoreError::IdNotFound(id) => Err(get_maze_not_found_error(&id)),
                _ => Err(get_maze_fetch_internal_error(&id, &err))
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::api::v1::handlers;
    use maze::{Definition, Maze};
    use storage::{SharedStore, Store, StoreError, MazeItem};

    use actix_web::{http::StatusCode, test, web, App};
    
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    #[derive(Clone, Debug)]
    struct MazeStoreItem {
        id: String,
        name: String,
        maze: Maze,
    }

    impl MazeStoreItem {
        pub fn to_maze_item(&self) -> MazeItem {
            MazeItem {
                id: self.id.clone(),
                name: self.name.clone()
            }
        } 
    } 

    //type MazeStoreItemMap HashMap<String, MazeStoreItem>;
    struct MockMazeStore {
        items: HashMap<String, MazeStoreItem>,
    }

    #[derive(Clone)]
    enum StoreStartupContent {
        Empty,
        OneMaze,
        TwoMazes,
        ThreeMazes,
    }

    fn maze_store_items_to_maze_items(from: Vec<MazeStoreItem>) -> Vec<MazeItem> {
        from.iter()
            .map( |value| value.to_maze_item())
            .collect()
    }

    fn maze_store_items_to_map(items: &Vec<MazeStoreItem>) -> HashMap<String, MazeStoreItem> {
        let mut map: HashMap<String, MazeStoreItem> = HashMap::new();
        for item in items {
            map.insert(item.id.clone(), item.clone());
        }
        map 
    }

    fn maze_items_from_map(from: &HashMap<String, MazeStoreItem>) -> Vec<MazeItem> {
        from.iter()
            .map(|(_key, value) | MazeItem {
                    id: value.id.clone(),
                    name: value.name.clone(),
            })
            .collect()
    }

    fn new_solvable_maze(id: &str, name: &str) -> Maze {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', 'W', ' ', ' ', 'W'],
            vec![' ', 'W', ' ', 'W', ' '],
            vec![' ', ' ', ' ', 'W', 'F'],
            vec!['W', ' ', 'W', ' ', ' '],
            vec![' ', ' ', ' ', 'W', ' '],
            vec!['W', 'W', ' ', ' ', ' '],
            vec!['W', 'W', ' ', 'W', ' '],
        ];
        let mut maze:Maze = Maze::new(Definition::from_vec(grid));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
    }    

    fn new_solvable_maze_store_item(id: &str, name: &str) -> MazeStoreItem {
        MazeStoreItem {
            id: id.to_string(),
            name: name.to_string(),
            maze: new_solvable_maze(id, name),
        }
    }

    fn get_startup_content(startup_content: StoreStartupContent, sort_asc: bool) -> Vec<MazeStoreItem> {
        let mut result: Vec<MazeStoreItem>;
        match startup_content {
            StoreStartupContent::Empty => {
                result = Vec::new();
            } 
            StoreStartupContent::OneMaze => {
                result = vec![
                    new_solvable_maze_store_item("maze_a.json", "maze_a")
                ]
            } 
            StoreStartupContent::TwoMazes => {
                result = vec![
                    new_solvable_maze_store_item("maze_b.json", "maze_b"),
                    new_solvable_maze_store_item("maze_a.json", "maze_a"),
                ]
            } 
            StoreStartupContent::ThreeMazes => {
                result = vec![
                    new_solvable_maze_store_item("maze_c.json", "maze_c"),
                    new_solvable_maze_store_item("maze_b.json", "maze_b"),
                    new_solvable_maze_store_item("maze_a.json", "maze_a"),
                ]
            } 
        }
        
        if sort_asc {
            result.sort_by_key(|item| item.name.clone());
        }

        result
    }

    fn new_item_map(startup_content: StoreStartupContent) -> HashMap<String, MazeStoreItem> {
        maze_store_items_to_map(&get_startup_content(startup_content, false))
    }

    impl MockMazeStore {
        pub fn new(startup_content: StoreStartupContent) -> Self {
            MockMazeStore {
                items: new_item_map(startup_content)
            }
        } 
    }

    impl Store for MockMazeStore {

        fn create_maze(&mut self, _maze: &mut Maze) -> Result<(), StoreError> {
            return Err(StoreError::Other("Mock interface not implemented".to_string()));
        }

        fn delete_maze(&mut self, id: &str) -> Result<(), StoreError> {
            if let Some(_) = self.items.remove(id) {
                Ok(())                
            } else {
                Err(StoreError::IdNotFound(id.to_string()))
            }
        }

        fn update_maze(&mut self, _maze: &mut Maze) -> Result<(), StoreError> {
            return Err(StoreError::Other("Mock interface not implemented".to_string()));
        }

        fn get_maze(&self, id: &str) -> Result<Maze, StoreError> {
            if let Some(store_item) = self.items.get(id) {
                return Ok(store_item.maze.clone());
            }
            Err(StoreError::IdNotFound(id.to_string()))
        }

        fn find_maze_by_name(&self, _name: &str) -> Result<MazeItem, StoreError> {
            return Err(StoreError::Other("Mock interface not implemented".to_string()));
        }

        fn get_maze_items(&self) -> Result<Vec<MazeItem>, StoreError> {
            let mut items: Vec<MazeItem> = maze_items_from_map(&self.items);
            items.sort_by_key(|item| item.name.clone());
            Ok(items)
        }
    }

    fn new_shared_mock_maze_store(startup_content: StoreStartupContent) -> SharedStore {
        Arc::new(RwLock::new(Box::new(MockMazeStore::new(startup_content))))
    }

    fn configure_mock_app(app: &mut web::ServiceConfig, startup_content: StoreStartupContent) {
        let mock_store = new_shared_mock_maze_store(startup_content);
    
        app.app_data(web::Data::new(mock_store.clone()))
            .service(
                web::scope("/api/v1")
                    .service(handlers::get_maze_list)
                    .service(handlers::get_maze)
                    .service(handlers::delete_maze),
            );
    }

    async fn run_get_mazes_test(startup_content: StoreStartupContent) {
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, startup_content.clone())),
        )
        .await;

        // Get
        let req = test::TestRequest::get().uri("/api/v1/mazes").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let maze_items: Vec<MazeItem> = serde_json::from_slice(&body).expect("failed to deserialize response");
        assert_eq!(
            maze_items,
            maze_store_items_to_maze_items(get_startup_content(startup_content, true))   
        );        
    }

    async fn run_get_maze_test(
        startup_content: StoreStartupContent, 
        id: &str, 
        expected_status_code: StatusCode, 
        expected_maze: Option<Maze>
        ) {
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, startup_content.clone())),
        )
        .await;

        // Get
        let url = format!("/api/v1/mazes/{}", id);
        let req = test::TestRequest::get().uri(&url).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Verify content
            let body = test::read_body(resp).await;
             let maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
             match expected_maze {
                Some(value) => {
                    assert_eq!(
                        maze,
                        value
                    );        
                }
                None => {
                    panic!("No maze comparison value provided for get_maze() test!");
                }
             }
        }
    }

    async fn run_delete_maze_test(
        startup_content: StoreStartupContent, 
        id: &str, 
        expected_status_code: StatusCode 
        ) {
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, startup_content.clone())),
        )
        .await;
        // Delete
        let url = format!("/api/v1/mazes/{}", id);
        let req = test::TestRequest::delete().uri(&url).to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Confirm it has been deleted
            let url2 = format!("/api/v1/mazes/{}", id);
            let req2 = test::TestRequest::get().uri(&url2).to_request();
            let resp2 = test::call_service(&app, req2).await;
            assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
        }
    }

    #[actix_web::test]
    async fn test_get_mazes_with_no_mazes() {
        run_get_mazes_test(StoreStartupContent::Empty).await
    }

    #[actix_web::test]
    async fn test_get_mazes_with_one_maze() {
        run_get_mazes_test(StoreStartupContent::OneMaze).await
    }

    #[actix_web::test]
    async fn test_get_mazes_with_two_mazes_that_require_sorting() {
        run_get_mazes_test(StoreStartupContent::TwoMazes).await
    }

    #[actix_web::test]
    async fn test_get_mazes_with_three_mazes_that_require_sorting() {
        run_get_mazes_test(StoreStartupContent::ThreeMazes).await
    }

    #[actix_web::test]
    async fn test_get_maze_that_exists() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_get_maze_test(StoreStartupContent::ThreeMazes, id, StatusCode::OK, Some(new_solvable_maze(id, name))).await
    }

    #[actix_web::test]
    async fn test_get_maze_that_does_not_exist() {
        run_get_maze_test(StoreStartupContent::ThreeMazes, "does_not_exist.json", StatusCode::NOT_FOUND, None).await
    }

    #[actix_web::test]
    async fn test_delete_maze_that_exists() {
        run_delete_maze_test(StoreStartupContent::ThreeMazes, "maze_a.json", StatusCode::OK).await
    }

    #[actix_web::test]
    async fn test_delete_maze_that_does_not_exist() {
        run_delete_maze_test(StoreStartupContent::ThreeMazes, "does_not_exist.json", StatusCode::NOT_FOUND).await
    }

}
