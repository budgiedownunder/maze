use super::clouds::CloudSpec;
use super::dome::spawn_dome;
use super::procedural::{make_sky_texture, SkySpec};
use bevy::prelude::*;
use std::f32::consts::PI;

pub(crate) fn spawn_day(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) {
    // Bright cool-white daylight — corridors actually feel lit.
    commands.spawn(AmbientLight {
        color: Color::srgb(1.0, 0.98, 0.95),
        brightness: 1_200.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.98, 0.95),
            illuminance: 18_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
    ));

    let sky_tex = images.as_mut().map(|imgs| {
        make_sky_texture(
            imgs,
            &SkySpec {
                // Classic sky-blue ceiling fading to near-white at the
                // horizon (atmospheric scattering proxy).
                zenith: [0.35, 0.55, 0.85],
                horizon: [0.85, 0.92, 0.97],
                nadir: [0.70, 0.78, 0.82],
            },
            Some(&CloudSpec {
                // Broken cloud cover — moderate count gives the sky a
                // partly-cloudy appearance without saturating the dome.
                count: 30,
                colour: [0.98, 0.98, 0.97],
                seed: 0xDA72_DA72_DA72_DA72,
            }),
        )
    });
    // Day has no stars — the dome entity returned by spawn_dome is
    // intentionally discarded.
    let _dome = spawn_dome(commands, meshes, materials, sky_tex);
}
