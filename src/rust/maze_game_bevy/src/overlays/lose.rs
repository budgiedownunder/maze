use crate::palette::COLOR_OVERLAY_BACKDROP;
use crate::state::GameState;
use bevy::prelude::*;

const COLOR_LOSE_RED: Color = Color::srgb(0.95, 0.3, 0.25);
const COLOR_RAIN: Color = Color::srgba(0.6, 0.75, 1.0, 0.55);
const COLOR_LIGHTNING_PEAK: Color = Color::srgba(1.0, 1.0, 1.0, 0.7);

#[derive(Component)]
pub(crate) struct LoseOverlay;

#[derive(Component)]
pub(crate) struct LoseMainText;

#[derive(Component)]
pub(crate) struct LoseSubText;

#[derive(Component)]
pub(crate) struct LoseBackground;

#[derive(Component)]
pub(crate) struct RainDrop {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) vel_x: f32,
    pub(crate) vel_y: f32,
}

#[derive(Component)]
pub(crate) struct LightningQuad {
    pub(crate) remaining: f32,
    pub(crate) duration: f32,
}

pub(crate) fn spawn_lose_overlay(commands: &mut Commands) {
    commands.spawn((
        LoseOverlay,
        LoseBackground,
        Sprite {
            color: COLOR_OVERLAY_BACKDROP,
            custom_size: Some(Vec2::new(340.0, 130.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 10.0),
    ));
    commands.spawn((
        LoseOverlay,
        LoseMainText,
        Text2d::new("You Lose!"),
        TextFont { font_size: 72.0, ..default() },
        TextColor(COLOR_LOSE_RED),
        Transform::from_xyz(0.0, 16.0, 11.0),
    ));
    commands.spawn((
        LoseOverlay,
        LoseSubText,
        Text2d::new("Time's up"),
        TextFont { font_size: 24.0, ..default() },
        TextColor(Color::WHITE),
        Transform::from_xyz(0.0, -36.0, 11.0),
    ));
}

pub(crate) fn lose_resize_system(
    window: Query<&Window>,
    mut last_width: Local<f32>,
    mut lose_texts: Query<(&mut TextFont, &mut Transform, Option<&LoseMainText>), With<LoseOverlay>>,
    mut lose_sprites: Query<&mut Sprite, With<LoseBackground>>,
) {
    let width = window.single().map(|w| w.width()).unwrap_or(1280.0);
    if (width - *last_width).abs() < 0.5 {
        return;
    }
    *last_width = width;

    let scale = (width / 5.5).min(96.0) / 96.0;
    for (mut font, mut t, is_main) in &mut lose_texts {
        if is_main.is_some() {
            font.font_size = 72.0 * scale;
            t.translation = Vec3::new(0.0, 16.0 * scale, 11.0);
        } else {
            font.font_size = (24.0 * scale).max(14.0);
            t.translation = Vec3::new(0.0, -36.0 * scale, 11.0);
        }
    }
    for mut sprite in &mut lose_sprites {
        sprite.custom_size = Some(Vec2::new(340.0 * scale, 130.0 * scale));
    }
}

pub(crate) fn rain_system(
    mut commands: Commands,
    time: Res<Time>,
    window: Query<&Window>,
    state: Res<GameState>,
    mut drops: Query<(Entity, &mut RainDrop, &mut Transform)>,
    mut rng: Local<u64>,
    mut timer: Local<f32>,
) {
    let Ok(win) = window.single() else { return; };
    let half_w = win.width() / 2.0;
    let half_h = win.height() / 2.0;
    let dt = time.delta_secs();

    for (entity, mut drop, mut transform) in &mut drops {
        drop.x += drop.vel_x * dt;
        drop.y += drop.vel_y * dt;
        transform.translation.x = drop.x;
        transform.translation.y = drop.y;
        if drop.y < -(half_h + 20.0) {
            commands.entity(entity).despawn();
        }
    }

    if !state.lost {
        return;
    }

    if *rng == 0 {
        *rng = time.elapsed_secs_f64().to_bits() | 1;
    }

    // Heavy rain — denser than the win-leaf system. 6 drops every 40 ms ≈ 150 drops/s.
    *timer += dt;
    while *timer >= 0.04 {
        *timer -= 0.04;
        for _ in 0..6 {
            let x = crate::world::lcg(&mut rng) * 2.0 * half_w - half_w;
            let vx = (crate::world::lcg(&mut rng) - 0.5) * 60.0;
            let vy = -(500.0 + crate::world::lcg(&mut rng) * 250.0);
            commands.spawn((
                RainDrop { x, y: half_h + 10.0, vel_x: vx, vel_y: vy },
                Sprite {
                    color: COLOR_RAIN,
                    custom_size: Some(Vec2::new(2.0, 12.0)),
                    ..default()
                },
                Transform::from_xyz(x, half_h + 10.0, 9.0),
            ));
        }
    }
}

pub(crate) fn lightning_system(
    mut commands: Commands,
    time: Res<Time>,
    window: Query<&Window>,
    state: Res<GameState>,
    mut flashes: Query<(Entity, &mut LightningQuad, &mut Sprite)>,
    mut rng: Local<u64>,
    mut cooldown: Local<f32>,
) {
    let dt = time.delta_secs();

    // Fade existing flashes (quadratic falloff so the peak feels sharp).
    for (entity, mut flash, mut sprite) in &mut flashes {
        flash.remaining -= dt;
        if flash.remaining <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let progress = flash.remaining / flash.duration;
        let alpha = 0.7 * progress * progress;
        sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha);
    }

    if !state.lost {
        return;
    }

    if *rng == 0 {
        *rng = time.elapsed_secs_f64().to_bits() | 2;
    }
    if *cooldown <= 0.0 {
        *cooldown = 3.0 + crate::world::lcg(&mut rng) * 3.0;
        return;
    }
    *cooldown -= dt;
    if *cooldown > 0.0 {
        return;
    }
    *cooldown = 3.0 + crate::world::lcg(&mut rng) * 3.0;

    let Ok(win) = window.single() else { return; };
    let (w, h) = (win.width(), win.height());
    // z=8 puts the flash behind the rain (z=9) and the lose overlay/text (z=10/11),
    // so the foreground stays sharp during the flash.
    let duration = 0.2;
    commands.spawn((
        LightningQuad { remaining: duration, duration },
        Sprite {
            color: COLOR_LIGHTNING_PEAK,
            custom_size: Some(Vec2::new(w, h)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 8.0),
    ));
}
