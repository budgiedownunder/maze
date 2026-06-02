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
use crate::world::CELL_SIZE;
use bevy::prelude::*;

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

#[derive(Component)]
pub(crate) struct RoofCell;

pub(crate) struct RoofAssets {
    /// Shared inset-tile slab, used by both roofed sky types.
    mesh: Option<Handle<Mesh>>,
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
    let dungeon_material = if config.sky_type == SkyType::Dungeon {
        dungeon::build_material(materials, images)
    } else {
        None
    };
    RoofAssets {
        mesh,
        dungeon_material,
    }
}

pub(crate) fn spawn_roof_for_cell(
    commands: &mut Commands,
    roof_assets: &RoofAssets,
    wall_assets: &WallAssets,
    grid: &[Vec<char>],
    r: usize,
    c: usize,
    config: &GameConfig,
) {
    let material = match config.sky_type {
        SkyType::Dungeon => roof_assets.dungeon_material.clone(),
        SkyType::Chamber => chamber::material_for_cell(wall_assets, grid, r, c, config),
        _ => return,
    };
    spawn_tile(commands, roof_assets.mesh.clone(), material, r, c);
}

/// Spawns one ceiling tile at the top of the walls over cell `(r, c)`.
fn spawn_tile(
    commands: &mut Commands,
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
    r: usize,
    c: usize,
) {
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    match (mesh, material) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                RoofCell,
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(x, WALL_HEIGHT, z),
            ));
        }
        _ => {
            commands.spawn((RoofCell, Transform::from_xyz(x, WALL_HEIGHT, z)));
        }
    }
}
