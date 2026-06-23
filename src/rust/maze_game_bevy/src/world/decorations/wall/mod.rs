pub(crate) mod glowing_glass;
pub(crate) mod poster;
pub(crate) mod rune;
pub(crate) mod vent;

use crate::state::GameConfig;
use crate::world::walls::{is_non_occluding_wall, WALL_THICKNESS};
use crate::world::{LevelPlacement, CELL_SIZE, HALF_CELL};
use bevy::prelude::*;
use maze::CellEntity;
use std::collections::HashMap;

#[derive(Component)]
pub(crate) struct WallDecoration;

// Sparse wall decorations. Each wall panel hashes `(row, col, face, seed)`;
// 1 in `WALL_DECORATION_FREQUENCY` panels gets a decoration, and the kind picks
// from `WALL_DECORATION_VARIANTS`. Decoration placement / orientation are derived
// from the wall's face id (0=N, 1=S, 2=E, 3=W).
pub(crate) const WALL_DECORATION_VARIANTS: u32 = 4;
pub(crate) const WALL_DECORATION_FREQUENCY: u32 = 10;
pub(crate) const DECORATION_W: f32 = 0.6;
pub(crate) const DECORATION_H: f32 = 0.8;
pub(crate) const DECORATION_THICKNESS: f32 = 0.02;
pub(crate) const DECORATION_Y: f32 = 1.7;
// Push the decoration just outside the wall's inside face so it does not
// z-fight with the brick texture; wall half-thickness + a small epsilon.
pub(crate) const DECORATION_OFFSET: f32 = WALL_THICKNESS / 2.0 + 0.005;

pub(crate) struct WallDecorationAssets {
    pub(crate) ns_mesh: Option<Handle<Mesh>>,
    pub(crate) ew_mesh: Option<Handle<Mesh>>,
    pub(crate) mats: [Option<Handle<StandardMaterial>>; WALL_DECORATION_VARIANTS as usize],
}

pub(crate) fn build_wall_decoration_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> WallDecorationAssets {
    // Two thin cuboid meshes — one for N/S faces (extent in X+Y, thin
    // in Z), one for E/W (extent in Y+Z, thin in X) — shared across all
    // decoration kinds.
    let ns_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(DECORATION_W, DECORATION_H, DECORATION_THICKNESS)));
    let ew_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(DECORATION_THICKNESS, DECORATION_H, DECORATION_W)));
    // One emissive-tinted material per decoration kind. Per-decoration emissive
    // colour tints the same monochrome texture differently so each decoration
    // type reads distinctly.
    let mats: [Option<Handle<StandardMaterial>>; WALL_DECORATION_VARIANTS as usize] = [
        vent::build_vent_material(materials, images),
        poster::build_poster_material(materials, images),
        rune::build_rune_material(materials, images),
        glowing_glass::build_glowing_glass_material(materials, images),
    ];
    WallDecorationAssets { ns_mesh, ew_mesh, mats }
}

/// Deterministic hash of `(row, col, face, seed)` → `Some(kind)` if a
/// wall decoration should be placed on this face, `None` otherwise. The
/// `1 / WALL_DECORATION_FREQUENCY` placement rate is chosen so decorations feel
/// sparse rather than wallpaper. Different constants from the other
/// landmark hashes so decoration placement isn't visually correlated with
/// wall tint or dead-end object kind.
pub(crate) fn wall_decoration_index(r: usize, c: usize, face_id: u32, seed: u64) -> Option<u32> {
    let mut h = seed.wrapping_mul(0x517C_C1B7_2722_0A95);
    h = h.wrapping_add((r as u64).wrapping_mul(0xC6BC_279E_C8C9_D5B1));
    h = h.wrapping_add((c as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    h = h.wrapping_add((face_id as u64).wrapping_mul(0x6EED_0E9D_A4D9_4A4F));
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    if h.is_multiple_of(WALL_DECORATION_FREQUENCY as u64) {
        Some(((h / WALL_DECORATION_FREQUENCY as u64) % WALL_DECORATION_VARIANTS as u64) as u32)
    } else {
        None
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_decoration(
    commands: &mut Commands,
    assets: &WallDecorationAssets,
    r: usize,
    c: usize,
    face_id: u32,
    seed: u64,
    mesh: Option<Handle<Mesh>>,
    pos: Vec3,
) {
    let Some(kind) = wall_decoration_index(r, c, face_id, seed) else {
        return;
    };
    let mat = assets.mats[kind as usize].clone();
    match (mesh, mat) {
        (Some(m), Some(mt)) => {
            commands.spawn((
                WallDecoration,
                Mesh3d(m),
                MeshMaterial3d(mt),
                Transform::from_translation(pos),
            ));
        }
        _ => {
            commands.spawn((WallDecoration, Transform::from_translation(pos)));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_wall_decorations_for_cell(
    commands: &mut Commands,
    assets: &WallDecorationAssets,
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    r: usize,
    c: usize,
    config: &GameConfig,
    placement: LevelPlacement,
) {
    if !config.landmarks.wall_decorations {
        return;
    }
    let rows = grid.len();
    let cols = grid[r].len();
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let dec_y = placement.world_y(DECORATION_Y);
    let seed = config.seed;

    // A decoration sits on a panel, so it's drawn only where a solid panel is —
    // against a solid `'W'` wall, or at the grid edge when the perimeter is walled.
    // A non-occluding neighbour (water / lava / iron fence) has its panel
    // suppressed; so does an open-sky grid edge with perimeter walls off — either
    // way no panel, so no decoration (otherwise it would float in mid-air).
    let solid_wall = |nr: usize, nc: usize| {
        grid[nr][nc] == 'W' && !is_non_occluding_wall(grid, cell_entities, config, nr, nc)
    };
    // The grid edge carries a panel only when the maze is walled in there:
    // always under an enclosed sky, otherwise per `GameConfig::perimeter_walls`.
    let walled_edge = config.sky_type.is_enclosed() || config.perimeter_walls;

    // North face
    if (r == 0 && walled_edge) || (r > 0 && solid_wall(r - 1, c)) {
        spawn_decoration(
            commands,
            assets,
            r,
            c,
            0,
            seed,
            assets.ns_mesh.clone(),
            Vec3::new(x, dec_y, z - HALF_CELL + DECORATION_OFFSET),
        );
    }
    // South face
    if (r + 1 >= rows && walled_edge) || (r + 1 < rows && solid_wall(r + 1, c)) {
        spawn_decoration(
            commands,
            assets,
            r,
            c,
            1,
            seed,
            assets.ns_mesh.clone(),
            Vec3::new(x, dec_y, z + HALF_CELL - DECORATION_OFFSET),
        );
    }
    // East face
    if (c + 1 >= cols && walled_edge) || (c + 1 < cols && solid_wall(r, c + 1)) {
        spawn_decoration(
            commands,
            assets,
            r,
            c,
            2,
            seed,
            assets.ew_mesh.clone(),
            Vec3::new(x + HALF_CELL - DECORATION_OFFSET, dec_y, z),
        );
    }
    // West face
    if (c == 0 && walled_edge) || (c > 0 && solid_wall(r, c - 1)) {
        spawn_decoration(
            commands,
            assets,
            r,
            c,
            3,
            seed,
            assets.ew_mesh.clone(),
            Vec3::new(x - HALF_CELL + DECORATION_OFFSET, dec_y, z),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wall_decoration_index_is_deterministic() {
        let seed = 0xDEADu64;
        assert_eq!(
            wall_decoration_index(3, 5, 2, seed),
            wall_decoration_index(3, 5, 2, seed)
        );
    }

    #[test]
    fn wall_decoration_index_kind_always_in_range() {
        for r in 0..30 {
            for c in 0..30 {
                for face in 0..4 {
                    if let Some(kind) = wall_decoration_index(r, c, face, 0xCAFEu64) {
                        assert!(kind < WALL_DECORATION_VARIANTS, "got kind {kind}");
                    }
                }
            }
        }
    }

    #[test]
    fn wall_decoration_index_respects_face_id() {
        // The same cell on different faces should be able to disagree
        // (otherwise face_id is being ignored).
        let mut some_differ = false;
        for r in 0..20 {
            for c in 0..20 {
                if wall_decoration_index(r, c, 0, 0xC0FFEEu64)
                    != wall_decoration_index(r, c, 1, 0xC0FFEEu64)
                {
                    some_differ = true;
                    break;
                }
            }
            if some_differ {
                break;
            }
        }
        assert!(some_differ, "face_id had no effect");
    }

    #[test]
    fn wall_decoration_index_frequency_within_tolerance() {
        // 50×50×4 = 10000 hash rolls. Expected ~1/FREQUENCY of them are Some.
        // Allow ±50% tolerance — hash uniformity isn't guaranteed for small
        // samples but the order of magnitude should match.
        let mut decorated = 0usize;
        let mut total = 0usize;
        let seed = 0xABCD_1234u64;
        for r in 0..50 {
            for c in 0..50 {
                for face in 0..4 {
                    total += 1;
                    if wall_decoration_index(r, c, face, seed).is_some() {
                        decorated += 1;
                    }
                }
            }
        }
        let expected = total as f64 / WALL_DECORATION_FREQUENCY as f64;
        let ratio = decorated as f64 / expected;
        assert!(
            (0.5..=1.5).contains(&ratio),
            "decorated {decorated} / expected {expected:.0} (ratio {ratio:.2})"
        );
    }
}
