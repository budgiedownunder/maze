//! Solid-wall cell rendering: the panels drawn around an open cell facing its
//! `'W'` neighbours (or the grid edge). The orientation geometry lives in
//! [`super::ns_panel`] / [`super::ew_panel`]; this module decides which faces to
//! draw and with which material — per face, honouring a neighbouring wall cell's
//! solid wall-type override. The per-cell tint and per-quadrant material hashes
//! that pick the default material also live here.

use super::{
    ew_panel, ns_panel, wall_override_kind, WallAssets, PANEL_Y, WALL_MATERIAL_BRICK,
    WALL_MATERIAL_VARIANTS, WALL_TINT_VARIANTS,
};
use crate::state::GameConfig;
use crate::world::{CELL_SIZE, HALF_CELL};
use bevy::prelude::*;
use maze::CellEntity;
use std::collections::HashMap;

/// Deterministic hash of `(row, col, seed)` → wall tint variant index in
/// `0..WALL_TINT_VARIANTS`. Used so each cell picks a stable tint for its walls;
/// the same seed always tints the same cells.
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
/// `0..WALL_MATERIAL_VARIANTS`. The maze is split into a 2×2 NW/NE/SW/SE grid;
/// each quadrant gets one kind and the seed permutes the quadrant-to-kind
/// assignment so different seeds rotate the mapping while keeping all four
/// quadrants distinct (`(zone + shuffle) % 4` over zones 0..4 is a permutation).
pub(crate) fn wall_material_index(r: usize, c: usize, rows: usize, cols: usize, seed: u64) -> usize {
    // Use 2*r < rows / 2*c < cols so the split lands at the midpoint without
    // floating-point or integer-division surprises.
    let zone_r: u64 = if r * 2 < rows { 0 } else { 1 };
    let zone_c: u64 = if c * 2 < cols { 0 } else { 1 };
    let zone = zone_r * 2 + zone_c;
    let shuffle = (seed.wrapping_mul(0xF0E1_D2C3_B4A5_9687) >> 32) % WALL_MATERIAL_VARIANTS as u64;
    ((zone + shuffle) % WALL_MATERIAL_VARIANTS as u64) as usize
}

/// The panel material kind for a face: a `'W'` neighbour's solid wall-type
/// override forces its texture; otherwise `default_kind` (variation / per-maze).
/// `neighbour` is `None` for a grid-edge face (no cell beyond it).
fn face_kind(
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    neighbour: Option<(usize, usize)>,
    default_kind: usize,
) -> usize {
    match neighbour {
        Some(rc) => {
            wall_override_kind(cell_entities.get(&rc).and_then(|v| v.first())).unwrap_or(default_kind)
        }
        None => default_kind,
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_walls_for_cell(
    commands: &mut Commands,
    assets: &WallAssets,
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    r: usize,
    c: usize,
    config: &GameConfig,
) {
    let rows = grid.len();
    let cols = grid[r].len();
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;

    // A face is drawn when its neighbour is a wall (`'W'`) or the grid edge. Its
    // material is the cell's default kind, unless the neighbouring wall cell
    // carries a solid wall-type override (then that texture is forced for the
    // panel facing it). `None` = a grid-edge face, which has no neighbour cell.
    let north = (r == 0 || grid[r - 1][c] == 'W').then(|| (r > 0).then(|| (r - 1, c)));
    let south = (r + 1 >= rows || grid[r + 1][c] == 'W').then(|| (r + 1 < rows).then(|| (r + 1, c)));
    let east = (c + 1 >= cols || grid[r][c + 1] == 'W').then(|| (c + 1 < cols).then(|| (r, c + 1)));
    let west = (c == 0 || grid[r][c - 1] == 'W').then(|| (c > 0).then(|| (r, c - 1)));

    // Material variation supersedes per-cell tint: when on, every wall in
    // this cell takes the quadrant's material kind and the `wall_tint`
    // toggle is bypassed. When off, fall back to the original tinted-brick
    // path (tint index 0 when `wall_tint` is also off).
    if config.landmarks.wall_material_variation {
        let default_kind = wall_material_index(r, c, rows, cols, config.seed);
        if let Some(n) = north {
            let kind = face_kind(cell_entities, n, default_kind);
            ns_panel::spawn_ns_face_material(commands, &assets.ns, kind, Vec3::new(x, PANEL_Y, z - HALF_CELL));
        }
        if let Some(n) = south {
            let kind = face_kind(cell_entities, n, default_kind);
            ns_panel::spawn_ns_face_material(commands, &assets.ns, kind, Vec3::new(x, PANEL_Y, z + HALF_CELL));
        }
        if let Some(n) = east {
            let kind = face_kind(cell_entities, n, default_kind);
            ew_panel::spawn_ew_face_material(commands, &assets.ew, kind, Vec3::new(x + HALF_CELL, PANEL_Y, z));
        }
        if let Some(n) = west {
            let kind = face_kind(cell_entities, n, default_kind);
            ew_panel::spawn_ew_face_material(commands, &assets.ew, kind, Vec3::new(x - HALF_CELL, PANEL_Y, z));
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
    // The texture kind for the tinted path is configured per difficulty via
    // `GameConfig.wall_type` — same set of four kinds the quadrant-variation
    // path uses, but a single choice for the whole maze. A non-occluding
    // per-maze type has no panel material; fall back to brick for any solid
    // panel still drawn around such cells.
    let default_kind = config.wall_type.to_kind_index().unwrap_or(WALL_MATERIAL_BRICK);

    if let Some(n) = north {
        let kind = face_kind(cell_entities, n, default_kind);
        ns_panel::spawn_ns_face_tinted(commands, &assets.ns, kind, tint, Vec3::new(x, PANEL_Y, z - HALF_CELL));
    }
    if let Some(n) = south {
        let kind = face_kind(cell_entities, n, default_kind);
        ns_panel::spawn_ns_face_tinted(commands, &assets.ns, kind, tint, Vec3::new(x, PANEL_Y, z + HALF_CELL));
    }
    if let Some(n) = east {
        let kind = face_kind(cell_entities, n, default_kind);
        ew_panel::spawn_ew_face_tinted(commands, &assets.ew, kind, tint, Vec3::new(x + HALF_CELL, PANEL_Y, z));
    }
    if let Some(n) = west {
        let kind = face_kind(cell_entities, n, default_kind);
        ew_panel::spawn_ew_face_tinted(commands, &assets.ew, kind, tint, Vec3::new(x - HALF_CELL, PANEL_Y, z));
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
