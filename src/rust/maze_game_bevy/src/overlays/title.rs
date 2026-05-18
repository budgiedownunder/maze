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

pub(crate) fn setup_title(mut commands: Commands, config: Res<GameConfig>) {
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
    // Subtitle
    commands.spawn((
        Text2d::new("Starting..."),
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

pub(crate) fn teardown_title(mut commands: Commands, query: Query<Entity, With<TitleEntity>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}
