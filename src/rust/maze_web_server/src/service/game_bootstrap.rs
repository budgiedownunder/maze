//! Startup seeding of the shipped curated games.
//!
//! On first launch this creates the `Easy` / `Tricky` / `Hard` **curated game
//! definitions** and the ordered **"Difficulty" curated collection** that
//! references them, owned by the default admin — the stored replacement for the
//! old `config.toml [game.play3d.*]` difficulty presets. It is idempotent (a
//! no-op once the "Difficulty" collection exists), so it runs safely on every
//! launch, mirroring the default-admin bootstrap.
//!
//! The per-difficulty preset values live here as code `const`s (the source of
//! truth once the config presets are retired). Each definition's opaque `config`
//! is produced through the same [`build_play3d_config_response`] path the
//! `play3d-config` endpoint uses, so a curated game reaches the host page in the
//! identical shape a difficulty preset does today.

use chrono::Utc;
use data_model::{GameCollection, GameDefinition, Rotation, User, Visibility};
use storage::{Error as StoreError, Store};
use uuid::Uuid;

use crate::api::v1::endpoints::handlers::build_play3d_config_response;
use crate::config::game::{LevelsConfig, Play3dDifficultyConfig};

/// The shipped, per-difficulty preset values — the numbers that vary between
/// `Easy` / `Tricky` / `Hard`. The scene, landmark and remaining level-meta
/// settings are identical across all three and equal the config defaults, so
/// they are supplied once by [`preset_difficulty_config`] rather than repeated here.
struct DifficultyPreset {
    /// Definition display name (and collection-item identity).
    name: &'static str,
    /// In-game splash title.
    title: &'static str,
    /// Status-bar mode label.
    mode: &'static str,
    rows: u32,
    cols: u32,
    timer_seconds: u32,
    /// Fixed generation seed (first-class on the definition and inside `config`).
    seed: u64,
    min_solution_length: u32,
    door_count: u32,
    spare_doors: u32,
    spare_keys: u32,
    enemy_count: u32,
    health_count: u32,
    treasure_count: u32,
    enemy_move_period_ms: u32,
    max_hp: u32,
    /// Number of stacked levels (1 = single level).
    level_count: u32,
}

const EASY: DifficultyPreset = DifficultyPreset {
    name: "Easy",
    title: "EASY 3D",
    mode: "Easy",
    rows: 10,
    cols: 10,
    timer_seconds: 120,
    seed: 8_080_808,
    min_solution_length: 15,
    door_count: 2,
    spare_doors: 0,
    spare_keys: 0,
    enemy_count: 1,
    health_count: 2,
    treasure_count: 3,
    enemy_move_period_ms: 1800,
    max_hp: 3,
    level_count: 1,
};

const TRICKY: DifficultyPreset = DifficultyPreset {
    name: "Tricky",
    title: "TRICKY 3D",
    mode: "Tricky",
    rows: 25,
    cols: 25,
    timer_seconds: 240,
    seed: 15_151_515,
    min_solution_length: 35,
    door_count: 3,
    spare_doors: 2,
    spare_keys: 1,
    enemy_count: 3,
    health_count: 3,
    treasure_count: 5,
    enemy_move_period_ms: 1500,
    max_hp: 3,
    level_count: 2,
};

const HARD: DifficultyPreset = DifficultyPreset {
    name: "Hard",
    title: "HARD 3D",
    mode: "Hard",
    rows: 40,
    cols: 40,
    timer_seconds: 600,
    seed: 25_252_525,
    min_solution_length: 45,
    door_count: 4,
    spare_doors: 3,
    spare_keys: 1,
    enemy_count: 5,
    health_count: 4,
    treasure_count: 8,
    enemy_move_period_ms: 1200,
    max_hp: 3,
    level_count: 3,
};

/// The shipped presets, in the order they appear in the "Difficulty" collection.
const DIFFICULTY_PRESETS: [DifficultyPreset; 3] = [EASY, TRICKY, HARD];

/// The name of the seeded curated collection.
const DIFFICULTY_COLLECTION_NAME: &str = "Difficulty";

/// Turns a [`DifficultyPreset`] into a full [`Play3dDifficultyConfig`]. The
/// varying values come from the preset; the scene, landmark, minimap and
/// remaining level-meta fields are identical across all three shipped presets
/// and equal the config defaults, so they come from [`Default`] here.
fn preset_difficulty_config(preset: &DifficultyPreset) -> Play3dDifficultyConfig {
    Play3dDifficultyConfig {
        title: Some(preset.title.to_string()),
        mode: preset.mode.to_string(),
        rows: preset.rows,
        cols: preset.cols,
        timer_seconds: preset.timer_seconds,
        seed: preset.seed,
        min_solution_length: preset.min_solution_length,
        door_count: preset.door_count,
        spare_doors: preset.spare_doors,
        spare_keys: preset.spare_keys,
        enemy_count: preset.enemy_count,
        health_count: preset.health_count,
        treasure_count: preset.treasure_count,
        enemy_move_period_ms: preset.enemy_move_period_ms,
        max_hp: preset.max_hp,
        levels: LevelsConfig { count: preset.level_count, ..LevelsConfig::default() },
        ..Play3dDifficultyConfig::default()
    }
}

/// Builds a curated definition's stored `config` — the camelCase wire shape the
/// host page consumes, minus the `difficulty` label (a stored game carries no
/// difficulty tag).
fn build_difficulty_config(preset: &DifficultyPreset) -> Result<serde_json::Value, StoreError> {
    let wire = build_play3d_config_response(
        &preset_difficulty_config(preset),
        preset.name.to_ascii_lowercase(),
        preset.title.to_string(),
    );
    let mut value = serde_json::to_value(&wire)
        .map_err(|err| StoreError::Other(format!("failed to build curated config: {err}")))?;
    if let Some(object) = value.as_object_mut() {
        object.remove("difficulty");
    }
    Ok(value)
}

/// Creates a curated definition for `preset` (or reuses an existing one of the
/// same name), returning its id. Reuse keeps re-seeding idempotent even if the
/// collection was deleted while the definitions were kept.
async fn ensure_difficulty_definition(
    store: &mut Box<dyn Store>,
    admin: &User,
    preset: &DifficultyPreset,
) -> Result<Uuid, StoreError> {
    let now = Utc::now();
    let mut definition = GameDefinition {
        id: Uuid::nil(),
        owner_id: Uuid::nil(),
        name: preset.name.to_string(),
        description: None,
        visibility: Visibility::Curated,
        seed: preset.seed,
        rotation: Rotation::Static,
        config: build_difficulty_config(preset)?,
        image_updated_at: None,
        created_at: now,
        updated_at: now,
    };

    match store.create_game_definition(admin, &mut definition).await {
        Ok(()) => Ok(definition.id),
        Err(StoreError::GameDefinitionNameAlreadyExists(_)) => store
            .get_game_definitions_for_owner(admin)
            .await?
            .into_iter()
            .find(|d| d.name == preset.name)
            .map(|d| d.id)
            .ok_or_else(|| {
                StoreError::Other(format!(
                    "curated definition '{}' vanished after a name collision",
                    preset.name
                ))
            }),
        Err(err) => Err(err),
    }
}

/// Seeds the curated "Difficulty" collection and its `Easy`/`Tricky`/`Hard`
/// definitions under the default admin, if not already present. Idempotent.
pub async fn init_difficulty_collection(
    store: &mut Box<dyn Store>,
    admin_username: &str,
) -> Result<(), StoreError> {
    // Own the curated content with the named default admin, falling back to any
    // active admin (e.g. if the default was renamed); skip if there is none yet.
    let admins: Vec<User> = store.get_admin_users().await?;
    let admin = match admins.iter().find(|u| u.username == admin_username).or_else(|| admins.first()) {
        Some(user) => user.clone(),
        None => return Ok(()),
    };

    // Idempotent: nothing to do once the curated "Difficulty" collection exists.
    let existing = store.get_game_collections_for_owner(&admin).await?;
    if existing
        .iter()
        .any(|c| c.visibility == Visibility::Curated && c.name == DIFFICULTY_COLLECTION_NAME)
    {
        return Ok(());
    }

    let mut definition_ids = Vec::with_capacity(DIFFICULTY_PRESETS.len());
    for preset in &DIFFICULTY_PRESETS {
        definition_ids.push(ensure_difficulty_definition(store, &admin, preset).await?);
    }

    let now = Utc::now();
    let mut collection = GameCollection {
        id: Uuid::nil(),
        owner_id: Uuid::nil(),
        name: DIFFICULTY_COLLECTION_NAME.to_string(),
        visibility: Visibility::Curated,
        description: Some("Warm up on Easy, then climb through Tricky and Hard.".to_string()),
        image_updated_at: None,
        items: Vec::new(),
        created_at: now,
        updated_at: now,
    };
    store.create_game_collection(&admin, &mut collection).await?;
    for definition_id in definition_ids {
        store.add_game_collection_item(&admin, collection.id, definition_id).await?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use storage::{FileStore, FileStoreConfig};

    fn fresh_store() -> (Box<dyn Store>, tempfile::TempDir) {
        let temp = tempfile::tempdir().expect("tempdir");
        let store: Box<dyn Store> = Box::new(FileStore::new(&FileStoreConfig {
            data_dir: temp.path().to_string_lossy().to_string(),
        }));
        (store, temp)
    }

    async fn seed_admin(store: &mut Box<dyn Store>) -> User {
        store
            .init_default_admin_user("admin", "admin@test.local", "hash")
            .await
            .expect("seed default admin")
    }

    #[tokio::test]
    async fn seeds_the_difficulty_definitions_and_ordered_collection() {
        let (mut store, _temp) = fresh_store();
        let admin = seed_admin(&mut store).await;

        init_difficulty_collection(&mut store, "admin").await.expect("bootstrap");

        // The curated content is owned by the admin (the only content in the store).
        let defs = store.get_game_definitions_for_owner(&admin).await.expect("admin defs");
        let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"Easy") && names.contains(&"Tricky") && names.contains(&"Hard"));

        // Easy carries the shipped values as a Static curated definition, and its
        // stored config is the wire shape minus the difficulty label.
        let easy = defs.iter().find(|d| d.name == "Easy").expect("easy def");
        assert_eq!(easy.seed, 8_080_808);
        assert_eq!(easy.visibility, Visibility::Curated);
        assert_eq!(easy.rotation, Rotation::Static);
        assert_eq!(easy.config["rows"], serde_json::json!(10));
        assert_eq!(easy.config["seed"], serde_json::json!(8_080_808u64));
        assert_eq!(easy.config["levels"]["count"], serde_json::json!(1));
        assert!(easy.config.get("difficulty").is_none(), "stored config carries no difficulty tag");

        // The collection references the three, in easy → tricky → hard order.
        let cols = store.get_game_collections_for_owner(&admin).await.expect("admin collections");
        let difficulty = cols.iter().find(|c| c.name == DIFFICULTY_COLLECTION_NAME).expect("difficulty collection");
        let ordered: Vec<Uuid> = difficulty.items.iter().map(|i| i.definition_id).collect();
        let expected: Vec<Uuid> = ["Easy", "Tricky", "Hard"]
            .iter()
            .map(|name| defs.iter().find(|d| &d.name == name).unwrap().id)
            .collect();
        assert_eq!(ordered, expected);
    }

    #[tokio::test]
    async fn is_idempotent_across_relaunches() {
        let (mut store, _temp) = fresh_store();
        let admin = seed_admin(&mut store).await;

        init_difficulty_collection(&mut store, "admin").await.expect("first launch");
        init_difficulty_collection(&mut store, "admin").await.expect("second launch");

        assert_eq!(store.get_game_definitions_for_owner(&admin).await.unwrap().len(), 3);
        assert_eq!(
            store.get_game_collections_for_owner(&admin).await.unwrap().iter()
                .filter(|c| c.name == DIFFICULTY_COLLECTION_NAME).count(),
            1
        );
    }

    #[tokio::test]
    async fn skips_when_no_admin_exists() {
        let (mut store, _temp) = fresh_store();
        // No admin seeded → nothing to own the curated content, so it is a no-op.
        init_difficulty_collection(&mut store, "admin").await.expect("no-op without admin");
        // Adding an admin afterwards, the earlier no-op left nothing to own.
        let admin = seed_admin(&mut store).await;
        assert!(store.get_game_collections_for_owner(&admin).await.unwrap().is_empty());
    }
}
