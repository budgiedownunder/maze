//! Treasure objects for `'T'` cells.
//!
//! Each uncollected treasure renders as an **open chest** (the shared
//! [`crate::world::objects::common::chest`] rig with its lid swung open) with a
//! pile of loot heaped almost overflowing inside. The chest is common to every
//! style; the loot pile is style-specific and lives in a sibling module chosen
//! by [`crate::state::TreasureStyle`]:
//!
//! - [`silver`] / [`gold`] — a mound of metallic coins.
//! - [`diamonds`] — a mound of clear faceted gems (bipyramids).
//! - [`jewels`] — a mound of multi-coloured gems.
//!
//! The per-style modules only supply their material(s); the loot geometry, the
//! open chest, the radiating sparkle ring, and the collection flourish are
//! defined once here.
//!
//! **Baked loot meshes.** A full pile is hundreds of tiny pieces, but the pile
//! layout is identical for every chest, so each pile is *baked once* into a
//! single combined [`Mesh`] (coins → one mesh; gems → one mesh per colour group)
//! and every chest references those shared handles. A chest therefore spawns
//! only a handful of loot entities (one per mesh + its outline) instead of one
//! per piece, keeping the entity count — and so the per-frame transform /
//! culling cost — low even with many treasures in a maze.
//!
//! Thin emissive light lines rise from points spread across the loot surface,
//! each fluctuating in length and fading out with distance (a base→tip alpha
//! gradient under additive blending) — a rising-and-falling glow in the loot's
//! own colour, beneath a tinted point light, so the loot reads as *glowing* with
//! treasure ([`treasure_sparkle_system`]).
//!
//! The chest faces outward from a dead-end (and along a corridor) via the same
//! [`crate::world::objects::common::yaw_toward_open_neighbour`] the dead-end /
//! key-holder chests use, so the open mouth and contents face the approaching
//! player.
//!
//! Treasure is auto-collected by walking onto it (like keys). The **open chest
//! itself stands free and stays behind, emptied** — only the loot + sparkles
//! sit under the collectible [`TreasureMarker`] root.
//! [`crate::tick::game_tick_system`] tags that root with [`CollectingTreasure`]
//! on the `TreasureCollected` event, and [`treasure_collection_system`] plays a
//! brief rise-and-shrink flourish before despawning it — leaving the open empty
//! chest. The engine clears the `'T'` cell on the first walk-over, so an emptied
//! chest is never re-collected. The score the treasure adds is folded into
//! [`maze::MazeGame::score`] by the engine, so the HUD updates automatically.

pub(crate) mod diamonds;
pub(crate) mod gold;
pub(crate) mod jewels;
pub(crate) mod silver;

use super::common::bake::{baked_handle, outline_scaled};
use super::common::{self, CommonObjectAssets};
use crate::state::TreasureStyle;
use crate::world::{LevelPlacement, CELL_SIZE};
use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use std::f32::consts::{PI, TAU};

// ---------- Tuning constants ----------

/// Coin piece scale (a thin disc) and gem piece size.
const COIN_SCALE: Vec3 = Vec3::new(0.056, 0.013, 0.056);
const GEM_SIZE: f32 = 0.05;

/// Number of colour groups the gem pile is split into — the loot is baked into
/// this many combined meshes so [`jewels`] can paint each group a different
/// colour (and [`diamonds`] reuses one colour across them). Matches the jewel
/// palette length.
const N_GEM_GROUPS: usize = 4;

/// Collection flourish duration (seconds) — matches the key holder.
const COLLECT_DURATION: f32 = 0.35;
/// How far (world units) the loot rises over the flourish as it shrinks away.
const COLLECT_RISE: f32 = 1.2;
/// Spin rate (rad/s) during collection — a faster "snatched away" feel.
const COLLECT_SPIN_RATE: f32 = 10.0;

/// Tinted point-light glow above the loot. No shadows — it's a small accent.
const GLOW_INTENSITY: f32 = 42_000.0;
const GLOW_RADIUS: f32 = 0.45;

/// Radiating sparkles: tiny emissive spheres that fly outward from the loot and
/// shrink away, looping on staggered phases — a cheap "glinting treasure"
/// shimmer (no particle system). Mirrors the key holder's sparks. The colour(s)
/// come from the per-style module so the shimmer matches the loot. The number of
/// rays per chest is chosen at spawn time (see rays_per_chest) rather than fixed.
/// Radiating light lines: thin emissive beams planted across the loot surface (a
/// phyllotaxis disc of `SPARK_FOOTPRINT_R`), each fluctuating in length so the
/// treasure shimmers. The beam mesh is a unit cylinder scaled thin × length ×
/// thin and rotated to point along its ray. The colour comes from the per-style
/// module.
const SPARK_LINE_THICK: f32 = 0.02;
const SPARK_LINE_LEN: f32 = 0.40;
const SPARK_FLICKER_SPEED: f32 = 1.5;
const SPARK_FOOTPRINT_R: f32 = 0.30;
/// The loot surface the beams are planted on: peak height at the centre,
/// dropping toward the rim, roughly following the mound.
const SPARK_SURFACE_PEAK_Y: f32 = 0.58;
const SPARK_SURFACE_DROP: f32 = 0.22;

/// Loot mound layers `(y, nx, nz, x_extent, z_extent)` filling the open trunk:
/// a full-width base brimming the chest, tapering as it rises to a peak almost
/// at the height of the open lid. Expanded into a dense, deterministic pile by
/// [`loot_positions`] (no RNG, so it's identical every load — which is what lets
/// the pile be baked into shared meshes).
const LOOT_LAYERS: &[(f32, usize, usize, f32, f32)] = &[
    (0.06, 14, 10, 0.32, 0.22),
    (0.16, 14, 10, 0.32, 0.22),
    (0.26, 12, 8, 0.29, 0.19),
    (0.36, 10, 8, 0.25, 0.16),
    (0.46, 8, 6, 0.20, 0.12),
    (0.55, 6, 4, 0.13, 0.08),
    (0.62, 4, 2, 0.06, 0.0),
];

/// Expands [`LOOT_LAYERS`] into individual local-frame loot positions, with a
/// small deterministic per-piece jitter so the pile doesn't read as a rigid
/// grid.
fn loot_positions() -> Vec<Vec3> {
    let mut out = Vec::new();
    let frac = |i: usize, n: usize, extent: f32| {
        if n > 1 {
            (i as f32 / (n - 1) as f32 - 0.5) * 2.0 * extent
        } else {
            0.0
        }
    };
    let mut i = 0usize;
    for &(y, nx, nz, ex, ez) in LOOT_LAYERS {
        for ix in 0..nx {
            for iz in 0..nz {
                let jx = (i as f32 * 1.7).sin() * 0.025;
                let jz = (i as f32 * 2.3).cos() * 0.025;
                out.push(Vec3::new(frac(ix, nx, ex) + jx, y, frac(iz, nz, ez) + jz));
                i += 1;
            }
        }
    }
    out
}

/// The local transform of coin piece `i` at `pos` — a thin disc with a small
/// deterministic tilt so the pile doesn't look stamped.
fn coin_transform(i: usize, pos: Vec3) -> Transform {
    let tilt_x = (i as f32 * 1.3).sin() * 0.5;
    let tilt_z = (i as f32 * 2.1).cos() * 0.5;
    Transform::from_translation(pos)
        .with_rotation(Quat::from_rotation_x(tilt_x) * Quat::from_rotation_z(tilt_z))
        .with_scale(COIN_SCALE)
}

/// The two cone transforms (crown apex-up + pavilion apex-down) forming the
/// faceted gem bipyramid at `pos`.
fn gem_transforms(pos: Vec3) -> [Transform; 2] {
    let crown_h = 0.5 * GEM_SIZE;
    let pavilion_h = 0.7 * GEM_SIZE;
    [
        Transform::from_translation(pos + Vec3::Y * (0.5 * crown_h))
            .with_scale(Vec3::new(GEM_SIZE, crown_h, GEM_SIZE)),
        Transform::from_translation(pos - Vec3::Y * (0.5 * pavilion_h))
            .with_rotation(Quat::from_rotation_x(PI))
            .with_scale(Vec3::new(GEM_SIZE, pavilion_h, GEM_SIZE)),
    ]
}

/// Per-cell anchor for the collectible loot. One is spawned at each `'T'` cell,
/// owning the loot pile + sparkle ring (its children) but NOT the open chest,
/// which stands free. Collecting the treasure tags this entity with
/// [`CollectingTreasure`], which despawns it (and its loot) once the flourish
/// finishes, leaving the emptied open chest behind.
#[derive(Component)]
pub(crate) struct TreasureMarker {
    pub(crate) cell: (usize, usize),
    /// Which level this chest belongs to.
    pub(crate) level: usize,
    /// This chest's level floor Y (`base_level_y[level]`). The collectible loot
    /// root rests at the floor and [`treasure_collection_system`] rewrites its
    /// absolute Y during the rise flourish, so it must re-apply this base.
    pub(crate) base_y: f32,
}

/// Tags each baked loot mesh (the coin pile, or one gem colour group). Lets the
/// headless tests count loot meshes to confirm the per-style rig family was
/// dispatched, without inspecting materials — the same role
/// [`super::enemy::ghost::GhostTag`] plays for the enemy rigs.
#[derive(Component)]
pub(crate) struct TreasureLoot;

/// Tags a treasure whose loot was just collected. [`treasure_collection_system`]
/// rises and shrinks the loot over [`COLLECT_DURATION`], then despawns it.
#[derive(Component, Default)]
pub(crate) struct CollectingTreasure {
    elapsed: f32,
}

/// One light line rising from the loot surface, animated by
/// [`treasure_sparkle_system`].
#[derive(Component)]
pub(crate) struct TreasureSparkle {
    /// Local-frame surface point the line is planted on (its bright base end).
    base: Vec3,
    /// Direction it points along (mostly up, fanning slightly outward).
    dir: Vec3,
    /// Phase offset (`0.0..1.0`) so the lines fluctuate out of lockstep.
    phase: f32,
}

/// Composite treasure assets — one material sub-struct per style (built in its
/// own module) plus the shared baked loot meshes and sparkle mesh. The baked
/// piles are built once and shared by every chest.
pub(crate) struct TreasureAssets {
    silver: silver::SilverAssets,
    gold: gold::GoldAssets,
    diamonds: diamonds::DiamondsAssets,
    jewels: jewels::JewelsAssets,
    coin_body: Option<Handle<Mesh>>,
    coin_outline: Option<Handle<Mesh>>,
    gem_bodies: Vec<Option<Handle<Mesh>>>,
    gem_outlines: Vec<Option<Handle<Mesh>>>,
    sparkle_mesh: Option<Handle<Mesh>>,
}

/// Shared references handed to each style's `spawn_*` so it can place its loot
/// meshes + sparkles without threading every handle through the call.
pub(crate) struct LootContext<'a> {
    coin_body: &'a Option<Handle<Mesh>>,
    coin_outline: &'a Option<Handle<Mesh>>,
    gem_bodies: &'a [Option<Handle<Mesh>>],
    gem_outlines: &'a [Option<Handle<Mesh>>],
    outline_mat: &'a Option<Handle<StandardMaterial>>,
    sparkle_mesh: &'a Option<Handle<Mesh>>,
    /// Sparkle rays this chest spawns (uniform across the maze; see rays_per_chest).
    ray_count: usize,
}

/// Builds a metallic loot material (silver / gold coins). Shared by the coin
/// styles via `super::metal_material`. A partial `metallic` (rather than a pure
/// `1.0` metal, which has no diffuse term and so just mirrors the dark corridor)
/// keeps a coin sheen while letting ambient + the treasure glow light the coins
/// diffusely; the caller's bright `emissive` then ensures they read clearly even
/// in unlit corridors.
fn metal_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    base_color: Color,
    emissive: LinearRgba,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color,
            metallic: 0.5,
            perceptual_roughness: 0.4,
            emissive,
            ..default()
        })
    })
}

/// Builds a glassy gem loot material (diamonds / jewels). Shared by the gem
/// styles via `super::gem_material`.
fn gem_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    base_color: Color,
    emissive: LinearRgba,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color,
            metallic: 0.0,
            perceptual_roughness: 0.1,
            emissive,
            ..default()
        })
    })
}

/// Builds a treasure-glow material: an unlit, additively-blended colour, so the
/// light lines read as glow that brightens the dark corridor and fades out along
/// each beam (via the beam mesh's base→tip alpha gradient). `color` is the
/// loot's colour, shared by the styles via `super::sparkle_material`.
fn sparkle_material(
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
    color: Color,
) -> Option<Handle<StandardMaterial>> {
    materials.as_mut().map(|m| {
        m.add(StandardMaterial {
            base_color: color,
            unlit: true,
            alpha_mode: AlphaMode::Add,
            ..default()
        })
    })
}

/// Builds the glow-beam mesh: a unit cylinder along `+Y` carrying a vertex-colour
/// alpha gradient — opaque at the base end (`y = -0.5`, planted on the loot) and
/// fading to zero at the tip (`y = +0.5`) — so an additively-blended material
/// renders a line that tails off the further it gets from the surface.
fn beam_mesh() -> Mesh {
    let mut mesh = Mesh::from(Cylinder::new(0.5, 1.0));
    let colors: Vec<[f32; 4]> = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(positions)) => positions
            .iter()
            .map(|p| [1.0, 1.0, 1.0, (0.5 - p[1]).clamp(0.0, 1.0)])
            .collect(),
        _ => Vec::new(),
    };
    if !colors.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
    mesh
}

pub(crate) fn build_treasure_assets(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    materials: &mut Option<ResMut<Assets<StandardMaterial>>>,
) -> TreasureAssets {
    let positions = loot_positions();

    // Coin pile → one combined body mesh + one outline mesh (shared by silver
    // and gold).
    let coin_base = Mesh::from(Cylinder::new(0.5, 1.0));
    let coin_xforms: Vec<Transform> = positions
        .iter()
        .enumerate()
        .map(|(i, &p)| coin_transform(i, p))
        .collect();
    let coin_body = baked_handle(meshes, &coin_base, &coin_xforms);
    let coin_outline = baked_handle(meshes, &coin_base, &outline_scaled(&coin_xforms));

    // Gem pile → one combined body + outline mesh per colour group (so jewels
    // can paint each group; diamonds reuses one colour across them).
    let gem_base = Mesh::from(Cone::new(0.5, 1.0));
    let mut gem_bodies = Vec::with_capacity(N_GEM_GROUPS);
    let mut gem_outlines = Vec::with_capacity(N_GEM_GROUPS);
    for g in 0..N_GEM_GROUPS {
        let xforms: Vec<Transform> = positions
            .iter()
            .enumerate()
            .filter(|(i, _)| i % N_GEM_GROUPS == g)
            .flat_map(|(_, &p)| gem_transforms(p))
            .collect();
        gem_bodies.push(baked_handle(meshes, &gem_base, &xforms));
        gem_outlines.push(baked_handle(meshes, &gem_base, &outline_scaled(&xforms)));
    }

    TreasureAssets {
        silver: silver::build_silver_assets(materials),
        gold: gold::build_gold_assets(materials),
        diamonds: diamonds::build_diamonds_assets(materials),
        jewels: jewels::build_jewels_assets(materials),
        coin_body,
        coin_outline,
        gem_bodies,
        gem_outlines,
        sparkle_mesh: meshes.as_mut().map(|m| m.add(beam_mesh())),
    }
}

/// The warm/cool point-light tint paired with each treasure style.
fn glow_color(style: TreasureStyle) -> Color {
    match style {
        TreasureStyle::Silver => Color::srgb(0.70, 0.75, 0.85),
        TreasureStyle::Gold => Color::srgb(1.0, 0.85, 0.40),
        TreasureStyle::Diamonds => Color::srgb(0.50, 0.90, 1.0),
        TreasureStyle::Jewels => Color::srgb(0.90, 0.60, 0.95),
    }
}

/// The additive-blended sparkle rays are the heavy per-frame cost (transparent
/// overdraw), so on a treasure-dense maze a real iPhone's GPU drowns where the
/// desktop simulator sails past. Each chest gets `rays_per_chest` rays — the same
/// count for every chest in a maze (so they look uniform), the total stays
/// bounded, and sparse mazes still get the full per-chest cap. Every chest also
/// keeps one point light; lights are far cheaper than the sparkle overdraw.
const MAX_RAYS_PER_CHEST: usize = 5;
const MAX_TOTAL_RAYS: usize = 120;

/// Sparkle rays each chest gets for a maze holding `num_chests` treasures:
/// `min(MAX_RAYS_PER_CHEST, MAX_TOTAL_RAYS / num_chests)`.
pub(crate) fn rays_per_chest(num_chests: usize) -> usize {
    (MAX_TOTAL_RAYS / num_chests.max(1)).min(MAX_RAYS_PER_CHEST)
}

/// Sparkle rays each chest gets across a whole multi-level run, given the
/// per-level treasure counts. Every level's treasure renders at once, so the
/// [`MAX_TOTAL_RAYS`] overdraw budget is global to the stack — bound by the
/// total treasure over all levels, not the per-level count. A single-level run
/// is identical to [`rays_per_chest`] of that level's count.
pub(crate) fn run_treasure_rays(level_treasure_counts: impl IntoIterator<Item = usize>) -> usize {
    rays_per_chest(level_treasure_counts.into_iter().sum())
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_treasure_for_cell(
    commands: &mut Commands,
    assets: &TreasureAssets,
    common_assets: &CommonObjectAssets,
    style: TreasureStyle,
    grid: &[Vec<char>],
    cell: char,
    r: usize,
    c: usize,
    // Sparkle rays for this chest — uniform across the maze (see rays_per_treasure).
    ray_count: usize,
    placement: LevelPlacement,
) {
    if cell != 'T' {
        return;
    }
    let x = placement.world_x(c as f32 * CELL_SIZE + 1.0);
    let z = placement.world_z(r as f32 * CELL_SIZE + 1.0);
    let base_y = placement.base_y();
    let yaw = common::yaw_toward_open_neighbour(grid, r, c);

    // The open chest stands free so it stays behind, emptied, after collection.
    // It takes the (already offset) world `x`/`z` and the level for its own Y.
    common::chest::spawn_chest(commands, common_assets, x, z, yaw, common::chest::ChestLid::Open, base_y);

    // Collectible loot root — positioned + yawed at the cell so its children use
    // the same local frame as the chest interior. The flourish rises/shrinks
    // this root, leaving the chest.
    let root = commands
        .spawn((
            TreasureMarker { cell: (r, c), level: placement.level, base_y },
            Transform::from_xyz(x, placement.world_y(0.0), z).with_rotation(Quat::from_rotation_y(yaw)),
            Visibility::default(),
        ))
        .id();

    // Style-tinted glow above the loot — one point light per chest.
    let glow = commands
        .spawn((
            PointLight {
                color: glow_color(style),
                intensity: GLOW_INTENSITY,
                radius: GLOW_RADIUS,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_xyz(0.0, SPARK_SURFACE_PEAK_Y, 0.0),
        ))
        .id();
    commands.entity(root).add_child(glow);

    // Style-specific loot pile + matching sparkle ring inside the open trunk.
    let ctx = LootContext {
        coin_body: &assets.coin_body,
        coin_outline: &assets.coin_outline,
        gem_bodies: &assets.gem_bodies,
        gem_outlines: &assets.gem_outlines,
        outline_mat: &common_assets.outline_mat,
        sparkle_mesh: &assets.sparkle_mesh,
        ray_count,
    };
    match style {
        TreasureStyle::Silver => silver::spawn_silver(commands, root, &assets.silver, &ctx),
        TreasureStyle::Gold => gold::spawn_gold(commands, root, &assets.gold, &ctx),
        TreasureStyle::Diamonds => diamonds::spawn_diamonds(commands, root, &assets.diamonds, &ctx),
        TreasureStyle::Jewels => jewels::spawn_jewels(commands, root, &assets.jewels, &ctx),
    }
}

/// Spawns the shared coin pile (body + outline meshes) under `root`, in
/// `coin_mat`. Shared by the [`silver`] / [`gold`] styles via
/// `super::spawn_coin_loot`.
fn spawn_coin_loot(
    commands: &mut Commands,
    root: Entity,
    ctx: &LootContext,
    coin_mat: &Option<Handle<StandardMaterial>>,
) {
    spawn_loot_mesh(commands, root, ctx.coin_body, ctx.coin_outline, coin_mat, ctx.outline_mat);
}

/// Spawns the shared gem pile — one baked mesh per colour group — under `root`,
/// cycling through `mats` (one material for [`diamonds`], the palette for
/// [`jewels`]). Shared via `super::spawn_gem_loot`.
fn spawn_gem_loot(
    commands: &mut Commands,
    root: Entity,
    ctx: &LootContext,
    mats: &[Option<Handle<StandardMaterial>>],
) {
    let n = mats.len().max(1);
    for g in 0..ctx.gem_bodies.len() {
        spawn_loot_mesh(
            commands,
            root,
            &ctx.gem_bodies[g],
            &ctx.gem_outlines[g],
            &mats[g % n],
            ctx.outline_mat,
        );
    }
}

/// Spawns one baked loot mesh as a `TreasureLoot`-tagged child of `root`, paired
/// with its inverted-hull outline. The tag (and an identity transform) is
/// attached even when render assets are absent (headless tests).
fn spawn_loot_mesh(
    commands: &mut Commands,
    root: Entity,
    body_mesh: &Option<Handle<Mesh>>,
    outline_mesh: &Option<Handle<Mesh>>,
    body_mat: &Option<Handle<StandardMaterial>>,
    outline_mat: &Option<Handle<StandardMaterial>>,
) {
    let mut body = commands.spawn((TreasureLoot, Transform::default(), Visibility::default()));
    if let (Some(mesh), Some(mat)) = (body_mesh.clone(), body_mat.clone()) {
        body.insert((Mesh3d(mesh), MeshMaterial3d(mat)));
    }
    let body = body.id();
    commands.entity(root).add_child(body);

    if let (Some(mesh), Some(mat)) = (outline_mesh.clone(), outline_mat.clone()) {
        let edge = commands
            .spawn((Mesh3d(mesh), MeshMaterial3d(mat), Transform::default()))
            .id();
        commands.entity(root).add_child(edge);
    }
}

/// Plants [`TreasureSparkle`] glow-beams across the loot surface, parented to a
/// group under `root` so they ride the collection flourish with the loot. Each
/// beam is oriented along its ray and cycles through `mats` (one colour for the
/// coin / diamond styles, the full palette for jewels) so the glow matches the
/// treasure's colour(s). Shared by the per-style modules via
/// `super::spawn_sparkles`.
fn spawn_sparkles(
    commands: &mut Commands,
    root: Entity,
    mesh: &Option<Handle<Mesh>>,
    mats: &[Option<Handle<StandardMaterial>>],
    ray_count: usize,
) {
    let group = commands
        .spawn((Transform::default(), Visibility::default()))
        .id();
    commands.entity(root).add_child(group);

    let Some(mesh) = mesh.clone() else {
        return;
    };
    if mats.is_empty() || ray_count == 0 {
        return;
    }
    // Phyllotaxis (sunflower) spread so the start points cover the loot surface
    // evenly instead of emanating from one central point.
    const GOLDEN_ANGLE: f32 = 2.399_963_2;
    for i in 0..ray_count {
        let t = (i as f32 + 0.5) / ray_count as f32;
        let r = t.sqrt() * SPARK_FOOTPRINT_R;
        let a = i as f32 * GOLDEN_ANGLE;
        let (bx, bz) = (r * a.cos(), r * a.sin());
        // Surface height follows the mound: higher near the centre, lower at the
        // rim.
        let base = Vec3::new(bx, SPARK_SURFACE_PEAK_Y - t * SPARK_SURFACE_DROP, bz);
        // Point mostly straight up from each surface point, fanning slightly
        // outward toward the rim.
        let dir = Vec3::new(bx, 1.0, bz).normalize();
        let Some(mat) = mats[i % mats.len()].clone() else {
            continue;
        };
        // Orient the cylinder's +Y axis along the ray; the animation system
        // leaves this rotation alone and only drives length + position.
        let rotation = Quat::from_rotation_arc(Vec3::Y, dir);
        let spark = commands
            .spawn((
                TreasureSparkle { base, dir, phase: t },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(mat),
                Transform {
                    translation: base,
                    rotation,
                    scale: Vec3::new(SPARK_LINE_THICK, SPARK_LINE_LEN, SPARK_LINE_THICK),
                },
            ))
            .id();
        commands.entity(group).add_child(spark);
    }
}

/// `Update`: makes each glow-beam's length rise and fall so the loot shimmers.
/// Rotation is fixed at spawn and left untouched here.
pub(crate) fn treasure_sparkle_system(
    time: Res<Time>,
    mut sparkles: Query<(&TreasureSparkle, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    for (spark, mut transform) in &mut sparkles {
        // Length rises and falls on a per-beam speed (keyed by phase) so the
        // lines fluctuate out of sync; the bright base end stays planted on the
        // surface while the faded tip extends and retracts.
        let speed = SPARK_FLICKER_SPEED * (0.7 + spark.phase);
        let pulse = 0.5 + 0.5 * (t * speed + spark.phase * TAU).sin();
        let len = SPARK_LINE_LEN * (0.2 + 0.8 * pulse);
        transform.scale = Vec3::new(SPARK_LINE_THICK, len, SPARK_LINE_THICK);
        transform.translation = spark.base + spark.dir * (len * 0.5);
    }
}

/// `Update`: plays the collection flourish on any treasure tagged
/// [`CollectingTreasure`] — the loot rises while shrinking to nothing and
/// spinning faster — then despawns it when the animation completes. The open
/// chest is a separate free-standing entity, so it stays behind, emptied.
pub(crate) fn treasure_collection_system(
    mut commands: Commands,
    time: Res<Time>,
    mut collecting: Query<(Entity, &mut CollectingTreasure, &mut Transform, &TreasureMarker)>,
) {
    let dt = time.delta_secs();
    for (entity, mut state, mut transform, marker) in &mut collecting {
        state.elapsed += dt;
        let progress = (state.elapsed / COLLECT_DURATION).min(1.0);
        if progress >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        // Ease-out so the loot leaps up quickly then settles into nothing.
        let eased = 1.0 - (1.0 - progress) * (1.0 - progress);
        transform.translation.y = marker.base_y + COLLECT_RISE * eased;
        transform.scale = Vec3::splat(1.0 - eased);
        transform.rotate_y(dt * COLLECT_SPIN_RATE);
    }
}

#[cfg(test)]
mod tests {
    use super::{rays_per_chest, run_treasure_rays};

    #[test]
    fn rays_per_chest_caps_then_scales_with_count() {
        // Sparse mazes: every chest gets the full per-chest cap.
        assert_eq!(rays_per_chest(1), 5);
        assert_eq!(rays_per_chest(10), 5);
        assert_eq!(rays_per_chest(24), 5); // 120 / 24 == 5, still at the cap
        // Denser mazes: the total budget scales the per-chest count down evenly.
        assert_eq!(rays_per_chest(30), 4); // 120 / 30
        assert_eq!(rays_per_chest(120), 1);
        // Degenerate input must not divide by zero.
        assert_eq!(rays_per_chest(0), 5);
    }

    #[test]
    fn run_treasure_rays_budgets_the_total_across_levels() {
        // A single level of 12 chests is under the cap → the full 5 rays each.
        assert_eq!(run_treasure_rays([12]), 5);
        // The SAME 12 chests on each of three levels (36 total) share the global
        // 120-ray budget → 3 rays each — NOT the 5 the per-level path would give.
        assert_eq!(run_treasure_rays([12, 12, 12]), 3);
        assert!(run_treasure_rays([12, 12, 12]) < rays_per_chest(12));
        // A sparse stack stays at the per-chest cap.
        assert_eq!(run_treasure_rays([3, 2, 1]), 5);
    }
}
