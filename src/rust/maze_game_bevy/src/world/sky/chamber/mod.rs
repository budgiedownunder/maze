//! Enclosed built-interior "sky" for [`crate::state::SkyType::Chamber`].
//!
//! Like the dungeon, the chamber has no open sky — [`crate::world::roof`] caps
//! every cell, here in the cell's own wall material. This module supplies the
//! matching backdrop: a near-black, faintly *warm* dome (so the grout gaps
//! between ceiling tiles read as warm dark seams rather than the dungeon's cold
//! ones) and a dim warm ambient + overhead light, for a torch-lit hall feel
//! that's a touch brighter and warmer than the dank dungeon.

use super::dome::spawn_dome;
use super::procedural::{make_sky_texture, SkySpec};
use bevy::prelude::*;
use std::f32::consts::PI;

// ---------- Tuning constants ----------

/// Ambient light brightness (Bevy units). A little above the dungeon's — a
/// built, lived-in hall rather than a dank cave. (The world's emissive-only
/// materials self-illuminate, so this only sets the floor for anything lit.)
const AMBIENT_BRIGHTNESS: f32 = 160.0;
/// Faint overhead light illuminance (Bevy units), angled straight down to
/// suggest a sealed ceiling rather than a sun/moon off the horizon.
const DIRECTIONAL_ILLUMINANCE: f32 = 2_000.0;

/// Gradient palette — linear-space RGB. Near-black but faintly warm: the dome
/// is only ever glimpsed through the grout gaps, so it should read as dim warm
/// dead air.
const ZENITH: [f32; 3] = [0.014, 0.012, 0.010];
const HORIZON: [f32; 3] = [0.008, 0.006, 0.005];
const NADIR: [f32; 3] = [0.0, 0.0, 0.0];

pub(crate) fn spawn_chamber(
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
            // No clouds indoors.
            None,
        )
    });
    // No stars — just the dark dome behind the ceiling.
    spawn_dome(commands, meshes, materials, sky_tex);
}
