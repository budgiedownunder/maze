use super::clouds::CloudSpec;
use super::dome::spawn_dome;
use super::procedural::{make_sky_texture, SkySpec};
use bevy::prelude::*;
use std::f32::consts::PI;

// ---------- Tuning constants ----------

/// PRNG seed for cloud placement. Day has no stars (no Star spawner).
const SEED: u64 = 0xDA72_DA72_DA72_DA72;

/// Cloud count for the partly-cloudy day sky — moderate broken cover.
const CLOUD_COUNT: u32 = 30;
/// White-grey cloud colour.
const CLOUD_COLOUR: [f32; 3] = [0.98, 0.98, 0.97];

/// Ambient + directional light preset — bright cool-white daylight so
/// corridors actually feel lit.
const AMBIENT_COLOR: Color = Color::srgb(1.0, 0.98, 0.95);
const AMBIENT_BRIGHTNESS: f32 = 1_200.0;
const DIRECTIONAL_COLOR: Color = Color::srgb(1.0, 0.98, 0.95);
const DIRECTIONAL_ILLUMINANCE: f32 = 18_000.0;

/// Gradient palette — classic sky-blue ceiling fading to near-white at
/// the horizon (atmospheric scattering proxy).
const ZENITH: [f32; 3] = [0.35, 0.55, 0.85];
const HORIZON: [f32; 3] = [0.85, 0.92, 0.97];
const NADIR: [f32; 3] = [0.70, 0.78, 0.82];

pub(crate) fn spawn_day(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) {
    commands.spawn(AmbientLight {
        color: AMBIENT_COLOR,
        brightness: AMBIENT_BRIGHTNESS,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
            color: DIRECTIONAL_COLOR,
            illuminance: DIRECTIONAL_ILLUMINANCE,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
    ));

    let sky_tex = images.as_mut().map(|imgs| {
        make_sky_texture(
            imgs,
            &SkySpec {
                zenith: ZENITH,
                horizon: HORIZON,
                nadir: NADIR,
            },
            Some(&CloudSpec {
                count: CLOUD_COUNT,
                colour: CLOUD_COLOUR,
                seed: SEED,
            }),
        )
    });
    // Day has no stars — the dome entity returned by spawn_dome is
    // intentionally discarded.
    let _dome = spawn_dome(commands, meshes, materials, sky_tex);
}
