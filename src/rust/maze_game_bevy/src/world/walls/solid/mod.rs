//! Solid-wall cell rendering: the panels drawn around an open cell facing its
//! `'W'` neighbours (or the grid edge). The orientation geometry lives in the
//! sibling [`ns_panel`] / [`ew_panel`] modules; this module decides which faces to
//! draw and with which material — per face, honouring a neighbouring wall cell's
//! solid wall-type override. The per-cell tint and per-quadrant material hashes
//! that pick the default material also live here.

pub(crate) mod ew_panel;
pub(crate) mod ns_panel;

use super::{
    is_non_occluding_wall, wall_override_kind, WallAssets, PANEL_Y, WALL_MATERIAL_BRICK,
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

/// Decides whether to draw a wall panel on one face of the current cell, and if
/// so which neighbour supplies its material.
///
/// `Some(Some(rc))` draws a panel against the solid `'W'` neighbour `rc` (whose
/// override textures it); `Some(None)` draws an outer panel at the grid edge;
/// `None` suppresses the panel. `neighbour` is the in-bounds neighbour cell, or
/// `None` for the grid edge.
///
/// A panel is drawn toward a **solid** wall from any cell, and toward the **grid
/// edge** only from a *passable* (open) cell — a non-occluding cell at the edge
/// draws no outer wall, so the skybox shows past it. Panels toward an open or
/// non-occluding neighbour are always suppressed, so a non-occluding region
/// knits into one continuous, see-across space.
fn face(
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    current_non_occluding: bool,
    neighbour: Option<(usize, usize)>,
) -> Option<Option<(usize, usize)>> {
    match neighbour {
        None => (!current_non_occluding).then_some(None),
        Some((nr, nc)) => {
            if grid[nr][nc] == 'W' && !is_non_occluding_wall(grid, cell_entities, config, nr, nc) {
                Some(Some((nr, nc)))
            } else {
                None
            }
        }
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

    // A face is drawn against a solid `'W'` neighbour (any cell) or the grid edge
    // (passable cells only — a non-occluding cell shows sky past its edge). Faces
    // toward open or non-occluding neighbours are suppressed so non-occluding
    // regions read as continuous. The panel material is the cell's default kind,
    // unless the neighbouring wall cell carries a solid wall-type override (then
    // that texture is forced). The inner `Option` is the neighbour cell, or
    // `None` for a grid-edge face. See [`face`].
    let current_non_occluding = is_non_occluding_wall(grid, cell_entities, config, r, c);
    let f = |neighbour| face(grid, cell_entities, config, current_non_occluding, neighbour);
    let north = f((r > 0).then(|| (r - 1, c)));
    let south = f((r + 1 < rows).then(|| (r + 1, c)));
    let east = f((c + 1 < cols).then(|| (r, c + 1)));
    let west = f((c > 0).then(|| (r, c - 1)));

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

    /// A `cell_entities` map with a single water (non-occluding) override at `rc`.
    fn map_with_water(rc: (usize, usize)) -> HashMap<(usize, usize), Vec<CellEntity>> {
        let mut m = HashMap::new();
        m.insert(
            rc,
            vec![serde_json::from_str::<CellEntity>(r#"{"type":"W","wallType":"water"}"#).unwrap()],
        );
        m
    }

    #[test]
    fn is_non_occluding_wall_detects_water_override() {
        let grid = vec![vec!['S', 'W'], vec!['W', 'F']];
        let config = GameConfig::default();
        let water = map_with_water((0, 1));
        assert!(is_non_occluding_wall(&grid, &water, &config, 0, 1));
        // A plain solid 'W' (no override) occludes.
        let empty = HashMap::new();
        assert!(!is_non_occluding_wall(&grid, &empty, &config, 0, 1));
        // A passable cell is never non-occluding.
        assert!(!is_non_occluding_wall(&grid, &water, &config, 0, 0));
    }

    #[test]
    fn face_passable_cell_draws_outer_wall_at_edge() {
        let grid = vec![vec!['S']];
        let config = GameConfig::default();
        let empty = HashMap::new();
        assert_eq!(face(&grid, &empty, &config, false, None), Some(None));
    }

    #[test]
    fn face_non_occluding_cell_shows_sky_at_edge() {
        // The grid-edge face of a non-occluding cell is suppressed so the skybox
        // shows past it.
        let grid = vec![vec!['W']];
        let config = GameConfig::default();
        let empty = HashMap::new();
        assert_eq!(face(&grid, &empty, &config, true, None), None);
    }

    #[test]
    fn face_draws_toward_solid_wall_neighbour() {
        let grid = vec![vec!['S', 'W']];
        let config = GameConfig::default();
        let empty = HashMap::new();
        // Drawn from a passable cell …
        assert_eq!(
            face(&grid, &empty, &config, false, Some((0, 1))),
            Some(Some((0, 1)))
        );
        // … and from a non-occluding cell (a pool still abuts a solid wall).
        assert_eq!(
            face(&grid, &empty, &config, true, Some((0, 1))),
            Some(Some((0, 1)))
        );
    }

    #[test]
    fn face_suppresses_toward_open_and_non_occluding_neighbours() {
        let grid = vec![vec!['S', ' ', 'W']];
        let config = GameConfig::default();
        let empty = HashMap::new();
        // Open neighbour → no panel.
        assert_eq!(face(&grid, &empty, &config, false, Some((0, 1))), None);
        // Non-occluding neighbour → no panel (the region knits together).
        let water = map_with_water((0, 2));
        assert_eq!(face(&grid, &water, &config, false, Some((0, 2))), None);
    }

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
