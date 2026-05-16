use super::{spawn_object, DeadEndAssets};
use bevy::prelude::*;

pub(crate) fn build_pillar_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.70, 0.70, 0.65, 1.0),
            ..default()
        })
    })
}

pub(crate) fn spawn_pillar(commands: &mut Commands, assets: &DeadEndAssets, x: f32, z: f32) {
    // Broken pillar — tall narrow marble column.
    spawn_object(
        commands,
        assets.cylinder.clone(),
        assets.pillar_mat.clone(),
        Vec3::new(x, 0.75, z),
        Vec3::new(0.35, 1.50, 0.35),
    );
}
