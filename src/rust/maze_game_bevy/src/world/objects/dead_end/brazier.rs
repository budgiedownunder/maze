use super::{spawn_object, DeadEndAssets};
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Stone column emissive RGB — neutral grey.
const STONE_EMISSIVE: LinearRgba = LinearRgba::new(0.45, 0.45, 0.45, 1.0);
/// Glowing bowl emissive RGB — over-bright orange that picks up the
/// bloom pipeline at distance.
const GLOW_EMISSIVE: LinearRgba = LinearRgba::new(1.4, 0.7, 0.15, 1.0);

/// Column world-Y centre (mesh is a unit cylinder, scaled to
/// [`COLUMN_SCALE`]; centre is half the column's scaled height).
const COLUMN_Y: f32 = 0.40;
/// Column scale `(x, y, z)`. Y is the column height; X and Z are equal
/// (round cylinder cross-section).
const COLUMN_SCALE: Vec3 = Vec3::new(0.40, 0.80, 0.40);

/// Bowl world-Y centre (sits on top of the column).
const BOWL_Y: f32 = 0.85;
/// Bowl scale `(x, y, z)`. Wider than column; short height for a shallow bowl.
const BOWL_SCALE: Vec3 = Vec3::new(0.50, 0.15, 0.50);

pub(crate) fn build_stone_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: STONE_EMISSIVE,
            ..default()
        })
    })
}

pub(crate) fn build_glow_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: GLOW_EMISSIVE,
            ..default()
        })
    })
}

pub(crate) fn spawn_brazier(commands: &mut Commands, assets: &DeadEndAssets, x: f32, z: f32) {
    // Stone column + glowing bowl on top.
    spawn_object(
        commands,
        assets.cylinder.clone(),
        assets.stone_mat.clone(),
        Vec3::new(x, COLUMN_Y, z),
        COLUMN_SCALE,
    );
    spawn_object(
        commands,
        assets.cylinder.clone(),
        assets.glow_mat.clone(),
        Vec3::new(x, BOWL_Y, z),
        BOWL_SCALE,
    );
}
