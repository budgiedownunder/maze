//! Level-indicator HUD.
//!
//! A top-left readout of the current level within a multi-level run, e.g.
//! `LEVEL  1 of 2`. Spawned only when the run has more than one level, so
//! single-level games show no indicator and look exactly as before. Anchored
//! below the score + time-bonus readouts and left-aligned so it grows rightward.

use crate::state::MultiLevelRun;
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Distance of the readout from the top / left screen edges. The top margin
/// sits below the score readout (`SCORE_MARGIN_TOP` = 30) and the time-bonus
/// readout (`BONUS_MARGIN_TOP` = 62) stacked above it.
const LEVEL_MARGIN_TOP: f32 = 94.0;
const LEVEL_MARGIN_LEFT: f32 = 12.0;

const COLOR_LEVEL: Color = Color::srgb(0.82, 0.84, 0.9);

/// Marker on the level-indicator text, tracking the last-rendered label so the
/// text is only rebuilt when the value actually changes.
#[derive(Component)]
pub(crate) struct LevelIndicator {
    last: String,
}

/// Spawns the level indicator — a no-op for a single-level run (the indicator
/// is meaningless when there is only one level, and omitting it keeps a
/// single-level game visually unchanged).
pub(crate) fn spawn_level_indicator(
    commands: &mut Commands,
    window: &Query<&Window>,
    run: &MultiLevelRun,
) {
    if run.level_count() <= 1 {
        return;
    }
    let (x, y) = anchor(window);
    let scale = window
        .single()
        .map(|w| crate::hud::hud_scale(w.width()))
        .unwrap_or(1.0);
    let label = level_text(run.current_level, run.level_count());
    commands.spawn((
        LevelIndicator {
            last: label.clone(),
        },
        Text2d::new(label),
        TextFont {
            font_size: 22.0,
            ..default()
        },
        TextColor(COLOR_LEVEL),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(x, y, 9.0).with_scale(Vec3::splat(scale)),
    ));
}

pub(crate) fn level_indicator_system(
    window: Query<&Window>,
    run: Res<MultiLevelRun>,
    mut hud: Query<(&mut LevelIndicator, &mut Text2d, &mut Transform)>,
) {
    let Ok((mut indicator, mut text, mut transform)) = hud.single_mut() else {
        return;
    };
    let (x, y) = anchor(&window);
    transform.translation.x = x;
    transform.translation.y = y;
    if let Ok(w) = window.single() {
        transform.scale = Vec3::splat(crate::hud::hud_scale(w.width()));
    }
    let label = level_text(run.current_level, run.level_count());
    if indicator.last != label {
        text.0 = label.clone();
        indicator.last = label;
    }
}

/// Top-left anchor point for the readout given the current window size.
fn anchor(window: &Query<&Window>) -> (f32, f32) {
    window
        .single()
        .map(|w| {
            (
                -w.width() / 2.0 + LEVEL_MARGIN_LEFT,
                w.height() / 2.0 - LEVEL_MARGIN_TOP,
            )
        })
        .unwrap_or((-628.0, 266.0))
}

/// The indicator label, e.g. `LEVEL  1 of 2` (1-based current level).
fn level_text(current_level: usize, total: usize) -> String {
    format!("LEVEL  {} of {}", current_level + 1, total)
}

#[cfg(test)]
mod tests {
    use super::level_text;

    #[test]
    fn level_text_is_one_based() {
        assert_eq!(level_text(0, 2), "LEVEL  1 of 2");
        assert_eq!(level_text(1, 2), "LEVEL  2 of 2");
        assert_eq!(level_text(2, 5), "LEVEL  3 of 5");
    }
}
