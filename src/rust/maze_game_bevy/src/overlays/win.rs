use crate::palette::COLOR_OVERLAY_BACKDROP;
use crate::state::GameState;
use bevy::prelude::*;
use std::f32::consts::PI;

const COLOR_WIN_GOLD: Color = Color::srgb(1.0, 0.8, 0.1);
pub(crate) const COLOR_ORB_LIGHT: Color = Color::srgb(1.0, 0.85, 0.2);

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

pub(crate) fn spawn_win_overlay(commands: &mut Commands, score: u64, elapsed_ms: u64) {
    commands.spawn((
        WinOverlay,
        WinBackground,
        Sprite {
            color: COLOR_OVERLAY_BACKDROP,
            custom_size: Some(Vec2::new(340.0, 150.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));
    commands.spawn((
        WinOverlay,
        WinMainText,
        Text2d::new("You Win!"),
        TextFont { font_size: 72.0, ..default() },
        TextColor(COLOR_WIN_GOLD),
        Transform::from_xyz(0.0, 26.0, 11.0),
    ));
    commands.spawn((
        WinOverlay,
        WinScoreText,
        Text2d::new(format!("Score  {}", score)),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -18.0, 11.0),
    ));
    commands.spawn((
        WinOverlay,
        WinSubText,
        Text2d::new(format!("Time  {}", format_elapsed(elapsed_ms))),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -46.0, 11.0),
    ));
}

#[allow(clippy::type_complexity)]
pub(crate) fn win_resize_system(
    window: Query<&Window>,
    mut last_width: Local<f32>,
    mut win_texts: Query<
        (
            &mut TextFont,
            &mut Transform,
            Option<&WinMainText>,
            Option<&WinScoreText>,
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
    for (mut font, mut t, is_main, is_score) in &mut win_texts {
        if is_main.is_some() {
            font.font_size = 72.0 * scale;
            t.translation = Vec3::new(0.0, 26.0 * scale, 11.0);
        } else if is_score.is_some() {
            font.font_size = (24.0 * scale).max(14.0);
            t.translation = Vec3::new(0.0, -18.0 * scale, 11.0);
        } else {
            font.font_size = (24.0 * scale).max(14.0);
            t.translation = Vec3::new(0.0, -46.0 * scale, 11.0);
        }
    }
    for mut sprite in &mut win_sprites {
        sprite.custom_size = Some(Vec2::new(340.0 * scale, 150.0 * scale));
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
    use super::format_elapsed;

    #[test]
    fn format_elapsed_renders_minutes_seconds_and_millis() {
        assert_eq!(format_elapsed(0), "0:00.000");
        assert_eq!(format_elapsed(83_456), "1:23.456");
        assert_eq!(format_elapsed(60_000), "1:00.000");
        assert_eq!(format_elapsed(9), "0:00.009");
        assert_eq!(format_elapsed(605_007), "10:05.007");
    }
}
