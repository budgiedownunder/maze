//! Enclosed-dungeon "sky".
//!
//! Unlike the open-air modes (night / sunrise / day / sunset), the dungeon
//! has no visible sky: [`super::super::roof`] caps every passable cell with a
//! dark-rock ceiling, so the player is sealed underground. This module supplies
//! the matching backdrop and lighting: a near-black dome (so any sliver between
//! ceiling panels reads as void rather than open sky) and a dim ambient + faint
//! overhead light preset for an oppressive, torch-lit feel.

use super::dome::spawn_dome;
use super::procedural::{make_sky_texture, SkySpec};
use bevy::prelude::*;
use std::f32::consts::PI;

// ---------- Tuning constants ----------

/// Ambient light brightness (Bevy units). Far dimmer than the open-air
/// modes — the world's emissive-only materials still self-illuminate, so
/// this only sets the floor for anything genuinely lit.
const AMBIENT_BRIGHTNESS: f32 = 120.0;
/// Faint overhead light illuminance (Bevy units), angled straight down to
/// suggest a sealed ceiling rather than a sun/moon off the horizon.
const DIRECTIONAL_ILLUMINANCE: f32 = 1_500.0;

/// Gradient palette — linear-space RGB. Near-black throughout: the dome is
/// only ever glimpsed through gaps, so it should read as cold dead air.
const ZENITH: [f32; 3] = [0.01, 0.01, 0.012];
const HORIZON: [f32; 3] = [0.005, 0.005, 0.006];
const NADIR: [f32; 3] = [0.0, 0.0, 0.0];

pub(crate) fn spawn_dungeon(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) {
    super::spawn_sky_lights(
        commands,
        AmbientLight {
            brightness: AMBIENT_BRIGHTNESS,
            ..default()
        },
        DirectionalLight {
            illuminance: DIRECTIONAL_ILLUMINANCE,
            shadows_enabled: false,
            ..default()
        },
        // Straight down — light from the ceiling, not the horizon.
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -PI / 2.0, 0.0, 0.0)),
    );

    let sky_tex = images.as_mut().map(|imgs| {
        make_sky_texture(
            imgs,
            &SkySpec {
                zenith: ZENITH,
                horizon: HORIZON,
                nadir: NADIR,
            },
            // No clouds underground.
            None,
        )
    });
    // No stars — just the dark dome behind the rock ceiling.
    spawn_dome(commands, meshes, materials, sky_tex);
}
