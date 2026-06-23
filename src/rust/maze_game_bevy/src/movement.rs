use crate::state::{Animation, GameState};
use crate::world::{camera_pos_for, explore_cell_raw};
use bevy::prelude::*;
use maze::MoveResult;
use std::f32::consts::PI;

const TURN_DUR: f32 = 0.12;
const MOVE_DUR: f32 = 0.18;
const PITCH_RATE: f32 = PI / 3.0; // rad/s — reaches ±30° in 0.5 s
const MAX_PITCH_DOWN: f32 = PI / 2.0; // 90° — straight down
const MAX_PITCH_UP: f32 = PI * 45.0 / 180.0; // 45° — half-way up

pub(crate) fn movement_system(
    time: Res<Time>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<GameState>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    // While paused, freeze movement, animation, and pitch entirely — the
    // camera keeps its current transform.
    if state.paused {
        return;
    }
    let dt = time.delta_secs();

    // Advance active animation; snap to target when complete
    let anim_done = if let Some(ref mut anim) = state.anim {
        anim.elapsed += dt;
        anim.elapsed >= anim.duration
    } else {
        false
    };
    if anim_done {
        let anim = state.anim.take().unwrap();
        state.visual_pos = anim.target_pos;
        state.visual_yaw = anim.target_yaw;
        // Win / lose detection lives in `crate::outcome::outcome_watcher_system`
        // — it runs every frame (not just on anim completion) so an
        // enemy-tick kill of a stationary player still surfaces the death
        // overlay immediately. The watcher gates on `state.anim.is_none()`
        // so move-triggered outcomes still wait for the camera to settle
        // on the destination cell.
    } else if state.anim.is_some() {
        let pos = state.anim.as_ref().unwrap().current_pos();
        let yaw = state.anim.as_ref().unwrap().current_yaw();
        state.visual_pos = pos;
        state.visual_yaw = yaw;
    }

    // Pitch — active even during movement animation, disabled after win or loss
    if !state.won && !state.lost {
        if let Some(ref keys) = keys {
            if keys.pressed(KeyCode::KeyQ) {
                state.visual_pitch = (state.visual_pitch + PITCH_RATE * dt).min(MAX_PITCH_UP);
            }
            if keys.pressed(KeyCode::KeyE) {
                state.visual_pitch = (state.visual_pitch - PITCH_RATE * dt).max(-MAX_PITCH_DOWN);
            }
        }
    }

    // Movement — only when idle and the game is still in play
    if state.anim.is_none() && !state.won && !state.lost {
        let Some(keys) = keys else { return; };
        let left = keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA);
        let right = keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD);
        let forward = keys.pressed(KeyCode::ArrowUp) || keys.pressed(KeyCode::KeyW);

        // Turns interpolate both yaw AND position so the camera orbits to
        // the back-edge of the cell relative to the NEW facing. Without
        // the position interpolation, the camera would only sit
        // off-centre after a forward move — a left/right turn would leave
        // the player visibly to one side of the cell rather than at its
        // back wall.
        if left {
            state.facing = state.facing.turn_left();
            let (start_yaw, start_pos) = (state.visual_yaw, state.visual_pos);
            let target_yaw = start_yaw + PI / 2.0;
            let (row, col) = (state.game.player_row(), state.game.player_col());
            state.anim = Some(Animation {
                start_pos,
                target_pos: camera_pos_for(row, col, target_yaw),
                start_yaw,
                target_yaw,
                elapsed: 0.0,
                duration: TURN_DUR,
            });
        } else if right {
            state.facing = state.facing.turn_right();
            let (start_yaw, start_pos) = (state.visual_yaw, state.visual_pos);
            let target_yaw = start_yaw - PI / 2.0;
            let (row, col) = (state.game.player_row(), state.game.player_col());
            state.anim = Some(Animation {
                start_pos,
                target_pos: camera_pos_for(row, col, target_yaw),
                start_yaw,
                target_yaw,
                elapsed: 0.0,
                duration: TURN_DUR,
            });
        } else if forward {
            let dir = state.facing.to_direction();
            match state.game.move_player(dir) {
                // `Stranded` reports a successful step onto an open door cell
                // that has just left the player too short of keys to finish —
                // the move itself succeeded, so the camera follows; the lose
                // surface (HUD message) is wired separately via
                // `game.is_lost()` / `game.lose_reason()`.
                MoveResult::Moved | MoveResult::Complete | MoveResult::Stranded => {
                    let (row, col) = (state.game.player_row(), state.game.player_col());
                    let nrows = state.grid.len();
                    let ncols = state.grid[0].len();
                    explore_cell_raw(&mut state.explored, nrows, ncols, row, col);
                    let (start_pos, start_yaw) = (state.visual_pos, state.visual_yaw);
                    // Forward moves don't change facing, so the target camera
                    // position uses the same yaw as the start.
                    let target_pos = camera_pos_for(row, col, start_yaw);
                    state.anim = Some(Animation {
                        start_pos,
                        target_pos,
                        start_yaw,
                        target_yaw: start_yaw,
                        elapsed: 0.0,
                        duration: MOVE_DUR,
                    });
                }
                // Held against a locked door with a key in the bag: the key is
                // consumed and the door begins opening (advanced by
                // `door_tick_system`). The player does not move — holding
                // forward simply waits out the open countdown, after which the
                // door reports `Open` and the next press moves through it.
                MoveResult::StartedUnlocking => {}
                // Locked door with no key, or a door still opening: no move.
                MoveResult::BlockedByLockedDoor => {}
                // The player's HP dropped to 0 from this move (either the
                // destination cell held an enemy, or — if the player was
                // already dying from a tick collision — the short-circuit
                // path returned Killed without processing). The death
                // overlay is spawned by the post-move `is_lost()` check
                // below, which reads `lose_reason()` and chooses the right
                // subtitle. No camera animation here — the player is dead.
                MoveResult::Killed => {}
                // Wall / boundary, or `Direction::None`: no move.
                MoveResult::Blocked | MoveResult::None => {}
            }
        }
    }

    // Update camera transform every frame. The move animation runs in the level's
    // local (ground) frame; lift + centre it onto the level currently being played
    // (zero on the bottom level, so single-level games are unchanged).
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation = state.visual_pos + state.camera_offset;
        transform.rotation =
            Quat::from_rotation_y(state.visual_yaw) * Quat::from_rotation_x(state.visual_pitch);
    }
}

/// Esc quits the native desktop app.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn quit_system(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut exit: bevy::ecs::message::MessageWriter<AppExit>,
) {
    if let Some(keys) = keys {
        if keys.just_pressed(KeyCode::Escape) {
            exit.write(AppExit::Success);
        }
    }
}

/// In the browser there is no application to quit, and writing `AppExit` would
/// halt the Bevy loop and freeze the game (the timer stops with no way to
/// resume). So Esc does not quit on wasm — it is handled as a pause toggle by
/// [`crate::overlays::pause::pause_system`] instead, leaving this system a no-op.
#[cfg(target_arch = "wasm32")]
pub(crate) fn quit_system() {}
