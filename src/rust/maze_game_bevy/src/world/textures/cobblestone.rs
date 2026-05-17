use crate::images::make_image;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;
/// Grid resolution: the texture is laid out as `CELLS × CELLS` cells,
/// each containing one cobble.
const CELLS: u32 = 4;
const CELL: u32 = W / CELLS;

/// Jitter range applied to cobble centres (texels). `±2` from the
/// nominal cell centre breaks the otherwise-tiled appearance.
const CENTRE_JITTER: i32 = 2;
/// Cobble radius range (texels). Base + 0..=2 of variation.
const RADIUS_BASE: i32 = 5;
const RADIUS_JITTER: u32 = 3;

/// Cobble centre brightness — fades to [`COBBLE_RIM_DARKEN`] below at
/// the cobble rim to fake a rounded shaded disc.
const COBBLE_PEAK_INTENSITY: u32 = 190;
const COBBLE_RIM_DARKEN: u32 = 60;
/// Grout background (gaps between cobbles).
const GROUT_INTENSITY: u8 = 35;

pub(crate) fn make_cobblestone_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let cx = x / CELL;
            let cy = y / CELL;
            // Deterministic jitter in the cobble's centre and radius based on
            // its cell index.
            let h = cx
                .wrapping_mul(73)
                .wrapping_add(cy.wrapping_mul(131))
                .wrapping_add(17);
            let jitter_x = (h % 5) as i32 - CENTRE_JITTER;
            let jitter_y = ((h / 5) % 5) as i32 - CENTRE_JITTER;
            let radius = RADIUS_BASE + ((h / 25) % RADIUS_JITTER) as i32;
            let centre_x = (cx * CELL + CELL / 2) as i32 + jitter_x;
            let centre_y = (cy * CELL + CELL / 2) as i32 + jitter_y;
            let dx = x as i32 - centre_x;
            let dy = y as i32 - centre_y;
            let dist_sq = dx * dx + dy * dy;
            let idx = ((y * W + x) * 4) as usize;
            let v = if dist_sq <= radius * radius {
                let normalised = dist_sq as u32 * COBBLE_RIM_DARKEN / (radius * radius) as u32;
                (COBBLE_PEAK_INTENSITY - normalised.min(COBBLE_RIM_DARKEN)) as u8
            } else {
                GROUT_INTENSITY
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
