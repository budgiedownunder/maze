pub(crate) mod iron_fence;
pub(crate) mod lava;
pub(crate) mod rim;
pub(crate) mod solid;
pub(crate) mod water;

pub(crate) use solid::spawn_walls_for_cell;

use crate::state::{GameConfig, WallType};
use crate::world::objects::overrides::resolve_wall_type;
use crate::world::textures::brick::make_brick_texture;
use crate::world::textures::cobblestone::make_cobblestone_texture;
use crate::world::textures::dressed_stone::make_dressed_stone_texture;
use crate::world::textures::wood::make_wood_texture;
use crate::world::CELL_SIZE;
use bevy::prelude::*;
use maze::CellEntity;
use solid::ew_panel::EwPanelAssets;
use solid::ns_panel::{NsPanelAssets, WallMaterialSpec};
use std::collections::HashMap;

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
pub(crate) const WALL_MATERIAL_DRESSED_STONE: usize = 1;
pub(crate) const WALL_MATERIAL_WOOD: usize = 2;
pub(crate) const WALL_MATERIAL_COBBLESTONE: usize = 3;

// Per-material emissive RGB tints + UV scales. N/S-facing panels
// (ahead / behind the player) use the lighter half; E/W-facing panels
// (sides) use the darker half so the same material reads as the same
// kind from any angle but still has directional shading depth.
// One pair per index in `0..WALL_MATERIAL_VARIANTS`.

// Per-material wall emissive tints fall into two families depending on
// whether the texture is monochrome or RGB-coloured:
//   - Greyscale textures (brick, dressed_stone): emissive carries the
//     chromaticity. Texture is greyscale, emissive RGB tints it.
//   - RGB-coloured textures (wood, cobblestone — Step 11.S): texture
//     carries per-pixel chromaticity AND per-plank/per-cobble tone
//     variation. Emissive must stay near-neutral brightness or it
//     compounds with the texture and saturates.

// Brick (greyscale texture) — slightly cool stone grey.
const NS_BRICK_EMISSIVE: (f32, f32, f32) = (0.38, 0.38, 0.40);
const EW_BRICK_EMISSIVE: (f32, f32, f32) = (0.14, 0.14, 0.16);
const BRICK_UV: Vec2 = Vec2::new(3.0, 5.0);

// Dressed stone (greyscale texture) — slightly warm pale stone.
const NS_DRESSED_STONE_EMISSIVE: (f32, f32, f32) = (0.50, 0.48, 0.42);
const EW_DRESSED_STONE_EMISSIVE: (f32, f32, f32) = (0.22, 0.21, 0.19);
const DRESSED_STONE_UV: Vec2 = Vec2::new(2.0, 3.0);

// Wood (RGB-coloured texture with per-plank tone palette). Neutral
// brightness — chromaticity lives in the texture.
const NS_WOOD_EMISSIVE: (f32, f32, f32) = (0.55, 0.55, 0.55);
const EW_WOOD_EMISSIVE: (f32, f32, f32) = (0.28, 0.28, 0.28);
const WOOD_UV: Vec2 = Vec2::new(1.0, 4.0);

// Cobblestone (RGB-coloured texture with per-cobble tone palette).
// Neutral brightness — chromaticity lives in the texture.
const NS_COBBLESTONE_EMISSIVE: (f32, f32, f32) = (0.45, 0.45, 0.45);
const EW_COBBLESTONE_EMISSIVE: (f32, f32, f32) = (0.17, 0.17, 0.17);
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
        ns: solid::ns_panel::build_ns_panel_assets(meshes, materials, &ns_specs),
        ew: solid::ew_panel::build_ew_panel_assets(meshes, materials, &ew_specs),
    }
}

/// The wall material kind (`WALL_MATERIAL_*` index) used by cell `(r, c)`.
/// Mirrors the kind-selection logic in [`solid::spawn_walls_for_cell`]: the
/// per-quadrant material variation when that landmark is on, otherwise the
/// single configured `wall_type`. A door panel reuses this so it always
/// renders in the same material as the wall it sits between.
pub(crate) fn wall_kind_for_cell(
    r: usize,
    c: usize,
    rows: usize,
    cols: usize,
    config: &GameConfig,
) -> usize {
    if config.landmarks.wall_material_variation {
        solid::wall_material_index(r, c, rows, cols, config.seed)
    } else {
        // A non-occluding per-maze wall_type has no panel material; fall back to
        // brick for any solid panel still drawn around it.
        config.wall_type.to_kind_index().unwrap_or(WALL_MATERIAL_BRICK)
    }
}

/// The forced `WALL_MATERIAL_*` index from a `'W'` cell's wall-type override, or
/// `None` when the cell has no wall override, a field-less one, or a
/// non-occluding type (which has no panel material). Used to texture an adjacent
/// open cell's panel from its wall neighbour's override; `None` means "use the
/// cell's normal default kind", so plain walls keep their variation/per-maze look.
pub(crate) fn wall_override_kind(entity: Option<&CellEntity>) -> Option<usize> {
    if let Some(CellEntity::Wall(over)) = entity {
        if let Some(wt) = over.wall_type {
            return WallType::from_wire_str(wt.as_wire_str()).to_kind_index();
        }
    }
    None
}

/// Whether the `'W'` cell at `(r, c)` is non-occluding (renders an in-cell pool
/// / bar lattice rather than a solid panel). A non-`'W'` cell is never
/// non-occluding. Resolves the cell's per-cell `wallType` override against the
/// per-maze default — the same resolution the spawn loop uses to decide whether
/// to skip the cell. Shared by the panel-suppression logic ([`solid`]) and the
/// door corridor / leaf logic ([`crate::world::objects::door`]), which both
/// treat a non-occluding neighbour as an opening, not a wall (its panel is
/// suppressed, so a swing has nothing to anchor against and a leaf must seal it).
pub(crate) fn is_non_occluding_wall(
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    r: usize,
    c: usize,
) -> bool {
    grid[r][c] == 'W'
        && resolve_wall_type(
            cell_entities.get(&(r, c)).and_then(|v| v.first()),
            config.wall_type,
        )
        .is_non_occluding()
}

/// `true` when the player can look *across* cell `(r, c)` to whatever lies
/// beyond — an open/passable cell, or a low water/lava pool seen over its
/// surface. A solid wall blocks the view, and an iron fence is looked *through*
/// (between its bars), not across — both are `false`. The iron fence uses this to
/// pick its edges: it draws a bar grille on an edge facing a looked-across
/// neighbour (open ground or a pool), and so never between two adjacent fences
/// (a fence isn't looked across, so the run stays continuous with no inner bars).
pub(crate) fn can_be_looked_across(
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    r: usize,
    c: usize,
) -> bool {
    grid[r][c] != 'W'
        || matches!(
            resolve_wall_type(
                cell_entities.get(&(r, c)).and_then(|v| v.first()),
                config.wall_type,
            ),
            WallType::Water | WallType::Lava
        )
}

/// `true` when cell `(r, c)` is a water or lava **pool** — a non-occluding `'W'`
/// cell whose surface is recessed below floor level. The pool rim
/// ([`rim::spawn_pool_rim`]) uses this to leave the shared edge between two
/// adjacent pools open (one continuous basin) while skirting every edge that
/// meets a non-pool cell.
pub(crate) fn is_pool(
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    r: usize,
    c: usize,
) -> bool {
    grid[r][c] == 'W'
        && matches!(
            resolve_wall_type(
                cell_entities.get(&(r, c)).and_then(|v| v.first()),
                config.wall_type,
            ),
            WallType::Water | WallType::Lava
        )
}

/// In-cell geometry for the non-occluding wall types: the water / lava pool
/// surfaces and the iron-fence bar lattice. Built once per session alongside
/// [`WallAssets`] and reused for every non-occluding `'W'` cell.
pub(crate) struct NonOccludingAssets {
    pub(crate) water: water::WaterAssets,
    pub(crate) lava: lava::LavaAssets,
    pub(crate) iron_fence: iron_fence::IronFenceAssets,
    pub(crate) rim: rim::RimAssets,
}

pub(crate) fn build_non_occluding_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> NonOccludingAssets {
    NonOccludingAssets {
        water: water::build_water_assets(meshes, materials),
        lava: lava::build_lava_assets(meshes, materials),
        iron_fence: iron_fence::build_iron_fence_assets(meshes, materials),
        rim: rim::build_rim_assets(meshes, materials, images),
    }
}

/// Spawns the in-cell geometry for a non-occluding wall cell `(r, c)`. Water and
/// lava render a floor-level pool surface that doubles as the cell's floor;
/// iron-fence renders bar grilles on its open edges (its floor tile is spawned
/// separately by the caller, since — unlike the pools — the fence stands on a
/// normal floor). A
/// solid `wall_type` never reaches here: the caller gates on
/// [`WallType::is_non_occluding`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_non_occluding_for_cell(
    commands: &mut Commands,
    assets: &NonOccludingAssets,
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    wall_type: WallType,
    r: usize,
    c: usize,
) {
    match wall_type {
        // Water / lava render a recessed surface plus rim skirts filling the band
        // up to floor level on every edge facing a non-pool cell.
        WallType::Water => {
            water::spawn_water(commands, &assets.water, r, c);
            rim::spawn_pool_rim(commands, &assets.rim, wall_type, grid, cell_entities, config, r, c);
        }
        WallType::Lava => {
            lava::spawn_lava(commands, &assets.lava, r, c);
            rim::spawn_pool_rim(commands, &assets.rim, wall_type, grid, cell_entities, config, r, c);
        }
        // The fence needs the grid + overrides to bar only the edges facing a
        // passable cell or a water/lava pool.
        WallType::IronFence => {
            iron_fence::spawn_iron_fence(commands, &assets.iron_fence, grid, cell_entities, config, r, c)
        }
        WallType::Brick | WallType::DressedStone | WallType::Wood | WallType::Cobblestone => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entity(json: &str) -> CellEntity {
        serde_json::from_str(json).expect("valid cell-entity JSON")
    }

    #[test]
    fn wall_override_kind_forces_solid_texture() {
        let cobble = entity(r#"{ "type": "W", "wallType": "cobblestone" }"#);
        assert_eq!(wall_override_kind(Some(&cobble)), Some(WALL_MATERIAL_COBBLESTONE));
        let wood = entity(r#"{ "type": "W", "wallType": "wood" }"#);
        assert_eq!(wall_override_kind(Some(&wood)), Some(WALL_MATERIAL_WOOD));
    }

    #[test]
    fn wall_override_kind_is_none_for_non_panel_cases() {
        // Non-occluding type has no panel material; a field-less or absent
        // override, or a non-wall entity, also yields None (use the default kind).
        assert_eq!(wall_override_kind(Some(&entity(r#"{ "type": "W", "wallType": "lava" }"#))), None);
        assert_eq!(wall_override_kind(Some(&entity(r#"{ "type": "W" }"#))), None);
        assert_eq!(wall_override_kind(Some(&entity(r#"{ "type": "E", "enemyType": "ghost" }"#))), None);
        assert_eq!(wall_override_kind(None), None);
    }
}
