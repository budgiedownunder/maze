/// Identifies the algorithm used to generate a maze.
///
/// Marked `#[non_exhaustive]` so that adding future variants (e.g. Prim's, Kruskal's)
/// does not break existing `match` arms in downstream crates.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum GenerationAlgorithm {
    /// Two-phase recursive backtracking:
    /// Phase 1 performs a random walk from start to finish (the spine);
    /// Phase 2 fills remaining space with branches off the spine,
    /// producing a perfect maze (no loops, exactly one path between any two cells).
    RecursiveBacktracking,
}
