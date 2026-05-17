use super::dome::spawn_dome;
use super::procedural::{make_sky_texture, SkySpec};
use super::stars::spawn_stars;
use bevy::prelude::*;
use std::f32::consts::PI;

const SEED: u64 = 0x5421_5421_5421_5421;

pub(crate) fn spawn_sunrise(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) {
    // Soft warm pink lighting — pre-dawn / dawn palette. Slightly
    // brighter than sunset because the sun has just crested the
    // horizon and the sky is filling with light.
    commands.spawn(AmbientLight {
        color: Color::srgb(1.0, 0.88, 0.80),
        brightness: 600.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.85, 0.75),
            illuminance: 12_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
    ));

    let sky_tex = images.as_mut().map(|imgs| {
        make_sky_texture(
            imgs,
            &SkySpec {
                // Pale blue overhead fading through peach to pale yellow
                // at the horizon.
                zenith: [0.50, 0.60, 0.70],
                horizon: [1.00, 0.85, 0.60],
                nadir: [0.35, 0.30, 0.30],
            },
            // Sunrise sky is clear — no clouds.
            None,
        )
    });
    let dome = spawn_dome(commands, meshes, materials, sky_tex);
    // More stars than sunset (less sky brightness here) but fewer than
    // night.
    spawn_stars(commands, dome, meshes, materials, 500, SEED);
}
