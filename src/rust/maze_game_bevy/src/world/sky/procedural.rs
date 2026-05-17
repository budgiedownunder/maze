//! Procedural sky-dome backdrop (gradient).
//!
//! Bakes a 512×256 equirectangular RGBA texture containing a vertical
//! gradient from zenith → horizon → nadir. The orchestrator also
//! invites the [`super::clouds`] painter to alpha-blend cloud blobs on
//! top before returning the handle.
//!
//! Stars are NOT painted into this texture — at 512×256 each texel
//! magnifies to ~10 screen pixels on the dome, which makes painted
//! stars read as obvious blocks. They are spawned as small 3D entities
//! instead; see [`super::stars`].
//!
//! The resulting [`Handle<Image>`] is consumed by
//! [`super::dome::spawn_dome`] and rendered unlit on the inverted-sphere
//! dome that surrounds the player.

use super::clouds::CloudSpec;
use super::linear_to_byte;
use crate::images::make_image;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Texture width in pixels (longitude resolution).
const W: u32 = 512;
/// Texture height in pixels (latitude resolution).
const H: u32 = 256;

// ---------- API ----------

/// Per-mode gradient parameters. Linear-space RGB triples `[r, g, b]`
/// in `0.0..=1.0`. Three control points (zenith / horizon / nadir) are
/// linearly interpolated based on a pixel's latitude.
pub(crate) struct SkySpec {
    /// Colour at the top of the dome (looking straight up).
    pub zenith: [f32; 3],
    /// Colour at the equator (horizon line).
    pub horizon: [f32; 3],
    /// Colour at the bottom of the dome (below the player). Mostly
    /// hidden by the floor; kept neutral so any gap renders sensibly.
    pub nadir: [f32; 3],
}

/// Builds the dome texture by painting the gradient then (optionally)
/// alpha-blending clouds on top.
pub(crate) fn make_sky_texture(
    images: &mut Assets<Image>,
    sky: &SkySpec,
    clouds: Option<&CloudSpec>,
) -> Handle<Image> {
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    paint_gradient(&mut pixels, sky);
    if let Some(spec) = clouds {
        super::clouds::paint(&mut pixels, W, H, spec);
    }
    images.add(make_image(W, H, pixels))
}

// ---------- Internals ----------

fn paint_gradient(pixels: &mut [u8], spec: &SkySpec) {
    for y in 0..H {
        // v = 1.0 at the top (zenith), 0.0 at the bottom (nadir).
        let v = 1.0 - (y as f32 + 0.5) / (H as f32);
        let (r, g, b) = if v >= 0.5 {
            let t = (v - 0.5) * 2.0;
            lerp_rgb(spec.horizon, spec.zenith, t)
        } else {
            let t = (0.5 - v) * 2.0;
            lerp_rgb(spec.horizon, spec.nadir, t)
        };
        let (r, g, b) = (linear_to_byte(r), linear_to_byte(g), linear_to_byte(b));
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255;
        }
    }
}

fn lerp_rgb(a: [f32; 3], b: [f32; 3], t: f32) -> (f32, f32, f32) {
    let t = t.clamp(0.0, 1.0);
    (
        a[0] * (1.0 - t) + b[0] * t,
        a[1] * (1.0 - t) + b[1] * t,
        a[2] * (1.0 - t) + b[2] * t,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::sky::next_unit;

    #[test]
    fn next_unit_in_range() {
        let mut s = 12345u64;
        for _ in 0..1000 {
            let v = next_unit(&mut s);
            assert!((0.0..1.0).contains(&v), "got {v}");
        }
    }

    #[test]
    fn next_unit_is_deterministic_for_same_seed() {
        let mut s1 = 0xDEAD_BEEFu64;
        let mut s2 = 0xDEAD_BEEFu64;
        for _ in 0..20 {
            assert_eq!(next_unit(&mut s1), next_unit(&mut s2));
        }
    }

    #[test]
    fn lerp_clamps_t_outside_unit_range() {
        let (r, g, b) = lerp_rgb([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], 2.0);
        assert_eq!((r, g, b), (0.0, 1.0, 0.0));
        let (r, g, b) = lerp_rgb([1.0, 0.0, 0.0], [0.0, 1.0, 0.0], -0.5);
        assert_eq!((r, g, b), (1.0, 0.0, 0.0));
    }
}
