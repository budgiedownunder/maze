/// Identifies the algorithm used to generate a maze.
///
/// Marked `#[non_exhaustive]` so that adding future variants (e.g. Prim's, Kruskal's)
/// does not break existing `match` arms in downstream crates.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum GenerationAlgorithm {
    /// Generates a perfect maze (no loops, exactly one path between any two cells)
    /// with the [growing-tree algorithm](https://weblog.jamisbuck.org/2011/1/27/maze-generation-growing-tree-algorithm):
    /// an active list of carved cells is grown by repeatedly extending one of
    /// them into a neighbour. The cell is *usually* the most-recently-carved
    /// (depth-first "river", à la recursive backtracking) but *sometimes* a
    /// random active cell (Prim's-style branching) — a blend tuned to spread
    /// dead-end branches evenly along the maze rather than piling long ones near
    /// the finish.
    RecursiveBacktracking,
}
