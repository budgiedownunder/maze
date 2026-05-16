use super::{spawn_object, DeadEndAssets};
use bevy::prelude::*;

pub(crate) fn build_chest_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.40, 0.25, 0.10, 1.0),
            ..default()
        })
    })
}

pub(crate) fn spawn_chest(commands: &mut Commands, assets: &DeadEndAssets, x: f32, z: f32) {
    // Chest — wide low wooden cuboid.
    spawn_object(
        commands,
        assets.cuboid.clone(),
        assets.chest_mat.clone(),
        Vec3::new(x, 0.25, z),
        Vec3::new(0.80, 0.50, 0.60),
    );
}
