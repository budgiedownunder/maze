use crate::images::make_image;
use crate::palette::EMISSIVE_ONLY_BASE;
use bevy::math::Affine2;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, TAU};

// ---------- Tuning constants ----------

const W: u32 = 64;
const H: u32 = 64;
/// Outer / inner radii of the surrounding ring (texels).
const R_OUTER: f32 = 26.0;
const R_INNER: f32 = 23.0;
/// Radius of the pentagram vertices (texels).
const R_PENT: f32 = 22.0;
/// Number of points in the pentagram.
const PENT_POINTS: usize = 5;
/// Half-width of a pentagram stroke (texels).
const STROKE_HALF_WIDTH: f32 = 1.4;

/// Brightness on the pentagram strokes (brightest part).
const STROKE_INTENSITY: u8 = 235;
/// Brightness on the surrounding ring.
const RING_INTENSITY: u8 = 200;
/// Brightness inside the disc (under the strokes, fills the pentagram interior).
const INNER_INTENSITY: u8 = 30;
/// Background brightness (outside the ring).
const BACKGROUND_INTENSITY: u8 = 10;

/// Sigil emissive RGB — arcane purple.
const SIGIL_EMISSIVE: LinearRgba = LinearRgba::new(0.55, 0.30, 0.85, 1.0);
const SIGIL_UV: Vec2 = Vec2::new(1.0, 1.0);

/// Sigil — ornamental glyph: an outer ring with an inscribed 5-point star.
/// Material tints arcane purple — distinct from rune (gold) and glowing
/// glass (amber) so the four floor-accent kinds remain visually distinct.
pub(crate) fn make_sigil_floor_accent_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let cx = W as f32 / 2.0;
    let cy = H as f32 / 2.0;
    // Five points of a pentagram on a circle of radius R_PENT.
    let pent_pts: [(f32, f32); PENT_POINTS] = std::array::from_fn(|i| {
        // -PI/2 puts the first point at the top.
        let a = -FRAC_PI_2 + (i as f32) * TAU / PENT_POINTS as f32;
        (cx + R_PENT * a.cos(), cy + R_PENT * a.sin())
    });
    // The pentagram strokes connect vertex i to vertex (i+2) % PENT_POINTS.
    let strokes: [((f32, f32), (f32, f32)); PENT_POINTS] =
        std::array::from_fn(|i| (pent_pts[i], pent_pts[(i + 2) % PENT_POINTS]));
    let mut pixels = vec![0u8; (W * H * 4) as usize];
    for y in 0..H {
        for x in 0..W {
            let idx = ((y * W + x) * 4) as usize;
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let d = (dx * dx + dy * dy).sqrt();
            let on_ring = (R_INNER..=R_OUTER).contains(&d);
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
            let on_stroke = min_stroke_dist < STROKE_HALF_WIDTH && d < R_PENT + 1.0;
            let v: u8 = if on_stroke {
                STROKE_INTENSITY
            } else if on_ring {
                RING_INTENSITY
            } else if d < R_INNER {
                INNER_INTENSITY
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

pub(crate) fn build_sigil_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> Option<Handle<StandardMaterial>> {
    let tex = images
        .as_mut()
        .map(|imgs| make_sigil_floor_accent_texture(imgs));
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: SIGIL_EMISSIVE,
            emissive_texture: tex,
            uv_transform: Affine2::from_scale(SIGIL_UV),
            ..default()
        })
    })
}
