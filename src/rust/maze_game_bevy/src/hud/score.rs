//! Score HUD.
//!
//! A top-left readout of the run's live score, reading
//! [`maze::MazeGame::score`] each frame. Left-anchored so the text grows
//! rightward as the value climbs (the score never shifts its left edge), and
//! repositioned each frame so window resizes track the top-left corner.

use crate::state::GameState;
use bevy::prelude::*;
use bevy::sprite::Anchor;

/// Distance of the readout from the top / left screen edges.
const SCORE_MARGIN_TOP: f32 = 30.0;
const SCORE_MARGIN_LEFT: f32 = 12.0;

const COLOR_SCORE: Color = Color::srgb(0.95, 0.9, 0.55);

/// Marker on the score-readout text, tracking the last-rendered score so the
/// text is only rebuilt when the value actually changes.
#[derive(Component)]
pub(crate) struct ScoreHud {
    last_score: u64,
}

pub(crate) fn spawn_score_hud(commands: &mut Commands, window: &Query<&Window>) {
    let (x, y) = top_left(window);
    commands.spawn((
        ScoreHud { last_score: 0 },
        Text2d::new(score_text(0)),
        TextFont {
            font_size: 26.0,
            ..default()
        },
        TextColor(COLOR_SCORE),
        Anchor::CENTER_LEFT,
        Transform::from_xyz(x, y, 9.0),
    ));
}

pub(crate) fn score_hud_system(
    window: Query<&Window>,
    state: Res<GameState>,
    mut hud: Query<(&mut ScoreHud, &mut Text2d, &mut Transform)>,
) {
    let Ok((mut score_hud, mut text, mut transform)) = hud.single_mut() else {
        return;
    };
    let (x, y) = top_left(&window);
    transform.translation.x = x;
    transform.translation.y = y;

    let score = state.game.score();
    if score_hud.last_score != score {
        text.0 = score_text(score);
        score_hud.last_score = score;
    }
}

/// Top-left anchor point for the readout given the current window size.
fn top_left(window: &Query<&Window>) -> (f32, f32) {
    window
        .single()
        .map(|w| {
            (
                -w.width() / 2.0 + SCORE_MARGIN_LEFT,
                w.height() / 2.0 - SCORE_MARGIN_TOP,
            )
        })
        .unwrap_or((-628.0, 330.0))
}

fn score_text(score: u64) -> String {
    format!("SCORE  {}", score)
}
