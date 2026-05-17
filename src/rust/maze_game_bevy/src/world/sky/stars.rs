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

use super::dome::SKY_RADIUS;
use super::next_unit;
use crate::palette::UNLIT_FULL_BRIGHT;
use bevy::prelude::*;
use std::f32::consts::TAU;

/// Marker for star entities. Useful for both the per-frame follow path
/// (via the dome parent) and for tests.
#[derive(Component)]
pub(crate) struct Star;

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
    // Shared mesh + material across every star. Subdivision 0 gives a
    // 20-triangle icosahedron, which is more than enough geometry for a
    // sub-pixel target.
    let mesh = meshes.as_mut().map(|m| {
        m.add(
            Sphere::new(STAR_RADIUS)
                .mesh()
                .ico(0)
                .expect("ico(0) is well within the subdivision limit"),
        )
    });
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

    let mut state = seed;
    for _ in 0..count {
        // Uniform sample on the upper hemisphere — `cos_theta` in `[0, 1)`
        // is the cosine of the polar angle from the zenith; combined with
        // a uniform azimuth this produces an even distribution across the
        // visible (upper) half of the celestial sphere.
        let cos_theta = next_unit(&mut state);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
        let phi = next_unit(&mut state) * TAU;
        let pos = Vec3::new(
            sin_theta * phi.cos() * STAR_DISTANCE,
            cos_theta * STAR_DISTANCE,
            sin_theta * phi.sin() * STAR_DISTANCE,
        );

        let star = {
            let mut entity = commands.spawn((Star, Transform::from_translation(pos)));
            if let (Some(mesh), Some(material)) = (mesh.clone(), material.clone()) {
                entity.insert((Mesh3d(mesh), MeshMaterial3d(material)));
            }
            entity.id()
        };
        // Parent to the dome so the dome's per-frame translation update
        // also moves all stars — keeping them angularly fixed relative
        // to the world as the player walks around.
        commands.entity(dome).add_child(star);
    }
}

