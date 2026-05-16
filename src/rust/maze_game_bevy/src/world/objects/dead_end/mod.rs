pub(crate) mod brazier;
pub(crate) mod chest;
pub(crate) mod pillar;
pub(crate) mod urn;

use crate::state::GameConfig;
use crate::world::CELL_SIZE;
use bevy::prelude::*;

// Dead-end landmark object variants. Each cell flagged as a dead-end
// (passable cell with exactly one open neighbour, excluding start/finish)
// hashes (row, col, seed) to pick one of these object kinds. Variants
// build from the shared cylinder / cuboid primitives.
pub(crate) const DEAD_END_OBJECT_VARIANTS: u32 = 4;

#[derive(Component)]
pub(crate) struct DeadEndObject;

pub(crate) struct DeadEndAssets {
    pub(crate) cylinder: Option<Handle<Mesh>>,
    pub(crate) cuboid: Option<Handle<Mesh>>,
    pub(crate) stone_mat: Option<Handle<StandardMaterial>>,
    pub(crate) glow_mat: Option<Handle<StandardMaterial>>,
    pub(crate) urn_mat: Option<Handle<StandardMaterial>>,
    pub(crate) pillar_mat: Option<Handle<StandardMaterial>>,
    pub(crate) chest_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_dead_end_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> DeadEndAssets {
    // A unit cylinder + unit cuboid that every dead-end object scales to
    // taste. One shared mesh per shape (instead of one per object kind)
    // keeps the asset count flat.
    let cylinder = meshes.as_mut().map(|m| m.add(Cylinder::new(0.5, 1.0)));
    let cuboid = meshes.as_mut().map(|m| m.add(Cuboid::new(1.0, 1.0, 1.0)));
    DeadEndAssets {
        cylinder,
        cuboid,
        stone_mat: brazier::build_stone_material(materials),
        glow_mat: brazier::build_glow_material(materials),
        urn_mat: urn::build_urn_material(materials),
        pillar_mat: pillar::build_pillar_material(materials),
        chest_mat: chest::build_chest_material(materials),
    }
}

/// Deterministic hash of `(row, col, seed)` → dead-end object kind in
/// `0..DEAD_END_OBJECT_VARIANTS`. Different constants from
/// `wall_tint_index` so the object kind and the cell tint don't
/// correlate visually.
pub(crate) fn dead_end_object_index(r: usize, c: usize, seed: u64) -> u32 {
    let mut h = seed.wrapping_mul(0x6EED_0E9D_A4D9_4A4F);
    h = h.wrapping_add((r as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    h = h.wrapping_add((c as u64).wrapping_mul(0xC6BC_279E_C8C9_D5B1));
    h ^= h >> 30;
    h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    h ^= h >> 27;
    (h % DEAD_END_OBJECT_VARIANTS as u64) as u32
}

/// `true` when `(r, c)` is a dead-end cell — a passable cell whose four
/// orthogonal neighbours include exactly one other passable cell. Start
/// and finish cells are excluded by the caller, not here, so this helper
/// stays purely topological.
pub(crate) fn is_dead_end(grid: &[Vec<char>], r: usize, c: usize) -> bool {
    let rows = grid.len();
    let cols = if grid.is_empty() { 0 } else { grid[0].len() };
    if r >= rows || c >= cols || grid[r][c] == 'W' {
        return false;
    }
    let mut open = 0u32;
    if r > 0 && grid[r - 1][c] != 'W' {
        open += 1;
    }
    if r + 1 < rows && grid[r + 1][c] != 'W' {
        open += 1;
    }
    if c > 0 && grid[r][c - 1] != 'W' {
        open += 1;
    }
    if c + 1 < cols && grid[r][c + 1] != 'W' {
        open += 1;
    }
    open == 1
}

pub(crate) fn spawn_object(
    commands: &mut Commands,
    mesh: Option<Handle<Mesh>>,
    mat: Option<Handle<StandardMaterial>>,
    pos: Vec3,
    scale: Vec3,
) {
    match (mesh, mat) {
        (Some(m), Some(mt)) => {
            commands.spawn((
                DeadEndObject,
                Mesh3d(m),
                MeshMaterial3d(mt),
                Transform::from_translation(pos).with_scale(scale),
            ));
        }
        _ => {
            commands.spawn((DeadEndObject, Transform::from_translation(pos)));
        }
    }
}

pub(crate) fn spawn_dead_end_object_for_cell(
    commands: &mut Commands,
    assets: &DeadEndAssets,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    config: &GameConfig,
) {
    // A single distinctive object per dead-end cell — brazier / urn /
    // broken pillar / chest, picked by hashing (row, col, seed). Skipped
    // for start / finish cells (the player stands on start, the finish
    // has the orb) and when the per-difficulty toggle is off.
    if !config.landmarks.dead_end_objects || cell == 'S' || cell == 'F' || !is_dead_end(grid, r, c) {
        return;
    }
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    let kind = dead_end_object_index(r, c, config.seed);
    match kind {
        0 => brazier::spawn_brazier(commands, assets, x, z),
        1 => urn::spawn_urn(commands, assets, x, z),
        2 => pillar::spawn_pillar(commands, assets, x, z),
        _ => chest::spawn_chest(commands, assets, x, z),
    }
}
