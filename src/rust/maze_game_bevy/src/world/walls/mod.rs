pub(crate) mod ew_panel;
pub(crate) mod ns_panel;

use crate::state::GameConfig;
use crate::world::textures::brick::make_brick_texture;
use crate::world::textures::cobblestone::make_cobblestone_texture;
use crate::world::textures::dressed_stone::make_dressed_stone_texture;
use crate::world::textures::wood::make_wood_texture;
use crate::world::{CELL_SIZE, HALF_CELL};
use bevy::prelude::*;
use ew_panel::EwPanelAssets;
use ns_panel::{NsPanelAssets, WallMaterialSpec};

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

// Per-quadrant wall material kinds for the heavier "wall_material_variation"
// landmark. Splits the maze into a 2×2 NW/NE/SW/SE grid; each quadrant gets
// one of these material kinds, seed-permuted so different seeds rotate which
// quadrant gets which kind. Supersedes the per-cell tint variation above when
// the `landmarks.wall_material_variation` toggle is on.
pub(crate) const WALL_MATERIAL_VARIANTS: usize = 4;
pub(crate) const WALL_MATERIAL_BRICK: usize = 0;
#[allow(dead_code)]
pub(crate) const WALL_MATERIAL_DRESSED_STONE: usize = 1;
#[allow(dead_code)]
pub(crate) const WALL_MATERIAL_WOOD: usize = 2;
#[allow(dead_code)]
pub(crate) const WALL_MATERIAL_COBBLESTONE: usize = 3;

// Per-material emissive RGB tints + UV scales. N/S-facing panels
// (ahead / behind the player) use the lighter half; E/W-facing panels
// (sides) use the darker half so the same material reads as the same
// kind from any angle but still has directional shading depth.
// One pair per index in `0..WALL_MATERIAL_VARIANTS`.

// Brick — pre-Step-5F look, slightly cool stone grey.
const NS_BRICK_EMISSIVE: (f32, f32, f32) = (0.38, 0.38, 0.40);
const EW_BRICK_EMISSIVE: (f32, f32, f32) = (0.14, 0.14, 0.16);
const BRICK_UV: Vec2 = Vec2::new(3.0, 5.0);

// Dressed stone — slightly warm pale stone.
const NS_DRESSED_STONE_EMISSIVE: (f32, f32, f32) = (0.50, 0.48, 0.42);
const EW_DRESSED_STONE_EMISSIVE: (f32, f32, f32) = (0.22, 0.21, 0.19);
const DRESSED_STONE_UV: Vec2 = Vec2::new(2.0, 3.0);

// Wood — warm brown, tall grain stretched vertically (UV.y = 4).
const NS_WOOD_EMISSIVE: (f32, f32, f32) = (0.35, 0.20, 0.10);
const EW_WOOD_EMISSIVE: (f32, f32, f32) = (0.18, 0.10, 0.05);
const WOOD_UV: Vec2 = Vec2::new(1.0, 4.0);

// Cobblestone — desaturated warm grey.
const NS_COBBLESTONE_EMISSIVE: (f32, f32, f32) = (0.32, 0.30, 0.28);
const EW_COBBLESTONE_EMISSIVE: (f32, f32, f32) = (0.12, 0.11, 0.10);
const COBBLESTONE_UV: Vec2 = Vec2::new(2.0, 2.0);

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
    // Build each material kind's texture once; both N/S and E/W panels reuse
    // the same handles.
    let brick_tex = images.as_mut().map(|imgs| make_brick_texture(imgs));
    let dressed_tex = images
        .as_mut()
        .map(|imgs| make_dressed_stone_texture(imgs));
    let wood_tex = images.as_mut().map(|imgs| make_wood_texture(imgs));
    let cobble_tex = images.as_mut().map(|imgs| make_cobblestone_texture(imgs));

    // N/S-facing panels (ahead / behind) — lighter base emissive per material
    // so they read brighter than the side panels at the same kind.
    let ns_specs: [WallMaterialSpec; WALL_MATERIAL_VARIANTS] = [
        WallMaterialSpec {
            texture: &brick_tex,
            emissive: NS_BRICK_EMISSIVE,
            uv_scale: BRICK_UV,
        },
        WallMaterialSpec {
            texture: &dressed_tex,
            emissive: NS_DRESSED_STONE_EMISSIVE,
            uv_scale: DRESSED_STONE_UV,
        },
        WallMaterialSpec {
            texture: &wood_tex,
            emissive: NS_WOOD_EMISSIVE,
            uv_scale: WOOD_UV,
        },
        WallMaterialSpec {
            texture: &cobble_tex,
            emissive: NS_COBBLESTONE_EMISSIVE,
            uv_scale: COBBLESTONE_UV,
        },
    ];
    // E/W-facing panels (sides) — darker variants of the same materials so
    // they read as the same kind seen edge-on.
    let ew_specs: [WallMaterialSpec; WALL_MATERIAL_VARIANTS] = [
        WallMaterialSpec {
            texture: &brick_tex,
            emissive: EW_BRICK_EMISSIVE,
            uv_scale: BRICK_UV,
        },
        WallMaterialSpec {
            texture: &dressed_tex,
            emissive: EW_DRESSED_STONE_EMISSIVE,
            uv_scale: DRESSED_STONE_UV,
        },
        WallMaterialSpec {
            texture: &wood_tex,
            emissive: EW_WOOD_EMISSIVE,
            uv_scale: WOOD_UV,
        },
        WallMaterialSpec {
            texture: &cobble_tex,
            emissive: EW_COBBLESTONE_EMISSIVE,
            uv_scale: COBBLESTONE_UV,
        },
    ];

    WallAssets {
        ns: ns_panel::build_ns_panel_assets(meshes, materials, &ns_specs),
        ew: ew_panel::build_ew_panel_assets(meshes, materials, &ew_specs),
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

/// Deterministic per-quadrant hash returning a wall-material kind in
/// `0..WALL_MATERIAL_VARIANTS`. The maze is split into a 2×2 NW/NE/SW/SE
/// grid; each quadrant gets one kind and the seed permutes the
/// quadrant-to-kind assignment so different seeds rotate the mapping while
/// keeping all four quadrants distinct (`(zone + shuffle) % 4` over zones
/// 0..4 is a permutation).
pub(crate) fn wall_material_index(
    r: usize,
    c: usize,
    rows: usize,
    cols: usize,
    seed: u64,
) -> usize {
    // Use 2*r < rows / 2*c < cols so the split lands at the midpoint without
    // floating-point or integer-division surprises.
    let zone_r: u64 = if r * 2 < rows { 0 } else { 1 };
    let zone_c: u64 = if c * 2 < cols { 0 } else { 1 };
    let zone = zone_r * 2 + zone_c;
    let shuffle = (seed.wrapping_mul(0xF0E1_D2C3_B4A5_9687) >> 32) % WALL_MATERIAL_VARIANTS as u64;
    ((zone + shuffle) % WALL_MATERIAL_VARIANTS as u64) as usize
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

    // Material variation supersedes per-cell tint: when on, every wall in
    // this cell takes the quadrant's material kind and the `wall_tint`
    // toggle is bypassed. When off, fall back to the original tinted-brick
    // path (tint index 0 when `wall_tint` is also off).
    if config.landmarks.wall_material_variation {
        let kind = wall_material_index(r, c, rows, cols, config.seed);
        // North face
        if r == 0 || grid[r - 1][c] == 'W' {
            ns_panel::spawn_ns_face_material(
                commands,
                &assets.ns,
                kind,
                Vec3::new(x, PANEL_Y, z - HALF_CELL),
            );
        }
        // South face
        if r + 1 >= rows || grid[r + 1][c] == 'W' {
            ns_panel::spawn_ns_face_material(
                commands,
                &assets.ns,
                kind,
                Vec3::new(x, PANEL_Y, z + HALF_CELL),
            );
        }
        // East face
        if c + 1 >= cols || grid[r][c + 1] == 'W' {
            ew_panel::spawn_ew_face_material(
                commands,
                &assets.ew,
                kind,
                Vec3::new(x + HALF_CELL, PANEL_Y, z),
            );
        }
        // West face
        if c == 0 || grid[r][c - 1] == 'W' {
            ew_panel::spawn_ew_face_material(
                commands,
                &assets.ew,
                kind,
                Vec3::new(x - HALF_CELL, PANEL_Y, z),
            );
        }
        return;
    }

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
        ns_panel::spawn_ns_face_tinted(
            commands,
            &assets.ns,
            tint,
            Vec3::new(x, PANEL_Y, z - HALF_CELL),
        );
    }
    // South face
    if r + 1 >= rows || grid[r + 1][c] == 'W' {
        ns_panel::spawn_ns_face_tinted(
            commands,
            &assets.ns,
            tint,
            Vec3::new(x, PANEL_Y, z + HALF_CELL),
        );
    }
    // East face
    if c + 1 >= cols || grid[r][c + 1] == 'W' {
        ew_panel::spawn_ew_face_tinted(
            commands,
            &assets.ew,
            tint,
            Vec3::new(x + HALF_CELL, PANEL_Y, z),
        );
    }
    // West face
    if c == 0 || grid[r][c - 1] == 'W' {
        ew_panel::spawn_ew_face_tinted(
            commands,
            &assets.ew,
            tint,
            Vec3::new(x - HALF_CELL, PANEL_Y, z),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn wall_tint_index_is_deterministic() {
        let seed = 0xDEAD_BEEFu64;
        assert_eq!(wall_tint_index(3, 5, seed), wall_tint_index(3, 5, seed));
    }

    #[test]
    fn wall_tint_index_always_in_range() {
        for r in 0..20 {
            for c in 0..20 {
                let idx = wall_tint_index(r, c, 0x1234_5678);
                assert!(idx < WALL_TINT_VARIANTS, "got {idx}");
            }
        }
    }

    #[test]
    fn wall_tint_index_changes_with_seed() {
        // Different seeds should produce a different tint for at least one cell
        // (otherwise the seed is being ignored).
        let mut diffs = 0;
        for r in 0..10 {
            for c in 0..10 {
                if wall_tint_index(r, c, 0) != wall_tint_index(r, c, 1) {
                    diffs += 1;
                }
            }
        }
        assert!(diffs > 0, "seed had no effect across 100 cells");
    }

    #[test]
    fn wall_material_index_is_deterministic() {
        let seed = 0xDEAD_BEEFu64;
        assert_eq!(
            wall_material_index(3, 5, 10, 10, seed),
            wall_material_index(3, 5, 10, 10, seed)
        );
    }

    #[test]
    fn wall_material_index_all_in_range() {
        for r in 0..20 {
            for c in 0..20 {
                let idx = wall_material_index(r, c, 20, 20, 0x1234_5678);
                assert!(idx < WALL_MATERIAL_VARIANTS, "got {idx}");
            }
        }
    }

    #[test]
    fn wall_material_index_quadrants_get_distinct_kinds() {
        // For a 10×10 grid sample one cell deep inside each quadrant —
        // (2,2) NW, (2,7) NE, (7,2) SW, (7,7) SE. All four must pick a
        // different material kind (the quadrant-to-kind mapping is a
        // permutation of 0..WALL_MATERIAL_VARIANTS).
        let seed = 0x1234_5678u64;
        let kinds: HashSet<usize> = [
            wall_material_index(2, 2, 10, 10, seed),
            wall_material_index(2, 7, 10, 10, seed),
            wall_material_index(7, 2, 10, 10, seed),
            wall_material_index(7, 7, 10, 10, seed),
        ]
        .into_iter()
        .collect();
        assert_eq!(kinds.len(), WALL_MATERIAL_VARIANTS);
    }

    #[test]
    fn wall_material_index_seed_permutes_mapping() {
        // For two different seeds, the cell at (2,2) (top-left quadrant)
        // should land on a different material kind for at least one seed
        // pair — otherwise the seed isn't actually permuting the mapping.
        let mut diffs = 0;
        for seed in 0..32u64 {
            if wall_material_index(2, 2, 10, 10, seed)
                != wall_material_index(2, 2, 10, 10, seed + 1)
            {
                diffs += 1;
            }
        }
        assert!(diffs > 0, "seed had no effect across 32 pairs");
    }
}
