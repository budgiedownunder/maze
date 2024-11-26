use storage::MazeItem;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::v1::handlers::get_maze_list
    ),
    components(schemas(MazeItem)), // Add schemas if necessary
    tags(
        (name = "v1", description = "Version 1 of the API")
    )
)]
pub struct ApiDocV1;