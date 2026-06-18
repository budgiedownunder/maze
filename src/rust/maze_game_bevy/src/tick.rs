//! Game tick driver — the single Bevy system that calls
//! `state.game.tick(dt_ms)` each `FixedUpdate` and dispatches the
//! resulting events to the appropriate per-entity handlers. Centralising
//! the tick call is important: `MazeGame::tick` mutates internal state
//! (door progress, enemy positions, HP, lost flag) so calling it from
//! multiple systems would double-step the world.
//!
//! Event handling:
//! - [`maze::GameEvent::DoorOpened`] → pin every leaf of the matching
//!   `DoorMarker` to its open pose permanently.
//! - [`maze::GameEvent::EnemyMoved`] → no-op here; the
//!   `enemy_animation_system` reads enemy state directly each frame.
//! - [`maze::GameEvent::PlayerDamaged`] → triggers the red full-screen
//!   damage flash via `state.damage_flash_timer`.
//! - [`maze::GameEvent::PlayerHealed`] → despawns the matching
//!   `HealthMarker` (the visual pickup).
//! - [`maze::GameEvent::PlayerNotHealed`] → silently dropped here. The
//!   implicit feedback (cell stays `'H'`, HP HUD stays full) is
//!   sufficient in a first-person 3D view; the React + MAUI bag-area UX
//!   surfaces the typed reason + message text.
//! - [`maze::GameEvent::KeyCollected`] → tags the matching `KeyMarker` with
//!   `CollectingKey`, so `key_collection_system` plays the rise-and-shrink
//!   flourish and despawns the holder.
//! - [`maze::GameEvent::TreasureCollected`] → tags the matching
//!   `TreasureMarker` with `CollectingTreasure`, so
//!   `treasure_collection_system` plays the rise-and-shrink flourish and
//!   despawns the rig. The score the treasure adds is already folded into
//!   `MazeGame::score`, so the HUD readout updates without extra work here.

use crate::state::GameState;
use crate::world::objects::door::DoorMarker;
use crate::world::objects::health::HealthMarker;
use crate::world::objects::key_holder::{CollectingKey, KeyMarker};
use crate::world::objects::treasure::{CollectingTreasure, TreasureMarker};
use bevy::prelude::*;
use maze::GameEvent;

/// Initial duration of the red damage flash, in milliseconds. Decremented
/// by [`damage_flash_system`]; the overlay alpha is linearly proportional.
pub(crate) const DAMAGE_FLASH_MS: f32 = 300.0;
/// Peak alpha of the damage-flash overlay at the moment damage lands.
const DAMAGE_FLASH_PEAK_ALPHA: f32 = 0.30;
/// Damage-flash overlay tint.
const DAMAGE_FLASH_COLOR: Color = Color::srgb(1.0, 0.1, 0.1);

/// Marker on the singleton red full-screen sprite that renders the
/// damage flash.
#[derive(Component)]
pub(crate) struct DamageFlash;

/// `FixedUpdate`: deterministic game-state tick driver. Pause / win /
/// lost all short-circuit so the world freezes at the moment of the
/// outcome.
pub(crate) fn game_tick_system(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<GameState>,
    mut doors: Query<&mut DoorMarker>,
    health_pickups: Query<(Entity, &HealthMarker)>,
    key_holders: Query<(Entity, &KeyMarker)>,
    treasures: Query<(Entity, &TreasureMarker)>,
) {
    if state.paused || state.won || state.lost {
        return;
    }
    let dt_ms = time.delta_secs() * 1000.0;
    for event in state.game.tick(dt_ms) {
        match event {
            GameEvent::DoorOpened { cell } => {
                for mut marker in &mut doors {
                    if marker.cell == cell {
                        marker.mark_opened();
                    }
                }
            }
            GameEvent::EnemyMoved { .. } => {
                // No per-event work — the per-frame `enemy_animation_system`
                // reads `state.game.enemies()` directly and interpolates.
            }
            GameEvent::PlayerDamaged { .. } => {
                state.damage_flash_timer = DAMAGE_FLASH_MS;
            }
            GameEvent::PlayerHealed { cell, .. } => {
                for (entity, marker) in &health_pickups {
                    if marker.cell == cell {
                        commands.entity(entity).despawn();
                    }
                }
            }
            GameEvent::PlayerNotHealed { .. } => {
                // Silently dropped here. The implicit feedback — cell
                // stays `'H'`, heart row stays full — is sufficient in
                // a first-person 3D view; the React + MAUI bag-area UX
                // surfaces the typed reason + message text.
            }
            GameEvent::KeyCollected { cell, .. } => {
                for (entity, marker) in &key_holders {
                    if marker.cell == cell {
                        commands.entity(entity).insert(CollectingKey::default());
                    }
                }
            }
            GameEvent::TreasureCollected { cell, .. } => {
                for (entity, marker) in &treasures {
                    if marker.cell == cell {
                        commands.entity(entity).insert(CollectingTreasure::default());
                    }
                }
            }
        }
    }
}

/// `Update`: drives the red damage-flash overlay. Spawned on demand the
/// first frame after `state.damage_flash_timer` goes positive; alpha
/// fades linearly with the timer; despawned when the timer reaches 0.
///
/// Runs through pause / win / lost so an in-flight flash from the last
/// non-paused tick finishes its fade naturally rather than freezing
/// mid-frame on the screen.
pub(crate) fn damage_flash_system(
    mut commands: Commands,
    time: Res<Time>,
    window: Query<&Window>,
    mut state: ResMut<GameState>,
    mut flashes: Query<(Entity, &mut Sprite, &mut Transform), With<DamageFlash>>,
) {
    if state.damage_flash_timer <= 0.0 {
        for (entity, _, _) in flashes.iter() {
            commands.entity(entity).despawn();
        }
        return;
    }
    let dt_ms = time.delta_secs() * 1000.0;
    state.damage_flash_timer = (state.damage_flash_timer - dt_ms).max(0.0);
    let progress = state.damage_flash_timer / DAMAGE_FLASH_MS;
    let alpha = DAMAGE_FLASH_PEAK_ALPHA * progress;
    let Ok(win) = window.single() else {
        return;
    };
    let size = Vec2::new(win.width(), win.height());
    let mut iter = flashes.iter_mut();
    if let Some((_, mut sprite, mut transform)) = iter.next() {
        sprite.color = DAMAGE_FLASH_COLOR.with_alpha(alpha);
        sprite.custom_size = Some(size);
        transform.translation.x = 0.0;
        transform.translation.y = 0.0;
    } else {
        commands.spawn((
            DamageFlash,
            Sprite {
                color: DAMAGE_FLASH_COLOR.with_alpha(alpha),
                custom_size: Some(size),
                ..default()
            },
            // Z between the world (low z) and the HUD (z 9) so the flash
            // tints the gameplay scene but doesn't obscure HUD readouts.
            Transform::from_xyz(0.0, 0.0, 8.0),
        ));
    }
}
