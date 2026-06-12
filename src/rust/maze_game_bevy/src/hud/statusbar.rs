use crate::state::GameConfig;
use bevy::prelude::*;

const COLOR_STATUSBAR_BG: Color = Color::srgba(0.10, 0.10, 0.14, 0.80);
const COLOR_STATUSBAR_TEXT: Color = Color::srgb(0.67, 0.60, 0.92);

const STATUSBAR_BG_W: f32 = 140.0;
const STATUSBAR_BG_H: f32 = 36.0;
const STATUSBAR_MARGIN: f32 = 12.0;

#[derive(Component)]
pub(crate) struct StatusBar;

#[derive(Component)]
pub(crate) struct ModeText;

pub(crate) fn spawn_statusbar(commands: &mut Commands, window: &Query<&Window>, config: &GameConfig) {
    // `statusbar_resize_system` repositions both nodes each frame so window
    // resizes track the bottom-left corner.
    let (sb_x, sb_y) = window
        .single()
        .map(|w| {
            (
                -w.width() / 2.0 + STATUSBAR_MARGIN + STATUSBAR_BG_W / 2.0,
                -w.height() / 2.0 + STATUSBAR_MARGIN + STATUSBAR_BG_H / 2.0,
            )
        })
        .unwrap_or((-500.0, -330.0));
    commands.spawn((
        StatusBar,
        Sprite {
            color: COLOR_STATUSBAR_BG,
            custom_size: Some(Vec2::new(STATUSBAR_BG_W, STATUSBAR_BG_H)),
            ..default()
        },
        Transform::from_xyz(sb_x, sb_y, 8.9),
    ));
    commands.spawn((
        ModeText,
        Text2d::new(config.mode.clone()),
        TextFont { font_size: 22.0, ..default() },
        TextColor(COLOR_STATUSBAR_TEXT),
        Transform::from_xyz(sb_x, sb_y, 9.0),
    ));
}

pub(crate) fn statusbar_resize_system(
    window: Query<&Window>,
    mut bg: Query<&mut Transform, (With<StatusBar>, Without<ModeText>)>,
    mut text: Query<&mut Transform, (With<ModeText>, Without<StatusBar>)>,
) {
    let Ok(win) = window.single() else { return; };
    let target_x = -win.width() / 2.0 + STATUSBAR_MARGIN + STATUSBAR_BG_W / 2.0;
    let target_y = -win.height() / 2.0 + STATUSBAR_MARGIN + STATUSBAR_BG_H / 2.0;
    for mut t in &mut bg {
        t.translation.x = target_x;
        t.translation.y = target_y;
    }
    for mut t in &mut text {
        t.translation.x = target_x;
        t.translation.y = target_y;
    }
}
