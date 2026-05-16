pub(crate) mod glowing_glass;
pub(crate) mod poster;
pub(crate) mod rune;
pub(crate) mod vent;

use crate::state::GameConfig;
use crate::world::walls::WALL_THICKNESS;
use crate::world::{CELL_SIZE, HALF_CELL};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct WallDecoration;

// Sparse wall decorations. Each wall panel hashes `(row, col, face, seed)`;
// 1 in `WALL_DECORATION_FREQUENCY` panels gets a decoration, and the kind picks
// from `WALL_DECORATION_VARIANTS`. Decoration placement / orientation are derived
// from the wall's face id (0=N, 1=S, 2=E, 3=W).
pub(crate) const WALL_DECORATION_VARIANTS: u32 = 4;
pub(crate) const WALL_DECORATION_FREQUENCY: u32 = 8;
pub(crate) const DECORATION_W: f32 = 0.6;
pub(crate) const DECORATION_H: f32 = 0.8;
pub(crate) const DECORATION_THICKNESS: f32 = 0.02;
pub(crate) const DECORATION_Y: f32 = 1.7;
// Push the decoration just outside the wall's inside face so it does not
// z-fight with the brick texture; wall half-thickness + a small epsilon.
pub(crate) const DECORATION_OFFSET: f32 = WALL_THICKNESS / 2.0 + 0.005;

pub(crate) struct DecorationAssets {
    pub(crate) ns_mesh: Option<Handle<Mesh>>,
    pub(crate) ew_mesh: Option<Handle<Mesh>>,
    pub(crate) mats: [Option<Handle<StandardMaterial>>; WALL_DECORATION_VARIANTS as usize],
}

pub(crate) fn build_decoration_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> DecorationAssets {
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
    DecorationAssets { ns_mesh, ew_mesh, mats }
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
    assets: &DecorationAssets,
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

pub(crate) fn spawn_decorations_for_cell(
    commands: &mut Commands,
    assets: &DecorationAssets,
    grid: &[Vec<char>],
    r: usize,
    c: usize,
    config: &GameConfig,
) {
    if !config.landmarks.wall_decorations {
        return;
    }
    let rows = grid.len();
    let cols = grid[r].len();
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    let seed = config.seed;

    // North face
    if r == 0 || grid[r - 1][c] == 'W' {
        spawn_decoration(
            commands,
            assets,
            r,
            c,
            0,
            seed,
            assets.ns_mesh.clone(),
            Vec3::new(x, DECORATION_Y, z - HALF_CELL + DECORATION_OFFSET),
        );
    }
    // South face
    if r + 1 >= rows || grid[r + 1][c] == 'W' {
        spawn_decoration(
            commands,
            assets,
            r,
            c,
            1,
            seed,
            assets.ns_mesh.clone(),
            Vec3::new(x, DECORATION_Y, z + HALF_CELL - DECORATION_OFFSET),
        );
    }
    // East face
    if c + 1 >= cols || grid[r][c + 1] == 'W' {
        spawn_decoration(
            commands,
            assets,
            r,
            c,
            2,
            seed,
            assets.ew_mesh.clone(),
            Vec3::new(x + HALF_CELL - DECORATION_OFFSET, DECORATION_Y, z),
        );
    }
    // West face
    if c == 0 || grid[r][c - 1] == 'W' {
        spawn_decoration(
            commands,
            assets,
            r,
            c,
            3,
            seed,
            assets.ew_mesh.clone(),
            Vec3::new(x - HALF_CELL + DECORATION_OFFSET, DECORATION_Y, z),
        );
    }
}
