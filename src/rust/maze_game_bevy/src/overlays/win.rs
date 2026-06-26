use crate::palette::COLOR_OVERLAY_BACKDROP;
use crate::state::GameState;
use bevy::prelude::*;
use std::f32::consts::PI;

const COLOR_WIN_GOLD: Color = Color::srgb(1.0, 0.8, 0.1);
pub(crate) const COLOR_ORB_LIGHT: Color = Color::srgb(1.0, 0.85, 0.2);
/// Bright gold for the record banner (high score / fastest time) — lighter than
/// the title's [`COLOR_WIN_GOLD`] so the two gold lines stay distinct.
const COLOR_RECORD_BANNER: Color = Color::srgb(1.0, 0.9, 0.35);

#[derive(Component)]
pub(crate) struct WinOverlay;

#[derive(Component)]
pub(crate) struct WinBackground;

#[derive(Component)]
pub(crate) struct WinMainText;

#[derive(Component)]
pub(crate) struct WinScoreText;

#[derive(Component)]
pub(crate) struct WinSubText;

#[derive(Component)]
pub(crate) struct WinHighScoreText;

#[derive(Component)]
pub(crate) struct WinLeaf {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) vel_x: f32,
    pub(crate) vel_y: f32,
    pub(crate) rot: f32,
    pub(crate) rot_speed: f32,
}

/// Formats an elapsed duration in milliseconds as `M:SS.mmm` — the run's
/// completion time, shown to millisecond precision (the on-screen clock only
/// shows the remaining countdown to whole seconds).
fn format_elapsed(ms: u64) -> String {
    let total_secs = ms / 1000;
    format!("{}:{:02}.{:03}", total_secs / 60, total_secs % 60, ms % 1000)
}

/// Base (scale-1) geometry for the win panel. The record banner adds a line
/// under the title, so its layout is wider + taller with a smaller title to fit
/// the longest banner ("High Score and Fastest Time"). Both `spawn_win_overlay`
/// and `win_resize_system` read this so the spawn and the resize stay in step.
struct WinLayout {
    width: f32,
    height: f32,
    title_y: f32,
    title_font: f32,
    banner_y: f32,
    banner_font: f32,
    score_y: f32,
    time_y: f32,
}

fn win_layout(has_banner: bool) -> WinLayout {
    if has_banner {
        WinLayout {
            width: 440.0,
            height: 196.0,
            title_y: 50.0,
            title_font: 56.0,
            banner_y: 8.0,
            banner_font: 22.0,
            score_y: -28.0,
            time_y: -54.0,
        }
    } else {
        WinLayout {
            width: 340.0,
            height: 150.0,
            title_y: 26.0,
            title_font: 72.0,
            banner_y: 0.0,
            banner_font: 0.0,
            score_y: -18.0,
            time_y: -46.0,
        }
    }
}

/// The record-banner text for the won run, or `None` when it set no record.
fn banner_text(high_score: bool, fastest_time: bool) -> Option<&'static str> {
    match (high_score, fastest_time) {
        (true, true) => Some("High Score and Fastest Time"),
        (true, false) => Some("High Score"),
        (false, true) => Some("Fastest Time"),
        (false, false) => None,
    }
}

pub(crate) fn spawn_win_overlay(
    commands: &mut Commands,
    score: u64,
    bonus: u64,
    elapsed_ms: u64,
    high_score: bool,
    fastest_time: bool,
) {
    let banner = banner_text(high_score, fastest_time);
    let l = win_layout(banner.is_some());
    commands.spawn((
        WinOverlay,
        WinBackground,
        Sprite {
            color: COLOR_OVERLAY_BACKDROP,
            custom_size: Some(Vec2::new(l.width, l.height)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));
    commands.spawn((
        WinOverlay,
        WinMainText,
        Text2d::new("You Win!"),
        TextFont { font_size: l.title_font, ..default() },
        TextColor(COLOR_WIN_GOLD),
        Transform::from_xyz(0.0, l.title_y, 11.0),
    ));
    // Celebratory banner under the title — only when this run set a record.
    if let Some(text) = banner {
        commands.spawn((
            WinOverlay,
            WinHighScoreText,
            Text2d::new(text),
            TextFont { font_size: l.banner_font, ..default() },
            TextColor(COLOR_RECORD_BANNER),
            Transform::from_xyz(0.0, l.banner_y, 11.0),
        ));
    }
    commands.spawn((
        WinOverlay,
        WinScoreText,
        Text2d::new(format!("Score  {}  (+{} bonus)", score, bonus)),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, l.score_y, 11.0),
    ));
    commands.spawn((
        WinOverlay,
        WinSubText,
        Text2d::new(format!("Time  {}", format_elapsed(elapsed_ms))),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, l.time_y, 11.0),
    ));
}

#[allow(clippy::type_complexity)]
pub(crate) fn win_resize_system(
    window: Query<&Window>,
    mut last_width: Local<f32>,
    high_score_present: Query<(), With<WinHighScoreText>>,
    mut win_texts: Query<
        (
            &mut TextFont,
            &mut Transform,
            Option<&WinMainText>,
            Option<&WinScoreText>,
            Option<&WinHighScoreText>,
        ),
        With<WinOverlay>,
    >,
    mut win_sprites: Query<&mut Sprite, With<WinBackground>>,
) {
    let width = window.single().map(|w| w.width()).unwrap_or(1280.0);
    if (width - *last_width).abs() < 0.5 {
        return;
    }
    *last_width = width;

    let scale = (width / 5.5).min(96.0) / 96.0;
    // The four-line (banner) layout is active whenever the banner entity exists;
    // pick the matching geometry so the resize tracks the spawned positions.
    let l = win_layout(!high_score_present.is_empty());
    for (mut font, mut t, is_main, is_score, is_high) in &mut win_texts {
        if is_high.is_some() {
            font.font_size = (l.banner_font * scale).max(14.0);
            t.translation = Vec3::new(0.0, l.banner_y * scale, 11.0);
        } else if is_main.is_some() {
            font.font_size = l.title_font * scale;
            t.translation = Vec3::new(0.0, l.title_y * scale, 11.0);
        } else if is_score.is_some() {
            font.font_size = (24.0 * scale).max(14.0);
            t.translation = Vec3::new(0.0, l.score_y * scale, 11.0);
        } else {
            font.font_size = (24.0 * scale).max(14.0);
            t.translation = Vec3::new(0.0, l.time_y * scale, 11.0);
        }
    }
    for mut sprite in &mut win_sprites {
        sprite.custom_size = Some(Vec2::new(l.width * scale, l.height * scale));
    }
}

pub(crate) fn leaf_system(
    mut commands: Commands,
    time: Res<Time>,
    window: Query<&Window>,
    state: Res<GameState>,
    mut leaves: Query<(Entity, &mut WinLeaf, &mut Transform)>,
    mut rng: Local<u64>,
    mut timer: Local<f32>,
) {
    let Ok(win) = window.single() else { return; };
    let half_w = win.width() / 2.0;
    let half_h = win.height() / 2.0;
    let dt = time.delta_secs();

    for (entity, mut leaf, mut transform) in &mut leaves {
        leaf.x += leaf.vel_x * dt;
        leaf.y += leaf.vel_y * dt;
        leaf.rot += leaf.rot_speed * dt;
        transform.translation.x = leaf.x;
        transform.translation.y = leaf.y;
        transform.rotation = Quat::from_rotation_z(leaf.rot);
        if leaf.y < -(half_h + 20.0) {
            commands.entity(entity).despawn();
        }
    }

    if !state.won {
        return;
    }

    if *rng == 0 {
        *rng = time.elapsed_secs_f64().to_bits() | 1;
    }

    *timer += dt;
    while *timer >= 0.1 {
        *timer -= 0.1;
        for _ in 0..4 {
            let x = crate::world::lcg(&mut rng) * 2.0 * half_w - half_w;
            let vx = (crate::world::lcg(&mut rng) - 0.5) * 80.0;
            let vy = -(100.0 + crate::world::lcg(&mut rng) * 100.0);
            let rot = crate::world::lcg(&mut rng) * 2.0 * PI;
            let rot_speed = (crate::world::lcg(&mut rng) - 0.5) * 6.0;
            let hue = crate::world::lcg(&mut rng);
            commands.spawn((
                WinLeaf { x, y: half_h + 10.0, vel_x: vx, vel_y: vy, rot, rot_speed },
                Sprite {
                    color: Color::srgba(1.0, 0.65 + hue * 0.35, 0.0, 0.85),
                    custom_size: Some(Vec2::new(12.0, 5.0)),
                    ..default()
                },
                Transform::from_xyz(x, half_h + 10.0, 9.0)
                    .with_rotation(Quat::from_rotation_z(rot)),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{banner_text, format_elapsed};

    #[test]
    fn format_elapsed_renders_minutes_seconds_and_millis() {
        assert_eq!(format_elapsed(0), "0:00.000");
        assert_eq!(format_elapsed(83_456), "1:23.456");
        assert_eq!(format_elapsed(60_000), "1:00.000");
        assert_eq!(format_elapsed(9), "0:00.009");
        assert_eq!(format_elapsed(605_007), "10:05.007");
    }

    #[test]
    fn banner_text_covers_every_record_combination() {
        assert_eq!(banner_text(true, true), Some("High Score and Fastest Time"));
        assert_eq!(banner_text(true, false), Some("High Score"));
        assert_eq!(banner_text(false, true), Some("Fastest Time"));
        assert_eq!(banner_text(false, false), None);
    }
}
