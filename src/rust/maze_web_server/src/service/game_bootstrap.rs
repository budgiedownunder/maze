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
//! truth for the shipped curated games). Each definition's opaque `config` is
//! the camelCase `StartConfig` the host page forwards to Bevy verbatim, built by
//! [`curated_config`] — the varying fields come from the preset, the rest are the
//! shipped defaults.

use chrono::Utc;
use data_model::{GameCollection, GameCollectionMeta, GameDefinition, PlayMode, Rotation, User, Visibility};
use storage::{Error as StoreError, Store};
use uuid::Uuid;

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

/// The shipped daily-challenge definition. A single `Daily`-rotation game: the
/// stored `seed` is the base the server folds today's UTC date into, so the
/// layout + its board rotate each day while the size/counts stay fixed. A
/// mid-weight preset (between Easy and Tricky), so the daily is approachable but
/// not trivial.
const DAILY: DifficultyPreset = DifficultyPreset {
    name: "Daily Maze",
    title: "DAILY 3D",
    mode: "Daily",
    rows: 20,
    cols: 20,
    timer_seconds: 180,
    seed: 20_260_101,
    min_solution_length: 25,
    door_count: 3,
    spare_doors: 1,
    spare_keys: 1,
    enemy_count: 2,
    health_count: 3,
    treasure_count: 4,
    enemy_move_period_ms: 1500,
    max_hp: 3,
    level_count: 1,
};

/// The name of the seeded curated daily-challenge collection.
const DAILY_COLLECTION_NAME: &str = "Daily Challenges";

/// Builds a curated definition's stored `config` — the camelCase `StartConfig`
/// the host page forwards to Bevy verbatim. The varying values come from the
/// preset; the scene, landmark, minimap and level-meta fields are identical
/// across every shipped preset and are the game's defaults, so they are literals
/// here. (A single-level game — `levels.count == 1` — leaves the rest of the
/// `levels` group inert.)
fn curated_config(preset: &DifficultyPreset) -> serde_json::Value {
    serde_json::json!({
        "rows": preset.rows,
        "cols": preset.cols,
        "timerSeconds": preset.timer_seconds,
        "seed": preset.seed,
        "minSolutionLength": preset.min_solution_length,
        "minimapCellPx": 10,
        "minimapRadius": 5,
        "title": preset.title,
        "mode": preset.mode,
        "landmarks": {
            "wallTint": true,
            "deadEndObjects": true,
            "wallDecorations": true,
            "floorAccents": true,
            "wallMaterialVariation": true
        },
        "skyType": "night",
        "wallType": "brick",
        "perimeterWalls": true,
        "doorStyle": "swing",
        "keyHolder": "pedestal",
        "doorCount": preset.door_count,
        "spareDoors": preset.spare_doors,
        "spareKeys": preset.spare_keys,
        "enemyCount": preset.enemy_count,
        "healthCount": preset.health_count,
        "treasureCount": preset.treasure_count,
        "enemyType": "goblin",
        "healthStyle": "heart",
        "enemyMovePeriodMs": preset.enemy_move_period_ms,
        "maxHp": preset.max_hp,
        "levels": {
            "count": preset.level_count,
            "finishType": "ladder",
            "difficultyChange": "easier",
            "resetBag": true,
            "alignment": "edge",
            "taper": false,
            "perimeterRandom": false,
            "hideCompletedEnemies": false,
            "top": null
        }
    })
}

/// Creates a curated definition for `preset` with the given `rotation` (or reuses
/// an existing one of the same name), returning its id. `Static` builds a fixed
/// board; `Daily` makes the server date-mix the seed + challenge key per UTC day.
/// Reuse keeps re-seeding idempotent even if the collection was deleted while the
/// definitions were kept.
async fn ensure_curated_definition(
    store: &mut Box<dyn Store>,
    admin: &User,
    preset: &DifficultyPreset,
    rotation: Rotation,
) -> Result<Uuid, StoreError> {
    let now = Utc::now();
    let mut definition = GameDefinition {
        id: Uuid::nil(),
        owner_id: Uuid::nil(),
        name: preset.name.to_string(),
        description: None,
        visibility: Visibility::Curated,
        seed: preset.seed,
        rotation,
        config: curated_config(preset),
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
        .any(|c| c.meta.visibility == Visibility::Curated && c.meta.name == DIFFICULTY_COLLECTION_NAME)
    {
        return Ok(());
    }

    let mut definition_ids = Vec::with_capacity(DIFFICULTY_PRESETS.len());
    for preset in &DIFFICULTY_PRESETS {
        definition_ids.push(ensure_curated_definition(store, &admin, preset, Rotation::Static).await?);
    }

    let now = Utc::now();
    let mut collection = GameCollection {
        meta: GameCollectionMeta {
            id: Uuid::nil(),
            owner_id: Uuid::nil(),
            name: DIFFICULTY_COLLECTION_NAME.to_string(),
            visibility: Visibility::Curated,
            play_mode: PlayMode::Arcade,
            description: Some("Warm up on Easy, then climb through Tricky and Hard.".to_string()),
            image_updated_at: None,
            created_at: now,
            updated_at: now,
        },
        items: Vec::new(),
    };
    store.create_game_collection(&admin, &mut collection).await?;
    store.set_game_collection_items(&admin, collection.meta.id, &definition_ids).await?;

    Ok(())
}

/// Seeds the curated "Daily Challenges" collection and its `Daily` definition
/// under the default admin, if not already present. Idempotent — a no-op once the
/// curated "Daily Challenges" collection exists — so it runs safely on every
/// launch, mirroring [`init_difficulty_collection`]. The `Daily` definition's
/// board rotates by UTC date, which the server derives at play-fetch; nothing
/// here schedules or rolls anything over.
pub async fn init_daily_challenges_collection(
    store: &mut Box<dyn Store>,
    admin_username: &str,
) -> Result<(), StoreError> {
    let admins: Vec<User> = store.get_admin_users().await?;
    let admin = match admins.iter().find(|u| u.username == admin_username).or_else(|| admins.first()) {
        Some(user) => user.clone(),
        None => return Ok(()),
    };

    // Idempotent: nothing to do once the curated "Daily Challenges" collection
    // exists.
    let existing = store.get_game_collections_for_owner(&admin).await?;
    if existing
        .iter()
        .any(|c| c.meta.visibility == Visibility::Curated && c.meta.name == DAILY_COLLECTION_NAME)
    {
        return Ok(());
    }

    let daily_id = ensure_curated_definition(store, &admin, &DAILY, Rotation::Daily).await?;

    let now = Utc::now();
    let mut collection = GameCollection {
        meta: GameCollectionMeta {
            id: Uuid::nil(),
            owner_id: Uuid::nil(),
            name: DAILY_COLLECTION_NAME.to_string(),
            visibility: Visibility::Curated,
            play_mode: PlayMode::Arcade,
            description: Some("A fresh maze every day — how fast can you clear today's?".to_string()),
            image_updated_at: None,
            created_at: now,
            updated_at: now,
        },
        items: Vec::new(),
    };
    store.create_game_collection(&admin, &mut collection).await?;
    store.set_game_collection_items(&admin, collection.meta.id, &[daily_id]).await?;

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

    #[test]
    fn curated_config_easy_matches_golden() {
        // Golden: the exact stored config the Easy preset produced before the
        // config machinery was collapsed into curated_config. Any drift in the
        // shipped defaults must be a deliberate edit to both sides.
        let expected = serde_json::json!({
            "rows": 10, "cols": 10, "timerSeconds": 120, "seed": 8_080_808u64,
            "minSolutionLength": 15, "minimapCellPx": 10, "minimapRadius": 5,
            "title": "EASY 3D", "mode": "Easy",
            "landmarks": { "wallTint": true, "deadEndObjects": true, "wallDecorations": true, "floorAccents": true, "wallMaterialVariation": true },
            "skyType": "night", "wallType": "brick", "perimeterWalls": true,
            "doorStyle": "swing", "keyHolder": "pedestal",
            "doorCount": 2, "spareDoors": 0, "spareKeys": 0,
            "enemyCount": 1, "healthCount": 2, "treasureCount": 3,
            "enemyType": "goblin", "healthStyle": "heart",
            "enemyMovePeriodMs": 1800, "maxHp": 3,
            "levels": { "count": 1, "finishType": "ladder", "difficultyChange": "easier", "resetBag": true, "alignment": "edge", "taper": false, "perimeterRandom": false, "hideCompletedEnemies": false, "top": null }
        });
        assert_eq!(curated_config(&EASY), expected);
    }

    #[test]
    fn curated_config_carries_preset_values_and_shared_defaults() {
        for preset in [&TRICKY, &HARD, &DAILY] {
            let cfg = curated_config(preset);
            // Varying, per preset.
            assert_eq!(cfg["rows"], preset.rows);
            assert_eq!(cfg["cols"], preset.cols);
            assert_eq!(cfg["seed"], preset.seed);
            assert_eq!(cfg["title"], preset.title);
            assert_eq!(cfg["mode"], preset.mode);
            assert_eq!(cfg["levels"]["count"], preset.level_count);
            // Shared shipped defaults.
            assert_eq!(cfg["skyType"], "night");
            assert_eq!(cfg["wallType"], "brick");
            assert_eq!(cfg["doorStyle"], "swing");
            assert_eq!(cfg["keyHolder"], "pedestal");
            assert_eq!(cfg["enemyType"], "goblin");
            assert_eq!(cfg["levels"]["finishType"], "ladder");
            assert_eq!(cfg["levels"]["top"], serde_json::Value::Null);
            // No difficulty tag on a stored game.
            assert!(cfg.get("difficulty").is_none());
        }
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
        let difficulty = cols.iter().find(|c| c.meta.name == DIFFICULTY_COLLECTION_NAME).expect("difficulty collection");
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
                .filter(|c| c.meta.name == DIFFICULTY_COLLECTION_NAME).count(),
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

    #[tokio::test]
    async fn seeds_the_daily_definition_and_collection() {
        let (mut store, _temp) = fresh_store();
        let admin = seed_admin(&mut store).await;

        init_daily_challenges_collection(&mut store, "admin").await.expect("bootstrap");

        // The daily definition is a curated, Daily-rotation game.
        let defs = store.get_game_definitions_for_owner(&admin).await.expect("admin defs");
        let daily = defs.iter().find(|d| d.name == "Daily Maze").expect("daily def");
        assert_eq!(daily.visibility, Visibility::Curated);
        assert_eq!(daily.rotation, Rotation::Daily);
        assert_eq!(daily.seed, 20_260_101);
        assert!(daily.config.get("difficulty").is_none(), "stored config carries no difficulty tag");

        // The curated "Daily Challenges" collection references it.
        let cols = store.get_game_collections_for_owner(&admin).await.expect("admin collections");
        let daily_collection = cols
            .iter()
            .find(|c| c.meta.name == DAILY_COLLECTION_NAME)
            .expect("daily collection");
        assert_eq!(daily_collection.meta.visibility, Visibility::Curated);
        let ordered: Vec<Uuid> = daily_collection.items.iter().map(|i| i.definition_id).collect();
        assert_eq!(ordered, vec![daily.id]);
    }

    #[tokio::test]
    async fn daily_seeding_is_idempotent_and_independent_of_difficulty() {
        let (mut store, _temp) = fresh_store();
        let admin = seed_admin(&mut store).await;

        // Both bootstraps run every launch; the daily one seeds exactly one def +
        // one collection however many times it runs, and doesn't disturb Difficulty.
        init_difficulty_collection(&mut store, "admin").await.expect("difficulty");
        init_daily_challenges_collection(&mut store, "admin").await.expect("daily first");
        init_daily_challenges_collection(&mut store, "admin").await.expect("daily second");

        let defs = store.get_game_definitions_for_owner(&admin).await.unwrap();
        assert_eq!(defs.iter().filter(|d| d.name == "Daily Maze").count(), 1);
        let cols = store.get_game_collections_for_owner(&admin).await.unwrap();
        assert_eq!(cols.iter().filter(|c| c.meta.name == DAILY_COLLECTION_NAME).count(), 1);
        // The Difficulty content is untouched — three defs + its collection.
        assert!(cols.iter().any(|c| c.meta.name == DIFFICULTY_COLLECTION_NAME));
        assert_eq!(defs.iter().filter(|d| ["Easy", "Tricky", "Hard"].contains(&d.name.as_str())).count(), 3);
    }
}
