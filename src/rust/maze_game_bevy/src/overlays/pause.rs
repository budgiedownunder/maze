use crate::palette::COLOR_OVERLAY_BACKDROP;
use crate::state::{dispatch_pause_state, GameState};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct PausedOverlay;

pub(crate) fn spawn_paused_overlay(commands: &mut Commands) {
    // Paused overlay — pre-spawned hidden so a pause toggle is a single
    // visibility flip (no per-toggle spawn/despawn flicker).
    commands.spawn((
        PausedOverlay,
        Visibility::Hidden,
        Sprite {
            color: COLOR_OVERLAY_BACKDROP,
            custom_size: Some(Vec2::new(340.0, 130.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));
    commands.spawn((
        PausedOverlay,
        Visibility::Hidden,
        Text2d::new("PAUSED"),
        TextFont { font_size: 64.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, 0.0, 11.0),
    ));
}

pub(crate) fn pause_system(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<GameState>,
    mut overlay: Query<&mut Visibility, With<PausedOverlay>>,
) {
    // No pause once the game has resolved
    if state.won || state.lost {
        return;
    }
    let Some(keys) = keys else { return; };
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    state.paused = !state.paused;
    let new_vis = if state.paused { Visibility::Visible } else { Visibility::Hidden };
    for mut v in &mut overlay {
        *v = new_vis;
    }
    dispatch_pause_state(state.paused);
}
