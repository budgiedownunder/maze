use crate::state::{GameConfig, MultiLevelRun};
use bevy::prelude::*;
use bevy::sprite::Anchor;

const COLOR_STATUSBAR_BG: Color = Color::srgba(0.10, 0.10, 0.14, 0.80);
const COLOR_STATUSBAR_TEXT: Color = Color::srgb(0.67, 0.60, 0.92);

const STATUSBAR_BG_W: f32 = 140.0;
const STATUSBAR_BG_H: f32 = 36.0;
/// Left edge of the maze-name text from the screen edge — matches the SCORE
/// readout's `SCORE_MARGIN_LEFT` so the two are left-aligned.
const STATUSBAR_LEFT: f32 = 12.0;
/// Left padding of the text inside the background pill.
const STATUSBAR_TEXT_PAD: f32 = 8.0;
/// Distance of the pill's centre below the top edge — places this row (which
/// shows the maze name) just under the top-left SCORE line.
const STATUSBAR_TOP: f32 = 68.0;
/// Extra drop applied **only** when the multi-level "LEVEL i of N" indicator is
/// shown (it sits between SCORE and this row), so the maze name clears it instead
/// of overlapping. Single-level games show no indicator and keep `STATUSBAR_TOP`.
const STATUSBAR_LEVEL_DROP: f32 = 30.0;

#[derive(Component)]
pub(crate) struct StatusBar;

#[derive(Component)]
pub(crate) struct ModeText;

pub(crate) fn spawn_statusbar(
    commands: &mut Commands,
    window: &Query<&Window>,
    config: &GameConfig,
    // Whether the level indicator is shown above this row (multi-level run), so
    // the maze name drops below it.
    level_shown: bool,
) {
    // `statusbar_resize_system` repositions both nodes each frame so window
    // resizes track the top-left corner (just below the SCORE line, or below the
    // LEVEL line on a multi-level run). The text is left-anchored to align with
    // SCORE; the pill sits behind it.
    let (bg_x, text_x, y) = window
        .single()
        .map(|w| positions(w.width(), w.height(), level_shown))
        .unwrap_or((-558.0, -628.0, 290.0));
    commands.spawn((
        StatusBar,
        Sprite {
            color: COLOR_STATUSBAR_BG,
            custom_size: Some(Vec2::new(STATUSBAR_BG_W, STATUSBAR_BG_H)),
            ..default()
        },
        Transform::from_xyz(bg_x, y, 8.9),
    ));
    commands.spawn((
        ModeText,
        Text2d::new(config.mode.clone()),
        TextFont { font_size: 22.0, ..default() },
        TextColor(COLOR_STATUSBAR_TEXT),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(text_x, y, 9.0),
    ));
}

/// Returns `(pill_centre_x, text_left_x, y)` for the current window size. The
/// text's left edge aligns with the SCORE readout; the pill is centred behind
/// it with a small left padding. `level_shown` drops the row below the LEVEL
/// indicator on a multi-level run.
fn positions(win_w: f32, win_h: f32, level_shown: bool) -> (f32, f32, f32) {
    let text_x = -win_w / 2.0 + STATUSBAR_LEFT;
    let bg_x = text_x - STATUSBAR_TEXT_PAD + STATUSBAR_BG_W / 2.0;
    let drop = if level_shown { STATUSBAR_LEVEL_DROP } else { 0.0 };
    let y = win_h / 2.0 - STATUSBAR_TOP - drop;
    (bg_x, text_x, y)
}

pub(crate) fn statusbar_resize_system(
    window: Query<&Window>,
    run: Res<MultiLevelRun>,
    mut bg: Query<&mut Transform, (With<StatusBar>, Without<ModeText>)>,
    mut text: Query<&mut Transform, (With<ModeText>, Without<StatusBar>)>,
) {
    let Ok(win) = window.single() else { return; };
    let (bg_x, text_x, y) = positions(win.width(), win.height(), run.level_count() > 1);
    for mut t in &mut bg {
        t.translation.x = bg_x;
        t.translation.y = y;
    }
    for mut t in &mut text {
        t.translation.x = text_x;
        t.translation.y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maze_name_drops_below_the_level_indicator_only_when_shown() {
        let (_, _, y_single) = positions(1280.0, 720.0, false);
        let (_, _, y_multi) = positions(1280.0, 720.0, true);
        // Smaller y is lower on screen; the name row drops by exactly the level
        // indicator's row when the indicator is shown, and not at all otherwise.
        assert!(y_multi < y_single, "the indicator pushes the name row down");
        assert_eq!(y_single - y_multi, STATUSBAR_LEVEL_DROP);
    }
}
