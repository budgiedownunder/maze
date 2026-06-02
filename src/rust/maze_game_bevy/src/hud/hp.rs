//! Life HUD.
//!
//! A bottom-of-screen row of heart icons reflecting
//! [`maze::MazeGame::hp`] / [`maze::MazeGame::max_hp`]. The row is built
//! once at spawn time (sized by `max_hp`, which doesn't change during a
//! session) and updated each frame: hearts at indices `< hp` render
//! filled-red, those at indices `>= hp` render empty / dim. Stacked
//! directly above [`crate::hud::bag`] so the two player-state rows live
//! together at the bottom of the screen.
//!
//! The heart sprite uses a procedurally-generated white heart silhouette
//! texture (built once at spawn) tinted per slot via the sprite colour,
//! so a single texture serves both the filled-red and empty-dim states.

use crate::hud::bag::BagHud;
use crate::images::make_image;
use crate::state::GameState;
use bevy::prelude::*;

const HEART_SIZE: f32 = 24.0;
const HEART_GAP: f32 = 6.0;
/// Reserved label width — sized for "LIFE" (4 letters at the 20 px font),
/// matching the bag row's `LABEL_W` for visual consistency.
const LABEL_W: f32 = 52.0;
const LABEL_GAP: f32 = 10.0;
/// Distance of the row's centre line above the bottom screen edge — sits
/// above the bag row (which uses `BAG_MARGIN_BOTTOM = 26`).
const HP_MARGIN_BOTTOM: f32 = 64.0;

/// Heart colour when the slot is currently filled (player has at least
/// this many HP).
const COLOR_HEART_FILLED: Color = Color::srgb(1.0, 0.18, 0.18);
/// Heart colour when the slot is empty (HP below this index).
const COLOR_HEART_EMPTY: Color = Color::srgb(0.25, 0.25, 0.28);
const COLOR_HP_LABEL: Color = Color::srgb(0.85, 0.85, 0.92);

const HEART_ICON_PX: u32 = 64;

/// Marker on the persistent "LIFE" label entity. Tracks the player's
/// HP from the previous frame so we only restyle heart icons when their
/// filled/empty state actually changes.
#[derive(Component)]
pub(crate) struct HpHud {
    last_hp: u32,
    last_max_hp: u32,
}

/// Marker on each heart icon, carrying its slot index for row layout +
/// fill-state restyles.
#[derive(Component)]
pub(crate) struct HpHeartIcon {
    pub(crate) index: u32,
}

pub(crate) fn spawn_hp_hud(
    commands: &mut Commands,
    window: &Query<&Window>,
    images: &mut Option<ResMut<Assets<Image>>>,
    max_hp: u32,
    starting_hp: u32,
) {
    let y = window
        .single()
        .map(|w| -w.height() / 2.0 + HP_MARGIN_BOTTOM)
        .unwrap_or(-292.0);
    let row_left = row_left_for(max_hp);
    let heart_icon = images.as_mut().map(|imgs| make_heart_icon_texture(imgs));
    commands.spawn((
        HpHud {
            last_hp: starting_hp,
            last_max_hp: max_hp,
        },
        Text2d::new("LIFE"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(COLOR_HP_LABEL),
        Transform::from_xyz(row_left + LABEL_W / 2.0, y, 9.0),
    ));
    for index in 0..max_hp {
        let color = if index < starting_hp {
            COLOR_HEART_FILLED
        } else {
            COLOR_HEART_EMPTY
        };
        let mut sprite = Sprite {
            color,
            custom_size: Some(Vec2::splat(HEART_SIZE)),
            ..default()
        };
        if let Some(image) = heart_icon.clone() {
            sprite.image = image;
        }
        commands.spawn((
            HpHeartIcon { index },
            sprite,
            Transform::from_xyz(heart_x(row_left, index), y, 9.0),
        ));
    }
}

/// Builds a white heart silhouette on a transparent background — the
/// sprite's per-slot tint paints it red (filled) or grey (empty) without
/// needing two textures. Uses the standard implicit heart curve so the
/// shape reads cleanly at icon size.
fn make_heart_icon_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let size = HEART_ICON_PX as f32;
    let mut pixels = vec![0u8; (HEART_ICON_PX * HEART_ICON_PX * 4) as usize];
    for y in 0..HEART_ICON_PX {
        for x in 0..HEART_ICON_PX {
            // Map pixel into the unit square [-1, 1] used by the heart
            // implicit equation. Inset the curve slightly so the lobes
            // and tip don't kiss the texture's edge.
            let scale = 1.15;
            let u = scale * (2.0 * x as f32 / size - 1.0);
            let v = scale * (1.0 - 2.0 * y as f32 / size);
            // Standard heart curve (lobes at +v, tip at -v):
            //   (u² + v² − 1)³ − u²·v³ ≤ 0
            let r2 = u * u + v * v - 1.0;
            let inside = r2 * r2 * r2 - u * u * v * v * v <= 0.0;
            if inside {
                let idx = ((y * HEART_ICON_PX + x) * 4) as usize;
                pixels[idx] = 255;
                pixels[idx + 1] = 255;
                pixels[idx + 2] = 255;
                pixels[idx + 3] = 255;
            }
        }
    }
    images.add(make_image(HEART_ICON_PX, HEART_ICON_PX, pixels))
}

/// X centre of the heart slot at `index` given the row's left edge.
fn heart_x(row_left: f32, index: u32) -> f32 {
    row_left + LABEL_W + LABEL_GAP + HEART_SIZE / 2.0 + index as f32 * (HEART_SIZE + HEART_GAP)
}

/// Computes the X coordinate of the row's left edge for a row that
/// renders `max_hp` heart icons. Centres the whole label + row block on
/// the screen horizontally.
fn row_left_for(max_hp: u32) -> f32 {
    let n = max_hp as f32;
    let icons_w = if max_hp == 0 {
        0.0
    } else {
        LABEL_GAP + n * HEART_SIZE + (n - 1.0) * HEART_GAP
    };
    let row_w = LABEL_W + icons_w;
    -row_w / 2.0
}

#[allow(clippy::type_complexity)]
pub(crate) fn hp_hud_system(
    window: Query<&Window>,
    state: Res<GameState>,
    mut label: Query<
        (&mut HpHud, &mut Transform),
        (Without<HpHeartIcon>, Without<BagHud>),
    >,
    mut hearts: Query<(&HpHeartIcon, &mut Sprite, &mut Transform), Without<HpHud>>,
) {
    let Ok(win) = window.single() else {
        return;
    };
    let Ok((mut hud, mut label_transform)) = label.single_mut() else {
        return;
    };

    let hp = state.game.hp();
    let max_hp = state.game.max_hp();
    let row_left = row_left_for(max_hp);
    let y = -win.height() / 2.0 + HP_MARGIN_BOTTOM;

    // Reposition the label each frame so window resizes track the bottom
    // edge (mirrors the bag HUD's behaviour).
    label_transform.translation.x = row_left + LABEL_W / 2.0;
    label_transform.translation.y = y;

    let hp_changed = hud.last_hp != hp;
    for (icon, mut sprite, mut transform) in hearts.iter_mut() {
        if hp_changed {
            sprite.color = if icon.index < hp {
                COLOR_HEART_FILLED
            } else {
                COLOR_HEART_EMPTY
            };
        }
        transform.translation.x = heart_x(row_left, icon.index);
        transform.translation.y = y;
    }
    hud.last_hp = hp;
    // max_hp doesn't change mid-session, but stash it anyway so future
    // dynamic-max-hp gameplay (e.g. permanent +1 max from a relic) drops
    // in without rewiring the field.
    hud.last_max_hp = max_hp;
}
