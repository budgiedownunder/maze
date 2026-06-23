pub(crate) mod finish;
pub(crate) mod hatch;
pub(crate) mod lines;
pub(crate) mod start;
pub(crate) mod tile;

use crate::world::textures::tile::make_tile_texture;
use crate::world::{LevelPlacement, CELL_SIZE};
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Floor-tile mesh thickness (units, vertical extent). Thin enough to
/// look flat from the player's eye height without z-fighting the grid
/// lines that sit just above it.
const FLOOR_THICKNESS: f32 = 0.01;

/// Fraction of a start / finish tile's thickness given to a plain-stone underside
/// cap. The cell then reads as ordinary floor from the level below — so an open
/// multi-level stack doesn't reveal the upper level's start / finish through the
/// floor — while its coloured top still shows from above. The remainder is the
/// coloured layer; the two stack flush within the normal tile thickness, so all
/// floor tiles share one height.
const STONE_CAP_FRAC: f32 = 0.35;

#[derive(Component)]
pub(crate) struct FloorCell;

/// Spawns a start / finish tile as two flush layers inside the normal tile
/// thickness: a plain-stone underside cap (untagged scenery) and the coloured top
/// `mat`, tagged with `marker` (`StartCell` / `FinishCell`) + [`FloorCell`]. From
/// below the cell looks like ordinary stone; from above it shows its start /
/// finish colour. With no render assets (headless), falls back to a single tagged
/// `FloorCell` at the tile centre — identical to the plain-tile path, so the
/// entity counts/positions the tests assert are unchanged.
pub(crate) fn spawn_capped_tile<M: Bundle>(
    commands: &mut Commands,
    assets: &FloorAssets,
    mat: Option<Handle<StandardMaterial>>,
    marker: M,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let y = placement.world_y(0.0);
    match (assets.floor_mesh.clone(), assets.tile_mat.clone(), mat) {
        (Some(mesh), Some(stone_mat), Some(top_mat)) => {
            let bottom = y - FLOOR_THICKNESS / 2.0;
            let cap_h = FLOOR_THICKNESS * STONE_CAP_FRAC;
            let top_h = FLOOR_THICKNESS - cap_h;
            // Plain-stone underside cap — the bottom slice, what the level below sees.
            commands.spawn((
                Transform::from_xyz(x, bottom + cap_h / 2.0, z)
                    .with_scale(Vec3::new(1.0, STONE_CAP_FRAC, 1.0)),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(stone_mat),
            ));
            // Coloured top — the logical floor cell, what the player on this level sees.
            commands.spawn((
                marker,
                FloorCell,
                Transform::from_xyz(x, bottom + cap_h + top_h / 2.0, z)
                    .with_scale(Vec3::new(1.0, 1.0 - STONE_CAP_FRAC, 1.0)),
                Mesh3d(mesh),
                MeshMaterial3d(top_mat),
            ));
        }
        _ => {
            commands.spawn((marker, FloorCell, Transform::from_xyz(x, y, z)));
        }
    }
}

pub(crate) struct FloorAssets {
    pub(crate) floor_mesh: Option<Handle<Mesh>>,
    pub(crate) tile_mat: Option<Handle<StandardMaterial>>,
    pub(crate) start_mat: Option<Handle<StandardMaterial>>,
    pub(crate) finish_mat: Option<Handle<StandardMaterial>>,
    pub(crate) lines: lines::LineAssets,
    /// Round-hatch meshes + materials (start cells above a ladder finish).
    pub(crate) hatch: hatch::HatchAssets,
}

pub(crate) fn build_floor_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> FloorAssets {
    // Thin cuboid floor tile — Plane3d does not resolve reliably in the asset
    // pipeline. Shared by tile / start / finish.
    let floor_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, FLOOR_THICKNESS, CELL_SIZE)));
    // Tile texture is shared by tile / start / finish materials — build once.
    let tile_tex = images.as_mut().map(|imgs| make_tile_texture(imgs));
    FloorAssets {
        floor_mesh,
        tile_mat: tile::build_tile_material(materials, &tile_tex),
        start_mat: start::build_start_material(materials, &tile_tex),
        finish_mat: finish::build_finish_material(materials, &tile_tex),
        lines: lines::build_line_assets(meshes, materials),
        hatch: hatch::build_hatch_assets(meshes, materials, &tile_tex),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_floor_for_cell(
    commands: &mut Commands,
    assets: &FloorAssets,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    placement: LevelPlacement,
    // True for a start cell sitting above a ladder finish on the level below: the
    // solid start tile is replaced by an (open) hatch lid the climb emerges
    // through. Ignored for every other cell.
    hatch_at_start: bool,
) {
    lines::spawn_lines_for_cell(commands, &assets.lines, grid, r, c, placement);
    match cell {
        'S' if hatch_at_start => hatch::spawn_hatch(commands, assets, r, c, placement),
        'S' => start::spawn_start(commands, assets, r, c, placement),
        'F' => finish::spawn_finish(commands, assets, r, c, placement),
        _ => tile::spawn_tile(commands, assets, r, c, placement),
    }
}
