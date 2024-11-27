use storage::MazeItem;
use storage::SharedStore;
use actix_web::{get, web, HttpResponse, Responder};

#[utoipa::path(
    get,
    path = "/api/v1/mazes",
    responses(
        (status = 200, description = "Maze definitions", body=[MazeItem])
    ),
    tags = ["v1"] // Explicitly set the tag name
)]
#[get("/mazes")]
pub async fn get_maze_list(store: web::Data<SharedStore>) -> impl Responder {
    let store_lock = match store.read() {
        Ok(lock) => lock,
        Err(_) => {
            return HttpResponse::InternalServerError().body("Failed to acquire store lock");
        }
    };

    match store_lock.get_maze_items() {
        Ok(stored_items) => HttpResponse::Ok().json(stored_items),
        Err(err) => {
            eprintln!("Error fetching maze items: {}", err);
            HttpResponse::InternalServerError().body(format!("Error: {}", err))
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::api::v1::handlers;
    use maze::Maze;
    use storage::{SharedStore, Store, StoreError, MazeItem};

    use actix_web::{http::StatusCode, test, web, App};
    
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};


    #[derive(Clone, Debug)]
    struct MazeStoreItem {
        id: String,
        name: String,
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

    fn get_startup_content(startup_content: StoreStartupContent, sort_asc: bool) -> Vec<MazeStoreItem> {
        let mut result: Vec<MazeStoreItem>;
        match startup_content {
            StoreStartupContent::Empty => {
                result = Vec::new();
            } 
            StoreStartupContent::OneMaze => {
                result = vec![
                    MazeStoreItem {id:"maze_a.json".to_string(), name: "maze_a".to_string()}
                ]
            } 
            StoreStartupContent::TwoMazes => {
                result = vec![
                    MazeStoreItem {id:"maze_b.json".to_string(), name: "maze_b".to_string()},
                    MazeStoreItem {id:"maze_a.json".to_string(), name: "maze_a".to_string()},
                ]
            } 
            StoreStartupContent::ThreeMazes => {
                result = vec![
                    MazeStoreItem {id:"maze_c.json".to_string(), name: "maze_c".to_string()},
                    MazeStoreItem {id:"maze_b.json".to_string(), name: "maze_b".to_string()},
                    MazeStoreItem {id:"maze_a.json".to_string(), name: "maze_a".to_string()},
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

        fn create_maze(&self, _maze: &mut Maze) -> Result<(), StoreError> {
            return Err(StoreError::Other("Mock interface not implemented".to_string()));
        }

        fn delete_maze(&self, _id: &str) -> Result<(), StoreError> {
            return Err(StoreError::Other("Mock interface not implemented".to_string()));
        }

        fn update_maze(&self, _maze: &mut Maze) -> Result<(), StoreError> {
            return Err(StoreError::Other("Mock interface not implemented".to_string()));
        }

        fn get_maze(&self, _id: &str) -> Result<Maze, StoreError> {
            return Err(StoreError::Other("Mock interface not implemented".to_string()));
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
                    .service(handlers::get_maze_list),
            );
    }

    async fn run_get_mazes_test(startup_content: StoreStartupContent) {
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, startup_content.clone())),
        )
        .await;

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

}
