//! Bag / inventory HUD.
//!
//! A bottom-of-screen row of item icons reflecting [`maze::MazeGame::bag`]
//! (the three top corners are taken by the clock, status bar, and minimap).
//! The row updates live in both directions — it grows when a key is picked up
//! and shrinks when one is consumed opening a door — by rebuilding the icon
//! entities whenever the bag length changes. Each icon's image + colour is
//! dispatched per [`maze::BagItem`] variant. Mirrors the world-space
//! `Text2d`/`Sprite` pattern of [`crate::hud::clock`]. The per-variant icon
//! textures live in sibling files (e.g. [`key`]).

mod key;

use crate::state::GameState;
use bevy::prelude::*;
use maze::BagItem;

const ICON_SIZE: f32 = 28.0;
const ICON_GAP: f32 = 8.0;
const LABEL_W: f32 = 52.0;
/// Gap between the "BAG" label and the first icon.
const LABEL_GAP: f32 = 10.0;
/// Distance of the row's centre line above the bottom screen edge.
const BAG_MARGIN_BOTTOM: f32 = 26.0;

/// Key icon tint — warm gold, matching the floating-key glow.
const COLOR_KEY_ICON: Color = Color::srgb(1.0, 0.82, 0.2);
const COLOR_BAG_LABEL: Color = Color::srgb(0.85, 0.85, 0.92);

/// Marker on the persistent "BAG" label entity. Tracks the last-rendered bag
/// length so the icon row is only rebuilt when the bag actually changes, and
/// holds the shared item-icon textures.
#[derive(Component)]
pub(crate) struct BagHud {
    last_len: usize,
    key_icon: Option<Handle<Image>>,
}

/// Marker on each item icon, carrying its slot index for row layout.
#[derive(Component)]
pub(crate) struct BagItemIcon {
    index: usize,
}

pub(crate) fn spawn_bag_hud(
    commands: &mut Commands,
    window: &Query<&Window>,
    images: &mut Option<ResMut<Assets<Image>>>,
) {
    let y = window
        .single()
        .map(|w| -w.height() / 2.0 + BAG_MARGIN_BOTTOM)
        .unwrap_or(-330.0);
    // Item-icon textures are built once here (alongside the world's other
    // procedural textures) and stored on the HUD for `bag_hud_system` to reuse.
    let key_icon = images.as_mut().map(|imgs| key::make_key_icon_texture(imgs));
    // The label is always present (even with an empty bag); `bag_hud_system`
    // repositions it and (re)builds the icons as the bag changes. Start
    // `last_len` at MAX so the first frame always builds the initial row.
    commands.spawn((
        BagHud {
            last_len: usize::MAX,
            key_icon,
        },
        Text2d::new("BAG"),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(COLOR_BAG_LABEL),
        Transform::from_xyz(0.0, y, 9.0),
    ));
}

/// Tint colour for a bag item's icon, dispatched per variant.
fn icon_color(item: &BagItem) -> Color {
    match item {
        BagItem::Key { .. } => COLOR_KEY_ICON,
    }
}

/// X centre of slot `index` given the row's left edge and the icon pitch.
fn icon_x(row_left: f32, index: usize) -> f32 {
    row_left + LABEL_W + LABEL_GAP + ICON_SIZE / 2.0 + index as f32 * (ICON_SIZE + ICON_GAP)
}

pub(crate) fn bag_hud_system(
    mut commands: Commands,
    window: Query<&Window>,
    state: Res<GameState>,
    mut label: Query<(&mut BagHud, &mut Transform), Without<BagItemIcon>>,
    mut icons: Query<(Entity, &BagItemIcon, &mut Transform), Without<BagHud>>,
) {
    let Ok(win) = window.single() else {
        return;
    };
    let Ok((mut hud, mut label_transform)) = label.single_mut() else {
        return;
    };

    let bag = state.game.bag();
    let n = bag.len();

    // Whole row (label + icons) is centred horizontally; compute its left edge.
    let icons_w = if n == 0 {
        0.0
    } else {
        LABEL_GAP + n as f32 * ICON_SIZE + (n - 1) as f32 * ICON_GAP
    };
    let row_w = LABEL_W + icons_w;
    let row_left = -row_w / 2.0;
    let y = -win.height() / 2.0 + BAG_MARGIN_BOTTOM;

    // Reposition the label each frame so window resizes track the bottom edge.
    label_transform.translation.x = row_left + LABEL_W / 2.0;
    label_transform.translation.y = y;

    if hud.last_len != n {
        // Bag changed — rebuild the icon row from scratch (rare: only on pickup
        // or door consumption). New icons are positioned at spawn time below.
        let key_icon = hud.key_icon.clone();
        for (entity, _, _) in &icons {
            commands.entity(entity).despawn();
        }
        for (index, item) in bag.iter().enumerate() {
            let mut sprite = Sprite {
                color: icon_color(item),
                custom_size: Some(Vec2::splat(ICON_SIZE)),
                ..default()
            };
            // A key renders as the key icon tinted gold; without the texture
            // (e.g. headless tests) the sprite falls back to a solid square.
            if let (BagItem::Key { .. }, Some(image)) = (item, key_icon.clone()) {
                sprite.image = image;
            }
            commands.spawn((
                BagItemIcon { index },
                sprite,
                Transform::from_xyz(icon_x(row_left, index), y, 9.0),
            ));
        }
        hud.last_len = n;
    } else {
        // Unchanged count — just track the bottom edge on resize.
        for (_, icon, mut transform) in &mut icons {
            transform.translation.x = icon_x(row_left, icon.index);
            transform.translation.y = y;
        }
    }
}
