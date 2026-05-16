use bevy::prelude::*;
use std::f32::consts::PI;

pub(crate) fn spawn_night(commands: &mut Commands) {
    commands.spawn(AmbientLight {
        brightness: 300.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
    ));
}
