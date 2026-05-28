//! Health-pickup entities — visual rigs for `'H'` cells. The marker
//! carries the pickup's `(row, col)` so the tick driver can despawn the
//! matching entity directly from a `GameEvent::PlayerHealed { cell, .. }`.
//!
//! Each health-pickup-style variant lives in a sibling module (currently
//! just [`heart`]) and produces its own asset sub-struct. The orchestrator
//! composes those into [`HealthAssets`] and dispatches per-cell spawning
//! to the right rig.

pub(crate) mod heart;

use bevy::prelude::*;

/// Per-pickup entity marker. `cell` matches the `'H'` cell coordinate so
/// the tick driver can despawn this entity from a
/// `GameEvent::PlayerHealed { cell, .. }` payload directly.
#[derive(Component)]
pub(crate) struct HealthMarker {
    pub(crate) cell: (usize, usize),
}

/// Composite health-pickup assets. One sub-struct per rig variant.
pub(crate) struct HealthAssets {
    pub(crate) heart: heart::HeartAssets,
}

pub(crate) fn build_health_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> HealthAssets {
    HealthAssets {
        heart: heart::build_heart_assets(meshes, materials),
    }
}

/// Spawns the per-cell health-pickup entity using the configured rig
/// variant.
pub(crate) fn spawn_health_for_cell(
    commands: &mut Commands,
    assets: &HealthAssets,
    cell: char,
    r: usize,
    c: usize,
) {
    if cell != 'H' {
        return;
    }
    // Single rig variant today; extend with a `match` on the configured
    // health-pickup style when additional rigs land.
    heart::spawn_heart(commands, &assets.heart, r, c);
}

/// `Update` system that applies the per-rig idle animation. Currently
/// forwards to the only rig's pulse animation; extend with a rig-aware
/// dispatch when additional rigs land.
pub(crate) use heart::heart_pulse_system as health_animation_system;
