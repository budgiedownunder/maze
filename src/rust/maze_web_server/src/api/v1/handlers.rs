use maze::{Maze, MazeError, Solution};
use storage::{MazeItem, Store, SharedStore, StoreError};
use actix_web::{delete, get, post, put, web, HttpResponse, Error};
use std::sync::{RwLockReadGuard, RwLockWriteGuard, RwLock, Arc};
use urlencoding::encode;

// **************************************************************************************************
// Private utility functions
// **************************************************************************************************
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

fn get_maze_create_internal_error(err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error creating maze: {}", err))
}

fn get_maze_not_found_error(id: &str) -> Error {
    actix_web::error::ErrorNotFound(format!("Maze with id '{}' not found", id))
}

fn get_maze_exists_error(id: &str) -> Error {
    actix_web::error::ErrorConflict(format!("Maze with id '{}' already exists", id))
}

fn get_maze_fetch_internal_error(id: &str, err: &StoreError) -> Error {
    actix_web::error::ErrorInternalServerError(format!("Error fetching maze item with id '{}': {}", id, err))
}

fn get_maze_id_mismatch_error(url_id: &str, maze_id: &str) -> Error {
    actix_web::error::ErrorBadRequest(format!("URL ID '{}' and body maze ID '{}' do not match", url_id, maze_id))
}

fn get_maze_solve_error_string(err: &MazeError) -> String {
    format!("The maze could not be solved: {}", err)
}

fn get_maze_solve_error(err: &MazeError) -> Error {
    actix_web::error::ErrorUnprocessableEntity(get_maze_solve_error_string(err))
}

// **************************************************************************************************
// Endpoint: GET /api/v1/mazes
// Handler:  get_maze_list()
// **************************************************************************************************
#[utoipa::path(
    summary = "Returns the list of available mazes",
    description = "This endpoint returns the list of maze IDs (and names) that the user currently has access to",
    get,
    path = "/api/v1/mazes",
    responses(
        (status = 200, description = "Maze list loaded sucessfully", body=[MazeItem]),
        (status = 400, description = "Invalid request"),
    ),
    security(
        ()
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

// **************************************************************************************************
// Endpoint: PUT /api/v1/mazes/
// Handler:  create_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Creates a new maze from the supplied definition",
    description = "This endpoint creates a new maze and, if successful, returns the newly created maze object containing its allocated ID",
    post,
    path = "/api/v1/mazes",
    request_body = Maze,
    responses(
        (status = 201, description = "Maze created successfully", body = Maze),
        (status = 400, description = "Invalid request"),
        (status = 409, description = "Maze with the given id already exists")
    ),
    security(
        ()
    ),
    tags = ["v1"]
)]
#[post("/mazes/")]
pub async fn create_maze(
    req: web::Json<Maze>,
    store: web::Data<SharedStore>,  
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store)?;
    let mut maze: Maze = req.into_inner();

    match store_lock.create_maze(&mut maze) {
        Ok(()) => Ok(
                HttpResponse::Created()
                .insert_header(("Location", format!("/api/v1/mazes/{}", encode(&maze.id))))
                .json(maze)),
        Err(err) => {
            match err {
                StoreError::IdAlreadyExists(id) => Err(get_maze_exists_error(&id)),
                _ => Err(get_maze_create_internal_error(&err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: GET /api/v1/maze/{id}
// Handler:  get_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Loads an existing maze definition",
    description = "This endpoint attempts to load a maze given its ID and, if successful, returns the maze definition",
    get,
    path = "/api/v1/mazes/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to retrieve")
    ),
    responses(
        (status = 200, description = "Maze retrieved successfully", body = Maze),
        (status = 404, description = "Maze not found")
    ),
    security(
        ()
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
// **************************************************************************************************
// Endpoint: PUT /api/v1/mazes/{id}
// Handler:  update_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Updates an existing maze with a new definition",
    description = "This endpoint attempts to update an existing maze given its ID and, if successful, returns the updated maze definition",
    put,
    path = "/api/v1/mazes/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to update")
    ),
    request_body = Maze,
    responses(
        (status = 200, description = "Maze updated successfully", body = Maze),
        (status = 400, description = "Invalid request"),
        (status = 404, description = "Maze not found")
    ),
    security(
        ()
    ),
    tags = ["v1"]
)]
#[put("/mazes/{id}")]
pub async fn update_maze(
    path: web::Path<String>, 
    req: web::Json<Maze>,
    store: web::Data<SharedStore>,  
) -> Result<HttpResponse, Error> {
    let mut store_lock = get_store_write_lock(&store)?;
    let id = path.into_inner();
    let mut maze = req.into_inner();

    if id != maze.id {
        return Err(get_maze_id_mismatch_error(&id, &maze.id));
    }

    match store_lock.update_maze(&mut maze) {
        Ok(_) => Ok(HttpResponse::Ok().json(maze)),
        Err(err) => {
            match err {
               StoreError::IdNotFound(id) => Err(get_maze_not_found_error(&id)),
                _ => Err(get_maze_fetch_internal_error(&id, &err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: DELETE /api/v1/mazes/{id}
// Handler:  delete_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Deletes an existing maze",
    description = "This endpoint attempts to delete an existing maze given its ID",
    delete,
    path = "/api/v1/mazes/{id}",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to delete")
    ),
    responses(
        (status = 200, description = "Maze deleted successfully", body = Maze),
        (status = 404, description = "Maze not found")
    ),
    security(
        ()
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
// **************************************************************************************************
// Endpoint: GET /api/v1/mazes/{id}/solution
// Handler:  get_maze_solution()
// **************************************************************************************************
#[utoipa::path(
    summary = "Attempts to solve an existing maze",
    description = "This endpoint attempts to solve a maze given its ID and, if successful, returns a maze solution containing the solution path",
    get,
    path = "/api/v1/mazes/{id}/solution",
    params(
        ("id" = String, Path, description = "Unique ID of the maze to solve")
    ),
    responses(
        (status = 200, description = "Maze solved successfully", body = Solution),
        (status = 404, description = "Maze not found"),
        (status = 422, description = "Maze could not be solved")
    ),
    security(
        ()
    ),
    tags = ["v1"]
)]
#[get("/mazes/{id}/solution")]
pub async fn get_maze_solution(
    path: web::Path<String>, 
    store: web::Data<SharedStore>,  
) -> Result<HttpResponse, Error> {
    let store_lock = get_store_read_lock(&store)?;
    let id = path.into_inner();

    match store_lock.get_maze(&id) {
        Ok(maze) => {
            match maze.solve() {
                Ok(solution) => Ok(HttpResponse::Ok().json(solution)),
                Err(err) => Err(get_maze_solve_error(&err))
            }
        }    
        Err(err) => {
            match err {
               StoreError::IdNotFound(id) => Err(get_maze_not_found_error(&id)),
                _ => Err(get_maze_fetch_internal_error(&id, &err))
            }    
        }
    }
}
// **************************************************************************************************
// Endpoint: POST /api/v1/solve-maze/
// Handler:  solve_maze()
// **************************************************************************************************
#[utoipa::path(
    summary = "Attempts to solve a maze definition that is supplied by the caller",
    description = "This endpoint attempts to solve a maze definition that is supplied by the caller and, if successful, returns a maze solution containing the solution path",
    post,
    path = "/api/v1/solve-maze",
    request_body = Maze,
    responses(
        (status = 200, description = "Maze solved successfully", body = Solution),
        (status = 400, description = "Invalid request"),
        (status = 422, description = "Maze could not be solved")
    ),
    security(
        ()
    ),
    tags = ["v1"]
)]
#[post("/solve-maze/")]
pub async fn solve_maze(
    req: web::Json<Maze>,  
) -> Result<HttpResponse, Error> {
    let maze: Maze = req.into_inner();
    match maze.solve() {
        Ok(solution) => Ok(HttpResponse::Ok().json(solution)),
        Err(err) => Err(get_maze_solve_error(&err))
    }
}

#[cfg(test)]
mod tests {
    // **************************************************************************************************
    // Unit tests for API and documentation endpoints, via injection of MockMazeStore
    // **************************************************************************************************
    use crate::api::v1::handlers;
    use crate::api::v1::handlers::get_maze_solve_error_string;
    use crate::api::v1::openapi::ApiDocV1;

    use maze::{Definition, Maze, MazeError, Solution, Path, Point};
    use storage::{SharedStore, Store, StoreError, MazeItem};

    use actix_web::{http::StatusCode, test, web, App};
    use utoipa::OpenApi;
    use utoipa_rapidoc::RapiDoc;
    use utoipa_redoc::{Redoc, Servable};
    use utoipa_swagger_ui::SwaggerUi;
        
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

    struct MockMazeStore {
        items: HashMap<String, MazeStoreItem>,
    }

    impl MockMazeStore {
        pub fn new(startup_content: StoreStartupContent) -> Self {
            MockMazeStore {
                items: new_item_map(startup_content)
            }
        }

        fn create_id_from_name(name: &str) -> String {
            format!("{}.json", name)
        }

    }

    impl Store for MockMazeStore {

       fn create_maze(&mut self, maze: &mut Maze) -> Result<(), StoreError> {
            let id = MockMazeStore::create_id_from_name(&maze.name);

            if let Some(_) = self.items.get(&id) {
                println!("{} already exists", id);
                return Err(StoreError::IdAlreadyExists(id.to_string()));
            }

            self.items.insert(
                id.to_string(),
                MazeStoreItem {
                    id: id,
                    name: maze.name.to_string(),
                    maze: maze.clone(),
            });

            Ok(())
        }

        fn delete_maze(&mut self, id: &str) -> Result<(), StoreError> {
            if let Some(_) = self.items.remove(id) {
                Ok(())                
            } else {
                Err(StoreError::IdNotFound(id.to_string()))
            }
        }

        fn update_maze(&mut self, maze: &mut Maze) -> Result<(), StoreError> {
            if let Some(_) = self.items.get(&maze.id) {
                self.items.insert(
                    maze.id.to_string(),
                    MazeStoreItem {
                        id: maze.id.to_string(),
                        name: maze.name.to_string(),
                        maze: maze.clone(),
                });
                return Ok(());
            }
            Err(StoreError::IdNotFound(maze.id.to_string()))
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

    #[derive(Clone)]
    enum StoreStartupContent {
        Empty,
        OneMaze,
        TwoMazes,
        ThreeMazes,
        SolutionTestMazes,
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

    fn new_solve_test_maze(id: &str, name: &str, with_start: bool, with_finish: bool, with_block: bool) -> Maze {
        let start_char:char = if with_start {'S'} else {' '};
        let finish_char:char = if with_finish {'F'} else {' '};
        let block_char:char = if with_block {'W'} else {' '};

        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec![start_char, 'W', ' '],
            vec![' ', 'W', ' '],
            vec![' ', block_char, finish_char],
        ];
        let mut maze:Maze = Maze::new(Definition::from_vec(grid));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
    }

    fn get_solve_test_maze_solution() -> Solution {
        let path = Path {
            points: vec![
                Point { row: 0, col: 0 },
                Point { row: 1, col: 0 },
                Point { row: 2, col: 0 },
                Point { row: 2, col: 1 },
                Point { row: 2, col: 2 },
            ],
        };
        Solution::new(path)       
    } 


    fn new_solve_test_maze_store_item(id: &str, name: &str, with_start: bool, with_finish: bool, with_block: bool) -> MazeStoreItem {
        MazeStoreItem {
            id: id.to_string(),
            name: name.to_string(),
            maze: new_solve_test_maze(id, name, with_start, with_finish, with_block),
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
            StoreStartupContent::SolutionTestMazes => {
                result = vec![
                    new_solve_test_maze_store_item("solvable.json", "solvable", true, true, false),
                    new_solve_test_maze_store_item("no_start.json", "no_start", false, true, false),
                    new_solve_test_maze_store_item("no_finish.json", "no_finish", true, false, false),
                    new_solve_test_maze_store_item("no_solution.json", "no_solution", true, true, true),
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

    fn new_shared_mock_maze_store(startup_content: StoreStartupContent) -> SharedStore {
        Arc::new(RwLock::new(Box::new(MockMazeStore::new(startup_content))))
    }

    fn configure_mock_app(app: &mut web::ServiceConfig, startup_content: StoreStartupContent) {
        let mock_store = new_shared_mock_maze_store(startup_content);
    
        app.app_data(web::Data::new(mock_store.clone()))
            .service(
                web::scope("/api/v1")
                    .service(handlers::get_maze_list)
                    .service(handlers::create_maze)
                    .service(handlers::get_maze)
                    .service(handlers::update_maze)
                    .service(handlers::delete_maze)
                    .service(handlers::get_maze_solution)
                    .service(handlers::solve_maze)
                )
                .service(SwaggerUi::new("api-docs/v1/swagger-ui/{_:.*}").url("/api-docs/v1/openapi.json", ApiDocV1::openapi()))
                .service(Redoc::with_url("/api-docs/v1/redoc", ApiDocV1::openapi()))
                .service(RapiDoc::new("/api-docs/v1/openapi.json").path("/api-docs/v1/rapidoc"));
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

    async fn run_create_maze_test(
        startup_content: StoreStartupContent, 
        maze: Maze,
        expected_status_code: StatusCode, 
        ) {
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, startup_content.clone())),
        )
        .await;

        // Create
        let url = format!("/api/v1/mazes/");
        let req = test::TestRequest::post()
            .uri(&url)
            .set_json(maze.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            let mut maze_copy = maze.clone();
            maze_copy.id = MockMazeStore::create_id_from_name(&maze.name);
            assert_eq!(maze_copy, response_maze);        
        }
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
                Some(value) => { assert_eq!(maze, value); }        
                None => { panic!("No maze comparison value provided for get_maze() test!"); }
            }
        }
    }

    async fn run_update_maze_test(
            startup_content: StoreStartupContent, 
            id: &str, 
            maze: Maze,
            expected_status_code: StatusCode, 
        ) {
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, startup_content.clone())),
        )
        .await;

        // Create
        let url = format!("/api/v1/mazes/{}", id);
        let req = test::TestRequest::put()
            .uri(&url)
            .set_json(maze.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            assert_eq!(maze, response_maze);        
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

    async fn validate_solution_response(
        context: &str,
        resp: actix_web::dev::ServiceResponse,
        expected_status_code: StatusCode,
        expected_solution: Option<Solution>,
        expected_err_message: Option<String>
    ) {
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Confirm and validate solution response
            let body = test::read_body(resp).await;
            let solution: Solution = serde_json::from_slice(&body).expect("failed to deserialize response");
             match expected_solution {
                Some(value) => { assert_eq!(solution, value);}        
                None => { panic!("{}", format!("No maze solution comparison value provided for {} test!", context)); }
            }
        }
        else {
            match expected_err_message {
                Some(value) => { 
                    // Validate error response
                    let body = test::read_body(resp).await;
                    let error_message = String::from_utf8(body.to_vec()).expect("Failed to parse body as UTF-8");
                    assert_eq!(error_message, value);
                }        
                None => { panic!("{}", format!("No maze solution error message provided for {} test!", context)); }
            }
        }
    }

    fn get_no_start_cell_error_str() -> String {
        get_maze_solve_error_string(&MazeError::new("no start cell found within maze".to_string()))
    }

    fn get_no_finish_cell_error_str() -> String {
        get_maze_solve_error_string(&MazeError::new("no finish cell found within maze".to_string()))
    }

    fn get_no_solution_error_str() -> String {
        get_maze_solve_error_string(&MazeError::new("no solution found".to_string()))    
    }

    async fn run_get_maze_solution_test(
        startup_content: StoreStartupContent, 
        id: &str, 
        expected_status_code: StatusCode,
        expected_solution: Option<Solution>,
        expected_err_message: Option<String>
        ) {
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, startup_content.clone())),
        )
        .await;
        
        let url = format!("/api/v1/mazes/{}/solution", id);
        let req = test::TestRequest::get().uri(&url).to_request();
        let resp = test::call_service(&app, req).await;

        validate_solution_response("get_maze_solution()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_solve_maze_test(
        maze: Maze,
        expected_status_code: StatusCode,
        expected_solution: Option<Solution>,
        expected_err_message: Option<String>
        ) {
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, StoreStartupContent::Empty)),
        )
        .await;

        let url = format!("/api/v1/solve-maze/");
        let req = test::TestRequest::post()
            .uri(&url)
            .set_json(maze.clone())
            .to_request();
        let resp = test::call_service(&app, req).await;

        validate_solution_response("solve_maze()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_get_url_test(
        url: &str
        ) {
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, StoreStartupContent::Empty)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(url)
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

 
    #[actix_web::test]
    async fn test_get_mazes_with_no_mazes() {
        run_get_mazes_test(StoreStartupContent::Empty).await;
    }

    #[actix_web::test]
    async fn test_get_mazes_with_one_maze() {
        run_get_mazes_test(StoreStartupContent::OneMaze).await;
    }

    #[actix_web::test]
    async fn test_get_mazes_with_two_mazes_that_require_sorting() {
        run_get_mazes_test(StoreStartupContent::TwoMazes).await;
    }

    #[actix_web::test]
    async fn test_get_mazes_with_three_mazes_that_require_sorting() {
        run_get_mazes_test(StoreStartupContent::ThreeMazes).await;
    }

    #[actix_web::test]
    async fn test_create_maze_that_does_not_exist() {
        run_create_maze_test(StoreStartupContent::ThreeMazes, new_solvable_maze("", "maze_d"), StatusCode::CREATED).await;
    }

    #[actix_web::test]
    async fn test_create_maze_that_already_exists() {
        run_create_maze_test(StoreStartupContent::ThreeMazes, new_solvable_maze("", "maze_a"), StatusCode::CONFLICT).await;
    }

    #[actix_web::test]
    async fn test_get_maze_that_exists() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_get_maze_test(StoreStartupContent::ThreeMazes, id, StatusCode::OK, Some(new_solvable_maze(id, name))).await;
    }

    #[actix_web::test]
    async fn test_update_maze_that_exists() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(StoreStartupContent::ThreeMazes, id, new_solvable_maze(id, name), StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn test_update_maze_that_does_not_exist() {
        let id = "maze_d.json";
        let name = "maze_d";
        run_update_maze_test(StoreStartupContent::ThreeMazes, id, new_solvable_maze(id, name), StatusCode::NOT_FOUND).await;
    }

    #[actix_web::test]
    async fn test_update_maze_with_mismatching_id() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(StoreStartupContent::ThreeMazes, id, new_solvable_maze("some_other_id", name), StatusCode::BAD_REQUEST).await;
    }

    #[actix_web::test]
    async fn test_get_maze_that_does_not_exist() {
        run_get_maze_test(StoreStartupContent::ThreeMazes, "does_not_exist.json", StatusCode::NOT_FOUND, None).await;
    }

    #[actix_web::test]
    async fn test_delete_maze_that_exists() {
        run_delete_maze_test(StoreStartupContent::ThreeMazes, "maze_a.json", StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn test_delete_maze_that_does_not_exist() {
        run_delete_maze_test(StoreStartupContent::ThreeMazes, "does_not_exist.json", StatusCode::NOT_FOUND).await;
    }

    #[actix_web::test]
    async fn test_get_maze_solution_that_should_succeed() {
        run_get_maze_solution_test(
            StoreStartupContent::SolutionTestMazes, "solvable.json", StatusCode::OK, 
            Some(get_solve_test_maze_solution()), None
        ).await;
    }

    #[actix_web::test]
    async fn test_get_maze_solution_should_fail_with_no_start() {
        run_get_maze_solution_test(
            StoreStartupContent::SolutionTestMazes, "no_start.json", StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_start_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_get_maze_solution_should_fail_with_no_finish() {
        run_get_maze_solution_test(
            StoreStartupContent::SolutionTestMazes, "no_finish.json", StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_get_maze_solution_should_fail_with_no_solution() {
        run_get_maze_solution_test(
            StoreStartupContent::SolutionTestMazes, "no_solution.json", StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_solution_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_solve_maze_that_should_succeed() {
        run_solve_maze_test(
            new_solve_test_maze("", "", true, true, false), 
            StatusCode::OK, 
            Some(get_solve_test_maze_solution()), 
            None
        ).await;
    }

    #[actix_web::test]
    async fn test_solve_maze_should_fail_with_no_start() {
        run_solve_maze_test(
            new_solve_test_maze("", "", false, true, false), 
            StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_start_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_solve_maze_should_fail_with_no_finish() {
        run_solve_maze_test(
            new_solve_test_maze("", "", true, false, false), 
            StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_solve_maze_should_fail_with_no_solution() {
        run_solve_maze_test(
            new_solve_test_maze("", "", true, true, true), 
            StatusCode::UNPROCESSABLE_ENTITY, None, 
            Some(get_no_solution_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn test_load_swagger_ui_page() {
        run_get_url_test("/api-docs/v1/swagger-ui/").await;
    }

    #[actix_web::test]
    async fn test_load_openapi_json() {
        run_get_url_test("/api-docs/v1/openapi.json").await;
    }

    #[actix_web::test]
    async fn test_load_redoc_page() {
        run_get_url_test("/api-docs/v1/redoc").await;
    }

    #[actix_web::test]
    async fn test_load_rapidoc_page() {
        run_get_url_test("/api-docs/v1/rapidoc").await;
    }
}
