use super::dome::spawn_dome;
use super::procedural::{make_sky_texture, SkySpec};
use super::stars::spawn_stars;
use bevy::prelude::*;
use std::f32::consts::PI;

// ---------- Tuning constants ----------

/// PRNG seed for the night starfield placement. Same seed every run so
/// players replaying the same maze see the same stars.
const SEED: u64 = 0x00C0_FFEE_4242_4242;

/// Number of entity stars spawned across the upper hemisphere.
const STAR_COUNT: u32 = 1000;

/// Ambient light brightness (Bevy units).
const AMBIENT_BRIGHTNESS: f32 = 300.0;
/// Directional ("moon") light illuminance (Bevy units).
const DIRECTIONAL_ILLUMINANCE: f32 = 8_000.0;

/// Gradient palette — linear-space RGB.
const ZENITH: [f32; 3] = [0.02, 0.02, 0.10];
const HORIZON: [f32; 3] = [0.0, 0.0, 0.02];
const NADIR: [f32; 3] = [0.0, 0.0, 0.0];

pub(crate) fn spawn_night(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) {
    commands.spawn(AmbientLight {
        brightness: AMBIENT_BRIGHTNESS,
        ..default()
    });

    commands.spawn((
        DirectionalLight {
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
            // Night has no clouds.
            None,
        )
    });
    let dome = spawn_dome(commands, meshes, materials, sky_tex);
    spawn_stars(commands, dome, meshes, materials, STAR_COUNT, SEED);
}
