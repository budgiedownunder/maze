//! Rim skirts for the recessed water / lava pools.
//!
//! A pool surface sits in a recess [`RECESS_DEPTH`] below floor level; with the
//! wall panels around it suppressed, the vertical band between the waterline and
//! the floor would otherwise show the black void. A rim skirt is a short vertical
//! wall filling that band. It is drawn on every edge **except** the edge between
//! two pools **of the same type** (so a run of water — or a run of lava — stays one
//! continuous sunken basin).
//!
//! In particular the border between a water cell and a lava cell **does** get a
//! rim (one skirt from each side, so the two never read as a single merged pool),
//! and the **maze perimeter** (grid edge) gets a rim too — framing an edge pool's
//! basin while the open sky still shows above the low rim (no solid wall is drawn
//! over a non-occluding edge cell — see [`super::solid::spawn_walls_for_cell`]).
//!
//! Each pool type textures its rim distinctly (a cool wet-stone tint for water, a
//! hot charred-basalt tint for lava) over the shared rough rock texture, so the
//! basin wall reads apart from the edge it borders. Skirts are inset half their
//! thickness into their own cell, so two opposing rims at a water↔lava border sit
//! back-to-back instead of z-fighting in the same plane.

use super::{pool_type_at, WALL_THICKNESS};
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::state::{GameConfig, WallType};
use crate::world::textures::rock::make_rock_texture;
use crate::world::{LevelPlacement, CELL_SIZE, HALF_CELL};
use bevy::math::Affine2;
use bevy::prelude::*;
use maze::CellEntity;
use std::collections::HashMap;

// ---------- Tuning constants ----------

/// How far below floor level (`y = 0`) a pool's waterline sits. The rim skirt
/// spans this band, from the floor down to the recessed surface. Shared with the
/// water / lava surface modules so the surface and its rim agree on the depth.
pub(crate) const RECESS_DEPTH: f32 = 0.3;

/// Texture repeats across one rim skirt (`u`, `v`). The rock tiles a few times
/// across the cell-wide face and once down the short band.
const RIM_UV: Vec2 = Vec2::new(3.0, 1.0);

/// Water rim emissive — a cool, dim blue-grey "wet stone" so the basin wall reads
/// as damp and distinct from the brick / tile it borders.
const WATER_RIM_EMISSIVE: LinearRgba = LinearRgba::new(0.16, 0.22, 0.34, 1.0);

/// Lava rim emissive — a dark charred basalt with a faint orange heat-glow, so
/// the rim reads as scorched rock around the molten pool.
const LAVA_RIM_EMISSIVE: LinearRgba = LinearRgba::new(0.42, 0.17, 0.06, 1.0);

/// Marker on a pool rim-skirt segment. Distinct from [`super::WallCell`] so it
/// isn't counted as a solid wall panel; the animation systems leave it alone.
#[derive(Component)]
pub(crate) struct PoolRim;

pub(crate) struct RimAssets {
    /// Skirt running along X (drawn on a north / south edge).
    ns_mesh: Option<Handle<Mesh>>,
    /// Skirt running along Z (drawn on an east / west edge).
    ew_mesh: Option<Handle<Mesh>>,
    water_mat: Option<Handle<StandardMaterial>>,
    lava_mat: Option<Handle<StandardMaterial>>,
}

fn build_rim_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    texture: &Option<Handle<Image>>,
    emissive: LinearRgba,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive,
            emissive_texture: texture.clone(),
            uv_transform: Affine2::from_scale(RIM_UV),
            ..default()
        })
    })
}

pub(crate) fn build_rim_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> RimAssets {
    // Short skirts (height = RECESS_DEPTH) at the cell boundary; full cell width
    // so adjacent rim segments meet at corners.
    let ns_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, RECESS_DEPTH, WALL_THICKNESS)));
    let ew_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(WALL_THICKNESS, RECESS_DEPTH, CELL_SIZE)));
    let rock = images.as_mut().map(|imgs| make_rock_texture(imgs));
    RimAssets {
        ns_mesh,
        ew_mesh,
        water_mat: build_rim_material(materials, &rock, WATER_RIM_EMISSIVE),
        lava_mat: build_rim_material(materials, &rock, LAVA_RIM_EMISSIVE),
    }
}

fn spawn_skirt(
    commands: &mut Commands,
    mesh: Option<Handle<Mesh>>,
    mat: Option<Handle<StandardMaterial>>,
    pos: Vec3,
) {
    match (mesh, mat) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((PoolRim, Transform::from_translation(pos), Mesh3d(mesh), MeshMaterial3d(mat)));
        }
        _ => {
            commands.spawn((PoolRim, Transform::from_translation(pos)));
        }
    };
}

/// Spawns the rim skirts around the pool cell `(r, c)`: a short basin wall on
/// every edge except one facing a same-type pool or the grid edge. `wall_type`
/// (`Water` or `Lava`) picks the rim texture tint and the same-type test.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_pool_rim(
    commands: &mut Commands,
    assets: &RimAssets,
    wall_type: WallType,
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let rows = grid.len();
    let cols = grid[r].len();
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let y = placement.world_y(-RECESS_DEPTH / 2.0);
    let mat = match wall_type {
        WallType::Lava => &assets.lava_mat,
        _ => &assets.water_mat,
    };
    // Inset each skirt half its thickness into the cell so a water↔lava border's
    // two opposing skirts sit back-to-back rather than coplanar (no z-fight).
    let inset = WALL_THICKNESS / 2.0;
    let edge = HALF_CELL - inset;

    // A rim is drawn on every edge unless the neighbour is an in-bounds pool of the
    // *same* type — so a run of the same liquid stays one basin, while a water↔lava
    // border (different types) and the maze perimeter (grid edge) are both walled.
    let rimmed = |nr: usize, nc: usize| {
        pool_type_at(grid, cell_entities, config, nr, nc) != Some(wall_type)
    };
    if r == 0 || rimmed(r - 1, c) {
        spawn_skirt(commands, assets.ns_mesh.clone(), mat.clone(), Vec3::new(x, y, z - edge));
    }
    if r + 1 >= rows || rimmed(r + 1, c) {
        spawn_skirt(commands, assets.ns_mesh.clone(), mat.clone(), Vec3::new(x, y, z + edge));
    }
    if c + 1 >= cols || rimmed(r, c + 1) {
        spawn_skirt(commands, assets.ew_mesh.clone(), mat.clone(), Vec3::new(x + edge, y, z));
    }
    if c == 0 || rimmed(r, c - 1) {
        spawn_skirt(commands, assets.ew_mesh.clone(), mat.clone(), Vec3::new(x - edge, y, z));
    }
}
