use super::{spawn_object, DeadEndAssets};
use bevy::prelude::*;

pub(crate) fn build_stone_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.45, 0.45, 0.45, 1.0),
            ..default()
        })
    })
}

pub(crate) fn build_glow_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> Option<Handle<StandardMaterial>> {
    // Over-bright orange that picks up the bloom pipeline at distance.
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(1.4, 0.7, 0.15, 1.0),
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
        Vec3::new(x, 0.40, z),
        Vec3::new(0.40, 0.80, 0.40),
    );
    spawn_object(
        commands,
        assets.cylinder.clone(),
        assets.glow_mat.clone(),
        Vec3::new(x, 0.85, z),
        Vec3::new(0.50, 0.15, 0.50),
    );
}
