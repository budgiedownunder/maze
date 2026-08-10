//! Enemy entities — visual rigs that mirror the runtime enemies tracked by
//! `MazeGame`. The marker carries the enemy's stable `id` so the animation
//! system can match it to the matching `maze::Enemy` each frame and lerp the
//! visual position between `(row, col)` and `(target_row, target_col)` using
//! `enemy.move_progress()`.
//!
//! Each enemy-type variant lives in a sibling module (currently just
//! [`goblin`]) and produces its own asset sub-struct. The orchestrator
//! composes those into [`EnemyAssets`] and dispatches per-cell spawning to
//! the right rig.

pub(crate) mod ghost;
pub(crate) mod goblin;

use crate::state::{EnemyType, GameConfig, GameState, MultiLevelRun};
use crate::world::{LevelPlacement, CELL_SIZE, LevelTag};
use crate::world::visibility::LevelWindow;
use bevy::prelude::*;
use std::collections::HashMap;

/// Per-enemy entity marker. The `id` matches `maze::Enemy::id` so
/// [`enemy_animation_system`] can find each enemy's current game-state
/// position each frame.
#[derive(Component)]
pub(crate) struct EnemyMarker {
    pub(crate) id: u32,
    /// Original `'E'` spawn cell — recorded for diagnostics / future
    /// respawn paths. The live runtime position comes from
    /// `state.game.enemies()` each frame.
    #[allow(dead_code)]
    pub(crate) spawn_cell: (usize, usize),
    /// Where this enemy's level sits in world space. [`enemy_animation_system`]
    /// recomputes the rig's absolute position each frame from its `(row, col)`,
    /// so it re-applies this placement's X/Z centring + Y lift to keep the enemy
    /// on its stacked, centred floor.
    pub(crate) placement: LevelPlacement,
}

/// Composite enemy assets. One sub-struct per rig variant.
pub(crate) struct EnemyAssets {
    pub(crate) goblin: goblin::GoblinAssets,
    pub(crate) ghost: ghost::GhostAssets,
}

pub(crate) fn build_enemy_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> EnemyAssets {
    EnemyAssets {
        goblin: goblin::build_goblin_assets(meshes, materials, images),
        ghost: ghost::build_ghost_assets(meshes, materials),
    }
}

/// Spawns the per-cell enemy entity using the rig variant selected by
/// `enemy_type`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_enemy_for_cell(
    commands: &mut Commands,
    assets: &EnemyAssets,
    enemy_type: EnemyType,
    cell: char,
    r: usize,
    c: usize,
    id: u32,
    placement: LevelPlacement,
) {
    if cell != 'E' {
        return;
    }
    match enemy_type {
        EnemyType::Goblin => goblin::spawn_goblin(commands, &assets.goblin, r, c, id, placement),
        EnemyType::Ghost => ghost::spawn_ghost(commands, &assets.ghost, r, c, id, placement),
    }
}

/// `Update` system that lerps each `EnemyMarker`'s transform between the
/// matching enemy's `(row, col)` and `(target_row, target_col)` using
/// `enemy.move_progress()` — the same door-pattern Bevy uses for
/// `DoorState::Opening`. Layers a slight idle bob on top so resting enemies
/// stay visually alive.
pub(crate) fn enemy_animation_system(
    state: Res<GameState>,
    config: Res<GameConfig>,
    run: Res<MultiLevelRun>,
    time: Res<Time>,
    window: Res<LevelWindow>,
    mut markers: Query<(&EnemyMarker, &mut Transform, &LevelTag)>,
) {
    // Build an id→Enemy lookup once per frame from the live (current) level's
    // game. Enemy counts are bounded by `MAX_ENEMY_COUNT`, so the O(n) build cost
    // is negligible. Only the current level's enemies are driven by it — the gate
    // below also stops an off-level marker whose id happens to collide with a
    // current-level enemy from being dragged around by it.
    let enemies = state.game.enemies();
    let lookup: HashMap<u32, &maze::Enemy> = enemies.iter().map(|e| (e.id, e)).collect();
    let current_level = run.current_level;
    let (base_y, bob) = match config.enemy_type {
        EnemyType::Goblin => (
            goblin::ENEMY_BASE_Y,
            (time.elapsed_secs() * goblin::BOB_RATE).sin() * goblin::BOB_AMPLITUDE,
        ),
        EnemyType::Ghost => (
            ghost::ENEMY_BASE_Y,
            (time.elapsed_secs() * ghost::BOB_RATE).sin() * ghost::BOB_AMPLITUDE,
        ),
    };
    for (marker, mut t, tag) in markers.iter_mut() {
        // Off-window floors are neither drawn nor animated.
        if !window.contains(tag.0) {
            continue;
        }
        // Only the current level's enemies follow their runtime position. Every
        // other level's enemies hold the position they were last left at (their
        // `MazeGame` isn't ticked) — completed levels below, not-yet-reached levels
        // above — and only take the idle bob, so they stay lively in place.
        if marker.placement.level == current_level {
            if let Some(enemy) = lookup.get(&marker.id) {
                let from = world_pos_for(enemy.row, enemy.col);
                let to = world_pos_for(enemy.target_row, enemy.target_col);
                let interp = from.lerp(to, enemy.move_progress());
                t.translation.x = marker.placement.world_x(interp.x);
                t.translation.z = marker.placement.world_z(interp.z);
                // Face the direction of travel — eyes (and teeth, when present)
                // are positioned on the rig's local +Z face, so rotating around Y
                // by the heading-angle aims them along the movement vector. A
                // resting enemy (target == current) keeps its prior rotation:
                // `dx` and `dz` are 0 so the conditional guard skips the update.
                let dx = to.x - from.x;
                let dz = to.z - from.z;
                if dx != 0.0 || dz != 0.0 {
                    // `atan2(dx, dz)` gives the angle whose +Z heading aligns with
                    // the (dx, dz) vector, matching how player camera yaw is
                    // measured elsewhere in the crate.
                    t.rotation = Quat::from_rotation_y(dx.atan2(dz));
                }
            }
        }
        // The idle bob keeps every enemy (current level or not) visually alive.
        t.translation.y = marker.placement.world_y(base_y) + bob;
    }
}

/// Despawns a completed lower level's enemies the moment the player climbs past
/// it, when [`GameConfig::hide_completed_enemies`] is set. The player only ever
/// ascends, so a level below `current_level` is never revisited — freeing its
/// enemy rigs (root + children) trims the live entity/memory load on a multi-level
/// stack. A no-op when the flag is off, before the first ascend, or for
/// single-level games. Shaped like `hatch_close_watcher` (fires once per level
/// change via a `Local` cursor).
pub(crate) fn despawn_completed_level_enemies_system(
    mut commands: Commands,
    config: Res<GameConfig>,
    run: Res<MultiLevelRun>,
    mut last_level: Local<usize>,
    enemies: Query<(Entity, &EnemyMarker)>,
) {
    if !config.hide_completed_enemies || run.current_level == *last_level {
        return;
    }
    *last_level = run.current_level;
    for (entity, marker) in &enemies {
        if marker.placement.level < run.current_level {
            commands.entity(entity).despawn();
        }
    }
}

/// World-space position for the centre of cell `(r, c)`. Matches the
/// `+ 1.0` half-cell offset other object rigs use.
fn world_pos_for(r: usize, c: usize) -> Vec3 {
    Vec3::new(c as f32 * CELL_SIZE + 1.0, 0.0, r as f32 * CELL_SIZE + 1.0)
}
