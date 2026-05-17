use super::clouds::CloudSpec;
use super::dome::spawn_dome;
use super::procedural::{make_sky_texture, SkySpec};
use super::stars::spawn_stars;
use bevy::prelude::*;
use std::f32::consts::PI;

const SEED: u64 = 0x5455_5455_5455_5455;

pub(crate) fn spawn_sunset(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) {
    // Warm dim orange lighting — the directional "sun" tinted toward
    // the red-gold end of the spectrum, the ambient a less saturated
    // companion so the maze still reads as a coherent space.
    commands.spawn(AmbientLight {
        color: Color::srgb(1.0, 0.75, 0.55),
        brightness: 500.0,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.65, 0.40),
            illuminance: 11_000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
    ));

    let sky_tex = images.as_mut().map(|imgs| {
        make_sky_texture(
            imgs,
            &SkySpec {
                // Faint navy ceiling fading to warm orange at the horizon
                // — the linear lerp produces the red-gold midband the
                // plan calls for.
                zenith: [0.10, 0.05, 0.20],
                horizon: [1.00, 0.45, 0.15],
                nadir: [0.20, 0.10, 0.10],
            },
            Some(&CloudSpec {
                count: 20,
                // Dark grey clouds — backlit sunset clouds typically
                // appear in silhouette against the bright horizon.
                colour: [0.20, 0.15, 0.15],
                seed: SEED,
            }),
        )
    });
    let dome = spawn_dome(commands, meshes, materials, sky_tex);
    spawn_stars(commands, dome, meshes, materials, 200, SEED);
}
