use crate::images::make_image;
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::math::Affine2;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;
/// Dark frame width (texels).
const FRAME: u32 = 4;

/// Frame brightness (8-bit greyscale).
const FRAME_INTENSITY: u8 = 25;
/// "Tear" brightness — horizontal lines that simulate paper damage.
const TEAR_INTENSITY: u8 = 90;
/// Top brightness of the fade gradient.
const POSTER_TOP_INTENSITY: f32 = 200.0;
/// Brightness drop from top to bottom of the gradient (so bottom ends
/// at `POSTER_TOP_INTENSITY - POSTER_FADE_RANGE`).
const POSTER_FADE_RANGE: f32 = 90.0;

/// Poster emissive RGB — warm orange.
const POSTER_EMISSIVE: LinearRgba = LinearRgba::new(0.65, 0.40, 0.18, 1.0);
const POSTER_UV: Vec2 = Vec2::new(1.0, 1.0);

/// Faded poster — bright at the top fading to mid at the bottom, with a dark
/// frame and a few horizontal "tears" for character. Material tints warm
/// orange.
pub(crate) fn make_poster_decoration_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let frame = !(FRAME..W - FRAME).contains(&x) || !(FRAME..H - FRAME).contains(&y);
            let in_tear = matches!(y, 18 | 19 | 36 | 37 | 48);
            let v: u8 = if frame {
                FRAME_INTENSITY
            } else if in_tear {
                TEAR_INTENSITY
            } else {
                let t = y as f32 / H as f32;
                (POSTER_TOP_INTENSITY - t * POSTER_FADE_RANGE) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_poster_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images.as_mut().map(|imgs| make_poster_decoration_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: POSTER_EMISSIVE,
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(POSTER_UV),
            ..default()
        })
    })
}
