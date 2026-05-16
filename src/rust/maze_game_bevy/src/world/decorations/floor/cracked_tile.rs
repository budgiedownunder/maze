use crate::images::make_image;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Cracked tile — stone-grey base with thin radial cracks emanating from a
/// central impact point. Material tints cool grey.
pub(crate) fn make_cracked_tile_floor_accent_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    let cx = W as f32 / 2.0;
    let cy = H as f32 / 2.0;
    // Six straight cracks radiating from the centre at fixed angles.
    let crack_angles: [f32; 6] = [
        0.0,
        std::f32::consts::FRAC_PI_3,
        std::f32::consts::FRAC_PI_3 * 2.0,
        std::f32::consts::PI,
        std::f32::consts::PI + std::f32::consts::FRAC_PI_3,
        std::f32::consts::PI + std::f32::consts::FRAC_PI_3 * 2.0,
    ];
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            // Distance to nearest crack line (projection of (dx,dy) onto a unit
            // vector perpendicular to each crack direction).
            let mut nearest = f32::INFINITY;
            for a in crack_angles {
                // Perpendicular distance to the crack line through the centre at angle `a`.
                let perp = (dx * -a.sin() + dy * a.cos()).abs();
                // Only count the half-ray pointing in direction `a`, not the other side.
                let along = dx * a.cos() + dy * a.sin();
                if along >= 0.0 {
                    nearest = nearest.min(perp);
                }
            }
            // Cracks are 1 px wide, brighter near the centre (less worn).
            let on_crack = nearest < 1.2 && dist < 28.0;
            let v: u8 = if on_crack {
                (180.0 - dist * 2.0).clamp(60.0, 220.0) as u8
            } else {
                // Base tile: gentle brightness variation around 140.
                let n =
                    (((x.wrapping_mul(13)).wrapping_add(y.wrapping_mul(7))) % 25) as i32 - 12;
                (140 + n).clamp(0, 255) as u8
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_cracked_tile_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images
        .as_mut()
        .map(|imgs| make_cracked_tile_floor_accent_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.50, 0.50, 0.48, 1.0),
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(Vec2::new(1.0, 1.0)),
            ..default()
        })
    })
}
