use super::common::{self, CommonObjectAssets};
use crate::state::GameConfig;
use crate::world::CELL_SIZE;
use bevy::prelude::*;

// Dead-end landmark object variants. Each cell flagged as a dead-end
// (passable cell with exactly one open neighbour, excluding start/finish)
// hashes (row, col, seed) to pick one of these object kinds. The rigs
// themselves live in `objects::common` and are shared with the key holder.
pub(crate) const DEAD_END_OBJECT_VARIANTS: u32 = 4;

/// Per-dead-end-cell anchor. One is spawned at each cell that receives a
/// landmark, tagging the cell as carrying a dead-end object. The shared prop
/// sub-meshes are untagged common geometry (also used by the key holder), so
/// counting `DeadEndObject` counts landmark *cells*, not sub-meshes — and the
/// key holder's reuse of the same prop rigs is correctly excluded.
#[derive(Component)]
pub(crate) struct DeadEndObject;

/// Deterministic hash of `(row, col, seed)` → dead-end object kind in
/// `0..DEAD_END_OBJECT_VARIANTS`. Different constants from
/// `wall_tint_index` so the object kind and the cell tint don't
/// correlate visually.
pub(crate) fn dead_end_object_index(r: usize, c: usize, seed: u64) -> u32 {
    let mut h = seed.wrapping_mul(0x6EED_0E9D_A4D9_4A4F);
    h = h.wrapping_add((r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    h = h.wrapping_add((c as u64).wrapping_mul(0xC6BC_279E_C8C9_D5B1));
    h ^= h >> 30;
    h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 27;
    (h % DEAD_END_OBJECT_VARIANTS as u64) as u32
}

/// `true` when `(r, c)` is a dead-end cell — a passable cell whose four
/// orthogonal neighbours include exactly one other passable cell. Start
/// and finish cells are excluded by the caller, not here, so this helper
/// stays purely topological.
pub(crate) fn is_dead_end(grid: &[Vec<char>], r: usize, c: usize) -> bool {
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

pub(crate) fn spawn_dead_end_object_for_cell(
    commands: &mut Commands,
    assets: &CommonObjectAssets,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    config: &GameConfig,
) {
    // A single distinctive object per dead-end cell — brazier / urn /
    // broken pillar / chest, picked by hashing (row, col, seed). Skipped
    // for start / finish cells (the player stands on start, the finish
    // has the orb), for key / door cells (they own the dead-end with their
    // holder / panel — a key is commonly placed in a dead-end), for enemy
    // and health-pickup cells (the goblin / heart entity owns the cell's
    // visual), and when the per-difficulty toggle is off.
    if !config.landmarks.dead_end_objects
        || matches!(cell, 'S' | 'F' | 'K' | 'D' | 'E' | 'H')
        || !is_dead_end(grid, r, c)
    {
        return;
    }
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    let kind = dead_end_object_index(r, c, config.seed);
    match kind {
        0 => common::brazier::spawn_brazier(commands, assets, x, z),
        1 => common::urn::spawn_urn(commands, assets, x, z),
        2 => common::pillar::spawn_pillar(commands, assets, x, z, 1.0),
        _ => common::chest::spawn_chest(
            commands,
            assets,
            x,
            z,
            common::yaw_toward_open_neighbour(grid, r, c),
        ),
    }
    // Anchor entity tagging this cell as carrying a dead-end landmark.
    commands.spawn((DeadEndObject, Transform::from_xyz(x, 0.0, z)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_end_object_index_is_deterministic() {
        let seed = 0xCAFEu64;
        assert_eq!(
            dead_end_object_index(3, 5, seed),
            dead_end_object_index(3, 5, seed)
        );
    }

    #[test]
    fn dead_end_object_index_always_in_range() {
        for r in 0..30 {
            for c in 0..30 {
                let kind = dead_end_object_index(r, c, 0x9999u64);
                assert!(kind < DEAD_END_OBJECT_VARIANTS, "got kind {kind}");
            }
        }
    }

    #[test]
    fn is_dead_end_single_open_neighbour() {
        // (1,1) has only south open
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', ' ', 'W'],
            vec!['W', ' ', 'W'],
        ];
        assert!(is_dead_end(&grid, 1, 1));
    }

    #[test]
    fn is_dead_end_corridor_false() {
        // (1,1) has east AND west open — two-way corridor, not a dead end
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec![' ', ' ', ' '],
            vec!['W', 'W', 'W'],
        ];
        assert!(!is_dead_end(&grid, 1, 1));
    }

    #[test]
    fn is_dead_end_junction_false() {
        // (1,1) has three open neighbours — T-junction, not a dead end
        let grid = vec![
            vec!['W', ' ', 'W'],
            vec![' ', ' ', ' '],
            vec!['W', 'W', 'W'],
        ];
        assert!(!is_dead_end(&grid, 1, 1));
    }

    #[test]
    fn is_dead_end_isolated_false() {
        // No open neighbours
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', ' ', 'W'],
            vec!['W', 'W', 'W'],
        ];
        assert!(!is_dead_end(&grid, 1, 1));
    }

    #[test]
    fn is_dead_end_corner_with_one_neighbour() {
        // Top-left cell; grid boundary counts as wall; only south open
        let grid = vec![vec![' ', 'W'], vec![' ', 'W']];
        assert!(is_dead_end(&grid, 0, 0));
    }

    #[test]
    fn is_dead_end_on_wall_false() {
        let grid = vec![vec!['W', 'W'], vec!['W', ' ']];
        assert!(!is_dead_end(&grid, 0, 0));
    }

    #[test]
    fn is_dead_end_out_of_bounds_false() {
        let grid = vec![vec![' ']];
        assert!(!is_dead_end(&grid, 5, 5));
    }
}
