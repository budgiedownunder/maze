use crate::images::make_image;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Rune glyph — bright circle on a dark background with a cross inside. The
/// circle is filled (lower intensity ~190) and the cross strokes punch
/// through with higher intensity (~245). Material tints bright cyan.
pub(crate) fn make_rune_decoration_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    let cx = W as f32 / 2.0;
    let cy = H as f32 / 2.0;
    let r_outer = 22.0_f32;
    let r_inner = 18.0_f32;
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let in_ring = d >= r_inner && d <= r_outer;
            let in_disk = d < r_inner;
            let on_cross_h = in_disk && (dy.abs() < 2.0);
            let on_cross_v = in_disk && (dx.abs() < 2.0);
            let v: u8 = if on_cross_h || on_cross_v {
                245
            } else if in_ring {
                220
            } else if in_disk {
                40
            } else {
                10
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
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.25, 0.60, 1.10, 1.0),
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(Vec2::new(1.0, 1.0)),
            ..default()
        })
    })
}
