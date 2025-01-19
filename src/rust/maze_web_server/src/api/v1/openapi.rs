use maze::Maze;
use storage::MazeItem;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info (
        title="Maze REST Web API",
        version = "1.0.0",
        description = "RESTful Web API for managing and solving mazes",
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    paths(
        crate::api::v1::handlers::get_maze_list,
        crate::api::v1::handlers::create_maze,
        crate::api::v1::handlers::get_maze,
        crate::api::v1::handlers::update_maze,
        crate::api::v1::handlers::delete_maze,
        crate::api::v1::handlers::get_maze_solution,
        crate::api::v1::handlers::solve_maze
    ),
    components(schemas(MazeItem, Maze)),
    servers(
        (url = "https://localhost:8443", description = "Local development server")
    ),
    tags(
        (name = "Maze Web API v1", description = "Version 1 of the Maze Web API")
    )
)]
pub struct ApiDocV1;