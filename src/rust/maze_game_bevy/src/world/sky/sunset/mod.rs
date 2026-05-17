use super::clouds::CloudSpec;
use super::dome::spawn_dome;
use super::procedural::{make_sky_texture, SkySpec};
use super::stars::spawn_stars;
use bevy::prelude::*;
use std::f32::consts::PI;

// ---------- Tuning constants ----------

/// PRNG seed for the sunset starfield + cloud placement.
const SEED: u64 = 0x5455_5455_5455_5455;

/// Star count — early evening, only the brightest stars are out.
const STAR_COUNT: u32 = 200;

/// Cloud count for the warm-orange sunset sky.
const CLOUD_COUNT: u32 = 20;
/// Backlit sunset clouds appear in silhouette against the bright
/// horizon — dark grey rather than white.
const CLOUD_COLOUR: [f32; 3] = [0.20, 0.15, 0.15];

/// Ambient + directional light preset — warm dim orange. The
/// directional "sun" tinted toward the red-gold end of the spectrum;
/// ambient is a less saturated companion so the maze still reads as a
/// coherent space.
const AMBIENT_COLOR: Color = Color::srgb(1.0, 0.75, 0.55);
const AMBIENT_BRIGHTNESS: f32 = 500.0;
const DIRECTIONAL_COLOR: Color = Color::srgb(1.0, 0.65, 0.40);
const DIRECTIONAL_ILLUMINANCE: f32 = 11_000.0;

/// Gradient palette — faint navy ceiling fading to warm orange at the
/// horizon. The linear lerp produces the red-gold midband.
const ZENITH: [f32; 3] = [0.10, 0.05, 0.20];
const HORIZON: [f32; 3] = [1.00, 0.45, 0.15];
const NADIR: [f32; 3] = [0.20, 0.10, 0.10];

pub(crate) fn spawn_sunset(
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
    let dome = spawn_dome(commands, meshes, materials, sky_tex);
    spawn_stars(commands, dome, meshes, materials, STAR_COUNT, SEED);
}
