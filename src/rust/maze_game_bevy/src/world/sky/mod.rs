pub(crate) mod chamber;
pub(crate) mod clouds;
pub(crate) mod day;
pub(crate) mod dome;
pub(crate) mod dungeon;
pub(crate) mod night;
pub(crate) mod procedural;
pub(crate) mod stars;
pub(crate) mod sunrise;
pub(crate) mod sunset;

use crate::state::{GameConfig, SkyType};
use bevy::prelude::*;

pub(crate) use dome::sky_dome_follow_camera;

// ---------- shared utilities (visible to every sky submodule) ----------

/// Splitmix-style PRNG returning a `f32` in `[0, 1)`. Used by both the
/// cloud-painter and the star-spawner so seeded sequences come from
/// the same family — same maze + same sky mode → same star + cloud
/// layout across reloads.
fn next_unit(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    // 2^32 = 4_294_967_296 — exactly representable as f32; division
    // gives a result in `[0, 1)`.
    ((*state >> 32) as f32) / 4_294_967_296.0
}

/// Linear-space `[0, 1]` colour component to an sRGB-encoded byte.
/// The dome texture is `Rgba8UnormSrgb`, so writes need to round-trip
/// through this conversion when blending in linear space.
fn linear_to_byte(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Inverse of [`linear_to_byte`].
fn byte_to_linear(b: u8) -> f32 {
    (b as f32) / 255.0
}

/// Spawns the configured sky/atmosphere mode — the procedural dome
/// (gradient + clouds + stars) and the paired ambient + directional
/// light preset. The dispatch is exhaustive on [`SkyType`] so adding
/// a new mode forces a new branch.
pub(crate) fn spawn_sky(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
    config: &GameConfig,
) {
    match config.sky_type {
        SkyType::Night => night::spawn_night(commands, meshes, materials, images),
        SkyType::Sunrise => sunrise::spawn_sunrise(commands, meshes, materials, images),
        SkyType::Day => day::spawn_day(commands, meshes, materials, images),
        SkyType::Sunset => sunset::spawn_sunset(commands, meshes, materials, images),
        SkyType::Dungeon => dungeon::spawn_dungeon(commands, meshes, materials, images),
        SkyType::Chamber => chamber::spawn_chamber(commands, meshes, materials, images),
    }
}
