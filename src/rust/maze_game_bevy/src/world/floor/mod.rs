pub(crate) mod finish;
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

#[derive(Component)]
pub(crate) struct FloorCell;

pub(crate) struct FloorAssets {
    pub(crate) floor_mesh: Option<Handle<Mesh>>,
    pub(crate) tile_mat: Option<Handle<StandardMaterial>>,
    pub(crate) start_mat: Option<Handle<StandardMaterial>>,
    pub(crate) finish_mat: Option<Handle<StandardMaterial>>,
    pub(crate) lines: lines::LineAssets,
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
    }
}

pub(crate) fn spawn_floor_for_cell(
    commands: &mut Commands,
    assets: &FloorAssets,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    lines::spawn_lines_for_cell(commands, &assets.lines, grid, r, c, placement);
    match cell {
        'S' => start::spawn_start(commands, assets, r, c, placement),
        'F' => finish::spawn_finish(commands, assets, r, c, placement),
        _ => tile::spawn_tile(commands, assets, r, c, placement),
    }
}
