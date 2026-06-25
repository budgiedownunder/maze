//! Per-cell ceiling for the roofed sky types.
//!
//! Two sky types are roofed-over instead of open-air:
//! - [`SkyType::Dungeon`] — a hewn dark-rock cave ceiling (see [`dungeon`]).
//! - [`SkyType::Chamber`] — a built ceiling in the cell's own wall material, so
//!   a brick maze gets a brick ceiling and a timber maze a timber one (see
//!   [`chamber`]).
//!
//! Both share the same inset-tile mesh: each cell's ceiling is a thin slab at
//! the top of the walls, inset by [`ROOF_GAP`] so a grid of dark grout lines
//! separates adjacent tiles. That visible structure is what keeps the ceiling
//! reading as a solid coffered surface rather than open sky — looking toward a
//! corner, the receding grid is the only depth cue. The ceiling occludes the
//! sky dome from inside and gives a rising portcullis grille somewhere to
//! retract into. Open-air sky types draw no ceiling.

pub(crate) mod chamber;
pub(crate) mod dungeon;

use crate::state::{GameConfig, SkyType};
use crate::world::walls::{WallAssets, WALL_HEIGHT};
use crate::world::{LevelPlacement, CELL_SIZE};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::f32::consts::TAU;

/// Ceiling-tile thickness (units).
const ROOF_THICKNESS: f32 = 0.2;

/// Inset applied to each edge of a ceiling tile, leaving a dark channel between
/// adjacent tiles. This grid of grout lines is what gives the ceiling visible
/// *structure* — without it, a uniform field overhead reads as sky (especially
/// looking toward a corner, where there are no other cues). The tile shows the
/// near-black dome through the gap, so the channel reads as a dark seam.
const ROOF_GAP: f32 = 0.02;

/// Side length of a ceiling tile after the per-edge inset.
const ROOF_TILE: f32 = CELL_SIZE - 2.0 * ROOF_GAP;

/// Radius of the opening cut in the roof tile over a ladder finish, so the climb
/// emerges through it. A touch wider than the hatch rim above (≈0.47) so the rim
/// shows cleanly through the ceiling rather than being clipped by it.
const ROOF_HOLE_RADIUS: f32 = 0.5;

#[derive(Component)]
pub(crate) struct RoofCell;

pub(crate) struct RoofAssets {
    /// Shared inset-tile slab, used by both roofed sky types.
    mesh: Option<Handle<Mesh>>,
    /// Flat inset tile with a central hole — the ceiling over a ladder finish, so
    /// the climb (and the hatch above it) reads as inset into the roof rather than
    /// blocked by a solid tile. Same material as a plain roof tile.
    holed_mesh: Option<Handle<Mesh>>,
    /// Dark-rock material for the [`SkyType::Dungeon`] ceiling. `None` for the
    /// other sky types — chamber pulls its per-cell material from
    /// [`WallAssets`] at spawn time.
    dungeon_material: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_roof_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
    config: &GameConfig,
) -> RoofAssets {
    let roofed = matches!(config.sky_type, SkyType::Dungeon | SkyType::Chamber);
    let mesh = if roofed {
        meshes
            .as_mut()
            .map(|m| m.add(Cuboid::new(ROOF_TILE, ROOF_THICKNESS, ROOF_TILE)))
    } else {
        None
    };
    let holed_mesh = if roofed {
        meshes.as_mut().map(|m| m.add(roof_hole_mesh()))
    } else {
        None
    };
    let dungeon_material = if config.sky_type == SkyType::Dungeon {
        dungeon::build_material(materials, images)
    } else {
        None
    };
    RoofAssets {
        mesh,
        holed_mesh,
        dungeon_material,
    }
}

/// A flat inset roof tile with a central circular hole — a ring of quads from the
/// hole circle out to the (inset) tile square, normals **down** so it reads from
/// below (where the ceiling is seen). UVs map tile-local X/Z to `[0, 1]` across
/// the tile so the shared roof texture tiles as it does on a plain tile.
///
/// The ring sits at `-ROOF_THICKNESS/2` in tile-local space — i.e. flush with the
/// **bottom face** of the neighbouring solid roof cuboids (whose centre `spawn_tile`
/// places at `world_y(WALL_HEIGHT)`). This both matches their visible underside and,
/// critically, keeps the tile clear of the floor of the level above: that floor is
/// coplanar with the roof's centre plane (`LEVEL_HEIGHT == WALL_HEIGHT`), so a tile
/// at `y = 0` here would z-fight with it.
fn roof_hole_mesh() -> Mesh {
    const N: usize = 48;
    let h = ROOF_TILE / 2.0;
    let y = -ROOF_THICKNESS / 2.0;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let uv = |p: Vec2| [(p.x + h) / ROOF_TILE, (p.y + h) / ROOF_TILE];
    for i in 0..=N {
        let t = i as f32 / N as f32 * TAU;
        let (s, c) = t.sin_cos();
        let m = c.abs().max(s.abs()).max(1e-3);
        let inner = Vec2::new(c, s) * ROOF_HOLE_RADIUS; // on the hole circle
        let outer = Vec2::new(c, s) * (h / m); // on the tile square boundary
        positions.push([inner.x, y, inner.y]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push(uv(inner));
        positions.push([outer.x, y, outer.y]);
        normals.push([0.0, -1.0, 0.0]);
        uvs.push(uv(outer));
    }
    for i in 0..N as u32 {
        let (a, b) = (i * 2, i * 2 + 1); // inner / outer at i
        let (c, d) = ((i + 1) * 2, (i + 1) * 2 + 1); // inner / outer at i+1
        // Wound so the front face points DOWN (matching the -Y normals), so the
        // ceiling renders for the level below it looks up at.
        indices.extend_from_slice(&[a, b, d, a, d, c]);
    }
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_roof_for_cell(
    commands: &mut Commands,
    roof_assets: &RoofAssets,
    wall_assets: &WallAssets,
    grid: &[Vec<char>],
    r: usize,
    c: usize,
    config: &GameConfig,
    placement: LevelPlacement,
    // True at a ladder-finish cell: the tile carries the climb's opening so the
    // ladder isn't sealed under a solid ceiling. Only ever set on a roofed level.
    holed: bool,
) {
    let material = match config.sky_type {
        SkyType::Dungeon => roof_assets.dungeon_material.clone(),
        SkyType::Chamber => chamber::material_for_cell(wall_assets, grid, r, c, config),
        _ => return,
    };
    let mesh = if holed {
        roof_assets.holed_mesh.clone()
    } else {
        roof_assets.mesh.clone()
    };
    spawn_tile(commands, mesh, material, r, c, placement);
}

/// Spawns one ceiling tile at the top of the walls over cell `(r, c)`.
fn spawn_tile(
    commands: &mut Commands,
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let roof_y = placement.world_y(WALL_HEIGHT);
    match (mesh, material) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                RoofCell,
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(x, roof_y, z),
            ));
        }
        _ => {
            commands.spawn((RoofCell, Transform::from_xyz(x, roof_y, z)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The holed roof tile is a well-formed ring of quads: an inner ring on the
    /// hole circle (radius [`ROOF_HOLE_RADIUS`]) and an outer ring on the inset
    /// tile boundary (within the tile half-extent), every normal pointing **down**
    /// so the ceiling renders for the level below.
    #[test]
    fn roof_hole_mesh_is_a_downward_ring_around_the_hole() {
        const N: usize = 48;
        let mesh = roof_hole_mesh();
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("expected Float32x3 positions");
        };
        // (N+1) rings × 2 verts (inner + outer) each.
        assert_eq!(positions.len(), (N + 1) * 2);
        let half = ROOF_TILE / 2.0;
        for (i, p) in positions.iter().enumerate() {
            if i % 2 == 0 {
                // Inner ring sits exactly on the hole circle.
                let radius = (p[0] * p[0] + p[2] * p[2]).sqrt();
                assert!((radius - ROOF_HOLE_RADIUS).abs() < 1e-4, "inner vertex off the hole circle");
            } else {
                // Outer ring sits on the inset tile's square boundary (so its
                // Chebyshev distance — the larger of |x| / |z| — equals the half-extent).
                let chebyshev = p[0].abs().max(p[2].abs());
                assert!((chebyshev - half).abs() < 1e-4, "outer vertex off the tile edge");
            }
        }
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("expected Float32x3 normals");
        };
        assert!(normals.iter().all(|n| *n == [0.0, -1.0, 0.0]), "every roof-hole normal faces down");
        // The ring sits at the solid tiles' bottom-face plane, not the centre — so it
        // doesn't z-fight with the (coplanar-with-the-centre) floor of the level above.
        assert!(
            positions.iter().all(|p| (p[1] - (-ROOF_THICKNESS / 2.0)).abs() < 1e-6),
            "the holed tile sits a half-thickness below the tile centre"
        );
        assert_eq!(mesh.indices().map(|i| i.len()), Some(N * 6));
    }
}
