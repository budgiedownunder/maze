pub(crate) mod floor;
pub(crate) mod wall;

use crate::state::GameConfig;
use bevy::prelude::*;
use maze::CellEntity;
use std::collections::HashMap;

pub(crate) struct DecorationAssets {
    pub(crate) wall: wall::WallDecorationAssets,
    pub(crate) floor: floor::FloorAccentAssets,
}

pub(crate) fn build_decoration_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> DecorationAssets {
    DecorationAssets {
        wall: wall::build_wall_decoration_assets(meshes, materials, images),
        floor: floor::build_floor_accent_assets(meshes, materials, images),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_decorations_for_cell(
    commands: &mut Commands,
    assets: &DecorationAssets,
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    cell: char,
    r: usize,
    c: usize,
    config: &GameConfig,
    level: usize,
) {
    wall::spawn_wall_decorations_for_cell(commands, &assets.wall, grid, cell_entities, r, c, config, level);
    floor::spawn_floor_accents_for_cell(commands, &assets.floor, grid, cell, r, c, config, level);
}
