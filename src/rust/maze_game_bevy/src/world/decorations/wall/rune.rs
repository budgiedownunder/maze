use crate::images::make_image;
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::math::Affine2;
use bevy::prelude::*;

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;
/// Outer / inner radii of the rune ring (texels).
const R_OUTER: f32 = 22.0;
const R_INNER: f32 = 18.0;
/// Half-width of the cross strokes inside the disc (texels).
const CROSS_HALF_WIDTH: f32 = 2.0;

/// Brightness on the cross strokes (brightest part of the glyph).
const CROSS_INTENSITY: u8 = 245;
/// Brightness on the outer ring.
const RING_INTENSITY: u8 = 220;
/// Brightness inside the disc (darker than ring, sets off the cross).
const DISC_INTENSITY: u8 = 40;
/// Background brightness (outside the ring).
const BACKGROUND_INTENSITY: u8 = 10;

/// Rune emissive RGB — rich gold.
const RUNE_EMISSIVE: LinearRgba = LinearRgba::new(1.00, 0.75, 0.10, 1.0);
const RUNE_UV: Vec2 = Vec2::new(1.0, 1.0);

/// Rune glyph — bright circle on a dark background with a cross inside. The
/// circle is filled (lower intensity ~190) and the cross strokes punch
/// through with higher intensity (~245). Material tints rich gold.
pub(crate) fn make_rune_decoration_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let cx = W as f32 / 2.0;
    let cy = H as f32 / 2.0;
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let in_ring = (R_INNER..=R_OUTER).contains(&d);
            let in_disk = d < R_INNER;
            let on_cross_h = in_disk && (dy.abs() < CROSS_HALF_WIDTH);
            let on_cross_v = in_disk && (dx.abs() < CROSS_HALF_WIDTH);
            let v: u8 = if on_cross_h || on_cross_v {
                CROSS_INTENSITY
            } else if in_ring {
                RING_INTENSITY
            } else if in_disk {
                DISC_INTENSITY
            } else {
                BACKGROUND_INTENSITY
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_rune_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images.as_mut().map(|imgs| make_rune_decoration_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: RUNE_EMISSIVE,
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(RUNE_UV),
            ..default()
        })
    })
}
