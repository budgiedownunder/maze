//! Heart health-pickup rig — a procedural red heart (two upper sphere
//! lobes + a downward-pointing pyramid tip) that hovers above the cell
//! with a gentle pulse animation. Matches the red-heart icon used by the
//! 2D editor and 2D game so the visual reads consistently across
//! surfaces.

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::{LevelPlacement, CELL_SIZE};
use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;

// ---------- Tuning constants ----------

/// Sphere radius for each upper "lobe" of the heart (units).
const LOBE_RADIUS: f32 = 0.18;
/// Horizontal lobe offset from the heart's centre (units).
const LOBE_OFFSET_X: f32 = 0.13;
/// Vertical lobe offset above the heart's centre (units).
const LOBE_OFFSET_Y: f32 = 0.10;

/// Downward-pointing pyramid — the heart's "tip" / point. Built as a
/// custom 4-sided pyramid mesh (rather than Bevy's circular `Cone`
/// primitive) so its front and back faces are planar at Z =
/// ±`TIP_HALF_DEPTH_Z`. `TIP_HALF_DEPTH_Z` is chosen so the four corners
/// of the pyramid's top face land exactly on the lobe spheres' surfaces
/// at the join Y — geometrically, for a corner at
/// `(±TIP_HALF_WIDTH_X, top_y, ±TIP_HALF_DEPTH_Z)` to sit on a lobe
/// centred at `(±LOBE_OFFSET_X, LOBE_OFFSET_Y, 0)` we need
/// `TIP_HALF_DEPTH_Z = sqrt(LOBE_RADIUS² − (TIP_HALF_WIDTH_X −
/// LOBE_OFFSET_X)² − (top_y − LOBE_OFFSET_Y)²)`, which evaluates to ≈ 0.10
/// for the current geometry. This gives a flat-faced tip whose top edges
/// tuck flush into the lobes rather than sticking out past their
/// silhouette.
const TIP_HALF_WIDTH_X: f32 = 0.26;
const TIP_HALF_DEPTH_Z: f32 = 0.10;
const TIP_HEIGHT: f32 = 0.45;
/// Vertical offset of the tip's centre below the heart's centre (units).
const TIP_OFFSET_Y: f32 = -0.20;

/// Resting Y position above the floor — keeps the pickup at roughly
/// player-shoulder height so it's clearly visible from the camera.
const HEART_BASE_Y: f32 = 0.9;

/// Heart emissive — bright red so it reads instantly as health.
const HEART_EMISSIVE: LinearRgba = LinearRgba::new(1.4, 0.15, 0.15, 1.0);


pub(crate) struct HeartAssets {
    lobe_mesh: Option<Handle<Mesh>>,
    tip_mesh: Option<Handle<Mesh>>,
    mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_heart_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> HeartAssets {
    let lobe_mesh = meshes.as_mut().map(|m| m.add(Sphere::new(LOBE_RADIUS)));
    let tip_mesh = meshes
        .as_mut()
        .map(|m| m.add(build_tip_pyramid_mesh(TIP_HALF_WIDTH_X, TIP_HALF_DEPTH_Z, TIP_HEIGHT)));
    let mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: HEART_EMISSIVE,
            ..default()
        })
    });
    HeartAssets {
        lobe_mesh,
        tip_mesh,
        mat,
    }
}

/// Spawns the Heart entity hierarchy for the `'H'` cell at `(r, c)`. The
/// marker carries the cell coordinate so the tick driver can despawn this
/// entity on `GameEvent::PlayerHealed { cell, .. }`.
pub(crate) fn spawn_heart(commands: &mut Commands, assets: &HeartAssets, r: usize, c: usize, placement: LevelPlacement) {
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let root = commands
        .spawn((
            super::HealthMarker { cell: (r, c), level: placement.level },
            placement.tag(),
            Transform::from_xyz(x, placement.world_y(HEART_BASE_Y), z),
            Visibility::default(),
        ))
        .id();
    if let (Some(lobe), Some(tip), Some(mat)) = (
        assets.lobe_mesh.clone(),
        assets.tip_mesh.clone(),
        assets.mat.clone(),
    ) {
        commands.entity(root).with_children(|parent| {
            // Two upper lobes — children inherit the root's transform.
            parent.spawn((
                Mesh3d(lobe.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(-LOBE_OFFSET_X, LOBE_OFFSET_Y, 0.0),
            ));
            parent.spawn((
                Mesh3d(lobe),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(LOBE_OFFSET_X, LOBE_OFFSET_Y, 0.0),
            ));
            // Tip pyramid — the custom mesh is built with its apex along
            // +Y to mirror Bevy's `Cone` convention, so rotating it 180°
            // around X flips the apex downward to form the heart's point.
            parent.spawn((
                Mesh3d(tip),
                MeshMaterial3d(mat),
                Transform::from_xyz(0.0, TIP_OFFSET_Y, 0.0)
                    .with_rotation(Quat::from_rotation_x(2.0 * FRAC_PI_2)),
            ));
        });
    }
}

// Idle animation (scale pulse + Y-spin) is driven uniformly across
// every `HealthMarker` by `super::health_animation_system` — see
// [`super`] module docs.

/// Builds a 4-sided pyramid mesh centred on its midpoint, with the apex
/// at `y = +height/2` and a rectangular base at `y = -height/2` spanning
/// `±half_width_x` along X and `±half_depth_z` along Z. Vertices are
/// duplicated per face to give each face a flat shading normal. The
/// asymmetric half-extents (X ≠ Z) are what give the heart its flat
/// front/back profile.
fn build_tip_pyramid_mesh(half_width_x: f32, half_depth_z: f32, height: f32) -> Mesh {
    let half_h = height * 0.5;
    let apex = [0.0_f32, half_h, 0.0];
    let b_fl = [-half_width_x, -half_h, half_depth_z];
    let b_fr = [half_width_x, -half_h, half_depth_z];
    let b_br = [half_width_x, -half_h, -half_depth_z];
    let b_bl = [-half_width_x, -half_h, -half_depth_z];

    // Outward face normals derived from the cross product of two base→apex
    // edges; factored to closed form to avoid recomputing per call.
    let n_front = normalize3([0.0, half_depth_z, height]);
    let n_back = normalize3([0.0, half_depth_z, -height]);
    let n_right = normalize3([height, half_width_x, 0.0]);
    let n_left = normalize3([-height, half_width_x, 0.0]);
    let n_base = [0.0, -1.0, 0.0];

    let positions: Vec<[f32; 3]> = vec![
        b_fl, b_fr, apex, // front (+Z)
        b_fr, b_br, apex, // right (+X)
        b_br, b_bl, apex, // back (-Z)
        b_bl, b_fl, apex, // left (-X)
        b_fl, b_bl, b_br, b_fr, // base (-Y)
    ];
    let normals: Vec<[f32; 3]> = vec![
        n_front, n_front, n_front, n_right, n_right, n_right, n_back, n_back, n_back, n_left,
        n_left, n_left, n_base, n_base, n_base, n_base,
    ];
    let uvs: Vec<[f32; 2]> = vec![
        [0.0, 1.0],
        [1.0, 1.0],
        [0.5, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.5, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.5, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [0.5, 0.0],
        [0.0, 0.0],
        [0.0, 1.0],
        [1.0, 1.0],
        [1.0, 0.0],
    ];
    let indices: Vec<u32> = vec![
        0, 1, 2, // front
        3, 4, 5, // right
        6, 7, 8, // back
        9, 10, 11, // left
        12, 13, 14, 12, 14, 15, // base (two triangles)
    ];

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn normalize3(v: [f32; 3]) -> [f32; 3] {
    let mag = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if mag > 0.0 {
        [v[0] / mag, v[1] / mag, v[2] / mag]
    } else {
        [0.0, 1.0, 0.0]
    }
}
