use super::{spawn_object, DeadEndAssets};
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Chest emissive RGB — dark wood brown.
const CHEST_EMISSIVE: LinearRgba = LinearRgba::new(0.40, 0.25, 0.10, 1.0);
/// Chest world-Y centre (mid-height of the scaled cuboid).
const CHEST_Y: f32 = 0.25;
/// Chest scale `(x, y, z)` — wide low rectangular box.
const CHEST_SCALE: Vec3 = Vec3::new(0.80, 0.50, 0.60);

pub(crate) fn build_chest_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: CHEST_EMISSIVE,
            ..default()
        })
    })
}

pub(crate) fn spawn_chest(commands: &mut Commands, assets: &DeadEndAssets, x: f32, z: f32) {
    spawn_object(
        commands,
        assets.cuboid.clone(),
        assets.chest_mat.clone(),
        Vec3::new(x, CHEST_Y, z),
        CHEST_SCALE,
    );
}
