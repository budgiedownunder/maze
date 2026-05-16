pub(crate) mod ew_panel;
pub(crate) mod ns_panel;

use crate::state::GameConfig;
use crate::world::textures::brick::make_brick_texture;
use crate::world::{CELL_SIZE, HALF_CELL};
use bevy::prelude::*;
use ew_panel::EwPanelAssets;
use ns_panel::NsPanelAssets;

pub(crate) const WALL_HEIGHT: f32 = 3.0;
pub(crate) const WALL_THICKNESS: f32 = 0.05;
// Inset each panel by this amount on each exposed edge to create visible border lines.
const BORDER_GAP: f32 = 0.01;
pub(crate) const PANEL_W: f32 = CELL_SIZE - 2.0 * BORDER_GAP;
pub(crate) const PANEL_H: f32 = WALL_HEIGHT - BORDER_GAP;
pub(crate) const PANEL_Y: f32 = (WALL_HEIGHT + BORDER_GAP) / 2.0;

// Per-cell wall-tint variants for spatial-orientation landmarks. Every
// passable cell hashes (row, col, GameConfig.seed) to pick one of these
// emissive offsets, so the same maze always tints the same cells but
// different cells (and so different corridor sections) read as subtly
// different shades. Offsets are added to the base emissive RGB and
// clamped at 0 — staying within roughly ±10% of the base so the maze
// still reads as a coherent space rather than a circus.
pub(crate) const WALL_TINT_VARIANTS: usize = 6;
pub(crate) const WALL_TINT_OFFSETS: [(f32, f32, f32); WALL_TINT_VARIANTS] = [
    (0.00, 0.00, 0.00),   // base
    (0.05, -0.02, -0.02), // warm
    (-0.04, 0.05, -0.02), // green
    (-0.02, -0.02, 0.05), // cool blue
    (-0.04, -0.04, -0.04), // dimmer
    (0.04, 0.04, 0.04),   // brighter
];

#[derive(Component)]
pub(crate) struct WallCell;

pub(crate) struct WallAssets {
    pub(crate) ns: NsPanelAssets,
    pub(crate) ew: EwPanelAssets,
}

pub(crate) fn build_wall_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> WallAssets {
    // Brick texture is shared between N/S and E/W panels — build once.
    let brick_tex = images.as_mut().map(|imgs| make_brick_texture(imgs));
    WallAssets {
        ns: ns_panel::build_ns_panel_assets(meshes, materials, &brick_tex),
        ew: ew_panel::build_ew_panel_assets(meshes, materials, &brick_tex),
    }
}

/// Deterministic hash of `(row, col, seed)` → wall tint variant index in
/// `0..WALL_TINT_VARIANTS`. Used by `spawn_world` so each cell picks a
/// stable tint for its walls; the same seed always tints the same cells.
pub(crate) fn wall_tint_index(r: usize, c: usize, seed: u64) -> usize {
    let mut h = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h = h.wrapping_add((r as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    h = h.wrapping_add((c as u64).wrapping_mul(0x94D0_49BB_1331_11EB));
    h ^= h >> 30;
    h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    h ^= h >> 27;
    (h % WALL_TINT_VARIANTS as u64) as usize
}

pub(crate) fn spawn_walls_for_cell(
    commands: &mut Commands,
    assets: &WallAssets,
    grid: &[Vec<char>],
    r: usize,
    c: usize,
    config: &GameConfig,
) {
    let rows = grid.len();
    let cols = grid[r].len();
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;

    // Per-cell wall-tint: hash (r, c, seed) → one of the
    // WALL_TINT_VARIANTS material variants so every cell's walls
    // pick up a subtly different shade, and the same maze always
    // looks the same. When the per-difficulty `landmarks.wall_tint`
    // toggle is off, every cell falls back to variant 0 (the base).
    let tint = if config.landmarks.wall_tint {
        wall_tint_index(r, c, config.seed)
    } else {
        0
    };

    // North face
    if r == 0 || grid[r - 1][c] == 'W' {
        ns_panel::spawn_ns_face(
            commands,
            &assets.ns,
            tint,
            Vec3::new(x, PANEL_Y, z - HALF_CELL),
        );
    }
    // South face
    if r + 1 >= rows || grid[r + 1][c] == 'W' {
        ns_panel::spawn_ns_face(
            commands,
            &assets.ns,
            tint,
            Vec3::new(x, PANEL_Y, z + HALF_CELL),
        );
    }
    // East face
    if c + 1 >= cols || grid[r][c + 1] == 'W' {
        ew_panel::spawn_ew_face(
            commands,
            &assets.ew,
            tint,
            Vec3::new(x + HALF_CELL, PANEL_Y, z),
        );
    }
    // West face
    if c == 0 || grid[r][c - 1] == 'W' {
        ew_panel::spawn_ew_face(
            commands,
            &assets.ew,
            tint,
            Vec3::new(x - HALF_CELL, PANEL_Y, z),
        );
    }
}
