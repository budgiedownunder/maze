use crate::overlays::win::COLOR_ORB_LIGHT;
use crate::world::CELL_SIZE;
use bevy::prelude::*;

const ORB_BASE_Y: f32 = 1.0;

#[derive(Component)]
pub(crate) struct FinishOrb;

pub(crate) struct OrbAssets {
    pub(crate) mesh: Option<Handle<Mesh>>,
    pub(crate) mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_orb_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> OrbAssets {
    let mesh = meshes.as_mut().map(|m| m.add(Sphere::new(0.35)));
    let mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: Color::BLACK,
            emissive: LinearRgba::new(1.2, 0.9, 0.1, 1.0),
            ..default()
        })
    });
    OrbAssets { mesh, mat }
}

pub(crate) fn spawn_orb(commands: &mut Commands, assets: &OrbAssets, r: usize, c: usize) {
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    match (assets.mesh.clone(), assets.mat.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                FinishOrb,
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(x, ORB_BASE_Y, z),
            ));
        }
        _ => {
            commands.spawn((FinishOrb, Transform::from_xyz(x, ORB_BASE_Y, z)));
        }
    }
    commands.spawn((
        PointLight {
            color: COLOR_ORB_LIGHT,
            intensity: 80_000.0,
            radius: 0.35,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(x, ORB_BASE_Y, z),
    ));
}

pub(crate) fn orb_system(time: Res<Time>, mut orb: Query<&mut Transform, With<FinishOrb>>) {
    if let Ok(mut t) = orb.single_mut() {
        t.translation.y = ORB_BASE_Y + 0.15 * (time.elapsed_secs() * 2.0).sin();
        t.rotate_y(time.delta_secs() * 1.2);
    }
}
