use std::collections::HashMap;
use std::env;
use std::fs;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf, MAIN_SEPARATOR_STR};
use async_trait::async_trait;
use unicase::UniCase;
use uuid::Uuid;

use data_model::{
    AuditOutcome, CollectionItem, EmailAuditEntry, GameCollection, GameDefinition, GranteeSummary,
    Maze, OneTimeToken, User, UserEmail, Visibility, truncate_email_audit_error_message,
};
use utils::file::{delete_dir, delete_file, dir_exists, file_exists};

use crate::store::{
    EmailAuditLog, GameStore, Manage, MazeStore, ScoreEntry, ScoreMetric, ScoreOrdering,
    ScoreStore, ScoreboardEntry, SortDirection, TokenStore, UserStore, normalize_item_order,
    reordered_items,
};
use crate::{
    file_store_migration,
    validation::{validate_email_format, validate_game_definition_config_size, validate_maze_cell_count, validate_maze_feature_count, validate_maze_object_counts, validate_user_fields},
    Error, MazeItem, Store, MAX_GAME_DEFINITION_CONFIG_BYTES,
};

/// Cell-count ceiling enforced by [`FileStore`] on `create_maze` and
/// `update_maze`. The filesystem imposes no practical row-size limit, so
/// this cap is a *runtime-cost* bound — large mazes are expensive to
/// generate, solve, render, and serialise — rather than a storage one.
/// 100×100 fits exactly at the cap.
pub const MAX_MAZE_CELLS: usize = 10_000;

/// File store configuration settings
#[derive(Debug, Clone)]
pub struct FileStoreConfig {
    /// The directory under which data is stored (default = "data", under the working directory)
    pub data_dir: String,
}

impl FileStoreConfig {
    /// Builds a config that points at a `data/` directory under the
    /// process's current working directory. Convenient for ad-hoc CLI
    /// usage; production deployments should construct the config
    /// explicitly with an absolute path.
    ///
    /// # Examples
    ///
    /// ```
    /// use storage::FileStoreConfig;
    ///
    /// let cfg = FileStoreConfig::default();
    /// assert_eq!(cfg.data_dir, "data");
    /// ```
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        FileStoreConfig {
            data_dir: "data".to_string(),
        }
    }
}

/// A file store that implements the [`Store`] trait
///
/// Maze objects are stored on disk as files named `<name>.json` (in the working directory), with the `id`
/// of the object assumed to be the file name
pub struct FileStore {
    /// Configuration settings
    config: FileStoreConfig,
    /// Full path to the root data directory
    data_dir: String,
    /// Full path to the root users directory
    users_dir: String,
    /// Full path to the one-time-tokens directory (one file per token).
    tokens_dir: String,
    /// Full path to the email audit log directory (one file per entry).
    audit_log_dir: String,
    /// Full path to the score history directory (one file per completed run).
    score_history_dir: String,
    /// Full path to the game definitions directory. Each definition owns a
    /// `<id>/` sub-folder holding `definition.json`, its optional `shares.json`
    /// (grantee-uuid list), and — later — its `image.png`.
    game_definitions_dir: String,
    /// Full path to the game collections directory. Each collection owns an
    /// `<id>/` sub-folder holding `collection.json` (with its ordered items),
    /// its optional `shares.json`, and — later — its `image.png`.
    game_collections_dir: String,
}

// Private trait used for accessing struct fields
trait FieldAccess {
    fn get_string_field(&self, field_name: &str) -> Option<String>;
}

// Private FieldAccess implementation for User
impl FieldAccess for User {
    fn get_string_field(&self, field_name: &str) -> Option<String> {
        match field_name {
            "username" => Some(self.username.clone()),
            "full_name" => Some(self.full_name.clone()),
            "email" => Some(self.email().to_string()),
            "password_hash" => Some(self.password_hash.clone()),
            _ => None,
        }
    }
}

impl FileStore {
    /// Creates a new file store instance
    ///
    /// # Returns
    ///
    /// A new file store instance if successful
    ///
    /// # Examples
    ///
    /// Try to create a new maze within a file store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_create = Maze::from_vec(grid);
    /// maze_to_create.name = "maze_1".to_string();
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Create a maze within the file store
    /// match store.create_maze(&owner, &mut maze_to_create).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created maze in the file store with id = {}",
    ///             maze_to_create.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create maze => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    pub fn new(config: &FileStoreConfig) -> Self {
        let mut store = FileStore {
            config: config.clone(),
            data_dir: "".to_string(),
            users_dir: "".to_string(),
            tokens_dir: "".to_string(),
            audit_log_dir: "".to_string(),
            score_history_dir: "".to_string(),
            game_definitions_dir: "".to_string(),
            game_collections_dir: "".to_string(),
        };

        match store.init() {
            Ok(_) => store,
            Err(error) => panic!("Failed to initialize file store: {error}"),
        }
    }

    // Initializes the file store
    fn init(&mut self) -> Result<(), Error> {
        self.data_dir = Self::make_data_dir(&self.config.data_dir)?;
        self.users_dir = self.make_users_dir()?;
        // Migrate any pre-multi-email `user.json` files in place (idempotent —
        // already-migrated files parse straight as the new shape and are
        // left alone).
        file_store_migration::migrate_users_dir(&self.users_dir)?;
        // Run the schema-versioned migration framework. On a fresh data_dir
        // this writes `.schema_version` to the current value via no-op
        // migrations; on an existing data_dir already at the current
        // version the call is cheap and rewrites nothing. Migration 0005
        // creates `<data_dir>/one_time_tokens/` so the path below always
        // resolves to an existing directory afterwards.
        file_store_migration::apply_pending_migrations(&self.data_dir)?;
        self.tokens_dir = Path::new(&self.data_dir)
            .join("one_time_tokens")
            .to_string_lossy()
            .to_string();
        self.audit_log_dir = Path::new(&self.data_dir)
            .join("email_audit_log")
            .to_string_lossy()
            .to_string();
        self.score_history_dir = Path::new(&self.data_dir)
            .join("score_history")
            .to_string_lossy()
            .to_string();
        self.game_definitions_dir = Path::new(&self.data_dir)
            .join("game_definitions")
            .to_string_lossy()
            .to_string();
        self.game_collections_dir = Path::new(&self.data_dir)
            .join("game_collections")
            .to_string_lossy()
            .to_string();
        Ok(())
    }

    // Returns the file path for a given token id
    fn token_file_path(&self, id: Uuid) -> String {
        Path::new(&self.tokens_dir)
            .join(format!("{id}.json"))
            .to_string_lossy()
            .to_string()
    }

    // Reads a token's JSON file from disk. Returns `TokenIdNotFound` if the
    // file does not exist (treats that as the canonical missing-token signal).
    fn read_token_raw(&self, id: Uuid) -> Result<OneTimeToken, Error> {
        let path = self.token_file_path(id);
        if !file_exists(&path) {
            return Err(Error::TokenIdNotFound(id.to_string()));
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader::<BufReader<File>, OneTimeToken>(reader).map_err(Error::from)
    }

    // Atomically writes a token JSON via tempfile + rename.
    fn write_token_file(&self, token: &OneTimeToken, overwrite: bool) -> Result<(), Error> {
        let target = self.token_file_path(token.id);
        if !overwrite && file_exists(&target) {
            return Err(Error::TokenIdExists(token.id.to_string()));
        }
        let json = token.to_json()?;
        let tmp = format!("{target}.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
        }
        fs::rename(&tmp, &target)?;
        Ok(())
    }

    // Enumerates token ids in the one_time_tokens directory.
    fn get_token_ids(&self) -> Result<Vec<Uuid>, Error> {
        if !dir_exists(&self.tokens_dir) {
            return Ok(Vec::new());
        }
        let ids: Vec<Uuid> = fs::read_dir(&self.tokens_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if !path.is_file() {
                        return None;
                    }
                    let name = path.file_stem()?.to_str()?;
                    Uuid::parse_str(name).ok()
                })
            })
            .collect();
        Ok(ids)
    }

    // Returns the file path for a given audit entry id.
    fn audit_entry_file_path(&self, id: Uuid) -> String {
        Path::new(&self.audit_log_dir)
            .join(format!("{id}.json"))
            .to_string_lossy()
            .to_string()
    }

    // Reads an audit row's JSON file from disk.
    fn read_audit_entry_raw(&self, id: Uuid) -> Result<EmailAuditEntry, Error> {
        let path = self.audit_entry_file_path(id);
        if !file_exists(&path) {
            return Err(Error::AuditEntryIdNotFound(id.to_string()));
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader::<BufReader<File>, EmailAuditEntry>(reader).map_err(Error::from)
    }

    // Atomically writes an audit row JSON via tempfile + rename.
    fn write_audit_entry_file(
        &self,
        entry: &EmailAuditEntry,
        overwrite: bool,
    ) -> Result<(), Error> {
        let target = self.audit_entry_file_path(entry.id);
        if !overwrite && file_exists(&target) {
            return Err(Error::AuditEntryIdExists(entry.id.to_string()));
        }
        let json = entry.to_json()?;
        let tmp = format!("{target}.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
        }
        fs::rename(&tmp, &target)?;
        Ok(())
    }

    // Enumerates audit entry ids in the email_audit_log directory.
    fn get_audit_entry_ids(&self) -> Result<Vec<Uuid>, Error> {
        if !dir_exists(&self.audit_log_dir) {
            return Ok(Vec::new());
        }
        let ids: Vec<Uuid> = fs::read_dir(&self.audit_log_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if !path.is_file() {
                        return None;
                    }
                    let name = path.file_stem()?.to_str()?;
                    Uuid::parse_str(name).ok()
                })
            })
            .collect();
        Ok(ids)
    }

    // ── score history JSON helpers (one file per completed run) ──────────────

    // Returns the file path for a given score entry id.
    fn score_entry_file_path(&self, id: Uuid) -> String {
        Path::new(&self.score_history_dir)
            .join(format!("{id}.json"))
            .to_string_lossy()
            .to_string()
    }

    // Reads a score row's JSON file from disk.
    fn read_score_entry_raw(&self, id: Uuid) -> Result<ScoreEntry, Error> {
        let path = self.score_entry_file_path(id);
        if !file_exists(&path) {
            return Err(Error::Other(format!("score entry {id} not found")));
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader::<BufReader<File>, ScoreEntry>(reader).map_err(Error::from)
    }

    // Atomically writes a score row JSON via tempfile + rename. Rejects a
    // duplicate id.
    fn write_score_entry_file(&self, entry: &ScoreEntry) -> Result<(), Error> {
        if !dir_exists(&self.score_history_dir) {
            fs::create_dir_all(&self.score_history_dir)?;
        }
        let target = self.score_entry_file_path(entry.id);
        if file_exists(&target) {
            return Err(Error::Other(format!(
                "score entry {} already exists",
                entry.id
            )));
        }
        let json = serde_json::to_string(entry)?;
        let tmp = format!("{target}.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
        }
        fs::rename(&tmp, &target)?;
        Ok(())
    }

    // Enumerates score entry ids in the score_history directory.
    fn get_score_entry_ids(&self) -> Result<Vec<Uuid>, Error> {
        if !dir_exists(&self.score_history_dir) {
            return Ok(Vec::new());
        }
        let ids: Vec<Uuid> = fs::read_dir(&self.score_history_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if !path.is_file() {
                        return None;
                    }
                    let name = path.file_stem()?.to_str()?;
                    Uuid::parse_str(name).ok()
                })
            })
            .collect();
        Ok(ids)
    }

    // Loads every score row, skipping any unreadable file.
    fn read_all_score_entries(&self) -> Result<Vec<ScoreEntry>, Error> {
        let mut entries = Vec::new();
        for id in self.get_score_entry_ids()? {
            match self.read_score_entry_raw(id) {
                Ok(entry) => entries.push(entry),
                Err(error) => {
                    log::warn!("FileStore score read: skipping unreadable entry '{id}' - {error}");
                }
            }
        }
        Ok(entries)
    }

    // Deletes every score-history file whose entry matches `pred`, returning the
    // number removed. Shared by the per-subject leaderboard clears (and reused by
    // the user-deletion cascade where a subject test isn't enough).
    fn delete_scores_matching(&self, pred: impl Fn(&ScoreEntry) -> bool) -> Result<u64, Error> {
        let mut removed = 0u64;
        for entry in self.read_all_score_entries()? {
            if pred(&entry) {
                let path = self.score_entry_file_path(entry.id);
                if file_exists(&path) {
                    fs::remove_file(&path)?;
                    removed += 1;
                }
            }
        }
        Ok(removed)
    }

    // The maze ids owned by `user_id`. A FileStore maze id is its full file
    // name (`"<name>.json"`, per `make_maze_id`), so this uses `file_name`, not
    // `file_stem`. Used to cascade-delete those mazes' score boards when the
    // user (and thus their mazes) is deleted.
    fn user_maze_ids(&self, user_id: Uuid) -> Vec<String> {
        let mazes_dir = Path::new(&self.user_dir_path(user_id)).join("mazes");
        let Ok(read) = fs::read_dir(&mazes_dir) else {
            return Vec::new();
        };
        read.filter_map(|e| {
            let path = e.ok()?.path();
            if !path.is_file() {
                return None;
            }
            Some(path.file_name()?.to_str()?.to_string())
        })
        .collect()
    }

    // Removes every score row whose player is `user_id` (when set) or whose maze
    // is in `maze_ids`. The FileStore counterpart to the SqlStore app-level
    // cascade (which the SQL FK only backstops).
    fn delete_score_rows(&self, user_id: Option<Uuid>, maze_ids: &[String]) -> Result<(), Error> {
        for id in self.get_score_entry_ids()? {
            let Ok(entry) = self.read_score_entry_raw(id) else {
                continue;
            };
            let by_user = user_id.is_some_and(|u| entry.user_id == u);
            let by_maze = entry
                .maze_id
                .as_deref()
                .is_some_and(|m| maze_ids.iter().any(|x| x == m));
            if by_user || by_maze {
                delete_file(&self.score_entry_file_path(id));
            }
        }
        Ok(())
    }

    // ── game definition helpers (one `<id>/` folder per definition) ──────────

    // The folder holding one definition's files (definition.json / shares.json /
    // — later — image.png).
    fn game_definition_dir_path(&self, id: Uuid) -> String {
        Path::new(&self.game_definitions_dir)
            .join(id.to_string())
            .to_string_lossy()
            .to_string()
    }

    // Path to a definition's `definition.json`.
    fn game_definition_file_path(&self, id: Uuid) -> String {
        Path::new(&self.game_definition_dir_path(id))
            .join("definition.json")
            .to_string_lossy()
            .to_string()
    }

    // Path to a definition's `image.png` (inside its `<id>/` folder).
    fn game_definition_image_file_path(&self, id: Uuid) -> String {
        Path::new(&self.game_definition_dir_path(id))
            .join("image.png")
            .to_string_lossy()
            .to_string()
    }

    // Reads a definition's JSON file. Returns `GameDefinitionIdNotFound` if the
    // file does not exist (the canonical missing signal).
    fn read_game_definition_raw(&self, id: Uuid) -> Result<GameDefinition, Error> {
        let path = self.game_definition_file_path(id);
        if !file_exists(&path) {
            return Err(Error::GameDefinitionIdNotFound(id.to_string()));
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader::<BufReader<File>, GameDefinition>(reader).map_err(Error::from)
    }

    // Atomically writes a definition's `definition.json` via tempfile + rename,
    // creating its `<id>/` folder. With `overwrite = false` a pre-existing id is
    // a programmer error (ids are fresh UUIDs), reported as `Error::Other`.
    fn write_game_definition_file(
        &self,
        definition: &GameDefinition,
        overwrite: bool,
    ) -> Result<(), Error> {
        let dir = self.game_definition_dir_path(definition.id);
        if !dir_exists(&dir) {
            fs::create_dir_all(&dir)?;
        }
        let target = self.game_definition_file_path(definition.id);
        if !overwrite && file_exists(&target) {
            return Err(Error::Other(format!(
                "game definition {} already exists",
                definition.id
            )));
        }
        let json = serde_json::to_string(definition)?;
        let tmp = format!("{target}.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
        }
        fs::rename(&tmp, &target)?;
        Ok(())
    }

    // Enumerates definition ids (the `<id>/` sub-folders of game_definitions).
    fn get_game_definition_ids(&self) -> Result<Vec<Uuid>, Error> {
        if !dir_exists(&self.game_definitions_dir) {
            return Ok(Vec::new());
        }
        let ids: Vec<Uuid> = fs::read_dir(&self.game_definitions_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if !path.is_dir() {
                        return None;
                    }
                    let name = path.file_name()?.to_str()?;
                    Uuid::parse_str(name).ok()
                })
            })
            .collect();
        Ok(ids)
    }

    // Loads every definition, skipping any unreadable file.
    fn read_all_game_definitions(&self) -> Result<Vec<GameDefinition>, Error> {
        let mut defs = Vec::new();
        for id in self.get_game_definition_ids()? {
            match self.read_game_definition_raw(id) {
                Ok(def) => defs.push(def),
                Err(error) => {
                    log::warn!("FileStore game definition read: skipping unreadable '{id}' - {error}");
                }
            }
        }
        Ok(defs)
    }

    // Sorts definitions case-insensitively by name (the list-read ordering).
    fn sort_definitions_by_name(defs: &mut [GameDefinition]) {
        defs.sort_by(|a, b| UniCase::new(a.name.as_str()).cmp(&UniCase::new(b.name.as_str())));
    }

    // Returns the id of `owner`'s definition named `name` (case-insensitive), or
    // `None`. Used to enforce the per-owner unique-name rule on create/update.
    fn find_owner_definition_id_by_name(
        &self,
        owner_id: Uuid,
        name: &str,
    ) -> Result<Option<Uuid>, Error> {
        let target = UniCase::new(name);
        for def in self.read_all_game_definitions()? {
            if def.owner_id == owner_id && UniCase::new(def.name.as_str()) == target {
                return Ok(Some(def.id));
            }
        }
        Ok(None)
    }

    // Path to a definition's `shares.json` (inside its `<id>/` folder).
    fn game_definition_shares_file_path(&self, def_id: Uuid) -> String {
        Path::new(&self.game_definition_dir_path(def_id))
            .join("shares.json")
            .to_string_lossy()
            .to_string()
    }

    // Reads a definition's grantee-uuid list (empty when no share file exists).
    fn read_game_definition_grantees(&self, def_id: Uuid) -> Result<Vec<Uuid>, Error> {
        let path = self.game_definition_shares_file_path(def_id);
        if !file_exists(&path) {
            return Ok(Vec::new());
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader::<BufReader<File>, Vec<Uuid>>(reader).map_err(Error::from)
    }

    // Resolves grantee ids to `{id, username}` summaries for the owner's
    // manage-shares view, loading each user by id. A grantee whose user record is
    // soft-deleted or missing is dropped — `read_user` returns `UserIdNotFound`
    // for both — so the FileStore matches the SqlStore JOIN (which excludes
    // `deleted_at IS NULL` and non-matching ids). Ordered by username to match
    // the SQL `ORDER BY u.username`.
    fn resolve_grantee_summaries(&self, ids: Vec<Uuid>) -> Result<Vec<GranteeSummary>, Error> {
        let mut summaries = Vec::with_capacity(ids.len());
        for id in ids {
            match self.read_user(id) {
                Ok(user) => summaries.push(GranteeSummary { id, username: user.username, avatar_updated_at: user.avatar_updated_at }),
                Err(Error::UserIdNotFound(_)) => {}
                Err(err) => return Err(err),
            }
        }
        summaries.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(summaries)
    }

    // Writes a definition's `shares.json`; an empty list removes the file so no
    // empty share record lingers.
    fn write_game_definition_grantees(
        &self,
        def_id: Uuid,
        grantees: &[Uuid],
    ) -> Result<(), Error> {
        let target = self.game_definition_shares_file_path(def_id);
        if grantees.is_empty() {
            delete_file(&target);
            return Ok(());
        }
        let dir = self.game_definition_dir_path(def_id);
        if !dir_exists(&dir) {
            fs::create_dir_all(&dir)?;
        }
        let json = serde_json::to_string(grantees)?;
        let tmp = format!("{target}.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
        }
        fs::rename(&tmp, &target)?;
        Ok(())
    }

    // ── game collection helpers (one `<id>/` folder per collection) ──────────

    // The folder holding one collection's files (collection.json / shares.json /
    // — later — image.png).
    fn game_collection_dir_path(&self, id: Uuid) -> String {
        Path::new(&self.game_collections_dir)
            .join(id.to_string())
            .to_string_lossy()
            .to_string()
    }

    // Path to a collection's `collection.json`.
    fn game_collection_file_path(&self, id: Uuid) -> String {
        Path::new(&self.game_collection_dir_path(id))
            .join("collection.json")
            .to_string_lossy()
            .to_string()
    }

    // Path to a collection's `image.png` (inside its `<id>/` folder).
    fn game_collection_image_file_path(&self, id: Uuid) -> String {
        Path::new(&self.game_collection_dir_path(id))
            .join("image.png")
            .to_string_lossy()
            .to_string()
    }

    // Reads a collection's JSON file, with its items ordered by `sort_order`.
    // Returns `GameCollectionIdNotFound` if the file does not exist.
    fn read_game_collection_raw(&self, id: Uuid) -> Result<GameCollection, Error> {
        let path = self.game_collection_file_path(id);
        if !file_exists(&path) {
            return Err(Error::GameCollectionIdNotFound(id.to_string()));
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        let mut collection: GameCollection = serde_json::from_reader(reader).map_err(Error::from)?;
        collection.items.sort_by_key(|i| i.sort_order);
        Ok(collection)
    }

    // Atomically writes a collection's `collection.json` via tempfile + rename,
    // creating its `<id>/` folder.
    fn write_game_collection_file(
        &self,
        collection: &GameCollection,
        overwrite: bool,
    ) -> Result<(), Error> {
        let dir = self.game_collection_dir_path(collection.id);
        if !dir_exists(&dir) {
            fs::create_dir_all(&dir)?;
        }
        let target = self.game_collection_file_path(collection.id);
        if !overwrite && file_exists(&target) {
            return Err(Error::Other(format!(
                "game collection {} already exists",
                collection.id
            )));
        }
        let json = serde_json::to_string(collection)?;
        let tmp = format!("{target}.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
        }
        fs::rename(&tmp, &target)?;
        Ok(())
    }

    // Enumerates collection ids (the `<id>/` sub-folders of game_collections).
    fn get_game_collection_ids(&self) -> Result<Vec<Uuid>, Error> {
        if !dir_exists(&self.game_collections_dir) {
            return Ok(Vec::new());
        }
        let ids: Vec<Uuid> = fs::read_dir(&self.game_collections_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    let path = e.path();
                    if !path.is_dir() {
                        return None;
                    }
                    let name = path.file_name()?.to_str()?;
                    Uuid::parse_str(name).ok()
                })
            })
            .collect();
        Ok(ids)
    }

    // Loads every collection, skipping any unreadable file.
    fn read_all_game_collections(&self) -> Result<Vec<GameCollection>, Error> {
        let mut collections = Vec::new();
        for id in self.get_game_collection_ids()? {
            match self.read_game_collection_raw(id) {
                Ok(collection) => collections.push(collection),
                Err(error) => {
                    log::warn!("FileStore game collection read: skipping unreadable '{id}' - {error}");
                }
            }
        }
        Ok(collections)
    }

    // Sorts collections case-insensitively by name (the list-read ordering).
    fn sort_collections_by_name(collections: &mut [GameCollection]) {
        collections
            .sort_by(|a, b| UniCase::new(a.name.as_str()).cmp(&UniCase::new(b.name.as_str())));
    }

    // Returns the id of `owner`'s collection named `name` (case-insensitive), or
    // `None`. Enforces the per-owner unique-name rule on create/update.
    fn find_owner_collection_id_by_name(
        &self,
        owner_id: Uuid,
        name: &str,
    ) -> Result<Option<Uuid>, Error> {
        let target = UniCase::new(name);
        for collection in self.read_all_game_collections()? {
            if collection.owner_id == owner_id && UniCase::new(collection.name.as_str()) == target {
                return Ok(Some(collection.id));
            }
        }
        Ok(None)
    }

    // Path to a collection's `shares.json` (inside its `<id>/` folder).
    fn game_collection_shares_file_path(&self, collection_id: Uuid) -> String {
        Path::new(&self.game_collection_dir_path(collection_id))
            .join("shares.json")
            .to_string_lossy()
            .to_string()
    }

    // Reads a collection's grantee-uuid list (empty when no share file exists).
    fn read_game_collection_grantees(&self, collection_id: Uuid) -> Result<Vec<Uuid>, Error> {
        let path = self.game_collection_shares_file_path(collection_id);
        if !file_exists(&path) {
            return Ok(Vec::new());
        }
        let file = File::open(&path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader::<BufReader<File>, Vec<Uuid>>(reader).map_err(Error::from)
    }

    // Writes a collection's `shares.json`; an empty list removes the file.
    fn write_game_collection_grantees(
        &self,
        collection_id: Uuid,
        grantees: &[Uuid],
    ) -> Result<(), Error> {
        let target = self.game_collection_shares_file_path(collection_id);
        if grantees.is_empty() {
            delete_file(&target);
            return Ok(());
        }
        let dir = self.game_collection_dir_path(collection_id);
        if !dir_exists(&dir) {
            fs::create_dir_all(&dir)?;
        }
        let json = serde_json::to_string(grantees)?;
        let tmp = format!("{target}.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
        }
        fs::rename(&tmp, &target)?;
        Ok(())
    }

    // Walks the audit log and clears `recipient_user_id` and
    // `triggered_by_user_id` columns equal to `id` — the FileStore
    // counterpart to the SQL `ON DELETE SET NULL` FK behaviour. Run by
    // `purge_user` so a hard-deleted user's audit history survives but
    // no longer re-identifies the user.
    fn null_audit_user_id_references(&self, id: Uuid) -> Result<(), Error> {
        for entry_id in self.get_audit_entry_ids()? {
            let mut entry = match self.read_audit_entry_raw(entry_id) {
                Ok(e) => e,
                Err(Error::AuditEntryIdNotFound(_)) => continue,
                Err(error) => {
                    log::warn!(
                        "FileStore purge_user: skipping unreadable audit entry '{entry_id}' - {error}"
                    );
                    continue;
                }
            };
            let mut mutated = false;
            if entry.recipient_user_id == Some(id) {
                entry.recipient_user_id = None;
                mutated = true;
            }
            if entry.triggered_by_user_id == Some(id) {
                entry.triggered_by_user_id = None;
                mutated = true;
            }
            if mutated {
                self.write_audit_entry_file(&entry, true)?;
            }
        }
        Ok(())
    }

    fn make_dir(dir: &str) -> Result<String, Error> {
        let path = PathBuf::from(dir);
        let normalized_path = path.strip_prefix(r"\\?\").unwrap_or(&path).to_path_buf();

        match fs::create_dir_all(normalized_path) {
            Ok(_) => Ok(dir.to_string()),
            Err(error) => Err(Error::Other(format!(
                "Failed to create directory: {dir} - {error}"
            ))),
        }
    }

    // Creates the data directory within the file store
    fn make_data_dir(data_dir: &str) -> Result<String, Error> {
        let os_path = PathBuf::from(data_dir);

        let path = if os_path.is_absolute() {
            os_path.clone()
        } else {
            env::current_dir()?.join(&os_path)
        };

        let normalized_path = path.strip_prefix(r"\\?\").unwrap_or(&path).to_path_buf();

        let dir_path: String = normalized_path
            .to_string_lossy()
            .replace('/', MAIN_SEPARATOR_STR);

        Self::make_dir(&dir_path)
    }

    // Creates a data sub-directory within the file store
    fn make_data_sub_dir(&self, sub_dir: &str) -> Result<String, Error> {
        let path = PathBuf::from(self.data_dir.clone()).join(sub_dir);
        let dir_path: String = path.to_string_lossy().to_string();
        Self::make_dir(&dir_path)
    }

    // Creates the users directory within the file store
    fn make_users_dir(&self) -> Result<String, Error> {
        self.make_data_sub_dir("users")
    }

    fn get_user_sub_dir_path(&self, id: Uuid, sub_dir: &str) -> String {
        PathBuf::from(self.user_dir_path(id))
            .join(sub_dir)
            .to_string_lossy()
            .to_string()
    }

    fn make_user_sub_dir(&self, id: Uuid, sub_dir: &str) -> Result<String, Error> {
        Self::make_dir(&self.get_user_sub_dir_path(id, sub_dir))
    }

    // Creates a user directory within the file store
    fn make_user_dir(&self, id: Uuid) -> Result<String, Error> {
        Self::make_dir(&self.user_dir_path(id))
    }

    // Returns the directory path for a given user id
    fn user_dir_path(&self, id: Uuid) -> String {
        Path::new(&self.users_dir)
            .join(id.to_string())
            .to_string_lossy()
            .to_string()
    }

    // Returns the file path for a given user
    fn user_file_path(&self, id: Uuid) -> String {
        Path::new(&self.user_dir_path(id))
            .join("user.json")
            .to_string_lossy()
            .to_string()
    }

    // Returns the file path for a given user's avatar image. The avatar is
    // always a PNG (the server canonicalises uploads), stored alongside
    // `user.json` in the user's directory so a hard-delete of that directory
    // removes the image with it.
    fn avatar_file_path(&self, id: Uuid) -> String {
        Path::new(&self.user_dir_path(id))
            .join("avatar.png")
            .to_string_lossy()
            .to_string()
    }

    // Returns whether a given user exists
    fn user_exists(&self, id: Uuid) -> bool {
        file_exists(&self.user_file_path(id))
    }

    // Returns whether a given user directory exists
    fn user_dir_exists(&self, id: Uuid) -> bool {
        dir_exists(&self.user_dir_path(id))
    }

    // Writes the file associated whether a given user
    fn write_user_file(&self, user: &User, overwrite: bool) -> Result<(), Error> {
        if !overwrite && self.user_exists(user.id) {
            return Err(Error::UserIdExists(user.id.to_string()));
        }

        if !self.user_dir_exists(user.id) {
            self.make_user_dir(user.id)?;
        }

        let s = user.to_json()?;
        let mut file = File::create(self.user_file_path(user.id))?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }

    // Validate user content
    fn validate_user(&self, user: &User, ignore_id: Uuid) -> Result<(), Error> {
        validate_user_fields(user)?;
        if self.user_name_exists(&user.username, ignore_id) {
            return Err(Error::UserNameExists());
        }
        // Check uniqueness for every email row on the user — mirrors the
        // global UNIQUE constraint on `user_emails.email` in the SQL schema.
        for row in &user.emails {
            if self.user_email_exists(&row.email, ignore_id) {
                return Err(Error::UserEmailExists());
            }
        }
        Ok(())
    }

    // Read a user definition without applying the soft-delete read filter.
    // Internal use only — the delete/purge paths need to see the row even
    // after `delete_at` has been set. Public callers must go through
    // [`read_user`] (or [`load_user_if_present`]) so that soft-deleted users
    // are invisible at the trait surface.
    fn read_user_raw(&self, id: Uuid) -> Result<User, Error> {
        if !self.user_exists(id) {
            return Err(Error::UserIdNotFound(id.to_string()));
        }
        let path = self.user_file_path(id);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        match serde_json::from_reader::<BufReader<File>, User>(reader) {
            Ok(user) => Ok(user),
            Err(error) => Err(Error::from(error)),
        }
    }

    // Read a user definition, treating soft-deleted users as if they did not
    // exist — every UserStore read path goes through this entrypoint.
    fn read_user(&self, id: Uuid) -> Result<User, Error> {
        let user = self.read_user_raw(id)?;
        if !user.is_active() {
            return Err(Error::UserIdNotFound(id.to_string()));
        }
        Ok(user)
    }

    // Locate the first user with a given string field value
    fn find_user_by_string_field(
        &self,
        field_name: &str,
        search_value: &str,
        ignore_id: Uuid,
    ) -> Result<User, Error> {
        let ids = self.get_user_ids()?;
        let search = UniCase::new(search_value);
        for id in ids {
            if id == ignore_id {
                continue;
            }
            let Some(user) = self.load_user_if_present(id)? else {
                continue;
            };
            let Some(user_value) = user.get_string_field(field_name) else {
                continue;
            };
            if UniCase::new(user_value) == search {
                return Ok(user);
            }
        }
        Err(Error::UserNotFound())
    }

    // Checks whether a given username exists in the file store
    fn user_name_exists(&self, name: &str, ignore_id: Uuid) -> bool {
        self.find_user_by_string_field("username", name, ignore_id)
            .is_ok()
    }

    // Checks whether a given user email exists in the file store, looking
    // across every email row of every user (mirrors the SQL `user_emails.email`
    // UNIQUE constraint).
    fn user_email_exists(&self, email: &str, ignore_id: Uuid) -> bool {
        self.find_user_by_any_email_internal(email, ignore_id).is_ok()
    }

    // Locates a user with any email row matching `search_value` (case-
    // insensitively), regardless of verification state. Used by
    // `user_email_exists` for uniqueness checks — those need to see every
    // address on every user, verified or not, otherwise an attacker could
    // squat on `victim@example.com` with `verified = false` and the
    // collision check would let another user re-register the same address.
    fn find_user_by_any_email_internal(
        &self,
        search_value: &str,
        ignore_id: Uuid,
    ) -> Result<User, Error> {
        let ids = self.get_user_ids()?;
        let search = UniCase::new(search_value);
        for id in ids {
            if id == ignore_id {
                continue;
            }
            let Some(user) = self.load_user_if_present(id)? else {
                continue;
            };
            if user
                .emails
                .iter()
                .any(|row| UniCase::new(row.email.clone()) == search)
            {
                return Ok(user);
            }
        }
        Err(Error::UserNotFound())
    }

    // Locates a user with a `verified = true` email row matching
    // `search_value` (case-insensitively). The verified filter lives in
    // the lookup itself rather than at every callsite — see the trait
    // doc-comment for rationale.
    fn find_user_by_verified_email_internal(
        &self,
        search_value: &str,
    ) -> Result<User, Error> {
        let ids = self.get_user_ids()?;
        let search = UniCase::new(search_value);
        for id in ids {
            let Some(user) = self.load_user_if_present(id)? else {
                continue;
            };
            if user.emails.iter().any(|row| {
                row.verified && UniCase::new(row.email.clone()) == search
            }) {
                return Ok(user);
            }
        }
        Err(Error::UserNotFound())
    }

    // Loads a user by id for iterator-style read paths, returning `None`
    // both for soft-deleted users (silent — that is normal state) and for
    // user directories whose `user.json` is missing or unreadable (with a
    // warning — that is a corruption-style symptom). All other errors
    // propagate.
    fn load_user_if_present(&self, id: Uuid) -> Result<Option<User>, Error> {
        match self.read_user_raw(id) {
            Ok(user) if !user.is_active() => Ok(None),
            Ok(user) => Ok(Some(user)),
            Err(Error::UserIdNotFound(_)) => {
                log::warn!("Skipping user directory '{id}': user.json is missing or unreadable");
                Ok(None)
            }
            Err(err) => Err(err),
        }
    }

    // Returns the list of user ids associated with the file store
    fn get_user_ids(&self) -> Result<Vec<Uuid>, Error> {
        let ids: Vec<Uuid> = fs::read_dir(&self.users_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.path().is_dir() {
                        e.file_name()
                            .to_str()
                            .and_then(|name| Uuid::parse_str(name).ok())
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(ids)
    }

    // Loads every active (non-soft-deleted) user, sorted by username. The
    // unbounded "load all users" primitive backing `get_users` (paged slice) and
    // `search_users_by_username_prefix` (prefix filter) — kept private so the
    // trait exposes only paged reads.
    fn read_all_active_users(&self) -> Result<Vec<User>, Error> {
        let mut users: Vec<User> = Vec::new();
        for id in self.get_user_ids()? {
            if let Some(user) = self.load_user_if_present(id)? {
                users.push(user);
            }
        }
        users.sort_by(|a, b| a.username.cmp(&b.username));
        Ok(users)
    }

    // Returns the maze id for a given maze name
    fn make_maze_id(&self, name: &str) -> String {
        format!("{name}.json")
    }

    /// Returns the absolute path to `owner`'s mazes directory under the
    /// configured data directory. Used by callers that need to list /
    /// snapshot the on-disk maze files outside of the `MazeStore` trait
    /// surface (e.g. backup tooling).
    ///
    /// # Examples
    ///
    /// Locate the mazes directory for a freshly-created user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    ///
    /// let dir = store.get_mazes_dir(&user);
    /// println!("Mazes for {} live at {}", user.username, dir);
    /// # });
    /// ```
    pub fn get_mazes_dir(&self, owner: &User) -> String {
        Path::new(&self.user_dir_path(owner.id))
            .join("mazes")
            .to_string_lossy()
            .to_string()
    }

    // Creates the mazes directory within the file store for a given owner
    fn make_user_mazes_dir(&self, owner: &User) -> Result<String, Error> {
        self.make_user_sub_dir(owner.id, "mazes")
    }

    // Returns whether a given mazes directory exists
    fn user_mazes_dir_exists(&self, owner: &User) -> bool {
        dir_exists(&self.get_mazes_dir(owner))
    }

    // Returns the maze file path for a given maze id
    fn maze_path(&self, owner: &User, id: &str) -> String {
        Path::new(&self.get_mazes_dir(owner))
            .join(id)
            .to_string_lossy()
            .to_string()
    }

    // Checks whether a given maze file exists
    fn maze_exists(&self, owner: &User, id: &str) -> bool {
        file_exists(&self.maze_path(owner, id))
    }

    // Counts the mazes `owner` currently owns (files in their mazes dir); 0 when
    // the dir doesn't exist yet. Used to enforce the per-user maze cap.
    fn count_owner_mazes(&self, owner: &User) -> usize {
        fs::read_dir(self.get_mazes_dir(owner))
            .map(|rd| rd.filter_map(|e| e.ok()).filter(|e| e.path().is_file()).count())
            .unwrap_or(0)
    }

    // Returns the actual on-disk filename of any maze whose name matches
    // `name` case-insensitively for `owner`, or None if no such maze
    // exists.
    //
    // Used by `find_maze_by_name` and `create_maze` so that case-insensitive
    // matching is enforced in code rather than via filesystem semantics —
    // NTFS and APFS-default are case-insensitive, ext4 is not, so relying
    // on the filesystem makes behaviour OS-dependent.
    fn find_maze_filename_ci(&self, owner: &User, name: &str) -> Option<String> {
        if name.is_empty() {
            return None;
        }
        let target = UniCase::new(self.make_maze_id(name));
        let mazes_dir = self.get_mazes_dir(owner);
        let entries = std::fs::read_dir(&mazes_dir).ok()?;
        for entry in entries.flatten() {
            if let Some(filename) = entry.file_name().to_str()
                && UniCase::new(filename.to_string()) == target
            {
                return Some(filename.to_string());
            }
        }
        None
    }

    // Wriets a maze file
    fn write_maze_file(
        &self,
        owner: &User,
        maze: &mut Maze,
        id: &str,
        overwrite: bool,
    ) -> Result<(), Error> {
        maze.id = id.to_string();

        if !self.user_mazes_dir_exists(owner) {
            self.make_user_mazes_dir(owner)?;
        }

        if !overwrite && self.maze_exists(owner, id) {
            return Err(Error::MazeIdExists(id.to_string()));
        }

        let s = maze.to_json()?;
        let mut file = File::create(self.maze_path(owner, id))?;
        file.write_all(s.as_bytes())?;
        Ok(())
    }
}

impl Default for FileStore {
    fn default() -> Self {
        Self::new(&FileStoreConfig::default())
    }
}

#[async_trait]
impl UserStore for FileStore {
    /// Adds the default admin user to the store if it doesn't already exist, else returns it
    ///
    /// # Examples
    ///
    /// Try to create a new user within a file store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the default admin user within the file store if needed
    /// match store.init_default_admin_user("admin", "admin@maze.local", "my_password_hash").await {
    ///     Ok(user) => {
    ///         println!(
    ///             "Successfully intiialized default admin user with id {} in the file store",
    ///             user.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to initialized default admin user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn init_default_admin_user(
        &mut self,
        username: &str,
        email: &str,
        password_hash: &str,
    ) -> Result<User, Error> {
        match self.find_user_by_name(username).await {
            Ok(user) => Ok(user),
            Err(error) => match error {
                Error::UserNotFound() => {
                    let mut user = User::default();
                    user.username = username.to_string();
                    user.set_primary_email_address(email);
                    user.is_admin = true;
                    user.password_hash = password_hash.to_string();
                    self.create_user(&mut user).await?;
                    Ok(user)
                }
                _ => Err(error),
            },
        }
    }
    /// Adds a new user to the store and sets the allocated `id` within the user object
    ///
    /// # Examples
    ///
    /// Try to create a new user within a file store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn create_user(&mut self, user: &mut User) -> Result<(), Error> {
        user.id = User::new_id();
        user.api_key = User::new_api_key();
        self.validate_user(user, Uuid::nil())?;
        self.write_user_file(user, false)?;
        self.make_user_mazes_dir(user)?;
        Ok(())
    }
    /// Deletes a user from the store
    ///
    /// # Examples
    ///
    /// Try to create and then delete a user within a file store
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         match store.delete_user(user.id).await {
    ///             Ok(_) => {
    ///                 println!("Successfully deleted user from the file store");
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to delete user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn delete_user(&mut self, id: Uuid) -> Result<(), Error> {
        if id.is_nil() {
            return Err(Error::UserIdMissing());
        }
        // Read the row through the raw path so an idempotent re-call on an
        // already-soft-deleted user surfaces UserIdNotFound rather than a
        // double-soft-delete (mirrors the SqlStore `WHERE deleted_at IS NULL`
        // guard on the UPDATE).
        let mut user = self.read_user_raw(id)?;
        if !user.is_active() {
            return Err(Error::UserIdNotFound(id.to_string()));
        }
        user.deleted_at = Some(generate_now_millis());
        // Scramble username to free the original value for reuse by a future
        // signup. The form `deleted-<uuid>` is 44 chars, well under
        // VARCHAR(64), and independent of the original username's length.
        user.username = format!("deleted-{id}");
        // Hard-clear cascaded data that has no audit value: pending sessions,
        // OAuth identities, email rows. These mirror the SqlStore cascade.
        user.logins.clear();
        user.oauth_identities.clear();
        user.emails.clear();
        self.write_user_file(&user, true)?;
        // Hard-delete the user's own score history and the boards of the mazes
        // about to be deleted (other players' runs on them). Runs before the
        // mazes are removed so `user_maze_ids` can still read them. Mirrors the
        // SqlStore app-level cascade.
        let owned_mazes = self.user_maze_ids(id);
        self.delete_score_rows(Some(id), &owned_mazes)?;
        // Hard-delete the user's mazes — the user asked to delete the
        // account, their content goes with it. Mirrors `mazes ON DELETE
        // CASCADE` in the SQL schema.
        let mazes_dir = Path::new(&self.user_dir_path(id))
            .join("mazes")
            .to_string_lossy()
            .to_string();
        if dir_exists(&mazes_dir) {
            delete_dir(&mazes_dir);
        }
        // Hard-delete the user's pending one-time tokens. A live reset or
        // invite token belonging to a deleted account is a phishing
        // vector. Mirrors `one_time_tokens ON DELETE CASCADE` in the SQL
        // schema, run explicitly here because soft-delete updates
        // `users.deleted_at` rather than removing the users row.
        for token_id in self.get_token_ids()? {
            match self.read_token_raw(token_id) {
                Ok(token) if token.user_id == id => {
                    delete_file(&self.token_file_path(token_id));
                }
                Ok(_) => {}
                Err(Error::TokenIdNotFound(_)) => {}
                Err(error) => {
                    log::warn!(
                        "FileStore delete_user: skipping unreadable token '{token_id}' - {error}"
                    );
                }
            }
        }
        // Hard-delete the user's game definitions + their share grants, and
        // strip the user from every remaining definition's grantee list. Mirrors
        // the SqlStore FK cascade on `game_definitions.owner_id` and
        // `game_definition_shares.grantee_user_id`.
        let mut owned_definition_ids: Vec<Uuid> = Vec::new();
        for def_id in self.get_game_definition_ids()? {
            match self.read_game_definition_raw(def_id) {
                Ok(def) if def.owner_id == id => {
                    owned_definition_ids.push(def_id);
                    delete_dir(&self.game_definition_dir_path(def_id));
                }
                Ok(_) => {
                    let mut grantees = self.read_game_definition_grantees(def_id)?;
                    if grantees.contains(&id) {
                        grantees.retain(|g| *g != id);
                        self.write_game_definition_grantees(def_id, &grantees)?;
                    }
                }
                Err(_) => {}
            }
        }
        // Clear the boards of the removed definitions (other players' runs on
        // them), keyed by the `def:<id>` challenge subject (Static + Daily).
        if !owned_definition_ids.is_empty() {
            self.delete_scores_matching(|e| {
                e.challenge.as_deref().is_some_and(|c| {
                    owned_definition_ids
                        .iter()
                        .any(|d| c == format!("def:{d}") || c.starts_with(&format!("def:{d}:")))
                })
            })?;
        }
        // Hard-delete the user's game collections + their share grants, and strip
        // the user from every remaining collection's grantee list. Mirrors the
        // SqlStore FK cascade on `game_collections.owner_id` and
        // `game_collection_shares.grantee_user_id`.
        for collection_id in self.get_game_collection_ids()? {
            match self.read_game_collection_raw(collection_id) {
                Ok(collection) if collection.owner_id == id => {
                    delete_dir(&self.game_collection_dir_path(collection_id));
                }
                Ok(_) => {
                    let mut grantees = self.read_game_collection_grantees(collection_id)?;
                    if grantees.contains(&id) {
                        grantees.retain(|g| *g != id);
                        self.write_game_collection_grantees(collection_id, &grantees)?;
                    }
                }
                Err(_) => {}
            }
        }
        Ok(())
    }
    /// True hard-delete: removes the user directory outright. Mirrors the
    /// SqlStore `purge_user` semantics — intended for retention /
    /// right-to-erasure flows where the soft-deleted row must also be
    /// cleared. Reachable on either an active or already-soft-deleted user.
    ///
    /// # Examples
    ///
    /// Soft-delete a user, then purge them so the row is truly gone
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// store.create_user(&mut user).await.expect("create_user");
    /// store.delete_user(user.id).await.expect("soft-delete");
    /// match store.purge_user(user.id).await {
    ///     Ok(_) => println!("User purged from the file store"),
    ///     Err(error) => println!("Failed to purge user => {}", error),
    /// }
    /// # });
    /// ```
    async fn purge_user(&mut self, id: Uuid) -> Result<(), Error> {
        if id.is_nil() {
            return Err(Error::UserIdMissing());
        }
        if !self.user_dir_exists(id) {
            return Err(Error::UserIdNotFound(id.to_string()));
        }
        // FileStore counterpart to the SQL `ON DELETE SET NULL` FK on
        // `email_audit_log` — purge clears user-id columns on every
        // audit row that referenced this user, so the audit history
        // survives but no longer re-identifies them.
        self.null_audit_user_id_references(id)?;
        // FileStore counterpart to the SQL `ON DELETE CASCADE` on
        // `score_history` — the score rows live in a flat directory (not under
        // the user dir being removed below), so drop them explicitly: the
        // user's own runs and the boards of their mazes.
        let owned_mazes = self.user_maze_ids(id);
        self.delete_score_rows(Some(id), &owned_mazes)?;
        delete_dir(&self.user_dir_path(id));
        if self.user_dir_exists(id) {
            return Err(Error::Other(format!(
                "user directory {id} still exists after purge_user"
            )));
        }
        Ok(())
    }
    /// Updates a user within the store
    ///
    /// # Examples
    ///
    /// Try to create and then update a user within a file store
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Change the user full name
    ///         user.full_name = "John Henry Smith".to_string();
    ///         match store.update_user(&mut user).await {
    ///             Ok(_) => {
    ///                 println!("Successfully update user within the file store");
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to update user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn update_user(&mut self, user: &mut User) -> Result<(), Error> {
        if user.id == Uuid::nil() {
            return Err(Error::UserIdMissing());
        }
        if !self.user_exists(user.id) {
            return Err(Error::UserIdNotFound(user.id.to_string()));
        }
        self.validate_user(user, user.id)?;
        self.write_user_file(user, true)?;
        Ok(())
    }
    /// Loads a user from the store
    ///
    /// # Examples
    ///
    /// Try to create and then load a user from within a file store
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to load it again and display the results
    ///         match store.get_user(user.id).await {
    ///             Ok(user_loaded) => {
    ///                 println!("Successfully loaded user from within the file store => {:?}", user_loaded);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to load user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_user(&self, id: Uuid) -> Result<User, Error> {
        self.read_user(id)
    }
    /// Locates a user by their username within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user from within a file store
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by username and display the results
    ///         match store.find_user_by_name(&user.username).await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the file store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_name(&self, name: &str) -> Result<User, Error> {
        self.find_user_by_string_field("username", name, Uuid::nil())
    }
    /// Locates a user by an email address within the store, returning the
    /// match only if the matching `user_emails` row is `verified = true`.
    /// Unverified rows are invisible to this lookup. See the trait
    /// doc-comment for the security rationale.
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user from within a file store by email
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by email and display the results
    ///         match store.find_user_by_verified_email(user.email()).await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the file store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_verified_email(&self, email: &str) -> Result<User, Error> {
        self.find_user_by_verified_email_internal(email)
    }

    /// Locates a user by an email address regardless of verification state.
    /// Delegates to the existing `find_user_by_any_email_internal` walker
    /// (originally added for the unique-email collision check) with
    /// `Uuid::nil()` as the ignore id, so every active user is considered.
    ///
    /// See the trait doc-comment for usage rules — auth code must use
    /// [`UserStore::find_user_by_verified_email`] instead.
    ///
    /// # Examples
    ///
    /// An unverified secondary email still resolves to its user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".to_string(),
    ///     full_name: "Alice".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    /// store.add_user_email(user.id, "alice2@example.com", false).await.unwrap();
    ///
    /// // The verified-only lookup misses the unverified address, but the
    /// // any-state lookup finds it.
    /// assert!(store.find_user_by_verified_email("alice2@example.com").await.is_err());
    /// let found = store.find_user_by_email_any_state("alice2@example.com").await.unwrap();
    /// assert_eq!(found.id, user.id);
    /// # });
    /// ```
    async fn find_user_by_email_any_state(&self, email: &str) -> Result<User, Error> {
        self.find_user_by_any_email_internal(email, Uuid::nil())
    }

    /// Locates a user by their api key within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user by its api key from within a file store
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by username and display the results
    ///         match store.find_user_by_api_key(user.api_key).await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the file store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_api_key(&self, api_key: Uuid) -> Result<User, Error> {
        let ids = self.get_user_ids()?;
        for id in ids {
            if let Some(user) = self.load_user_if_present(id)?
                && user.api_key == api_key
            {
                return Ok(user);
            }
        }
        Err(Error::UserNotFound())
    }
    /// Locates a user by their login id within the store
    ///
    /// # Examples
    ///
    /// Try to create and then locate a user by its login id within a file store
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail, UserLogin};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// 
    /// // Create the login tokens
    /// let login = UserLogin::new(24, Some("123.456.789.012".to_string()), Some("Device info string".to_string()));
    /// let search_login_id = login.id; 
    /// let logins = vec![login];
    /// 
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins,
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again using the login id and display the results
    ///         match store.find_user_by_login_id(search_login_id).await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the file store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_login_id(&self, login_id: Uuid) -> Result<User, Error>{
        let ids = self.get_user_ids()?;
        for id in ids {
            if let Some(user) = self.load_user_if_present(id)?
                && user.contains_valid_login(login_id)
            {
                return Ok(user);
            }
        }
        Err(Error::UserNotFound())
    }
    /// Locates a user by an OAuth identity `(provider, provider_user_id)` pair.
    /// `provider` is matched case-insensitively (canonical providers are stored
    /// lowercase: "google", "github"); `provider_user_id` is matched exactly (it
    /// is an opaque stable id from the identity provider).
    ///
    /// # Examples
    ///
    /// Try to create a user with a linked Google identity and then locate it by
    /// its OAuth identity within a file store
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{OAuthIdentity, User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition with a linked Google identity
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![OAuthIdentity::new(
    ///         "google".to_string(),
    ///         "google-sub-jsmith".to_string(),
    ///         Some("jsmith@company.com".to_string()),
    ///     )],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to find it again by its OAuth identity and display the results
    ///         match store.find_user_by_oauth_identity("google", "google-sub-jsmith").await {
    ///             Ok(user_found) => {
    ///                 println!("Successfully found user within the file store => {:?}", user_found);
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to find user => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_user_by_oauth_identity(&self, provider: &str, provider_user_id: &str) -> Result<User, Error> {
        let ids = self.get_user_ids()?;
        for id in ids {
            if let Some(user) = self.load_user_if_present(id)?
                && user.oauth_identities.iter().any(|identity| {
                    identity.provider.eq_ignore_ascii_case(provider)
                        && identity.provider_user_id == provider_user_id
                })
            {
                return Ok(user);
            }
        }
        Err(Error::UserNotFound())
    }
    /// A page of active users, ordered by username then id, sliced to
    /// `limit`/`offset` (pass a large `limit` for "all"). See
    /// [`UserStore::get_users`].
    ///
    /// # Examples
    ///
    /// Try to create a user within a file store and then load the list of registered users and display their count
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to load the user list and display the results
    ///         match store.get_users(10, 0).await {
    ///             Ok(users_found) => {
    ///                 println!("Successfully loaded {} users from within the file store", users_found.len());
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to load users => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_users(&self, limit: u32, offset: u32) -> Result<Vec<User>, Error> {
        // Dev-only backend: load all active users, then slice the page.
        let mut users = self.read_all_active_users()?;
        users.sort_by(|a, b| a.username.cmp(&b.username).then(a.id.cmp(&b.id)));
        Ok(users.into_iter().skip(offset as usize).take(limit as usize).collect())
    }

    /// A page of active users whose username starts with `prefix`
    /// (case-insensitive). See [`UserStore::search_users_by_username_prefix`].
    ///
    /// # Examples
    ///
    /// Prefix-match users, case-insensitively
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// for name in ["alice", "alina", "bob"] {
    ///     let mut u = User {
    ///         id: Uuid::nil(), is_admin: false, username: name.to_string(),
    ///         full_name: name.to_string(),
    ///         emails: vec![UserEmail::new_primary_verified(&format!("{name}@example.com"))],
    ///         password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///         oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///         last_sign_in_at: None, avatar_updated_at: None,
    ///     };
    ///     store.create_user(&mut u).await.unwrap();
    /// }
    ///
    /// let hits = store.search_users_by_username_prefix("AL", 10, 0).await.unwrap();
    /// assert_eq!(
    ///     hits.iter().map(|u| u.username.clone()).collect::<Vec<_>>(),
    ///     vec!["alice", "alina"]
    /// );
    /// # });
    /// ```
    async fn search_users_by_username_prefix(
        &self,
        prefix: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<User>, Error> {
        // Dev-only backend: filter the active users in memory (`get_users`
        // already excludes soft-deleted), sort, and slice the page.
        let prefix = prefix.trim().to_lowercase();
        if prefix.is_empty() {
            return Ok(Vec::new());
        }
        let mut users: Vec<User> = self
            .read_all_active_users()?
            .into_iter()
            .filter(|u| u.username.to_lowercase().starts_with(&prefix))
            .collect();
        users.sort_by(|a, b| {
            a.username
                .to_lowercase()
                .cmp(&b.username.to_lowercase())
                .then(a.id.cmp(&b.id))
        });
        Ok(users.into_iter().skip(offset as usize).take(limit as usize).collect())
    }

    /// Returns the list of admin users within the store
    ///
    /// # Examples
    ///
    /// Try to create an admin user within a file store and then load the list of admin users and display their count
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Create the admin user definition
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: true,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    ///
    /// // Create the admin user within the file store
    /// match store.create_user(&mut user).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created admin user with id {} in the file store",
    ///             user.id
    ///         );
    ///         // Now attempt to load the admin user list and display the results
    ///         match store.get_admin_users().await {
    ///             Ok(admins_found) => {
    ///                 println!("Successfully loaded {} admin users from within the file store", admins_found.len());
    ///             }
    ///             Err(error) => {
    ///                 println!(
    ///                     "Failed to load admin users => {}",
    ///                      error
    ///                 );
    ///             }
    ///         }
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create user => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_admin_users(&self) -> Result<Vec<User>, Error> {
        let ids = self.get_user_ids()?;
        let mut admins: Vec<User> = Vec::new();
        for id in ids {
            if let Some(user) = self.load_user_if_present(id)?
                && user.is_admin
            {
                admins.push(user);
            }
        }
        Ok(admins)
    }

    /// Returns whether at least one user exists in the file store.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if any valid user is present, `Ok(false)` if the store is
    /// empty (or contains only orphan directories).
    ///
    /// # Examples
    ///
    /// Check whether the store has any users before deciding to seed a
    /// default admin account
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// match store.has_users().await {
    ///     Ok(true) => println!("Store already has users — skip bootstrap"),
    ///     Ok(false) => println!("Store is empty — seed a default admin"),
    ///     Err(error) => println!("Failed to check store: {}", error),
    /// }
    /// # });
    /// ```
    async fn has_users(&self) -> Result<bool, Error> {
        for id in self.get_user_ids()? {
            if self.load_user_if_present(id)?.is_some() {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns whether at least one *active* admin user is present in the
    /// file store (`is_admin = true` AND `deleted_at IS NULL`).
    ///
    /// `load_user_if_present` already filters soft-deleted users out of the
    /// iteration, so the body only needs to check `is_admin`. Used by
    /// startup so a soft-deleted lone admin doesn't prevent the default
    /// admin from being recreated on next launch.
    ///
    /// # Returns
    ///
    /// `Ok(true)` if at least one active admin user exists, `Ok(false)`
    /// otherwise (no users, no admins, or every admin has been soft-deleted).
    ///
    /// # Examples
    ///
    /// Probe the store before deciding whether to seed a default admin
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// match store.has_active_admin_user().await {
    ///     Ok(true) => println!("Active admin already present — skip bootstrap"),
    ///     Ok(false) => println!("No active admin — seed a default admin"),
    ///     Err(error) => println!("Failed to check store: {}", error),
    /// }
    /// # });
    /// ```
    async fn has_active_admin_user(&self) -> Result<bool, Error> {
        for id in self.get_user_ids()? {
            if let Some(user) = self.load_user_if_present(id)?
                && user.is_admin
            {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Adds a non-primary email row to the user. See the `UserStore`
    /// trait doc-comment for the full contract; pass `verified = true`
    /// for trusted sources (OAuth-link, admin seed) and `verified = false`
    /// for self-asserted user-typed emails.
    ///
    /// # Examples
    ///
    /// Add a secondary unverified email to an existing user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "alice".to_string(),
    ///     full_name: "Alice".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// let row = store
    ///     .add_user_email(user.id, "alice2@example.com", false)
    ///     .await
    ///     .expect("add secondary");
    /// assert!(!row.verified);
    /// # });
    /// ```
    async fn add_user_email(
        &mut self,
        user_id: Uuid,
        email: &str,
        verified: bool,
    ) -> Result<UserEmail, Error> {
        let mut user = self.read_user(user_id)?;
        // Validate format first so callers see EmailMissing / EmailInvalid
        // before any uniqueness probe.
        validate_email_format(email)?;
        // Reject if THIS user already has this address. (We can't lean on
        // `user_email_exists(_, user_id)` for this — that helper skips the
        // user_id passed as `ignore_id`.)
        if user
            .emails
            .iter()
            .any(|r| r.email.eq_ignore_ascii_case(email))
        {
            return Err(Error::UserEmailExists());
        }
        // Reject if any OTHER user already has this address (mirrors the
        // SQL `user_emails.email` UNIQUE constraint).
        if self.user_email_exists(email, user_id) {
            return Err(Error::UserEmailExists());
        }
        let row = UserEmail {
            email: email.to_string(),
            is_primary: false,
            verified,
            verified_at: if verified {
                Some(generate_now_millis())
            } else {
                None
            },
        };
        user.emails.push(row.clone());
        self.write_user_file(&user, true)?;
        Ok(row)
    }

    /// Removes a non-primary, non-last email row from the user. See the
    /// trait doc-comment for the rejection rules.
    ///
    /// # Examples
    ///
    /// Add a secondary email then remove it
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// store.add_user_email(user.id, "alice2@example.com", true).await.expect("add");
    /// store.remove_user_email(user.id, "alice2@example.com").await.expect("remove");
    /// # });
    /// ```
    async fn remove_user_email(
        &mut self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), Error> {
        let mut user = self.read_user(user_id)?;
        let idx = find_email_row_index(&user, email)?;
        if user.emails.len() == 1 {
            return Err(Error::UserEmailIsLast());
        }
        if user.emails[idx].is_primary {
            return Err(Error::UserEmailIsPrimary());
        }
        user.emails.remove(idx);
        // Drop any OAuth identity rows whose `provider_email` matches the
        // removed address. See the trait doc for the invariant this
        // upholds — otherwise an OAuth provider could still authenticate
        // the user via branch 1 of `account::resolve` (which matches by
        // `(provider, provider_user_id)`, not by current email).
        user.oauth_identities.retain(|id| match id.provider_email.as_deref() {
            Some(addr) => !addr.eq_ignore_ascii_case(email),
            None => true,
        });
        self.write_user_file(&user, true)?;
        Ok(())
    }

    /// Promotes the named email row to primary. The target must already
    /// be `verified = true`; promoting an unverified row is rejected to
    /// stop a session-hijacker from redirecting password resets to an
    /// attacker-controlled mailbox.
    ///
    /// # Examples
    ///
    /// Promote a verified secondary to primary
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// store.add_user_email(user.id, "alice2@example.com", true).await.expect("add");
    /// store.set_primary_email(user.id, "alice2@example.com").await.expect("promote");
    /// # });
    /// ```
    async fn set_primary_email(
        &mut self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), Error> {
        let mut user = self.read_user(user_id)?;
        let idx = find_email_row_index(&user, email)?;
        if !user.emails[idx].verified {
            return Err(Error::UserEmailNotVerified());
        }
        // Clear the flag on every other row, then set it on the target.
        // Done as two passes (rather than in one loop with the index) so
        // the invariant "exactly one primary" is restored before we save.
        for (i, row) in user.emails.iter_mut().enumerate() {
            row.is_primary = i == idx;
        }
        self.write_user_file(&user, true)?;
        Ok(())
    }

    /// Marks the named email row verified, refreshing `verified_at` to
    /// the current time. Idempotent — re-marking an already-verified row
    /// just updates the timestamp.
    ///
    /// # Examples
    ///
    /// Add an unverified secondary and then verify it
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// store.add_user_email(user.id, "alice2@example.com", false).await.expect("add");
    /// store.mark_email_verified(user.id, "alice2@example.com").await.expect("mark verified");
    /// # });
    /// ```
    async fn mark_email_verified(
        &mut self,
        user_id: Uuid,
        email: &str,
    ) -> Result<(), Error> {
        let mut user = self.read_user(user_id)?;
        let idx = find_email_row_index(&user, email)?;
        user.emails[idx].verified = true;
        user.emails[idx].verified_at = Some(generate_now_millis());
        self.write_user_file(&user, true)?;
        Ok(())
    }
    /// Stores (or replaces) the user's avatar PNG at `users/<id>/avatar.png`
    /// and stamps [`data_model::User::avatar_updated_at`]. The bytes are
    /// written via tempfile + rename so a concurrent reader never sees a
    /// half-written image, and the marker is stamped only after the bytes
    /// land so the "has an avatar" signal is never set without bytes behind
    /// it. The bytes are stored verbatim — the caller is responsible for
    /// having canonicalised them to a PNG.
    ///
    /// # Examples
    ///
    /// Create a user, set an avatar, and read it back
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    ///
    /// store.set_user_avatar(user.id, vec![0x89, 0x50, 0x4E, 0x47]).await.unwrap();
    /// let bytes = store.get_user_avatar(user.id).await.unwrap();
    /// assert_eq!(bytes, Some(vec![0x89, 0x50, 0x4E, 0x47]));
    /// assert!(store.get_user(user.id).await.unwrap().avatar_updated_at.is_some());
    /// # });
    /// ```
    async fn set_user_avatar(&mut self, id: Uuid, png_bytes: Vec<u8>) -> Result<(), Error> {
        // `read_user` applies the soft-delete filter, so an unknown or
        // soft-deleted id surfaces UserIdNotFound here before any bytes land.
        let mut user = self.read_user(id)?;
        let target = self.avatar_file_path(id);
        let tmp = format!("{target}.tmp");
        {
            let mut file = File::create(&tmp)?;
            file.write_all(&png_bytes)?;
        }
        fs::rename(&tmp, &target)?;
        user.avatar_updated_at = Some(generate_now_millis());
        self.write_user_file(&user, true)?;
        Ok(())
    }
    /// Loads the user's avatar bytes, or `None` when no avatar file is
    /// present (never set, since cleared, or no such user directory).
    ///
    /// # Examples
    ///
    /// A freshly-created user has no avatar
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    ///
    /// assert_eq!(store.get_user_avatar(user.id).await.unwrap(), None);
    /// # });
    /// ```
    async fn get_user_avatar(&self, id: Uuid) -> Result<Option<Vec<u8>>, Error> {
        let path = self.avatar_file_path(id);
        if !file_exists(&path) {
            return Ok(None);
        }
        Ok(Some(fs::read(&path)?))
    }
    /// Removes the user's avatar file if present and clears
    /// [`data_model::User::avatar_updated_at`]. Idempotent — clearing a user
    /// with no avatar (or no such user) is a successful no-op.
    ///
    /// # Examples
    ///
    /// Setting then clearing leaves no avatar behind
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(),
    ///     is_admin: false,
    ///     username: "jsmith".to_string(),
    ///     full_name: "John Smith".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("jsmith@company.com")],
    ///     password_hash: "Hashed password".to_string(),
    ///     api_key: Uuid::nil(),
    ///     logins: vec![],
    ///     oauth_identities: vec![],
    ///     deleted_at: None,
    ///     created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    /// store.set_user_avatar(user.id, vec![1, 2, 3]).await.unwrap();
    ///
    /// store.clear_user_avatar(user.id).await.unwrap();
    /// assert_eq!(store.get_user_avatar(user.id).await.unwrap(), None);
    /// assert!(store.get_user(user.id).await.unwrap().avatar_updated_at.is_none());
    /// # });
    /// ```
    async fn clear_user_avatar(&mut self, id: Uuid) -> Result<(), Error> {
        let path = self.avatar_file_path(id);
        if file_exists(&path) {
            delete_file(&path);
        }
        // Clear the marker only when an active user currently advertises one,
        // avoiding a needless user.json rewrite on the common no-op path.
        if let Ok(mut user) = self.read_user(id)
            && user.avatar_updated_at.is_some()
        {
            user.avatar_updated_at = None;
            self.write_user_file(&user, true)?;
        }
        Ok(())
    }
}

/// Returns the index of the email row matching `email` (case-insensitively)
/// within `user.emails`, or `Error::UserEmailNotFound` if no row matches.
/// Centralises the lookup that every email-mutating `UserStore` method
/// performs against the in-memory `User` value before it writes back.
fn find_email_row_index(user: &User, email: &str) -> Result<usize, Error> {
    user.emails
        .iter()
        .position(|row| row.email.eq_ignore_ascii_case(email))
        .ok_or_else(|| Error::UserEmailNotFound(email.to_string()))
}

/// Returns the current UTC time truncated to millisecond precision so it
/// round-trips losslessly through both the JSON-on-disk format and the SQL
/// store's RFC 3339 storage format. Mirrors what
/// `UserEmail::new_primary_verified` does.
fn generate_now_millis() -> chrono::DateTime<chrono::Utc> {
    use chrono::SubsecRound;
    chrono::Utc::now().trunc_subsecs(3)
}

/// Atomically writes image bytes to `path` (tempfile + rename), creating the
/// parent folder if missing. Shared by the definition + collection image writers.
fn write_image_atomically(path: &str, bytes: &[u8]) -> Result<(), Error> {
    if let Some(parent) = Path::new(path).parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent)?;
    }
    let tmp = format!("{path}.tmp");
    {
        let mut file = File::create(&tmp)?;
        file.write_all(bytes)?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

/// Reads image bytes from `path`, or `None` when no image file is present.
fn read_image_if_present(path: &str) -> Result<Option<Vec<u8>>, Error> {
    if !file_exists(path) {
        return Ok(None);
    }
    Ok(Some(fs::read(path)?))
}

/// Removes the image file at `path` if present. Idempotent.
fn remove_image_if_present(path: &str) -> Result<(), Error> {
    if file_exists(path) {
        fs::remove_file(path)?;
    }
    Ok(())
}

#[async_trait]
impl MazeStore for FileStore {
    /// Returns the cell-count ceiling enforced by this file store on
    /// create/update — see [`crate::MAX_MAZE_CELLS`].
    ///
    /// # Examples
    ///
    /// Read the cap from a fresh file store rooted at a temporary directory
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// use storage::{FileStore, FileStoreConfig, MazeStore};
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// assert_eq!(store.max_maze_cells(), Some(10_000));
    /// # });
    /// ```
    fn max_maze_cells(&self) -> Option<usize> {
        Some(MAX_MAZE_CELLS)
    }
    /// Returns the per-user maze cap enforced on create — see
    /// [`crate::MAX_MAZES_PER_USER`].
    fn max_mazes_per_user(&self) -> Option<usize> {
        Some(crate::MAX_MAZES_PER_USER)
    }
    /// Creates a new maze within the file store instance
    ///
    /// # Examples
    ///
    /// Try to create a new maze within a file store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_create = Maze::from_vec(grid);
    /// maze_to_create.name = "maze_1".to_string();
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Create maze within the file store
    /// match store.create_maze(&owner, &mut maze_to_create).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully created maze in the file store with id = {}",
    ///             maze_to_create.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to create maze => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn create_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error> {
        if maze.name.is_empty() {
            return Err(Error::MazeNameMissing());
        }
        validate_maze_cell_count(
            maze.definition.row_count(),
            maze.definition.col_count(),
            MAX_MAZE_CELLS,
        )?;
        validate_maze_feature_count(&maze.definition.grid, maze::MAX_TOTAL_FEATURES)?;
        validate_maze_object_counts(&maze.definition.grid)?;
        // Enforce the per-user maze cap before writing.
        let count = self.count_owner_mazes(owner);
        if count >= crate::MAX_MAZES_PER_USER {
            return Err(Error::MazeCountLimitReached { count, max: crate::MAX_MAZES_PER_USER });
        }
        // Reject case-insensitive name collision before writing — the
        // `write_maze_file` overwrite check uses `Path::exists`, which
        // is case-insensitive on NTFS/APFS but case-sensitive on ext4.
        // Without this guard, "Treasure" and "TREASURE" can both be
        // created on Linux but only one on Windows.
        if let Some(existing) = self.find_maze_filename_ci(owner, &maze.name) {
            return Err(Error::MazeIdExists(existing));
        }
        let id = self.make_maze_id(&maze.name);
        self.write_maze_file(owner, maze, &id, false)?;
        Ok(())
    }
    /// Deletes an existing maze from within the file store instance
    ///
    /// # Examples
    ///
    /// Try to delete an existing maze from within a file store
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Delete maze from within the file store
    /// let id = "maze_1.json".to_string();
    ///
    /// match store.delete_maze(&owner, &id).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully delete maze from the file store",
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to delete maze with id {} => {}",
    ///             id,
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn delete_maze(&mut self, owner: &User, id: &str) -> Result<(), Error> {
        if id.is_empty() {
            return Err(Error::MazeIdMissing());
        }
        if !self.maze_exists(owner, id) {
            return Err(Error::MazeIdNotFound(id.to_string()));
        }
        delete_file(&self.maze_path(owner, id));
        self.delete_score_rows(None, &[id.to_string()])?;
        Ok(())
    }
    /// Updates an existing maze within the file store instance
    ///
    /// # Examples
    ///
    /// Try to update an existing maze within a file store with new content
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_update = Maze::from_vec(grid);
    /// maze_to_update.name = "maze_1".to_string();
    /// maze_to_update.id = "maze_1".to_string();
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Update maze within the file store
    /// match store.update_maze(&owner, &mut maze_to_update).await {
    ///     Ok(_) => {
    ///         println!(
    ///             "Successfully updated maze in the file store with id = {}",
    ///             maze_to_update.id
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to update maze => {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn update_maze(&mut self, owner: &User, maze: &mut Maze) -> Result<(), Error> {
        if maze.id.is_empty() {
            return Err(Error::MazeIdMissing());
        }
        validate_maze_cell_count(
            maze.definition.row_count(),
            maze.definition.col_count(),
            MAX_MAZE_CELLS,
        )?;
        validate_maze_feature_count(&maze.definition.grid, maze::MAX_TOTAL_FEATURES)?;
        validate_maze_object_counts(&maze.definition.grid)?;
        if !self.maze_exists(owner, &maze.id) {
            return Err(Error::MazeIdNotFound(maze.id.to_string()));
        }
        self.write_maze_file(owner, maze, &maze.id.clone(), true)?;
        Ok(())
    }
    /// Loads a maze from within the file store instance
    ///
    /// # Returns
    ///
    /// The maze instance if successful
    ///
    /// # Examples
    ///
    /// Try to create and then reload a maze from within a file store and, if successful, print it
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{Maze, User};
    /// use maze::{MazePath, MazePrinter};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error,  UserStore};
    /// use utils::StdoutLinePrinter;
    /// use uuid::Uuid;
    ///
    /// let grid: Vec<Vec<char>> = vec![
    ///    vec!['S', ' ', 'W'],
    ///    vec![' ', 'F', 'W']
    /// ];
    /// let mut maze_to_create = Maze::from_vec(grid);
    /// maze_to_create.name = "maze_1".to_string();
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Create the maze within the store
    /// if let Err(error) = store.create_maze(&owner, &mut maze_to_create).await {
    ///     println!(
    ///         "Failed to create maze => {}",
    ///         error
    ///     );
    ///     return;
    /// }
    ///
    /// // Now reload the maze from the store
    /// match store.get_maze(&owner, &maze_to_create.id).await {
    ///     Ok(loaded_maze) => {
    ///         println!("Successfully loaded maze:");
    ///         let mut print_target = StdoutLinePrinter::new();
    ///         let empty_path = MazePath { points: vec![] };
    ///         loaded_maze.print(&mut print_target, empty_path);
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to load maze with id '{}' => {}",
    ///             maze_to_create.id,
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_maze(&self, owner: &User, id: &str) -> Result<Maze, Error> {
        if !self.maze_exists(owner, id) {
            return Err(Error::MazeIdNotFound(id.to_string()));
        }
        let path = self.maze_path(owner, id);
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        match serde_json::from_reader::<BufReader<File>, Maze>(reader) {
            Ok(mut maze) => {
                maze.id = id.to_string();
                Ok(maze)
            }
            Err(error) => Err(Error::from(error)),
        }
    }
    /// Locates a maze item by name from within the file store instance
    ///
    /// # Returns
    ///
    /// The maze item if successful
    ///
    /// # Examples
    ///
    /// Try to find the maze item with name `my_maze` from within a file store and, if successful,
    /// print its details
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// let id = "my_maze".to_string();
    ///
    /// // Attempt to find the maze item
    /// match store.find_maze_by_name(&owner, &id).await {
    ///     Ok(maze_item) => {
    ///         println!("Successfully found maze item => id = {}, name = {}",
    ///             maze_item.id,
    ///             maze_item.name
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to find maze item with id '{}' => {}",
    ///             id,
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn find_maze_by_name(&self, owner: &User, name: &str) -> Result<MazeItem, Error> {
        // Case-insensitive lookup, implemented in code rather than via
        // filesystem semantics — see `find_maze_filename_ci` for rationale.
        match self.find_maze_filename_ci(owner, name) {
            Some(id) => Ok(MazeItem {
                id,
                name: name.to_string(),
                definition: None,
            }),
            None => Err(Error::MazeNameNotFound(name.to_string())),
        }
    }
    /// Returns the list of maze items within the file store instance, sorted
    /// alphabetically in ascending order, optionally including the
    /// maze definitions as a JSON string
    ///
    /// # Returns
    ///
    /// The maze items if successful
    ///
    /// # Examples
    ///
    /// Try to load the maze items within a file store and, if successful,
    /// print the number of items found
    ///
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, MazeStore, Store, Error, UserStore};
    /// use uuid::Uuid;
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Locate the owner by username
    /// let find_user_result: Result<User, Error> = store.find_user_by_name("a_username").await;
    /// let owner = match find_user_result {
    ///    Ok(user) => user,
    ///    Err(error) => {
    ///        println!("Error fetching user: {:?}", error);
    ///        return ;
    ///    }
    /// };
    ///
    /// // Attempt to load the maze items along with their definitions
    /// match store.get_maze_items(&owner, true).await {
    ///     Ok(maze_items) => {
    ///         println!("Successfully loaded {} maze items",
    ///             maze_items.len()
    ///         );
    ///     }
    ///     Err(error) => {
    ///         println!(
    ///             "Failed to load maze items=> {}",
    ///             error
    ///         );
    ///     }
    /// }
    /// # });
    /// ```
    async fn get_maze_items(
        &self,
        owner: &User,
        include_definitions: bool,
    ) -> Result<Vec<MazeItem>, Error> {
        let mut items: Vec<MazeItem> = Vec::new();
        let mazes_dir = self.get_mazes_dir(owner);

        let mut paths: Vec<_> = fs::read_dir(mazes_dir)?
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<_, std::io::Error>>()?;

        paths.sort();

        for path in paths {
            let Some(path_str) = path.to_str() else { continue };
            let Some(extension) = path.extension() else { continue };
            if extension != "json" {
                continue;
            }
            let Some(name) = path.file_stem() else { continue };
            let Some(name_str) = name.to_str() else { continue };

            let mut name_use = name_str.to_string();
            let mut definition: Option<String> = None;
            if let Ok(maze_loaded) = self.get_maze(owner, path_str).await {
                if include_definitions {
                    definition = Some(
                        serde_json::to_string(&maze_loaded)
                            .expect("Failed to serialize"),
                    );
                }
                if !maze_loaded.name.is_empty() {
                    name_use = maze_loaded.name.to_string();
                }
            }

            items.push(MazeItem {
                id: path_str.to_string(),
                name: name_use,
                definition,
            });
        }
        Ok(items)
    }
}

#[async_trait]
impl Manage for FileStore {
    /// Resets the file store to its initial empty state by deleting the
    /// entire data directory and re-creating the empty user-tree skeleton.
    ///
    /// # Returns
    ///
    /// `Ok(())` on success, `Err(...)` if the data directory could not be
    /// removed or re-initialised.
    ///
    /// # Examples
    ///
    /// Empty the store before running a test scenario
    /// ```
    /// # tokio_test::block_on(async {
    ///
    /// use storage::{FileStore, FileStoreConfig, Manage, Store};
    ///
    /// // Create the file store
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    ///
    /// // Wipe any existing content
    /// if let Err(error) = store.empty().await {
    ///     panic!("Failed to empty the store: {}", error);
    /// }
    /// # });
    /// ```
    async fn empty(&mut self) -> Result<(), Error> {
        let root_path = Path::new(&self.data_dir);
        if root_path.is_dir()
            && let Err(error) = fs::remove_dir_all(root_path)
        {
            return Err(Error::Other(format!(
                "Failed to delete root data directory: {} - {}",
                self.data_dir, error
            )));
        }
        if let Err(error) = self.init() {
            return Err(Error::Other(format!(
                "Failed to reinitialize FileStore: {error}"
            )));
        }
        Ok(())
    }
}

#[async_trait]
impl TokenStore for FileStore {
    /// Persists a one-time token. The caller is responsible for assigning
    /// the `id` and timestamps — typically via [`OneTimeToken::new`].
    /// Rejects with [`Error::TokenIdExists`] on a duplicate id.
    ///
    /// # Examples
    ///
    /// Issue a password-reset token for a freshly-created user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{OneTimeToken, TokenPurpose, User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, TokenStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// let token = OneTimeToken::new(user.id, TokenPurpose::PasswordReset, None, 1);
    /// store.create_token(&token).await.expect("create_token");
    /// # });
    /// ```
    async fn create_token(&mut self, token: &OneTimeToken) -> Result<(), Error> {
        if token.id.is_nil() {
            return Err(Error::Other("token id must not be nil".to_string()));
        }
        if token.user_id.is_nil() {
            return Err(Error::UserIdMissing());
        }
        // Ensure the directory exists; it normally does (created by
        // migration 0005) but covers the edge case where empty() ran
        // between init and create.
        if !dir_exists(&self.tokens_dir) {
            fs::create_dir_all(&self.tokens_dir)?;
        }
        self.write_token_file(token, false)
    }

    /// Loads an active (non-expired, non-consumed) token by id. Returns
    /// `Err(TokenIdNotFound)` for unknown ids and for tokens past their
    /// `expires_at`.
    ///
    /// # Examples
    ///
    /// Round-trip a token through `create_token` + `find_token`
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{OneTimeToken, TokenPurpose, User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, TokenStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// let token = OneTimeToken::new(user.id, TokenPurpose::PasswordReset, None, 1);
    /// store.create_token(&token).await.expect("create_token");
    /// let loaded = store.find_token(token.id).await.expect("find_token");
    /// assert_eq!(loaded.user_id, user.id);
    /// # });
    /// ```
    async fn find_token(&self, id: Uuid) -> Result<OneTimeToken, Error> {
        let token = self.read_token_raw(id)?;
        if token.is_expired() {
            return Err(Error::TokenIdNotFound(id.to_string()));
        }
        Ok(token)
    }

    /// Marks the token consumed and returns the consumed row. A second
    /// call against the same id surfaces `Err(TokenAlreadyConsumed)`.
    /// Expired tokens fail with `Err(TokenExpired)`.
    ///
    /// # Examples
    ///
    /// Single-use enforcement: the second consume call fails
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{OneTimeToken, TokenPurpose, User, UserEmail};
    /// use storage::{Error, FileStore, FileStoreConfig, Store, TokenStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// let token = OneTimeToken::new(user.id, TokenPurpose::PasswordReset, None, 1);
    /// store.create_token(&token).await.expect("create_token");
    /// store.consume_token(token.id).await.expect("first consume");
    /// assert!(matches!(
    ///     store.consume_token(token.id).await,
    ///     Err(Error::TokenAlreadyConsumed())
    /// ));
    /// # });
    /// ```
    async fn consume_token(&mut self, id: Uuid) -> Result<OneTimeToken, Error> {
        // FileStore consumption is read-modify-write: read the file,
        // reject if already consumed or expired, populate `consumed_at`,
        // and atomically rewrite via tempfile + rename. The race window
        // between read and rename is small in practice and is accepted
        // for FileStore (single-process dev / small-scale deployments).
        // SqlStore is the correct answer where consume races matter.
        let mut token = self.read_token_raw(id)?;
        if token.is_consumed() {
            return Err(Error::TokenAlreadyConsumed());
        }
        if token.is_expired() {
            return Err(Error::TokenExpired());
        }
        token.consumed_at = Some(generate_now_millis());
        self.write_token_file(&token, true)?;
        Ok(token)
    }

    /// Removes every outstanding [`data_model::TokenPurpose::EmailVerification`]
    /// token belonging to `user_id` whose `target_email` matches the
    /// supplied address (case-insensitive). Used by the verification
    /// re-send handler so re-issuing supersedes prior tokens.
    ///
    /// # Examples
    ///
    /// Two verification tokens issued for the same address — purging
    /// removes both
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{OneTimeToken, TokenPurpose, User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, Store, TokenStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".into(),
    ///     full_name: "Alice".into(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "hash".into(), api_key: Uuid::nil(),
    ///     logins: vec![], oauth_identities: vec![], deleted_at: None,
    ///     created_at: chrono::Utc::now(), last_sign_in_at: None,
    ///     avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.expect("create_user");
    /// for _ in 0..2 {
    ///     let t = OneTimeToken::new(
    ///         user.id, TokenPurpose::EmailVerification,
    ///         Some("alice@example.com".into()), 24,
    ///     );
    ///     store.create_token(&t).await.expect("create_token");
    /// }
    /// let purged = store
    ///     .purge_email_verification_tokens(user.id, "alice@example.com")
    ///     .await
    ///     .expect("purge");
    /// assert_eq!(purged, 2);
    /// # });
    /// ```
    async fn purge_email_verification_tokens(
        &mut self,
        user_id: Uuid,
        target_email: &str,
    ) -> Result<u64, Error> {
        let mut purged: u64 = 0;
        for id in self.get_token_ids()? {
            match self.read_token_raw(id) {
                Ok(token)
                    if token.user_id == user_id
                        && token.purpose == data_model::TokenPurpose::EmailVerification
                        && token
                            .target_email
                            .as_deref()
                            .map(|t| t.eq_ignore_ascii_case(target_email))
                            .unwrap_or(false) =>
                {
                    delete_file(&self.token_file_path(id));
                    purged += 1;
                }
                Ok(_) => {}
                Err(Error::TokenIdNotFound(_)) => {}
                Err(error) => {
                    log::warn!(
                        "FileStore purge_email_verification_tokens: skipping unreadable token '{id}' - {error}"
                    );
                }
            }
        }
        Ok(purged)
    }

    /// Removes every token whose `expires_at` is in the past AND that
    /// has not been consumed. Returns the number of rows deleted.
    /// Intended as a periodic housekeeping sweep.
    ///
    /// # Examples
    ///
    /// Purging a fresh store is a no-op
    /// ```
    /// # tokio_test::block_on(async {
    /// use storage::{FileStore, FileStoreConfig, Store, TokenStore};
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// assert_eq!(store.purge_expired().await.expect("purge"), 0);
    /// # });
    /// ```
    async fn purge_expired(&mut self) -> Result<u64, Error> {
        let mut purged: u64 = 0;
        for id in self.get_token_ids()? {
            match self.read_token_raw(id) {
                Ok(token) if token.is_expired() && !token.is_consumed() => {
                    delete_file(&self.token_file_path(id));
                    purged += 1;
                }
                Ok(_) => {}
                Err(Error::TokenIdNotFound(_)) => {}
                Err(error) => {
                    log::warn!(
                        "FileStore purge_expired: skipping unreadable token '{id}' - {error}"
                    );
                }
            }
        }
        Ok(purged)
    }
}

#[async_trait]
impl EmailAuditLog for FileStore {
    /// Inserts a new audit row synchronously, before the actual send is
    /// attempted. Caller builds the entry via
    /// [`EmailAuditEntry::new_pending`]; this method just persists it.
    /// Returns the assigned id on success.
    ///
    /// # Examples
    ///
    /// Record a pending password-reset send
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::EmailAuditEntry;
    /// use storage::{EmailAuditLog, FileStore, FileStoreConfig, Store};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let entry = EmailAuditEntry::new_pending(
    ///     Some(Uuid::new_v4()), "alice@example.com", "password_reset",
    ///     None, None, "stub",
    /// );
    /// let id = store.record_pending(&entry).await.expect("record_pending");
    /// assert_eq!(id, entry.id);
    /// # });
    /// ```
    async fn record_pending(&mut self, entry: &EmailAuditEntry) -> Result<Uuid, Error> {
        if entry.id.is_nil() {
            return Err(Error::Other("audit entry id must not be nil".to_string()));
        }
        if !dir_exists(&self.audit_log_dir) {
            fs::create_dir_all(&self.audit_log_dir)?;
        }
        let mut to_write = entry.clone();
        to_write.error_message = to_write
            .error_message
            .as_deref()
            .map(truncate_email_audit_error_message);
        self.write_audit_entry_file(&to_write, false)?;
        Ok(entry.id)
    }

    /// Flips a previously-recorded `pending` row to `accepted` (with
    /// `provider_message_id`) or `failed` (with `error_class` and
    /// `error_message`). Once written, an audit row only moves forward —
    /// passing `AuditOutcome::Pending` is rejected.
    ///
    /// # Examples
    ///
    /// Mark the audit row as accepted after the provider responds
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{AuditOutcome, EmailAuditEntry};
    /// use storage::{EmailAuditLog, FileStore, FileStoreConfig, Store};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let entry = EmailAuditEntry::new_pending(
    ///     Some(Uuid::new_v4()), "alice@example.com", "password_reset",
    ///     None, None, "stub",
    /// );
    /// store.record_pending(&entry).await.expect("record_pending");
    /// store
    ///     .update_outcome(entry.id, AuditOutcome::Accepted, Some("provider-123"), None, None)
    ///     .await
    ///     .expect("update_outcome");
    /// # });
    /// ```
    async fn update_outcome(
        &mut self,
        id: Uuid,
        outcome: AuditOutcome,
        provider_message_id: Option<&str>,
        error_class: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<(), Error> {
        if matches!(outcome, AuditOutcome::Pending) {
            return Err(Error::Other(
                "update_outcome cannot move a row back to pending".to_string(),
            ));
        }
        let mut entry = self.read_audit_entry_raw(id)?;
        entry.outcome = outcome;
        entry.provider_message_id = provider_message_id.map(|s| s.to_string());
        entry.error_class = error_class.map(|s| s.to_string());
        entry.error_message = error_message.map(truncate_email_audit_error_message);
        self.write_audit_entry_file(&entry, true)
    }

    /// Loads a single audit row by id. Returns
    /// `Err(AuditEntryIdNotFound)` for unknown ids.
    ///
    /// # Examples
    ///
    /// Load back a recorded row
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::EmailAuditEntry;
    /// use storage::{EmailAuditLog, FileStore, FileStoreConfig, Store};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let entry = EmailAuditEntry::new_pending(
    ///     Some(Uuid::new_v4()), "alice@example.com", "password_reset",
    ///     None, None, "stub",
    /// );
    /// store.record_pending(&entry).await.expect("record_pending");
    /// let loaded = store.find_audit_entry(entry.id).await.expect("find");
    /// assert_eq!(loaded.recipient_email, "alice@example.com");
    /// # });
    /// ```
    async fn find_audit_entry(&self, id: Uuid) -> Result<EmailAuditEntry, Error> {
        self.read_audit_entry_raw(id)
    }

    /// Returns the `limit` most recent audit rows for a user,
    /// `recipient_user_id = user_id`, sorted by `created_at` descending
    /// with id as a deterministic tie-breaker.
    ///
    /// # Examples
    ///
    /// Read back the most recent two audit entries for a user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::EmailAuditEntry;
    /// use storage::{EmailAuditLog, FileStore, FileStoreConfig, Store};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let user_id = Uuid::new_v4();
    /// for template in ["password_reset", "email_verification"] {
    ///     let e = EmailAuditEntry::new_pending(
    ///         Some(user_id), "alice@example.com", template,
    ///         None, None, "stub",
    ///     );
    ///     store.record_pending(&e).await.expect("record_pending");
    /// }
    /// let recent = store
    ///     .find_recent_audit_entries_for_user(user_id, 5)
    ///     .await
    ///     .expect("find_recent");
    /// assert_eq!(recent.len(), 2);
    /// # });
    /// ```
    async fn find_recent_audit_entries_for_user(
        &self,
        user_id: Uuid,
        limit: u32,
    ) -> Result<Vec<EmailAuditEntry>, Error> {
        let mut matches: Vec<EmailAuditEntry> = Vec::new();
        for entry_id in self.get_audit_entry_ids()? {
            match self.read_audit_entry_raw(entry_id) {
                Ok(entry) if entry.recipient_user_id == Some(user_id) => matches.push(entry),
                Ok(_) => {}
                Err(Error::AuditEntryIdNotFound(_)) => {}
                Err(error) => {
                    log::warn!(
                        "FileStore find_recent_audit_entries_for_user: skipping unreadable entry '{entry_id}' - {error}"
                    );
                }
            }
        }
        // Sort by created_at descending; tie-break on id for determinism
        // when multiple rows land in the same millisecond bucket.
        matches.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.id.cmp(&a.id))
        });
        matches.truncate(limit as usize);
        Ok(matches)
    }
}

/// Comparator mirroring the SqlStore `ORDER BY`: the primary metric in the
/// requested direction, then the other metric in its fixed best direction
/// (score DESC / elapsed_ms ASC), then `recorded_at` ASC and `id` ASC.
fn score_cmp(ordering: ScoreOrdering, a: &ScoreEntry, b: &ScoreEntry) -> std::cmp::Ordering {
    let primary = match ordering.metric {
        ScoreMetric::Time => a.elapsed_ms.cmp(&b.elapsed_ms),
        ScoreMetric::Score => a.score.cmp(&b.score),
    };
    let primary = match ordering.direction {
        SortDirection::Ascending => primary,
        SortDirection::Descending => primary.reverse(),
    };
    let secondary = match ordering.metric {
        ScoreMetric::Time => b.score.cmp(&a.score), // score DESC
        ScoreMetric::Score => a.elapsed_ms.cmp(&b.elapsed_ms), // elapsed_ms ASC
    };
    primary
        .then(secondary)
        .then(a.recorded_at.cmp(&b.recorded_at))
        .then(a.id.cmp(&b.id))
}

/// Filters, sorts by `ordering`, and returns the `[offset .. offset+limit]`
/// page of a board.
fn paged_board(
    entries: Vec<ScoreEntry>,
    keep: impl Fn(&ScoreEntry) -> bool,
    ordering: ScoreOrdering,
    limit: u32,
    offset: u32,
) -> Vec<ScoreEntry> {
    let mut matched: Vec<ScoreEntry> = entries.into_iter().filter(|e| keep(e)).collect();
    matched.sort_by(|a, b| score_cmp(ordering, a, b));
    matched
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect()
}

impl FileStore {
    /// Wraps a board page into [`ScoreboardEntry`]s, resolving each row's
    /// player `username` and `avatar_updated_at` when `include_usernames` is
    /// set. Reads each distinct player once (`load_user_if_present` — `None`
    /// for an absent player) and caches both fields. Deleted players never
    /// reach here: the delete cascade removes their score rows first.
    fn attach_usernames(
        &self,
        page: Vec<ScoreEntry>,
        include_usernames: bool,
    ) -> Result<Vec<ScoreboardEntry>, Error> {
        if !include_usernames {
            return Ok(page
                .into_iter()
                .map(|entry| ScoreboardEntry {
                    entry,
                    username: None,
                    avatar_updated_at: None,
                })
                .collect());
        }
        // Cache the (username, avatar_updated_at) pair per player — both come
        // from the same player-file read, mirroring the SqlStore board JOIN.
        type PlayerFields = (Option<String>, Option<chrono::DateTime<chrono::Utc>>);
        let mut cache: HashMap<Uuid, PlayerFields> = HashMap::new();
        let mut out = Vec::with_capacity(page.len());
        for entry in page {
            let (username, avatar_updated_at) = match cache.get(&entry.user_id) {
                Some(fields) => fields.clone(),
                None => {
                    let fields = self
                        .load_user_if_present(entry.user_id)?
                        .map(|u| (Some(u.username), u.avatar_updated_at))
                        .unwrap_or((None, None));
                    cache.insert(entry.user_id, fields.clone());
                    fields
                }
            };
            out.push(ScoreboardEntry {
                entry,
                username,
                avatar_updated_at,
            });
        }
        Ok(out)
    }
}

#[async_trait]
impl ScoreStore for FileStore {
    /// Persists a completed run's score, returning its id.
    ///
    /// The entry's `id` must be non-nil and its subject valid; the run then
    /// appears on the matching board and in the player's history. See
    /// [`ScoreStore::record_score`].
    ///
    /// # Examples
    ///
    /// Record a curated-game run and read it back from the challenge board
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{
    ///     FileStore, FileStoreConfig, ScoreEntry, ScoreMetric, ScoreOrdering,
    ///     ScoreStore, SortDirection, UserStore,
    /// };
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".to_string(),
    ///     full_name: "Alice".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    ///
    /// let entry = ScoreEntry {
    ///     id: Uuid::new_v4(), user_id: user.id,
    ///     maze_id: None, challenge: Some("hard:42".to_string()),
    ///     score: 5, elapsed_ms: 83_456, recorded_at: chrono::Utc::now(),
    /// };
    /// store.record_score(&entry).await.unwrap();
    ///
    /// let highest = ScoreOrdering {
    ///     metric: ScoreMetric::Score,
    ///     direction: SortDirection::Descending,
    /// };
    /// let board = store
    ///     .challenge_leaderboard("hard:42", highest, 10, 0, true)
    ///     .await
    ///     .unwrap();
    /// assert_eq!(board.len(), 1);
    /// assert_eq!(board[0].entry.score, 5);
    /// # });
    /// ```
    async fn record_score(&mut self, entry: &ScoreEntry) -> Result<Uuid, Error> {
        if entry.id.is_nil() {
            return Err(Error::Other("score entry id must not be nil".to_string()));
        }
        crate::store::validate_score_subject(entry)?;
        self.write_score_entry_file(entry)?;
        Ok(entry.id)
    }

    /// A ranked, paged leaderboard for a stored maze. See
    /// [`ScoreStore::maze_leaderboard`].
    ///
    /// # Examples
    ///
    /// Record a maze run and read it back from the maze board
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{
    ///     FileStore, FileStoreConfig, ScoreEntry, ScoreMetric, ScoreOrdering,
    ///     ScoreStore, SortDirection, UserStore,
    /// };
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".to_string(),
    ///     full_name: "Alice".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    ///
    /// let entry = ScoreEntry {
    ///     id: Uuid::new_v4(), user_id: user.id,
    ///     maze_id: Some("Maze_1.json".to_string()), challenge: None,
    ///     score: 12, elapsed_ms: 40_000, recorded_at: chrono::Utc::now(),
    /// };
    /// store.record_score(&entry).await.unwrap();
    ///
    /// let fastest = ScoreOrdering {
    ///     metric: ScoreMetric::Time,
    ///     direction: SortDirection::Ascending,
    /// };
    /// let board = store
    ///     .maze_leaderboard("Maze_1.json", fastest, 10, 0, true)
    ///     .await
    ///     .unwrap();
    /// assert_eq!(board.len(), 1);
    /// assert_eq!(board[0].entry.elapsed_ms, 40_000);
    /// # });
    /// ```
    async fn maze_leaderboard(
        &self,
        maze_id: &str,
        ordering: ScoreOrdering,
        limit: u32,
        offset: u32,
        include_usernames: bool,
    ) -> Result<Vec<ScoreboardEntry>, Error> {
        let all = self.read_all_score_entries()?;
        let page = paged_board(
            all,
            |e| e.maze_id.as_deref() == Some(maze_id),
            ordering,
            limit,
            offset,
        );
        self.attach_usernames(page, include_usernames)
    }

    /// A ranked, paged leaderboard for a curated/shared challenge. See
    /// [`ScoreStore::challenge_leaderboard`].
    ///
    /// # Examples
    ///
    /// Record a challenge run and read it back from the challenge board
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{
    ///     FileStore, FileStoreConfig, ScoreEntry, ScoreMetric, ScoreOrdering,
    ///     ScoreStore, SortDirection, UserStore,
    /// };
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".to_string(),
    ///     full_name: "Alice".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    ///
    /// let entry = ScoreEntry {
    ///     id: Uuid::new_v4(), user_id: user.id,
    ///     maze_id: None, challenge: Some("hard:42".to_string()),
    ///     score: 5, elapsed_ms: 83_456, recorded_at: chrono::Utc::now(),
    /// };
    /// store.record_score(&entry).await.unwrap();
    ///
    /// let highest = ScoreOrdering {
    ///     metric: ScoreMetric::Score,
    ///     direction: SortDirection::Descending,
    /// };
    /// let board = store
    ///     .challenge_leaderboard("hard:42", highest, 10, 0, false)
    ///     .await
    ///     .unwrap();
    /// assert_eq!(board.len(), 1);
    /// assert_eq!(board[0].entry.score, 5);
    /// # });
    /// ```
    async fn challenge_leaderboard(
        &self,
        challenge: &str,
        ordering: ScoreOrdering,
        limit: u32,
        offset: u32,
        include_usernames: bool,
    ) -> Result<Vec<ScoreboardEntry>, Error> {
        let all = self.read_all_score_entries()?;
        let page = paged_board(
            all,
            |e| e.challenge.as_deref() == Some(challenge),
            ordering,
            limit,
            offset,
        );
        self.attach_usernames(page, include_usernames)
    }

    /// A page of a player's own runs, most recent first. See
    /// [`ScoreStore::user_history`].
    ///
    /// # Examples
    ///
    /// Record a run and read it back from the player's history
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, ScoreEntry, ScoreStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".to_string(),
    ///     full_name: "Alice".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    ///
    /// let entry = ScoreEntry {
    ///     id: Uuid::new_v4(), user_id: user.id,
    ///     maze_id: Some("Maze_1.json".to_string()), challenge: None,
    ///     score: 3, elapsed_ms: 12_000, recorded_at: chrono::Utc::now(),
    /// };
    /// store.record_score(&entry).await.unwrap();
    ///
    /// let history = store.user_history(user.id, 10, 0).await.unwrap();
    /// assert_eq!(history.len(), 1);
    /// assert_eq!(history[0].maze_id.as_deref(), Some("Maze_1.json"));
    /// # });
    /// ```
    async fn user_history(
        &self,
        user_id: Uuid,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ScoreEntry>, Error> {
        let mut matched: Vec<ScoreEntry> = self
            .read_all_score_entries()?
            .into_iter()
            .filter(|e| e.user_id == user_id)
            .collect();
        // Recent first: recorded_at DESC, id DESC.
        matched.sort_by(|a, b| b.recorded_at.cmp(&a.recorded_at).then(b.id.cmp(&a.id)));
        Ok(matched
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect())
    }

    /// Deletes every score for a stored maze, returning the number removed. See
    /// [`ScoreStore::clear_maze_scores`].
    ///
    /// # Examples
    ///
    /// Record a maze run, then clear that maze's board
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, ScoreEntry, ScoreStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".to_string(),
    ///     full_name: "Alice".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    /// let entry = ScoreEntry {
    ///     id: Uuid::new_v4(), user_id: user.id,
    ///     maze_id: Some("Maze_1.json".to_string()), challenge: None,
    ///     score: 3, elapsed_ms: 12_000, recorded_at: chrono::Utc::now(),
    /// };
    /// store.record_score(&entry).await.unwrap();
    ///
    /// let removed = store.clear_maze_scores("Maze_1.json").await.unwrap();
    /// assert_eq!(removed, 1);
    /// # });
    /// ```
    async fn clear_maze_scores(&mut self, maze_id: &str) -> Result<u64, Error> {
        self.delete_scores_matching(|e| e.maze_id.as_deref() == Some(maze_id))
    }

    /// Deletes every score for one curated/shared challenge. See
    /// [`ScoreStore::clear_challenge_scores`].
    ///
    /// # Examples
    ///
    /// Record a challenge run, then clear that challenge's board
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, ScoreEntry, ScoreStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".to_string(),
    ///     full_name: "Alice".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    /// let entry = ScoreEntry {
    ///     id: Uuid::new_v4(), user_id: user.id,
    ///     maze_id: None, challenge: Some("hard:42".to_string()),
    ///     score: 5, elapsed_ms: 83_456, recorded_at: chrono::Utc::now(),
    /// };
    /// store.record_score(&entry).await.unwrap();
    ///
    /// let removed = store.clear_challenge_scores("hard:42").await.unwrap();
    /// assert_eq!(removed, 1);
    /// # });
    /// ```
    async fn clear_challenge_scores(&mut self, challenge: &str) -> Result<u64, Error> {
        self.delete_scores_matching(|e| e.challenge.as_deref() == Some(challenge))
    }

    /// Deletes every score whose `challenge` matches a definition's prefix (all
    /// of its per-maze boards). See [`ScoreStore::clear_challenge_scores_prefix`].
    ///
    /// # Examples
    ///
    /// Clearing `def:abc` removes both its base board and its `def:abc:1` sub-board
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{User, UserEmail};
    /// use storage::{FileStore, FileStoreConfig, ScoreEntry, ScoreStore, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut user = User {
    ///     id: Uuid::nil(), is_admin: false, username: "alice".to_string(),
    ///     full_name: "Alice".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("alice@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut user).await.unwrap();
    /// for challenge in ["def:abc", "def:abc:1"] {
    ///     let entry = ScoreEntry {
    ///         id: Uuid::new_v4(), user_id: user.id,
    ///         maze_id: None, challenge: Some(challenge.to_string()),
    ///         score: 1, elapsed_ms: 1_000, recorded_at: chrono::Utc::now(),
    ///     };
    ///     store.record_score(&entry).await.unwrap();
    /// }
    ///
    /// let removed = store.clear_challenge_scores_prefix("def:abc").await.unwrap();
    /// assert_eq!(removed, 2);
    /// # });
    /// ```
    async fn clear_challenge_scores_prefix(&mut self, prefix: &str) -> Result<u64, Error> {
        let dated = format!("{prefix}:");
        self.delete_scores_matching(|e| {
            e.challenge
                .as_deref()
                .is_some_and(|c| c == prefix || c.starts_with(&dated))
        })
    }
}

#[async_trait]
impl GameStore for FileStore {
    fn max_definitions_per_user(&self) -> Option<usize> {
        Some(crate::MAX_DEFINITIONS_PER_USER)
    }

    fn max_collections_per_user(&self) -> Option<usize> {
        Some(crate::MAX_COLLECTIONS_PER_USER)
    }

    /// Stores a new definition for `owner`, assigning its id and timestamps in
    /// place.
    ///
    /// Rejects a blank name, an oversized config, a name that collides with one
    /// of the owner's existing definitions, or exceeding
    /// [`Self::max_definitions_per_user`]. See [`GameStore::create_game_definition`].
    ///
    /// # Examples
    ///
    /// Create a definition and read it back by id
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Tower".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Private,
    ///     seed: 7, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    /// assert!(!def.id.is_nil());
    ///
    /// let loaded = store.get_game_definition(def.id).await.unwrap();
    /// assert_eq!(loaded.name, "Tower");
    /// assert_eq!(loaded.owner_id, owner.id);
    /// # });
    /// ```
    async fn create_game_definition(
        &mut self,
        owner: &User,
        definition: &mut GameDefinition,
    ) -> Result<(), Error> {
        if definition.name.trim().is_empty() {
            return Err(Error::GameDefinitionNameMissing());
        }
        let config_json = serde_json::to_string(&definition.config)?;
        validate_game_definition_config_size(config_json.len(), MAX_GAME_DEFINITION_CONFIG_BYTES)?;
        if self
            .find_owner_definition_id_by_name(owner.id, &definition.name)?
            .is_some()
        {
            return Err(Error::GameDefinitionNameAlreadyExists(definition.name.clone()));
        }
        // Enforce the per-user definition cap.
        let count = self.read_all_game_definitions()?.iter().filter(|d| d.owner_id == owner.id).count();
        if count >= crate::MAX_DEFINITIONS_PER_USER {
            return Err(Error::GameDefinitionCountLimitReached { count, max: crate::MAX_DEFINITIONS_PER_USER });
        }
        definition.owner_id = owner.id;
        if definition.id.is_nil() {
            definition.id = Uuid::new_v4();
        }
        let now = generate_now_millis();
        definition.created_at = now;
        definition.updated_at = now;
        self.write_game_definition_file(definition, false)?;
        Ok(())
    }

    /// Loads any definition by id, or [`Error::GameDefinitionIdNotFound`]. See
    /// [`GameStore::get_game_definition`].
    ///
    /// # Examples
    ///
    /// Create a definition and read it back by id
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Tower".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Private,
    ///     seed: 7, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    ///
    /// let loaded = store.get_game_definition(def.id).await.unwrap();
    /// assert_eq!(loaded.name, "Tower");
    /// # });
    /// ```
    async fn get_game_definition(&self, id: Uuid) -> Result<GameDefinition, Error> {
        self.read_game_definition_raw(id)
    }

    /// Updates the owner's definition in place, preserving its id/owner/creation
    /// fields and refreshing `updated_at`. Rejects a blank name, oversized
    /// config, or a name colliding with another of the owner's definitions. See
    /// [`GameStore::update_game_definition`].
    ///
    /// # Examples
    ///
    /// Rename a definition and read the new name back
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Tower".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Private,
    ///     seed: 7, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    ///
    /// def.name = "Keep".to_string();
    /// store.update_game_definition(&owner, &mut def).await.unwrap();
    /// assert_eq!(store.get_game_definition(def.id).await.unwrap().name, "Keep");
    /// # });
    /// ```
    async fn update_game_definition(
        &mut self,
        owner: &User,
        definition: &mut GameDefinition,
    ) -> Result<(), Error> {
        let existing = self.read_game_definition_raw(definition.id)?;
        if existing.owner_id != owner.id {
            // Not the owner's definition — indistinguishable from absent.
            return Err(Error::GameDefinitionIdNotFound(definition.id.to_string()));
        }
        if definition.name.trim().is_empty() {
            return Err(Error::GameDefinitionNameMissing());
        }
        let config_json = serde_json::to_string(&definition.config)?;
        validate_game_definition_config_size(config_json.len(), MAX_GAME_DEFINITION_CONFIG_BYTES)?;
        if let Some(other) = self.find_owner_definition_id_by_name(owner.id, &definition.name)?
            && other != definition.id
        {
            return Err(Error::GameDefinitionNameAlreadyExists(definition.name.clone()));
        }
        // Preserve the immutable identity/creation fields; refresh updated_at.
        definition.owner_id = owner.id;
        definition.created_at = existing.created_at;
        definition.updated_at = generate_now_millis();
        self.write_game_definition_file(definition, true)?;
        Ok(())
    }

    /// Deletes the owner's definition, along with its shares and image. See
    /// [`GameStore::delete_game_definition`].
    ///
    /// # Examples
    ///
    /// Create then delete a definition; the id is gone afterwards
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Tower".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Private,
    ///     seed: 7, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    ///
    /// store.delete_game_definition(&owner, def.id).await.unwrap();
    /// assert!(store.get_game_definition(def.id).await.is_err());
    /// # });
    /// ```
    async fn delete_game_definition(&mut self, owner: &User, id: Uuid) -> Result<(), Error> {
        let existing = self.read_game_definition_raw(id)?;
        if existing.owner_id != owner.id {
            return Err(Error::GameDefinitionIdNotFound(id.to_string()));
        }
        delete_dir(&self.game_definition_dir_path(id));
        Ok(())
    }

    /// Grants `grantee` access to the owner's definition (idempotent). See
    /// [`GameStore::grant_game_definition_access`].
    ///
    /// # Examples
    ///
    /// Grant access, then confirm the grantee appears in the grantee list
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let grantee = Uuid::new_v4();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Tower".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Shared,
    ///     seed: 7, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    ///
    /// store.grant_game_definition_access(&owner, def.id, grantee).await.unwrap();
    /// assert!(store.get_game_definition_grantees(def.id).await.unwrap().contains(&grantee));
    /// # });
    /// ```
    async fn grant_game_definition_access(
        &mut self,
        owner: &User,
        id: Uuid,
        grantee: Uuid,
    ) -> Result<(), Error> {
        let existing = self.read_game_definition_raw(id)?;
        if existing.owner_id != owner.id {
            return Err(Error::GameDefinitionIdNotFound(id.to_string()));
        }
        let mut grantees = self.read_game_definition_grantees(id)?;
        if !grantees.contains(&grantee) {
            grantees.push(grantee);
            self.write_game_definition_grantees(id, &grantees)?;
        }
        Ok(())
    }

    /// Revokes `grantee`'s access to the owner's definition (idempotent). See
    /// [`GameStore::revoke_game_definition_access`].
    ///
    /// # Examples
    ///
    /// Grant then revoke; the grantee list is empty afterwards
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let grantee = Uuid::new_v4();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Tower".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Shared,
    ///     seed: 7, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    /// store.grant_game_definition_access(&owner, def.id, grantee).await.unwrap();
    ///
    /// store.revoke_game_definition_access(&owner, def.id, grantee).await.unwrap();
    /// assert!(store.get_game_definition_grantees(def.id).await.unwrap().is_empty());
    /// # });
    /// ```
    async fn revoke_game_definition_access(
        &mut self,
        owner: &User,
        id: Uuid,
        grantee: Uuid,
    ) -> Result<(), Error> {
        let existing = self.read_game_definition_raw(id)?;
        if existing.owner_id != owner.id {
            return Err(Error::GameDefinitionIdNotFound(id.to_string()));
        }
        let mut grantees = self.read_game_definition_grantees(id)?;
        let before = grantees.len();
        grantees.retain(|g| *g != grantee);
        if grantees.len() != before {
            self.write_game_definition_grantees(id, &grantees)?;
        }
        Ok(())
    }

    /// All of `owner`'s own definitions, sorted by name. See
    /// [`GameStore::get_game_definitions_for_owner`].
    ///
    /// # Examples
    ///
    /// Two definitions come back sorted by name
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// for name in ["Zeta", "Alpha"] {
    ///     let mut def = GameDefinition {
    ///         id: Uuid::nil(), owner_id: Uuid::nil(), name: name.to_string(),
    ///         description: None, image_updated_at: None, visibility: Visibility::Private,
    ///         seed: 1, rotation: Rotation::Static, config: serde_json::json!({}),
    ///         created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    ///     };
    ///     store.create_game_definition(&owner, &mut def).await.unwrap();
    /// }
    ///
    /// let names: Vec<String> = store
    ///     .get_game_definitions_for_owner(&owner).await.unwrap()
    ///     .into_iter().map(|d| d.name).collect();
    /// assert_eq!(names, vec!["Alpha", "Zeta"]);
    /// # });
    /// ```
    async fn get_game_definitions_for_owner(&self, owner: &User) -> Result<Vec<GameDefinition>, Error> {
        let mut defs: Vec<GameDefinition> = self
            .read_all_game_definitions()?
            .into_iter()
            .filter(|d| d.owner_id == owner.id)
            .collect();
        Self::sort_definitions_by_name(&mut defs);
        Ok(defs)
    }

    /// A page of the definitions `viewer` may see (owner ∨ curated/public ∨
    /// granted), ordered by name then id. See [`GameStore::get_visible_game_definitions`].
    ///
    /// # Examples
    ///
    /// A public definition is visible to another user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut viewer = User {
    ///     id: Uuid::nil(), is_admin: false, username: "viewer".to_string(),
    ///     full_name: "Viewer".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("viewer@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut viewer).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Open".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Public,
    ///     seed: 1, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    ///
    /// let visible = store.get_visible_game_definitions(&viewer, 10, 0).await.unwrap();
    /// assert_eq!(visible.iter().map(|d| d.name.as_str()).collect::<Vec<_>>(), vec!["Open"]);
    /// # });
    /// ```
    async fn get_visible_game_definitions(
        &self,
        viewer: &User,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<GameDefinition>, Error> {
        // Dev-only backend: evaluate the "visible to me" predicate in memory
        // (one file per id, so each definition is naturally distinct), sort, and
        // slice the page. The SqlStore does this as one paged predicate query.
        let mut defs: Vec<GameDefinition> = Vec::new();
        for def in self.read_all_game_definitions()? {
            let visible = def.owner_id == viewer.id
                || matches!(def.visibility, Visibility::Public | Visibility::Curated)
                || (def.visibility == Visibility::Shared
                    && self.read_game_definition_grantees(def.id)?.contains(&viewer.id));
            if visible {
                defs.push(def);
            }
        }
        defs.sort_by(|a, b| {
            UniCase::new(a.name.as_str()).cmp(&UniCase::new(b.name.as_str())).then(a.id.cmp(&b.id))
        });
        Ok(defs.into_iter().skip(offset as usize).take(limit as usize).collect())
    }

    /// The user ids currently granted access to a definition. See
    /// [`GameStore::get_game_definition_grantees`].
    ///
    /// # Examples
    ///
    /// A freshly-created definition has no grantees
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Tower".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Shared,
    ///     seed: 7, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    ///
    /// assert!(store.get_game_definition_grantees(def.id).await.unwrap().is_empty());
    /// # });
    /// ```
    async fn get_game_definition_grantees(&self, id: Uuid) -> Result<Vec<Uuid>, Error> {
        self.read_game_definition_grantees(id)
    }

    /// A definition's grantees resolved to `{id, username}`. See
    /// [`GameStore::get_game_definition_grantee_summaries`].
    ///
    /// # Examples
    ///
    /// Read back the resolved grantee list after a grant
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, GranteeSummary, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut friend = User {
    ///     id: Uuid::nil(), is_admin: false, username: "friend".to_string(),
    ///     full_name: "Friend".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("friend@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut friend).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Tower".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Shared,
    ///     seed: 7, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    /// store.grant_game_definition_access(&owner, def.id, friend.id).await.unwrap();
    ///
    /// let grantees = store.get_game_definition_grantee_summaries(def.id).await.unwrap();
    /// assert_eq!(grantees, vec![GranteeSummary { id: friend.id, username: "friend".into(), avatar_updated_at: None }]);
    /// # });
    /// ```
    async fn get_game_definition_grantee_summaries(
        &self,
        id: Uuid,
    ) -> Result<Vec<GranteeSummary>, Error> {
        let ids = self.read_game_definition_grantees(id)?;
        self.resolve_grantee_summaries(ids)
    }

    /// Stores (or replaces) a definition's image and stamps its
    /// `image_updated_at`, scoped to `owner`. See [`GameStore::set_game_definition_image`].
    ///
    /// # Examples
    ///
    /// Set, read back, then clear a definition's image
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "framer".to_string(),
    ///     full_name: "Framer".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("framer@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Framed".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Public,
    ///     seed: 1, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    ///
    /// store.set_game_definition_image(&owner, def.id, vec![1, 2, 3]).await.unwrap();
    /// assert_eq!(store.get_game_definition_image(def.id).await.unwrap(), Some(vec![1, 2, 3]));
    /// assert!(store.get_game_definition(def.id).await.unwrap().image_updated_at.is_some());
    ///
    /// store.clear_game_definition_image(&owner, def.id).await.unwrap();
    /// assert_eq!(store.get_game_definition_image(def.id).await.unwrap(), None);
    /// # });
    /// ```
    async fn set_game_definition_image(
        &mut self,
        owner: &User,
        id: Uuid,
        png_bytes: Vec<u8>,
    ) -> Result<(), Error> {
        let mut def = self.read_game_definition_raw(id)?;
        if def.owner_id != owner.id {
            return Err(Error::GameDefinitionIdNotFound(id.to_string()));
        }
        write_image_atomically(&self.game_definition_image_file_path(id), &png_bytes)?;
        def.image_updated_at = Some(generate_now_millis());
        self.write_game_definition_file(&def, true)?;
        Ok(())
    }

    /// Loads a definition's image bytes, or `None` when it has none. See
    /// [`GameStore::get_game_definition_image`].
    ///
    /// # Examples
    ///
    /// Set a definition image, then read the bytes back
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Framed".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Public,
    ///     seed: 1, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    ///
    /// store.set_game_definition_image(&owner, def.id, vec![1, 2, 3]).await.unwrap();
    /// assert_eq!(store.get_game_definition_image(def.id).await.unwrap(), Some(vec![1, 2, 3]));
    /// # });
    /// ```
    async fn get_game_definition_image(&self, id: Uuid) -> Result<Option<Vec<u8>>, Error> {
        read_image_if_present(&self.game_definition_image_file_path(id))
    }

    /// Removes a definition's image and clears its marker, scoped to `owner`
    /// (idempotent). See [`GameStore::clear_game_definition_image`].
    ///
    /// # Examples
    ///
    /// Set then clear a definition's image; the bytes are gone afterwards
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameDefinition, Rotation, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut def = GameDefinition {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Framed".to_string(),
    ///     description: None, image_updated_at: None, visibility: Visibility::Public,
    ///     seed: 1, rotation: Rotation::Static, config: serde_json::json!({}),
    ///     created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_definition(&owner, &mut def).await.unwrap();
    /// store.set_game_definition_image(&owner, def.id, vec![1, 2, 3]).await.unwrap();
    ///
    /// store.clear_game_definition_image(&owner, def.id).await.unwrap();
    /// assert_eq!(store.get_game_definition_image(def.id).await.unwrap(), None);
    /// # });
    /// ```
    async fn clear_game_definition_image(&mut self, owner: &User, id: Uuid) -> Result<(), Error> {
        // Idempotent + owner-scoped: an unknown or not-owned definition has
        // nothing for this owner to clear, so it is a successful no-op.
        let mut def = match self.read_game_definition_raw(id) {
            Ok(def) if def.owner_id == owner.id => def,
            Ok(_) | Err(Error::GameDefinitionIdNotFound(_)) => return Ok(()),
            Err(error) => return Err(error),
        };
        remove_image_if_present(&self.game_definition_image_file_path(id))?;
        if def.image_updated_at.is_some() {
            def.image_updated_at = None;
            self.write_game_definition_file(&def, true)?;
        }
        Ok(())
    }

    // ── Collections ──

    /// Stores a new collection for `owner`, assigning its id and timestamps in
    /// place.
    ///
    /// Rejects a blank name, a name that collides with one of the owner's
    /// existing collections, or exceeding [`Self::max_collections_per_user`]. See
    /// [`GameStore::create_game_collection`].
    ///
    /// # Examples
    ///
    /// Create a collection and read it back by id
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Private, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    /// assert!(!collection.id.is_nil());
    ///
    /// let loaded = store.get_game_collection(collection.id).await.unwrap();
    /// assert_eq!(loaded.name, "Campaign");
    /// assert_eq!(loaded.owner_id, owner.id);
    /// # });
    /// ```
    async fn create_game_collection(
        &mut self,
        owner: &User,
        collection: &mut GameCollection,
    ) -> Result<(), Error> {
        if collection.name.trim().is_empty() {
            return Err(Error::GameCollectionNameMissing());
        }
        if self
            .find_owner_collection_id_by_name(owner.id, &collection.name)?
            .is_some()
        {
            return Err(Error::GameCollectionNameAlreadyExists(collection.name.clone()));
        }
        // Enforce the per-user collection cap.
        let count = self.read_all_game_collections()?.iter().filter(|c| c.owner_id == owner.id).count();
        if count >= crate::MAX_COLLECTIONS_PER_USER {
            return Err(Error::GameCollectionCountLimitReached { count, max: crate::MAX_COLLECTIONS_PER_USER });
        }
        collection.owner_id = owner.id;
        if collection.id.is_nil() {
            collection.id = Uuid::new_v4();
        }
        let now = generate_now_millis();
        collection.created_at = now;
        collection.updated_at = now;
        normalize_item_order(&mut collection.items);
        self.write_game_collection_file(collection, false)?;
        Ok(())
    }

    /// Loads any collection by id, or [`Error::GameCollectionIdNotFound`]. See
    /// [`GameStore::get_game_collection`].
    ///
    /// # Examples
    ///
    /// Create a collection and read it back by id
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Private, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    ///
    /// let loaded = store.get_game_collection(collection.id).await.unwrap();
    /// assert_eq!(loaded.name, "Campaign");
    /// # });
    /// ```
    async fn get_game_collection(&self, id: Uuid) -> Result<GameCollection, Error> {
        self.read_game_collection_raw(id)
    }

    /// Updates a collection's metadata in place (membership is managed by the
    /// item methods), preserving its id/owner/creation fields. See
    /// [`GameStore::update_game_collection`].
    ///
    /// # Examples
    ///
    /// Rename a collection and read the new name back
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Private, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    ///
    /// collection.name = "Saga".to_string();
    /// store.update_game_collection(&owner, &mut collection).await.unwrap();
    /// assert_eq!(store.get_game_collection(collection.id).await.unwrap().name, "Saga");
    /// # });
    /// ```
    async fn update_game_collection(
        &mut self,
        owner: &User,
        collection: &mut GameCollection,
    ) -> Result<(), Error> {
        let existing = self.read_game_collection_raw(collection.id)?;
        if existing.owner_id != owner.id {
            return Err(Error::GameCollectionIdNotFound(collection.id.to_string()));
        }
        if collection.name.trim().is_empty() {
            return Err(Error::GameCollectionNameMissing());
        }
        if let Some(other) = self.find_owner_collection_id_by_name(owner.id, &collection.name)?
            && other != collection.id
        {
            return Err(Error::GameCollectionNameAlreadyExists(collection.name.clone()));
        }
        // Metadata-only update: items are managed by the item methods, so keep
        // the persisted membership regardless of what the caller passed.
        collection.owner_id = owner.id;
        collection.created_at = existing.created_at;
        collection.updated_at = generate_now_millis();
        collection.items = existing.items;
        self.write_game_collection_file(collection, true)?;
        Ok(())
    }

    /// Deletes the owner's collection, along with its shares and image. See
    /// [`GameStore::delete_game_collection`].
    ///
    /// # Examples
    ///
    /// Create then delete a collection; the id is gone afterwards
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Private, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    ///
    /// store.delete_game_collection(&owner, collection.id).await.unwrap();
    /// assert!(store.get_game_collection(collection.id).await.is_err());
    /// # });
    /// ```
    async fn delete_game_collection(&mut self, owner: &User, id: Uuid) -> Result<(), Error> {
        let existing = self.read_game_collection_raw(id)?;
        if existing.owner_id != owner.id {
            return Err(Error::GameCollectionIdNotFound(id.to_string()));
        }
        delete_dir(&self.game_collection_dir_path(id));
        Ok(())
    }

    /// Appends a definition to the end of the owner's collection (idempotent).
    /// See [`GameStore::add_game_collection_item`].
    ///
    /// # Examples
    ///
    /// Add an item and confirm it lands in the collection
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Private, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    ///
    /// let def_id = Uuid::new_v4();
    /// store.add_game_collection_item(&owner, collection.id, def_id).await.unwrap();
    /// let items = store.get_game_collection(collection.id).await.unwrap().items;
    /// assert_eq!(items.iter().map(|i| i.definition_id).collect::<Vec<_>>(), vec![def_id]);
    /// # });
    /// ```
    async fn add_game_collection_item(
        &mut self,
        owner: &User,
        collection_id: Uuid,
        definition_id: Uuid,
    ) -> Result<(), Error> {
        let mut collection = self.read_game_collection_raw(collection_id)?;
        if collection.owner_id != owner.id {
            return Err(Error::GameCollectionIdNotFound(collection_id.to_string()));
        }
        if collection.items.iter().any(|i| i.definition_id == definition_id) {
            return Ok(());
        }
        let sort_order = collection.items.len() as u32;
        collection.items.push(CollectionItem {
            definition_id,
            sort_order,
        });
        collection.updated_at = generate_now_millis();
        self.write_game_collection_file(&collection, true)?;
        Ok(())
    }

    /// Removes a definition from the owner's collection and closes the resulting
    /// order gap (idempotent). See [`GameStore::remove_game_collection_item`].
    ///
    /// # Examples
    ///
    /// Add then remove an item; the collection is empty afterwards
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Private, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    /// let def_id = Uuid::new_v4();
    /// store.add_game_collection_item(&owner, collection.id, def_id).await.unwrap();
    ///
    /// store.remove_game_collection_item(&owner, collection.id, def_id).await.unwrap();
    /// assert!(store.get_game_collection(collection.id).await.unwrap().items.is_empty());
    /// # });
    /// ```
    async fn remove_game_collection_item(
        &mut self,
        owner: &User,
        collection_id: Uuid,
        definition_id: Uuid,
    ) -> Result<(), Error> {
        let mut collection = self.read_game_collection_raw(collection_id)?;
        if collection.owner_id != owner.id {
            return Err(Error::GameCollectionIdNotFound(collection_id.to_string()));
        }
        let before = collection.items.len();
        collection.items.retain(|i| i.definition_id != definition_id);
        if collection.items.len() != before {
            normalize_item_order(&mut collection.items);
            collection.updated_at = generate_now_millis();
            self.write_game_collection_file(&collection, true)?;
        }
        Ok(())
    }

    /// Reorders the owner's collection so its items follow `ordered`. See
    /// [`GameStore::reorder_game_collection_items`].
    ///
    /// # Examples
    ///
    /// Add two items, reverse their order, and confirm the new sequence
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Private, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    /// let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    /// store.add_game_collection_item(&owner, collection.id, first).await.unwrap();
    /// store.add_game_collection_item(&owner, collection.id, second).await.unwrap();
    ///
    /// store.reorder_game_collection_items(&owner, collection.id, &[second, first]).await.unwrap();
    /// let order: Vec<Uuid> = store
    ///     .get_game_collection(collection.id).await.unwrap()
    ///     .items.iter().map(|i| i.definition_id).collect();
    /// assert_eq!(order, vec![second, first]);
    /// # });
    /// ```
    async fn reorder_game_collection_items(
        &mut self,
        owner: &User,
        collection_id: Uuid,
        ordered: &[Uuid],
    ) -> Result<(), Error> {
        let mut collection = self.read_game_collection_raw(collection_id)?;
        if collection.owner_id != owner.id {
            return Err(Error::GameCollectionIdNotFound(collection_id.to_string()));
        }
        collection.items = reordered_items(std::mem::take(&mut collection.items), ordered);
        collection.updated_at = generate_now_millis();
        self.write_game_collection_file(&collection, true)?;
        Ok(())
    }

    /// Grants `grantee` access to the owner's collection (idempotent). See
    /// [`GameStore::grant_game_collection_access`].
    ///
    /// # Examples
    ///
    /// Grant access, then confirm the grantee appears in the grantee list
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let grantee = Uuid::new_v4();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Shared, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    ///
    /// store.grant_game_collection_access(&owner, collection.id, grantee).await.unwrap();
    /// assert!(store.get_game_collection_grantees(collection.id).await.unwrap().contains(&grantee));
    /// # });
    /// ```
    async fn grant_game_collection_access(
        &mut self,
        owner: &User,
        id: Uuid,
        grantee: Uuid,
    ) -> Result<(), Error> {
        let existing = self.read_game_collection_raw(id)?;
        if existing.owner_id != owner.id {
            return Err(Error::GameCollectionIdNotFound(id.to_string()));
        }
        let mut grantees = self.read_game_collection_grantees(id)?;
        if !grantees.contains(&grantee) {
            grantees.push(grantee);
            self.write_game_collection_grantees(id, &grantees)?;
        }
        Ok(())
    }

    /// Revokes `grantee`'s access to the owner's collection (idempotent). See
    /// [`GameStore::revoke_game_collection_access`].
    ///
    /// # Examples
    ///
    /// Grant then revoke; the grantee list is empty afterwards
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let grantee = Uuid::new_v4();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Shared, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    /// store.grant_game_collection_access(&owner, collection.id, grantee).await.unwrap();
    ///
    /// store.revoke_game_collection_access(&owner, collection.id, grantee).await.unwrap();
    /// assert!(store.get_game_collection_grantees(collection.id).await.unwrap().is_empty());
    /// # });
    /// ```
    async fn revoke_game_collection_access(
        &mut self,
        owner: &User,
        id: Uuid,
        grantee: Uuid,
    ) -> Result<(), Error> {
        let existing = self.read_game_collection_raw(id)?;
        if existing.owner_id != owner.id {
            return Err(Error::GameCollectionIdNotFound(id.to_string()));
        }
        let mut grantees = self.read_game_collection_grantees(id)?;
        let before = grantees.len();
        grantees.retain(|g| *g != grantee);
        if grantees.len() != before {
            self.write_game_collection_grantees(id, &grantees)?;
        }
        Ok(())
    }

    /// All of `owner`'s own collections, sorted by name. See
    /// [`GameStore::get_game_collections_for_owner`].
    ///
    /// # Examples
    ///
    /// Two collections come back sorted by name
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// for name in ["Zeta", "Alpha"] {
    ///     let mut collection = GameCollection {
    ///         id: Uuid::nil(), owner_id: Uuid::nil(), name: name.to_string(),
    ///         visibility: Visibility::Private, description: None, image_updated_at: None,
    ///         items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    ///     };
    ///     store.create_game_collection(&owner, &mut collection).await.unwrap();
    /// }
    ///
    /// let names: Vec<String> = store
    ///     .get_game_collections_for_owner(&owner).await.unwrap()
    ///     .into_iter().map(|c| c.name).collect();
    /// assert_eq!(names, vec!["Alpha", "Zeta"]);
    /// # });
    /// ```
    async fn get_game_collections_for_owner(&self, owner: &User) -> Result<Vec<GameCollection>, Error> {
        let mut collections: Vec<GameCollection> = self
            .read_all_game_collections()?
            .into_iter()
            .filter(|c| c.owner_id == owner.id)
            .collect();
        Self::sort_collections_by_name(&mut collections);
        Ok(collections)
    }

    /// A page of the collections `viewer` may see, ordered by name then id — the
    /// collection counterpart of [`FileStore::get_visible_game_definitions`]. See
    /// [`GameStore::get_visible_game_collections`].
    ///
    /// # Examples
    ///
    /// A public collection is visible to another user
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut viewer = User {
    ///     id: Uuid::nil(), is_admin: false, username: "viewer".to_string(),
    ///     full_name: "Viewer".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("viewer@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut viewer).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Open".to_string(),
    ///     visibility: Visibility::Public, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    ///
    /// let visible = store.get_visible_game_collections(&viewer, 10, 0).await.unwrap();
    /// assert_eq!(visible.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), vec!["Open"]);
    /// # });
    /// ```
    async fn get_visible_game_collections(
        &self,
        viewer: &User,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<GameCollection>, Error> {
        let mut collections: Vec<GameCollection> = Vec::new();
        for collection in self.read_all_game_collections()? {
            let visible = collection.owner_id == viewer.id
                || matches!(collection.visibility, Visibility::Public | Visibility::Curated)
                || (collection.visibility == Visibility::Shared
                    && self.read_game_collection_grantees(collection.id)?.contains(&viewer.id));
            if visible {
                collections.push(collection);
            }
        }
        collections.sort_by(|a, b| {
            UniCase::new(a.name.as_str()).cmp(&UniCase::new(b.name.as_str())).then(a.id.cmp(&b.id))
        });
        Ok(collections.into_iter().skip(offset as usize).take(limit as usize).collect())
    }

    /// The user ids currently granted access to a collection. See
    /// [`GameStore::get_game_collection_grantees`].
    ///
    /// # Examples
    ///
    /// A freshly-created collection has no grantees
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Shared, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    ///
    /// assert!(store.get_game_collection_grantees(collection.id).await.unwrap().is_empty());
    /// # });
    /// ```
    async fn get_game_collection_grantees(&self, id: Uuid) -> Result<Vec<Uuid>, Error> {
        self.read_game_collection_grantees(id)
    }

    /// A collection's grantees resolved to `{id, username}`. See
    /// [`GameStore::get_game_collection_grantee_summaries`].
    ///
    /// # Examples
    ///
    /// Read back the resolved grantee list after a grant
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, GranteeSummary, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut friend = User {
    ///     id: Uuid::nil(), is_admin: false, username: "friend".to_string(),
    ///     full_name: "Friend".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("friend@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut friend).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Campaign".to_string(),
    ///     visibility: Visibility::Shared, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    /// store.grant_game_collection_access(&owner, collection.id, friend.id).await.unwrap();
    ///
    /// let grantees = store.get_game_collection_grantee_summaries(collection.id).await.unwrap();
    /// assert_eq!(grantees, vec![GranteeSummary { id: friend.id, username: "friend".into(), avatar_updated_at: None }]);
    /// # });
    /// ```
    async fn get_game_collection_grantee_summaries(
        &self,
        id: Uuid,
    ) -> Result<Vec<GranteeSummary>, Error> {
        let ids = self.read_game_collection_grantees(id)?;
        self.resolve_grantee_summaries(ids)
    }

    /// Stores (or replaces) a collection's image and stamps its
    /// `image_updated_at`, scoped to `owner`. See [`GameStore::set_game_collection_image`].
    ///
    /// # Examples
    ///
    /// Set, read back, then clear a collection's image
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "framer".to_string(),
    ///     full_name: "Framer".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("framer@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Framed".to_string(),
    ///     visibility: Visibility::Public, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    ///
    /// store.set_game_collection_image(&owner, collection.id, vec![9, 9]).await.unwrap();
    /// assert_eq!(store.get_game_collection_image(collection.id).await.unwrap(), Some(vec![9, 9]));
    ///
    /// store.clear_game_collection_image(&owner, collection.id).await.unwrap();
    /// assert_eq!(store.get_game_collection_image(collection.id).await.unwrap(), None);
    /// # });
    /// ```
    async fn set_game_collection_image(
        &mut self,
        owner: &User,
        id: Uuid,
        png_bytes: Vec<u8>,
    ) -> Result<(), Error> {
        let mut collection = self.read_game_collection_raw(id)?;
        if collection.owner_id != owner.id {
            return Err(Error::GameCollectionIdNotFound(id.to_string()));
        }
        write_image_atomically(&self.game_collection_image_file_path(id), &png_bytes)?;
        collection.image_updated_at = Some(generate_now_millis());
        self.write_game_collection_file(&collection, true)?;
        Ok(())
    }

    /// Loads a collection's image bytes, or `None` when it has none. See
    /// [`GameStore::get_game_collection_image`].
    ///
    /// # Examples
    ///
    /// Set a collection image, then read the bytes back
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Framed".to_string(),
    ///     visibility: Visibility::Public, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    ///
    /// store.set_game_collection_image(&owner, collection.id, vec![9, 9]).await.unwrap();
    /// assert_eq!(store.get_game_collection_image(collection.id).await.unwrap(), Some(vec![9, 9]));
    /// # });
    /// ```
    async fn get_game_collection_image(&self, id: Uuid) -> Result<Option<Vec<u8>>, Error> {
        read_image_if_present(&self.game_collection_image_file_path(id))
    }

    /// Removes a collection's image and clears its marker, scoped to `owner`
    /// (idempotent). See [`GameStore::clear_game_collection_image`].
    ///
    /// # Examples
    ///
    /// Set then clear a collection's image; the bytes are gone afterwards
    /// ```
    /// # tokio_test::block_on(async {
    /// use data_model::{GameCollection, User, UserEmail, Visibility};
    /// use storage::{FileStore, FileStoreConfig, GameStore, Store, UserStore};
    /// use uuid::Uuid;
    ///
    /// let temp = tempfile::tempdir().unwrap();
    /// let mut store = FileStore::new(&FileStoreConfig {
    ///     data_dir: temp.path().to_string_lossy().to_string(),
    /// });
    /// let mut owner = User {
    ///     id: Uuid::nil(), is_admin: false, username: "owner".to_string(),
    ///     full_name: "Owner".to_string(),
    ///     emails: vec![UserEmail::new_primary_verified("owner@example.com")],
    ///     password_hash: "h".to_string(), api_key: Uuid::nil(), logins: vec![],
    ///     oauth_identities: vec![], deleted_at: None, created_at: chrono::Utc::now(),
    ///     last_sign_in_at: None, avatar_updated_at: None,
    /// };
    /// store.create_user(&mut owner).await.unwrap();
    /// let mut collection = GameCollection {
    ///     id: Uuid::nil(), owner_id: Uuid::nil(), name: "Framed".to_string(),
    ///     visibility: Visibility::Public, description: None, image_updated_at: None,
    ///     items: vec![], created_at: chrono::Utc::now(), updated_at: chrono::Utc::now(),
    /// };
    /// store.create_game_collection(&owner, &mut collection).await.unwrap();
    /// store.set_game_collection_image(&owner, collection.id, vec![9, 9]).await.unwrap();
    ///
    /// store.clear_game_collection_image(&owner, collection.id).await.unwrap();
    /// assert_eq!(store.get_game_collection_image(collection.id).await.unwrap(), None);
    /// # });
    /// ```
    async fn clear_game_collection_image(&mut self, owner: &User, id: Uuid) -> Result<(), Error> {
        let mut collection = match self.read_game_collection_raw(id) {
            Ok(collection) if collection.owner_id == owner.id => collection,
            Ok(_) | Err(Error::GameCollectionIdNotFound(_)) => return Ok(()),
            Err(error) => return Err(error),
        };
        remove_image_if_present(&self.game_collection_image_file_path(id))?;
        if collection.image_updated_at.is_some() {
            collection.image_updated_at = None;
            self.write_game_collection_file(&collection, true)?;
        }
        Ok(())
    }
}

impl Store for FileStore {}

#[cfg(test)]
mod tests {
    use super::*;
    //****************************************************************
    // Utility functions
    //****************************************************************
    // Create a new, empty store rooted at a fresh temp directory. The
    // returned `TempDir` must outlive the `FileStore` (RAII deletes the
    // directory on drop), so callers bind both: `let (store, _temp) = ...`.
    // Per-test temp dirs make every test independent — no `--test-threads=1`
    // needed.
    async fn new_store() -> (FileStore, tempfile::TempDir) {
        let temp = tempfile::tempdir().expect("tempdir");
        let store = FileStore::new(&FileStoreConfig {
            data_dir: temp.path().to_string_lossy().to_string(),
        });
        (store, temp)
    }

    // Initialize a User struct
    fn init_test_user(
        is_admin: bool,
        username: &str,
        full_name: &str,
        email: &str,
        password_hash: &str,
    ) -> User {
        User {
            id: User::new_id(),
            is_admin,
            username: username.to_string(),
            full_name: full_name.to_string(),
            emails: vec![data_model::UserEmail::new_primary_verified(email)],
            password_hash: password_hash.to_string(),
            api_key: User::new_api_key(),
            logins: vec![],
            oauth_identities: vec![],
            deleted_at: None,
            created_at: chrono::Utc::now(),
            last_sign_in_at: None,
            avatar_updated_at: None,
        }
    }

    // Create a user in the file store
    async fn create_user(
        store: &mut FileStore,
        is_admin: bool,
        username: &str,
        full_name: &str,
        email: &str,
        password_hash: &str,
    ) -> User {
        let mut user = init_test_user(is_admin, username, full_name, email, password_hash);

        if let Err(error) = store.create_user(&mut user).await {
            panic!("{}", error);
        }
        user
    }

    // ── score_history smoke tests. The full cross-backend contract suite lives
    //    in tests/. ────────────────────────────────────────────────────────────

    fn challenge_score(user_id: Uuid, challenge: &str, score: u64, elapsed_ms: u64) -> ScoreEntry {
        ScoreEntry {
            id: Uuid::new_v4(),
            user_id,
            maze_id: None,
            challenge: Some(challenge.to_string()),
            score,
            elapsed_ms,
            recorded_at: chrono::Utc::now(),
        }
    }

    #[tokio::test]
    async fn score_challenge_leaderboard_orders_and_pages() {
        let (mut store, _t) = new_store().await;
        let user = create_user(&mut store, false, "alice", "Alice", "alice@example.com", "hash").await;
        store.record_score(&challenge_score(user.id, "hard:1", 10, 5000)).await.unwrap();
        store.record_score(&challenge_score(user.id, "hard:1", 2, 1000)).await.unwrap();
        store.record_score(&challenge_score(user.id, "hard:1", 6, 3000)).await.unwrap();

        let fastest = ScoreOrdering { metric: ScoreMetric::Time, direction: SortDirection::Ascending };
        let highest = ScoreOrdering { metric: ScoreMetric::Score, direction: SortDirection::Descending };

        let fast = store.challenge_leaderboard("hard:1", fastest, 10, 0, false).await.unwrap();
        assert_eq!(fast.iter().map(|e| e.entry.elapsed_ms).collect::<Vec<_>>(), vec![1000, 3000, 5000]);
        let high = store.challenge_leaderboard("hard:1", highest, 10, 0, false).await.unwrap();
        assert_eq!(high.iter().map(|e| e.entry.score).collect::<Vec<_>>(), vec![10, 6, 2]);
        assert!(high.iter().all(|e| e.username.is_none()));

        // Paging: limit 1, offset 1 of fastest → the middle (3000 ms) run.
        let page = store.challenge_leaderboard("hard:1", fastest, 1, 1, false).await.unwrap();
        assert_eq!(page.len(), 1);
        assert_eq!(page[0].entry.elapsed_ms, 3000);

        // include_usernames=true resolves the player's name.
        let named = store.challenge_leaderboard("hard:1", highest, 10, 0, true).await.unwrap();
        assert!(named.iter().all(|e| e.username.as_deref() == Some("alice")));
    }

    #[tokio::test]
    async fn score_record_enforces_subject_invariant() {
        let (mut store, _t) = new_store().await;
        let user = create_user(&mut store, false, "alice", "Alice", "alice@example.com", "hash").await;
        let mut both = challenge_score(user.id, "easy:1", 1, 100);
        both.maze_id = Some("m1".to_string()); // both subjects set → rejected
        assert!(store.record_score(&both).await.is_err());
        let mut neither = challenge_score(user.id, "easy:1", 1, 100);
        neither.challenge = None; // neither subject set → rejected
        assert!(store.record_score(&neither).await.is_err());
    }

    #[tokio::test]
    async fn score_delete_user_cascades_history() {
        let (mut store, _t) = new_store().await;
        let user = create_user(&mut store, false, "alice", "Alice", "alice@example.com", "hash").await;
        store.record_score(&challenge_score(user.id, "easy:1", 1, 100)).await.unwrap();
        assert_eq!(store.user_history(user.id, 10, 0).await.unwrap().len(), 1);
        store.delete_user(user.id).await.unwrap();
        assert_eq!(store.user_history(user.id, 10, 0).await.unwrap().len(), 0);
    }

    // Initialize a Maze struct
    fn init_test_maze(
        store: &FileStore,
        name: &str,
        set_id: bool,
        set_name: bool,
    ) -> (String, Maze) {
        #[rustfmt::skip]
        let grid: Vec<Vec<char>> = vec![
            vec!['S', ' ', 'W'],
            vec!['F', ' ', 'W']
        ];
        let mut maze = Maze::from_vec(grid);
        if set_name {
            maze.name = name.to_string();
        }
        let id = store.make_maze_id(name);
        if set_id {
            maze.id = id.clone();
        }
        (id, maze)
    }
    //****************************************************************
    // FileStore-specific tests
    //
    // The bulk of FileStore behaviour — user/maze CRUD, find/list, error
    // semantics — is now exercised through the backend-agnostic
    // `Store` trait contract suite in `tests/file_store_contract.rs`. Only
    // tests that depend on private FileStore symbols (`maze_path`,
    // `users_dir`, `write_maze_file`) or on filesystem-level edge cases
    // (orphaned user directories, pre-existing on-disk files) remain here.
    //****************************************************************

    // ─── Orphaned-directory recovery ──────────────────────────────────

    #[tokio::test]
    async fn get_users_skips_orphaned_user_directory() {
        let (mut store, _temp) = new_store().await;
        let _ = create_user(&mut store, false, "valid", "", "valid@company.com", "hash").await;
        let orphan_id = Uuid::new_v4();
        std::fs::create_dir_all(std::path::Path::new(&store.users_dir).join(orphan_id.to_string()))
            .expect("failed to create orphan directory");
        let users = store.get_users(u32::MAX, 0).await.expect("get_users should succeed despite orphaned directory");
        assert_eq!(users.len(), 1);
        assert_eq!(users[0].username, "valid");
    }

    #[tokio::test]
    async fn find_user_by_verified_email_skips_orphaned_user_directory() {
        let (mut store, _temp) = new_store().await;
        let _ = create_user(&mut store, false, "valid", "", "valid@company.com", "hash").await;
        let orphan_id = Uuid::new_v4();
        std::fs::create_dir_all(std::path::Path::new(&store.users_dir).join(orphan_id.to_string()))
            .expect("failed to create orphan directory");
        store.find_user_by_verified_email("valid@company.com").await.expect("find_user_by_verified_email should succeed despite orphaned directory");
    }

    #[tokio::test]
    async fn find_user_by_oauth_identity_skips_orphaned_user_directory() {
        use data_model::OAuthIdentity;
        let (mut store, _temp) = new_store().await;
        let mut alice = init_test_user(false, "valid", "", "valid@company.com", "hash");
        alice.oauth_identities.push(OAuthIdentity::new(
            "google".to_string(),
            "sub-1".to_string(),
            Some("valid@company.com".to_string()),
        ));
        store.create_user(&mut alice).await.expect("create user");
        let orphan_id = Uuid::new_v4();
        std::fs::create_dir_all(std::path::Path::new(&store.users_dir).join(orphan_id.to_string()))
            .expect("failed to create orphan directory");
        store.find_user_by_oauth_identity("google", "sub-1").await
            .expect("find_user_by_oauth_identity should succeed despite orphaned directory");
    }

    #[tokio::test]
    async fn find_user_by_login_id_skips_orphaned_user_directory() {
        let (mut store, _temp) = new_store().await;
        let mut user = init_test_user(false, "valid", "", "valid@company.com", "hash");
        let login = data_model::UserLogin::new(24, None, None);
        let login_id = login.id;
        user.logins.push(login);
        store.create_user(&mut user).await.expect("failed to create user");
        let orphan_id = Uuid::new_v4();
        std::fs::create_dir_all(std::path::Path::new(&store.users_dir).join(orphan_id.to_string()))
            .expect("failed to create orphan directory");
        store.find_user_by_login_id(login_id).await.expect("find_user_by_login_id should succeed despite orphaned directory");
    }

    // ─── Private `write_maze_file` overwrite-flag behaviour ──────────

    #[tokio::test]
    async fn can_save_maze_to_valid_file_path() {
        let (mut store, _temp) = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "test",
            "",
            "test@company.com",
            "password_hash",
        ).await;
        let (id, mut maze) = init_test_maze(&store, "maze", true, true);

        match store.write_maze_file(&owner, &mut maze, &id, true) {
            Ok(_) => {}
            Err(error) => panic!("Failed to save to file: {error}"),
        }
    }

    #[tokio::test]
    #[should_panic(expected = "A maze with id 'maze.json' already exists")]
    async fn cannot_save_maze_to_existing_file_path_if_overwrite_disabled() {
        let (mut store, _temp) = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "test",
            "",
            "test@company.com",
            "password_hash",
        ).await;
        let (id, mut maze) = init_test_maze(&store, "maze", true, true);
        let path = store.maze_path(&owner, &id);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.write_maze_file(&owner, &mut maze, &id, false) {
            Ok(_) => {
                panic!(
                    "Successfully saved to existing file: {path} despite overwrite being false"
                );
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    #[tokio::test]
    async fn can_save_maze_to_existing_file_path_if_overwrite_enabled() {
        let (mut store, _temp) = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "test",
            "",
            "test@company.com",
            "password_hash",
        ).await;
        let (id, mut maze) = init_test_maze(&store, "maze", false, true);
        let path = store.maze_path(&owner, &id);
        let mut _file = File::create(&path).expect("Failed to create file");

        match store.write_maze_file(&owner, &mut maze, &id, true) {
            Ok(_) => {}
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    // ─── Pre-existing on-disk maze file (orphan-file detection) ──────

    #[tokio::test]
    #[should_panic(expected = "A maze with id 'maze.json' already exists")]
    async fn cannot_create_maze_that_exists() {
        let (mut store, _temp) = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "test",
            "",
            "test@company.com",
            "password_hash",
        ).await;
        let (id, mut maze) = init_test_maze(&store, "maze", false, true);
        let path = store.maze_path(&owner, &id);
        let mut _file = File::create(&path).expect("Failed to create file");

        let result = store.create_maze(&owner, &mut maze).await;
        match result {
            Ok(_) => {
                panic!(
                    "Successfully created maze when file: {path} existed, when should not have"
                );
            }
            Err(error) => {
                panic!("{}", error);
            }
        }
    }

    // ─── max_maze_cells cap enforcement ──────────────────────────────

    fn make_sized_maze(name: &str, rows: usize, cols: usize) -> Maze {
        use data_model::MazeDefinition;
        let mut maze = Maze::new(MazeDefinition::new(rows, cols));
        maze.name = name.to_string();
        maze
    }

    #[tokio::test]
    async fn file_store_max_maze_cells_returns_cap() {
        let (store, _temp) = new_store().await;
        assert_eq!(store.max_maze_cells(), Some(MAX_MAZE_CELLS));
    }

    #[tokio::test]
    async fn file_store_create_maze_accepts_at_cap() {
        let (mut store, _temp) = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "owner",
            "",
            "owner@company.com",
            "hash",
        )
        .await;
        // 100 × 100 = 10,000 = MAX_MAZE_CELLS
        let mut maze = make_sized_maze("at-cap", 100, 100);
        store
            .create_maze(&owner, &mut maze)
            .await
            .expect("at-cap create succeeds");
        assert!(!maze.id.is_empty());
    }

    #[tokio::test]
    async fn file_store_create_maze_accepts_just_under_cap() {
        let (mut store, _temp) = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "owner",
            "",
            "owner@company.com",
            "hash",
        )
        .await;
        // 99 × 100 = 9,900 < 10,000
        let mut maze = make_sized_maze("under-cap", 99, 100);
        store
            .create_maze(&owner, &mut maze)
            .await
            .expect("under-cap create succeeds");
    }

    #[tokio::test]
    async fn file_store_create_maze_rejects_over_cap() {
        let (mut store, _temp) = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "owner",
            "",
            "owner@company.com",
            "hash",
        )
        .await;
        // 101 × 100 = 10,100 > 10,000
        let mut maze = make_sized_maze("over-cap", 101, 100);
        let err = store
            .create_maze(&owner, &mut maze)
            .await
            .expect_err("over-cap create should fail");
        match err {
            Error::MazeHasTooManyCells { rows, cols, max } => {
                assert_eq!(rows, 101);
                assert_eq!(cols, 100);
                assert_eq!(max, MAX_MAZE_CELLS);
            }
            other => panic!("expected MazeHasTooManyCells, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn file_store_update_maze_rejects_over_cap() {
        use data_model::MazeDefinition;
        let (mut store, _temp) = new_store().await;
        let owner = create_user(
            &mut store,
            false,
            "owner",
            "",
            "owner@company.com",
            "hash",
        )
        .await;
        // Seed at half cap, then try to update to over cap.
        let mut maze = make_sized_maze("resize-me", 50, 50);
        store
            .create_maze(&owner, &mut maze)
            .await
            .expect("seed create");
        maze.definition = MazeDefinition::new(120, 100); // 12,000 cells
        let err = store
            .update_maze(&owner, &mut maze)
            .await
            .expect_err("over-cap update should fail");
        match err {
            Error::MazeHasTooManyCells { rows, cols, max } => {
                assert_eq!(rows, 120);
                assert_eq!(cols, 100);
                assert_eq!(max, MAX_MAZE_CELLS);
            }
            other => panic!("expected MazeHasTooManyCells, got {other:?}"),
        }
    }
}
