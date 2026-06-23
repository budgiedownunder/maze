//! Level-to-level transitions for multi-level runs.
//!
//! Reaching an **interim** finish no longer snaps straight to the next level —
//! it starts a [`LevelTransition`] whose style matches the finish rig:
//!
//! - **Ladder** — the camera climbs from the finish cell up to the next level's
//!   start (directly above), reading as walking up the rungs, and settles facing
//!   the start cell's default direction (as at game start).
//! - **Portal** — a screen flash + rings rippling out from the portal; the camera
//!   jumps to the next cell behind the white-out (which also covers a `centre`-
//!   aligned start that isn't directly above).
//!
//! For the whole transition the clock and player movement are frozen
//! ([`crate::hud::clock::tick_clock_system`] / [`crate::movement::movement_system`]
//! both bail on `state.transition.is_some()`); on completion the run swaps to the
//! next level via [`crate::world::advance_to_next_level`].

use crate::state::{FinishType, GameConfig, GameState, LevelTransition, MultiLevelRun};
use crate::world::{advance_to_next_level, camera_pos_for, cell_world_xz, initial_facing, LevelPlacement};
use bevy::prelude::*;
use maze::MazeGame;
use std::f32::consts::PI;

/// Duration of a ladder climb (seconds) — slow enough to read as a climb.
const LADDER_DUR: f32 = 1.6;
/// Duration of a portal step-through (seconds).
const PORTAL_DUR: f32 = 1.0;
/// Upward pitch the ladder camera holds while climbing (radians) — looks up the
/// rungs, easing back to level after the turn.
const LADDER_UP_PITCH: f32 = 25.0 * PI / 180.0;
/// Fraction of the ladder transition spent rising (facing fixed), then pausing at
/// the top; the remainder turns to the start cell's default facing. Splitting the
/// climb and the turn keeps the camera from twisting *while* climbing.
const CLIMB_FRAC: f32 = 0.55;
const PAUSE_FRAC: f32 = 0.15;

/// Begins the transition off the just-completed interim finish. Resolves the rig
/// kind (the same `finish_type` + ladder-validity logic the finish drew with) and
/// the next level's start pose, then arms `state.transition`. The level swap
/// itself is deferred to [`transition_system`] on completion.
pub(crate) fn start_level_transition(
    state: &mut GameState,
    run: &MultiLevelRun,
    config: &GameConfig,
) {
    let current = run.current_level;
    let (fr, fc) = (state.game.player_row(), state.game.player_col());
    let cur_placement = level_placement(run, config, current, &state.grid);
    let finish_xz = cell_world_xz(fr, fc, cur_placement);

    // Parse the next level for its start cell + pose.
    let next_index = current + 1;
    let next_game = MazeGame::from_json(&run.levels[next_index])
        .expect("multi-level run holds maze JSON produced by the generator");
    let next_grid = next_game.grid().to_vec();
    let (sr, sc) = (next_game.player_row(), next_game.player_col());
    let next_placement = level_placement(run, config, next_index, &next_grid);
    let start_xz = cell_world_xz(sr, sc, next_placement);
    // Arrive facing the start cell's default direction (as at game start) — the
    // same pose `advance_to_next_level` will settle on.
    let target_yaw = initial_facing(&next_grid, sr, sc).to_yaw();
    let target_pos = camera_pos_for(sr, sc, target_yaw) + next_placement.camera_offset();

    // A ladder needs the next start directly above the finish; otherwise (e.g.
    // `centre` alignment offsets it) the rig — and so the transition — is a portal.
    let ladder_allowed = finish_xz.distance_squared(start_xz) < 1e-3;
    let mut kind = config.finish_type.concrete_for_cell(fr, fc, config.seed);
    if kind == FinishType::Ladder && !ladder_allowed {
        kind = FinishType::Portal;
    }

    let start_pos = state.visual_pos + state.camera_offset;
    let duration = match kind {
        FinishType::Portal => PORTAL_DUR,
        _ => LADDER_DUR,
    };
    // Keep the player's entry facing for the start of the transition; the ladder
    // holds it fixed while climbing (so the view doesn't twist mid-climb), the
    // portal keeps it through the flash. The turn to the start-cell default
    // happens at the very end (ladder) or behind the flash (portal).
    let (start_yaw, start_pitch) = (state.visual_yaw, state.visual_pitch);

    state.transition = Some(LevelTransition {
        kind,
        elapsed: 0.0,
        duration,
        start_pos,
        target_pos,
        start_yaw,
        target_yaw,
        start_pitch,
        target_pitch: 0.0,
    });
}

/// `LevelPlacement` for `level`, sized from `grid` against the run's base footprint.
fn level_placement(
    run: &MultiLevelRun,
    config: &GameConfig,
    level: usize,
    grid: &[Vec<char>],
) -> LevelPlacement {
    LevelPlacement::for_level(
        level,
        grid.len(),
        grid.first().map_or(0, |row| row.len()),
        run.base_dims.0,
        run.base_dims.1,
        config.layered_alignment,
    )
}

/// `Update`: drives an active transition's camera each frame and, on completion,
/// swaps the run to the next level. A no-op when no transition is active (so
/// single-level games never touch it).
pub(crate) fn transition_system(
    time: Res<Time>,
    mut state: ResMut<GameState>,
    mut run: ResMut<MultiLevelRun>,
    config: Res<GameConfig>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let dt = time.delta_secs();
    let Some(transition) = state.transition.as_mut() else {
        return;
    };
    transition.elapsed += dt;
    let done = transition.is_complete();
    let (pos, yaw, pitch) = camera_pose(transition);
    if let Ok(mut tf) = camera.single_mut() {
        tf.translation = pos;
        tf.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);
    }
    if done {
        state.transition = None;
        advance_to_next_level(state.as_mut(), run.as_mut(), &config);
    }
}

/// The camera world pose for this frame of `transition`.
fn camera_pose(t: &LevelTransition) -> (Vec3, f32, f32) {
    match t.kind {
        // Portal: hold the start pose until the flash whites out at the halfway
        // point, then the next pose — the jump is hidden behind the flash.
        FinishType::Portal => {
            if t.elapsed < t.duration * 0.5 {
                (t.start_pos, t.start_yaw, t.start_pitch)
            } else {
                (t.target_pos, t.target_yaw, t.target_pitch)
            }
        }
        // Ladder: climb facing one way, pause at the top, then turn to the start
        // cell's default facing — so the view never twists while climbing.
        _ => ladder_pose(t),
    }
}

/// The ladder transition's three phases: rise (entry facing fixed, tilting up to
/// look up the rungs), a brief pause at the top, then a turn to the start cell's
/// default facing while levelling the pitch.
fn ladder_pose(t: &LevelTransition) -> (Vec3, f32, f32) {
    let raw = (t.elapsed / t.duration).clamp(0.0, 1.0);
    if raw < CLIMB_FRAC {
        let p = smoothstep(raw / CLIMB_FRAC);
        let pos = t.start_pos.lerp(t.target_pos, p);
        let pitch = t.start_pitch + (LADDER_UP_PITCH - t.start_pitch) * p;
        (pos, t.start_yaw, pitch)
    } else if raw < CLIMB_FRAC + PAUSE_FRAC {
        (t.target_pos, t.start_yaw, LADDER_UP_PITCH)
    } else {
        let p = smoothstep((raw - CLIMB_FRAC - PAUSE_FRAC) / (1.0 - CLIMB_FRAC - PAUSE_FRAC));
        let yaw = lerp_angle(t.start_yaw, t.target_yaw, p);
        let pitch = LADDER_UP_PITCH + (t.target_pitch - LADDER_UP_PITCH) * p;
        (t.target_pos, yaw, pitch)
    }
}

/// Smoothstep easing (0→1).
fn smoothstep(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

/// Lerp between two angles along the shortest arc, so a near-180° climb-to-default
/// turn doesn't spin the long way round.
fn lerp_angle(a: f32, b: f32, t: f32) -> f32 {
    let mut delta = (b - a) % (2.0 * PI);
    if delta > PI {
        delta -= 2.0 * PI;
    } else if delta < -PI {
        delta += 2.0 * PI;
    }
    a + delta * t
}

/// Peak opacity of the portal white-out flash.
const FLASH_PEAK_ALPHA: f32 = 0.9;
/// Portal flash colour — white.
const FLASH_COLOR: Color = Color::WHITE;

/// Full-screen white-out sprite for a portal transition.
#[derive(Component)]
pub(crate) struct TransitionFlash;

/// `Update`: drives the portal transition's screen flash — a white-out that ramps
/// to its peak at the halfway point (covering the camera jump) and fades back to
/// clear. Present only during a portal transition; despawned otherwise. Ladders
/// (a visible climb) get no flash.
pub(crate) fn transition_fx_system(
    mut commands: Commands,
    window: Query<&Window>,
    state: Res<GameState>,
    mut flashes: Query<(Entity, &mut Sprite), With<TransitionFlash>>,
) {
    let alpha = match state.transition.as_ref() {
        Some(t) if t.kind == FinishType::Portal => {
            // 0 at the ends, peak at the midpoint (= the camera jump).
            FLASH_PEAK_ALPHA * (t.progress() * PI).sin().max(0.0)
        }
        _ => {
            for (entity, _) in flashes.iter() {
                commands.entity(entity).despawn();
            }
            return;
        }
    };
    let Ok(win) = window.single() else {
        return;
    };
    let size = Vec2::new(win.width(), win.height());
    if let Some((_, mut sprite)) = flashes.iter_mut().next() {
        sprite.color = FLASH_COLOR.with_alpha(alpha);
        sprite.custom_size = Some(size);
    } else {
        commands.spawn((
            TransitionFlash,
            Sprite {
                color: FLASH_COLOR.with_alpha(alpha),
                custom_size: Some(size),
                ..default()
            },
            // Above the world, below the HUD (z 9) so readouts stay legible.
            Transform::from_xyz(0.0, 0.0, 8.5),
        ));
    }
}
