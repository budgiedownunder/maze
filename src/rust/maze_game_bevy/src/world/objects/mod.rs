pub(crate) mod common;
pub(crate) mod dead_end;
pub(crate) mod door;
pub(crate) mod enemy;
pub(crate) mod finish;
pub(crate) mod health;
pub(crate) mod key_holder;
pub(crate) mod overrides;
pub(crate) mod treasure;

use crate::state::{GameConfig, TreasureStyle};
use crate::world::LevelPlacement;
use bevy::prelude::*;
use maze::CellEntity;

pub(crate) struct ObjectAssets {
    pub(crate) finish: finish::FinishAssets,
    /// Shared decorative-prop assets (brazier / urn / pillar / chest + the
    /// inverted-hull primitives), consumed by both the dead-end landmarks and
    /// the key holder.
    pub(crate) common: common::CommonObjectAssets,
    pub(crate) key_holder: key_holder::KeyHolderAssets,
    /// Door slab assets. Doors are spawned by `spawn_world` (not
    /// `spawn_objects_for_cell`) because their panel borrows the cell's wall
    /// material, which is only available alongside the wall assets.
    pub(crate) door: door::DoorAssets,
    pub(crate) enemy: enemy::EnemyAssets,
    pub(crate) health: health::HealthAssets,
    pub(crate) treasure: treasure::TreasureAssets,
}

pub(crate) fn build_object_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> ObjectAssets {
    // The prop rigs come first: the finish ladder and the floating key bake
    // their own geometry against the shared outline material they carry.
    let common = common::build_common_object_assets(meshes, materials);
    ObjectAssets {
        finish: finish::build_finish_assets(meshes, materials, &common),
        key_holder: key_holder::build_key_holder_assets(meshes, materials, &common),
        common,
        door: door::build_door_assets(meshes, materials),
        enemy: enemy::build_enemy_assets(meshes, materials, images),
        health: health::build_health_assets(meshes, materials),
        treasure: treasure::build_treasure_assets(meshes, materials),
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
    // The cell's override (if any), used to pick a per-cell rig in place of the
    // per-maze `GameConfig` default.
    cell_entity: Option<&CellEntity>,
    enemy_id: u32,
    // Sparkle rays a treasure in this cell gets — uniform across the maze (see
    // treasure::rays_per_chest); ignored for non-treasure cells.
    treasure_rays: usize,
    placement: LevelPlacement,
    // Whether this cell's level is the run's top (final) level. The final level's
    // finish keeps the gold orb; interim levels draw a transition rig instead. A
    // single-level game is its own final level, so the orb always shows —
    // unchanged from before multi-level runs.
    is_final: bool,
    // Whether an interim finish here may use a ladder — true only when the next
    // level's start sits directly above to climb onto; otherwise it falls back to
    // a portal. Ignored on the final level (orb) and non-finish cells.
    ladder_allowed: bool,
    // Cells whose dead-end landmark must be suppressed — the gallery code-spawns a
    // finish rig into these plain-space alcoves and a landmark would clash. Empty
    // for ordinary levels.
    dead_end_skip: &[(usize, usize)],
) {
    finish::spawn_finish_for_cell(
        commands,
        &assets.finish,
        grid,
        cell,
        r,
        c,
        placement,
        config.finish_type,
        config.seed,
        is_final,
        ladder_allowed,
        config.hide_finish_orb,
        config.disable_orb_shadows,
        config.disable_orb_light,
    );
    if !dead_end_skip.contains(&(r, c)) {
        dead_end::spawn_dead_end_object_for_cell(commands, &assets.common, grid, cell, r, c, config, placement);
    }
    let key_holder = overrides::resolve_key_holder(
        cell_entity,
        config.key_holder,
        config.key_holder_random,
        config.seed,
        r,
        c,
    );
    let enemy_type = overrides::resolve_enemy_type(
        cell_entity,
        config.enemy_type,
        config.enemy_type_random,
        config.seed,
        r,
        c,
    );
    let health_style = overrides::resolve_health_style(
        cell_entity,
        config.health_style,
        config.health_style_random,
        config.seed,
        r,
        c,
    );
    let treasure_style = overrides::resolve_treasure_style(cell_entity, TreasureStyle::default());
    key_holder::spawn_key_holder_for_cell(
        commands,
        &assets.key_holder,
        &assets.common,
        key_holder,
        config,
        grid,
        cell,
        r,
        c,
        placement,
    );
    enemy::spawn_enemy_for_cell(commands, &assets.enemy, enemy_type, cell, r, c, enemy_id, placement);
    health::spawn_health_for_cell(commands, &assets.health, health_style, cell, r, c, placement);
    treasure::spawn_treasure_for_cell(
        commands,
        &assets.treasure,
        &assets.common,
        treasure_style,
        grid,
        cell,
        r,
        c,
        treasure_rays,
        placement,
        config,
    );
}
