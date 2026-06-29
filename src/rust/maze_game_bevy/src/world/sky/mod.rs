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

use crate::state::{GameConfig, GameState, MultiLevelRun, SkyType};
use bevy::prelude::*;

pub(crate) use dome::sky_dome_follow_camera;

/// Marker on every entity the active sky spawns — the dome (its child stars ride
/// along on a recursive despawn) and the ambient + directional lights. Tagging
/// them lets [`sky_switch_on_level_change`] tear the whole sky down and respawn a
/// different one when the player climbs into a level with a different sky.
#[derive(Component)]
pub(crate) struct SkyEntity;

/// Each level's effective sky type (index = level), computed by `spawn_world` with
/// the same rule [`crate::world::level_render_config`] uses for the dome+lighting:
/// the top level takes the `[levels.top]` override, every other level the base
/// sky. The swap watcher reads it to decide when to re-skin the global dome. A
/// single-level run holds one entry (its base sky) and never triggers a swap.
#[derive(Resource)]
pub(crate) struct LevelSkies(pub(crate) Vec<SkyType>);

// ---------- shared utilities (visible to every sky submodule) ----------

/// Spawns the paired ambient + directional light preset for a sky mode, tagging
/// both [`SkyEntity`] so a sky swap despawns them with the dome. Every submodule
/// routes its two lights through here so the tag can't be forgotten.
pub(crate) fn spawn_sky_lights(
    commands: &mut Commands,
    ambient: AmbientLight,
    directional: DirectionalLight,
    directional_xform: Transform,
) {
    commands.spawn((SkyEntity, ambient));
    commands.spawn((SkyEntity, directional, directional_xform));
}

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

/// Re-skins the single global dome + lighting when the player crosses into a level
/// whose effective sky differs from the one currently shown — so a run can rise
/// from sealed dungeon floors out to an open-sky summit. The swap is timed to the
/// **occluded moment** of the climb / step-through (see
/// [`crate::transition::sky_swap_due`]): a ladder swaps as the camera clears the
/// hatch hole onto the upper level, a portal at the white-out flash peak — so the
/// player keeps the level-below sky for the visible part of the transition and
/// emerges already in the new one, rather than watching it change mid-climb or
/// snap on arrival. Before that point (and with no transition active) the
/// displayed level is simply `current_level`. Shaped like
/// [`crate::world::floor::hatch::hatch_close_watcher`]: it fires once per level
/// change (`Local` last level), and is inert for a single-level run (the index
/// never moves) and for an unchanged sky (no churn). Only one sky exists at a
/// time: the despawn drops the old dome's material + sky texture so Bevy reclaims
/// them on the next asset sweep, before the new sky's are added.
#[allow(clippy::too_many_arguments)]
pub(crate) fn sky_switch_on_level_change(
    mut commands: Commands,
    run: Res<MultiLevelRun>,
    state: Res<GameState>,
    skies: Res<LevelSkies>,
    config: Res<GameConfig>,
    mut last_level: Local<usize>,
    sky_entities: Query<Entity, With<SkyEntity>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
    mut images: Option<ResMut<Assets<Image>>>,
) {
    // Once the climb/step-through reaches its masked swap point the player is
    // committed to the next level; show its sky then. Before that (and with no
    // transition) the displayed level is the one being played, so the level-below
    // sky holds for the visible part of the transition.
    let crossing = state
        .transition
        .as_ref()
        .is_some_and(crate::transition::sky_swap_due);
    let displayed_level = if crossing {
        run.current_level + 1
    } else {
        run.current_level
    };
    if displayed_level == *last_level {
        return;
    }
    let prev = *last_level;
    // The previously-shown level's sky is the one currently spawned; compare it to
    // the level being entered. A transition always targets an in-range level (you
    // never transition off the final level), but guard rather than panic.
    let (Some(&new_sky), Some(&old_sky)) =
        (skies.0.get(displayed_level), skies.0.get(prev))
    else {
        return;
    };
    *last_level = displayed_level;
    if new_sky == old_sky {
        return;
    }
    // Despawn the whole sky set. The dome despawns recursively, so its child stars
    // go with it — dropping the dome material + sky texture and the star
    // mesh/material handles, so the asset store stays flat across the swap.
    for entity in &sky_entities {
        commands.entity(entity).despawn();
    }
    // `spawn_sky` reads only `sky_type`; clone the live config with the new sky so
    // the rest (clouds/star seeds etc. live in the submodules) is untouched.
    let sky_config = GameConfig {
        sky_type: new_sky,
        ..config.clone()
    };
    spawn_sky(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut images,
        &sky_config,
    );
}
