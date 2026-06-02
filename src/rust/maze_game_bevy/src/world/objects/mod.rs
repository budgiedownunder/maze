pub(crate) mod dead_end;
pub(crate) mod door;
pub(crate) mod enemy;
pub(crate) mod finish;
pub(crate) mod health;
pub(crate) mod key_holder;

use crate::state::GameConfig;
use bevy::prelude::*;

pub(crate) struct ObjectAssets {
    pub(crate) finish: finish::FinishAssets,
    pub(crate) dead_end: dead_end::DeadEndAssets,
    pub(crate) key_holder: key_holder::KeyHolderAssets,
    /// Door slab assets. Doors are spawned by `spawn_world` (not
    /// `spawn_objects_for_cell`) because their panel borrows the cell's wall
    /// material, which is only available alongside the wall assets.
    pub(crate) door: door::DoorAssets,
    pub(crate) enemy: enemy::EnemyAssets,
    pub(crate) health: health::HealthAssets,
}

pub(crate) fn build_object_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> ObjectAssets {
    ObjectAssets {
        finish: finish::build_finish_assets(meshes, materials),
        dead_end: dead_end::build_dead_end_assets(meshes, materials),
        key_holder: key_holder::build_key_holder_assets(meshes, materials),
        door: door::build_door_assets(meshes, materials),
        enemy: enemy::build_enemy_assets(meshes, materials, images),
        health: health::build_health_assets(meshes, materials),
    }
}

/// `enemy_id` is consumed only when `cell == 'E'`; the caller is
/// responsible for bumping its counter on each `'E'` it dispatches to
/// match the row-major scan order `MazeGame` uses when seeding enemies.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_objects_for_cell(
    commands: &mut Commands,
    assets: &ObjectAssets,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    config: &GameConfig,
    enemy_id: u32,
) {
    finish::spawn_finish_for_cell(commands, &assets.finish, cell, r, c);
    dead_end::spawn_dead_end_object_for_cell(
        commands,
        &assets.dead_end,
        grid,
        cell,
        r,
        c,
        config,
    );
    key_holder::spawn_key_holder_for_cell(commands, &assets.key_holder, config.key_holder, cell, r, c);
    enemy::spawn_enemy_for_cell(commands, &assets.enemy, config.enemy_type, cell, r, c, enemy_id);
    health::spawn_health_for_cell(commands, &assets.health, config.health_style, cell, r, c);
}
