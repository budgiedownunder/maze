/// Identifies the algorithm used to generate a maze.
///
/// Marked `#[non_exhaustive]` so that adding future variants (e.g. Prim's, Kruskal's)
/// does not break existing `match` arms in downstream crates.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum GenerationAlgorithm {
    /// Generates a perfect maze (no loops, exactly one path between any two cells)
    /// using a single-pass iterative depth-first search from the start cell.
    ///
    /// See [Randomized depth-first search](https://en.wikipedia.org/wiki/Maze_generation_algorithm#Randomized_depth-first_search).
    RecursiveBacktracking,
}
