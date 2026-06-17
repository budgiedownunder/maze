//! Pure maze-grid topology helpers shared across crates — small functions over
//! the character grid that classify a cell by its local connectivity, with no
//! dependence on the generation or game features.

/// Returns `true` when `(r, c)` is a **dead-end** cell: a passable (non-`'W'`)
/// cell whose four orthogonal neighbours include exactly one other passable
/// cell — i.e. a corridor that terminates here. Out-of-bounds coordinates and
/// wall cells are never dead ends.
///
/// This is the single source of truth for "dead-end" across the workspace: the
/// generator places treasure dead-end-first and the 3D renderer decorates
/// dead-ends, so sharing one predicate keeps placement and rendering in
/// agreement by construction. The check is **purely topological** — start /
/// finish (and any other feature) cells are not special-cased, so a caller that
/// wants to exclude them filters on the cell character itself.
///
/// # Examples
///
/// ```
/// use maze::is_dead_end;
///
/// let grid = vec![
///     vec!['W', 'W', 'W', 'W', 'W'],
///     vec!['W', ' ', ' ', ' ', 'W'],
///     vec!['W', 'W', 'W', 'W', 'W'],
/// ];
/// assert!(is_dead_end(&grid, 1, 1)); // the corridor's left end — one open neighbour
/// assert!(!is_dead_end(&grid, 1, 2)); // a through-corridor — two open neighbours
/// assert!(!is_dead_end(&grid, 1, 0)); // a wall is never a dead end
/// ```
pub fn is_dead_end(grid: &[Vec<char>], r: usize, c: usize) -> bool {
    let rows = grid.len();
    let cols = if grid.is_empty() { 0 } else { grid[0].len() };
    if r >= rows || c >= cols || grid[r][c] == 'W' {
        return false;
    }
    let mut open = 0u32;
    if r > 0 && grid[r - 1][c] != 'W' {
        open += 1;
    }
    if r + 1 < rows && grid[r + 1][c] != 'W' {
        open += 1;
    }
    if c > 0 && grid[r][c - 1] != 'W' {
        open += 1;
    }
    if c + 1 < cols && grid[r][c + 1] != 'W' {
        open += 1;
    }
    open == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_open_neighbour_is_a_dead_end() {
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', ' ', 'W'],
            vec!['W', ' ', 'W'],
        ];
        assert!(is_dead_end(&grid, 1, 1)); // opens only south
    }

    #[test]
    fn through_corridor_ends_are_dead_ends_but_the_middle_is_not() {
        let grid = vec![
            vec!['W', 'W', 'W', 'W', 'W'],
            vec!['W', ' ', ' ', ' ', 'W'],
            vec!['W', 'W', 'W', 'W', 'W'],
        ];
        assert!(is_dead_end(&grid, 1, 1)); // left end
        assert!(is_dead_end(&grid, 1, 3)); // right end
        assert!(!is_dead_end(&grid, 1, 2)); // two open neighbours
    }

    #[test]
    fn junction_is_not_a_dead_end() {
        let grid = vec![
            vec!['W', ' ', 'W'],
            vec![' ', ' ', ' '],
            vec!['W', ' ', 'W'],
        ];
        assert!(!is_dead_end(&grid, 1, 1)); // four open neighbours
    }

    #[test]
    fn wall_out_of_bounds_and_empty_are_not_dead_ends() {
        let grid = vec![vec!['W', ' '], vec![' ', ' ']];
        assert!(!is_dead_end(&grid, 0, 0)); // a wall
        assert!(!is_dead_end(&grid, 5, 5)); // out of bounds
        assert!(!is_dead_end(&[], 0, 0)); // empty grid
    }

    #[test]
    fn non_wall_feature_cells_are_classified_purely_by_topology() {
        // A treasure / key / etc. char on a stub cell is still a dead end —
        // only the wall char and the open-neighbour count matter.
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', 'T', 'W'],
            vec!['W', ' ', 'W'],
        ];
        assert!(is_dead_end(&grid, 1, 1)); // 'T' opens only south
    }
}
