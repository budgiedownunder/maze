use crate::overlays::win;
use crate::state::{
    dispatch_game_result, Animation, GameClock, GameConfig, GameOutcome, GameResult, GameState,
};
use crate::world::{cell_centre, explore_cell_raw};
use bevy::prelude::*;
use maze::MoveResult;
use std::f32::consts::PI;

const TURN_DUR: f32 = 0.12;
const MOVE_DUR: f32 = 0.18;
const PITCH_RATE: f32 = PI / 3.0; // rad/s — reaches ±30° in 0.5 s
const MAX_PITCH_DOWN: f32 = PI / 2.0; // 90° — straight down
const MAX_PITCH_UP: f32 = PI * 45.0 / 180.0; // 45° — half-way up

pub(crate) fn movement_system(
    mut commands: Commands,
    time: Res<Time>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<GameState>,
    clock: Res<GameClock>,
    config: Res<GameConfig>,
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

        // Check whether the player just arrived at the finish cell
        let (r, c) = (state.game.player_row(), state.game.player_col());
        if !state.won && state.grid[r][c] == 'F' {
            state.won = true;
            win::spawn_win_overlay(&mut commands);
            dispatch_game_result(&GameResult {
                outcome: GameOutcome::Win,
                elapsed_ms: (clock.elapsed_secs * 1000.0) as u64,
                difficulty: config.difficulty.clone(),
                rows: state.grid.len() as u32,
                cols: state.grid.first().map(|r| r.len()).unwrap_or(0) as u32,
                seed: if config.rows > 0 { Some(config.seed) } else { None },
                extras: std::collections::BTreeMap::new(),
            });
        }
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

        if left {
            state.facing = state.facing.turn_left();
            let (start_yaw, start_pos) = (state.visual_yaw, state.visual_pos);
            state.anim = Some(Animation {
                start_pos,
                target_pos: start_pos,
                start_yaw,
                target_yaw: start_yaw + PI / 2.0,
                elapsed: 0.0,
                duration: TURN_DUR,
            });
        } else if right {
            state.facing = state.facing.turn_right();
            let (start_yaw, start_pos) = (state.visual_yaw, state.visual_pos);
            state.anim = Some(Animation {
                start_pos,
                target_pos: start_pos,
                start_yaw,
                target_yaw: start_yaw - PI / 2.0,
                elapsed: 0.0,
                duration: TURN_DUR,
            });
        } else if forward {
            let dir = state.facing.to_direction();
            let result = state.game.move_player(dir);
            if matches!(result, MoveResult::Moved | MoveResult::Complete) {
                let (row, col) = (state.game.player_row(), state.game.player_col());
                let nrows = state.grid.len();
                let ncols = state.grid[0].len();
                explore_cell_raw(&mut state.explored, nrows, ncols, row, col);
                let target_pos = cell_centre(row, col);
                let (start_pos, start_yaw) = (state.visual_pos, state.visual_yaw);
                state.anim = Some(Animation {
                    start_pos,
                    target_pos,
                    start_yaw,
                    target_yaw: start_yaw,
                    elapsed: 0.0,
                    duration: MOVE_DUR,
                });
            }
        }
    }

    // Update camera transform every frame
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation = state.visual_pos;
        transform.rotation =
            Quat::from_rotation_y(state.visual_yaw) * Quat::from_rotation_x(state.visual_pitch);
    }
}

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
