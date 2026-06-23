//! Goblin enemy rig — the default `EnemyType` variant. Opaque blob body
//! with two front-facing emissive eye spots. Idle motion (bob) is applied
//! by the parent `enemy_animation_system`; this module just builds the
//! mesh/material assets and assembles the per-cell entity hierarchy.

use crate::images::make_image;
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::{LevelPlacement, CELL_SIZE};
use bevy::prelude::*;
use std::f32::consts::PI;

// ---------- Tuning constants ----------

/// Goblin body sphere radius (units).
const BODY_RADIUS: f32 = 0.32;
/// Eye sphere radius (units) — child of the body.
const EYE_RADIUS: f32 = 0.07;
/// Eye horizontal separation from body centre (units).
const EYE_OFFSET_X: f32 = 0.12;
/// Eye vertical offset above body centre (units).
const EYE_OFFSET_Y: f32 = 0.07;
/// Eye forward offset from body centre (units; +Z by convention).
const EYE_OFFSET_Z: f32 = 0.27;

/// Tooth cone base radius — sized so a row reads as a grin rather than a
/// second pair of eyes.
const TOOTH_RADIUS: f32 = 0.035;
/// Tooth cone height (apex-to-base) — taller than wide so each tooth
/// reads as a sharp triangular fang.
const TOOTH_HEIGHT: f32 = 0.08;
/// Number of teeth in the row.
const TOOTH_COUNT: usize = 5;
/// Half-width of the tooth row — teeth are spread evenly across
/// `[-W, +W]` horizontally.
const GRIN_HALF_WIDTH: f32 = 0.13;
/// Vertical centre of the tooth row, below body centre.
const GRIN_OFFSET_Y: f32 = -0.10;
/// Forward offset of the teeth from body centre — slightly more forward
/// than the eyes so they sit on the front face of the head.
const GRIN_OFFSET_Z: f32 = 0.29;
/// Vertical curvature of the tooth row: outer teeth lift this far above
/// the row's centre, producing a parabolic smile arc.
const GRIN_CURVE_LIFT: f32 = 0.025;

// Body texture dimensions — wide enough that the painted mouth ellipse
// has plenty of pixels along the longitudinal axis (which wraps the full
// equator) without the patch reading pixellated.
const BODY_TEX_W: u32 = 256;
const BODY_TEX_H: u32 = 128;

// The mouth is painted onto the body texture as an upward-arcing dark
// region in the front-bottom area. Coordinates below are in UV space:
// `u` wraps the full longitude (0 at the seam; `u = 0.25` lands on the
// body's local +Z "front" face under Bevy's `Sphere` UV mapping). `v`
// runs north-to-south on Bevy's `Sphere` — `v = 0` is the +Y pole,
// `v = 1` is the -Y pole — so `v > 0.5` puts the mouth in the lower
// hemisphere, below the eye row.

/// UV centre of the painted mouth — `u = 0.25` aims for the body's local
/// front face; `v = 0.60` puts the mouth in the lower hemisphere, with
/// its painted top edge aligned with the tops of the tooth row.
const MOUTH_UV_X: f32 = 0.25;
const MOUTH_UV_Y: f32 = 0.60;
/// Half-extent of the mouth ellipse along each axis. Wider than tall to
/// match the width of the tooth row.
const MOUTH_UV_HALF_W: f32 = 0.13;
const MOUTH_UV_HALF_H: f32 = 0.06;
/// Upward curvature of the mouth's top edge — outer edges sit this far
/// (in UV space) above the centre, mirroring the parabolic lift of the
/// tooth row so the mouth's top arc matches the tops of the teeth.
const MOUTH_UV_CURVE_LIFT: f32 = 0.018;

/// Goblin body emissive — desaturated sickly green so it reads as
/// "approach with caution" at a glance.
const BODY_EMISSIVE: LinearRgba = LinearRgba::new(0.35, 0.55, 0.20, 1.0);
/// Goblin eye emissive — hot amber for contrast against the body green.
const EYE_EMISSIVE: LinearRgba = LinearRgba::new(1.4, 0.7, 0.05, 1.0);
/// Goblin tooth emissive — bright off-white so the fangs pop against the
/// dark mouth painted on the body without being eye-bright.
const TOOTH_EMISSIVE: LinearRgba = LinearRgba::new(1.0, 0.95, 0.85, 1.0);

/// Resting Y position — keeps the goblin floating just above the floor so
/// the player sees the menacing shape clearly from a player-cell-height
/// camera. Read by [`super::enemy_animation_system`].
pub(crate) const ENEMY_BASE_Y: f32 = 0.5;

/// Idle bob frequency (radians/sec) — same cadence as the finish orb so
/// the world feels consistent. Read by [`super::enemy_animation_system`].
pub(crate) const BOB_RATE: f32 = 2.0;
/// Idle bob amplitude (units of vertical travel from the resting Y).
pub(crate) const BOB_AMPLITUDE: f32 = 0.08;

pub(crate) struct GoblinAssets {
    body_mesh: Option<Handle<Mesh>>,
    body_mat: Option<Handle<StandardMaterial>>,
    eye_mesh: Option<Handle<Mesh>>,
    eye_mat: Option<Handle<StandardMaterial>>,
    tooth_mesh: Option<Handle<Mesh>>,
    tooth_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_goblin_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> GoblinAssets {
    let body_mesh = meshes.as_mut().map(|m| m.add(Sphere::new(BODY_RADIUS)));
    let eye_mesh = meshes.as_mut().map(|m| m.add(Sphere::new(EYE_RADIUS)));
    let tooth_mesh = meshes.as_mut().map(|m| {
        m.add(Cone {
            radius: TOOTH_RADIUS,
            height: TOOTH_HEIGHT,
        })
    });
    // The body's mouth is painted onto its emissive_texture rather than
    // spawned as a separate child entity. Using emissive=WHITE keeps the
    // texture's pixel colours intact (no per-frame tint multiplication),
    // so the green body and the black mouth read at their true colours.
    let body_tex = images.as_mut().map(|imgs| make_body_texture(imgs));
    let body_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: LinearRgba::WHITE,
            emissive_texture: body_tex,
            ..default()
        })
    });
    let eye_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: EYE_EMISSIVE,
            ..default()
        })
    });
    let tooth_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: TOOTH_EMISSIVE,
            ..default()
        })
    });
    GoblinAssets {
        body_mesh,
        body_mat,
        eye_mesh,
        eye_mat,
        tooth_mesh,
        tooth_mat,
    }
}

/// Builds the goblin body's emissive texture: a flat green field for
/// most of the surface, with a dark upward-arcing ellipse painted at the
/// front-bottom for the mouth. UV layout follows Bevy's `Sphere`
/// convention — `u` wraps longitude (0..1) and `v` runs south-pole-up
/// (0 at -Y, 1 at +Y); the mouth lives near `u = 0.75` (body's +Z
/// front face) and `v = 0.36` (below the equator).
fn make_body_texture(images: &mut Assets<Image>) -> Handle<Image> {
    let body_r = (BODY_EMISSIVE.red.sqrt().clamp(0.0, 1.0) * 255.0) as u8;
    let body_g = (BODY_EMISSIVE.green.sqrt().clamp(0.0, 1.0) * 255.0) as u8;
    let body_b = (BODY_EMISSIVE.blue.sqrt().clamp(0.0, 1.0) * 255.0) as u8;
    let mut pixels = vec![0u8; (BODY_TEX_W * BODY_TEX_H * 4) as usize];
    for y in 0..BODY_TEX_H {
        for x in 0..BODY_TEX_W {
            let idx = ((y * BODY_TEX_W + x) * 4) as usize;
            let u = (x as f32 + 0.5) / BODY_TEX_W as f32;
            let v = (y as f32 + 0.5) / BODY_TEX_H as f32;
            // Distance to the mouth ellipse's centre, in UV units. Wrap
            // `du` through the seam so a mouth near `u = 0` or `u = 1`
            // still reads as one contiguous patch.
            let mut du = u - MOUTH_UV_X;
            if du > 0.5 {
                du -= 1.0;
            } else if du < -0.5 {
                du += 1.0;
            }
            // Lift the mouth's centreline parabolically with `du` so the
            // ellipse's top edge arcs upward at its outer ends, matching
            // the tooth row's curve. "Up" is `-v` under Bevy's
            // top-to-bottom V axis, so the lift subtracts.
            let t = du / MOUTH_UV_HALF_W;
            let dv = v - (MOUTH_UV_Y - MOUTH_UV_CURVE_LIFT * t * t);
            let nu = du / MOUTH_UV_HALF_W;
            let nv = dv / MOUTH_UV_HALF_H;
            let inside_mouth = nu * nu + nv * nv <= 1.0;
            let (r, g, b) = if inside_mouth {
                (0, 0, 0)
            } else {
                (body_r, body_g, body_b)
            };
            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255;
        }
    }
    images.add(make_image(BODY_TEX_W, BODY_TEX_H, pixels))
}

/// Spawns the Goblin entity hierarchy for the `'E'` cell at `(r, c)` with
/// the given enemy `id`. The marker carries the id so the animation system
/// can match it to `maze::Enemy` each frame. Eyes are children of the body
/// so they stay fixed-forward relative to it.
pub(crate) fn spawn_goblin(
    commands: &mut Commands,
    assets: &GoblinAssets,
    r: usize,
    c: usize,
    id: u32,
    placement: LevelPlacement,
) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let y = placement.world_y(ENEMY_BASE_Y);
    let body = match (assets.body_mesh.clone(), assets.body_mat.clone()) {
        (Some(mesh), Some(mat)) => commands
            .spawn((
                super::EnemyMarker {
                    id,
                    spawn_cell: (r, c),
                    placement,
                },
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(x, y, z),
            ))
            .id(),
        _ => commands
            .spawn((
                super::EnemyMarker {
                    id,
                    spawn_cell: (r, c),
                    placement,
                },
                Transform::from_xyz(x, y, z),
            ))
            .id(),
    };
    if let (Some(eye_mesh), Some(eye_mat)) = (assets.eye_mesh.clone(), assets.eye_mat.clone()) {
        commands.entity(body).with_children(|parent| {
            parent.spawn((
                Mesh3d(eye_mesh.clone()),
                MeshMaterial3d(eye_mat.clone()),
                Transform::from_xyz(-EYE_OFFSET_X, EYE_OFFSET_Y, EYE_OFFSET_Z),
            ));
            parent.spawn((
                Mesh3d(eye_mesh),
                MeshMaterial3d(eye_mat),
                Transform::from_xyz(EYE_OFFSET_X, EYE_OFFSET_Y, EYE_OFFSET_Z),
            ));
        });
    }
    if let (Some(tooth_mesh), Some(tooth_mat)) = (
        assets.tooth_mesh.clone(),
        assets.tooth_mat.clone(),
    ) {
        commands.entity(body).with_children(|parent| {
            // Teeth spread evenly along the row's horizontal axis with a
            // parabolic upward curve at the edges. Bevy's Cone primitive
            // points along +Y by default; the PI rotation around X
            // inverts that so each fang's apex points downward into the
            // mouth painted on the body texture.
            for i in 0..TOOTH_COUNT {
                let t = if TOOTH_COUNT == 1 {
                    0.0
                } else {
                    -1.0 + 2.0 * (i as f32) / (TOOTH_COUNT as f32 - 1.0)
                };
                let x = GRIN_HALF_WIDTH * t;
                let y = GRIN_OFFSET_Y + GRIN_CURVE_LIFT * t * t;
                parent.spawn((
                    Mesh3d(tooth_mesh.clone()),
                    MeshMaterial3d(tooth_mat.clone()),
                    Transform::from_xyz(x, y, GRIN_OFFSET_Z)
                        .with_rotation(Quat::from_rotation_x(PI)),
                ));
            }
        });
    }
}
