//! 3D entity star field.
//!
//! Stars are tiny emissive spheres placed at distance just inside the
//! sky dome and parented to it, so they follow the camera's position
//! (via the dome's follow-camera system) but stay at fixed angular
//! positions relative to the world — exactly the "infinitely far"
//! behaviour expected of a starfield.
//!
//! Each star is sized so it covers roughly 1 screen pixel at typical
//! FoV / window sizes; texture-based stars (painted into the sky-dome
//! image) are stuck at the dome texture's resolution and end up as
//! ~10-pixel blocks once magnified onto the dome, which is why this
//! module exists as a separate path.
//!
//! The whole field is **one entity**. Stars never move relative to each other
//! or to the dome, so the sphere is stamped at every star position and merged
//! into a single mesh. Spawning them individually cost 1000 entities under a
//! night sky — more than half of everything in a scene — and every one of them
//! was carried through extraction, culling and batching on every frame. Baking
//! is visually identical: same positions, same material, same parent.

use super::dome::SKY_RADIUS;
use super::next_unit;
use crate::palette::UNLIT_FULL_BRIGHT;
use bevy::prelude::*;
use std::f32::consts::TAU;

/// Marker for the starfield. **One per sky, not one per star** — the field is
/// baked into a single mesh (see the module docs).
#[derive(Component)]
pub(crate) struct StarField;

/// Distance from the dome centre at which stars sit. Slightly inside
/// the dome surface so depth ordering puts stars in front of the
/// painted dome texture rather than z-fighting with it.
const STAR_DISTANCE: f32 = SKY_RADIUS - 5.0;

/// Sphere radius for each star. At [`STAR_DISTANCE`] this subtends
/// roughly 2 screen pixels on a 720-tall window with a 45° vertical
/// FoV. Anything smaller drops below the rasteriser's reliable
/// threshold for sub-pixel triangles and the stars stop appearing on
/// screen at all.
const STAR_RADIUS: f32 = 0.7;

pub(crate) fn spawn_stars(
    commands: &mut Commands,
    dome: Entity,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    count: u32,
    seed: u64,
) {
    // Positions first, then one merged mesh: the sphere is cheap to stamp and
    // the result draws as a single object.
    let mut state = seed;
    let positions: Vec<Vec3> = (0..count)
        .map(|_| {
            // Uniform sample on the upper hemisphere — `cos_theta` in `[0, 1)`
            // is the cosine of the polar angle from the zenith; combined with
            // a uniform azimuth this produces an even distribution across the
            // visible (upper) half of the celestial sphere.
            let cos_theta = next_unit(&mut state);
            let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
            let phi = next_unit(&mut state) * TAU;
            Vec3::new(
                sin_theta * phi.cos() * STAR_DISTANCE,
                cos_theta * STAR_DISTANCE,
                sin_theta * phi.sin() * STAR_DISTANCE,
            )
        })
        .collect();

    let baked = match (meshes.as_mut(), positions.is_empty()) {
        (Some(store), false) => {
            let base = Sphere::new(STAR_RADIUS)
                .mesh()
                .ico(0)
                .expect("ico(0) is well within the subdivision limit");
            let mut acc = base
                .clone()
                .transformed_by(Transform::from_translation(positions[0]));
            for pos in &positions[1..] {
                let _ = acc.merge(&base.clone().transformed_by(Transform::from_translation(*pos)));
            }
            Some(store.add(acc))
        }
        _ => None,
    };

    let material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            // With `unlit: true` Bevy outputs `base_color` directly and
            // skips the lighting calculation — so the colour MUST live
            // in `base_color` (emissive is gated behind the PBR
            // lighting path and silently does nothing under `unlit`).
            base_color: UNLIT_FULL_BRIGHT,
            unlit: true,
            ..default()
        })
    });

    // Spawned even without render assets so headless counting still sees the
    // field, matching every other spawn helper here.
    let field = {
        let mut entity = commands.spawn((StarField, Transform::default()));
        if let (Some(mesh), Some(material)) = (baked, material) {
            entity.insert((Mesh3d(mesh), MeshMaterial3d(material)));
        }
        entity.id()
    };
    // Parented to the dome so the dome's per-frame translation update moves the
    // field with it — keeping the stars angularly fixed as the player walks.
    commands.entity(dome).add_child(field);
}

