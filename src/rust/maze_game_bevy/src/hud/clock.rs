use crate::overlays::lose;
use crate::palette::{CLOCK_GOLD, CLOCK_RED};
use crate::state::{dispatch_game_result, GameClock, GameConfig, GameOutcome, GameResult, GameState};
use bevy::prelude::*;

const COLOR_CLOCK_FLASH: Color = Color::srgba(1.0, 0.85, 0.0, 0.6);

const CLOCK_WARN_SECS: f32 = 30.0;
const CLOCK_FLASH_SECS: f32 = 15.0;
const CLOCK_FLASH_HZ: f32 = 1.5;

#[derive(Component)]
pub(crate) struct ClockText;

#[derive(Component)]
pub(crate) struct ClockBackground;

pub(crate) fn spawn_clock_hud(commands: &mut Commands, window: &Query<&Window>) {
    // Countdown text — top-centre HUD. Initial position from the current window;
    // clock_text_system / clock_flash_system reposition each frame so resizes track.
    let clock_y = window
        .single()
        .map(|w| w.height() / 2.0 - 32.0)
        .unwrap_or(330.0);
    // Flashing background sits behind the text. Alpha is driven each frame by
    // clock_flash_system; starts at the COLOR_CLOCK_FLASH peak and gets clamped to 0
    // on the first frame because remaining_secs is well above CLOCK_FLASH_SECS.
    commands.spawn((
        ClockBackground,
        Sprite {
            color: COLOR_CLOCK_FLASH,
            custom_size: Some(Vec2::new(140.0, 52.0)),
            ..default()
        },
        Transform::from_xyz(0.0, clock_y, 8.9),
    ));
    commands.spawn((
        ClockText,
        Text2d::new("--:--"),
        TextFont { font_size: 36.0, ..default() },
        TextColor(CLOCK_GOLD),
        Transform::from_xyz(0.0, clock_y, 9.0),
    ));
}

pub(crate) fn tick_clock_system(
    mut commands: Commands,
    time: Res<Time>,
    mut clock: ResMut<GameClock>,
    mut state: ResMut<GameState>,
    config: Res<GameConfig>,
) {
    if state.won || state.lost || state.paused {
        return;
    }
    let dt = time.delta_secs();
    clock.elapsed_secs += dt;
    clock.remaining_secs -= dt;
    if clock.remaining_secs > 0.0 {
        return;
    }
    clock.remaining_secs = 0.0;
    state.lost = true;

    // Spawn lose UI — mirrors the win-UI spawn in movement_system. Colours swapped for a
    // muted red theme so it reads as loss without resorting to skulls or anything morbid.
    // The Bevy clock owns the timeout end-to-end (the inner `MazeGame` doesn't model it);
    // the subtitle is hardcoded here rather than read from `MazeGame::lose_reason()`.
    lose::spawn_lose_overlay(&mut commands, "Time's up");
    dispatch_game_result(&GameResult {
        outcome: GameOutcome::Lose,
        elapsed_ms: (clock.elapsed_secs * 1000.0) as u64,
        score: state.game.score(),
        difficulty: config.difficulty.clone(),
        rows: state.grid.len() as u32,
        cols: state.grid.first().map(|r| r.len()).unwrap_or(0) as u32,
        seed: if config.rows > 0 { Some(config.seed) } else { None },
        extras: std::collections::BTreeMap::new(),
    });
}

pub(crate) fn clock_text_system(
    window: Query<&Window>,
    mut clock: ResMut<GameClock>,
    mut texts: Query<(&mut Text2d, &mut Transform, &mut TextColor), With<ClockText>>,
) {
    // Anchor the HUD to the top edge each frame so window resizes track.
    let half_h = window.single().map(|w| w.height() / 2.0).unwrap_or(360.0);
    let target_y = half_h - 32.0;

    // Throttle text content updates to once per whole-second change.
    let remaining = clock.remaining_secs.max(0.0).ceil() as i32;
    let text_changed = remaining != clock.last_displayed_secs;
    if text_changed {
        clock.last_displayed_secs = remaining;
    }
    let mins = remaining / 60;
    let secs = remaining % 60;
    let warn = clock.remaining_secs <= CLOCK_WARN_SECS;
    for (mut text, mut transform, mut colour) in &mut texts {
        if text_changed {
            text.0 = format!("{:02}:{:02}", mins, secs);
            colour.0 = if warn { CLOCK_RED } else { CLOCK_GOLD };
        }
        transform.translation.y = target_y;
    }
}

pub(crate) fn clock_flash_system(
    time: Res<Time>,
    clock: Res<GameClock>,
    state: Res<GameState>,
    window: Query<&Window>,
    mut backgrounds: Query<(&mut Sprite, &mut Transform), With<ClockBackground>>,
) {
    let half_h = window.single().map(|w| w.height() / 2.0).unwrap_or(360.0);
    let target_y = half_h - 32.0;

    // Hide entirely outside the flash window or once the game ends — the win / lose
    // overlay should own the screen at that point.
    let pulse_alpha = if state.won || state.lost || clock.remaining_secs > CLOCK_FLASH_SECS {
        0.0
    } else {
        let peak = COLOR_CLOCK_FLASH.alpha();
        let phase = time.elapsed_secs() * CLOCK_FLASH_HZ * std::f32::consts::TAU;
        peak * (phase.sin() * 0.5 + 0.5)
    };

    for (mut sprite, mut transform) in &mut backgrounds {
        sprite.color = COLOR_CLOCK_FLASH.with_alpha(pulse_alpha);
        transform.translation.y = target_y;
    }
}
