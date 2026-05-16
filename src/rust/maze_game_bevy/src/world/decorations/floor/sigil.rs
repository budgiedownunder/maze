use crate::images::make_image;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Sigil — ornamental glyph: an outer ring with an inscribed 5-point star.
/// Material tints arcane purple — distinct from rune (gold) and glowing
/// glass (amber) so the four floor-accent kinds remain visually distinct.
pub(crate) fn make_sigil_floor_accent_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    let cx = W as f32 / 2.0;
    let cy = H as f32 / 2.0;
    let r_outer = 26.0;
    let r_inner = 23.0;
    // Five points of a pentagram on a circle of radius r_pent.
    let r_pent = 22.0;
    let pent_pts: [(f32, f32); 5] = std::array::from_fn(|i| {
        // -PI/2 puts the first point at the top.
        let a = -std::f32::consts::FRAC_PI_2 + (i as f32) * std::f32::consts::TAU / 5.0;
        (cx + r_pent * a.cos(), cy + r_pent * a.sin())
    });
    // The pentagram strokes connect vertex i to vertex (i+2) % 5.
    let strokes: [((f32, f32), (f32, f32)); 5] =
        std::array::from_fn(|i| (pent_pts[i], pent_pts[(i + 2) % 5]));
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let on_ring = d >= r_inner && d <= r_outer;
            // Distance to the nearest pentagram stroke (line segment).
            let mut min_stroke_dist = f32::INFINITY;
            for ((ax, ay), (bx, by)) in strokes {
                let abx = bx - ax;
                let aby = by - ay;
                let apx = x as f32 - ax;
                let apy = y as f32 - ay;
                let ab_len_sq = abx * abx + aby * aby;
                let t = ((apx * abx + apy * aby) / ab_len_sq).clamp(0.0, 1.0);
                let projx = ax + t * abx;
                let projy = ay + t * aby;
                let stroke_d = ((x as f32 - projx).powi(2) + (y as f32 - projy).powi(2)).sqrt();
                if stroke_d < min_stroke_dist {
                    min_stroke_dist = stroke_d;
                }
            }
            let on_stroke = min_stroke_dist < 1.4 && d < r_pent + 1.0;
            let v: u8 = if on_stroke {
                235
            } else if on_ring {
                200
            } else if d < r_inner {
                30
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

pub(crate) fn build_sigil_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images
        .as_mut()
        .map(|imgs| make_sigil_floor_accent_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.55, 0.30, 0.85, 1.0),
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(Vec2::new(1.0, 1.0)),
            ..default()
        })
    })
}
