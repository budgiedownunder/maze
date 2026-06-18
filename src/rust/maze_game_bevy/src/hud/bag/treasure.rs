//! Procedural treasure icons for the bag HUD — one per [`maze::TreasureStyle`],
//! generated at runtime (no asset files) with their colours baked in (so the
//! bag sprite renders them untinted). Silver and gold are a vertical metal
//! **ingot bar** recessed in a black slot; diamonds is a bright faceted gem;
//! jewels is a gem quartered into four jewel-palette colours.

use crate::images::make_image;
use bevy::prelude::*;
use maze::TreasureStyle;

const ICON_W: u32 = 64;
const ICON_H: u32 = 64;

/// Builds the treasure icon texture for a given style, with colours baked in.
pub(crate) fn make_treasure_icon_texture(
    images: &mut Assets<Image>,
    style: TreasureStyle,
) -> Handle<Image> {
    let pixels = match style {
        TreasureStyle::Silver => bar_in_slot_pixels([0.80, 0.82, 0.88]),
        TreasureStyle::Gold => bar_in_slot_pixels([0.95, 0.78, 0.25]),
        TreasureStyle::Diamonds => gem_pixels(GemFill::Solid([0.90, 0.96, 1.0])),
        TreasureStyle::Jewels => gem_pixels(GemFill::Quartered),
    };
    images.add(make_image(ICON_W, ICON_H, pixels))
}

/// Writes an opaque RGB pixel.
fn put(pixels: &mut [u8], idx: usize, rgb: [f32; 3]) {
    pixels[idx] = (rgb[0] * 255.0) as u8;
    pixels[idx + 1] = (rgb[1] * 255.0) as u8;
    pixels[idx + 2] = (rgb[2] * 255.0) as u8;
    pixels[idx + 3] = 255;
}

/// A vertical metal ingot bar (the given colour) inset within a black
/// rectangle, so the bar reads as recessed in a slot. Everything outside the
/// black rectangle is transparent.
fn bar_in_slot_pixels(bar_rgb: [f32; 3]) -> Vec<u8> {
    // Outer black slot and the inset bar, as inclusive pixel bounds.
    let (slot_x0, slot_x1, slot_y0, slot_y1) = (16.0, 48.0, 8.0, 56.0);
    let (bar_x0, bar_x1, bar_y0, bar_y1) = (26.0, 38.0, 14.0, 50.0);
    let mut pixels = vec![0u8; (ICON_W * ICON_H * 4) as usize];
    for y in 0..ICON_H {
        for x in 0..ICON_W {
            let idx = ((y * ICON_W + x) * 4) as usize;
            let fx = x as f32;
            let fy = y as f32;
            if (bar_x0..=bar_x1).contains(&fx) && (bar_y0..=bar_y1).contains(&fy) {
                put(&mut pixels, idx, bar_rgb);
            } else if (slot_x0..=slot_x1).contains(&fx) && (slot_y0..=slot_y1).contains(&fy) {
                // The recessed black slot around the bar.
                put(&mut pixels, idx, [0.04, 0.04, 0.06]);
            }
            // Otherwise transparent.
        }
    }
    pixels
}

/// How a gem's interior is coloured.
enum GemFill {
    /// One solid colour everywhere.
    Solid([f32; 3]),
    /// Four jewel-palette colours, one per quadrant.
    Quartered,
}

/// Jewel-palette quadrant colours for the `Quartered` gem (top-left clockwise:
/// ruby, emerald, amethyst, sapphire).
const JEWEL_NW: [f32; 3] = [0.85, 0.15, 0.25]; // ruby
const JEWEL_NE: [f32; 3] = [0.15, 0.70, 0.35]; // emerald
const JEWEL_SE: [f32; 3] = [0.65, 0.25, 0.85]; // amethyst
const JEWEL_SW: [f32; 3] = [0.20, 0.40, 0.90]; // sapphire

/// A faceted gem — a rhombus tapering to top and bottom points — filled per
/// `GemFill`. The `Quartered` fill splits it along the centre lines into four
/// jewel-coloured quadrants.
fn gem_pixels(fill: GemFill) -> Vec<u8> {
    let cx = ICON_W as f32 / 2.0;
    let cy = ICON_H as f32 / 2.0;
    let half_w = 24.0;
    let half_h = 28.0;
    let mut pixels = vec![0u8; (ICON_W * ICON_H * 4) as usize];
    for y in 0..ICON_H {
        for x in 0..ICON_W {
            let idx = ((y * ICON_W + x) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            // Rhombus interior: |dx|/half_w + |dy|/half_h <= 1.
            if dx.abs() / half_w + dy.abs() / half_h <= 1.0 {
                let rgb = match fill {
                    GemFill::Solid(c) => c,
                    GemFill::Quartered => match (dx < 0.0, dy < 0.0) {
                        (true, true) => JEWEL_NW,
                        (false, true) => JEWEL_NE,
                        (false, false) => JEWEL_SE,
                        (true, false) => JEWEL_SW,
                    },
                };
                put(&mut pixels, idx, rgb);
            }
        }
    }
    pixels
}
