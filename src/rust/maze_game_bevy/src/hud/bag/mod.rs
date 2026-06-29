//! Bag / inventory HUD.
//!
//! A bottom-of-screen row of grouped item "chips" reflecting the player's
//! holdings (the three top corners are taken by the clock, status bar, and
//! minimap). Each distinct item type renders as one chip — a tinted icon plus a
//! rolling `xN` count: a single key chip for [`maze::MazeGame::bag`] keys, and
//! one chip per collected [`maze::TreasureStyle`] from
//! [`maze::MazeGame::collected_treasure`]. A type whose count is zero is omitted
//! entirely, so the key chip disappears once keys are consumed and a treasure
//! style only appears once collected. The "BAG" label and first chips sit on a
//! top row; when they exceed the window width, overflow chips wrap onto rows
//! below it (the bottom-most row pinned just above the screen edge). The chip
//! set is rebuilt only when it changes; positions track window resizes every
//! frame. Per-type icon textures live in sibling files ([`key`], [`treasure`]).

mod key;
mod treasure;

use crate::state::{GameState, MultiLevelRun};
use bevy::prelude::*;
use maze::{BagItem, TreasureStyle};

const ICON_SIZE: f32 = 28.0;
/// Gap between an icon and its `× N` count text.
const ICON_TEXT_GAP: f32 = 4.0;
/// Horizontal allowance for the `× N` count text within a chip.
const CHIP_TEXT_W: f32 = 30.0;
/// Gap between adjacent chips.
const CHIP_GAP: f32 = 16.0;
const LABEL_W: f32 = 52.0;
/// Gap between the "BAG" label and the first chip.
const LABEL_GAP: f32 = 10.0;
/// Distance of the bottom row's centre line above the bottom screen edge.
const BAG_MARGIN_BOTTOM: f32 = 26.0;
/// Inset from the left/right screen edges for the bag row.
const SIDE_MARGIN: f32 = 20.0;
/// Vertical pitch between wrapped rows (the BAG row on top, overflow below it).
const ROW_PITCH: f32 = ICON_SIZE + 12.0;
const COUNT_FONT_SIZE: f32 = 18.0;

/// Width of a single chip (icon + gap + count text), used for wrap layout.
const CHIP_W: f32 = ICON_SIZE + ICON_TEXT_GAP + CHIP_TEXT_W;

/// Key icon tint — warm gold, matching the floating-key glow.
const COLOR_KEY_ICON: Color = Color::srgb(1.0, 0.82, 0.2);
const COLOR_BAG_LABEL: Color = Color::srgb(0.85, 0.85, 0.92);
const COLOR_COUNT_TEXT: Color = Color::srgb(0.92, 0.92, 0.96);

/// What a bag chip represents — a key group or a per-style treasure group.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ChipKind {
    Key,
    Treasure(TreasureStyle),
}

/// One grouped bag entry: an item type and how many the player holds (always
/// `>= 1` — zero-count types are never emitted).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Chip {
    kind: ChipKind,
    count: u32,
}

/// Builds the grouped chip list from the player's holdings: a single key chip
/// (count = keys still in the bag) followed by one chip per collected treasure
/// style, in the order `collected_treasure` returns. Any type with a zero count
/// is omitted, so the key chip vanishes once keys are consumed to zero and a
/// treasure style only appears once collected.
fn compute_chips(bag: &[BagItem], collected: &[(TreasureStyle, u32)]) -> Vec<Chip> {
    let mut chips = Vec::new();
    let key_count = bag
        .iter()
        .filter(|item| matches!(item, BagItem::Key { .. }))
        .count() as u32;
    if key_count > 0 {
        chips.push(Chip {
            kind: ChipKind::Key,
            count: key_count,
        });
    }
    for &(style, count) in collected {
        if count > 0 {
            chips.push(Chip {
                kind: ChipKind::Treasure(style),
                count,
            });
        }
    }
    chips
}

/// Sprite tint for a chip's icon. The key icon is a white silhouette tinted
/// gold; treasure icons bake their own colours, so they render untinted.
fn chip_color(kind: ChipKind) -> Color {
    match kind {
        ChipKind::Key => COLOR_KEY_ICON,
        ChipKind::Treasure(_) => Color::WHITE,
    }
}

/// Index of a treasure style into the per-style icon-texture array.
fn treasure_style_index(style: TreasureStyle) -> usize {
    match style {
        TreasureStyle::Silver => 0,
        TreasureStyle::Gold => 1,
        TreasureStyle::Diamonds => 2,
        TreasureStyle::Jewels => 3,
    }
}

/// Marker on the persistent "BAG" label entity. Caches the last-rendered chip
/// set so the row is only rebuilt when it changes, and holds the shared
/// item-icon textures.
#[derive(Component)]
pub(crate) struct BagHud {
    last_chips: Vec<Chip>,
    key_icon: Option<Handle<Image>>,
    /// Per-style treasure icons, indexed by [`treasure_style_index`].
    treasure_icons: Option<[Handle<Image>; 4]>,
}

/// Marker on each chip part (icon sprite or its count text), carrying the chip's
/// slot index so per-frame layout can reposition it on resize.
#[derive(Component)]
pub(crate) struct BagChipPart {
    index: usize,
    is_text: bool,
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
    let treasure_icons = images.as_mut().map(|imgs| {
        [
            treasure::make_treasure_icon_texture(imgs, TreasureStyle::Silver),
            treasure::make_treasure_icon_texture(imgs, TreasureStyle::Gold),
            treasure::make_treasure_icon_texture(imgs, TreasureStyle::Diamonds),
            treasure::make_treasure_icon_texture(imgs, TreasureStyle::Jewels),
        ]
    });
    // The label is always present (even with an empty bag); `bag_hud_system`
    // repositions it and (re)builds the chips as holdings change.
    commands.spawn((
        BagHud {
            last_chips: Vec::new(),
            key_icon,
            treasure_icons,
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

/// Per-chip layout result: the icon centre, count-text centre, and row Y.
struct ChipLayout {
    icon_x: f32,
    text_x: f32,
    y: f32,
}

/// Full bag layout: where to place the "BAG" label and each chip. The whole row
/// (label + chips) is centred horizontally on the screen.
struct BagLayout {
    label_x: f32,
    label_y: f32,
    chips: Vec<ChipLayout>,
}

/// Lays the label + chips out as a horizontally **centred** row, wrapping onto
/// further rows when they exceed the usable width. The **top** row carries the
/// "BAG" label and the first chips (so reading order is BAG → cheapest →
/// richest); overflow wraps onto rows **below**, with the bottom-most row pinned
/// just above the bottom screen edge. Each row is centred on its own width.
fn layout_chips(n_chips: usize, win_w: f32, win_h: f32) -> BagLayout {
    let base_y = -win_h / 2.0 + BAG_MARGIN_BOTTOM;
    let max_inner = (win_w - 2.0 * SIDE_MARGIN).max(CHIP_W);
    let chip_pitch = CHIP_W + CHIP_GAP;

    // Greedily assign chips to rows. Row 0 reserves room for the BAG label;
    // wrapped rows use the full usable width. `rows[r]` = chip count on row r.
    let mut rows: Vec<usize> = Vec::new();
    let mut count = 0usize;
    let mut used = LABEL_W + LABEL_GAP; // row 0 begins after the label
    for _ in 0..n_chips {
        let next = if count == 0 {
            used + CHIP_W
        } else {
            used + CHIP_GAP + CHIP_W
        };
        if count > 0 && next > max_inner {
            rows.push(count);
            count = 1;
            used = CHIP_W; // wrapped rows have no label
        } else {
            used = next;
            count += 1;
        }
    }
    rows.push(count); // the final (or only) row, even when it holds no chips

    // Position each row centred on x = 0. Row 0 (with the BAG label) sits on
    // top and the rows descend toward the bottom edge, so the bottom-most row
    // is pinned at `base_y` and earlier rows stack above it.
    let row_total = rows.len();
    let mut chips = Vec::with_capacity(n_chips);
    let mut label_x = -LABEL_W / 2.0; // used when row 0 carries no chips
    let mut label_y = base_y;
    for (row, &k) in rows.iter().enumerate() {
        let has_label = row == 0;
        let chips_w = if k == 0 {
            0.0
        } else {
            k as f32 * CHIP_W + (k as f32 - 1.0) * CHIP_GAP
        };
        let row_w = if has_label {
            LABEL_W + if k > 0 { LABEL_GAP } else { 0.0 } + chips_w
        } else {
            chips_w
        };
        let row_left = -row_w / 2.0;
        let y = base_y + (row_total - 1 - row) as f32 * ROW_PITCH;
        let mut cursor = if has_label {
            label_x = row_left + LABEL_W / 2.0;
            label_y = y;
            row_left + LABEL_W + LABEL_GAP
        } else {
            row_left
        };
        for _ in 0..k {
            chips.push(ChipLayout {
                icon_x: cursor + ICON_SIZE / 2.0,
                text_x: cursor + ICON_SIZE + ICON_TEXT_GAP + CHIP_TEXT_W / 2.0,
                y,
            });
            cursor += chip_pitch;
        }
    }
    BagLayout {
        label_x,
        label_y,
        chips,
    }
}

/// Y coordinate of the top edge of the bag HUD's highest row. Other bottom HUD
/// rows (e.g. HP) stack above this so they never collide with the bag — when a
/// narrow window wraps the bag onto extra rows, the reported top edge rises
/// accordingly.
pub(crate) fn top_edge_y(
    game: &maze::MazeGame,
    treasure: &[(TreasureStyle, u32)],
    win_w: f32,
    win_h: f32,
) -> f32 {
    let chips = compute_chips(game.bag(), treasure);
    let layout = layout_chips(chips.len(), win_w, win_h);
    let base_y = -win_h / 2.0 + BAG_MARGIN_BOTTOM;
    let top_row_y = layout.chips.iter().map(|c| c.y).fold(base_y, f32::max);
    top_row_y + ICON_SIZE / 2.0
}

pub(crate) fn bag_hud_system(
    mut commands: Commands,
    window: Query<&Window>,
    state: Res<GameState>,
    run: Res<MultiLevelRun>,
    mut label: Query<(&mut BagHud, &mut Transform), Without<BagChipPart>>,
    mut parts: Query<(Entity, &BagChipPart, &mut Transform), Without<BagHud>>,
) {
    let Ok(win) = window.single() else {
        return;
    };
    let Ok((mut hud, mut label_transform)) = label.single_mut() else {
        return;
    };

    // Treasure chips show the whole run's tally (banked + live), so they keep
    // accumulating across level transitions; keys come from the live bag (which
    // carries forward when `reset_bag_between_levels` is false).
    let treasure = run.cumulative_treasure(&state.game.collected_treasure());
    let chips = compute_chips(state.game.bag(), &treasure);
    let layout = layout_chips(chips.len(), win.width(), win.height());

    // Centre the label with the chips each frame so resizes re-centre the row.
    label_transform.translation.x = layout.label_x;
    label_transform.translation.y = layout.label_y;

    if hud.last_chips != chips {
        // Holdings changed — rebuild the chip parts from scratch (rare: only on
        // pickup or key consumption). Each part is spawned at its computed
        // position so it appears correctly placed on its first frame (the
        // per-frame resize pass below won't see these new entities until next
        // frame, as the spawn commands are deferred).
        let key_icon = hud.key_icon.clone();
        let treasure_icons = hud.treasure_icons.clone();
        for (entity, _, _) in &parts {
            commands.entity(entity).despawn();
        }
        for (index, chip) in chips.iter().enumerate() {
            let icon_handle = match chip.kind {
                ChipKind::Key => key_icon.clone(),
                ChipKind::Treasure(style) => treasure_icons
                    .as_ref()
                    .map(|icons| icons[treasure_style_index(style)].clone()),
            };
            let mut sprite = Sprite {
                color: chip_color(chip.kind),
                custom_size: Some(Vec2::splat(ICON_SIZE)),
                ..default()
            };
            // Without the texture (e.g. headless tests) the sprite falls back to
            // a solid tinted square.
            if let Some(image) = icon_handle {
                sprite.image = image;
            }
            let pos = &layout.chips[index];
            commands.spawn((
                BagChipPart {
                    index,
                    is_text: false,
                },
                sprite,
                Transform::from_xyz(pos.icon_x, pos.y, 9.0),
            ));
            commands.spawn((
                BagChipPart {
                    index,
                    is_text: true,
                },
                // ASCII 'x' (not the × multiplication sign) — the default font
                // has no glyph for U+00D7 and would draw a missing-glyph box.
                Text2d::new(format!("x{}", chip.count)),
                TextFont {
                    font_size: COUNT_FONT_SIZE,
                    ..default()
                },
                TextColor(COLOR_COUNT_TEXT),
                Transform::from_xyz(pos.text_x, pos.y, 9.0),
            ));
        }
        hud.last_chips = chips;
    }

    // Reposition every chip part each frame so window resizes track the bottom
    // edge and re-flow the wrap.
    for (_, part, mut transform) in &mut parts {
        if let Some(pos) = layout.chips.get(part.index) {
            transform.translation.x = if part.is_text { pos.text_x } else { pos.icon_x };
            transform.translation.y = pos.y;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_chips_omits_zero_counts_and_orders_keys_first() {
        let bag = vec![BagItem::Key { id: 0 }, BagItem::Key { id: 1 }];
        let collected = vec![(TreasureStyle::Silver, 3), (TreasureStyle::Gold, 1)];
        let chips = compute_chips(&bag, &collected);
        assert_eq!(chips.len(), 3);
        assert_eq!(chips[0].kind, ChipKind::Key);
        assert_eq!(chips[0].count, 2);
        assert_eq!(chips[1].kind, ChipKind::Treasure(TreasureStyle::Silver));
        assert_eq!(chips[1].count, 3);
        assert_eq!(chips[2].kind, ChipKind::Treasure(TreasureStyle::Gold));
        assert_eq!(chips[2].count, 1);
    }

    #[test]
    fn compute_chips_drops_the_key_chip_when_no_keys_are_held() {
        // An empty bag (all keys consumed) emits no key chip — only treasure.
        let chips = compute_chips(&[], &[(TreasureStyle::Diamonds, 1)]);
        assert_eq!(chips.len(), 1);
        assert_eq!(chips[0].kind, ChipKind::Treasure(TreasureStyle::Diamonds));
    }

    #[test]
    fn compute_chips_is_empty_with_nothing_held() {
        assert!(compute_chips(&[], &[]).is_empty());
    }

    #[test]
    fn layout_wraps_onto_a_row_below_when_out_of_width() {
        // A narrow window forces the chips onto stacked rows; the BAG label and
        // first chips sit on the top row, and overflow wraps onto rows below, so
        // later chips have a smaller Y than the first.
        let layout = layout_chips(4, 320.0, 600.0);
        assert_eq!(layout.chips.len(), 4);
        assert!(
            layout.chips.last().unwrap().y < layout.chips.first().unwrap().y,
            "overflow chips should wrap onto rows below the BAG row"
        );
        // The BAG label shares the top row with the first chip.
        assert_eq!(layout.label_y, layout.chips.first().unwrap().y);
    }

    #[test]
    fn layout_centres_the_single_row_on_screen() {
        // One key chip on a wide screen: the label sits left of centre and the
        // chip right of it, so the label + chip span is centred about x = 0.
        let layout = layout_chips(1, 1280.0, 720.0);
        assert_eq!(layout.chips.len(), 1);
        let chip_right = layout.chips[0].icon_x + ICON_SIZE / 2.0 + ICON_TEXT_GAP + CHIP_TEXT_W;
        let label_left = layout.label_x - LABEL_W / 2.0;
        // The row's left and right extents are mirror images about x = 0.
        assert!(
            (label_left + chip_right).abs() < 1.0,
            "row should be centred: label_left={label_left}, chip_right={chip_right}"
        );
    }
}
