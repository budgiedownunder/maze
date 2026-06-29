use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::{LevelPlacement, CELL_SIZE, HALF_CELL};
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Floor-line strip width (units, perpendicular to its run direction).
const LINE_W: f32 = 0.06;
/// Floor-line strip thickness (units, vertical extent).
const LINE_H: f32 = 0.01;
/// Y position of the line strip — slightly above the floor tile (top at
/// y=0.005) so the strips render in front without z-fighting.
const LINE_Y: f32 = 0.015;

/// Emissive RGB for the floor-line strip — neutral mid-grey so it reads
/// as a faint orientation cue rather than a coloured accent.
const LINE_EMISSIVE: LinearRgba = LinearRgba::new(0.28, 0.28, 0.28, 1.0);

#[derive(Component)]
pub(crate) struct FloorLine;

pub(crate) struct LineAssets {
    pub(crate) line_ew: Option<Handle<Mesh>>,
    pub(crate) line_ns: Option<Handle<Mesh>>,
    pub(crate) line_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_line_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> LineAssets {
    // E-W strip (runs along X), N-S strip (runs along Z).
    let line_ew = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(CELL_SIZE, LINE_H, LINE_W)));
    let line_ns = meshes
        .as_mut()
        .map(|m| m.add(Cuboid::new(LINE_W, LINE_H, CELL_SIZE)));
    let line_mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: LINE_EMISSIVE,
            ..default()
        })
    });
    LineAssets { line_ew, line_ns, line_mat }
}

fn spawn_line(
    commands: &mut Commands,
    mesh: Option<Handle<Mesh>>,
    mat: Option<Handle<StandardMaterial>>,
    pos: Vec3,
) {
    match (mesh, mat) {
        (Some(m), Some(mt)) => {
            commands.spawn((
                FloorLine,
                Transform::from_translation(pos),
                Mesh3d(m),
                MeshMaterial3d(mt),
            ));
        }
        _ => {
            commands.spawn((FloorLine, Transform::from_translation(pos)));
        }
    }
}

pub(crate) fn spawn_lines_for_cell(
    commands: &mut Commands,
    assets: &LineAssets,
    grid: &[Vec<char>],
    r: usize,
    c: usize,
    placement: LevelPlacement,
) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let line_y = placement.world_y(LINE_Y);

    // Each shared edge is spawned once: always South + East; North/West only
    // when the neighbour in that direction is a wall or grid boundary.
    spawn_line(
        commands,
        assets.line_ew.clone(),
        assets.line_mat.clone(),
        Vec3::new(x, line_y,z + HALF_CELL),
    );
    spawn_line(
        commands,
        assets.line_ns.clone(),
        assets.line_mat.clone(),
        Vec3::new(x + HALF_CELL, line_y,z),
    );
    if r == 0 || grid[r - 1][c] == 'W' {
        spawn_line(
            commands,
            assets.line_ew.clone(),
            assets.line_mat.clone(),
            Vec3::new(x, line_y,z - HALF_CELL),
        );
    }
    if c == 0 || grid[r][c - 1] == 'W' {
        spawn_line(
            commands,
            assets.line_ns.clone(),
            assets.line_mat.clone(),
            Vec3::new(x - HALF_CELL, line_y,z),
        );
    }
}
