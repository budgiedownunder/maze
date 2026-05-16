use crate::world::{CELL_SIZE, HALF_CELL};
use bevy::prelude::*;

const LINE_W: f32 = 0.06;
const LINE_H: f32 = 0.01;
const LINE_Y: f32 = 0.015;

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
            base_color: Color::BLACK,
            emissive: LinearRgba::new(0.28, 0.28, 0.28, 1.0),
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
) {
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;

    // Each shared edge is spawned once: always South + East; North/West only
    // when the neighbour in that direction is a wall or grid boundary.
    spawn_line(
        commands,
        assets.line_ew.clone(),
        assets.line_mat.clone(),
        Vec3::new(x, LINE_Y, z + HALF_CELL),
    );
    spawn_line(
        commands,
        assets.line_ns.clone(),
        assets.line_mat.clone(),
        Vec3::new(x + HALF_CELL, LINE_Y, z),
    );
    if r == 0 || grid[r - 1][c] == 'W' {
        spawn_line(
            commands,
            assets.line_ew.clone(),
            assets.line_mat.clone(),
            Vec3::new(x, LINE_Y, z - HALF_CELL),
        );
    }
    if c == 0 || grid[r][c - 1] == 'W' {
        spawn_line(
            commands,
            assets.line_ns.clone(),
            assets.line_mat.clone(),
            Vec3::new(x - HALF_CELL, LINE_Y, z),
        );
    }
}
