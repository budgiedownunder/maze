pub(crate) mod brazier;
pub(crate) mod chest;
pub(crate) mod pillar;
pub(crate) mod urn;

use crate::palette::EMISSIVE_ONLY_BASE;
use crate::state::{GameConfig, GridFacing};
use crate::world::{initial_facing, CELL_SIZE};
use bevy::prelude::*;
use bevy::render::render_resource::Face;
use std::f32::consts::{FRAC_PI_2, PI};

// Dead-end landmark object variants. Each cell flagged as a dead-end
// (passable cell with exactly one open neighbour, excluding start/finish)
// hashes (row, col, seed) to pick one of these object kinds. Variants
// build from the shared cylinder / cuboid / cone primitives.
pub(crate) const DEAD_END_OBJECT_VARIANTS: u32 = 4;

// ---------- Outline tuning ----------

/// Uniform scale-up factor applied to each sibling outline mesh. The
/// outline shell uses the original body's mesh handle scaled by this
/// factor with `cull_mode: Some(Face::Front)`, so the only visible part
/// of the outline is a thin dark rim poking out around the body's
/// silhouette — i.e. the classic inverted-hull outline trick.
pub(crate) const OUTLINE_SCALE: f32 = 1.06;

/// Base colour of the shared outline material. Outline meshes are
/// `unlit: true`, so the rendered output is purely this colour (no
/// lighting interaction).
const OUTLINE_BASE_COLOR: Color = Color::BLACK;

#[derive(Component)]
pub(crate) struct DeadEndObject;

/// Marker for brazier bowls. Queried by [`brazier_flicker_system`] to
/// modulate the shared glow material each frame. Halo entities don't
/// carry this marker — the halo uses a separate, steady material that
/// frames the flickering bowl.
#[derive(Component)]
pub(crate) struct BrazierBowl;

pub(crate) struct DeadEndAssets {
    pub(crate) cylinder: Option<Handle<Mesh>>,
    pub(crate) cuboid: Option<Handle<Mesh>>,
    pub(crate) cone: Option<Handle<Mesh>>,
    pub(crate) stone_mat: Option<Handle<StandardMaterial>>,
    pub(crate) glow_mat: Option<Handle<StandardMaterial>>,
    pub(crate) halo_mat: Option<Handle<StandardMaterial>>,
    pub(crate) urn_mat: Option<Handle<StandardMaterial>>,
    pub(crate) dark_terracotta_mat: Option<Handle<StandardMaterial>>,
    pub(crate) pillar_mat: Option<Handle<StandardMaterial>>,
    pub(crate) groove_mat: Option<Handle<StandardMaterial>>,
    pub(crate) chest_mat: Option<Handle<StandardMaterial>>,
    pub(crate) lid_mat: Option<Handle<StandardMaterial>>,
    pub(crate) hinge_mat: Option<Handle<StandardMaterial>>,
    pub(crate) leather_mat: Option<Handle<StandardMaterial>>,
    pub(crate) lock_mat: Option<Handle<StandardMaterial>>,
    pub(crate) outline_mat: Option<Handle<StandardMaterial>>,
    pub(crate) pillar_outline_mat: Option<Handle<StandardMaterial>>,
}

pub(crate) fn build_dead_end_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> DeadEndAssets {
    // Shared unit primitives: every dead-end sub-mesh transforms one of
    // these via `Transform::with_scale` instead of materialising its own
    // mesh asset, keeping the mesh count flat.
    let cylinder = meshes.as_mut().map(|m| m.add(Cylinder::new(0.5, 1.0)));
    let cuboid = meshes.as_mut().map(|m| m.add(Cuboid::new(1.0, 1.0, 1.0)));
    let cone = meshes.as_mut().map(|m| m.add(Cone::new(0.5, 1.0)));
    DeadEndAssets {
        cylinder,
        cuboid,
        cone,
        stone_mat: brazier::build_stone_material(materials),
        glow_mat: brazier::build_glow_material(materials),
        halo_mat: brazier::build_halo_material(materials),
        urn_mat: urn::build_urn_material(materials),
        dark_terracotta_mat: urn::build_dark_terracotta_material(materials),
        pillar_mat: pillar::build_pillar_material(materials),
        groove_mat: pillar::build_groove_material(materials),
        chest_mat: chest::build_chest_material(materials),
        lid_mat: chest::build_lid_material(materials),
        hinge_mat: chest::build_hinge_material(materials),
        leather_mat: chest::build_leather_material(materials),
        lock_mat: chest::build_lock_material(materials),
        outline_mat: build_outline_material(materials, OUTLINE_BASE_COLOR),
        pillar_outline_mat: build_outline_material(materials, pillar::OUTLINE_BASE_COLOR),
    }
}

fn build_outline_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    base_color: Color,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color,
            unlit: true,
            // Render only the back faces of the enlarged outline shell.
            // The visible portion is the thin rim that pokes past the
            // body's silhouette — classic inverted-hull outline.
            cull_mode: Some(Face::Front),
            ..default()
        })
    })
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

/// Spawns a body entity (carrying [`DeadEndObject`] + the caller's
/// `extras` bundle, typically `()` or a marker like [`BrazierBowl`]) and
/// a sibling outline entity (no `DeadEndObject` marker, reuses the same
/// mesh handle with the outline material at [`OUTLINE_SCALE`]).
///
/// The outline transform preserves the body's translation + rotation
/// and multiplies its scale by [`OUTLINE_SCALE`] so the outline shell
/// wraps the body cleanly regardless of orientation.
pub(crate) fn spawn_with_outline<B: Bundle>(
    commands: &mut Commands,
    mesh: Option<Handle<Mesh>>,
    body_mat: Option<Handle<StandardMaterial>>,
    outline_mat: Option<Handle<StandardMaterial>>,
    body_xform: Transform,
    extras: B,
) {
    let outline_xform = Transform {
        translation: body_xform.translation,
        rotation: body_xform.rotation,
        scale: body_xform.scale * OUTLINE_SCALE,
    };
    match (mesh.clone(), body_mat) {
        (Some(m), Some(mt)) => {
            commands.spawn((
                DeadEndObject,
                Mesh3d(m),
                MeshMaterial3d(mt),
                body_xform,
                extras,
            ));
        }
        _ => {
            commands.spawn((DeadEndObject, body_xform, extras));
        }
    }
    if let (Some(m), Some(mt)) = (mesh, outline_mat) {
        // Outline deliberately does NOT carry `DeadEndObject` so the
        // existing `count(DeadEndObject)` tests stay accurate.
        commands.spawn((Mesh3d(m), MeshMaterial3d(mt), outline_xform));
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
    // has the orb), for key / door cells (they own the dead-end with their
    // holder / panel — a key is commonly placed in a dead-end), for enemy
    // and health-pickup cells (the goblin / heart entity owns the cell's
    // visual), and when the per-difficulty toggle is off.
    if !config.landmarks.dead_end_objects
        || matches!(cell, 'S' | 'F' | 'K' | 'D' | 'E' | 'H')
        || !is_dead_end(grid, r, c)
    {
        return;
    }
    let x = c as f32 * CELL_SIZE + 1.0;
    let z = r as f32 * CELL_SIZE + 1.0;
    let kind = dead_end_object_index(r, c, config.seed);
    match kind {
        0 => brazier::spawn_brazier(commands, assets, x, z),
        1 => urn::spawn_urn(commands, assets, x, z),
        2 => pillar::spawn_pillar(commands, assets, x, z),
        _ => chest::spawn_chest(commands, assets, x, z, chest_yaw_for_open_neighbour(grid, r, c)),
    }
}

/// Rotation around Y to orient the chest's local +Z (lock face) toward
/// the cell's single open neighbour, so the player walking into the
/// dead-end sees the keyhole rather than a blank back face.
///
/// Coordinate mapping: in the maze grid, row+1 is world +Z (south) and
/// col+1 is world +X (east). Bevy's `Quat::from_rotation_y(θ)` rotates
/// +Z toward +X for positive θ, so a yaw of `π/2` rotates the lock face
/// from default south (+Z) to east (+X), and so on around the compass.
fn chest_yaw_for_open_neighbour(grid: &[Vec<char>], r: usize, c: usize) -> f32 {
    // `initial_facing` cycles S→E→N→W and returns the first open
    // neighbour. For a dead-end (exactly one open neighbour) the result
    // is unique, which is exactly what we want here.
    match initial_facing(grid, r, c) {
        GridFacing::South => 0.0,
        GridFacing::East => FRAC_PI_2,
        GridFacing::North => PI,
        GridFacing::West => -FRAC_PI_2,
    }
}

/// Modulates the shared brazier glow material's emissive each frame
/// with two slightly detuned sine waves, giving a non-uniform flicker
/// without an explicit PRNG. Every brazier in the maze shares the same
/// `glow_mat` handle, so a single material update animates them all in
/// lockstep — finding any [`BrazierBowl`] entity is enough to get the
/// handle.
pub(crate) fn brazier_flicker_system(
    time: Res<Time>,
    bowls: Query<&MeshMaterial3d<StandardMaterial>, With<BrazierBowl>>,
    materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    // `Assets<StandardMaterial>` only exists when the PBR / asset
    // plugins are loaded. Tests using `MinimalPlugins` don't have it,
    // so the parameter is `Option<ResMut<…>>` and the system no-ops.
    let Some(mut materials) = materials else { return };
    let Some(handle) = bowls.iter().next() else {
        return;
    };
    let Some(mat) = materials.get_mut(&handle.0) else {
        return;
    };
    let factor = brazier::flicker_factor(time.elapsed_secs());
    mat.emissive = LinearRgba::new(
        brazier::GLOW_EMISSIVE.red * factor,
        brazier::GLOW_EMISSIVE.green * factor,
        brazier::GLOW_EMISSIVE.blue * factor,
        1.0,
    );
}

/// Small helper so per-object modules can build their tinted
/// emissive-only materials in a single line without re-importing the
/// palette constant or the `StandardMaterial { … }` literal.
pub(crate) fn build_emissive_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    emissive: LinearRgba,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: EMISSIVE_ONLY_BASE,
            emissive,
            ..default()
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_end_object_index_is_deterministic() {
        let seed = 0xCAFEu64;
        assert_eq!(
            dead_end_object_index(3, 5, seed),
            dead_end_object_index(3, 5, seed)
        );
    }

    #[test]
    fn dead_end_object_index_always_in_range() {
        for r in 0..30 {
            for c in 0..30 {
                let kind = dead_end_object_index(r, c, 0x9999u64);
                assert!(kind < DEAD_END_OBJECT_VARIANTS, "got kind {kind}");
            }
        }
    }

    #[test]
    fn is_dead_end_single_open_neighbour() {
        // (1,1) has only south open
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', ' ', 'W'],
            vec!['W', ' ', 'W'],
        ];
        assert!(is_dead_end(&grid, 1, 1));
    }

    #[test]
    fn is_dead_end_corridor_false() {
        // (1,1) has east AND west open — two-way corridor, not a dead end
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec![' ', ' ', ' '],
            vec!['W', 'W', 'W'],
        ];
        assert!(!is_dead_end(&grid, 1, 1));
    }

    #[test]
    fn is_dead_end_junction_false() {
        // (1,1) has three open neighbours — T-junction, not a dead end
        let grid = vec![
            vec!['W', ' ', 'W'],
            vec![' ', ' ', ' '],
            vec!['W', 'W', 'W'],
        ];
        assert!(!is_dead_end(&grid, 1, 1));
    }

    #[test]
    fn is_dead_end_isolated_false() {
        // No open neighbours
        let grid = vec![
            vec!['W', 'W', 'W'],
            vec!['W', ' ', 'W'],
            vec!['W', 'W', 'W'],
        ];
        assert!(!is_dead_end(&grid, 1, 1));
    }

    #[test]
    fn is_dead_end_corner_with_one_neighbour() {
        // Top-left cell; grid boundary counts as wall; only south open
        let grid = vec![vec![' ', 'W'], vec![' ', 'W']];
        assert!(is_dead_end(&grid, 0, 0));
    }

    #[test]
    fn is_dead_end_on_wall_false() {
        let grid = vec![vec!['W', 'W'], vec!['W', ' ']];
        assert!(!is_dead_end(&grid, 0, 0));
    }

    #[test]
    fn is_dead_end_out_of_bounds_false() {
        let grid = vec![vec![' ']];
        assert!(!is_dead_end(&grid, 5, 5));
    }

    #[test]
    fn flicker_factor_stays_within_amplitude_envelope() {
        // The phase term `sin(a) + 0.4 * sin(b)` is bounded in [-1.4, 1.4];
        // `factor = 1 + AMPLITUDE * phase * 0.5` is therefore bounded in
        // `[1 - 0.7*AMP, 1 + 0.7*AMP]`. Sweep `t` to spot-check.
        let max_phase = 1.4_f32;
        let upper = 1.0 + brazier::FLICKER_AMPLITUDE * max_phase * 0.5;
        let lower = 1.0 - brazier::FLICKER_AMPLITUDE * max_phase * 0.5;
        // Step through several seconds of game time.
        for i in 0..2000 {
            let t = i as f32 * 0.013;
            let f = brazier::flicker_factor(t);
            assert!(
                f <= upper + 1e-4 && f >= lower - 1e-4,
                "factor {f} out of envelope [{lower}, {upper}] at t={t}"
            );
        }
    }
}
