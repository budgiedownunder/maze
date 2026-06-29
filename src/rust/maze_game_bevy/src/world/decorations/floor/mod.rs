pub(crate) mod cracked_tile;
pub(crate) mod mosaic;
pub(crate) mod moss;
pub(crate) mod sigil;

use crate::state::GameConfig;
use crate::world::{LevelPlacement, CELL_SIZE};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct FloorAccent;

pub(crate) const FLOOR_ACCENT_VARIANTS: u32 = 4;
/// X-Z extent of the accent patch (units). Smaller than CELL_SIZE (2.0)
/// so the existing floor tile + grid lines remain visible around the
/// patch.
const FLOOR_ACCENT_SIZE: f32 = 1.2;
/// Mesh thickness (units, vertical extent). Thin enough to look flat.
const FLOOR_ACCENT_THICKNESS: f32 = 0.01;
/// Y position: above the floor tile (top at y=0.005) AND the grid lines
/// (top at y=0.020) so the accent sits cleanly on the surface with no
/// z-fighting.
const FLOOR_ACCENT_Y: f32 = 0.025;

pub(crate) struct FloorAccentAssets {
    pub(crate) mesh: Option<Handle<Mesh>>,
    pub(crate) mats: [Option<Handle<StandardMaterial>>; FLOOR_ACCENT_VARIANTS as usize],
}

pub(crate) fn build_floor_accent_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> FloorAccentAssets {
    // A single thin horizontal cuboid mesh — extent in X+Z, thin in Y —
    // shared across all accent kinds. Pixel intensity drives emissive;
    // each kind's material tints the same monochrome texture differently
    // so the four accent types read distinctly.
    let mesh = meshes.as_mut().map(|m| {
        m.add(Cuboid::new(
            FLOOR_ACCENT_SIZE,
            FLOOR_ACCENT_THICKNESS,
            FLOOR_ACCENT_SIZE,
        ))
    });
    let mats: [Option<Handle<StandardMaterial>>; FLOOR_ACCENT_VARIANTS as usize] = [
        moss::build_moss_material(materials, images),
        cracked_tile::build_cracked_tile_material(materials, images),
        mosaic::build_mosaic_material(materials, images),
        sigil::build_sigil_material(materials, images),
    ];
    FloorAccentAssets { mesh, mats }
}

/// Deterministic hash of `(row, col, seed)` → floor-accent kind in
/// `0..FLOOR_ACCENT_VARIANTS`. Different multiplicative constants from
/// `wall_tint_index`, `dead_end_object_index`, and `wall_decoration_index`
/// so the accent kind doesn't correlate visually with the other landmark
/// hashes.
pub(crate) fn floor_accent_index(r: usize, c: usize, seed: u64) -> u32 {
    let mut h = seed.wrapping_mul(0x9999_BBBB_5555_7777);
    h = h.wrapping_add((r as u64).wrapping_mul(0x517C_C1B7_2722_0A95));
    h = h.wrapping_add((c as u64).wrapping_mul(0xC6BC_279E_C8C9_D5B1));
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    (h % FLOOR_ACCENT_VARIANTS as u64) as u32
}

/// `true` when `(r, c)` is a junction cell — a passable cell whose four
/// orthogonal neighbours include more than two other passable cells (a
/// T-junction or 4-way intersection). Start and finish cells are
/// excluded by the caller, not here, so this helper stays purely
/// topological.
pub(crate) fn is_junction(grid: &[Vec<char>], r: usize, c: usize) -> bool {
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
    open > 2
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_floor_accents_for_cell(
    commands: &mut Commands,
    assets: &FloorAccentAssets,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    config: &GameConfig,
    placement: LevelPlacement,
) {
    // A single flat accent per junction cell — moss / cracked tile /
    // mosaic / sigil, picked by hashing (row, col, seed). Skipped for
    // start / finish (they have their own visual identity) and when the
    // per-difficulty toggle is off.
    if !config.landmarks.floor_accents
        || cell == 'S'
        || cell == 'F'
        || !is_junction(grid, r, c)
    {
        return;
    }
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let y = placement.world_y(FLOOR_ACCENT_Y);
    let kind = floor_accent_index(r, c, config.seed);
    let mat = assets.mats[kind as usize].clone();
    match (assets.mesh.clone(), mat) {
        (Some(mesh), Some(mt)) => {
            commands.spawn((
                FloorAccent,
                Mesh3d(mesh),
                MeshMaterial3d(mt),
                Transform::from_xyz(x, y, z),
            ));
        }
        _ => {
            commands.spawn((FloorAccent, Transform::from_xyz(x, y, z)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floor_accent_index_is_deterministic() {
        let seed = 0xCAFEu64;
        assert_eq!(
            floor_accent_index(3, 5, seed),
            floor_accent_index(3, 5, seed)
        );
    }

    #[test]
    fn floor_accent_index_always_in_range() {
        for r in 0..30 {
            for c in 0..30 {
                let kind = floor_accent_index(r, c, 0x9999u64);
                assert!(kind < FLOOR_ACCENT_VARIANTS, "got kind {kind}");
            }
        }
    }

    #[test]
    fn is_junction_three_open_neighbours_true() {
        // T-junction at (1,1): N, E, W open; S closed
        let grid = vec![
            vec!['W', ' ', 'W'],
            vec![' ', ' ', ' '],
            vec!['W', 'W', 'W'],
        ];
        assert!(is_junction(&grid, 1, 1));
    }

    #[test]
    fn is_junction_four_open_neighbours_true() {
        // 4-way junction at (1,1)
        let grid = vec![
            vec!['W', ' ', 'W'],
            vec![' ', ' ', ' '],
            vec!['W', ' ', 'W'],
        ];
        assert!(is_junction(&grid, 1, 1));
    }

    #[test]
    fn is_junction_corridor_false() {
        // East + west open — two-way corridor, not a junction
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec![' ', ' ', ' '],
            vec!['W', 'W', 'W'],
        ];
        assert!(!is_junction(&grid, 1, 1));
    }

    #[test]
    fn is_junction_dead_end_false() {
        // Only south open — dead end, not a junction
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', ' ', 'W'],
            vec!['W', ' ', 'W'],
        ];
        assert!(!is_junction(&grid, 1, 1));
    }

    #[test]
    fn is_junction_isolated_false() {
        // No open neighbours
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', ' ', 'W'],
            vec!['W', 'W', 'W'],
        ];
        assert!(!is_junction(&grid, 1, 1));
    }

    #[test]
    fn is_junction_on_wall_false() {
        let grid = vec![vec!['W', 'W'], vec!['W', ' ']];
        assert!(!is_junction(&grid, 0, 0));
    }

    #[test]
    fn is_junction_out_of_bounds_false() {
        let grid = vec![vec![' ']];
        assert!(!is_junction(&grid, 5, 5));
    }
}
