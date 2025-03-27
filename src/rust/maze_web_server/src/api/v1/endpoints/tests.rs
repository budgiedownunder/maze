#[cfg(test)]
mod test_definitions {
    // **************************************************************************************************
    // Unit tests for API and documentation endpoints, via injection of MockStore
    // **************************************************************************************************
    use crate::api::v1::endpoints::handlers;
    use crate::api::v1::endpoints::handlers::get_maze_solve_error_string;
    use crate::api::v1::endpoints::handlers::{CreateUserRequest, UpdateUserRequest, UserItem};
    use crate::middleware::auth::auth_middleware;
    use crate::api::v1::openapi::ApiDocV1;
    
    use data_model::{Maze, MazeDefinition, MazePoint, User};
    use maze::{Error as MazeError, MazePath, MazeSolution};
    use storage::{Error as StoreError, SharedStore, Store, store::MazeStore, store::UserStore, store::Manage, MazeItem, validation::validate_user_fields};

    use actix_web::{http::StatusCode, test, dev::{Service, ServiceResponse}, web, App, middleware::from_fn, Error};

    use actix_http;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    use utoipa::OpenApi;
    use utoipa_rapidoc::RapiDoc;
    use utoipa_redoc::{Redoc, Servable};
    use utoipa_swagger_ui::SwaggerUi;
    use uuid::Uuid;

    const ADMIN_USERNAME_PREFIX:&str = "admin_";
    const USERNAME_PREFIX:&str = "user_";
    const INVALID_USERNAME:&str = "INVALID_USERNAME";

    const NEW_ADMIN_USERNAME_1: &str = "new_admin_1";
    const NEW_USERNAME_1: &str = "new_user_1";

    const VALID_ADMIN_USERNAME_1:&str = "admin_1";
    const VALID_ADMIN_USERNAME_2:&str = "admin_2";
    const VALID_USERNAME_1:&str = "user_1";
    const VALID_USERNAME_2:&str = "user_2";

    /**************/
    /* Mock maze  */
    /**************/
    #[derive(Clone, Debug)]
    struct MockMaze {
        id: String,
        name: String,
        maze: Maze,
    }

    impl MockMaze {
        pub fn to_maze_item(&self, include_definitions: bool) -> MazeItem {
            MazeItem {
                id: self.id.clone(),
                name: self.name.clone(),
                definition: if include_definitions {
                    Some(serde_json::to_string(&self.maze.definition).expect("Failed to serialize"))
                } else {
                    None
                },
            }
        }

        fn create_id_from_name(name: &str) -> String {
            format!("{}.json", name)
        }
    }

    /**************/
    /* Mock user  */
    /**************/
    #[derive(Clone, Debug)]
   struct MockUser {
        user: User,
        mazes: HashMap<String, MockMaze>,
    }

    impl MockUser {
        fn default() -> MockUser {
            MockUser {
                user: User::default(),
                mazes: HashMap::new(),
            }
        }

        fn to_user_item(&self) -> UserItem {
            UserItem {
                id: self.user.id,
                is_admin: self.user.is_admin,
                username: self.user.username.clone(),
                full_name: self.user.full_name.clone(),
                email: self.user.email.clone(),
            }  
        }          
        
        fn new_from_user(user: &User) -> Self {
            let mut new_user = user.clone();
            new_user.id = User::new_id();
            new_user.api_key = User::new_api_key();
            MockUser {
                user: new_user,
                mazes: HashMap::new(),
            }
        }        
    } 

    /**************/
    /* Mock store */
    /**************/
    struct MockStore {
        users: HashMap<Uuid, MockUser>,
    }

    impl MockStore {
        pub fn new(user_defs: &Vec<UserDefinition>) -> Self {
            MockStore {
                users: new_users_map(user_defs)
            }
        }

        fn get_mock_user(&self, id: Uuid) -> Result<&MockUser, StoreError> {
            if let Some(mock_user) = self.users.get(&id) {
                return Ok(mock_user);
            }
            Err(StoreError::UserIdNotFound(id.to_string()))
        }

        fn get_mock_user_mut(&mut self, id: Uuid) -> Result<&mut MockUser, StoreError> {
            if let Some(mock_user) = self.users.get_mut(&id) {
                return Ok(mock_user);
            }
            Err(StoreError::UserIdNotFound(id.to_string()))
        }

        /// Find the api key to use for a given username. If the username does not exist,
        /// return an invalid key to simulate an invalid access attempt 
        fn get_api_key_to_use(&self, caller_username: Option<&str>) -> Uuid {
            if let Some(username) = caller_username {
                if let Ok(user) = self.find_user_by_name(username) {
                    return user.api_key;
                }
            }
            User::new_api_key()
        }

        /// Locates a user in a user map by their username
        fn find_user_by_name_in_map(user_map: &HashMap<Uuid, MockUser>, username: &str, ignore_id: Uuid) -> Result<User, StoreError> {
            for v in user_map.values() {
                if v.user.username == username && v.user.id != ignore_id{
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }    

        /// Locates a user id in a user map by their username
        fn find_user_id_by_name_in_map(user_map: &HashMap<Uuid, MockUser>, username: &str, ignore_id: Uuid) -> Uuid {
            match MockStore::find_user_by_name_in_map(user_map, username, ignore_id) {
                Ok(user) => user.id,
                _ => Uuid::nil(),
            }
        }

        /// Locates a user id in a user map by their username - return nil if it is not found
        fn find_user_id_by_name(&self, username: &str, ignore_id: Uuid) -> Uuid {
            match MockStore::find_user_by_name_in_map(&self.users, username, ignore_id) {
                Ok(user) => user.id,
                _ => Uuid::nil(),
            }
        }

        // Checks whether a given username exists in the file store
        fn user_name_exists(&self, name: &str, ignore_id: Uuid) -> bool {
            self.find_user_id_by_name(name, ignore_id) != Uuid::nil()
        }

        /// Locates a user by their email within the store
        fn find_user_by_email(&self, email: &str, ignore_id: Uuid) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user.email == email && v.user.id != ignore_id {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }

        // Checks whether a given user email exists in the file store
        fn user_email_exists(&self, email: &str, ignore_id: Uuid) -> bool {
            self.find_user_by_email(email, ignore_id).is_ok()
        }

        // Validate user content
        fn validate_user(&self, user: &User, ignore_id: Uuid) -> Result<(), StoreError> {
            validate_user_fields(user)?;
            if user.password_hash.is_empty() {
                return Err(StoreError::UserPasswordMissing());
            }
            if self.user_name_exists(&user.username, ignore_id) {
                return Err(StoreError::UserNameExists());
            }
            if !user.email.is_empty() && self.user_email_exists(&user.email, ignore_id) {
                return Err(StoreError::UserEmailExists());
            }
            Ok(())
        }        
    }

    impl MazeStore for MockStore {

        fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?;
            let id = MockMaze::create_id_from_name(&maze.name);

            if mock_user.mazes.contains_key(&id) {
                return Err(StoreError::MazeIdExists(id.to_string()));
            }

            maze.id = id.clone();

            mock_user.mazes.insert(
                id.to_string(),
                MockMaze {
                    id,
                    name: maze.name.to_string(),
                    maze: maze.clone(),
            });

            Ok(())
        }

        fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?;
            if mock_user.mazes.remove(id).is_some() {
                Ok(())
            } else {
                Err(StoreError::MazeIdNotFound(id.to_string()))
            }
        }

        fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?;
            if mock_user.mazes.contains_key(&maze.id) {
                mock_user.mazes.insert(
                    maze.id.to_string(),
                    MockMaze {
                        id: maze.id.to_string(),
                        name: maze.name.to_string(),
                        maze: maze.clone(),
                });
                return Ok(());
            }
            Err(StoreError::MazeIdNotFound(maze.id.to_string()))
        }

        fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, StoreError> {
            let mock_user = self.get_mock_user(owner.id)?;
            if let Some(mock_maze) = mock_user.mazes.get(id) {
                return Ok(mock_maze.maze.clone());
            }
            Err(StoreError::MazeIdNotFound(id.to_string()))
        }

        fn find_maze_by_name(&self, _owner: &User, _name: &str) -> Result<MazeItem, StoreError> {
            Err(StoreError::Other("Mock interface not implemented".to_string()))
        }

        fn get_maze_items(&self, owner: &User, include_definitions: bool) -> Result<Vec<MazeItem>, StoreError> {
            let mock_user = self.get_mock_user(owner.id)?;
            let mut items: Vec<MazeItem> = maze_items_from_map(&mock_user.mazes, include_definitions);
            items.sort_by_key(|item| item.name.clone());
            Ok(items)
        }
    }

    impl UserStore for MockStore {
        /// Adds the default admin user to the store if it doesn't already exist, else returns it
        fn init_default_admin_user(&mut self, _username: &str, _password_hash: &str) -> Result<User, StoreError> {
            Err(StoreError::Other("init_default_admin_user() not implemented for MockStore".to_string()))
        }
        /// Adds a new user to the store and sets the allocated `id` within the user object
        fn create_user(&mut self, user: &mut User) -> Result<(), StoreError> {
            let mock_user = MockUser::new_from_user(user);
            user.id = mock_user.user.id;
            self.validate_user(user, Uuid::nil())?;
            self.users.insert(mock_user.user.id, mock_user);
            Ok(())
        }
        /// Deletes a user from the store
        fn delete_user(&mut self, id: Uuid) -> Result<(), StoreError> {
            if self.users.remove(&id).is_some() {
                Ok(())
            } else {
                Err(StoreError::UserIdNotFound(id.to_string()))
            }
        }
        /// Updates a user within the store
        fn update_user(&mut self, user: &mut User) -> Result<(), StoreError> {
            self.validate_user(user, user.id)?;
            let mock_user = self.get_mock_user_mut(user.id)?;
            mock_user.user = user.clone();
            Ok(())
        }
        /// Loads a user from the store
        fn get_user(&self, id: Uuid) -> Result<User, StoreError> {
            if let Some(mock_user) = self.users.get(&id) {
                return Ok(mock_user.user.clone());
            }
            Err(StoreError::UserIdNotFound(id.to_string()))
        }
        /// Locates a user by their username within the store
        fn find_user_by_name(&self, name: &str) -> Result<User, StoreError> {
            MockStore::find_user_by_name_in_map(&self.users, name, Uuid::nil())
        }
        /// Locates a user by their api key within the store
        fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user.api_key == api_key {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }
        /// Returns the list of users within the store, sorted
        /// alphabetically by username in ascending order
        fn get_users(&self) -> Result<Vec<User>, StoreError> {
            let mut users: Vec<User> = self.users.values()
                .map( |value| value.user.clone())
                .collect();

            users.sort_by_key(|user| user.username.clone());
            Ok(users)
        }
    }

    impl Manage for MockStore {
        fn empty(&mut self) -> Result<(), StoreError> {
            self.users = HashMap::new();
            Ok(())
        }
    }

    impl Store for MockStore {}

    /****************/
    /* Mock content */
    /****************/
    #[derive(Clone)]
    enum MazeContent {
        Empty,
        OneMaze,
        TwoMazes,
        ThreeMazes,
        SolutionTestMazes,
    }

    fn maze_store_mock_mazes_to_maze_items(from: Vec<MockMaze>, include_definitions: bool) -> Vec<MazeItem> {
        from.iter()
            .map( |value| value.to_maze_item(include_definitions))
            .collect()
    }

    fn mazes_to_map(mazes: &Vec<MockMaze>) -> HashMap<String, MockMaze> {
        let mut map: HashMap<String, MockMaze> = HashMap::new();
        for maze in mazes {
            map.insert(maze.id.clone(), maze.clone());
        }
        map
    }

    fn maze_items_from_map(from: &HashMap<String, MockMaze>, include_definitions: bool) -> Vec<MazeItem> {
        from.iter()
            .map(|(_key, value) | MazeItem {
                    id: value.id.clone(),
                    name: value.name.clone(),
                    definition: if include_definitions {
                        Some(serde_json::to_string(&value.maze.definition).expect("Failed to serialize"))
                    } else {
                        None
                    },
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
        let mut maze:Maze = Maze::new(MazeDefinition::from_vec(grid));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
    }

    fn new_solvable_maze_store_item(id: &str, name: &str) -> MockMaze {
        MockMaze {
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
        let mut maze:Maze = Maze::new(MazeDefinition::from_vec(grid));
        maze.id = id.to_string();
        maze.name = name.to_string();
        maze
    }

    fn get_solve_test_maze_solution() -> MazeSolution {
        let path = MazePath {
            points: vec![
                MazePoint { row: 0, col: 0 },
                MazePoint { row: 1, col: 0 },
                MazePoint { row: 2, col: 0 },
                MazePoint { row: 2, col: 1 },
                MazePoint { row: 2, col: 2 },
            ],
        };
        MazeSolution::new(path)
    }


    fn new_solve_test_maze_store_item(id: &str, name: &str, with_start: bool, with_finish: bool, with_block: bool) -> MockMaze {
        MockMaze {
            id: id.to_string(),
            name: name.to_string(),
            maze: new_solve_test_maze(id, name, with_start, with_finish, with_block),
        }
    }

    fn get_maze_content(maze_content: MazeContent, sort_asc: bool) -> Vec<MockMaze> {
        let mut result: Vec<MockMaze>;
        match maze_content {
            MazeContent::Empty => {
                result = Vec::new();
            }
            MazeContent::OneMaze => {
                result = vec![
                    new_solvable_maze_store_item("maze_a.json", "maze_a")
                ]
            }
            MazeContent::TwoMazes => {
                result = vec![
                    new_solvable_maze_store_item("maze_b.json", "maze_b"),
                    new_solvable_maze_store_item("maze_a.json", "maze_a"),
                ]
            }
            MazeContent::ThreeMazes => {
                result = vec![
                    new_solvable_maze_store_item("maze_c.json", "maze_c"),
                    new_solvable_maze_store_item("maze_b.json", "maze_b"),
                    new_solvable_maze_store_item("maze_a.json", "maze_a"),
                ]
            }
            MazeContent::SolutionTestMazes => {
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

    fn new_mazes_map(maze_content: MazeContent) -> HashMap<String, MockMaze> {
        mazes_to_map(&get_maze_content(maze_content, false))
    }

    fn new_user(username: &str, is_admin: bool) -> User {
        let mut user = User::default();
        user.id = User::new_id();
        user.username = username.to_string();
        user.is_admin = is_admin;
        user.api_key = User::new_api_key();
        user.email = new_email(username);
        user.password_hash = "password_hash".to_string();
        user
    }

    fn new_email(username: &str) -> String {
        format!("{}@company.com", username)
    }

    #[derive(Clone)]
    struct UserDefinition {
        username: String,
        is_admin: bool,
        mazes: MazeContent,
    }

    fn append_user_defs(user_defs: &mut Vec<UserDefinition>, num: i32, is_admin: bool, mazes: &MazeContent) {
        let username_prefix = if is_admin { ADMIN_USERNAME_PREFIX } else { USERNAME_PREFIX};
        for i in 1..(num+1) {
            user_defs.push( UserDefinition {
                username: format!("{}{}", username_prefix, i),
                is_admin,
                mazes: mazes.clone(),
            });
        }
    }

    struct CreateUsersDef {
        num_admin_users: i32,
        num_users: i32,
        mazes: MazeContent,
    }

    impl CreateUsersDef {
        pub fn new(
            num_admin_users: i32,
            num_users: i32,
            mazes: MazeContent
        ) -> Self {
            CreateUsersDef {
                num_admin_users,
                num_users,
                mazes: mazes.clone(),
            }
        }    
    }

    fn create_user_defs(def: &CreateUsersDef) -> Vec<UserDefinition> {
        let mut user_defs = vec![];
        append_user_defs(&mut user_defs, def.num_users, false, &def.mazes);
        append_user_defs(&mut user_defs, def.num_admin_users, true, &def.mazes);
        user_defs
    }

    fn new_mock_user(user_def: &UserDefinition) -> MockUser {
        let user =  new_user(&user_def.username, user_def.is_admin);
        MockUser {
            user,
            mazes: new_mazes_map(user_def.mazes.clone()),
        }
    }

    fn new_shared_mock_maze_store(mock_store: MockStore) -> SharedStore {
        Arc::new(RwLock::new(Box::new(mock_store)))
    }

    fn new_users_map(user_defs:&Vec<UserDefinition>) -> HashMap<Uuid, MockUser> {
        let mut map: HashMap<Uuid, MockUser> = HashMap::new();
        for user_def in user_defs {
            let mock_user = new_mock_user(user_def);
            map.insert(mock_user.user.id, mock_user);
        }
        map
    }

    fn maze_store_mock_users_to_user_items(from: &HashMap<Uuid, MockUser>) -> Vec<UserItem> {
        let mut users: Vec<UserItem> = from.values()
            .map( |value| value.to_user_item())
            .collect();

       users.sort_by_key(|user| user.username.clone());
       users
    }

    fn configure_mock_app(app: &mut web::ServiceConfig, mock_store: SharedStore)  {

        app.app_data(web::Data::new(mock_store.clone()))
            .service(
                web::scope("/api/v1")
                    .wrap(from_fn(auth_middleware))
                    // Mazes
                    .service(handlers::get_mazes)
                    .service(handlers::create_maze)
                    .service(handlers::delete_maze)
                    .service(handlers::get_maze)
                    .service(handlers::get_maze_solution)
                    .service(handlers::solve_maze)
                    .service(handlers::update_maze)
                    // Users
                    .service(handlers::get_users)
                    .service(handlers::create_user)
                    .service(handlers::delete_user)
                    .service(handlers::get_user)
                    .service(handlers::update_user)
                )
                .service(SwaggerUi::new("api-docs/v1/swagger-ui/{_:.*}").url("/api-docs/v1/openapi.json", ApiDocV1::openapi()))
                .service(Redoc::with_url("/api-docs/v1/redoc", ApiDocV1::openapi()))
                .service(RapiDoc::new("/api-docs/v1/openapi.json").path("/api-docs/v1/rapidoc"));
    }

    fn create_test_get_request(url: &str, api_key: Option<Uuid>) -> actix_http::Request {
        let mut req = test::TestRequest::get().uri(url);

        if let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }
        req.to_request()
    }

    fn create_test_post_request<T: serde::Serialize>(url: &str, api_key: Option<Uuid>, body_obj: &T) -> actix_http::Request {
        let mut req = test::TestRequest::post().uri(url);

        if let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }
        req.set_json(body_obj).to_request()
    }

    fn create_test_put_request<T: serde::Serialize>(url: &str, api_key: Option<Uuid>, body_obj: &T) -> actix_http::Request {
        let mut req = test::TestRequest::put().uri(url);

        if let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }
        req.set_json(body_obj).to_request()
    }

    fn create_test_delete_request(url: &str, api_key: Option<Uuid>) -> actix_http::Request {
        let mut req = test::TestRequest::delete().uri(url);

        if let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }
        req.to_request()
    }

    fn create_shared_mock_store(
        user_defs:&Vec<UserDefinition>,
        caller_username: Option<&str>
     ) -> (SharedStore, HashMap<Uuid, MockUser>, Uuid) {
        let mock_store = MockStore::new(user_defs);
        let mock_users = mock_store.users.clone();
        let api_key = mock_store.get_api_key_to_use(caller_username);
        let shared_mock_store = new_shared_mock_maze_store(mock_store);
        (shared_mock_store, mock_users, api_key)
    }

    async fn create_test_app(
        user_defs: &Vec<UserDefinition>,
        caller_username: Option<&str>
    ) -> (impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>, HashMap<Uuid, MockUser>, Uuid) {
        let (shared_mock_store, mock_users, api_key) = create_shared_mock_store(user_defs, caller_username);
        let app = test::init_service(
            App::new().configure(|cfg| configure_mock_app(cfg, shared_mock_store.clone())),
        )
        .await;

        (app, mock_users, api_key)
    }

    async fn run_get_users_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        expected_status_code: StatusCode,
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, mock_users, api_key) = create_test_app(&user_defs, caller_username).await;
        let path_str = "/api/v1/users".to_string();
        let req = create_test_get_request(&path_str, Some(api_key));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);
        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let user_items: Vec<UserItem> = serde_json::from_slice(&body).expect("failed to deserialize response");
            let expected_user_items = maze_store_mock_users_to_user_items(&mock_users);
            assert_eq!(user_items, expected_user_items);
        } 
    }

    impl CreateUserRequest {
        pub fn new(
            is_admin: bool,
            username: &str,
            full_name: &str,
            email: &str,
            password: &str
        ) -> CreateUserRequest {
            CreateUserRequest {
                is_admin,
                username: username.to_string(),
                full_name: full_name.to_string(),
                email: email.to_string(),
                password: password.to_string(),
            }
        }

        pub fn to_user_item(&self) -> UserItem {
            UserItem {
                id: Uuid::nil(),
                is_admin: self.is_admin,
                username: self.username.clone(),
                full_name: self.full_name.clone(),
                email: self.email.clone(),
            }            
        }

    }    

    fn create_password(blank: bool) -> String {
        if blank {
            "".to_string()
        } else {
            "dummy_password".to_string()
        }
    }

    fn new_create_user_request(is_admin: bool, username: &str, email: Option<&str>, blank_password: bool) -> CreateUserRequest {
        let email_use = if let Some(s) = email {
            s
        } else {
            &new_email(username)
        };

        CreateUserRequest::new(is_admin, username, 
            &format!("{} full name", username), 
            email_use, 
            &create_password(blank_password) 
        )    
    }

    async fn run_create_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        create_req: &CreateUserRequest,
        expected_status_code: StatusCode,
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, _, api_key) = create_test_app(&user_defs, caller_username).await;
        let url = "/api/v1/users".to_string();
        let req = create_test_post_request(&url, Some(api_key), &create_req);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::CREATED {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize response");
            let mut expected_user_response = create_req.to_user_item();
            expected_user_response.id = response_user.id; 
            assert_eq!(expected_user_response, response_user);
        }
    }

    async fn run_get_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        target_username: &str,
        expected_status_code: StatusCode,
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, mock_users, api_key) = create_test_app(&user_defs, caller_username).await;
        let id = MockStore::find_user_id_by_name_in_map(&mock_users, target_username, Uuid::nil());
        let url = format!("/api/v1/users/{}", id);
        let req = create_test_get_request(&url, Some(api_key));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize response");
            let dummy_user = MockUser::default();
            let expected_user = mock_users.get(&id).unwrap_or(&dummy_user);
            let expected_user_response = expected_user.to_user_item();
            assert_eq!(expected_user_response, response_user);
        }
    }

    impl UpdateUserRequest {
        pub fn new(
            is_admin: bool,
            username: &str,
            full_name: &str,
            email: &str
        ) -> UpdateUserRequest {
            UpdateUserRequest {
                is_admin,
                username: username.to_string(),
                full_name: full_name.to_string(),
                email: email.to_string()
            }
        }

        pub fn to_user_item(&self) -> UserItem {
            UserItem {
                id: Uuid::nil(),
                is_admin: self.is_admin,
                username: self.username.clone(),
                full_name: self.full_name.clone(),
                email: self.email.clone(),
            }            
        }

    }    

    fn new_update_user_request(is_admin: bool, username: &str, email: Option<&str>) -> UpdateUserRequest {
        let email_use = if let Some(s) = email {
            s
        } else {
            &new_email(username)
        };

        UpdateUserRequest::new(is_admin, username, 
            &format!("Updated {} full name", username), 
            email_use
        )    
    }    

    async fn run_update_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        target_username: &str,
        update_req: &UpdateUserRequest,
        expected_status_code: StatusCode,
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, mock_users, api_key) = create_test_app(&user_defs, caller_username).await;
        let id = MockStore::find_user_id_by_name_in_map(&mock_users, target_username, Uuid::nil());
        let url = format!("/api/v1/users/{}", id);
        let req = create_test_put_request(&url, Some(api_key), &update_req);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize response");
            let mut expected_response_user = update_req.to_user_item();
            expected_response_user.id = response_user.id;
            assert_eq!(expected_response_user, response_user);
        }
    }    

    async fn run_delete_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        target_username: &str,
        expected_status_code: StatusCode
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, mock_users, api_key) = create_test_app(&user_defs, caller_username).await;
        let id = MockStore::find_user_id_by_name_in_map(&mock_users, target_username, Uuid::nil());
        let url = format!("/api/v1/users/{}", id);
        let req = create_test_delete_request(&url, Some(api_key));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            if Some(target_username) == caller_username {
                return;
            }

            // Confirm it has been deleted
            let url2 = format!("/api/v1/users/{}", id);
            let req2 = create_test_get_request(&url2, Some(api_key));
            let resp2 = test::call_service(&app, req2).await;
            assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
        }
    }

    async fn run_get_mazes_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        include_definitions: bool,
        expected_maze_content:MazeContent
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, _, api_key) = create_test_app(&user_defs, caller_username).await;
        let path_str = format!("/api/v1/mazes?includeDefinitions={}", include_definitions);
        let req = create_test_get_request(&path_str, Some(api_key));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let maze_items: Vec<MazeItem> = serde_json::from_slice(&body).expect("failed to deserialize response");
        assert_eq!(
            maze_items,
            maze_store_mock_mazes_to_maze_items(get_maze_content(expected_maze_content, true), include_definitions)
        );
    }

    async fn run_create_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        maze: Maze,
        expected_status_code: StatusCode,
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, _, api_key) = create_test_app(&user_defs, caller_username).await;
        let url = "/api/v1/mazes".to_string();
        let req = create_test_post_request(&url, Some(api_key), &maze);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::CREATED {
            let body = test::read_body(resp).await;
            let response_maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            let mut maze_copy = maze.clone();
            maze_copy.id = MockMaze::create_id_from_name(&maze.name);
            assert_eq!(maze_copy, response_maze);
        }
    }

    async fn run_get_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        id: &str,
        expected_status_code: StatusCode,
        expected_maze: Option<Maze>
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, _, api_key) = create_test_app(&user_defs, caller_username).await;
        let url = format!("/api/v1/mazes/{}", id);
        let req = create_test_get_request(&url, Some(api_key));
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
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        id: &str,
        maze: Maze,
        expected_status_code: StatusCode,
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, _, api_key) = create_test_app(&user_defs, caller_username).await;
        let url = format!("/api/v1/mazes/{}", id);
        let req = create_test_put_request(&url, Some(api_key), &maze);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            assert_eq!(maze, response_maze);
        }
    }

    async fn run_delete_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        id: &str,
        expected_status_code: StatusCode
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, _, api_key) = create_test_app(&user_defs, caller_username).await;
        let url = format!("/api/v1/mazes/{}", id);
        let req = create_test_delete_request(&url, Some(api_key));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Confirm it has been deleted
            let url2 = format!("/api/v1/mazes/{}", id);
            let req2 = create_test_get_request(&url2, Some(api_key));
            let resp2 = test::call_service(&app, req2).await;
            assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
        }
    }

    async fn validate_solution_response(
        context: &str,
        resp: actix_web::dev::ServiceResponse,
        expected_status_code: StatusCode,
        expected_solution: Option<MazeSolution>,
        expected_err_message: Option<String>
    ) {
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Confirm and validate solution response
            let body = test::read_body(resp).await;
            let solution: MazeSolution = serde_json::from_slice(&body).expect("failed to deserialize response");
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
        get_maze_solve_error_string(&MazeError::Solve("no start cell found within maze".to_string()))
    }

    fn get_no_finish_cell_error_str() -> String {
        get_maze_solve_error_string(&MazeError::Solve("no finish cell found within maze".to_string()))
    }

    fn get_no_solution_error_str() -> String {
        get_maze_solve_error_string(&MazeError::Solve("no solution found".to_string()))
    }

    async fn run_get_maze_solution_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        id: &str,
        expected_status_code: StatusCode,
        expected_solution: Option<MazeSolution>,
        expected_err_message: Option<String>
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, _, api_key) = create_test_app(&user_defs, caller_username).await;
        let url = format!("/api/v1/mazes/{}/solution", id);
        let req = create_test_get_request(&url, Some(api_key));
        let resp = test::call_service(&app, req).await;

        validate_solution_response("get_maze_solution()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_solve_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        maze: Maze,
        expected_status_code: StatusCode,
        expected_solution: Option<MazeSolution>,
        expected_err_message: Option<String>
    ) {
        let user_defs = create_user_defs(create_users_def);
        let (app, _, api_key) = create_test_app(&user_defs, caller_username).await;
        let url = "/api/v1/solve-maze".to_string();
        let req = create_test_post_request(&url, Some(api_key), &maze);
        let resp = test::call_service(&app, req).await;

        validate_solution_response("solve_maze()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_get_url_test(
        url: &str
        ) {

        let (app, _, _) = create_test_app(&vec![], None).await;
        let req = create_test_get_request(url, None);
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
    /*********************************************************************/
    /* Endpoint tests                                                    */
    /*********************************************************************/
    /**********/
    /* Users  */
    /**********/
    // Get users
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_users_with_no_users_with_invalid_api_key() {
        run_get_users_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn cannot_get_users_with_one_non_admin_user_with_non_admin_caller() {
        run_get_users_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), Some(VALID_USERNAME_1), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_one_admin_user() {
        run_get_users_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_one_admin_and_one_non_admin_user() {
        run_get_users_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_ten_admin_and_five_non_admin_users() {
        run_get_users_test(&CreateUsersDef::new(10, 5, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_2), StatusCode::OK).await;
    }

    // Create user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_create_admin_user_with_invalid_api_key() {
        run_create_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None, false),
            StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_admin_user_with_admin_caller() {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None , false),
            StatusCode::CREATED).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username() {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            &new_create_user_request(true, "", None , false),
            StatusCode::BAD_REQUEST).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password() {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None , true),
            StatusCode::BAD_REQUEST).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_non_admin_caller() {
        run_create_user_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), Some(VALID_USERNAME_1), 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None, false),
            StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username() {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            &new_create_user_request(true, VALID_ADMIN_USERNAME_1, None , false), 
            StatusCode::CONFLICT).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email() {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            &new_create_user_request(true, VALID_ADMIN_USERNAME_2, Some(&new_email(VALID_ADMIN_USERNAME_1)), false), 
            StatusCode::CONFLICT).await;
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_create_non_admin_user_with_invalid_api_key() {
        run_create_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, 
            &new_create_user_request(false, NEW_USERNAME_1, None, false), 
            StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_non_admin_user_with_admin_caller() {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            &new_create_user_request(false, NEW_USERNAME_1, None, false),
            StatusCode::CREATED).await;
    }
 
    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_non_admin_caller() {
        run_create_user_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), Some(VALID_USERNAME_1), 
            &new_create_user_request(false, NEW_USERNAME_1, None, false),
            StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username() {
        run_create_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            &new_create_user_request(true, VALID_USERNAME_1, None, false),
            StatusCode::CONFLICT).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email() {
        run_create_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            &new_create_user_request(true, VALID_USERNAME_2, Some(&new_email(VALID_USERNAME_1)), false),
            StatusCode::CONFLICT).await;
    }

    // Get user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_user_that_exists_with_invalid_api_key() {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), None, 
                          VALID_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_get_user_that_exists_with_admin_caller() {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
                          VALID_USERNAME_1, StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn can_get_admin_user_that_exists_with_admin_caller() {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
                          VALID_ADMIN_USERNAME_1, StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_exists_with_non_admin_caller() {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_USERNAME_1), 
                          VALID_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_does_not_exist_with_admin_caller() {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
                          VALID_USERNAME_2, StatusCode::NOT_FOUND).await;
    }

    // Update user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_update_admin_user_with_invalid_api_key() {
        run_update_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, NEW_ADMIN_USERNAME_1,
            &new_update_user_request(true, NEW_ADMIN_USERNAME_1, None), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_update_admin_user_with_admin_caller() {
        run_update_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_ADMIN_USERNAME_1, &new_update_user_request(true, NEW_ADMIN_USERNAME_1, None),
            StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_non_admin_caller() {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_USERNAME_1), 
            VALID_ADMIN_USERNAME_1, &new_update_user_request(true, NEW_ADMIN_USERNAME_1, None),
            StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_missing_username() {
        run_update_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_ADMIN_USERNAME_1, &new_update_user_request(true, "", None),
            StatusCode::BAD_REQUEST).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_username() {
        run_update_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_ADMIN_USERNAME_1, &new_update_user_request(true, VALID_ADMIN_USERNAME_2, None),
            StatusCode::CONFLICT).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_email() {
        run_update_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_ADMIN_USERNAME_1, &new_update_user_request(true, VALID_ADMIN_USERNAME_1, Some(&new_email(VALID_ADMIN_USERNAME_2))),
            StatusCode::CONFLICT).await;
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_update_non_admin_user_with_invalid_api_key() {
        run_update_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, NEW_ADMIN_USERNAME_1,
            &new_update_user_request(false, NEW_ADMIN_USERNAME_1, None), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_update_non_admin_user_with_admin_caller() {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_USERNAME_1, &new_update_user_request(false, NEW_USERNAME_1, None),
            StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_missing_username() {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_USERNAME_1, &new_update_user_request(false, "", None),
            StatusCode::BAD_REQUEST).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_username() {
        run_update_user_test(&CreateUsersDef::new(1, 2, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_USERNAME_1, &new_update_user_request(false, VALID_USERNAME_2, None),
            StatusCode::CONFLICT).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_email() {
        run_update_user_test(&CreateUsersDef::new(1, 2, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_USERNAME_1, &new_update_user_request(false, VALID_USERNAME_1, Some(&new_email(VALID_USERNAME_2))),
            StatusCode::CONFLICT).await;
    }

    #[actix_web::test]
    async fn can_upgrade_non_admin_user_to_admin_with_admin_caller() {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_USERNAME_1, &new_update_user_request(true, VALID_USERNAME_1, None),
            StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn can_downgrade_admin_user_to_non_admin_with_admin_caller() {
        run_update_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), 
            VALID_ADMIN_USERNAME_1, &new_update_user_request(false, VALID_ADMIN_USERNAME_1, None),
            StatusCode::OK).await;
    }

    // Delete user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_delete_user_with_invalid_api_key() {
        run_delete_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None,
            NEW_ADMIN_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_admin_user_with_admin_caller() {
        run_delete_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1),
            VALID_ADMIN_USERNAME_2, StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn can_delete_last_admin_user_with_admin_caller() {
        run_delete_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1),
            VALID_ADMIN_USERNAME_1, StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_admin_user_with_admin_caller() {
        run_delete_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1),
            VALID_ADMIN_USERNAME_2, StatusCode::NOT_FOUND).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_non_admin_user_with_admin_caller() {
        run_delete_user_test(&CreateUsersDef::new(2, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1),
            VALID_USERNAME_1, StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_non_admin_user_with_admin_caller() {
        run_delete_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1),
            VALID_USERNAME_1, StatusCode::NOT_FOUND).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_admin_user_with_non_admin_caller() {
        run_delete_user_test(&CreateUsersDef::new(2, 1, MazeContent::Empty), Some(VALID_USERNAME_1),
            VALID_ADMIN_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_non_admin_user_with_non_admin_caller() {
        run_delete_user_test(&CreateUsersDef::new(2, 1, MazeContent::Empty), Some(VALID_USERNAME_1),
            VALID_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    /**********/
    /* Mazes  */
    /**********/

    // Get mazes
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_mazes_with_no_mazes_with_invalid_api_key() {
        run_get_mazes_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, false, MazeContent::Empty).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_no_mazes() {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), Some(VALID_USERNAME_1), false, MazeContent::Empty).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_without_definitions() {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::OneMaze), Some(VALID_USERNAME_1), false, MazeContent::OneMaze).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_with_defintions() {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::OneMaze), Some(VALID_USERNAME_1), true, MazeContent::OneMaze).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_without_definitions() {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::TwoMazes), Some(VALID_USERNAME_1), false, MazeContent::TwoMazes).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_with_definitions() {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::TwoMazes), Some(VALID_USERNAME_1), true, MazeContent::TwoMazes).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_without_definitions() {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), false, MazeContent::ThreeMazes).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_with_definitions() {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), true, MazeContent::ThreeMazes).await;
    }

    // Create maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_create_maze_that_does_not_exist_with_invalid_api_key() {
        run_create_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), new_solvable_maze("", "maze_d"), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_create_maze_that_does_not_exist() {
        run_create_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), new_solvable_maze("", "maze_d"), StatusCode::CREATED).await;
    }

    #[actix_web::test]
    async fn cannot_create_maze_that_already_exists() {
        run_create_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), new_solvable_maze("", "maze_a"), StatusCode::CONFLICT).await;
    }

    // Get maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_maze_that_exists_with_invalid_api_key() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_get_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), id, StatusCode::UNAUTHORIZED, Some(new_solvable_maze(id, name))).await;
    }

    #[actix_web::test]
    async fn can_get_maze_that_exists() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_get_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), id, StatusCode::OK, Some(new_solvable_maze(id, name))).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_that_does_not_exist() {
        run_get_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), "does_not_exist.json", StatusCode::NOT_FOUND, None).await;
    }

    // Update maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_update_maze_that_exists_with_invalid_api_key() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), id, new_solvable_maze(id, name), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_update_maze_that_exists() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), id, new_solvable_maze(id, name), StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_that_does_not_exist() {
        let id = "maze_d.json";
        let name = "maze_d";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), id, new_solvable_maze(id, name), StatusCode::NOT_FOUND).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_with_mismatching_id() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), id, new_solvable_maze("some_other_id", name), StatusCode::BAD_REQUEST).await;
    }

    // Delete maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_delete_maze_that_exists_with_invalid_api_key() {
        run_delete_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), "maze_a.json", StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_delete_maze_that_exists() {
        run_delete_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes),Some(VALID_USERNAME_1), "maze_a.json", StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn cannot_delete_maze_that_does_not_exist() {
        run_delete_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), "does_not_exist.json", StatusCode:: NOT_FOUND).await;
    }

    // Get maze solution
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_maze_solution_that_should_succeed_with_invalid_api_key() {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(INVALID_USERNAME), "solvable.json", StatusCode::UNAUTHORIZED,
            Some(get_solve_test_maze_solution()), None
        ).await;
    }

    #[actix_web::test]
    async fn can_get_maze_solution_that_should_succeed() {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), "solvable.json", StatusCode::OK,
            Some(get_solve_test_maze_solution()), None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_start() {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), "no_start.json", StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_start_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_finish() {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), "no_finish.json", StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_solution() {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), "no_solution.json", StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_solution_error_str())
        ).await;
    }

    // Solve maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn canot_solve_maze_that_should_succeed_with_invalid_api_key() {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(INVALID_USERNAME),
            new_solve_test_maze("", "", true, true, false),
            StatusCode::UNAUTHORIZED,
            Some(get_solve_test_maze_solution()),
            None
        ).await;
    }

    #[actix_web::test]
    async fn can_solve_maze_that_should_succeed() {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            new_solve_test_maze("", "", true, true, false),
            StatusCode::OK,
            Some(get_solve_test_maze_solution()),
            None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_start() {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            new_solve_test_maze("", "", false, true, false),
            StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_start_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_yhat_should_fail_with_no_finish() {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            new_solve_test_maze("", "", true, false, false),
            StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_solution() {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            new_solve_test_maze("", "", true, true, true),
            StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_solution_error_str())
        ).await;
    }

    // API documentation page load
    #[actix_web::test]
    async fn can_load_swagger_ui_page() {
        run_get_url_test("/api-docs/v1/swagger-ui/").await;
    }

    #[actix_web::test]
    async fn can_load_openapi_json() {
        run_get_url_test("/api-docs/v1/openapi.json").await;
    }

    #[actix_web::test]
    async fn can_load_redoc_page() {
        run_get_url_test("/api-docs/v1/redoc").await;
    }

    #[actix_web::test]
    async fn can_load_rapidoc_page() {
        run_get_url_test("/api-docs/v1/rapidoc").await;
    }
}
