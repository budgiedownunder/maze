use super::{spawn_object, DeadEndAssets};
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Pillar emissive RGB — pale marble grey.
const PILLAR_EMISSIVE: LinearRgba = LinearRgba::new(0.70, 0.70, 0.65, 1.0);
/// Pillar world-Y centre (mid-height of the scaled cylinder).
const PILLAR_Y: f32 = 0.75;
/// Pillar scale `(x, y, z)` — narrow circular column, tall (broken at top).
const PILLAR_SCALE: Vec3 = Vec3::new(0.35, 1.50, 0.35);

pub(crate) fn build_pillar_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: PILLAR_EMISSIVE,
            ..default()
        })
    })
}

pub(crate) fn spawn_pillar(commands: &mut Commands, assets: &DeadEndAssets, x: f32, z: f32) {
    spawn_object(
        commands,
        assets.cylinder.clone(),
        assets.pillar_mat.clone(),
        Vec3::new(x, PILLAR_Y, z),
        PILLAR_SCALE,
    );
}
