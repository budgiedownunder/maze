pub(crate) mod finish;
pub(crate) mod hatch;
pub(crate) mod lines;
pub(crate) mod start;
pub(crate) mod tile;

use crate::world::textures::tile::make_tile_texture;
use crate::world::walls::rim::RECESS_DEPTH;
use crate::world::walls::WALL_THICKNESS;
use crate::world::{LevelPlacement, CELL_SIZE, POOL_GAP};
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Floor-tile mesh thickness (units, vertical extent). Thin enough to
/// look flat from the player's eye height without z-fighting the grid
/// lines that sit just above it.
pub(crate) const FLOOR_THICKNESS: f32 = 0.01;

/// Fraction of a start / finish tile's thickness given to a plain-stone underside
/// cap. The cell then reads as ordinary floor from the level below — so an open
/// multi-level stack doesn't reveal the upper level's start / finish through the
/// floor — while its coloured top still shows from above. The remainder is the
/// coloured layer; the two stack flush within the normal tile thickness, so all
/// floor tiles share one height.
const STONE_CAP_FRAC: f32 = 0.35;

#[derive(Component)]
pub(crate) struct FloorCell;

/// Marker on a pool cell's outer floor-edge seal — the floor-stone wall that
/// closes a floating (lifted) pool cell's exposed basin side, below the rim, so
/// from the level below (or its exposed surrounding ring under taper) the cell
/// reads as the level's solid floor edge rather than glowing liquid. Distinct from
/// [`FloorCell`] / [`crate::world::UndersideSeal`]; tagged so the rendering tests
/// can count them.
#[derive(Component)]
pub(crate) struct PoolEdgeSeal;

/// Spawns a start / finish tile as two flush layers inside the normal tile
/// thickness: a plain-stone underside cap (untagged scenery) and the coloured top
/// `mat`, tagged with `marker` (`StartCell` / `FinishCell`) + [`FloorCell`]. From
/// below the cell looks like ordinary stone; from above it shows its start /
/// finish colour. With no render assets (headless), falls back to a single tagged
/// `FloorCell` at the tile centre — identical to the plain-tile path, so the
/// entity counts/positions the tests assert are unchanged.
pub(crate) fn spawn_capped_tile<M: Bundle>(
    commands: &mut Commands,
    assets: &FloorAssets,
    mat: Option<Handle<StandardMaterial>>,
    marker: M,
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let y = placement.world_y(0.0);
    match (assets.floor_mesh.clone(), assets.tile_mat.clone(), mat) {
        (Some(mesh), Some(stone_mat), Some(top_mat)) => {
            let bottom = y - FLOOR_THICKNESS / 2.0;
            let cap_h = FLOOR_THICKNESS * STONE_CAP_FRAC;
            let top_h = FLOOR_THICKNESS - cap_h;
            // Plain-stone underside cap — the bottom slice, what the level below sees.
            commands.spawn((
                placement.tag(),
                Transform::from_xyz(x, bottom + cap_h / 2.0, z)
                    .with_scale(Vec3::new(1.0, STONE_CAP_FRAC, 1.0)),
                Mesh3d(mesh.clone()),
                MeshMaterial3d(stone_mat),
            ));
            // Coloured top — the logical floor cell, what the player on this level sees.
            commands.spawn((
                marker,
                FloorCell,
                placement.tag(),
                Transform::from_xyz(x, bottom + cap_h + top_h / 2.0, z)
                    .with_scale(Vec3::new(1.0, 1.0 - STONE_CAP_FRAC, 1.0)),
                Mesh3d(mesh),
                MeshMaterial3d(top_mat),
            ));
        }
        _ => {
            commands.spawn((marker, FloorCell, placement.tag(), Transform::from_xyz(x, y, z)));
        }
    }
}

pub(crate) struct FloorAssets {
    pub(crate) floor_mesh: Option<Handle<Mesh>>,
    /// Thin vertical wall (cell-wide, `POOL_GAP − RECESS_DEPTH` tall) for a pool
    /// cell's outer floor-edge seal on a north / south edge; `ew` for east / west.
    pool_edge_ns_mesh: Option<Handle<Mesh>>,
    pool_edge_ew_mesh: Option<Handle<Mesh>>,
    pub(crate) tile_mat: Option<Handle<StandardMaterial>>,
    pub(crate) start_mat: Option<Handle<StandardMaterial>>,
    pub(crate) finish_mat: Option<Handle<StandardMaterial>>,
    pub(crate) lines: lines::LineAssets,
    /// Round-hatch meshes + materials (start cells above a ladder finish).
    pub(crate) hatch: hatch::HatchAssets,
}

pub(crate) fn build_floor_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    images: &mut Option<ResMut<Assets<Image>>>,
) -> FloorAssets {
    // Thin cuboid floor tile — Plane3d does not resolve reliably in the asset
    // pipeline. Shared by tile / start / finish.
    let floor_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, FLOOR_THICKNESS, CELL_SIZE)));
    // Pool floor-edge seal walls: cell-wide, spanning from the recessed surface
    // down to the lift's gap bottom (the rim covers floor→surface above them).
    let edge_h = POOL_GAP - RECESS_DEPTH;
    let pool_edge_ns_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, edge_h, WALL_THICKNESS)));
    let pool_edge_ew_mesh = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(WALL_THICKNESS, edge_h, CELL_SIZE)));
    // Tile texture is shared by tile / start / finish materials — build once.
    let tile_tex = images.as_mut().map(|imgs| make_tile_texture(imgs));
    FloorAssets {
        floor_mesh,
        pool_edge_ns_mesh,
        pool_edge_ew_mesh,
        tile_mat: tile::build_tile_material(materials, &tile_tex),
        start_mat: start::build_start_material(materials, &tile_tex),
        finish_mat: finish::build_finish_material(materials, &tile_tex),
        lines: lines::build_line_assets(meshes, materials),
        hatch: hatch::build_hatch_assets(meshes, materials, &tile_tex),
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_floor_for_cell(
    commands: &mut Commands,
    assets: &FloorAssets,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    placement: LevelPlacement,
    // True for a start cell sitting above a ladder finish on the level below: the
    // solid start tile is replaced by an (open) hatch lid the climb emerges
    // through. Ignored for every other cell.
    hatch_at_start: bool,
    // True when the level below is roofed — the hatch then leaves its underside
    // to that level's holed roof tile. Only meaningful with `hatch_at_start`.
    below_roofed: bool,
    // How far this level was lifted for its pools — the hatch drops its underside
    // cap by this so it's flush with the surrounding sealed cells.
    gap: f32,
) {
    lines::spawn_lines_for_cell(commands, &assets.lines, grid, r, c, placement);
    match cell {
        'S' if hatch_at_start => hatch::spawn_hatch(commands, assets, r, c, placement, below_roofed, gap),
        'S' => start::spawn_start(commands, assets, r, c, placement),
        'F' => finish::spawn_finish(commands, assets, r, c, placement),
        _ => tile::spawn_tile(commands, assets, r, c, placement),
    }
}

/// Seals the exposed (grid-boundary) outer sides of a pool cell `(r, c)` on a
/// level lifted by `gap` to hold its pools. The rim ([`crate::world::walls::rim`])
/// already fills the band from the floor down to the recessed surface; below that
/// the basin's outer side is open, and on a floating level — a tapered upper level
/// over a larger one — that side faces the level below's exposed surrounding ring,
/// so the glowing liquid shows through. A floor-stone wall on each grid-boundary
/// edge, spanning the recessed surface down to the gap bottom and flush with the
/// floor edge, makes the cell read as the level's solid floor edge from below and
/// all the way around. Interior edges are backed by the neighbour's own seal, so
/// only the grid boundary needs one. No-op off a lifted level (`gap == 0`).
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_pool_edge_seal(
    commands: &mut Commands,
    assets: &FloorAssets,
    r: usize,
    c: usize,
    rows: usize,
    cols: usize,
    placement: LevelPlacement,
    gap: f32,
) {
    let seal_h = gap - RECESS_DEPTH;
    if seal_h <= 0.0 {
        return;
    }
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    // Top at the recessed surface, bottom at the gap bottom (where the underside cap
    // sits), so the rim above and the cap below meet it flush.
    let centre_y = placement.world_y(-RECESS_DEPTH - seal_h / 2.0);
    // Inset half the wall thickness so the outer face lands exactly on the cell
    // boundary, in line with the surrounding floor / underside-seal edges.
    let edge = CELL_SIZE / 2.0 - WALL_THICKNESS / 2.0;
    // Scale the (POOL_GAP-tall) mesh to this lift's gap, so it stays correct if the
    // gap ever differs from POOL_GAP.
    let scale_y = seal_h / (POOL_GAP - RECESS_DEPTH);
    // The seal is spawned once and the render assets added to it, rather than
    // built separately per branch: an entity assembled twice can have the two
    // copies drift apart, and a level tag present on only the asset-less branch
    // is invisible to every headless test while leaving the shipped geometry
    // impossible to hide.
    let mut seal = |mesh: &Option<Handle<Mesh>>, pos: Vec3| {
        let mut entity = commands.spawn((
            PoolEdgeSeal,
            placement.tag(),
            Transform::from_translation(pos).with_scale(Vec3::new(1.0, scale_y, 1.0)),
        ));
        if let (Some(mesh), Some(mat)) = (mesh.clone(), assets.tile_mat.clone()) {
            entity.insert((Mesh3d(mesh), MeshMaterial3d(mat)));
        }
    };
    if r == 0 {
        seal(&assets.pool_edge_ns_mesh, Vec3::new(x, centre_y, z - edge));
    }
    if r + 1 >= rows {
        seal(&assets.pool_edge_ns_mesh, Vec3::new(x, centre_y, z + edge));
    }
    if c + 1 >= cols {
        seal(&assets.pool_edge_ew_mesh, Vec3::new(x + edge, centre_y, z));
    }
    if c == 0 {
        seal(&assets.pool_edge_ew_mesh, Vec3::new(x - edge, centre_y, z));
    }
}
