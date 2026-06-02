//! Procedural key icon for the bag HUD — an opaque white key silhouette on a
//! transparent background, generated at runtime (no asset file). The bag sprite
//! tints it gold; the transparent background lets only the key shape show.

use crate::images::make_image;
use bevy::prelude::*;

const ICON_W: u32 = 64;
const ICON_H: u32 = 64;

/// Bow centre Y, outer radius, and the radius of the hole hollowed out of it.
const BOW_CY: f32 = 17.0;
const BOW_OUTER: f32 = 15.0;
const BOW_HOLE: f32 = 6.0;

/// Builds the key icon texture: a round bow (a solid disc with a circular hole),
/// a shaft, and two teeth in opaque white, everything else fully transparent.
pub(crate) fn make_key_icon_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let cx = ICON_W as f32 / 2.0;
    let mut pixels = vec![0u8; (ICON_W * ICON_H * 4) as usize];
    for y in 0..ICON_H {
        for x in 0..ICON_W {
            let idx = ((y * ICON_W + x) * 4) as usize;
            let fx = x as f32;
            let fy = y as f32;
            // Bow: a solid round head near the top with a circular hole bored
            // through it (an annulus), so it reads as a key's loop at icon size.
            let d = ((fx - cx).powi(2) + (fy - BOW_CY).powi(2)).sqrt();
            let in_bow = (BOW_HOLE..=BOW_OUTER).contains(&d);
            // Shaft: a vertical bar down the centre; starts below the hole so it
            // meets the bow's solid lower rim without filling the hole.
            let in_shaft = (fx - cx).abs() <= 3.5 && (24.0..=56.0).contains(&fy);
            // Two teeth jutting to one side near the bottom (the bit).
            let in_tooth_long = (cx + 3.5..=46.0).contains(&fx) && (44.0..=49.0).contains(&fy);
            let in_tooth_short = (cx + 3.5..=43.0).contains(&fx) && (52.0..=56.0).contains(&fy);
            if in_bow || in_shaft || in_tooth_long || in_tooth_short {
                pixels[idx] = 255;
                pixels[idx + 1] = 255;
                pixels[idx + 2] = 255;
                pixels[idx + 3] = 255;
            }
            // Otherwise the pixel stays the transparent (0, 0, 0, 0) background.
        }
    }
    images.add(make_image(ICON_W, ICON_H, pixels))
}
