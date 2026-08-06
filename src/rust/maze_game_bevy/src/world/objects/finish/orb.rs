use crate::overlays::win::COLOR_ORB_LIGHT;
use crate::palette::EMISSIVE_ONLY_BASE;
use crate::state::GameState;
use crate::world::{icosphere, CELL_SIZE, LevelPlacement};
use bevy::prelude::*;

// ---------- Tuning constants ----------

/// Orb sphere radius (units).
const ORB_RADIUS: f32 = 0.35;
/// Orb resting Y position — hovers near the floor for a distinctive
/// low glow source at the finish cell. Because the orb sits well below
/// the camera's optical axis at close range, perspective projection
/// stretches it into an ellipse the instant the player walks onto the
/// finish cell — [`orb_system`] despawns the orb on `state.won` to
/// avoid that visual.
const ORB_BASE_Y: f32 = 0.7;
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

/// The bobbing finish orb. Stores the run level it sits on so [`orb_system`],
/// which sets the orb's absolute Y each frame from a constant resting height,
/// keeps it at the stacked Y offset rather than snapping back to level 0.
#[derive(Component)]
pub(crate) struct FinishOrb {
    base_y: f32,
}

pub(crate) struct OrbAssets {
    pub(crate) mesh: Option<Handle<Mesh>>,
    pub(crate) mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_orb_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> OrbAssets {
    // The light below casts shadows, so the orb is drawn seven times a frame —
    // once for the view and six for the cube map. Its triangle count is paid
    // over each of them.
    let mesh = meshes.as_mut().map(|m| m.add(icosphere(ORB_RADIUS, 3)));
    let mat = materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive: ORB_EMISSIVE,
            ..default()
        })
    });
    OrbAssets { mesh, mat }
}

/// Native override of [`GameConfig::hide_finish_orb`] — `MAZE_NO_ORB=1`. The
/// browser host sets the flag from `/game/?orb=0`; a native run has no host to
/// do it. Mirrors the `MAZE_DEBUG_MEM` convention, and is ignored under
/// `cfg(test)` for the same reason.
const NO_ORB_ENV: &str = "MAZE_NO_ORB";

/// Whether an env value asks for the orb to be left out. `1` / `true` (any
/// case); anything else, including unset, leaves it in.
pub(crate) fn hide_orb_from(value: Option<&str>) -> bool {
    matches!(value, Some(v) if v.eq_ignore_ascii_case("1") || v.eq_ignore_ascii_case("true"))
}

fn hide_orb_env() -> bool {
    if cfg!(test) {
        return false;
    }
    hide_orb_from(std::env::var(NO_ORB_ENV).ok().as_deref())
}

pub(crate) fn spawn_orb(
    commands: &mut Commands,
    assets: &OrbAssets,
    r: usize,
    c: usize,
    placement: LevelPlacement,
    hide: bool,
) {
    // Skipped rather than hidden: a hidden light still costs its shadow passes
    // in some pipelines, and the point of the flag is to remove the cost, not
    // the pixels. Winning is grid-based, so the finish still works without it.
    if hide || hide_orb_env() {
        return;
    }
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let y = placement.world_y(ORB_BASE_Y);
    // Only the level's floor base is stored: `orb_system` re-derives the bob Y
    // from it; the orb's X/Z (offset above) are fixed for the level's lifetime.
    let base_y = placement.base_y();
    match (assets.mesh.clone(), assets.mat.clone()) {
        (Some(mesh), Some(mat)) => {
            commands.spawn((
                FinishOrb { base_y },
                placement.tag(),
                Mesh3d(mesh),
                MeshMaterial3d(mat),
                Transform::from_xyz(x, y, z),
            ));
        }
        _ => {
            commands.spawn((FinishOrb { base_y }, placement.tag(), Transform::from_xyz(x, y, z)));
        }
    }
    commands.spawn((
        placement.tag(),
        PointLight {
            color: COLOR_ORB_LIGHT,
            intensity: ORB_LIGHT_INTENSITY,
            radius: ORB_LIGHT_RADIUS,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(x, y, z),
    ));
}

pub(crate) fn orb_system(
    mut commands: Commands,
    time: Res<Time>,
    state: Res<GameState>,
    mut orb: Query<(Entity, &mut Transform, &FinishOrb)>,
) {
    let Ok((entity, mut t, finish_orb)) = orb.single_mut() else {
        return;
    };
    // On win, despawn the orb so its near-floor position doesn't read
    // as a stretched ellipse at the close-range, off-axis viewing
    // angle the player ends up at. The point light at the same
    // position is left alone — the lingering glow reads as a
    // celebratory effect at the finish cell.
    if state.won {
        commands.entity(entity).despawn();
        return;
    }
    t.translation.y =
        finish_orb.base_y + ORB_BASE_Y + BOB_AMPLITUDE * (time.elapsed_secs() * BOB_RATE).sin();
    t.rotate_y(time.delta_secs() * SPIN_RATE);
}
