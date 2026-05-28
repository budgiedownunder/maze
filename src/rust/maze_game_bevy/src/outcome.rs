//! Win / lose outcome watcher — a per-frame `Update` system that detects
//! transitions on `MazeGame::is_complete()` / `MazeGame::is_lost()` and
//! spawns the matching overlay + dispatches the `GameResult` exactly once.
//!
//! Splitting this out of `movement_system` matters for outcomes that fire
//! without an in-flight player move: an enemy-tick collision can drop the
//! player to 0 HP while the player is standing still, with no movement
//! animation to gate on. The watcher runs every frame regardless of input
//! and so picks the transition up the moment it happens.
//!
//! Gated on `state.anim.is_none()` so that outcomes triggered mid-move
//! (player walking onto `F`, walking through an open door into a strand,
//! walking into an enemy that drops HP to 0) still surface only after the
//! camera has settled on the destination cell.

use crate::overlays::{lose, win};
use crate::state::{
    dispatch_game_result, GameClock, GameConfig, GameOutcome, GameResult, GameState,
};
use bevy::prelude::*;

pub(crate) fn outcome_watcher_system(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    clock: Res<GameClock>,
    config: Res<GameConfig>,
) {
    if state.anim.is_some() {
        return;
    }
    if !state.won && state.game.is_complete() {
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
        return;
    }
    if !state.lost && state.game.is_lost() {
        state.lost = true;
        let subtitle = match state.game.lose_reason() {
            Some(maze::LoseReason::Killed) => "You died!",
            Some(maze::LoseReason::Stranded) | None => "You're stranded!",
        };
        lose::spawn_lose_overlay(&mut commands, subtitle);
        dispatch_game_result(&GameResult {
            outcome: GameOutcome::Lose,
            elapsed_ms: (clock.elapsed_secs * 1000.0) as u64,
            difficulty: config.difficulty.clone(),
            rows: state.grid.len() as u32,
            cols: state.grid.first().map(|r| r.len()).unwrap_or(0) as u32,
            seed: if config.rows > 0 { Some(config.seed) } else { None },
            extras: std::collections::BTreeMap::new(),
        });
    }
}
