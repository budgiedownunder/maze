use crate::overlays::win::COLOR_ORB_LIGHT;
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::world::CELL_SIZE;
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Orb sphere radius (units).
const ORB_RADIUS: f32 = 0.35;
/// Orb resting Y position. Set to the player's eye height so the orb
/// sits on the camera's optical axis — perspective projection only
/// renders a sphere as a perfect circle when it's at the optical
/// centre; at any other position the off-axis angle stretches the
/// sphere into an ellipse, and the effect is most visible on narrow
/// (portrait / phone) viewports where the vertical FOV is widest.
/// Floating the orb at eye level keeps it circular on every aspect.
const ORB_BASE_Y: f32 = 1.7;
/// Orb emissive RGB — warm gold.
const ORB_EMISSIVE: LinearRgba = LinearRgba::new(1.2, 0.9, 0.1, 1.0);

/// Per-second bob frequency (radians).
const BOB_RATE: f32 = 2.0;
/// Bob amplitude (units of vertical travel from the resting Y).
const BOB_AMPLITUDE: f32 = 0.15;
/// Per-second rotation rate around Y (radians).
const SPIN_RATE: f32 = 1.2;

/// Point light intensity at the orb (lumens-ish; Bevy PBR units).
const ORB_LIGHT_INTENSITY: f32 = 80_000.0;
/// Point light source radius (units) — softens the shadow penumbra.
const ORB_LIGHT_RADIUS: f32 = 0.35;

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
    let mesh = meshes.as_mut().map(|m| m.add(Sphere::new(ORB_RADIUS)));
    let mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: ORB_EMISSIVE,
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
            intensity: ORB_LIGHT_INTENSITY,
            radius: ORB_LIGHT_RADIUS,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(x, ORB_BASE_Y, z),
    ));
}

pub(crate) fn orb_system(time: Res<Time>, mut orb: Query<&mut Transform, With<FinishOrb>>) {
    if let Ok(mut t) = orb.single_mut() {
        t.translation.y = ORB_BASE_Y + BOB_AMPLITUDE * (time.elapsed_secs() * BOB_RATE).sin();
        t.rotate_y(time.delta_secs() * SPIN_RATE);
    }
}
