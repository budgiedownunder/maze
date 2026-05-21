//! The swinging door slab — the panel that fills the corridor opening at a
//! `'D'` cell and rotates around its hinge as the door opens.

use crate::world::walls::{PANEL_H, PANEL_W, PANEL_Y};
use bevy::prelude::*;

/// Door slab thickness (units). Thicker than a wall panel
/// ([`crate::world::walls::WALL_THICKNESS`]) so the door reads as a distinct
/// movable object rather than a flat wall section.
pub(crate) const DOOR_THICKNESS: f32 = 0.12;

pub(crate) struct PanelAssets {
    mesh: Option<Handle<Mesh>>,
}

pub(crate) fn build_panel_assets(meshes: &mut Option<ResMut<Assets<Mesh>>>) -> PanelAssets {
    let mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(PANEL_W, PANEL_H, DOOR_THICKNESS)));
    PanelAssets { mesh }
}

/// Spawns the slab as a child of the hinge `pivot`, spanning the doorway from
/// the hinge (local origin) out along local `+X`. `material` is the cell's wall
/// material so the closed door reads as part of the surrounding wall.
pub(crate) fn spawn_panel(
    commands: &mut Commands,
    assets: &PanelAssets,
    material: Option<Handle<StandardMaterial>>,
    pivot: Entity,
) {
    if let (Some(mesh), Some(mat)) = (assets.mesh.clone(), material) {
        let panel = commands
            .spawn((
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(PANEL_W / 2.0, PANEL_Y, 0.0),
            ))
            .id();
        commands.entity(pivot).add_child(panel);
    }
}
