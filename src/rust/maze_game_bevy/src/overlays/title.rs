use crate::palette::COLOR_GOLD;
use crate::state::{AppState, GameConfig, TitleTimer};
use bevy::prelude::*;

const COLOR_SPLASH_SHADOW: Color = Color::srgb(0.25, 0.15, 0.0);

#[derive(Component)]
pub(crate) struct TitleEntity;

#[derive(Component, PartialEq)]
pub(crate) enum TitleTextKind {
    Shadow,
    Gold,
    Sub,
}

/// The splash subtitle text for the given time left on the start countdown.
///
/// Whole seconds, rounded up, so each value is shown for a full second:
/// "(3s)" while 2–3 s remain, "(2s)" while 1–2 s remain, "(1s)" for the final
/// second. It never reads "(0s)" — the title tears down the instant the timer
/// finishes.
pub(crate) fn countdown_label(remaining_secs: f32) -> String {
    let seconds_left = remaining_secs.max(0.0).ceil() as u32;
    format!("Starting...({seconds_left}s)")
}

pub(crate) fn setup_title(
    mut commands: Commands,
    config: Res<GameConfig>,
    timer: Res<TitleTimer>,
) {
    commands.spawn((Camera2d, TitleEntity));
    let title = config.title.clone();
    // Shadow layer — offset down-right; font size updated reactively by title_resize_system
    commands.spawn((
        Text2d::new(title.clone()),
        TextFont { font_size: 96.0, ..default() },
        TextColor(COLOR_SPLASH_SHADOW),
        Transform::from_translation(Vec3::new(4.0, -4.0, -0.1)),
        TitleEntity,
        TitleTextKind::Shadow,
    ));
    // Main gold layer
    commands.spawn((
        Text2d::new(title),
        TextFont { font_size: 96.0, ..default() },
        TextColor(COLOR_GOLD),
        TitleEntity,
        TitleTextKind::Gold,
    ));
    // Subtitle — a live countdown ticking down to the auto-transition.
    commands.spawn((
        Text2d::new(countdown_label(timer.0.remaining_secs())),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_translation(Vec3::new(0.0, -80.0, 0.0)),
        TitleEntity,
        TitleTextKind::Sub,
    ));
}

pub(crate) fn title_resize_system(
    window: Query<&Window>,
    mut last_width: Local<f32>,
    mut texts: Query<(&mut TextFont, &mut Transform, &TitleTextKind)>,
) {
    let width = window.single().map(|w| w.width()).unwrap_or(1280.0);
    if (width - *last_width).abs() < 0.5 {
        return;
    }
    *last_width = width;

    let font_size = (width / 5.5).min(96.0);
    let shadow_off = font_size / 24.0;
    let subtitle_size = (font_size / 4.0).max(14.0);
    let subtitle_y = -(font_size * 0.85);

    for (mut font, mut t, kind) in &mut texts {
        match kind {
            TitleTextKind::Sub => {
                font.font_size = subtitle_size;
                t.translation = Vec3::new(0.0, subtitle_y, 0.0);
            }
            TitleTextKind::Shadow => {
                font.font_size = font_size;
                t.translation = Vec3::new(shadow_off, -shadow_off, -0.1);
            }
            TitleTextKind::Gold => {
                font.font_size = font_size;
            }
        }
    }
}

pub(crate) fn tick_title(
    time: Res<Time>,
    mut timer: ResMut<TitleTimer>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        next_state.set(AppState::Playing);
    }
}

pub(crate) fn update_title_countdown(
    timer: Res<TitleTimer>,
    mut texts: Query<(&mut Text2d, &TitleTextKind)>,
) {
    let desired = countdown_label(timer.0.remaining_secs());
    for (mut text, kind) in &mut texts {
        if *kind == TitleTextKind::Sub && text.0 != desired {
            text.0 = desired.clone();
        }
    }
}

pub(crate) fn teardown_title(mut commands: Commands, query: Query<Entity, With<TitleEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::countdown_label;

    #[test]
    fn countdown_label_rounds_each_second_up() {
        // Each whole-second value shows for its full second: the ceiling of the
        // remaining time. A fresh 3 s timer reads "(3s)"; the final second reads
        // "(1s)"; it never reaches "(0s)".
        assert_eq!(countdown_label(3.0), "Starting...(3s)");
        assert_eq!(countdown_label(2.5), "Starting...(3s)");
        assert_eq!(countdown_label(2.0), "Starting...(2s)");
        assert_eq!(countdown_label(1.01), "Starting...(2s)");
        assert_eq!(countdown_label(1.0), "Starting...(1s)");
        assert_eq!(countdown_label(0.01), "Starting...(1s)");
    }

    #[test]
    fn countdown_label_clamps_non_positive_to_zero() {
        // A spent or slightly-negative remaining time must not underflow the
        // unsigned cast — it floors at "(0s)".
        assert_eq!(countdown_label(0.0), "Starting...(0s)");
        assert_eq!(countdown_label(-1.0), "Starting...(0s)");
    }
}
