use crate::images::make_image;
use bevy::math::Affine2;
use bevy::prelude::*;

/// Glowing glass — outer stone frame, inner leaded grid of glass
/// panes separated by thin came. Per-pane base intensity
/// varies slightly so different panes read as different glass when
/// tinted; a radial brightness factor brightens the centre so the
/// panel reads as backlit. Material tints warm amber.
pub(crate) fn make_glowing_glass_decoration_texture(images: &mut Assets<Image>) -> Handle<Image> {
    const W: u32 = 64;
    const H: u32 = 64;
    const FRAME: u32 = 4; // outer stone frame width
    const CELLS: u32 = 4; // 4×4 grid of glass panes
    let inner = W - 2 * FRAME;
    let cell_w = inner / CELLS;
    let cx = W as f32 / 2.0;
    let cy = H as f32 / 2.0;
    let max_r = (cx * cx + cy * cy).sqrt();
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let frame = !(FRAME..W - FRAME).contains(&x) || !(FRAME..H - FRAME).contains(&y);
            let v: u8 = if frame {
                25
            } else {
                let in_x = x - FRAME;
                let in_y = y - FRAME;
                let cell_x = in_x / cell_w;
                let cell_y = in_y / cell_w;
                let rel_x = in_x % cell_w;
                let rel_y = in_y % cell_w;
                let on_lead = rel_x == 0 || rel_y == 0 || rel_x == cell_w - 1 || rel_y == cell_w - 1;
                if on_lead {
                    45
                } else {
                    // Glass-pane base brightness varies per cell so panes
                    // read distinctly; radial falloff brightens the centre.
                    let cell_hash = cell_x.wrapping_mul(11).wrapping_add(cell_y.wrapping_mul(17));
                    let base = 160u32 + cell_hash % 50;
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let radial = 1.0 - (dist / max_r) * 0.4;
                    ((base as f32) * radial).clamp(0.0, 255.0) as u8
                }
            };
            pixels[idx] = v;
            pixels[idx + 1] = v;
            pixels[idx + 2] = v;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(W, H, pixels))
}

pub(crate) fn build_glowing_glass_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images
        .as_mut()
        .map(|imgs| make_glowing_glass_decoration_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(1.00, 0.60, 0.20, 1.0),
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(Vec2::new(1.0, 1.0)),
            ..default()
        })
    })
}
