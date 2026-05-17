//! Cloud blobs painted into the sky-dome texture.
//!
//! Clouds are part of the dome texture (unlike stars, which are 3D
//! entities — see [`super::stars`]). Each blob is an alpha-blended
//! ellipse centred in the middle band of the upper hemisphere, with a
//! soft radial falloff so the cloud has fluffy edges rather than a hard
//! outline. Placement is seeded so the same sky mode + seed renders
//! the same cloud layout across reloads.

use super::{byte_to_linear, linear_to_byte, next_unit};

// ---------- Tuning constants ----------

/// Per-cloud horizontal radius (texture pixels). Stays in `[BLOB_RX_MIN, BLOB_RX_MIN + BLOB_RX_JITTER)`.
const BLOB_RX_MIN: f32 = 16.0;
/// Random extent added to [`BLOB_RX_MIN`] per cloud.
const BLOB_RX_JITTER: f32 = 18.0;

/// Per-cloud vertical radius is a fraction of its horizontal radius
/// — clouds are wider than they are tall.
const BLOB_RY_RATIO_MIN: f32 = 0.45;
/// Random extent added to [`BLOB_RY_RATIO_MIN`] per cloud.
const BLOB_RY_RATIO_JITTER: f32 = 0.25;
/// Floor on the vertical radius so very flat clouds stay at least
/// this tall in pixels.
const BLOB_RY_MIN: f32 = 6.0;

/// Vertical band (as fractions of texture height, measured from the
/// top) where cloud centres are placed: 0.15..0.45 keeps clouds in the
/// middle of the upper hemisphere so the zenith stays open and the
/// horizon stays clear.
const BAND_TOP: f32 = 0.15;
const BAND_RANGE: f32 = 0.30;

/// Peak per-pixel opacity of a cloud blob at its centre. Above ~0.85
/// the cloud reads as too-solid; below ~0.5 it disappears under the
/// gradient.
const BLOB_PEAK_ALPHA: f32 = 0.85;

/// Exponent on the radial falloff. > 1.0 produces softer edges
/// (alpha drops away faster than linearly from the cloud centre).
const BLOB_FALLOFF_EXPONENT: f32 = 1.5;

/// Seed offset so cloud placement is decorrelated from any other
/// PRNG sequence driven by the same per-mode seed (e.g. stars).
const CLOUD_SEED_OFFSET: u64 = 0xC10D_C10D_C10D_C10D;

// ---------- API ----------

/// Per-mode cloud configuration. `count == 0` is allowed but the
/// per-mode call sites typically pass `None` to skip cloud painting
/// entirely (skipping the gradient-blend loop is cheaper than blending
/// zero clouds).
pub(crate) struct CloudSpec {
    /// How many cloud blobs to draw.
    pub count: u32,
    /// Linear-RGB cloud colour. White/grey for day, dark grey for
    /// backlit sunset clouds, etc.
    pub colour: [f32; 3],
    /// Seed for placement; same seed → same layout.
    pub seed: u64,
}

/// Paints `spec.count` cloud blobs into the texture, alpha-blending
/// against whatever the gradient pass already wrote.
pub(crate) fn paint(pixels: &mut [u8], w: u32, h: u32, spec: &CloudSpec) {
    let mut state = spec.seed.wrapping_add(CLOUD_SEED_OFFSET);
    for _ in 0..spec.count {
        let cx = (next_unit(&mut state) * w as f32) as i32;
        let cy_ratio = BAND_TOP + next_unit(&mut state) * BAND_RANGE;
        let cy = (cy_ratio * h as f32) as i32;
        let rx = BLOB_RX_MIN + next_unit(&mut state) * BLOB_RX_JITTER;
        let ry =
            (rx * (BLOB_RY_RATIO_MIN + next_unit(&mut state) * BLOB_RY_RATIO_JITTER)).max(BLOB_RY_MIN);
        paint_blob(pixels, w, h, cx, cy, rx, ry, spec.colour);
    }
}

#[allow(clippy::too_many_arguments)]
fn paint_blob(
    pixels: &mut [u8],
    w: u32,
    h: u32,
    cx: i32,
    cy: i32,
    rx: f32,
    ry: f32,
    colour: [f32; 3],
) {
    let ymin = (cy as f32 - ry).floor() as i32;
    let ymax = (cy as f32 + ry).ceil() as i32;
    let xmin = (cx as f32 - rx).floor() as i32;
    let xmax = (cx as f32 + rx).ceil() as i32;
    for y in ymin..=ymax {
        if y < 0 || y >= h as i32 {
            continue;
        }
        for x in xmin..=xmax {
            // X wraps around the sphere — clouds at the longitude seam
            // should still render across the join. Y clamps because the
            // sphere has poles, not a vertical wrap.
            let xw = x.rem_euclid(w as i32) as u32;
            let dx = (x - cx) as f32 / rx;
            let dy = (y - cy) as f32 / ry;
            let r2 = dx * dx + dy * dy;
            if r2 > 1.0 {
                continue;
            }
            let alpha = (1.0 - r2.sqrt()).powf(BLOB_FALLOFF_EXPONENT) * BLOB_PEAK_ALPHA;
            let idx = ((y as u32 * w + xw) * 4) as usize;
            let base_r = byte_to_linear(pixels[idx]);
            let base_g = byte_to_linear(pixels[idx + 1]);
            let base_b = byte_to_linear(pixels[idx + 2]);
            let r = base_r * (1.0 - alpha) + colour[0] * alpha;
            let g = base_g * (1.0 - alpha) + colour[1] * alpha;
            let b = base_b * (1.0 - alpha) + colour[2] * alpha;
            pixels[idx] = linear_to_byte(r);
            pixels[idx + 1] = linear_to_byte(g);
            pixels[idx + 2] = linear_to_byte(b);
        }
    }
}
