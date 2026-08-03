//! Baking a composite prop into one combined mesh per material.
//!
//! The decorative props are each built from many scaled unit primitives, and
//! every visible one is paired with a slightly larger inverted-hull outline
//! sibling. Spawned one entity per part, a single prop costs dozens of entities
//! — each extracted, culled and batched every frame it is in view, on every
//! dead-end cell of every level.
//!
//! A prop's parts never move relative to each other, so they do not need to be
//! separate entities. [`RigBuilder`] stamps each part into a combined mesh per
//! material, in the prop's local frame, once at asset-build time; [`BakedRig`]
//! then spawns that as one entity per material at each cell, carrying the cell's
//! position and yaw. The look is unchanged, outlines included.
//!
//! Parts that move independently stay separate entities — the key's radiating
//! sparks, and the floating key group itself, which bobs and spins as a whole
//! (its own sub-meshes are static within it, so they bake).

use bevy::prelude::*;

/// Uniform scale-up factor applied to each sibling outline stamp. The outline
/// shell reuses the body's geometry scaled by this factor, with a material set
/// to `cull_mode: Some(Face::Front)`, so only a thin dark rim pokes out around
/// the body's silhouette — the classic inverted-hull outline trick.
pub(crate) const OUTLINE_SCALE: f32 = 1.06;

/// The shared unit primitives the prop rigs stamp. Held as plain meshes rather
/// than assets: baking merges their vertices into the per-prop combined meshes,
/// so the primitives themselves are never drawn.
pub(crate) struct UnitMeshes {
    pub(crate) cylinder: Mesh,
    pub(crate) cuboid: Mesh,
    pub(crate) cone: Mesh,
}

impl UnitMeshes {
    pub(crate) fn new() -> Self {
        Self {
            cylinder: Mesh::from(Cylinder::new(0.5, 1.0)),
            cuboid: Mesh::from(Cuboid::new(1.0, 1.0, 1.0)),
            cone: Mesh::from(Cone::new(0.5, 1.0)),
        }
    }
}

/// Accumulates a prop's sub-meshes into one combined mesh per material slot.
///
/// A slot is an index into the material list handed to [`RigBuilder::new`]; each
/// prop names its own (`const BODY: usize = 0`, and so on). A slot that receives
/// no parts spawns nothing, so a rig can declare a slot only some of its
/// variants fill.
pub(crate) struct RigBuilder {
    slots: Vec<RigSlot>,
}

struct RigSlot {
    material: Option<Handle<StandardMaterial>>,
    mesh: Option<Mesh>,
}

impl RigBuilder {
    pub(crate) fn new(materials: &[Option<Handle<StandardMaterial>>]) -> Self {
        Self {
            slots: materials
                .iter()
                .map(|material| RigSlot { material: material.clone(), mesh: None })
                .collect(),
        }
    }

    /// Stamps `base` at `xform` — a pose in the prop's local frame — into `slot`.
    pub(crate) fn add(&mut self, slot: usize, base: &Mesh, xform: Transform) {
        stamp(&mut self.slots[slot].mesh, base, xform);
    }

    /// Stamps a body part into `slot` and its inverted-hull outline shell into
    /// `outline` — the pairing every visible prop part ships with.
    pub(crate) fn add_with_outline(
        &mut self,
        slot: usize,
        outline: usize,
        base: &Mesh,
        xform: Transform,
    ) {
        self.add(slot, base, xform);
        self.add(outline, base, outline_xform(xform));
    }

    /// Adds each combined mesh to the asset store, paired with its slot's
    /// material. Without render assets (headless tests) the handles are `None`
    /// but the filled slots survive, so the rig still spawns the same entities.
    pub(crate) fn finish(self, meshes: &mut Option<ResMut<Assets<Mesh>>>) -> BakedRig {
        BakedRig {
            slots: self
                .slots
                .into_iter()
                .map(|slot| {
                    slot.mesh.map(|mesh| BakedSlot {
                        mesh: meshes.as_mut().map(|store| store.add(mesh)),
                        material: slot.material,
                    })
                })
                .collect(),
        }
    }
}

/// A prop's baked geometry — one drawable per material slot, built once and
/// shared by every instance of that prop in the run.
pub(crate) struct BakedRig {
    /// Indexed by slot; `None` where the slot received no parts.
    slots: Vec<Option<BakedSlot>>,
}

struct BakedSlot {
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<StandardMaterial>>,
}

impl BakedRig {
    /// Spawns the rig at `xform` — one entity per filled slot, returned in slot
    /// order so a caller can tag one of them (the brazier's flickering bowl).
    /// `parent`, when given, adopts them so they ride an animated group (the
    /// key's bob and spin).
    pub(crate) fn spawn(
        &self,
        commands: &mut Commands,
        xform: Transform,
        parent: Option<Entity>,
    ) -> Vec<Option<Entity>> {
        self.slots
            .iter()
            .map(|slot| {
                let slot = slot.as_ref()?;
                let mut entity = commands.spawn(xform);
                // Spawned even without render assets, matching the other spawn
                // helpers, so headless entity counts match a rendering build.
                if let (Some(mesh), Some(material)) = (slot.mesh.clone(), slot.material.clone()) {
                    entity.insert((Mesh3d(mesh), MeshMaterial3d(material)));
                }
                let id = entity.id();
                if let Some(parent) = parent {
                    commands.entity(parent).add_child(id);
                }
                Some(id)
            })
            .collect()
    }
}

/// Merges `base`, posed by `xform`, into `acc`.
fn stamp(acc: &mut Option<Mesh>, base: &Mesh, xform: Transform) {
    let piece = base.clone().transformed_by(xform);
    match acc {
        Some(mesh) => {
            let _ = mesh.merge(&piece);
        }
        None => *acc = Some(piece),
    }
}

/// The inverted-hull outline pose for a body part: the same placement scaled up
/// about its own centre by [`OUTLINE_SCALE`].
pub(crate) fn outline_xform(body: Transform) -> Transform {
    Transform { scale: body.scale * OUTLINE_SCALE, ..body }
}

/// Bakes one combined mesh from `base` stamped at every transform in
/// `transforms` (assumed non-empty). Lets a whole loot pile become a single
/// drawable mesh shared by every chest of that style.
pub(crate) fn bake(base: &Mesh, transforms: &[Transform]) -> Mesh {
    let mut acc = None;
    for xform in transforms {
        stamp(&mut acc, base, *xform);
    }
    acc.expect("a baked pile has at least one piece")
}

/// Adds a baked mesh built from `base` at `transforms` to the asset store,
/// returning its handle (or `None` headless / when there are no pieces).
pub(crate) fn baked_handle(
    meshes: &mut Option<ResMut<Assets<Mesh>>>,
    base: &Mesh,
    transforms: &[Transform],
) -> Option<Handle<Mesh>> {
    if transforms.is_empty() {
        return None;
    }
    meshes.as_mut().map(|m| m.add(bake(base, transforms)))
}

/// The inverted-hull outline transforms for a set of body transforms.
pub(crate) fn outline_scaled(transforms: &[Transform]) -> Vec<Transform> {
    transforms.iter().map(|t| outline_xform(*t)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vertex_count(mesh: &Mesh) -> usize {
        mesh.attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("a baked mesh keeps its positions")
            .len()
    }

    #[test]
    fn outline_pose_scales_up_without_moving_the_part() {
        let body = Transform::from_xyz(1.0, 2.0, 3.0)
            .with_rotation(Quat::from_rotation_y(0.5))
            .with_scale(Vec3::new(0.2, 0.4, 0.6));
        let outline = outline_xform(body);
        assert_eq!(outline.translation, body.translation);
        assert_eq!(outline.rotation, body.rotation);
        assert_eq!(outline.scale, body.scale * OUTLINE_SCALE);
    }

    /// An outline slot collects the shell of every part a prop has, so one merge
    /// mixes all of these bases — including the chest's hand-built lid and the
    /// key's torus bow. A mismatched attribute set would return `Err` and
    /// silently drop the piece rather than fail loudly.
    #[test]
    fn stamping_mixed_bases_keeps_every_vertex() {
        let prims = UnitMeshes::new();
        let bases = [
            prims.cuboid,
            prims.cylinder,
            prims.cone,
            super::super::chest::half_cylinder_mesh(),
            Mesh::from(Torus::new(0.085, 0.18)),
        ];
        let mut acc = None;
        for (i, base) in bases.iter().enumerate() {
            stamp(&mut acc, base, Transform::from_xyz(i as f32, 0.0, 0.0));
        }
        let merged = acc.expect("five stamps produce a mesh");
        assert_eq!(
            vertex_count(&merged),
            bases.iter().map(vertex_count).sum::<usize>(),
        );
    }

    #[test]
    fn baking_repeats_the_base_once_per_transform() {
        let base = UnitMeshes::new().cuboid;
        let xforms = [
            Transform::IDENTITY,
            Transform::from_xyz(1.0, 0.0, 0.0),
            Transform::from_xyz(2.0, 0.0, 0.0),
        ];
        assert_eq!(vertex_count(&bake(&base, &xforms)), 3 * vertex_count(&base));
    }

    /// Headless, the meshes and materials are absent — but the filled slots must
    /// still be there, or a rendering build and a test build would spawn
    /// different numbers of entities and every entity count would be a fiction.
    #[test]
    fn empty_slots_are_dropped_and_filled_ones_survive_without_assets() {
        let prims = UnitMeshes::new();
        let mut rig = RigBuilder::new(&[None, None, None]);
        rig.add_with_outline(0, 2, &prims.cuboid, Transform::IDENTITY);
        let baked = rig.finish(&mut None);
        assert!(baked.slots[0].is_some(), "the body slot was filled");
        assert!(baked.slots[1].is_none(), "slot 1 received no parts");
        assert!(baked.slots[2].is_some(), "the outline slot was filled");
    }
}
