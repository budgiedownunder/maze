//! Non-occluding iron-fence cell — grilles of thin vertical iron bars rising the
//! full wall height, placed on the cell's *open edges* (like a door's leaves)
//! over a *normal floor* (unlike water/lava, the fence stands on a regular floor
//! tile, spawned by the caller). A grille is drawn on an edge facing a *passable*
//! cell or a water/lava **pool** — never toward a solid wall, another iron fence
//! (the run stays continuous), or the maze perimeter. The bars are sparse enough
//! to see through, and with the wall panels around the cell suppressed (see
//! [`super::solid::spawn_walls_for_cell`]) the player sees across the cell to
//! whatever lies beyond — a visual barrier, not a sight barrier.

use super::{can_be_looked_across, WALL_HEIGHT};
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::state::GameConfig;
use crate::world::{CELL_SIZE, HALF_CELL};
use bevy::prelude::*;
use maze::CellEntity;
use std::collections::HashMap;

// ---------- Tuning constants ----------

/// Bars per edge grille. Spread across the cell width they read as a row of iron
/// bars while still leaving gaps to see through.
const BARS_PER_EDGE: usize = 7;

/// Square cross-section side of each bar (units). Thin so the grille is mostly
/// open air.
const BAR_THICKNESS: f32 = 0.07;

/// Bar emissive — a dark cool grey so the iron reads as a dim metal silhouette
/// rather than a glowing accent.
const IRON_EMISSIVE: LinearRgba = LinearRgba::new(0.18, 0.19, 0.22, 1.0);

/// Marker on the root of an iron-fence cell. Spawned per non-occluding
/// iron-fence `'W'` cell; the bars hang as children so the whole set of grilles
/// can be addressed (and despawned) through this one entity.
#[derive(Component)]
pub(crate) struct IronFenceBars;

pub(crate) struct IronFenceAssets {
    bar_mesh: Option<Handle<Mesh>>,
    bar_material: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_iron_fence_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> IronFenceAssets {
    let bar_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(BAR_THICKNESS, WALL_HEIGHT, BAR_THICKNESS)));
    let bar_material = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: IRON_EMISSIVE,
            ..default()
        })
    });
    IronFenceAssets {
        bar_mesh,
        bar_material,
    }
}

/// Offset (relative to the cell centre) of bar `i` along an edge, spreading the
/// `BARS_PER_EDGE` bars evenly across the cell with a half-step margin at each
/// end.
fn bar_offset(i: usize) -> f32 {
    let step = CELL_SIZE / BARS_PER_EDGE as f32;
    (i as f32 + 0.5) * step - HALF_CELL
}

/// Spawns the iron-bar grilles for cell `(r, c)`: one row of bars on each edge
/// facing a passable cell or a water/lava pool. The caller spawns the floor tile
/// separately (the fence stands on a normal floor).
pub(crate) fn spawn_iron_fence(
    commands: &mut Commands,
    assets: &IronFenceAssets,
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    r: usize,
    c: usize,
) {
    let rows = grid.len();
    let cols = grid[r].len();
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    let root = commands
        .spawn((
            IronFenceBars,
            Transform::from_xyz(x, WALL_HEIGHT / 2.0, z),
            Visibility::default(),
        ))
        .id();
    let (Some(bar_mesh), Some(bar_material)) = (assets.bar_mesh.clone(), assets.bar_material.clone())
    else {
        return;
    };

    // An edge is barred when its (in-bounds) neighbour can be looked across — an
    // open cell or a water/lava pool. A solid wall, another iron fence (looked
    // through, not across), and the maze perimeter all yield no grille.
    let barred = |nr: usize, nc: usize| can_be_looked_across(grid, cell_entities, config, nr, nc);

    // N/S grilles run their bars along local X at z ±HALF_CELL; E/W grilles run
    // along local Z at x ±HALF_CELL. Positions are local to the cell-centre root.
    commands.entity(root).with_children(|parent| {
        let mut spawn_bar = |lx: f32, lz: f32| {
            parent.spawn((
                Mesh3d(bar_mesh.clone()),
                MeshMaterial3d(bar_material.clone()),
                Transform::from_xyz(lx, 0.0, lz),
            ));
        };
        if r > 0 && barred(r - 1, c) {
            for i in 0..BARS_PER_EDGE {
                spawn_bar(bar_offset(i), -HALF_CELL);
            }
        }
        if r + 1 < rows && barred(r + 1, c) {
            for i in 0..BARS_PER_EDGE {
                spawn_bar(bar_offset(i), HALF_CELL);
            }
        }
        if c + 1 < cols && barred(r, c + 1) {
            for i in 0..BARS_PER_EDGE {
                spawn_bar(HALF_CELL, bar_offset(i));
            }
        }
        if c > 0 && barred(r, c - 1) {
            for i in 0..BARS_PER_EDGE {
                spawn_bar(-HALF_CELL, bar_offset(i));
            }
        }
    });
}
