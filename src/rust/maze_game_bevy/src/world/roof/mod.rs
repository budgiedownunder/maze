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

/// Clearance kept between the roof tile's TOP and the plane it caps (the wall-top,
/// which in a multi-level stack is *also* the floor of the level above — they share
/// the plane because `LEVEL_HEIGHT == WALL_HEIGHT`). The tile is hung just under that
/// plane so it never pokes up through and occludes that floor. Comfortably larger
/// than `FLOOR_THICKNESS`, so it also clears the floor tile's own underside.
const ROOF_CLEARANCE: f32 = 0.02;

/// The Y a roof tile is centred at, given `ceiling` = `placement.world_y(WALL_HEIGHT)`
/// (the wall-top plane). Dropped by half the tile thickness + [`ROOF_CLEARANCE`] so
/// the tile's top sits just below `ceiling` rather than `ROOF_THICKNESS/2` above it.
fn roof_center_y(ceiling: f32) -> f32 {
    ceiling - ROOF_THICKNESS / 2.0 - ROOF_CLEARANCE
}

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

/// A `ROOF_THICKNESS`-thick inset roof tile with a central circular hole — built to
/// match the solid [`Cuboid`] roof tiles around it (same `ROOF_TILE` footprint, same
/// thickness, centred at `world_y(WALL_HEIGHT)`) so its edges/grout read identically,
/// just with the opening the climb passes through. It's a ring slab: a **bottom**
/// face (what the level below looks up at), a **top** face, the **outer** side walls
/// at the tile edge, and the **inner** hole walls (the opening's rim). The bottom
/// face's UVs match a Bevy `Cuboid`'s **bottom** face (`u = (h - x)/size`,
/// `v = (h - z)/size`) and the top face its **top** face (`u = (x + h)/size`,
/// `v = (z + h)/size`), so the shared roof texture samples the same texels (same
/// shade) and lines up with the neighbours.
fn roof_hole_mesh() -> Mesh {
    const N: usize = 48;
    let h = ROOF_TILE / 2.0;
    let t = ROOF_THICKNESS / 2.0;
    let bottom_uv = |x: f32, z: f32| [(h - x) / ROOF_TILE, (h - z) / ROOF_TILE];
    let top_uv = |x: f32, z: f32| [(x + h) / ROOF_TILE, (z + h) / ROOF_TILE];
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    // Push one quad (verts in CCW order for the desired front, per the right-hand
    // rule) as two triangles, with a flat per-quad normal and per-vertex UVs.
    let mut quad = |v: [[f32; 3]; 4], n: [f32; 3], uv: [[f32; 2]; 4]| {
        let b = positions.len() as u32;
        positions.extend_from_slice(&v);
        normals.extend_from_slice(&[n; 4]);
        uvs.extend_from_slice(&uv);
        indices.extend_from_slice(&[b, b + 1, b + 2, b, b + 2, b + 3]);
    };
    // The hole-circle and tile-square points at an angle.
    let corner = |a: f32| {
        let (s, c) = a.sin_cos();
        let m = c.abs().max(s.abs()).max(1e-3);
        (Vec2::new(c, s) * ROOF_HOLE_RADIUS, Vec2::new(c, s) * (h / m))
    };
    for i in 0..N {
        let (in0, out0) = corner(i as f32 / N as f32 * TAU);
        let (in1, out1) = corner((i + 1) as f32 / N as f32 * TAU);
        // Bottom ring — front faces DOWN (the ceiling the level below sees).
        quad(
            [[in0.x, -t, in0.y], [out0.x, -t, out0.y], [out1.x, -t, out1.y], [in1.x, -t, in1.y]],
            [0.0, -1.0, 0.0],
            [bottom_uv(in0.x, in0.y), bottom_uv(out0.x, out0.y), bottom_uv(out1.x, out1.y), bottom_uv(in1.x, in1.y)],
        );
        // Top ring — front faces UP (matches the solid tiles seen from the level above).
        quad(
            [[in0.x, t, in0.y], [in1.x, t, in1.y], [out1.x, t, out1.y], [out0.x, t, out0.y]],
            [0.0, 1.0, 0.0],
            [top_uv(in0.x, in0.y), top_uv(in1.x, in1.y), top_uv(out1.x, out1.y), top_uv(out0.x, out0.y)],
        );
        // Outer side wall at the tile edge — front faces OUTWARD; gives the tile the
        // same visible thickness/grout as the solid cuboid neighbours.
        let on = (out0 + out1).normalize_or_zero();
        quad(
            [[out0.x, -t, out0.y], [out0.x, t, out0.y], [out1.x, t, out1.y], [out1.x, -t, out1.y]],
            [on.x, 0.0, on.y],
            [bottom_uv(out0.x, out0.y), bottom_uv(out0.x, out0.y), bottom_uv(out1.x, out1.y), bottom_uv(out1.x, out1.y)],
        );
        // Inner hole wall — front faces INWARD (the opening's rim thickness).
        let inn = -(in0 + in1).normalize_or_zero();
        quad(
            [[in0.x, -t, in0.y], [in1.x, -t, in1.y], [in1.x, t, in1.y], [in0.x, t, in0.y]],
            [inn.x, 0.0, inn.y],
            [bottom_uv(in0.x, in0.y), bottom_uv(in1.x, in1.y), bottom_uv(in1.x, in1.y), bottom_uv(in0.x, in0.y)],
        );
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

/// Spawns one ceiling tile just under the top of the walls over cell `(r, c)` — hung
/// a hair below the wall-top plane (see [`roof_center_y`]) so in a multi-level stack
/// it doesn't poke up through the floor of the level above.
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
    let roof_y = roof_center_y(placement.world_y(WALL_HEIGHT));
    match (mesh, material) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                RoofCell,
                placement.tag(),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(x, roof_y, z),
            ));
        }
        _ => {
            commands.spawn((RoofCell, placement.tag(), Transform::from_xyz(x, roof_y, z)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The roof tile's TOP must sit below the plane it caps (the wall-top, shared
    /// with the floor of the level above), or it occludes that floor from above.
    #[test]
    fn the_roof_tile_top_stays_below_the_plane_it_caps() {
        let ceiling = 3.0;
        let top = roof_center_y(ceiling) + ROOF_THICKNESS / 2.0;
        assert!(top < ceiling, "roof top {top} must sit below the capped plane {ceiling}");
        // ...and below the floor tile's own underside, so it doesn't z-fight it.
        assert!(
            top < ceiling - crate::world::floor::FLOOR_THICKNESS / 2.0,
            "roof top must clear the floor-above's underside",
        );
    }

    /// The holed roof tile is a `ROOF_THICKNESS`-thick ring slab: it spans the full
    /// tile thickness (like the solid cuboids), every vertex is inside the tile
    /// footprint and no nearer the centre than the hole rim, and its bottom face
    /// (normal `-Y`, at `y = -t`) carries the cuboid BOTTOM-face UV so the rock
    /// texture samples the same texels / shade as the solid neighbours.
    #[test]
    fn roof_hole_mesh_is_a_thick_holed_slab_matching_the_cuboid_bottom() {
        let mesh = roof_hole_mesh();
        let h = ROOF_TILE / 2.0;
        let t = ROOF_THICKNESS / 2.0;
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(positions)) =
            mesh.attribute(Mesh::ATTRIBUTE_POSITION)
        else {
            panic!("expected Float32x3 positions");
        };
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(normals)) =
            mesh.attribute(Mesh::ATTRIBUTE_NORMAL)
        else {
            panic!("expected Float32x3 normals");
        };
        let Some(bevy::mesh::VertexAttributeValues::Float32x2(uvs)) =
            mesh.attribute(Mesh::ATTRIBUTE_UV_0)
        else {
            panic!("expected Float32x2 UVs");
        };
        // Spans the full thickness, like the solid cuboid tiles.
        let min_y = positions.iter().map(|p| p[1]).fold(f32::INFINITY, f32::min);
        let max_y = positions.iter().map(|p| p[1]).fold(f32::NEG_INFINITY, f32::max);
        assert!((min_y + t).abs() < 1e-5 && (max_y - t).abs() < 1e-5, "slab spans [-t, t]");
        // Every vertex is within the tile footprint and outside the hole rim.
        for p in positions {
            assert!(p[0].abs().max(p[2].abs()) <= h + 1e-4, "vertex within the tile footprint");
            assert!(
                (p[0] * p[0] + p[2] * p[2]).sqrt() >= ROOF_HOLE_RADIUS - 1e-4,
                "vertex no nearer the centre than the hole rim",
            );
        }
        // The bottom face (normal -Y) sits at -t and carries the cuboid BOTTOM-face UV.
        let mut saw_bottom = false;
        for ((p, n), uv) in positions.iter().zip(normals).zip(uvs) {
            if *n == [0.0, -1.0, 0.0] {
                saw_bottom = true;
                assert!((p[1] + t).abs() < 1e-5, "bottom face at -t");
                let want = [(h - p[0]) / ROOF_TILE, (h - p[2]) / ROOF_TILE];
                assert!(
                    (uv[0] - want[0]).abs() < 1e-4 && (uv[1] - want[1]).abs() < 1e-4,
                    "bottom UV {uv:?} should match the cuboid bottom face {want:?}"
                );
            }
        }
        assert!(saw_bottom, "the slab has a bottom face");
    }
}
