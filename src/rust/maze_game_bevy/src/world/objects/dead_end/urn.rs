use super::{spawn_object, DeadEndAssets};
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Urn emissive RGB — terracotta brown.
const URN_EMISSIVE: LinearRgba = LinearRgba::new(0.55, 0.30, 0.15, 1.0);
/// Urn world-Y centre (squat cylinder, centre is half its scaled height).
const URN_Y: f32 = 0.35;
/// Urn scale `(x, y, z)` — moderately wide, short for a squat profile.
const URN_SCALE: Vec3 = Vec3::new(0.50, 0.70, 0.50);

pub(crate) fn build_urn_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: URN_EMISSIVE,
            ..default()
        })
    })
}

pub(crate) fn spawn_urn(commands: &mut Commands, assets: &DeadEndAssets, x: f32, z: f32) {
    spawn_object(
        commands,
        assets.cylinder.clone(),
        assets.urn_mat.clone(),
        Vec3::new(x, URN_Y, z),
        URN_SCALE,
    );
}
