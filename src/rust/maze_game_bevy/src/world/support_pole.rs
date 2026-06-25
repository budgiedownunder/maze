//! Support poles bracing a floating upper level at its corners.
//!
//! A solid wall on a level rises a full `LEVEL_HEIGHT` and meets the floor of the
//! level above, so it carries that floor; a level with no solid walls (e.g. all
//! water / lava) carries nothing, so on an open-perimeter stack the upper floor
//! appears to **float**. To brace it we drop a slim vertical pole at each **corner**
//! of the upper level that isn't already held up — i.e. that doesn't sit over a
//! solid wall and isn't carried by a solid perimeter wall at the lower level's edge.
//!
//! At most **four** poles per level (one per corner), fewer when corners are
//! supported. Each pole sits at the corner cell's *outward* corner, inset just
//! inside the footprint, and rises from the lower floor to the floor it braces.
//!
//! Visual-only: placed by [`crate::world::spawn_world`] after the per-level
//! geometry, carries no collision, and spans the (possibly 6b-lifted) gap. Poles
//! rise from a lower level to the one above it — never from the top level, which
//! holds nothing up.

use bevy::prelude::*;

/// Pole radius — slim, so it reads as a support column without crowding the cell.
const POLE_RADIUS: f32 = 0.12;

/// How far inside the footprint corner a pole sits, so it tucks just inside the
/// upper level's outward corner rather than poking past it.
pub(crate) const CORNER_INSET: f32 = 0.18;

/// Marker on a support-pole entity.
#[derive(Component)]
pub(crate) struct SupportPole;

/// The corner CELLS of a `rows`×`cols` upper level that need a support pole — the
/// (up to 4) distinct corners for which `supported(corner_row, corner_col)` is
/// false. Pure (no Bevy) so the corner rule is unit-testable; the caller decides
/// what "supported" means (a solid wall below, or a perimeter wall at the edge).
pub(crate) fn corner_poles(
    rows: usize,
    cols: usize,
    supported: impl Fn(usize, usize) -> bool,
) -> Vec<(usize, usize)> {
    if rows == 0 || cols == 0 {
        return Vec::new();
    }
    let mut corners = vec![(0, 0), (0, cols - 1), (rows - 1, 0), (rows - 1, cols - 1)];
    corners.sort_unstable();
    corners.dedup(); // a 1-wide / 1-tall level shares corners
    corners.into_iter().filter(|&(r, c)| !supported(r, c)).collect()
}

pub(crate) struct SupportPoleAssets {
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_support_pole_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> SupportPoleAssets {
    // A unit-height cylinder scaled in Y per pole, so every pole shares one mesh.
    let mesh = meshes.as_mut().map(|m| m.add(Cylinder::new(POLE_RADIUS, 1.0)));
    let material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            // Dark dressed-stone grey, distinct from the floors / pools around it.
            base_color: Color::srgb(0.30, 0.29, 0.27),
            perceptual_roughness: 0.9,
            ..default()
        })
    });
    SupportPoleAssets { mesh, material }
}

/// Spawns a vertical pole spanning `bottom_y..top_y` at world `(x, z)`. A
/// degenerate (non-positive) span stays harmless.
pub(crate) fn spawn_support_pole(
    commands: &mut Commands,
    assets: &SupportPoleAssets,
    x: f32,
    z: f32,
    bottom_y: f32,
    top_y: f32,
) {
    let height = (top_y - bottom_y).max(0.0);
    let mid_y = (bottom_y + top_y) / 2.0;
    let transform = Transform::from_xyz(x, mid_y, z).with_scale(Vec3::new(1.0, height, 1.0));
    match (assets.mesh.clone(), assets.material.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((SupportPole, transform, Mesh3d(mesh), MeshMaterial3d(mat)));
        }
        _ => {
            commands.spawn((SupportPole, transform));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_poles_returns_only_unsupported_corners_capped_at_four() {
        // Nothing supported → all four corners.
        assert_eq!(corner_poles(5, 5, |_, _| false), vec![(0, 0), (0, 4), (4, 0), (4, 4)]);
        // Everything supported (e.g. a solid perimeter) → none.
        assert!(corner_poles(5, 5, |_, _| true).is_empty());
        // One corner carried (e.g. a solid wall below it) → the other three.
        let three = corner_poles(5, 5, |r, c| (r, c) == (4, 4));
        assert_eq!(three, vec![(0, 0), (0, 4), (4, 0)]);
        // A 1-wide level shares corners → at most two distinct.
        assert_eq!(corner_poles(3, 1, |_, _| false), vec![(0, 0), (2, 0)]);
    }
}
