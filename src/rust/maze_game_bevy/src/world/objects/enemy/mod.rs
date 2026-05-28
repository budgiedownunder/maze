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

pub(crate) mod goblin;

use crate::state::GameState;
use crate::world::CELL_SIZE;
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
}

/// Composite enemy assets. One sub-struct per rig variant.
pub(crate) struct EnemyAssets {
    pub(crate) goblin: goblin::GoblinAssets,
}

pub(crate) fn build_enemy_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> EnemyAssets {
    EnemyAssets {
        goblin: goblin::build_goblin_assets(meshes, materials, images),
    }
}

/// Spawns the per-cell enemy entity using the configured rig variant.
pub(crate) fn spawn_enemy_for_cell(
    commands: &mut Commands,
    assets: &EnemyAssets,
    cell: char,
    r: usize,
    c: usize,
    id: u32,
) {
    if cell != 'E' {
        return;
    }
    // Single rig variant today; extend with a `match` on the configured
    // enemy type when additional rigs land.
    goblin::spawn_goblin(commands, &assets.goblin, r, c, id);
}

/// `Update` system that lerps each `EnemyMarker`'s transform between the
/// matching enemy's `(row, col)` and `(target_row, target_col)` using
/// `enemy.move_progress()` — the same door-pattern Bevy uses for
/// `DoorState::Opening`. Layers a slight idle bob on top so resting enemies
/// stay visually alive.
pub(crate) fn enemy_animation_system(
    state: Res<GameState>,
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
    let bob = (time.elapsed_secs() * goblin::BOB_RATE).sin() * goblin::BOB_AMPLITUDE;
    for (marker, mut t) in markers.iter_mut() {
        let Some(enemy) = lookup.get(&marker.id) else {
            continue;
        };
        let from = world_pos_for(enemy.row, enemy.col);
        let to = world_pos_for(enemy.target_row, enemy.target_col);
        let interp = from.lerp(to, enemy.move_progress());
        t.translation.x = interp.x;
        t.translation.z = interp.z;
        t.translation.y = goblin::ENEMY_BASE_Y + bob;
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
