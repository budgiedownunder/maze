use super::dome::spawn_dome;
use super::procedural::{make_sky_texture, SkySpec};
use super::stars::spawn_stars;
use bevy::prelude::*;
use std::f32::consts::PI;

// ---------- Tuning constants ----------

/// PRNG seed for the sunrise starfield placement.
const SEED: u64 = 0x5421_5421_5421_5421;

/// Star count — fewer than night (the sky is brightening) but more than
/// sunset (where the day is on the way out).
const STAR_COUNT: u32 = 500;

/// Ambient + directional light preset — soft warm pink. Slightly
/// brighter than sunset because the sun has just crested the horizon
/// and the sky is filling with light.
const AMBIENT_COLOR: Color = Color::srgb(1.0, 0.88, 0.80);
const AMBIENT_BRIGHTNESS: f32 = 600.0;
const DIRECTIONAL_COLOR: Color = Color::srgb(1.0, 0.85, 0.75);
const DIRECTIONAL_ILLUMINANCE: f32 = 12_000.0;

/// Gradient palette — pale blue overhead fading through peach to pale
/// yellow at the horizon.
const ZENITH: [f32; 3] = [0.50, 0.60, 0.70];
const HORIZON: [f32; 3] = [1.00, 0.85, 0.60];
const NADIR: [f32; 3] = [0.35, 0.30, 0.30];

pub(crate) fn spawn_sunrise(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) {
    super::spawn_sky_lights(
        commands,
        AmbientLight {
            color: AMBIENT_COLOR,
            brightness: AMBIENT_BRIGHTNESS,
            ..default()
        },
        DirectionalLight {
            color: DIRECTIONAL_COLOR,
            illuminance: DIRECTIONAL_ILLUMINANCE,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 4.0, PI / 4.0, 0.0)),
    );

    let sky_tex = images.as_mut().map(|imgs| {
        make_sky_texture(
            imgs,
            &SkySpec {
                zenith: ZENITH,
                horizon: HORIZON,
                nadir: NADIR,
            },
            // Sunrise sky is clear — no clouds.
            None,
        )
    });
    let dome = spawn_dome(commands, meshes, materials, sky_tex);
    spawn_stars(commands, dome, meshes, materials, STAR_COUNT, SEED);
}
