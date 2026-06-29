//! Time-bonus HUD.
//!
//! A top-left readout (one line below SCORE) of the time bonus the run would
//! earn if it finished right now — [`crate::state::time_bonus`] evaluated
//! against the live clock. It holds at the maximum through an initial lead time,
//! then ticks down as the timer runs, so the player can watch the reward for a
//! faster finish shrink in real time — reaching zero once the run passes the
//! bonus cutoff (half the total time). Frozen — like the clock it reads — during
//! pause, level transitions, and the end overlays.

use crate::state::{time_bonus, GameClock};
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Distance of the readout from the top / left screen edges. Sits one line below
/// the SCORE readout (which uses a 30 px top margin).
const BONUS_MARGIN_TOP: f32 = 62.0;
const BONUS_MARGIN_LEFT: f32 = 12.0;

/// Matches the warm-yellow SCORE readout above it (`hud::score::COLOR_SCORE`) so
/// the two top-left lines read as one group.
const COLOR_BONUS: Color = Color::srgb(0.95, 0.9, 0.55);

/// Marker on the readout text, tracking the last-rendered bonus so the text is
/// only rebuilt when the displayed value changes.
#[derive(Component)]
pub(crate) struct TimeBonusHud {
    last_bonus: u64,
}

pub(crate) fn spawn_time_bonus_hud(commands: &mut Commands, window: &Query<&Window>) {
    let (x, y) = top_left(window);
    let scale = window
        .single()
        .map(|w| crate::hud::hud_scale(w.width()))
        .unwrap_or(1.0);
    commands.spawn((
        // `u64::MAX` forces the first system tick to render the real value.
        TimeBonusHud { last_bonus: u64::MAX },
        Text2d::new(bonus_text(0)),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(COLOR_BONUS),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(x, y, 9.0).with_scale(Vec3::splat(scale)),
    ));
}

pub(crate) fn time_bonus_hud_system(
    window: Query<&Window>,
    clock: Res<GameClock>,
    mut hud: Query<(&mut TimeBonusHud, &mut Text2d, &mut Transform)>,
) {
    let Ok((mut bonus_hud, mut text, mut transform)) = hud.single_mut() else {
        return;
    };
    let (x, y) = top_left(&window);
    transform.translation.x = x;
    transform.translation.y = y;
    if let Ok(w) = window.single() {
        // Shrink on narrow widths in step with SCORE so the left-anchored stack
        // keeps clear of the centred clock.
        transform.scale = Vec3::splat(crate::hud::hud_scale(w.width()));
    }

    // Total game time is the clock's starting value (elapsed + remaining), so the
    // cutoff tracks the clock the player actually sees (incl. the demo's long one).
    let bonus = time_bonus(clock.elapsed_secs, clock.elapsed_secs + clock.remaining_secs);
    if bonus_hud.last_bonus != bonus {
        text.0 = bonus_text(bonus);
        bonus_hud.last_bonus = bonus;
    }
}

/// Top-left anchor point for the readout given the current window size.
fn top_left(window: &Query<&Window>) -> (f32, f32) {
    window
        .single()
        .map(|w| {
            (
                -w.width() / 2.0 + BONUS_MARGIN_LEFT,
                w.height() / 2.0 - BONUS_MARGIN_TOP,
            )
        })
        .unwrap_or((-628.0, 298.0))
}

fn bonus_text(bonus: u64) -> String {
    format!("BONUS  +{}", bonus)
}
