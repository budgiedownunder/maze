use crate::images::make_image;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;

/// Coarse value-noise lattice resolution (cells across the texture). The
/// lattice wraps at this size so the texture tiles seamlessly — adjacent
/// ceiling panels meet without a visible grid seam.
const GRID: u32 = 8;

/// Mid-grey base intensity the noise modulates around. Greyscale, like the
/// other wall textures — the emissive tint in the material carries the colour.
const BASE: f32 = 150.0;
/// How far the smooth blotch noise pushes intensity either side of `BASE`.
const COARSE_AMP: f32 = 70.0;
/// Per-texel grain on top of the blotches, for a rough (not glassy) surface.
const FINE_AMP: f32 = 24.0;
/// Coarse-noise values below this darken into a crevice, giving the rock its
/// cracked / pitted reading rather than smooth mottling.
const CREVICE_THRESHOLD: f32 = 0.30;
/// Maximum darkening (intensity units) at the bottom of a crevice.
const CREVICE_DARKEN: f32 = 80.0;

/// Hashes an integer lattice coordinate to a value in `[0, 1)`.
fn hash(x: u32, y: u32) -> f32 {
    let mut h = x.wrapping_mul(0x9E37_79B1).wrapping_add(y.wrapping_mul(0x85EB_CA77));
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65535.0
}

/// Smoothstep easing for the bilinear interpolation weights.
fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Bilinearly-interpolated value noise on a `GRID`×`GRID` lattice that wraps,
/// sampled at fractional lattice coordinates. Produces the smooth blotches the
/// rock face is built from.
fn value_noise(fx: f32, fy: f32) -> f32 {
    let x0 = fx.floor() as i32;
    let y0 = fy.floor() as i32;
    let tx = smooth(fx - x0 as f32);
    let ty = smooth(fy - y0 as f32);
    let wrap = |v: i32| -> u32 { v.rem_euclid(GRID as i32) as u32 };
    let v00 = hash(wrap(x0), wrap(y0));
    let v10 = hash(wrap(x0 + 1), wrap(y0));
    let v01 = hash(wrap(x0), wrap(y0 + 1));
    let v11 = hash(wrap(x0 + 1), wrap(y0 + 1));
    let a = v00 + (v10 - v00) * tx;
    let b = v01 + (v11 - v01) * tx;
    a + (b - a) * ty
}

/// Builds a greyscale, tileable rock-face texture for the dungeon ceiling.
/// Smooth blotches (coarse value noise) + per-texel grain + dark crevices,
/// all monochrome — the [`crate::world::roof`] material multiplies it by a
/// dark emissive tint so the ceiling reads as dim damp stone.
pub(crate) fn make_rock_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut pixels = vec![255u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let fx = x as f32 / W as f32 * GRID as f32;
            let fy = y as f32 / H as f32 * GRID as f32;
            let coarse = value_noise(fx, fy);
            let fine = hash(x.wrapping_mul(3).wrapping_add(1), y.wrapping_mul(7).wrapping_add(1));
            let mut v = BASE + (coarse - 0.5) * COARSE_AMP + (fine - 0.5) * FINE_AMP;
            if coarse < CREVICE_THRESHOLD {
                v -= CREVICE_DARKEN * (1.0 - coarse / CREVICE_THRESHOLD);
            }
            let v = v.clamp(0.0, 255.0) as u8;
            let idx = ((y * W + x) * 4) as usize;
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}
