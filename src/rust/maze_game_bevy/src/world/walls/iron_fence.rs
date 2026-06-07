//! Non-occluding iron-fence cell — grilles of thin vertical iron bars rising the
//! full wall height, placed on the cell's *open edges* (like a door's leaves)
//! over a *normal floor* (unlike water/lava, the fence stands on a regular floor
//! tile, spawned by the caller). A grille is drawn on an edge facing a *passable*
//! cell, a water/lava **pool**, or the **maze perimeter** (sky shows through the
//! bars) — never toward a solid wall or another iron fence (the run stays
//! continuous). The bars are sparse enough
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

/// Which of the cell's four edges carry a bar grille, as `[north, south, east,
/// west]`. An edge is barred when its in-bounds neighbour can be looked across (an
/// open cell or a water/lava pool), or when it's the maze perimeter under an open
/// sky (sky beyond, seen through the bars). A solid wall and another iron fence
/// (looked through, not across) yield no grille; and under an **enclosed** sky the
/// perimeter is walled (a solid panel) instead of barred, so the edge isn't a
/// grille there either.
fn edges_barred(
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    r: usize,
    c: usize,
) -> [bool; 4] {
    let rows = grid.len();
    let cols = grid[r].len();
    let across = |nr: usize, nc: usize| can_be_looked_across(grid, cell_entities, config, nr, nc);
    // The maze perimeter shows bars (sky through them) only under an open sky;
    // under Dungeon / Chamber it is walled instead (the solid panel from `face`).
    let perimeter = !config.sky_type.is_enclosed();
    [
        if r == 0 { perimeter } else { across(r - 1, c) },
        if r + 1 >= rows { perimeter } else { across(r + 1, c) },
        if c + 1 >= cols { perimeter } else { across(r, c + 1) },
        if c == 0 { perimeter } else { across(r, c - 1) },
    ]
}

/// Spawns the iron-bar grilles for cell `(r, c)`: one row of bars on each edge
/// facing a passable cell, a water/lava pool, or the maze perimeter (the player
/// sees sky through the bars there). The caller spawns the floor tile separately
/// (the fence stands on a normal floor).
pub(crate) fn spawn_iron_fence(
    commands: &mut Commands,
    assets: &IronFenceAssets,
    grid: &[Vec<char>],
    cell_entities: &HashMap<(usize, usize), Vec<CellEntity>>,
    config: &GameConfig,
    r: usize,
    c: usize,
) {
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

    let [bar_n, bar_s, bar_e, bar_w] = edges_barred(grid, cell_entities, config, r, c);

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
        if bar_n {
            for i in 0..BARS_PER_EDGE {
                spawn_bar(bar_offset(i), -HALF_CELL);
            }
        }
        if bar_s {
            for i in 0..BARS_PER_EDGE {
                spawn_bar(bar_offset(i), HALF_CELL);
            }
        }
        if bar_e {
            for i in 0..BARS_PER_EDGE {
                spawn_bar(HALF_CELL, bar_offset(i));
            }
        }
        if bar_w {
            for i in 0..BARS_PER_EDGE {
                spawn_bar(-HALF_CELL, bar_offset(i));
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edges_barred_includes_the_maze_perimeter() {
        // Fence at the top-left corner: N and W are the grid boundary (barred — sky
        // through the bars), S and E face open cells (barred). All four barred.
        let grid = vec![vec!['W', ' '], vec![' ', 'F']];
        let config = GameConfig::default();
        let empty = HashMap::new();
        assert_eq!(edges_barred(&grid, &empty, &config, 0, 0), [true; 4]);
    }

    #[test]
    fn edges_barred_skips_solid_walls() {
        // A fence walled in on every side by solid 'W' cells: no grille anywhere
        // (the solid panels are the barrier; nothing to see across or through).
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', 'W', 'W'],
            vec!['W', 'W', 'W'],
        ];
        let config = GameConfig::default();
        let empty = HashMap::new();
        assert_eq!(edges_barred(&grid, &empty, &config, 1, 1), [false; 4]);
    }

    #[test]
    fn edges_barred_walls_the_perimeter_under_enclosed_sky() {
        // Corner fence under Chamber: the N & W grid-boundary edges are walled (not
        // barred — a solid panel takes their place), while the S & E open edges
        // still bar.
        let grid = vec![vec!['W', ' '], vec![' ', 'F']];
        let empty = HashMap::new();
        let config = GameConfig {
            sky_type: crate::state::SkyType::Chamber,
            ..GameConfig::default()
        };
        assert_eq!(
            edges_barred(&grid, &empty, &config, 0, 0),
            [false, true, true, false]
        );
    }
}
