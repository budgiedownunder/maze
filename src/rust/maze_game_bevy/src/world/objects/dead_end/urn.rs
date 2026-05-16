use super::{spawn_object, DeadEndAssets};
use bevy::prelude::*;

pub(crate) fn build_urn_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.55, 0.30, 0.15, 1.0),
            ..default()
        })
    })
}

pub(crate) fn spawn_urn(commands: &mut Commands, assets: &DeadEndAssets, x: f32, z: f32) {
    // Squat terracotta cylinder.
    spawn_object(
        commands,
        assets.cylinder.clone(),
        assets.urn_mat.clone(),
        Vec3::new(x, 0.35, z),
        Vec3::new(0.50, 0.70, 0.50),
    );
}
