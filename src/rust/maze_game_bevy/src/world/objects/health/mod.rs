//! Health-pickup entities — visual rigs for `'H'` cells. The marker
//! carries the pickup's `(row, col)` so the tick driver can despawn the
//! matching entity directly from a `GameEvent::PlayerHealed { cell, .. }`.
//!
//! Each health-pickup-style variant lives in a sibling module and
//! produces its own asset sub-struct. The orchestrator composes those
//! into [`HealthAssets`] and dispatches per-cell spawning to the right
//! rig. Idle animation (gentle scale pulse + slow Y-spin) is shared
//! across rigs via [`health_animation_system`], applied uniformly to
//! every `HealthMarker` regardless of which rig produced it.

pub(crate) mod heart;
pub(crate) mod potion;

use crate::state::HealthStyle;
use bevy::prelude::*;

/// Scale-pulse frequency (radians/sec) applied to every health pickup.
const PULSE_RATE: f32 = 2.0;
/// Scale-pulse amplitude — half the peak-to-peak swing.
const PULSE_AMPLITUDE: f32 = 0.08;
/// Y-axis rotation rate (radians/sec) for the slow idle spin.
const SPIN_RATE: f32 = 1.0;

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
    pub(crate) potion: potion::PotionAssets,
}

pub(crate) fn build_health_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> HealthAssets {
    HealthAssets {
        heart: heart::build_heart_assets(meshes, materials),
        potion: potion::build_potion_assets(meshes, materials),
    }
}

/// Spawns the per-cell health-pickup entity using the rig variant
/// selected by `health_style`.
pub(crate) fn spawn_health_for_cell(
    commands: &mut Commands,
    assets: &HealthAssets,
    health_style: HealthStyle,
    cell: char,
    r: usize,
    c: usize,
) {
    if cell != 'H' {
        return;
    }
    match health_style {
        HealthStyle::Heart => heart::spawn_heart(commands, &assets.heart, r, c),
        HealthStyle::Potion => potion::spawn_potion(commands, &assets.potion, r, c),
    }
}

/// `Update` system that drives the shared idle animation for every
/// `HealthMarker`: a gentle scale pulse layered on top of a slow Y-axis
/// spin. Rigs that need a rig-specific motion can override per-frame
/// transform fields in their own systems — this one only touches scale
/// and rotation.
pub(crate) fn health_animation_system(
    time: Res<Time>,
    mut pickups: Query<&mut Transform, With<HealthMarker>>,
) {
    let scale = 1.0 + (time.elapsed_secs() * PULSE_RATE).sin() * PULSE_AMPLITUDE;
    let yaw = time.elapsed_secs() * SPIN_RATE;
    for mut t in pickups.iter_mut() {
        t.scale = Vec3::splat(scale);
        t.rotation = Quat::from_rotation_y(yaw);
    }
}
