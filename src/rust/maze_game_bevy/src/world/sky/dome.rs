//! Inverted-sphere sky dome.
//!
//! Spawns a single large sphere around the player, painted on the inside
//! face with a procedurally generated panoramic sky texture. The dome
//! follows the camera's *position* every frame (but not its rotation),
//! so stars and clouds appear infinitely far away — looking around
//! sweeps the sky past the player, walking around does not move the
//! visible sky.

use crate::palette::UNLIT_FULL_BRIGHT;
use bevy::prelude::*;
use bevy::render::render_resource::Face;

/// Marker component for the dome entity. One per scene; queried by the
/// follow-camera system below and by the lib.rs smoke tests.
#[derive(Component)]
pub(crate) struct SkyDome;

/// Radius of the dome sphere. Picked to be:
/// - Comfortably outside the largest configured maze (40×40 cells at
///   `CELL_SIZE = 2.0` = 80 units across, so the player is at most ~80
///   units from the centre when wandering — well inside 500).
/// - Inside Bevy's default camera far plane (1000.0).
pub(crate) const SKY_RADIUS: f32 = 500.0;

pub(crate) fn spawn_dome(
    commands: &mut Commands,
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    sky_texture: Option<Handle<Image>>,
) -> Entity {
    // UV-sphere mesh so the procedurally generated equirectangular
    // texture maps cleanly to lat/lon. `64 x 32` segments give a smooth
    // silhouette without spending pixels on detail the dome doesn't have.
    let mesh = meshes
        .as_mut()
        .map(|m| m.add(Sphere::new(SKY_RADIUS).mesh().uv(64, 32)));
    let material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: UNLIT_FULL_BRIGHT,
            base_color_texture: sky_texture,
            // Unlit so the sky paints its own colour rather than picking
            // up the per-mode ambient/directional light tint (which would
            // be a second-order multiplication and defeat the palette).
            unlit: true,
            // Camera sits inside the sphere; cull the FRONT (outward)
            // faces so the inward faces of the dome remain visible to
            // the player.
            cull_mode: Some(Face::Front),
            ..default()
        })
    });
    let mut entity = commands.spawn((SkyDome, super::SkyEntity, Transform::default()));
    if let (Some(mesh), Some(material)) = (mesh, material) {
        entity.insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
    entity.id()
}

/// Each frame, snap the dome's translation to the camera's translation
/// while leaving rotation alone. The dome then behaves like a skybox:
/// stars/clouds stay in fixed angular positions around the player
/// regardless of where they walk in the maze.
pub(crate) fn sky_dome_follow_camera(
    camera_q: Query<&Transform, (With<Camera3d>, Without<SkyDome>)>,
    mut dome_q: Query<&mut Transform, With<SkyDome>>,
) {
    let Ok(cam_xform) = camera_q.single() else {
        return;
    };
    for mut dome_xform in dome_q.iter_mut() {
        dome_xform.translation = cam_xform.translation;
    }
}
