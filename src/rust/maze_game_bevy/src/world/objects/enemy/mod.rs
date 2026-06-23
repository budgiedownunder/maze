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

use crate::state::{EnemyType, GameConfig, GameState};
use crate::world::{world_y, CELL_SIZE};
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
    /// Run level this enemy sits on. [`enemy_animation_system`] rewrites the
    /// rig's absolute Y each frame from a per-type resting height, so it must
    /// re-apply the level offset to keep the enemy on its stacked floor.
    pub(crate) level: usize,
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
    level: usize,
) {
    if cell != 'E' {
        return;
    }
    match enemy_type {
        EnemyType::Goblin => goblin::spawn_goblin(commands, &assets.goblin, r, c, id, level),
        EnemyType::Ghost => ghost::spawn_ghost(commands, &assets.ghost, r, c, id, level),
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
    time: Res<Time>,
    mut markers: Query<(&EnemyMarker, &mut Transform)>,
) {
    let enemies = state.game.enemies();
    if enemies.is_empty() {
        return;
    }
    // Build an id→Enemy lookup once per frame. Enemy counts are bounded
    // by `MAX_ENEMY_COUNT` in the maze crate, so the O(n) build cost is
    // negligible vs the alternative of a nested per-marker scan.
    let lookup: HashMap<u32, &maze::Enemy> = enemies.iter().map(|e| (e.id, e)).collect();
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
    for (marker, mut t) in markers.iter_mut() {
        let Some(enemy) = lookup.get(&marker.id) else {
            continue;
        };
        let from = world_pos_for(enemy.row, enemy.col);
        let to = world_pos_for(enemy.target_row, enemy.target_col);
        let interp = from.lerp(to, enemy.move_progress());
        t.translation.x = interp.x;
        t.translation.z = interp.z;
        t.translation.y = world_y(marker.level, base_y) + bob;
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

/// World-space position for the centre of cell `(r, c)`. Matches the
/// `+ 1.0` half-cell offset other object rigs use.
fn world_pos_for(r: usize, c: usize) -> Vec3 {
    Vec3::new(c as f32 * CELL_SIZE + 1.0, 0.0, r as f32 * CELL_SIZE + 1.0)
}
