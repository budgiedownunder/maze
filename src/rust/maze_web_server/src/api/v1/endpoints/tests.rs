#[cfg(test)]
mod test_definitions {
    // **************************************************************************************************
    // Unit tests for API and documentation endpoints, via injection of MockStore
    // **************************************************************************************************
    use crate::api::v1::endpoints::handlers::{get_maze_solve_error_string, get_maze_generate_error_string};
    use crate::api::v1::endpoints::handlers::{AppFeaturesResponse, ChangePasswordRequest, CreateUserRequest, LoginRequest, LoginResponse, SignupRequest, UpdateProfileRequest, UserItem, UpdateUserRequest};
    use crate::{create_app, config::app::{AppConfig, AppFeaturesConfig}, oauth::{NoOpConnector, SharedOAuthConnector}, SharedFeatures};
    
    use actix_http;
    use actix_web::{http::StatusCode, test, dev::{Service, ServiceResponse}, web, Error, http::Method};
    use auth::{config::PasswordHashConfig, hashing::hash_password};
    use chrono::{DateTime, Utc};
    use data_model::{Maze, MazeDefinition, MazePoint, User, UserLogin};
    use maze::{Error as MazeError, GenerationAlgorithm, GeneratorOptions, MazePath, MazeSolution, MazeSolver};
    use pretty_assertions::assert_eq;
    use serde::Serialize;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};
    use tokio::sync::{RwLock as AsyncRwLock, RwLockReadGuard};
    use storage::{Error as StoreError, SharedStore, Store, store::MazeStore, store::UserStore, store::Manage, MazeItem, validation::validate_user_fields};
    use uuid::Uuid;

    const ADMIN_USERNAME_PREFIX:&str = "admin_";
    const USERNAME_PREFIX:&str = "user_";
    const VALID_USER_PASSWORD: &str = "Password1!";
    const INVALID_USERNAME: &str = "INVALID_USERNAME";
    const INVALID_EMAIL: &str = "invalid@example.com";
    const INVALID_USER_PASSWORD: &str = "BAD PASSWORD";

    const NEW_ADMIN_USERNAME_1: &str = "new_admin_1";
    const NEW_USERNAME_1: &str = "new_user_1";

    const VALID_ADMIN_USERNAME_1: &str = "admin_1";
    const VALID_ADMIN_USERNAME_2: &str = "admin_2";
    const VALID_USERNAME_1: &str = "user_1";
    const VALID_USERNAME_2: &str = "user_2";
    const VALID_USER_EMAIL_1: &str = "user_1@company.com";

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
            format!("{name}.json")
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
                email: self.user.email().to_string(),
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
                if let Ok(user) = MockStore::find_user_by_name_in_map(&self.users, username, Uuid::nil()) {
                    return user.api_key;
                }
            }
            User::new_api_key()
        }

        fn login_user_by_name_in_map(&mut self, username: &str) -> Result<UserLogin, StoreError> {
            for v in self.users.values_mut() {
                if v.user.username == username {
                    let login = v.user.create_login(24, Some("123.456.789.123".to_string()), Some("Some device information".to_string()));
                    return Ok(login.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }

        fn add_user_login(&mut self, username: Option<&str>) -> Result<Uuid, StoreError> {
            if let Some(username) = username {
                if let Ok(login) = self.login_user_by_name_in_map(username) {
                    return Ok(login.id);        
                }
            }
            Err(StoreError::UserNotFound())
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

        /// Locates a user by their email within the store. Looks across every
        /// row of every user (matching the SQL `user_emails.email` UNIQUE).
        fn find_user_by_email(&self, email: &str, ignore_id: Uuid) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user.id == ignore_id {
                    continue;
                }
                if v.user.emails.iter().any(|row| row.email.eq_ignore_ascii_case(email)) {
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
            // OAuth-only users have an empty password_hash; password-only
            // signup still requires one.
            if user.password_hash.is_empty() && user.oauth_identities.is_empty() {
                return Err(StoreError::UserPasswordMissing());
            }
            if self.user_name_exists(&user.username, ignore_id) {
                return Err(StoreError::UserNameExists());
            }
            for row in &user.emails {
                if self.user_email_exists(&row.email, ignore_id) {
                    return Err(StoreError::UserEmailExists());
                }
            }
            Ok(())
        }
    }

    #[async_trait]
    impl MazeStore for MockStore {

        async fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError> {
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

        async fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), StoreError> {
            let mock_user = self.get_mock_user_mut(owner.id)?;
            if mock_user.mazes.remove(id).is_some() {
                Ok(())
            } else {
                Err(StoreError::MazeIdNotFound(id.to_string()))
            }
        }

        async fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), StoreError> {
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

        async fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, StoreError> {
            let mock_user = self.get_mock_user(owner.id)?;
            if let Some(mock_maze) = mock_user.mazes.get(id) {
                return Ok(mock_maze.maze.clone());
            }
            Err(StoreError::MazeIdNotFound(id.to_string()))
        }

        async fn find_maze_by_name(&self, _owner: &User, _name: &str) -> Result<MazeItem, StoreError> {
            Err(StoreError::Other("Mock interface not implemented".to_string()))
        }

        async fn get_maze_items(&self, owner: &User, include_definitions: bool) -> Result<Vec<MazeItem>, StoreError> {
            let mock_user = self.get_mock_user(owner.id)?;
            let mut items: Vec<MazeItem> = maze_items_from_map(&mock_user.mazes, include_definitions);
            items.sort_by_key(|item| item.name.clone());
            Ok(items)
        }
    }

    #[async_trait]
    impl UserStore for MockStore {
        /// Adds the default admin user to the store if it doesn't already exist, else returns it
        async fn init_default_admin_user(&mut self, _username: &str, _email: &str, _password_hash: &str) -> Result<User, StoreError> {
            Err(StoreError::Other("init_default_admin_user() not implemented for MockStore".to_string()))
        }
        /// Adds a new user to the store and sets the allocated `id` within the user object
        async fn create_user(&mut self, user: &mut User) -> Result<(), StoreError> {
            let mock_user = MockUser::new_from_user(user);
            user.id = mock_user.user.id;
            self.validate_user(user, Uuid::nil())?;
            self.users.insert(mock_user.user.id, mock_user);
            Ok(())
        }
        /// Deletes a user from the store
        async fn delete_user(&mut self, id: Uuid) -> Result<(), StoreError> {
            if self.users.remove(&id).is_some() {
                Ok(())
            } else {
                Err(StoreError::UserIdNotFound(id.to_string()))
            }
        }
        /// Updates a user within the store
        async fn update_user(&mut self, user: &mut User) -> Result<(), StoreError> {
            self.validate_user(user, user.id)?;
            let mock_user = self.get_mock_user_mut(user.id)?;
            mock_user.user = user.clone();
            Ok(())
        }
        /// Loads a user from the store
        async fn get_user(&self, id: Uuid) -> Result<User, StoreError> {
            if let Some(mock_user) = self.users.get(&id) {
                return Ok(mock_user.user.clone());
            }
            Err(StoreError::UserIdNotFound(id.to_string()))
        }
        /// Locates a user by their username within the store
        async fn find_user_by_name(&self, name: &str) -> Result<User, StoreError> {
            MockStore::find_user_by_name_in_map(&self.users, name, Uuid::nil())
        }
        /// Locates a user by their email address within the store
        async fn find_user_by_email(&self, email: &str) -> Result<User, StoreError> {
            self.find_user_by_email(email, Uuid::nil())
        }
        /// Locates a user by their api key within the store
        async fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user.api_key == api_key {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }

        async fn find_user_by_login_id(&self, login_id: Uuid) -> Result<User, StoreError>{
            for v in self.users.values() {
                if v.user.contains_valid_login(login_id) {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }

        async fn find_user_by_oauth_identity(&self, provider: &str, provider_user_id: &str) -> Result<User, StoreError> {
            for v in self.users.values() {
                if v.user.oauth_identities.iter().any(|i| {
                    i.provider.eq_ignore_ascii_case(provider) && i.provider_user_id == provider_user_id
                }) {
                    return Ok(v.user.clone());
                }
            }
            Err(StoreError::UserNotFound())
        }
        /// Returns the list of users within the store, sorted
        /// alphabetically by username in ascending order
        async fn get_users(&self) -> Result<Vec<User>, StoreError> {
            let mut users: Vec<User> = self.users.values()
                .map( |value| value.user.clone())
                .collect();

            users.sort_by_key(|user| user.username.clone());
            Ok(users)
        }

        /// Returns the list of admin users within the store
        async fn get_admin_users(&self) -> Result<Vec<User>, StoreError> {
            let admins: Vec<User> = self.users.values()
                .filter(|v| v.user.is_admin)
                .map(|v| v.user.clone())
                .collect();
            Ok(admins)
        }

        async fn has_users(&self) -> Result<bool, StoreError> {
            Ok(!self.users.is_empty())
        }
    }

    #[async_trait]
    impl Manage for MockStore {
        async fn empty(&mut self) -> Result<(), StoreError> {
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
        from.values().map(|value| MazeItem {
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

    fn new_user(username: &str, is_admin: bool, password_hash: &str) -> User {
        let mut user = User::default();
        user.id = User::new_id();
        user.username = username.to_string();
        user.is_admin = is_admin;
        user.api_key = User::new_api_key();
        user.set_primary_email_address(&new_email(username));
        user.password_hash = password_hash.to_string();
        user
    }

    fn new_email(username: &str) -> String {
        format!("{username}@company.com")
    }

    #[derive(Clone)]
    struct UserDefinition {
        username: String,
        is_admin: bool,
        password_hash: String,
        mazes: MazeContent,
    }

    fn append_user_defs(user_defs: &mut Vec<UserDefinition>, num: i32, is_admin: bool, password_hash: &str, mazes: &MazeContent) {
        let username_prefix = if is_admin { ADMIN_USERNAME_PREFIX } else { USERNAME_PREFIX};
        for i in 1..(num+1) {
            user_defs.push( UserDefinition {
                username: format!("{username_prefix}{i}"),
                is_admin,
                password_hash: password_hash.to_string(), 
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
        append_user_defs(&mut user_defs, def.num_users, false, "", &def.mazes);
        append_user_defs(&mut user_defs, def.num_admin_users, true, "", &def.mazes);
        user_defs
    }

    fn new_mock_user(user_def: &UserDefinition) -> MockUser {
        let user =  new_user(&user_def.username, user_def.is_admin, &user_def.password_hash);
        MockUser {
            user,
            mazes: new_mazes_map(user_def.mazes.clone()),
        }
    }

    fn new_shared_mock_maze_store(mock_store: MockStore) -> SharedStore {
        Arc::new(AsyncRwLock::new(Box::new(mock_store)))
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

    fn create_test_request<T: Serialize>(
        method: Method,
        url: &str,
        api_key: Option<Uuid>,
        login_id: Option<Uuid>,
        json_body: Option<&T>,
    ) -> actix_http::Request {
        let mut req = test::TestRequest::default()
            .method(method)
            .uri(url);

        if let Some(login_id) = login_id {
            req = req.insert_header(("Authorization", format!("Bearer {login_id}")));
        }
        else if  let Some(api_key) = api_key {
            req = req.insert_header(("X-API-KEY", api_key.to_string()));
        }    

        if let Some(body) = json_body {
            req = req.set_json(body);
        }

        req.to_request()
    }    

    fn create_test_get_request(url: &str, api_key: Option<Uuid>, login_id: Option<Uuid>) -> actix_http::Request {
        create_test_request(Method::GET, url, api_key, login_id, None::<&()>)
    }

    fn create_test_post_request<T: serde::Serialize>(url: &str, api_key: Option<Uuid>, login_id: Option<Uuid>, body_obj: Option<&T>) -> actix_http::Request {
        create_test_request(Method::POST, url, api_key, login_id, body_obj)
    }

    fn create_test_put_request<T: serde::Serialize>(url: &str, api_key: Option<Uuid>, login_id: Option<Uuid>, body_obj: &T) -> actix_http::Request {
        create_test_request(Method::PUT, url, api_key, login_id, Some(body_obj))
    }

    fn create_test_delete_request(url: &str, api_key: Option<Uuid>, login_id: Option<Uuid>) -> actix_http::Request {
        create_test_request(Method::DELETE, url, api_key, login_id, None::<&()>)
    }

    fn create_shared_mock_store(
        user_defs:&Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
     ) -> (SharedStore, HashMap<Uuid, MockUser>, Uuid, Option<Uuid>) {
        let mut mock_store = MockStore::new(user_defs);
        let api_key = mock_store.get_api_key_to_use(caller_username);
        let mut login_id = None;
        if add_login {
            if let Ok(user_login_id) = mock_store.add_user_login(caller_username) {
                login_id = Some(user_login_id);
            }
        }
        let mock_users = mock_store.users.clone();
        let shared_mock_store = new_shared_mock_maze_store(mock_store);
        (shared_mock_store, mock_users, api_key, login_id)
    }

    fn set_valid_password_hashes(hash_config: &PasswordHashConfig, user_defs: &mut Vec<UserDefinition>) {
        let password_hash = match hash_password(VALID_USER_PASSWORD, hash_config) {
            Ok(hash) => hash,
            Err(_) => "".to_string(),            
        };
        for user_def in user_defs {
            user_def.password_hash = password_hash.to_string();
        }    
    }

    async fn create_test_app_with_config(
        user_defs: &mut Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
        features: SharedFeatures,
        app_config: AppConfig,
    ) -> (impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>, SharedStore, HashMap<Uuid, MockUser>, Option<Uuid>, Option<Uuid>) {
        set_valid_password_hashes(&app_config.security.password_hash, user_defs);

        let (shared_mock_store, mock_users, api_key, login_id) = create_shared_mock_store(user_defs, caller_username, add_login);
        let connector: SharedOAuthConnector = Arc::new(NoOpConnector);
        let app = test::init_service(
            create_app(&app_config.security.password_hash, web::Data::new(shared_mock_store.clone()), web::Data::new(features), web::Data::new(connector), ".".to_string())
            .app_data(web::Data::new(app_config))
        )
        .await;

        (app, shared_mock_store, mock_users, Some(api_key), login_id)
    }

    async fn create_test_app_with_features(
        user_defs: &mut Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
        features: SharedFeatures,
    ) -> (impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>, SharedStore, HashMap<Uuid, MockUser>, Option<Uuid>, Option<Uuid>) {
        let mut config = AppConfig::default();
        config.security.password_hash = auth::config::PasswordHashConfig::for_testing();
        create_test_app_with_config(user_defs, caller_username, add_login, features, config).await
    }

    async fn create_test_app(
        user_defs: &mut Vec<UserDefinition>,
        caller_username: Option<&str>,
        add_login: bool,
    ) -> (impl Service<actix_http::Request, Response = ServiceResponse, Error = Error>, SharedStore, HashMap<Uuid, MockUser>, Option<Uuid>, Option<Uuid>) {
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        create_test_app_with_features(user_defs, caller_username, add_login, features).await
    }

    fn get_invalid_email_or_password_error_str() -> String {
        "Invalid email or password".to_string()
    }

    fn get_email_and_password_must_be_provided_error_str() -> String {
        "Email and password must be provided".to_string()
    }

    async fn get_store_read_lock(
        shared_store: &Arc<AsyncRwLock<Box<dyn Store>>>,
    ) -> RwLockReadGuard<'_, Box<dyn Store>> {
        shared_store.read().await
    }

    async fn verify_user_login_presence(shared_store: &Arc<AsyncRwLock<Box<dyn Store>>>, email: &str, login_id: Uuid, expected_presence: bool) {
        let store_lock = get_store_read_lock(shared_store).await;
        // Confirm login id associated with user
        match store_lock.find_user_by_email(email).await {
            Ok(user) => {
                let presence = user.contains_valid_login(login_id);
                if presence != expected_presence {
                    panic!("{}", format!("User contains_login() returned an unexpected value (expected = {expected_presence}, returned = {presence})"));
                }
            },
            Err(err) => panic!("{}", format!("Failed to locate user for login id = {login_id} => {err}"))
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_login_logout_test(
        create_users_def: &CreateUsersDef,
        email: &str,
        password: &str,
        expected_login_status_code: StatusCode,
        expected_login_err_message: Option<String>,
        run_logout_test: bool,
        set_logout_login_id: bool,
        expected_logout_status_code: Option<StatusCode>,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, shared_store, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let login_url = "/api/v1/login".to_string();
        let login_request = LoginRequest {
            email: email.to_string(),
            password: password.to_string(),
        };
        let login_req = create_test_post_request(&login_url, None, None, Some(&login_request));
        let login_resp = test::call_service(&app, login_req).await;

        assert_eq!(login_resp.status(), expected_login_status_code);

        if expected_login_status_code == StatusCode::OK {
            let login_resp_body = test::read_body(login_resp).await;
            let login_response: LoginResponse = serde_json::from_slice(&login_resp_body).expect("failed to deserialize login response");
            let login_id = login_response.login_token_id;
            assert_ne!(login_id, Uuid::nil());
            assert_ne!(login_response.login_token_expires_at, DateTime::<Utc>::default());

            if run_logout_test {
                verify_user_login_presence(&shared_store, email, login_id, true).await;

                // Logout
                let logout_url = "/api/v1/logout".to_string();
                let logout_login_id = set_logout_login_id.then_some(login_id);
                let logout_req = create_test_post_request(&logout_url, None, logout_login_id, None::<&()>);
                let logout_resp = test::call_service(&app, logout_req).await;

                if let Some(expected_logout_status_code) = expected_logout_status_code {
                    assert_eq!(logout_resp.status(), expected_logout_status_code);
                    if expected_logout_status_code == StatusCode::NO_CONTENT {
                        verify_user_login_presence(&shared_store, email, login_id, false).await;
                    }
                }
            }

        } else {
            match expected_login_err_message {
                Some(value) => {
                    // Validate error response
                    let login_resp_body = test::read_body(login_resp).await;
                    let error_message = String::from_utf8(login_resp_body.to_vec()).expect("Failed to parse login response body as UTF-8");
                    assert_eq!(error_message, value);
                }
                None => { panic!("No error message provided for login test!"); }
            }
        }
    }

    async fn run_get_users_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let path_str = "/api/v1/users".to_string();
        let req = create_test_get_request(&path_str, api_key, login_id);
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
            "Password1!".to_string()
        }
    }

    fn new_create_user_request(is_admin: bool, username: &str, email: Option<&str>, blank_password: bool) -> CreateUserRequest {
        let email_use = if let Some(s) = email {
            s
        } else {
            &new_email(username)
        };

        CreateUserRequest::new(is_admin, username, 
            &format!("{username} full name"), 
            email_use, 
            &create_password(blank_password) 
        )    
    }

    async fn run_create_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        create_req: &CreateUserRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users".to_string();
        let req = create_test_post_request(&url, api_key, login_id, Some(&create_req));
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
        use_login: bool, 
        target_username: &str,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let id = MockStore::find_user_id_by_name_in_map(&mock_users, target_username, Uuid::nil());
        let url = format!("/api/v1/users/{id}");
        let req = create_test_get_request(&url, api_key, login_id);
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
            &format!("Updated {username} full name"), 
            email_use
        )    
    }    

    async fn run_update_user_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        target_username: &str,
        update_req: &UpdateUserRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let id = MockStore::find_user_id_by_name_in_map(&mock_users, target_username, Uuid::nil());
        let url = format!("/api/v1/users/{id}");
        let req = create_test_put_request(&url, api_key, login_id, &update_req);
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
        use_login: bool, 
        target_username: &str,
        expected_status_code: StatusCode
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let id = MockStore::find_user_id_by_name_in_map(&mock_users, target_username, Uuid::nil());
        let url = format!("/api/v1/users/{id}");
        let req = create_test_delete_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            if Some(target_username) == caller_username {
                return;
            }

            // Confirm it has been deleted
            let url2 = format!("/api/v1/users/{id}");
            let req2 = create_test_get_request(&url2, api_key, None);
            let resp2 = test::call_service(&app, req2).await;
            assert_eq!(resp2.status(), StatusCode::NOT_FOUND);
        }
    }

    async fn run_get_mazes_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        include_definitions: bool,
        expected_maze_content:MazeContent
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let path_str = format!("/api/v1/mazes?includeDefinitions={include_definitions}");
        let req = create_test_get_request(&path_str, api_key, login_id);
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
        use_login: bool, 
        maze: Maze,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/mazes".to_string();
        let req = create_test_post_request(&url, api_key, login_id, Some(&maze));
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
        use_login: bool, 
        id: &str,
        expected_status_code: StatusCode,
        expected_maze: Option<Maze>
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = format!("/api/v1/mazes/{id}");
        let req = create_test_get_request(&url, api_key, login_id);
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
        use_login: bool, 
        id: &str,
        maze: Maze,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = format!("/api/v1/mazes/{id}");
        let req = create_test_put_request(&url,api_key, login_id, &maze);
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
        use_login: bool, 
        id: &str,
        expected_status_code: StatusCode
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = format!("/api/v1/mazes/{id}");
        let req = create_test_delete_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            // Confirm it has been deleted
            let url2 = format!("/api/v1/mazes/{id}");
            let req2 = create_test_get_request(&url2, api_key, login_id);
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
                None => { panic!("{}", format!("No maze solution comparison value provided for {context} test!")); }
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
                None => { panic!("{}", format!("No maze solution error message provided for {context} test!")); }
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
        use_login: bool, 
        id: &str,
        expected_status_code: StatusCode,
        expected_solution: Option<MazeSolution>,
        expected_err_message: Option<String>
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = format!("/api/v1/mazes/{id}/solution");
        let req = create_test_get_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;

        validate_solution_response("get_maze_solution()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_solve_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool, 
        maze: Maze,
        expected_status_code: StatusCode,
        expected_solution: Option<MazeSolution>,
        expected_err_message: Option<String>
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/solve-maze".to_string();
        let req = create_test_post_request(&url, api_key, login_id, Some(&maze));
        let resp = test::call_service(&app, req).await;

        validate_solution_response("solve_maze()", resp, expected_status_code, expected_solution, expected_err_message).await;
    }

    async fn run_get_url_test(
        url: &str
     ) {

        let (app, _, _, _, _) = create_test_app(&mut vec![], None, false).await;
        let req = create_test_get_request(url, None, None);
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
    }
    /*********************************************************************/
    /* Endpoint tests                                                    */
    /*********************************************************************/
    /**********/
    /* Users  */
    /**********/

    // Reusable test wrapper functions
    async fn run_cannot_get_users_with_one_non_admin_user_with_non_admin_caller(use_login: bool) {
        run_get_users_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), Some(VALID_USERNAME_1), use_login, StatusCode::UNAUTHORIZED).await;
    }

    async fn run_can_get_users_with_one_admin_user_with_api_key(use_login: bool) {
        run_get_users_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), use_login, StatusCode::OK).await;
    }

    async fn run_can_get_users_with_one_admin_and_one_non_admin_user_with_api_key(use_login: bool) {
        run_get_users_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_1), use_login, StatusCode::OK).await;
    }

    async fn run_can_get_users_with_ten_admin_and_five_non_admin_users(use_login: bool) {
        run_get_users_test(&CreateUsersDef::new(10, 5, MazeContent::Empty), Some(VALID_ADMIN_USERNAME_2), use_login, StatusCode::OK).await;
    }

    async fn run_can_create_non_existent_admin_user_with_admin_caller(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None , false),
            StatusCode::CREATED).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login,
            &new_create_user_request(true, "", None,  false),
            StatusCode::BAD_REQUEST).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login,
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None , true),
            StatusCode::BAD_REQUEST).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_non_admin_caller(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), 
            Some(VALID_USERNAME_1), use_login, 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None, false),
            StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, VALID_ADMIN_USERNAME_1, None , false), 
            StatusCode::CONFLICT).await;
    }

    async fn run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, VALID_ADMIN_USERNAME_2, Some(&new_email(VALID_ADMIN_USERNAME_1)), false), 
            StatusCode::CONFLICT).await;
    }

    async fn run_can_create_non_existent_non_admin_user_with_admin_caller(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(false, NEW_USERNAME_1, None, false),
            StatusCode::CREATED).await;
    }

    async fn run_cannot_create_non_existent_non_admin_user_with_non_admin_caller(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), 
            Some(VALID_USERNAME_1), use_login, 
            &new_create_user_request(false, NEW_USERNAME_1, None, false),
            StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, VALID_USERNAME_1, None, false),
            StatusCode::CONFLICT).await;
    }

    async fn run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email(use_login: bool) {
        run_create_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, 
            &new_create_user_request(true, VALID_USERNAME_2, Some(&new_email(VALID_USERNAME_1)), false),
            StatusCode::CONFLICT).await;
    }

    async fn run_can_get_user_that_exists_with_admin_caller(use_login: bool) {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          Some(VALID_ADMIN_USERNAME_1), use_login, 
                          VALID_USERNAME_1, StatusCode::OK).await;
    }

    async fn run_can_get_admin_user_that_exists_with_admin_caller(use_login: bool) {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          Some(VALID_ADMIN_USERNAME_1), use_login, 
                          VALID_ADMIN_USERNAME_1, StatusCode::OK).await;
    }

    async fn run_cannot_get_user_that_exists_with_non_admin_caller(use_login: bool) {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          Some(VALID_USERNAME_1), use_login, 
                          VALID_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_get_user_that_does_not_exist_with_admin_caller(use_login: bool) {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          Some(VALID_ADMIN_USERNAME_1), use_login, 
                          VALID_USERNAME_2, StatusCode::NOT_FOUND).await;
    }

    async fn run_can_update_admin_user_with_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(true, NEW_ADMIN_USERNAME_1, None),
            StatusCode::OK).await;
    }

    async fn run_cannot_update_admin_user_with_non_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(true, NEW_ADMIN_USERNAME_1, None),
            StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_update_admin_user_with_admin_caller_but_missing_username(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(true, "", None),
            StatusCode::BAD_REQUEST).await;
    }

    async fn run_cannot_update_admin_user_with_admin_caller_but_existing_username(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(true, VALID_ADMIN_USERNAME_2, None),
            StatusCode::CONFLICT).await;
    }

    async fn run_cannot_update_admin_user_with_admin_caller_but_existing_email(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login,
            VALID_ADMIN_USERNAME_1, &new_update_user_request(true, VALID_ADMIN_USERNAME_1, Some(&new_email(VALID_ADMIN_USERNAME_2))),
            StatusCode::CONFLICT).await;
    }

    async fn run_can_update_non_admin_user_with_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(false, NEW_USERNAME_1, None),
            StatusCode::OK).await;
    }

    async fn run_cannot_update_non_admin_user_with_admin_caller_but_missing_username(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(false, "", None),
            StatusCode::BAD_REQUEST).await;
    }

    async fn run_cannot_update_non_admin_user_with_admin_caller_but_existing_username(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 2, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(false, VALID_USERNAME_2, None),
            StatusCode::CONFLICT).await;
    }

    async fn run_cannot_update_non_admin_user_with_admin_caller_but_existing_email(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 2, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(false, VALID_USERNAME_1, Some(&new_email(VALID_USERNAME_2))),
            StatusCode::CONFLICT).await;
    }

    async fn run_can_upgrade_non_admin_user_to_admin_with_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, 
            &new_update_user_request(true, VALID_USERNAME_1, None),
            StatusCode::OK).await;
    }

    async fn run_can_downgrade_admin_user_to_non_admin_with_admin_caller(use_login: bool) {
        run_update_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, 
            &new_update_user_request(false, VALID_ADMIN_USERNAME_1, None),
            StatusCode::OK).await;
    }

    async fn run_can_delete_existing_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_2, StatusCode::OK).await;
    }
    
    async fn run_cannot_delete_last_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty),
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, StatusCode::CONFLICT).await;
    }


    async fn run_cannot_delete_non_existent_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(1, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_ADMIN_USERNAME_2, StatusCode::NOT_FOUND).await;
    }

    async fn run_can_delete_existing_non_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 1, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, StatusCode::OK).await;
    }

    async fn run_cannot_delete_non_existent_non_admin_user_with_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 0, MazeContent::Empty), 
            Some(VALID_ADMIN_USERNAME_1), use_login, VALID_USERNAME_1, StatusCode::NOT_FOUND).await;
    }

    async fn run_cannot_delete_existing_admin_user_with_non_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 1, MazeContent::Empty), 
            Some(VALID_USERNAME_1), use_login, VALID_ADMIN_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_delete_existing_non_admin_user_with_non_admin_caller(use_login: bool) {
        run_delete_user_test(&CreateUsersDef::new(2, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login, VALID_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    async fn run_cannot_delete_me_when_last_admin(use_login: bool) {
        run_delete_me_test(
            &CreateUsersDef::new(1, 0, MazeContent::Empty),
            Some(VALID_ADMIN_USERNAME_1),
            use_login,
            StatusCode::CONFLICT,
        ).await;
    }

    async fn run_can_delete_me_when_not_last_admin(use_login: bool) {
        run_delete_me_test(
            &CreateUsersDef::new(2, 0, MazeContent::Empty),
            Some(VALID_ADMIN_USERNAME_1),
            use_login,
            StatusCode::NO_CONTENT,
        ).await;
    }

    async fn run_can_get_mazes_with_no_mazes(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::Empty), Some(VALID_USERNAME_1), use_login, false, MazeContent::Empty).await;
    }

    async fn run_can_get_mazes_with_one_maze_without_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::OneMaze), Some(VALID_USERNAME_1), use_login, false, MazeContent::OneMaze).await;
    }

    async fn run_can_get_mazes_with_one_maze_with_defintions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::OneMaze), Some(VALID_USERNAME_1), use_login, true, MazeContent::OneMaze).await;
    }

    async fn run_can_get_mazes_with_two_mazes_that_require_sorting_without_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::TwoMazes), Some(VALID_USERNAME_1), use_login, false, MazeContent::TwoMazes).await;
    }

    async fn run_can_get_mazes_with_two_mazes_that_require_sorting_with_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::TwoMazes), Some(VALID_USERNAME_1), use_login, true, MazeContent::TwoMazes).await;
    }

    async fn run_can_get_mazes_with_three_mazes_that_require_sorting_without_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, false, MazeContent::ThreeMazes).await;
    }

    async fn run_can_get_mazes_with_three_mazes_that_require_sorting_with_definitions(use_login: bool) {
        run_get_mazes_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, true, MazeContent::ThreeMazes).await;
    }

    async fn run_can_create_maze_that_does_not_exist(use_login: bool) {
        run_create_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, new_solvable_maze("", "maze_d"), StatusCode::CREATED).await;
    }

    async fn run_cannot_create_maze_that_already_exists(use_login: bool) {
        run_create_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, new_solvable_maze("", "maze_a"), StatusCode::CONFLICT).await;
    }

    async fn run_can_get_maze_that_exists(use_login: bool) {
        let id = "maze_a.json";
        let name = "maze_a";
        run_get_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, id, StatusCode::OK, Some(new_solvable_maze(id, name))).await;
    }

    async fn run_cannot_get_maze_that_does_not_exist(use_login: bool) {
        run_get_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, "does_not_exist.json", StatusCode::NOT_FOUND, None).await;
    }

    async fn run_can_update_maze_that_exists(use_login: bool) {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, id, new_solvable_maze(id, name), StatusCode::OK).await;
    }

    async fn run_cannot_update_maze_that_does_not_exist(use_login: bool) {
        let id = "maze_d.json";
        let name = "maze_d";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, id, new_solvable_maze(id, name), StatusCode::NOT_FOUND).await;
    }

    async fn run_cannot_update_maze_with_mismatching_id(use_login: bool) {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), use_login, id, new_solvable_maze("some_other_id", name), StatusCode::BAD_REQUEST).await;
    }

    async fn run_can_get_maze_solution_that_should_succeed(use_login: bool) {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), use_login, "solvable.json", StatusCode::OK,
            Some(get_solve_test_maze_solution()), None
        ).await;
    }

    async fn run_cannot_get_maze_solution_that_should_fail_with_no_start(use_login: bool) {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), use_login, "no_start.json", StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_start_cell_error_str())
        ).await;
    }

    async fn run_cannot_get_maze_solution_that_should_fail_with_no_finish(use_login: bool) {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), use_login, "no_finish.json", StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    async fn run_cannot_get_maze_solution_that_should_fail_with_no_solution(use_login: bool) {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(VALID_USERNAME_1), use_login, "no_solution.json", StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_solution_error_str())
        ).await;
    }

    async fn run_can_solve_maze_that_should_succeed(use_login: bool) {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            new_solve_test_maze("", "", true, true, false),
            StatusCode::OK,
            Some(get_solve_test_maze_solution()),
            None
        ).await;
    }

    async fn run_cannot_solve_maze_that_should_fail_with_no_start(use_login: bool) {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            new_solve_test_maze("", "", false, true, false),
            StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_start_cell_error_str())
        ).await;
    }

    async fn run_cannot_solve_maze_yhat_should_fail_with_no_finish(use_login: bool) {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            new_solve_test_maze("", "", true, false, false),
            StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_finish_cell_error_str())
        ).await;
    }

    async fn run_cannot_solve_maze_that_should_fail_with_no_solution(use_login: bool) {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            new_solve_test_maze("", "", true, true, true),
            StatusCode::UNPROCESSABLE_ENTITY, None,
            Some(get_no_solution_error_str())
        ).await;
    }

    // Login
    #[actix_web::test]
    async fn cannot_login_if_no_users_exist() {
        run_login_logout_test(&CreateUsersDef::new(0, 0, MazeContent::Empty),
            INVALID_EMAIL,
            INVALID_USER_PASSWORD,
            StatusCode::UNAUTHORIZED,
            Some(get_invalid_email_or_password_error_str()),
            false,
            false,
            None 
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_no_email() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            "",
            INVALID_USER_PASSWORD,
            StatusCode::UNPROCESSABLE_ENTITY,
            Some(get_email_and_password_must_be_provided_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_no_password() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            INVALID_EMAIL,
            "",
            StatusCode::UNPROCESSABLE_ENTITY,
            Some(get_email_and_password_must_be_provided_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_email_does_not_exist() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            INVALID_EMAIL,
            INVALID_USER_PASSWORD,
            StatusCode::UNAUTHORIZED,
            Some(get_invalid_email_or_password_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_email_format_is_invalid() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            "notanemail",
            INVALID_USER_PASSWORD,
            StatusCode::UNAUTHORIZED,
            Some(get_invalid_email_or_password_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn cannot_login_if_email_exists_and_bad_password() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            VALID_USER_EMAIL_1,
            INVALID_USER_PASSWORD,
            StatusCode::UNAUTHORIZED,
            Some(get_invalid_email_or_password_error_str()),
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn can_login_with_valid_credentials() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            VALID_USER_EMAIL_1,
            VALID_USER_PASSWORD,
            StatusCode::OK,
            None,
            false,
            false,
            None
        ).await;
    }

    #[actix_web::test]
    async fn can_login_and_logout_with_valid_credentials() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            VALID_USER_EMAIL_1,
            VALID_USER_PASSWORD,
            StatusCode::OK,
            None,
            true,
            true,
            Some(StatusCode::NO_CONTENT)
        ).await;
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_logout_if_login_id_not_set_in_logout_header() {
        run_login_logout_test(&CreateUsersDef::new(1, 1, MazeContent::Empty),
            VALID_USER_EMAIL_1,
            VALID_USER_PASSWORD,
            StatusCode::OK,
            None,
            true,
            false,
            None
        ).await;
    }

    // Renew
    #[actix_web::test]
    async fn can_renew_with_valid_token() {
        use crate::api::v1::endpoints::handlers::RenewResponse;
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, shared_store, _, _, _) = create_test_app(&mut user_defs, None, false).await;

        // Log in first
        let login_request = LoginRequest { email: VALID_USER_EMAIL_1.to_string(), password: VALID_USER_PASSWORD.to_string() };
        let login_req = create_test_post_request("/api/v1/login", None, None, Some(&login_request));
        let login_resp = test::call_service(&app, login_req).await;
        assert_eq!(login_resp.status(), StatusCode::OK);

        let login_resp_body = test::read_body(login_resp).await;
        let login_response: LoginResponse = serde_json::from_slice(&login_resp_body).expect("failed to deserialize login response");
        let login_id = login_response.login_token_id;
        let original_expiry = login_response.login_token_expires_at;

        // Renew the token
        let renew_req = create_test_post_request("/api/v1/login/renew", None, Some(login_id), None::<&()>);
        let renew_resp = test::call_service(&app, renew_req).await;
        assert_eq!(renew_resp.status(), StatusCode::OK);

        let renew_resp_body = test::read_body(renew_resp).await;
        let renew_response: RenewResponse = serde_json::from_slice(&renew_resp_body).expect("failed to deserialize renew response");

        // Token ID is unchanged
        assert_eq!(renew_response.login_token_id, login_id);
        // Expiry is extended
        assert!(renew_response.login_token_expires_at >= original_expiry);
        // Login still present in store
        verify_user_login_presence(&shared_store, VALID_USER_EMAIL_1, login_id, true).await;
    }

    #[actix_web::test]
    async fn cannot_renew_with_api_key() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, api_key, _) = create_test_app(&mut user_defs, Some(VALID_USERNAME_1), false).await;
        let renew_req = create_test_post_request("/api/v1/login/renew", api_key, None, None::<&()>);
        let renew_resp = test::call_service(&app, renew_req).await;
        assert_eq!(renew_resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_renew_without_auth_header() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let renew_req = create_test_post_request("/api/v1/login/renew", None, None, None::<&()>);
        test::call_service(&app, renew_req).await;
    }

    // Get users
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn run_test_cannot_get_users_with_no_users_with_invalid_api_key() {
        run_get_users_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, false, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn cannot_get_users_with_one_non_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_get_users_with_one_non_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_users_with_one_non_admin_user_with_non_admin_caller_with_login() {
        run_cannot_get_users_with_one_non_admin_user_with_non_admin_caller(true).await;
    }    

    #[actix_web::test]
    async fn can_get_users_with_one_admin_user_with_api_key() {
        run_can_get_users_with_one_admin_user_with_api_key(false).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_one_admin_user_with_login() {
        run_can_get_users_with_one_admin_user_with_api_key(true).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_one_admin_and_one_non_admin_user_with_api_key() {
        run_can_get_users_with_one_admin_and_one_non_admin_user_with_api_key(false).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_one_admin_and_one_non_admin_user_with_login() {
        run_can_get_users_with_one_admin_and_one_non_admin_user_with_api_key(true).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_ten_admin_and_five_non_admin_users_with_api_key() {
        run_can_get_users_with_ten_admin_and_five_non_admin_users(false).await;
    }

    #[actix_web::test]
    async fn can_get_users_with_ten_admin_and_five_non_admin_users_with_login() {
        run_can_get_users_with_ten_admin_and_five_non_admin_users(true).await;
    }

    // Create user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_create_admin_user_with_invalid_api_key() {
        run_create_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, 
            &new_create_user_request(true, NEW_ADMIN_USERNAME_1, None, false),
            StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_admin_user_with_admin_caller_with_api_key() {
        run_can_create_non_existent_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_admin_user_with_admin_caller_with_login() {
        run_can_create_non_existent_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username_with_login() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password_with_login() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_missing_password(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_non_admin_caller_with_login() {
        run_cannot_create_non_existent_admin_user_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username_with_login() {
       run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email_with_api_key() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email_with_login() {
        run_cannot_create_non_existent_admin_user_with_admin_caller_but_existing_email(true).await;
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_create_non_admin_user_with_invalid_api_key() {
        run_create_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, 
            &new_create_user_request(false, NEW_USERNAME_1, None, false), 
            StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_non_admin_user_with_admin_caller_with_api_key() {
        run_can_create_non_existent_non_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_create_non_existent_non_admin_user_with_admin_caller_with_login() {
        run_can_create_non_existent_non_admin_user_with_admin_caller(true).await;
    }
    
    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_create_non_existent_non_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_non_admin_caller_with_login() {
        run_cannot_create_non_existent_non_admin_user_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username_with_api_key() {
        run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username_with_login() {
        run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email_with_api_key() {
        run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email_with_login() {
        run_cannot_create_non_existent_non_admin_user_with_admin_caller_but_existing_email(true).await;
    }

    // Get user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_user_that_exists_with_invalid_api_key() {
        run_get_user_test(&CreateUsersDef::new(1, 1, MazeContent::Empty), 
                          None, false, 
                          VALID_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_get_user_that_exists_with_admin_caller_with_api_key() {
        run_can_get_user_that_exists_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_get_user_that_exists_with_admin_caller_with_login() {
        run_can_get_user_that_exists_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn can_get_admin_user_that_exists_with_admin_caller_with_api_key() {
        run_can_get_admin_user_that_exists_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_get_admin_user_that_exists_with_admin_caller_with_login() {
        run_can_get_admin_user_that_exists_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_exists_with_non_admin_caller_with_api_key() {
        run_cannot_get_user_that_exists_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_exists_with_non_admin_caller_with_login() {
        run_cannot_get_user_that_exists_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_does_not_exist_with_admin_caller_with_api_key() {
        run_cannot_get_user_that_does_not_exist_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_user_that_does_not_exist_with_admin_caller_with_login() {
        run_cannot_get_user_that_does_not_exist_with_admin_caller(true).await;
    }

    // Update user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_update_admin_user_with_invalid_api_key() {
        run_update_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, NEW_ADMIN_USERNAME_1,
            &new_update_user_request(true, NEW_ADMIN_USERNAME_1, None), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_update_admin_user_with_admin_caller_with_api_key() {
        run_can_update_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_update_admin_user_with_admin_caller_with_login() {
        run_can_update_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_update_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_non_admin_caller_with_login() {
        run_cannot_update_admin_user_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_missing_username_with_api_key() {
        run_cannot_update_admin_user_with_admin_caller_but_missing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_missing_username_with_login() {
        run_cannot_update_admin_user_with_admin_caller_but_missing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_username_with_api_key() {
        run_cannot_update_admin_user_with_admin_caller_but_existing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_username_with_login() {
        run_cannot_update_admin_user_with_admin_caller_but_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_email_with_api_key() {
        run_cannot_update_admin_user_with_admin_caller_but_existing_email(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_admin_user_with_admin_caller_but_existing_email_with_login() {
        run_cannot_update_admin_user_with_admin_caller_but_existing_email(true).await;
    }

    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_update_non_admin_user_with_invalid_api_key() {
        run_update_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, NEW_ADMIN_USERNAME_1,
            &new_update_user_request(false, NEW_ADMIN_USERNAME_1, None), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_update_non_admin_user_with_admin_caller_with_api_key() {
        run_can_update_non_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_update_non_admin_user_with_admin_caller_with_login() {
        run_can_update_non_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_missing_username_with_api_key() {
        run_cannot_update_non_admin_user_with_admin_caller_but_missing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_missing_username_with_login() {
        run_cannot_update_non_admin_user_with_admin_caller_but_missing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_username_with_api_key() {
        run_cannot_update_non_admin_user_with_admin_caller_but_existing_username(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_username_with_login() {
        run_cannot_update_non_admin_user_with_admin_caller_but_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_email_with_api_key() {
        run_cannot_update_non_admin_user_with_admin_caller_but_existing_email(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_non_admin_user_with_admin_caller_but_existing_email_with_login() {
        run_cannot_update_non_admin_user_with_admin_caller_but_existing_email(true).await;
    }

    #[actix_web::test]
    async fn can_upgrade_non_admin_user_to_admin_with_admin_caller_with_api_key() {
        run_can_upgrade_non_admin_user_to_admin_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_upgrade_non_admin_user_to_admin_with_admin_caller_with_login() {
        run_can_upgrade_non_admin_user_to_admin_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn can_downgrade_admin_user_to_non_admin_with_admin_caller_with_api_key() {
        run_can_downgrade_admin_user_to_non_admin_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_downgrade_admin_user_to_non_admin_with_admin_caller_with_login() {
        run_can_downgrade_admin_user_to_non_admin_with_admin_caller(true).await;
    }

    // Delete user
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_delete_user_with_invalid_api_key() {
        run_delete_user_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), 
            None, false, NEW_ADMIN_USERNAME_1, StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_admin_user_with_admin_caller_with_api_key() {
        run_can_delete_existing_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_admin_user_with_admin_caller_with_login() {
        run_can_delete_existing_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_last_admin_user_with_admin_caller_with_api_key() {
        run_cannot_delete_last_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_last_admin_user_with_admin_caller_with_login() {
        run_cannot_delete_last_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_admin_user_with_admin_caller_with_api_key() {
        run_cannot_delete_non_existent_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_admin_user_with_admin_caller_with_login() {
        run_cannot_delete_non_existent_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_non_admin_user_with_admin_caller_with_api_key() {
        run_can_delete_existing_non_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn can_delete_existing_non_admin_user_with_admin_caller_with_login() {
        run_can_delete_existing_non_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_non_admin_user_with_admin_caller_with_api_key() {
        run_cannot_delete_non_existent_non_admin_user_with_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_non_existent_non_admin_user_with_admin_caller_with_login() {
        run_cannot_delete_non_existent_non_admin_user_with_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_delete_existing_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_admin_user_with_non_admin_caller_with_login() {
        run_cannot_delete_existing_admin_user_with_non_admin_caller(true).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_non_admin_user_with_non_admin_caller_with_api_key() {
        run_cannot_delete_existing_non_admin_user_with_non_admin_caller(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_existing_non_admin_user_with_non_admin_caller_with_login() {
        run_cannot_delete_existing_non_admin_user_with_non_admin_caller(true).await;
    }

    /**********/
    /* Mazes  */
    /**********/

    // Get mazes
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_mazes_with_no_mazes_with_invalid_api_key() {
        run_get_mazes_test(&CreateUsersDef::new(0, 0, MazeContent::Empty), None, false, false, MazeContent::Empty).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_no_mazes_with_api_key() {
        run_can_get_mazes_with_no_mazes(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_no_mazes_with_login() {
        run_can_get_mazes_with_no_mazes(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_without_definitions_with_api_key() {
        run_can_get_mazes_with_one_maze_without_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_without_definitions_with_login() {
        run_can_get_mazes_with_one_maze_without_definitions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_with_defintions_with_api_key() {
        run_can_get_mazes_with_one_maze_with_defintions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_one_maze_with_defintions_with_login() {
        run_can_get_mazes_with_one_maze_with_defintions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_without_definitions_with_api_key() {
        run_can_get_mazes_with_two_mazes_that_require_sorting_without_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_without_definitions_with_login() {
        run_can_get_mazes_with_two_mazes_that_require_sorting_without_definitions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_with_definitions_with_api_key() {
        run_can_get_mazes_with_two_mazes_that_require_sorting_with_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_two_mazes_that_require_sorting_with_definitions_with_login() {
        run_can_get_mazes_with_two_mazes_that_require_sorting_with_definitions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_without_definitions_with_api_key() {
        run_can_get_mazes_with_three_mazes_that_require_sorting_without_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_without_definitions_with_login() {
        run_can_get_mazes_with_three_mazes_that_require_sorting_without_definitions(true).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_with_definitions_with_api_key() {
        run_can_get_mazes_with_three_mazes_that_require_sorting_with_definitions(false).await;
    }

    #[actix_web::test]
    async fn can_get_mazes_with_three_mazes_that_require_sorting_with_definitions_with_login() {
        run_can_get_mazes_with_three_mazes_that_require_sorting_with_definitions(true).await;
    }

    // Create maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_create_maze_that_does_not_exist_with_invalid_api_key() {
        run_create_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), false, new_solvable_maze("", "maze_d"), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_create_maze_that_does_not_exist_with_api_key() {
        run_can_create_maze_that_does_not_exist(false).await;
    }

    #[actix_web::test]
    async fn can_create_maze_that_does_not_exist_with_login() {
        run_can_create_maze_that_does_not_exist(true).await;
    }

    #[actix_web::test]
    async fn cannot_create_maze_that_already_exists_with_api_key() {
        run_cannot_create_maze_that_already_exists(false).await;
    }

    #[actix_web::test]
    async fn cannot_create_maze_that_already_exists_with_login() {
        run_cannot_create_maze_that_already_exists(true).await;
    }

    // Get maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_maze_that_exists_with_invalid_api_key() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_get_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), false, id, StatusCode::UNAUTHORIZED, Some(new_solvable_maze(id, name))).await;
    }

    #[actix_web::test]
    async fn can_get_maze_that_exists_with_api_key() {
        run_can_get_maze_that_exists(false).await;
    }

    #[actix_web::test]
    async fn can_get_maze_that_exists_with_login() {
        run_can_get_maze_that_exists(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_that_does_not_exist_with_api_key() {
        run_cannot_get_maze_that_does_not_exist(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_that_does_not_exist_with_login() {
        run_cannot_get_maze_that_does_not_exist(true).await;
    }

    // Update maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_update_maze_that_exists_with_invalid_api_key() {
        let id = "maze_a.json";
        let name = "maze_a";
        run_update_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), false, id, new_solvable_maze(id, name), StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_update_maze_that_exists_with_api_key() {
        run_can_update_maze_that_exists(false).await;
    }

    #[actix_web::test]
    async fn can_update_maze_that_exists_with_login() {
        run_can_update_maze_that_exists(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_that_does_not_exist_with_api_key() {
        run_cannot_update_maze_that_does_not_exist(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_that_does_not_exist_with_login() {
        run_cannot_update_maze_that_does_not_exist(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_with_mismatching_id_with_api_key() {
        run_cannot_update_maze_with_mismatching_id(false).await;
    }

    #[actix_web::test]
    async fn cannot_update_maze_with_mismatching_id_with_login() {
        run_cannot_update_maze_with_mismatching_id(true).await;
    }

    // Delete maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_delete_maze_that_exists_with_invalid_api_key() {
        run_delete_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(INVALID_USERNAME), false, "maze_a.json", StatusCode::UNAUTHORIZED).await;
    }

    #[actix_web::test]
    async fn can_delete_maze_that_exists() {
        run_delete_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes),Some(VALID_USERNAME_1), false, "maze_a.json", StatusCode::OK).await;
    }

    #[actix_web::test]
    async fn cannot_delete_maze_that_does_not_exist() {
        run_delete_maze_test(&CreateUsersDef::new(0, 1, MazeContent::ThreeMazes), Some(VALID_USERNAME_1), false, "does_not_exist.json", StatusCode:: NOT_FOUND).await;
    }

    // Get maze solution
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_get_maze_solution_that_should_succeed_with_invalid_api_key() {
        run_get_maze_solution_test(
            &CreateUsersDef::new(0, 1, MazeContent::SolutionTestMazes),
            Some(INVALID_USERNAME), false, "solvable.json", StatusCode::UNAUTHORIZED,
            Some(get_solve_test_maze_solution()), None
        ).await;
    }

    #[actix_web::test]
    async fn can_get_maze_solution_that_should_succeed_with_api_key() {
        run_can_get_maze_solution_that_should_succeed(false).await;
    }

    #[actix_web::test]
    async fn can_get_maze_solution_that_should_succeed_with_login() {
        run_can_get_maze_solution_that_should_succeed(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_start_with_api_key() {
        run_cannot_get_maze_solution_that_should_fail_with_no_start(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_start_with_login() {
        run_cannot_get_maze_solution_that_should_fail_with_no_start(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_finish_with_api_key() {
        run_cannot_get_maze_solution_that_should_fail_with_no_finish(false).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_finish_with_login() {
        run_cannot_get_maze_solution_that_should_fail_with_no_finish(true).await;
    }

    #[actix_web::test]
    async fn cannot_get_maze_solution_that_should_fail_with_no_solution_with_api_key() {
        run_cannot_get_maze_solution_that_should_fail_with_no_solution(false).await;
    }

    // Solve maze
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn canot_solve_maze_that_should_succeed_with_invalid_api_key() {
        run_solve_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(INVALID_USERNAME),
            false,
            new_solve_test_maze("", "", true, true, false),
            StatusCode::UNAUTHORIZED,
            Some(get_solve_test_maze_solution()),
            None
        ).await;
    }

    #[actix_web::test]
    async fn can_solve_maze_that_should_succeed_with_api_key() {
        run_can_solve_maze_that_should_succeed(false).await;
    }

    #[actix_web::test]
    async fn can_solve_maze_that_should_succeed_with_login() {
        run_can_solve_maze_that_should_succeed(true).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_start_with_api_key() {
        run_cannot_solve_maze_that_should_fail_with_no_start(false).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_start_with_login() {
        run_cannot_solve_maze_that_should_fail_with_no_start(true).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_yhat_should_fail_with_no_finish_with_api_key() {
        run_cannot_solve_maze_yhat_should_fail_with_no_finish(false).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_yhat_should_fail_with_no_finish_with_login() {
        run_cannot_solve_maze_yhat_should_fail_with_no_finish(true).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_solution_with_api_key() {
        run_cannot_solve_maze_that_should_fail_with_no_solution(false).await;
    }

    #[actix_web::test]
    async fn cannot_solve_maze_that_should_fail_with_no_solution_with_login() {
        run_cannot_solve_maze_that_should_fail_with_no_solution(true).await;
    }

    // **************************************************************************************************
    // Generate maze helpers
    // **************************************************************************************************

    fn new_generate_options(
        row_count: usize,
        col_count: usize,
        start: Option<MazePoint>,
        finish: Option<MazePoint>,
        min_spine_length: Option<usize>,
        max_retries: Option<usize>,
    ) -> GeneratorOptions {
        GeneratorOptions {
            row_count,
            col_count,
            algorithm: GenerationAlgorithm::RecursiveBacktracking,
            start,
            finish,
            min_spine_length,
            max_retries,
            branch_from_finish: None,
            seed: None,
        }
    }

    async fn validate_generate_response(
        context: &str,
        resp: actix_web::dev::ServiceResponse,
        expected_status_code: StatusCode,
        expected_rows: Option<usize>,
        expected_cols: Option<usize>,
        expected_err_message: Option<String>,
    ) {
        assert_eq!(resp.status(), expected_status_code);
        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let maze: Maze = serde_json::from_slice(&body).expect("failed to deserialize response");
            match (expected_rows, expected_cols) {
                (Some(rows), Some(cols)) => {
                    assert_eq!(maze.definition.row_count(), rows);
                    assert_eq!(maze.definition.col_count(), cols);
                }
                _ => panic!("{}", format!("No maze dimension comparison values provided for {context} test!")),
            }
            assert_eq!(maze.id, "", "{}: expected empty id", context);
            assert_eq!(maze.name, "", "{}: expected empty name", context);
            maze.solve().unwrap_or_else(|_| panic!("{context}: generated maze must be solvable"));
        } else if let Some(value) = expected_err_message {
            let body = test::read_body(resp).await;
            let error_message = String::from_utf8(body.to_vec()).expect("Failed to parse body as UTF-8");
            assert_eq!(error_message, value);
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_generate_maze_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        options: GeneratorOptions,
        expected_status_code: StatusCode,
        expected_rows: Option<usize>,
        expected_cols: Option<usize>,
        expected_err_message: Option<String>,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/mazes/generate".to_string();
        let req = create_test_post_request(&url, api_key, login_id, Some(&options));
        let resp = test::call_service(&app, req).await;
        validate_generate_response("generate_maze()", resp, expected_status_code, expected_rows, expected_cols, expected_err_message).await;
    }

    fn get_generate_row_count_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("row_count must be at least 3".to_string()))
    }

    fn get_generate_col_count_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("col_count must be at least 3".to_string()))
    }

    fn get_generate_start_out_of_bounds_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("start is out of bounds".to_string()))
    }

    fn get_generate_finish_out_of_bounds_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("finish is out of bounds".to_string()))
    }

    fn get_generate_start_equals_finish_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("start and finish must be different cells".to_string()))
    }

    fn get_generate_max_retries_zero_error_str() -> String {
        get_maze_generate_error_string(&MazeError::Generate("max_retries is 0, no attempts made".to_string()))
    }

    async fn run_can_generate_maze_that_should_succeed(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, None, None, None),
            StatusCode::OK, Some(5), Some(5), None,
        ).await;
    }

    async fn run_can_generate_maze_with_minimum_row_count(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(3, 5, None, None, None, None),
            StatusCode::OK, Some(3), Some(5), None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_row_count_too_small(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(2, 5, None, None, None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_row_count_error_str()),
        ).await;
    }

    async fn run_can_generate_maze_with_minimum_col_count(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 3, None, None, None, None),
            StatusCode::OK, Some(5), Some(3), None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_col_count_too_small(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 2, None, None, None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_col_count_error_str()),
        ).await;
    }

    async fn run_can_generate_maze_with_explicit_start_and_finish(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, Some(MazePoint { row: 0, col: 0 }), Some(MazePoint { row: 4, col: 4 }), None, None),
            StatusCode::OK, Some(5), Some(5), None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_start_out_of_bounds(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, Some(MazePoint { row: 10, col: 10 }), None, None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_start_out_of_bounds_error_str()),
        ).await;
    }

    async fn run_cannot_generate_maze_with_finish_out_of_bounds(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, Some(MazePoint { row: 10, col: 10 }), None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_finish_out_of_bounds_error_str()),
        ).await;
    }

    async fn run_cannot_generate_maze_with_start_equals_finish(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, Some(MazePoint { row: 0, col: 0 }), Some(MazePoint { row: 0, col: 0 }), None, None),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_start_equals_finish_error_str()),
        ).await;
    }

    async fn run_can_generate_maze_with_valid_min_spine_length(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, None, Some(3), None),
            StatusCode::OK, Some(5), Some(5), None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_impossible_min_spine_length(use_login: bool) {
        // min_spine_length=1000 is impossible for a 5×5 maze; max_retries=1 keeps the test fast
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, None, Some(1000), Some(1)),
            StatusCode::UNPROCESSABLE_ENTITY, None, None, None,
        ).await;
    }

    async fn run_cannot_generate_maze_with_max_retries_zero(use_login: bool) {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1), use_login,
            new_generate_options(5, 5, None, None, None, Some(0)),
            StatusCode::UNPROCESSABLE_ENTITY, None, None,
            Some(get_generate_max_retries_zero_error_str()),
        ).await;
    }

    // Generate maze tests
    #[actix_web::test]
    #[should_panic(expected = "Unauthorized request")]
    async fn cannot_generate_maze_with_invalid_api_key() {
        run_generate_maze_test(
            &CreateUsersDef::new(0, 1, MazeContent::Empty),
            Some(INVALID_USERNAME), false,
            new_generate_options(5, 5, None, None, None, None),
            StatusCode::UNAUTHORIZED, None, None, None,
        ).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_that_should_succeed_with_api_key() {
        run_can_generate_maze_that_should_succeed(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_that_should_succeed_with_login() {
        run_can_generate_maze_that_should_succeed(true).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_minimum_row_count_with_api_key() {
        run_can_generate_maze_with_minimum_row_count(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_minimum_row_count_with_login() {
        run_can_generate_maze_with_minimum_row_count(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_row_count_too_small_with_api_key() {
        run_cannot_generate_maze_with_row_count_too_small(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_row_count_too_small_with_login() {
        run_cannot_generate_maze_with_row_count_too_small(true).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_minimum_col_count_with_api_key() {
        run_can_generate_maze_with_minimum_col_count(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_minimum_col_count_with_login() {
        run_can_generate_maze_with_minimum_col_count(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_col_count_too_small_with_api_key() {
        run_cannot_generate_maze_with_col_count_too_small(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_col_count_too_small_with_login() {
        run_cannot_generate_maze_with_col_count_too_small(true).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_explicit_start_and_finish_with_api_key() {
        run_can_generate_maze_with_explicit_start_and_finish(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_explicit_start_and_finish_with_login() {
        run_can_generate_maze_with_explicit_start_and_finish(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_start_out_of_bounds_with_api_key() {
        run_cannot_generate_maze_with_start_out_of_bounds(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_start_out_of_bounds_with_login() {
        run_cannot_generate_maze_with_start_out_of_bounds(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_finish_out_of_bounds_with_api_key() {
        run_cannot_generate_maze_with_finish_out_of_bounds(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_finish_out_of_bounds_with_login() {
        run_cannot_generate_maze_with_finish_out_of_bounds(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_start_equals_finish_with_api_key() {
        run_cannot_generate_maze_with_start_equals_finish(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_start_equals_finish_with_login() {
        run_cannot_generate_maze_with_start_equals_finish(true).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_valid_min_spine_length_with_api_key() {
        run_can_generate_maze_with_valid_min_spine_length(false).await;
    }

    #[actix_web::test]
    async fn can_generate_maze_with_valid_min_spine_length_with_login() {
        run_can_generate_maze_with_valid_min_spine_length(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_impossible_min_spine_length_with_api_key() {
        run_cannot_generate_maze_with_impossible_min_spine_length(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_impossible_min_spine_length_with_login() {
        run_cannot_generate_maze_with_impossible_min_spine_length(true).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_max_retries_zero_with_api_key() {
        run_cannot_generate_maze_with_max_retries_zero(false).await;
    }

    #[actix_web::test]
    async fn cannot_generate_maze_with_max_retries_zero_with_login() {
        run_cannot_generate_maze_with_max_retries_zero(true).await;
    }

    // **************************************************************************************************
    // signup / get_me / delete_me helpers
    // **************************************************************************************************

    fn new_signup_request(email: &str, blank_password: bool) -> SignupRequest {
        SignupRequest {
            email: email.to_string(),
            password: create_password(blank_password),
        }
    }

    async fn run_signup_test(
        create_users_def: &CreateUsersDef,
        signup_req: &SignupRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        // No caller — signup is an unguarded endpoint
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let url = "/api/v1/signup".to_string();
        let req = create_test_post_request(&url, None, None, Some(signup_req));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::CREATED {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize signup response");
            // is_admin must always be false regardless of what the caller sends
            assert!(!response_user.is_admin, "signup must never create an admin user");
            assert!(!response_user.username.is_empty(), "auto-generated username must not be empty");
            assert_eq!(response_user.email, signup_req.email);
            assert_ne!(response_user.id, Uuid::nil());
        }
    }

    async fn run_get_me_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users/me".to_string();
        let req = create_test_get_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize get_me response");
            // Verify the returned profile matches the caller's own data
            if let Some(username) = caller_username {
                let caller_id = MockStore::find_user_id_by_name_in_map(&mock_users, username, Uuid::nil());
                let dummy_user = MockUser::default();
                let expected_user = mock_users.get(&caller_id).unwrap_or(&dummy_user);
                assert_eq!(response_user, expected_user.to_user_item());
            }
        }
    }

    async fn run_delete_me_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, shared_store, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users/me".to_string();
        let req = create_test_delete_request(&url, api_key, login_id);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::NO_CONTENT {
            // Verify the caller's account is gone from the store
            if let Some(username) = caller_username {
                let store_lock = get_store_read_lock(&shared_store).await;
                assert!(
                    store_lock.find_user_by_name(username).await.is_err(),
                    "user '{username}' should have been deleted but was still found"
                );
            }
        }
    }

    // **************************************************************************************************
    // Tests: POST /api/v1/signup
    // **************************************************************************************************

    #[actix_web::test]
    async fn signup_with_valid_details_succeeds() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &new_signup_request(&new_email(NEW_USERNAME_1), false),
            StatusCode::CREATED,
        ).await;
    }

    #[actix_web::test]
    async fn signup_always_creates_non_admin_user() {
        // Even if an attacker crafts a request with is_admin, SignupRequest has no such field
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &new_signup_request(&new_email(NEW_USERNAME_1), false),
            StatusCode::CREATED,
        ).await;
    }

    #[actix_web::test]
    async fn can_signup_and_username_is_generated_from_email() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(0, 0, MazeContent::Empty));
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_post_request("/api/v1/signup", None, None, Some(&SignupRequest {
            email: VALID_USER_EMAIL_1.to_string(),
            password: VALID_USER_PASSWORD.to_string(),
        }));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
        let body = test::read_body(resp).await;
        let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize signup response");
        assert!(response_user.username.starts_with(VALID_USERNAME_1), "expected username to start with '{}', got '{}'", VALID_USERNAME_1, response_user.username);
    }

    #[actix_web::test]
    async fn signup_with_duplicate_email_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &new_signup_request(&new_email(VALID_USERNAME_1), false),
            StatusCode::CONFLICT,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_blank_password_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &new_signup_request(&new_email(NEW_USERNAME_1), true),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_short_password_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "Abc1!".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_no_uppercase_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "password1!".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_no_lowercase_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "PASSWORD1!".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_no_digit_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "Password!".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    #[actix_web::test]
    async fn signup_with_no_special_character_fails() {
        run_signup_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            &SignupRequest { email: new_email(NEW_USERNAME_1), password: "Password1".to_string() },
            StatusCode::BAD_REQUEST,
        ).await;
    }

    // **************************************************************************************************
    // Tests: GET /api/v1/users/me
    // **************************************************************************************************

    #[actix_web::test]
    async fn get_me_as_regular_user_with_api_key_succeeds() {
        run_get_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            false,
            StatusCode::OK,
        ).await;
    }

    #[actix_web::test]
    async fn get_me_as_regular_user_with_login_succeeds() {
        run_get_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            true,
            StatusCode::OK,
        ).await;
    }

    #[actix_web::test]
    async fn get_me_as_admin_with_login_succeeds() {
        run_get_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_ADMIN_USERNAME_1),
            true,
            StatusCode::OK,
        ).await;
    }

    #[actix_web::test]
    #[should_panic]
    async fn get_me_unauthenticated_fails() {
        run_get_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            None,
            false,
            StatusCode::UNAUTHORIZED,
        ).await;
    }

    // **************************************************************************************************
    // Tests: DELETE /api/v1/users/me
    // **************************************************************************************************

    #[actix_web::test]
    async fn delete_me_with_api_key_succeeds() {
        run_delete_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            false,
            StatusCode::NO_CONTENT,
        ).await;
    }

    #[actix_web::test]
    async fn delete_me_with_login_succeeds() {
        run_delete_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            true,
            StatusCode::NO_CONTENT,
        ).await;
    }

    #[actix_web::test]
    #[should_panic]
    async fn delete_me_unauthenticated_fails() {
        run_delete_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            None,
            false,
            StatusCode::UNAUTHORIZED,
        ).await;
    }

    #[actix_web::test]
    async fn delete_me_removes_user_from_store() {
        // Verifies the user is gone and subsequent auth with deleted credentials returns 401
        run_delete_me_test(
            &CreateUsersDef::new(1, 2, MazeContent::OneMaze),
            Some(VALID_USERNAME_1),
            false,
            StatusCode::NO_CONTENT,
        ).await;
    }

    #[actix_web::test]
    async fn cannot_delete_me_when_last_admin_with_api_key() {
        run_cannot_delete_me_when_last_admin(false).await;
    }

    #[actix_web::test]
    async fn cannot_delete_me_when_last_admin_with_login() {
        run_cannot_delete_me_when_last_admin(true).await;
    }

    #[actix_web::test]
    async fn can_delete_me_when_not_last_admin_with_api_key() {
        run_can_delete_me_when_not_last_admin(false).await;
    }

    #[actix_web::test]
    async fn can_delete_me_when_not_last_admin_with_login() {
        run_can_delete_me_when_not_last_admin(true).await;
    }

    // **************************************************************************************************
    // change_password_me / update_profile_me helpers
    // **************************************************************************************************

    impl ChangePasswordRequest {
        pub fn new(current_password: &str, new_password: &str) -> ChangePasswordRequest {
            ChangePasswordRequest {
                current_password: current_password.to_string(),
                new_password: new_password.to_string(),
            }
        }
    }

    impl UpdateProfileRequest {
        pub fn new(username: &str, full_name: &str, email: &str) -> UpdateProfileRequest {
            UpdateProfileRequest {
                username: username.to_string(),
                full_name: full_name.to_string(),
                email: email.to_string(),
            }
        }

        pub fn to_user_item(&self) -> UserItem {
            UserItem {
                id: Uuid::nil(),
                is_admin: false,
                username: self.username.clone(),
                full_name: self.full_name.clone(),
                email: self.email.clone(),
            }
        }
    }

    fn new_update_profile_request(username: &str, email: Option<&str>) -> UpdateProfileRequest {
        let email_use = email.unwrap_or(&new_email(username)).to_string();
        UpdateProfileRequest::new(username, &format!("Updated {username} full name"), &email_use)
    }

    async fn run_change_password_me_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        change_req: &ChangePasswordRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, _, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users/me/password".to_string();
        let req = create_test_put_request(&url, api_key, login_id, change_req);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);
    }

    async fn run_update_profile_me_test(
        create_users_def: &CreateUsersDef,
        caller_username: Option<&str>,
        use_login: bool,
        update_req: &UpdateProfileRequest,
        expected_status_code: StatusCode,
    ) {
        let mut user_defs = create_user_defs(create_users_def);
        let (app, _, mock_users, api_key, login_id) = create_test_app(&mut user_defs, caller_username, use_login).await;
        let url = "/api/v1/users/me/profile".to_string();
        let req = create_test_put_request(&url, api_key, login_id, update_req);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), expected_status_code);

        if expected_status_code == StatusCode::OK {
            let body = test::read_body(resp).await;
            let response_user: UserItem = serde_json::from_slice(&body).expect("failed to deserialize update_profile_me response");
            // id and is_admin come from the authenticated caller, not the request
            let caller_id = MockStore::find_user_id_by_name_in_map(&mock_users, caller_username.unwrap_or(""), Uuid::nil());
            let dummy_user = MockUser::default();
            let original_user = mock_users.get(&caller_id).unwrap_or(&dummy_user);
            assert_eq!(response_user.id, original_user.user.id);
            assert_eq!(response_user.is_admin, original_user.user.is_admin);
            assert_eq!(response_user.username, update_req.username);
            assert_eq!(response_user.full_name, update_req.full_name);
            assert_eq!(response_user.email, update_req.email);
        }
    }

    // change_password_me scenario helpers
    async fn run_can_change_password_with_valid_current_password(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NewPassword1!"),
            StatusCode::NO_CONTENT,
        ).await;
    }

    async fn run_cannot_change_password_with_wrong_current_password(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(INVALID_USER_PASSWORD, "NewPassword1!"),
            StatusCode::UNAUTHORIZED,
        ).await;
    }

    async fn run_cannot_change_password_with_empty_current_password(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new("", "NewPassword1!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_too_short(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "Sh0rt!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_no_uppercase(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "nouppercase1!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_no_lowercase(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NOLOWERCASE1!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_no_digit(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NoDigitHere!"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_change_password_with_new_password_no_special_char(use_login: bool) {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NoSpecial1"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    // update_profile_me scenario helpers
    async fn run_can_update_profile_with_new_username_and_email(use_login: bool) {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &new_update_profile_request("updated_username_1", None),
            StatusCode::OK,
        ).await;
    }

    async fn run_can_update_profile_keeping_same_username_and_email(use_login: bool) {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &new_update_profile_request(VALID_USERNAME_1, None),
            StatusCode::OK,
        ).await;
    }

    async fn run_cannot_update_profile_with_existing_username(use_login: bool) {
        // user_2 tries to take user_1's username
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 2, MazeContent::Empty),
            Some(VALID_USERNAME_2),
            use_login,
            &new_update_profile_request(VALID_USERNAME_1, None),
            StatusCode::CONFLICT,
        ).await;
    }

    async fn run_cannot_update_profile_with_existing_email(use_login: bool) {
        // user_2 tries to take user_1's email
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 2, MazeContent::Empty),
            Some(VALID_USERNAME_2),
            use_login,
            &new_update_profile_request(VALID_USERNAME_2, Some(&new_email(VALID_USERNAME_1))),
            StatusCode::CONFLICT,
        ).await;
    }

    async fn run_cannot_update_profile_with_empty_username(use_login: bool) {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &UpdateProfileRequest::new("", "Some Full Name", &new_email(VALID_USERNAME_1)),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    async fn run_cannot_update_profile_with_invalid_email(use_login: bool) {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            Some(VALID_USERNAME_1),
            use_login,
            &UpdateProfileRequest::new(VALID_USERNAME_1, "Some Full Name", "not-a-valid-email"),
            StatusCode::BAD_REQUEST,
        ).await;
    }

    // change_password_me tests
    #[actix_web::test]
    async fn can_change_password_with_valid_current_password_with_api_key() {
        run_can_change_password_with_valid_current_password(false).await;
    }
    #[actix_web::test]
    async fn can_change_password_with_valid_current_password_with_login() {
        run_can_change_password_with_valid_current_password(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_wrong_current_password_with_api_key() {
        run_cannot_change_password_with_wrong_current_password(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_wrong_current_password_with_login() {
        run_cannot_change_password_with_wrong_current_password(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_empty_current_password_with_api_key() {
        run_cannot_change_password_with_empty_current_password(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_empty_current_password_with_login() {
        run_cannot_change_password_with_empty_current_password(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_too_short_with_api_key() {
        run_cannot_change_password_with_new_password_too_short(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_too_short_with_login() {
        run_cannot_change_password_with_new_password_too_short(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_uppercase_with_api_key() {
        run_cannot_change_password_with_new_password_no_uppercase(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_uppercase_with_login() {
        run_cannot_change_password_with_new_password_no_uppercase(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_lowercase_with_api_key() {
        run_cannot_change_password_with_new_password_no_lowercase(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_lowercase_with_login() {
        run_cannot_change_password_with_new_password_no_lowercase(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_digit_with_api_key() {
        run_cannot_change_password_with_new_password_no_digit(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_digit_with_login() {
        run_cannot_change_password_with_new_password_no_digit(true).await;
    }

    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_special_char_with_api_key() {
        run_cannot_change_password_with_new_password_no_special_char(false).await;
    }
    #[actix_web::test]
    async fn cannot_change_password_with_new_password_no_special_char_with_login() {
        run_cannot_change_password_with_new_password_no_special_char(true).await;
    }

    #[actix_web::test]
    #[should_panic]
    async fn cannot_change_password_unauthenticated() {
        run_change_password_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            None,
            false,
            &ChangePasswordRequest::new(VALID_USER_PASSWORD, "NewPassword1!"),
            StatusCode::UNAUTHORIZED,
        ).await;
    }

    // update_profile_me tests
    #[actix_web::test]
    async fn can_update_profile_with_new_username_and_email_with_api_key() {
        run_can_update_profile_with_new_username_and_email(false).await;
    }
    #[actix_web::test]
    async fn can_update_profile_with_new_username_and_email_with_login() {
        run_can_update_profile_with_new_username_and_email(true).await;
    }

    #[actix_web::test]
    async fn can_update_profile_keeping_same_username_and_email_with_api_key() {
        run_can_update_profile_keeping_same_username_and_email(false).await;
    }
    #[actix_web::test]
    async fn can_update_profile_keeping_same_username_and_email_with_login() {
        run_can_update_profile_keeping_same_username_and_email(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_profile_with_existing_username_with_api_key() {
        run_cannot_update_profile_with_existing_username(false).await;
    }
    #[actix_web::test]
    async fn cannot_update_profile_with_existing_username_with_login() {
        run_cannot_update_profile_with_existing_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_profile_with_existing_email_with_api_key() {
        run_cannot_update_profile_with_existing_email(false).await;
    }
    #[actix_web::test]
    async fn cannot_update_profile_with_existing_email_with_login() {
        run_cannot_update_profile_with_existing_email(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_profile_with_empty_username_with_api_key() {
        run_cannot_update_profile_with_empty_username(false).await;
    }
    #[actix_web::test]
    async fn cannot_update_profile_with_empty_username_with_login() {
        run_cannot_update_profile_with_empty_username(true).await;
    }

    #[actix_web::test]
    async fn cannot_update_profile_with_invalid_email_with_api_key() {
        run_cannot_update_profile_with_invalid_email(false).await;
    }
    #[actix_web::test]
    async fn cannot_update_profile_with_invalid_email_with_login() {
        run_cannot_update_profile_with_invalid_email(true).await;
    }

    #[actix_web::test]
    #[should_panic]
    async fn cannot_update_profile_unauthenticated() {
        run_update_profile_me_test(
            &CreateUsersDef::new(1, 1, MazeContent::Empty),
            None,
            false,
            &new_update_profile_request(VALID_USERNAME_1, None),
            StatusCode::UNAUTHORIZED,
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

    // **************************************************************************************************
    // Tests: GET /api/v1/features
    // **************************************************************************************************
    #[actix_web::test]
    async fn get_features_returns_defaults() {
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        let req = create_test_get_request("/api/v1/features", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let response: AppFeaturesResponse = serde_json::from_slice(&body).expect("failed to deserialize features response");
        assert!(response.allow_signup);
    }

    #[actix_web::test]
    async fn get_features_respects_config() {
        let mut user_defs = vec![];
        let features = AppFeaturesConfig { allow_signup: false };
        let features: SharedFeatures = Arc::new(RwLock::new(features));
        let (app, _, _, _, _) = create_test_app_with_features(&mut user_defs, None, false, features).await;
        let req = create_test_get_request("/api/v1/features", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let response: AppFeaturesResponse = serde_json::from_slice(&body).expect("failed to deserialize features response");
        assert!(!response.allow_signup);
    }

    #[actix_web::test]
    async fn get_features_no_auth_required() {
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, _, _) = create_test_app(&mut user_defs, None, false).await;
        // No api_key or login_id — endpoint must be accessible without authentication
        let req = create_test_get_request("/api/v1/features", None, None);
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // **************************************************************************************************
    // Tests: PUT /api/v1/admin/features
    // **************************************************************************************************

    fn make_admin_features_config_toml(allow_signup: bool) -> (AppConfig, std::path::PathBuf) {
        let temp_path = std::env::temp_dir().join(format!("maze_test_{}.toml", Uuid::new_v4()));
        std::fs::write(&temp_path, format!("[features]\nallow_signup = {allow_signup}\n")).unwrap();
        let config = AppConfig { config_path: temp_path.to_string_lossy().to_string(), ..AppConfig::default() };
        (config, temp_path)
    }

    #[actix_web::test]
    async fn cannot_update_admin_features_with_non_admin_caller_with_api_key() {
        let admin_username = &format!("{ADMIN_USERNAME_PREFIX}1");
        let non_admin_username = &format!("{USERNAME_PREFIX}1");
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, api_key, _) = create_test_app(&mut user_defs, Some(non_admin_username), false).await;
        let _ = admin_username;
        let req = create_test_put_request("/api/v1/admin/features", api_key, None, &AppFeaturesResponse { allow_signup: false, ..Default::default() });
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn cannot_update_admin_features_with_non_admin_caller_with_login() {
        let non_admin_username = &format!("{USERNAME_PREFIX}1");
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 1, MazeContent::Empty));
        let (app, _, _, _, login_id) = create_test_app(&mut user_defs, Some(non_admin_username), true).await;
        let req = create_test_put_request("/api/v1/admin/features", None, login_id, &AppFeaturesResponse { allow_signup: false, ..Default::default() });
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[actix_web::test]
    async fn update_admin_features_updates_live_state() {
        let admin_username = &format!("{ADMIN_USERNAME_PREFIX}1");
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 0, MazeContent::Empty));
        let (app_config, temp_path) = make_admin_features_config_toml(true);
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let (app, _, _, api_key, _) = create_test_app_with_config(&mut user_defs, Some(admin_username), false, features, app_config).await;

        // Disable signup via admin PUT
        let put_req = create_test_put_request("/api/v1/admin/features", api_key, None, &AppFeaturesResponse { allow_signup: false, ..Default::default() });
        let put_resp = test::call_service(&app, put_req).await;
        assert_eq!(put_resp.status(), StatusCode::OK);
        let body = test::read_body(put_resp).await;
        let response: AppFeaturesResponse = serde_json::from_slice(&body).expect("failed to deserialize response");
        assert!(!response.allow_signup);

        // GET /features now reflects the new value
        let get_req = create_test_get_request("/api/v1/features", None, None);
        let get_resp = test::call_service(&app, get_req).await;
        let body = test::read_body(get_resp).await;
        let features_response: AppFeaturesResponse = serde_json::from_slice(&body).expect("failed to deserialize features response");
        assert!(!features_response.allow_signup);

        let _ = std::fs::remove_file(&temp_path);
    }

    #[actix_web::test]
    async fn update_admin_features_persists_to_config_toml() {
        let admin_username = &format!("{ADMIN_USERNAME_PREFIX}1");
        let mut user_defs = create_user_defs(&CreateUsersDef::new(1, 0, MazeContent::Empty));
        let (app_config, temp_path) = make_admin_features_config_toml(true);
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig::default()));
        let (app, _, _, api_key, _) = create_test_app_with_config(&mut user_defs, Some(admin_username), false, features, app_config).await;

        let put_req = create_test_put_request("/api/v1/admin/features", api_key, None, &AppFeaturesResponse { allow_signup: false, ..Default::default() });
        let put_resp = test::call_service(&app, put_req).await;
        assert_eq!(put_resp.status(), StatusCode::OK);

        // Verify the temp config file was updated on disk
        let content = std::fs::read_to_string(&temp_path).expect("failed to read temp config file");
        let parsed: toml::Table = content.parse().expect("failed to parse updated config toml");
        let allow_signup = parsed["features"]["allow_signup"].as_bool().expect("allow_signup missing");
        assert!(!allow_signup);

        let _ = std::fs::remove_file(&temp_path);
    }

    #[actix_web::test]
    async fn signup_blocked_when_allow_signup_disabled() {
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig { allow_signup: false }));
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app_with_features(&mut user_defs, None, false, features).await;
        let req = create_test_post_request("/api/v1/signup", None, None, Some(&SignupRequest {
            email: VALID_USER_EMAIL_1.to_string(),
            password: VALID_USER_PASSWORD.to_string(),
        }));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[actix_web::test]
    async fn signup_allowed_when_allow_signup_enabled() {
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig { allow_signup: true }));
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app_with_features(&mut user_defs, None, false, features).await;
        let req = create_test_post_request("/api/v1/signup", None, None, Some(&SignupRequest {
            email: VALID_USER_EMAIL_1.to_string(),
            password: VALID_USER_PASSWORD.to_string(),
        }));
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    // ============================================================================
    // OAuth handler tests
    // ============================================================================

    use crate::oauth::{
        BeginFlow, FlowOrigin, NormalisedIdentity, OAuthConnector, OAuthError,
        OAuthProviderPublic, PersistedState,
    };
    use async_trait::async_trait;

    /// Test connector that returns canned values. Configured via its fields
    /// so each test can drive specific branches of the handler.
    struct FakeConnector {
        providers: Vec<OAuthProviderPublic>,
        authorize_url: String,
        state_nonce: String,
        identity: Option<NormalisedIdentity>,
        complete_error: Option<OAuthError>,
    }

    impl FakeConnector {
        fn google_only() -> Self {
            Self {
                providers: vec![OAuthProviderPublic {
                    name: "google".into(),
                    display_name: "Google".into(),
                }],
                authorize_url: "https://provider.example.com/authorize?fake=1".into(),
                state_nonce: "fake-state-nonce".into(),
                identity: None,
                complete_error: None,
            }
        }
    }

    #[async_trait]
    impl OAuthConnector for FakeConnector {
        fn enabled_providers(&self) -> Vec<OAuthProviderPublic> { self.providers.clone() }

        async fn begin(&self, provider: &str, origin: FlowOrigin) -> Result<BeginFlow, OAuthError> {
            if !self.providers.iter().any(|p| p.name == provider) {
                return Err(OAuthError::UnknownOrDisabledProvider(provider.into()));
            }
            Ok(BeginFlow {
                authorize_url: self.authorize_url.clone(),
                persisted: PersistedState {
                    state: self.state_nonce.clone(),
                    pkce_verifier: "fake-pkce-verifier".into(),
                    origin,
                    provider: provider.to_string(),
                    created_at_unix: chrono::Utc::now().timestamp(),
                    client_state: None,
                },
            })
        }

        async fn complete(
            &self,
            _provider: &str,
            _code: &str,
            _cookie_state: &PersistedState,
        ) -> Result<NormalisedIdentity, OAuthError> {
            if let Some(err_msg) = self.complete_error.as_ref().map(|e| e.to_string()) {
                return Err(OAuthError::ProviderResponse(err_msg));
            }
            self.identity
                .clone()
                .ok_or_else(|| OAuthError::ProviderResponse("test connector has no identity".into()))
        }
    }

    async fn create_test_app_with_oauth_connector(
        connector: Arc<dyn OAuthConnector>,
    ) -> impl Service<actix_http::Request, Response = ServiceResponse, Error = Error> {
        let mut user_defs: Vec<UserDefinition> = vec![];
        let app_config = AppConfig::default();
        let features: SharedFeatures = Arc::new(RwLock::new(app_config.features.clone()));
        set_valid_password_hashes(&app_config.security.password_hash, &mut user_defs);
        let (shared_mock_store, _, _, _) = create_shared_mock_store(&user_defs, None, false);
        test::init_service(
            create_app(
                &app_config.security.password_hash,
                web::Data::new(shared_mock_store),
                web::Data::new(features),
                web::Data::new(connector as crate::oauth::SharedOAuthConnector),
                ".".to_string(),
            )
            .app_data(web::Data::new(app_config)),
        )
        .await
    }

    #[actix_web::test]
    async fn oauth_start_web_origin_redirects_with_state_cookie() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector.clone()).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/start?origin=web")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").expect("Location header").to_str().unwrap();
        assert_eq!(location, "https://provider.example.com/authorize?fake=1");
        let cookie = resp
            .headers()
            .get_all("set-cookie")
            .find_map(|h| h.to_str().ok().filter(|s| s.starts_with("maze_oauth_state=")))
            .expect("state cookie present");
        assert!(cookie.contains("HttpOnly"), "cookie must be HttpOnly: {cookie}");
        assert!(cookie.contains("Secure"), "cookie must be Secure: {cookie}");
        assert!(cookie.contains("SameSite=Lax"), "cookie must be SameSite=Lax: {cookie}");
    }

    #[actix_web::test]
    async fn oauth_start_mobile_origin_redirects_with_state_cookie() {
        // Same response shape as web origin (302 + Set-Cookie). Returning a
        // redirect — rather than JSON for the mobile client to dispatch — is
        // what lets the platform browser carry the state cookie through the
        // round trip; a JSON-then-fetch design would land the cookie in the
        // mobile client's HTTP cookie jar instead of the system browser's.
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/start?origin=mobile")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").expect("Location header").to_str().unwrap();
        assert_eq!(location, "https://provider.example.com/authorize?fake=1");
        let cookie = resp
            .headers()
            .get_all("set-cookie")
            .find_map(|h| h.to_str().ok().filter(|s| s.starts_with("maze_oauth_state=")))
            .expect("state cookie present");
        assert!(cookie.contains("HttpOnly"), "cookie must be HttpOnly: {cookie}");
        assert!(cookie.contains("Secure"), "cookie must be Secure: {cookie}");
    }

    #[actix_web::test]
    async fn oauth_start_unknown_provider_returns_404() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/microsoft/start?origin=web")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_web::test]
    async fn oauth_callback_with_no_state_cookie_redirects_to_login_with_error() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;
        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=xyz")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.starts_with("/login?error=invalid_state"), "got: {location}");
    }

    #[actix_web::test]
    async fn oauth_callback_with_state_mismatch_redirects_to_login_with_error() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;

        // Build a valid cookie value with state "real-state".
        let persisted = PersistedState {
            state: "real-state".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=different-state")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.starts_with("/login?error=state_mismatch"), "got: {location}");
    }

    #[actix_web::test]
    async fn oauth_callback_with_provider_path_mismatch_returns_error() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;

        // Cookie says google, URL path says github.
        let persisted = PersistedState {
            state: "s".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/github/callback?code=abc&state=s")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.starts_with("/login?error=provider_mismatch"), "got: {location}");
    }

    #[actix_web::test]
    async fn oauth_callback_happy_path_redirects_with_token() {
        let mut connector = FakeConnector::google_only();
        connector.identity = Some(NormalisedIdentity {
            provider: "google".into(),
            provider_user_id: "google-sub-1".into(),
            email: Some("oauth_user@example.com".into()),
            email_verified: true,
            display_name: Some("Oauth User".into()),
        });
        let connector: Arc<dyn OAuthConnector> = Arc::new(connector);
        let app = create_test_app_with_oauth_connector(connector).await;

        let persisted = PersistedState {
            state: "real-state".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=real-state")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(location.starts_with("/oauth/callback#token="), "got: {location}");
        assert!(location.contains("&expires_at="), "got: {location}");
        // Cleared state cookie must accompany success too.
        let cookie = resp
            .headers()
            .get_all("set-cookie")
            .find_map(|h| h.to_str().ok().filter(|s| s.starts_with("maze_oauth_state=")))
            .expect("state cookie present (cleared)");
        assert!(cookie.contains("Max-Age=0"), "cleared cookie should set Max-Age=0: {cookie}");
    }

    #[actix_web::test]
    async fn oauth_callback_mobile_origin_uses_same_host_for_errors_with_reason_and_state() {
        // The MAUI WebAuthenticator (and WinUIEx) filter incoming custom-scheme
        // activations by host of the registered CallbackUrl. Errors must
        // therefore use the SAME host as success, distinguished by a `reason`
        // query parameter, with `client_state` echoed so WinUIEx can correlate.
        let connector: Arc<dyn OAuthConnector> = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;

        // Build a cookie whose state DOES NOT match the callback's `state` query.
        let persisted = PersistedState {
            state: "real-state".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Mobile,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: Some(r#"{"appInstanceId":"","signinId":"abc-123"}"#.to_string()),
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=DIFFERENT")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Mobile origin returns a 200 HTML bridge page (not a 302) so the
        // system browser tab doesn't spin forever after the OS hands the
        // `maze-app://` activation to the MAUI app; see `mobile_callback_html`.
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let body = std::str::from_utf8(&body).unwrap();
        // SAME host as success (oauth-callback), not a different oauth-error host.
        // Params live in the fragment to sidestep Facebook's `#_=_`; see
        // `mobile_callback_url` for the full rationale.
        assert!(
            body.contains("maze-app://oauth-callback#"),
            "error must use same host as success in HTML body: {body}"
        );
        // Reason instead of token.
        assert!(body.contains("reason=state_mismatch"), "got: {body}");
        // client_state echoed back so WinUIEx can correlate. The HTML body
        // embeds the URL inside attribute values (meta-refresh `content` and
        // `<a href>`), where `&` is HTML-escaped to `&amp;` for valid HTML.
        assert!(
            body.contains("&amp;state=%7B%22appInstanceId%22%3A%22%22%2C%22signinId%22%3A%22abc-123%22%7D"),
            "client_state must be echoed url-encoded in HTML body: {body}"
        );
    }

    #[actix_web::test]
    async fn oauth_callback_new_user_flag_reflects_account_resolve_outcome() {
        // The post-Step-7 polish item #1 needs the client to know whether an
        // OAuth flow created a new user (so the Account UI can be opened with
        // a welcome banner) or signed in an existing one (no banner). The
        // server signals this via `&new_user=true` on the redirect URL when
        // and only when account::resolve returns `Created`. This test calls
        // the callback twice for the same identity:
        //   1. First call → empty store → branch 3 → Created → new_user=true.
        //   2. Second call → identity now exists → branch 1 → SignedIn → no flag.
        let mut connector = FakeConnector::google_only();
        connector.identity = Some(NormalisedIdentity {
            provider: "google".into(),
            provider_user_id: "google-sub-flag".into(),
            email: Some("flag_user@example.com".into()),
            email_verified: true,
            display_name: None,
        });
        let connector: Arc<dyn OAuthConnector> = Arc::new(connector);
        let app = create_test_app_with_oauth_connector(connector).await;

        let persisted = PersistedState {
            state: "s".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        // First callback: account::resolve creates a new user.
        let first = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=s")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let first_resp = test::call_service(&app, first).await;
        assert_eq!(first_resp.status(), StatusCode::FOUND);
        let first_location = first_resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(
            first_location.contains("&new_user=true"),
            "first sign-in (Created) must flag new_user: {first_location}"
        );

        // Second callback for the same identity: account::resolve finds the
        // existing user via (provider, provider_user_id) → SignedIn → no flag.
        let second = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=s")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let second_resp = test::call_service(&app, second).await;
        assert_eq!(second_resp.status(), StatusCode::FOUND);
        let second_location = second_resp.headers().get("Location").unwrap().to_str().unwrap();
        assert!(
            !second_location.contains("new_user"),
            "returning user (SignedIn) must not flag new_user: {second_location}"
        );
    }

    #[actix_web::test]
    async fn oauth_callback_mobile_origin_echoes_client_state_on_redirect() {
        // WinUIEx WebAuthenticator (and similar URL-scheme brokers) need their
        // client-supplied `state` echoed back on the maze-app:// callback so
        // they can correlate the activation with the in-flight task.
        let mut connector = FakeConnector::google_only();
        connector.identity = Some(NormalisedIdentity {
            provider: "google".into(),
            provider_user_id: "google-sub-mobile".into(),
            email: Some("mobile_user@example.com".into()),
            email_verified: true,
            display_name: None,
        });
        let connector: Arc<dyn OAuthConnector> = Arc::new(connector);
        let app = create_test_app_with_oauth_connector(connector).await;

        let persisted = PersistedState {
            state: "real-state".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Mobile,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: Some(r#"{"appInstanceId":"","signinId":"abc-123"}"#.to_string()),
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let req = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=real-state")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, req).await;
        // Mobile success returns a 200 HTML bridge page (not a 302); see
        // `mobile_callback_html` for the rationale.
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let body = std::str::from_utf8(&body).unwrap();
        assert!(body.contains("maze-app://oauth-callback#token="), "got: {body}");
        // The URL lives inside HTML attribute values, so inter-param `&`
        // is escaped to `&amp;` for valid HTML.
        assert!(body.contains("&amp;expires_at="), "got: {body}");
        // Critical: client_state must be present and percent-encoded.
        assert!(
            body.contains("&amp;state=%7B%22appInstanceId%22%3A%22%22%2C%22signinId%22%3A%22abc-123%22%7D"),
            "client_state must be echoed back url-encoded: {body}"
        );
    }

    #[actix_web::test]
    async fn get_features_includes_oauth_providers_from_connector() {
        let connector = Arc::new(FakeConnector::google_only());
        let app = create_test_app_with_oauth_connector(connector).await;
        let req = test::TestRequest::get().uri("/api/v1/features").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let features: AppFeaturesResponse = test::read_body_json(resp).await;
        assert_eq!(features.oauth_providers.len(), 1);
        assert_eq!(features.oauth_providers[0].name, "google");
        assert_eq!(features.oauth_providers[0].display_name, "Google");
    }

    #[actix_web::test]
    async fn get_features_returns_empty_oauth_providers_with_noop_connector() {
        // Default test app uses NoOpConnector via create_test_app_with_features.
        let features: SharedFeatures = Arc::new(RwLock::new(AppFeaturesConfig { allow_signup: true }));
        let mut user_defs = vec![];
        let (app, _, _, _, _) = create_test_app_with_features(&mut user_defs, None, false, features).await;
        let req = test::TestRequest::get().uri("/api/v1/features").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body: AppFeaturesResponse = test::read_body_json(resp).await;
        assert!(body.oauth_providers.is_empty());
    }

    #[actix_web::test]
    async fn oauth_callback_extends_session_via_renew() {
        // Locks in shared session-lifetime path: the bearer token issued by
        // OAuth callback participates in the existing /login/renew flow
        // exactly like a password-issued token.
        let mut connector = FakeConnector::google_only();
        connector.identity = Some(NormalisedIdentity {
            provider: "google".into(),
            provider_user_id: "google-sub-renew".into(),
            email: Some("renew_user@example.com".into()),
            email_verified: true,
            display_name: None,
        });
        let connector: Arc<dyn OAuthConnector> = Arc::new(connector);
        let app = create_test_app_with_oauth_connector(connector).await;

        let persisted = PersistedState {
            state: "s".into(),
            pkce_verifier: "v".into(),
            origin: FlowOrigin::Web,
            provider: "google".into(),
            created_at_unix: chrono::Utc::now().timestamp(),
            client_state: None,
        };
        let cookie_val = crate::oauth::state::encode(&persisted).unwrap();

        let cb = test::TestRequest::get()
            .uri("/api/v1/auth/oauth/google/callback?code=abc&state=s")
            .insert_header(("cookie", format!("maze_oauth_state={cookie_val}")))
            .to_request();
        let resp = test::call_service(&app, cb).await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let location = resp.headers().get("Location").unwrap().to_str().unwrap();
        // location like "/oauth/callback#token=<uuid>&expires_at=..."
        let token_id = location
            .strip_prefix("/oauth/callback#token=")
            .and_then(|s| s.split('&').next())
            .expect("token id present in location");

        let renew_req = test::TestRequest::post()
            .uri("/api/v1/login/renew")
            .insert_header(("Authorization", format!("Bearer {token_id}")))
            .to_request();
        let renew_resp = test::call_service(&app, renew_req).await;
        assert_eq!(renew_resp.status(), StatusCode::OK, "OAuth-issued token must work with /login/renew");
    }
}
