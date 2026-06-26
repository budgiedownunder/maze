//! Per-cell rig resolution.
//!
//! A cell may carry an override choosing a specific visual rig
//! (`enemyType` / `healthStyle` / `keyHolder` / `doorStyle`) instead of the
//! single per-maze rig held in [`GameConfig`](crate::state::GameConfig). These
//! helpers resolve the effective rig for one cell: the per-cell override when
//! present, otherwise the supplied config default.
//!
//! The override carries the engine's (`data_model`) rig enums, which are bridged
//! to this crate's `state::` rig enums through their shared wire string — the
//! same lenient `from_wire_str` mapping used everywhere else, so an unknown
//! value falls back to the rig's default rather than erroring.

use crate::state::{DoorStyle, EnemyType, HealthStyle, KeyHolderStyle, TreasureStyle, WallType};
use maze::CellEntity;

/// The enemy rig for a cell: the cell's `enemyType` override; else, when the
/// difficulty selected `enemy_type = "random"` (`random`), a concrete rig rolled
/// per `(seed, row, col)`; else `default`.
pub(crate) fn resolve_enemy_type(
    entity: Option<&CellEntity>,
    default: EnemyType,
    random: bool,
    seed: u64,
    row: usize,
    col: usize,
) -> EnemyType {
    if let Some(CellEntity::Enemy(over)) = entity {
        if let Some(t) = over.enemy_type {
            return EnemyType::from_wire_str(t.as_wire_str());
        }
    }
    if random {
        return EnemyType::random_for_cell(row, col, seed);
    }
    default
}

/// The health-pickup rig for a cell: the cell's `healthStyle` override; else,
/// when the difficulty selected `health_style = "random"`, a concrete rig rolled
/// per `(seed, row, col)`; else `default`.
pub(crate) fn resolve_health_style(
    entity: Option<&CellEntity>,
    default: HealthStyle,
    random: bool,
    seed: u64,
    row: usize,
    col: usize,
) -> HealthStyle {
    if let Some(CellEntity::Health(over)) = entity {
        if let Some(s) = over.health_style {
            return HealthStyle::from_wire_str(s.as_wire_str());
        }
    }
    if random {
        return HealthStyle::random_for_cell(row, col, seed);
    }
    default
}

/// The key-holder rig for a cell: the cell's `keyHolder` override; else, when the
/// difficulty selected `key_holder = "random"`, a concrete rig rolled per
/// `(seed, row, col)`; else `default`.
pub(crate) fn resolve_key_holder(
    entity: Option<&CellEntity>,
    default: KeyHolderStyle,
    random: bool,
    seed: u64,
    row: usize,
    col: usize,
) -> KeyHolderStyle {
    if let Some(CellEntity::Key(over)) = entity {
        if let Some(h) = over.key_holder {
            return KeyHolderStyle::from_wire_str(h.as_wire_str());
        }
    }
    if random {
        return KeyHolderStyle::random_for_cell(row, col, seed);
    }
    default
}

/// The treasure rig for a cell: the cell's `style` override, else `default`.
pub(crate) fn resolve_treasure_style(
    entity: Option<&CellEntity>,
    default: TreasureStyle,
) -> TreasureStyle {
    if let Some(CellEntity::Treasure(over)) = entity {
        if let Some(s) = over.style {
            return TreasureStyle::from_wire_str(s.as_wire_str());
        }
    }
    default
}

/// The door rig for a cell: the cell's `doorStyle` override, else `default`.
pub(crate) fn resolve_door_style(entity: Option<&CellEntity>, default: DoorStyle) -> DoorStyle {
    if let Some(CellEntity::Door(over)) = entity {
        if let Some(s) = over.door_style {
            return DoorStyle::from_wire_str(s.as_wire_str());
        }
    }
    default
}

/// The wall type for a `'W'` cell: the cell's `wallType` override, else
/// `default` (the per-maze `GameConfig.wall_type`). Drives whether the cell
/// renders a solid panel or non-occluding in-cell geometry — see
/// [`WallType::is_non_occluding`].
pub(crate) fn resolve_wall_type(entity: Option<&CellEntity>, default: WallType) -> WallType {
    if let Some(CellEntity::Wall(over)) = entity {
        if let Some(wt) = over.wall_type {
            return WallType::from_wire_str(wt.as_wire_str());
        }
    }
    default
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build a `CellEntity` from its wire JSON — avoids depending on the
    // override payload structs directly (they are not part of the `maze`
    // re-export surface).
    fn entity(json: &str) -> CellEntity {
        serde_json::from_str(json).expect("valid cell-entity JSON")
    }

    #[test]
    fn enemy_override_picks_the_overridden_rig() {
        let e = entity(r#"{ "type": "E", "enemyType": "ghost" }"#);
        assert_eq!(
            resolve_enemy_type(Some(&e), EnemyType::Goblin, false, 0, 0, 0),
            EnemyType::Ghost,
        );
    }

    #[test]
    fn enemy_without_override_field_falls_back_to_default() {
        // A field-less enemy entity (e.g. a numeric-only override) carries no
        // rig choice, so the config default wins.
        let e = entity(r#"{ "type": "E", "damage": 5 }"#);
        assert_eq!(
            resolve_enemy_type(Some(&e), EnemyType::Ghost, false, 0, 0, 0),
            EnemyType::Ghost,
        );
    }

    #[test]
    fn enemy_with_no_entity_falls_back_to_default() {
        assert_eq!(
            resolve_enemy_type(None, EnemyType::Ghost, false, 0, 0, 0),
            EnemyType::Ghost,
        );
    }

    #[test]
    fn wrong_variant_falls_back_to_default() {
        // A health override on the lookup must not satisfy an enemy resolve.
        let h = entity(r#"{ "type": "H", "healthStyle": "potion" }"#);
        assert_eq!(
            resolve_enemy_type(Some(&h), EnemyType::Goblin, false, 0, 0, 0),
            EnemyType::Goblin,
        );
    }

    #[test]
    fn health_override_picks_the_overridden_rig() {
        let h = entity(r#"{ "type": "H", "healthStyle": "potion" }"#);
        assert_eq!(
            resolve_health_style(Some(&h), HealthStyle::Heart, false, 0, 0, 0),
            HealthStyle::Potion,
        );
        assert_eq!(
            resolve_health_style(None, HealthStyle::Heart, false, 0, 0, 0),
            HealthStyle::Heart,
        );
    }

    #[test]
    fn key_override_picks_the_overridden_rig() {
        let k = entity(r#"{ "type": "K", "keyHolder": "chest" }"#);
        assert_eq!(
            resolve_key_holder(Some(&k), KeyHolderStyle::Pedestal, false, 0, 0, 0),
            KeyHolderStyle::Chest,
        );
        let floating = entity(r#"{ "type": "K", "keyHolder": "floating_key" }"#);
        assert_eq!(
            resolve_key_holder(Some(&floating), KeyHolderStyle::Pedestal, false, 0, 0, 0),
            KeyHolderStyle::FloatingKey,
        );
        assert_eq!(
            resolve_key_holder(None, KeyHolderStyle::Pedestal, false, 0, 0, 0),
            KeyHolderStyle::Pedestal,
        );
    }

    #[test]
    fn random_selectors_roll_a_concrete_rig_only_without_an_override() {
        // With `random` on and no override, the rig is the seeded per-cell roll
        // (not the `default`), and it is stable for a given (seed, cell).
        let want = EnemyType::random_for_cell(2, 3, 99);
        assert_eq!(
            resolve_enemy_type(None, EnemyType::Goblin, true, 99, 2, 3),
            want,
        );
        // An explicit override still wins over `random`.
        let e = entity(r#"{ "type": "E", "enemyType": "ghost" }"#);
        assert_eq!(
            resolve_enemy_type(Some(&e), EnemyType::Goblin, true, 99, 2, 3),
            EnemyType::Ghost,
        );
        // Health + key roll their own seeded rigs too.
        assert_eq!(
            resolve_health_style(None, HealthStyle::Heart, true, 99, 2, 3),
            HealthStyle::random_for_cell(2, 3, 99),
        );
        assert_eq!(
            resolve_key_holder(None, KeyHolderStyle::Pedestal, true, 99, 2, 3),
            KeyHolderStyle::random_for_cell(2, 3, 99),
        );
    }

    #[test]
    fn treasure_override_picks_the_overridden_rig() {
        let t = entity(r#"{ "type": "T", "style": "gold" }"#);
        assert_eq!(
            resolve_treasure_style(Some(&t), TreasureStyle::Silver),
            TreasureStyle::Gold,
        );
        let diamonds = entity(r#"{ "type": "T", "style": "diamonds" }"#);
        assert_eq!(
            resolve_treasure_style(Some(&diamonds), TreasureStyle::Silver),
            TreasureStyle::Diamonds,
        );
        // A treasure entity with only a value override carries no style choice,
        // so the default wins.
        let bare = entity(r#"{ "type": "T", "value": 50 }"#);
        assert_eq!(
            resolve_treasure_style(Some(&bare), TreasureStyle::Silver),
            TreasureStyle::Silver,
        );
        assert_eq!(
            resolve_treasure_style(None, TreasureStyle::Silver),
            TreasureStyle::Silver,
        );
    }

    #[test]
    fn door_override_picks_the_overridden_rig() {
        let d = entity(r#"{ "type": "D", "doorStyle": "portcullis" }"#);
        assert_eq!(
            resolve_door_style(Some(&d), DoorStyle::Swing),
            DoorStyle::Portcullis,
        );
        assert_eq!(resolve_door_style(None, DoorStyle::Swing), DoorStyle::Swing);
    }

    #[test]
    fn wall_override_picks_the_overridden_type() {
        let lava = entity(r#"{ "type": "W", "wallType": "lava" }"#);
        assert_eq!(resolve_wall_type(Some(&lava), WallType::Brick), WallType::Lava);
        let cobble = entity(r#"{ "type": "W", "wallType": "cobblestone" }"#);
        assert_eq!(
            resolve_wall_type(Some(&cobble), WallType::Brick),
            WallType::Cobblestone,
        );
    }

    #[test]
    fn wall_without_override_field_falls_back_to_per_maze_default() {
        // A field-less wall entity carries no type choice, so the per-maze
        // default (the config's `wall_type`) wins.
        let bare = entity(r#"{ "type": "W" }"#);
        assert_eq!(resolve_wall_type(Some(&bare), WallType::Wood), WallType::Wood);
        assert_eq!(resolve_wall_type(None, WallType::Wood), WallType::Wood);
    }
}
