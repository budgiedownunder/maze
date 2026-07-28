//! On-startup migrations for FileStore.
//!
//! Two pieces live here:
//!
//! 1. **`migrate_users_dir`** — a one-shot, idempotent rewrite of any
//!    legacy single-email `user.json` files into the multi-email shape.
//!    Runs unconditionally on every `FileStore::new()`. Old-shape files
//!    get rewritten and the original kept alongside as `user.json.bak`;
//!    new-shape files parse straight through and are left alone.
//!
//! 2. **Schema-versioned migration framework** — `apply_pending_migrations`
//!    reads `<data_dir>/.schema_version` (defaulting to `0` if absent),
//!    runs every registered migration with a higher version in order,
//!    and writes the new version after each successful migration so a
//!    failure mid-batch leaves the schema at the last successful step.
//!    The version-counter aligns with the SQL migration numbers
//!    (`0001_initial.sql` / `0002_user_emails.sql`); 0001 and 0002 are
//!    no-op entries in the FileStore registry to bring the version
//!    counter in step without re-doing work — existing deployments that
//!    have run `migrate_users_dir` are already in the post-0002 shape.
//!    Real FileStore migrations register at 0003 and above.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use chrono::Utc;
use data_model::{OAuthIdentity, User, UserEmail, UserLogin};
use serde::Deserialize;
use uuid::Uuid;

use crate::Error;

/// Discriminator over the two on-disk user.json shapes. `serde(untagged)`
/// tries the current shape first; on failure, falls back to the legacy
/// shape. We carry the `User` value in `New` (rather than a marker type)
/// because untagged deserialization needs the full type to reject anything
/// that isn't a well-formed new-shape `User` — otherwise a partial-match
/// loose deserializer would swallow legacy files.
#[derive(Deserialize)]
#[serde(untagged)]
enum UserOnDisk {
    /// Current shape (post 0002 migration). Field unused after the variant
    /// matches — we only need the discriminator.
    #[allow(dead_code)]
    New(User),
    /// Pre-migration shape. The single `email: String` field becomes a
    /// primary, verified `UserEmail` row when the file is rewritten.
    Old(LegacyUser),
}

/// Pre-migration `User` JSON layout. Mirrors the field set as it existed
/// before `email: String` became `emails: Vec<UserEmail>`.
#[derive(Deserialize)]
struct LegacyUser {
    id: Uuid,
    is_admin: bool,
    username: String,
    full_name: String,
    email: String,
    password_hash: String,
    api_key: Uuid,
    logins: Vec<UserLogin>,
    #[serde(default)]
    oauth_identities: Vec<OAuthIdentity>,
}

impl LegacyUser {
    fn into_user(self) -> User {
        let primary = UserEmail {
            email: self.email,
            is_primary: true,
            verified: true,
            verified_at: Some(Utc::now()),
        };
        User {
            id: self.id,
            is_admin: self.is_admin,
            username: self.username,
            full_name: self.full_name,
            emails: vec![primary],
            password_hash: self.password_hash,
            api_key: self.api_key,
            logins: self.logins,
            oauth_identities: self.oauth_identities,
            deleted_at: None,
            created_at: Utc::now(),
            last_sign_in_at: None,
            avatar_updated_at: None,
        }
    }
}

/// Walks the users directory, migrating any old-shape `user.json` files in
/// place. Files already in the new shape are left untouched.
///
/// Idempotent — safe to call on every startup.
pub fn migrate_users_dir(users_dir: &str) -> Result<(), Error> {
    let dir = Path::new(users_dir);
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // Only operate on directories whose name is a valid UUID — matches
        // FileStore's existing `get_user_ids` filtering and avoids touching
        // anything stray.
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if Uuid::parse_str(name).is_err() {
            continue;
        }
        let user_file = path.join("user.json");
        if !user_file.is_file() {
            continue;
        }
        migrate_user_file(&user_file)?;
    }
    Ok(())
}

/// Migrates a single `user.json` file in place if it is in the old shape.
/// Returns `Ok(())` whether or not a rewrite happened.
fn migrate_user_file(path: &Path) -> Result<(), Error> {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(error) => {
            log::warn!(
                "FileStore migration: skipping unreadable file {} - {}",
                path.display(),
                error
            );
            return Ok(());
        }
    };
    let parsed: UserOnDisk = match serde_json::from_str(&raw) {
        Ok(p) => p,
        Err(error) => {
            // Garbage on disk shouldn't crash startup; the regular load path
            // will surface the error when the user is actually accessed.
            log::warn!(
                "FileStore migration: skipping unparseable file {} - {}",
                path.display(),
                error
            );
            return Ok(());
        }
    };
    let legacy = match parsed {
        UserOnDisk::New(_) => return Ok(()), // already migrated
        UserOnDisk::Old(legacy) => legacy,
    };
    let new_user = legacy.into_user();
    let new_json = serde_json::to_string(&new_user)?;
    write_backup_then_rewrite(path, &raw, &new_json)?;
    log::info!(
        "FileStore migration: migrated {} to multi-email shape",
        path.display()
    );
    Ok(())
}

// ─────────────────────────── Schema-version framework ───────────────────────────

/// Filename of the schema-version marker inside `data_dir`.
const SCHEMA_VERSION_FILE: &str = ".schema_version";

/// Highest schema version the framework currently knows about. Used only
/// by tests today; bumping this is `MIGRATIONS`'s job — never edit it
/// by hand.
#[cfg(test)]
const CURRENT_SCHEMA_VERSION: u32 = max_registered_version(MIGRATIONS);

/// Per-migration runner signature. Each migration receives the data
/// directory and applies its transformation against on-disk state. Errors
/// halt the framework before the next version is written.
type MigrationFn = fn(&Path) -> Result<(), Error>;

/// Ordered registry of FileStore migrations. The framework runs every
/// migration whose version is greater than the value stored in
/// `<data_dir>/.schema_version`, in ascending order, writing the new
/// version after each one.
///
/// **Versions 1 and 2 are intentional no-ops.** They align the FileStore
/// counter with the SQL `0001_initial.sql` and `0002_user_emails.sql`
/// migrations already applied to existing deployments. Existing
/// deployments are already in the post-0002 shape (the predecessor's
/// `migrate_users_dir` ran on every startup); brand-new `data_dir`s have
/// no users to migrate. Either way the no-op entries cleanly bring the
/// version counter to 2 without doing redundant work.
///
/// **Version 4 is also a no-op.** The matching SQL migration adds a new
/// `users.deleted_at` column. The FileStore data shape is updated by the
/// `#[serde(default, skip_serializing_if = "Option::is_none")]` on the
/// new `User.deleted_at` field — existing `user.json` files round-trip
/// without rewriting. The framework entry exists to advance the version
/// counter in step with the SQL backend.
///
/// **Version 5** creates the `<data_dir>/one_time_tokens/` directory used
/// by the FileStore `TokenStore` impl (one file per token). Idempotent —
/// `fs::create_dir_all` is a no-op if the directory already exists.
///
/// **Version 6** creates the `<data_dir>/email_audit_log/` directory used
/// by the FileStore `EmailAuditLog` impl (one file per audit entry).
/// Idempotent.
///
/// **Version 7 is a no-op.** The matching SQL migration adds an
/// `error_message TEXT` column to `email_audit_log`. The FileStore data
/// shape is updated by the `#[serde(default, skip_serializing_if =
/// "Option::is_none")]` on the new `EmailAuditEntry.error_message`
/// field — existing audit-row JSON files round-trip without rewriting.
/// The framework entry exists to advance the version counter in step
/// with the SQL backend.
///
/// **Version 8** counterpart to `migrations/0008_user_timestamps.sql`.
/// `User.created_at` is non-nullable in the application's struct, so
/// every existing `users/<uuid>/user.json` must be rewritten to carry
/// the field. The migration captures the migration-run timestamp at
/// startup and writes it into `created_at` and `last_sign_in_at` for
/// any file that lacks them — mirrors the Rust-side
/// `backfill_user_timestamps_if_null` on the SQL side. Idempotent —
/// files already carrying both fields are left alone.
///
/// **Version 9** creates the `<data_dir>/score_history/` directory used
/// by the FileStore `ScoreStore` impl (one file per completed run).
/// Idempotent.
///
/// **Version 11** creates the `<data_dir>/game_definitions/` parent directory
/// used by the FileStore `GameStore` impl (each definition owns an `<id>/`
/// sub-folder, created lazily on write). Version 10 is skipped — the SQL
/// `0010_user_avatars` migration has no FileStore directory counterpart
/// (avatars ride each user's dir as `avatar.png`). Idempotent.
///
/// **Version 12** creates the `<data_dir>/game_collections/` parent directory
/// used by the FileStore `GameStore` collection impl (each collection owns an
/// `<id>/` sub-folder, created lazily on write). Idempotent.
///
/// **Version 13** is a no-op that aligns the counter with the SQL
/// `0013_featured_game_items.sql` migration. The FileStore keeps the featured
/// list in a single root file `featured_game_items.json`, created lazily on the
/// first feature — there is no directory to pre-create.
///
/// **Version 14 is a no-op.** The matching SQL migration adds a nullable
/// `play_mode` column to `game_collections`. The FileStore data shape is updated
/// by the `#[serde(default)]` on the new `GameCollection.play_mode` field —
/// existing `collection.json` files round-trip without rewriting (an absent value
/// loads as the default `Arcade`). The framework entry exists to advance the
/// version counter in step with the SQL backend.
const MIGRATIONS: &[(u32, MigrationFn)] = &[
    (1, no_op_migration),
    (2, no_op_migration),
    (3, migrate_0003_user_emails_verified_reset),
    (4, no_op_migration),
    (5, migrate_0005_create_one_time_tokens_dir),
    (6, migrate_0006_create_email_audit_log_dir),
    (7, no_op_migration),
    (8, migrate_0008_user_timestamps),
    (9, migrate_0009_create_score_history_dir),
    (11, migrate_0011_create_game_definitions_dir),
    (12, migrate_0012_create_game_collections_dir),
    (13, no_op_migration),
    (14, no_op_migration),
];

const fn max_registered_version(migrations: &[(u32, MigrationFn)]) -> u32 {
    let mut max = 0u32;
    let mut i = 0;
    while i < migrations.len() {
        let (v, _) = migrations[i];
        if v > max {
            max = v;
        }
        i += 1;
    }
    max
}

fn no_op_migration(_data_dir: &Path) -> Result<(), Error> {
    Ok(())
}

/// FileStore migration 0005 — counterpart to
/// `migrations/0005_one_time_tokens.sql`. The SQL side creates a table;
/// the FileStore side creates the per-token directory used by the
/// `TokenStore` impl. Idempotent — re-running on an existing directory
/// is a no-op.
fn migrate_0005_create_one_time_tokens_dir(data_dir: &Path) -> Result<(), Error> {
    let dir = data_dir.join("one_time_tokens");
    fs::create_dir_all(&dir)?;
    Ok(())
}

/// FileStore migration 0006 — counterpart to
/// `migrations/0006_email_audit_log.sql`. The SQL side creates a table;
/// the FileStore side creates the per-row directory used by the
/// `EmailAuditLog` impl. Idempotent.
fn migrate_0006_create_email_audit_log_dir(data_dir: &Path) -> Result<(), Error> {
    let dir = data_dir.join("email_audit_log");
    fs::create_dir_all(&dir)?;
    Ok(())
}

/// FileStore migration 0009 — counterpart to
/// `migrations/0009_score_history.sql`. The SQL side creates a table; the
/// FileStore side creates the per-row directory used by the `ScoreStore` impl.
/// Idempotent.
fn migrate_0009_create_score_history_dir(data_dir: &Path) -> Result<(), Error> {
    let dir = data_dir.join("score_history");
    fs::create_dir_all(&dir)?;
    Ok(())
}

/// FileStore migration 0011 — counterpart to
/// `migrations/0011_game_definitions.sql`. The SQL side creates the
/// `game_definitions` + `game_definition_shares` tables; the FileStore side
/// creates the `game_definitions/` parent directory. Each definition owns an
/// `<id>/` sub-folder (`definition.json` + optional `shares.json`/`image.png`),
/// created lazily on write. Version 10 is skipped — the SQL `0010_user_avatars`
/// migration has no FileStore directory counterpart (avatars ride each user's
/// dir as `avatar.png`). Idempotent.
fn migrate_0011_create_game_definitions_dir(data_dir: &Path) -> Result<(), Error> {
    fs::create_dir_all(data_dir.join("game_definitions"))?;
    Ok(())
}

/// FileStore migration 0012 — counterpart to
/// `migrations/0012_game_collections.sql`. The SQL side creates the
/// `game_collections` + `game_collection_items` + `game_collection_shares`
/// tables; the FileStore side creates the `game_collections/` parent directory.
/// Each collection owns an `<id>/` sub-folder (`collection.json` + optional
/// `shares.json` / `image.png`), created lazily on write. Idempotent.
fn migrate_0012_create_game_collections_dir(data_dir: &Path) -> Result<(), Error> {
    fs::create_dir_all(data_dir.join("game_collections"))?;
    Ok(())
}

/// FileStore migration 0008 — counterpart to
/// `migrations/0008_user_timestamps.sql` and
/// `sql_store::backfill_user_timestamps_if_null`. Walks every
/// `users/<uuid>/user.json` and writes the migration-run timestamp
/// (captured once at the top of this function) into `created_at` for
/// any file that lacks it. `last_sign_in_at` is set to the most recent
/// `logins[*].created_at` if the file has any logins (the most accurate
/// "when did this user last sign in" we can reconstruct); files without
/// logins keep `last_sign_in_at` absent so the welcome-banner trigger
/// fires correctly on their first actual sign-in. Idempotent — files
/// that already carry both fields are left alone.
fn migrate_0008_user_timestamps(data_dir: &Path) -> Result<(), Error> {
    let users_dir = data_dir.join("users");
    if !users_dir.is_dir() {
        return Ok(());
    }
    let now = Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
    for entry in fs::read_dir(&users_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if Uuid::parse_str(name).is_err() {
            continue;
        }
        let user_file = path.join("user.json");
        if !user_file.is_file() {
            continue;
        }
        migrate_0008_user_file(&user_file, &now)?;
    }
    Ok(())
}

fn migrate_0008_user_file(path: &Path, now_iso: &str) -> Result<(), Error> {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(error) => {
            log::warn!(
                "FileStore migration 0008: skipping unreadable file {} - {}",
                path.display(),
                error
            );
            return Ok(());
        }
    };
    let mut value: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(error) => {
            log::warn!(
                "FileStore migration 0008: skipping unparseable file {} - {}",
                path.display(),
                error
            );
            return Ok(());
        }
    };
    let serde_json::Value::Object(ref mut map) = value else {
        log::warn!(
            "FileStore migration 0008: skipping non-object file {}",
            path.display(),
        );
        return Ok(());
    };
    let mut mutated = false;
    if !map.contains_key("created_at") {
        map.insert(
            "created_at".to_string(),
            serde_json::Value::String(now_iso.to_string()),
        );
        mutated = true;
    }
    // `last_sign_in_at` is backfilled to the most recent
    // `logins[*].created_at` — the timestamp of the user's latest login,
    // which is the most accurate evidence we have of when they signed
    // in. Users with no logins keep `last_sign_in_at` absent so the
    // welcome-banner trigger `User::is_first_sign_in()` (=
    // `last_sign_in_at.is_none() && logins.is_empty()`) correctly fires
    // on their first actual sign-in. Each `created_at` is parsed as a
    // chrono `DateTime` rather than lex-compared, since the legacy
    // serializer may have written timestamps with mixed sub-second
    // precision and lex-order would diverge from chronological order
    // across a `.000Z` / `Z` boundary.
    let most_recent_login_iso = map
        .get("logins")
        .and_then(|v| v.as_array())
        .and_then(|arr| {
            arr.iter()
                .filter_map(|login| login.get("created_at")?.as_str())
                .filter_map(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .max()
                .map(|dt| dt.with_timezone(&chrono::Utc).to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        });
    if !map.contains_key("last_sign_in_at")
        && let Some(iso) = most_recent_login_iso
    {
        map.insert("last_sign_in_at".to_string(), serde_json::Value::String(iso));
        mutated = true;
    }
    if mutated {
        let new_json = serde_json::to_string(&value)?;
        rewrite_atomically(path, &new_json)?;
    }
    Ok(())
}


/// Run every pending migration in `MIGRATIONS` against `data_dir`.
///
/// Behaviour:
/// - `<data_dir>/.schema_version` missing → treat as 0 (run every migration).
/// - Each migration runs in order; after success its version is written
///   atomically so a subsequent failure doesn't lose progress already made.
/// - A migration that returns `Err` aborts the run; the error propagates
///   and `.schema_version` reflects the last migration that succeeded.
/// - `.schema_version` higher than the registry's max version is rejected
///   with a clear error rather than silently downgrading the counter.
pub fn apply_pending_migrations(data_dir: &str) -> Result<(), Error> {
    apply_migrations_to(data_dir, MIGRATIONS)
}

/// The schema version currently recorded for `data_dir` (`0` when
/// `.schema_version` is absent — a brand-new data dir). Read *before*
/// [`apply_pending_migrations`] to tell a fresh store from a reopened one.
pub fn current_schema_version(data_dir: &str) -> Result<u32, Error> {
    read_schema_version(Path::new(data_dir))
}

/// Internal worker: same contract as `apply_pending_migrations` but takes
/// the registry as a parameter so tests can inject custom migration sets
/// without mutating the const.
fn apply_migrations_to(
    data_dir: &str,
    migrations: &[(u32, MigrationFn)],
) -> Result<(), Error> {
    let path = Path::new(data_dir);
    let mut current = read_schema_version(path)?;
    let max = max_registered_version(migrations);
    if current > max {
        return Err(Error::Other(format!(
            "{SCHEMA_VERSION_FILE} ({current}) is higher than the migration runner's max version ({max}); refusing to downgrade"
        )));
    }
    for (version, runner) in migrations {
        if *version <= current {
            continue;
        }
        runner(path)?;
        write_schema_version(path, *version)?;
        current = *version;
        log::info!("FileStore migration {version} applied");
    }
    Ok(())
}

/// FileStore migration 0003 — counterpart to
/// `migrations/0003_user_emails_verified_reset.sql`. Walks every
/// `users/<uuid>/user.json` and, for non-admin users, sets
/// `verified = false, verified_at = None` on every email that is **not**
/// matched by an OAuth identity's `provider_email` for the same user.
///
/// Admin users are skipped wholesale (their emails stay verified).
/// OAuth-matched emails on non-admin users stay verified — the OAuth
/// provider has attested to the address.
///
/// Idempotent: re-running against already-flipped data is a no-op (admin
/// and OAuth-matched emails were untouched on the first pass; other
/// emails are already `verified = false` and stay so).
fn migrate_0003_user_emails_verified_reset(data_dir: &Path) -> Result<(), Error> {
    let users_dir = data_dir.join("users");
    if !users_dir.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&users_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if Uuid::parse_str(name).is_err() {
            continue;
        }
        let user_file = path.join("user.json");
        if !user_file.is_file() {
            continue;
        }
        migrate_0003_user_file(&user_file)?;
    }
    Ok(())
}

fn migrate_0003_user_file(path: &Path) -> Result<(), Error> {
    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(error) => {
            log::warn!(
                "FileStore migration 0003: skipping unreadable file {} - {}",
                path.display(),
                error
            );
            return Ok(());
        }
    };
    let mut user: User = match serde_json::from_str(&raw) {
        Ok(u) => u,
        Err(error) => {
            log::warn!(
                "FileStore migration 0003: skipping unparseable file {} - {}",
                path.display(),
                error
            );
            return Ok(());
        }
    };
    if user.is_admin {
        return Ok(());
    }
    let oauth_emails: std::collections::HashSet<String> = user
        .oauth_identities
        .iter()
        .filter_map(|oi| oi.provider_email.clone())
        .collect();
    let mut mutated = false;
    for email in &mut user.emails {
        if oauth_emails.contains(&email.email) {
            continue;
        }
        if email.verified || email.verified_at.is_some() {
            email.verified = false;
            email.verified_at = None;
            mutated = true;
        }
    }
    if mutated {
        let new_json = serde_json::to_string(&user)?;
        rewrite_atomically(path, &new_json)?;
    }
    Ok(())
}

/// Atomically rewrite `path` with `new_json` via tempfile + rename.
/// Used by per-version migrations that need to update `user.json` files
/// without leaving a partial write on disk if the process is killed.
fn rewrite_atomically(path: &Path, new_json: &str) -> Result<(), Error> {
    let parent = path.parent().ok_or_else(|| {
        Error::Other(format!(
            "user.json has no parent directory: {}",
            path.display()
        ))
    })?;
    let tmp = parent.join("user.json.tmp");
    {
        let mut tmp_file = fs::File::create(&tmp)?;
        tmp_file.write_all(new_json.as_bytes())?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

fn read_schema_version(data_dir: &Path) -> Result<u32, Error> {
    let path = data_dir.join(SCHEMA_VERSION_FILE);
    match fs::read_to_string(&path) {
        Ok(s) => s.trim().parse::<u32>().map_err(|e| {
            Error::Other(format!(
                "failed to parse {}: {e}",
                path.display()
            ))
        }),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(error) => Err(error.into()),
    }
}

fn write_schema_version(data_dir: &Path, version: u32) -> Result<(), Error> {
    let target = data_dir.join(SCHEMA_VERSION_FILE);
    let tmp = data_dir.join(".schema_version.tmp");
    {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(version.to_string().as_bytes())?;
    }
    fs::rename(&tmp, &target)?;
    Ok(())
}

/// Writes `original_json` to `<path>.bak`, then atomically rewrites `path`
/// with `new_json` via tempfile + rename.
fn write_backup_then_rewrite(
    path: &Path,
    original_json: &str,
    new_json: &str,
) -> Result<(), Error> {
    let backup: PathBuf = {
        let mut p = path.as_os_str().to_owned();
        p.push(".bak");
        PathBuf::from(p)
    };
    {
        let mut backup_file = fs::File::create(&backup)?;
        backup_file.write_all(original_json.as_bytes())?;
    }
    let parent = path.parent().ok_or_else(|| {
        Error::Other(format!("user.json has no parent directory: {}", path.display()))
    })?;
    let tmp = parent.join("user.json.tmp");
    {
        let mut tmp_file = fs::File::create(&tmp)?;
        tmp_file.write_all(new_json.as_bytes())?;
    }
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;

    fn write_legacy_user(dir: &Path) -> Uuid {
        let id = Uuid::new_v4();
        let user_dir = dir.join(id.to_string());
        std::fs::create_dir_all(&user_dir).expect("create user dir");
        let path = user_dir.join("user.json");
        let json = format!(
            r#"{{"id":"{id}","is_admin":false,"username":"alice","full_name":"Alice","email":"alice@example.com","password_hash":"hash","api_key":"{key}","logins":[],"oauth_identities":[]}}"#,
            id = id,
            key = Uuid::new_v4(),
        );
        let mut f = File::create(&path).expect("create user.json");
        f.write_all(json.as_bytes()).expect("write");
        id
    }

    #[test]
    fn migrates_old_shape_in_place_and_writes_bak() {
        let temp = tempfile::tempdir().expect("tempdir");
        let id = write_legacy_user(temp.path());

        migrate_users_dir(temp.path().to_str().unwrap()).expect("migrate");

        let user_path = temp.path().join(id.to_string()).join("user.json");
        let bak_path = temp.path().join(id.to_string()).join("user.json.bak");

        assert!(user_path.exists(), "user.json should still exist");
        assert!(bak_path.exists(), "user.json.bak should exist");

        let new_json = std::fs::read_to_string(&user_path).expect("read user.json");
        let new_user: User = serde_json::from_str(&new_json).expect("parse new shape");
        assert_eq!(new_user.emails.len(), 1);
        assert_eq!(new_user.emails[0].email, "alice@example.com");
        assert!(new_user.emails[0].is_primary);
        assert!(new_user.emails[0].verified);

        let bak_json = std::fs::read_to_string(&bak_path).expect("read .bak");
        assert!(bak_json.contains("\"email\":\"alice@example.com\""));
        assert!(!bak_json.contains("\"emails\""));
    }

    #[test]
    fn migration_is_idempotent_on_new_shape_files() {
        let temp = tempfile::tempdir().expect("tempdir");
        let id = write_legacy_user(temp.path());

        // First run migrates.
        migrate_users_dir(temp.path().to_str().unwrap()).expect("first migrate");
        let bak_path = temp.path().join(id.to_string()).join("user.json.bak");
        let bak_first = std::fs::read_to_string(&bak_path).expect("first bak");

        // Delete the .bak so we can detect whether the second run rewrote anything.
        std::fs::remove_file(&bak_path).expect("remove bak");

        // Second run should not rewrite (file is already in the new shape).
        migrate_users_dir(temp.path().to_str().unwrap()).expect("second migrate");
        assert!(
            !bak_path.exists(),
            "second run must not rewrite an already-migrated file"
        );
        // Sanity: original .bak content was the legacy shape.
        assert!(bak_first.contains("\"email\":\"alice@example.com\""));
    }

    #[test]
    fn ignores_non_user_directories_and_non_uuid_names() {
        let temp = tempfile::tempdir().expect("tempdir");
        let stray = temp.path().join("not-a-uuid");
        std::fs::create_dir_all(&stray).expect("create stray");
        let stray_user = stray.join("user.json");
        std::fs::write(&stray_user, "garbage").expect("write garbage");

        // Must not error and must not touch the stray file.
        migrate_users_dir(temp.path().to_str().unwrap()).expect("migrate");

        let still_garbage = std::fs::read_to_string(&stray_user).expect("read");
        assert_eq!(still_garbage, "garbage");
    }

    #[test]
    fn handles_missing_directory() {
        let temp = tempfile::tempdir().expect("tempdir");
        let missing = temp.path().join("missing");
        // Must not error on a non-existent directory.
        migrate_users_dir(missing.to_str().unwrap()).expect("migrate");
    }

    // ───────────────────── Schema-version framework tests ─────────────────────

    fn schema_version_path(data_dir: &Path) -> PathBuf {
        data_dir.join(SCHEMA_VERSION_FILE)
    }

    fn read_version_file(data_dir: &Path) -> String {
        std::fs::read_to_string(schema_version_path(data_dir))
            .expect("read .schema_version")
            .trim()
            .to_string()
    }

    #[test]
    fn apply_pending_migrations_writes_current_version_on_empty_data_dir() {
        let temp = tempfile::tempdir().expect("tempdir");
        apply_pending_migrations(temp.path().to_str().unwrap())
            .expect("migrations must succeed");
        assert!(schema_version_path(temp.path()).exists());
        assert_eq!(
            read_version_file(temp.path()),
            CURRENT_SCHEMA_VERSION.to_string()
        );
    }

    #[test]
    fn apply_pending_migrations_is_idempotent_when_at_current_version() {
        let temp = tempfile::tempdir().expect("tempdir");
        // First run brings the counter to the current version.
        apply_pending_migrations(temp.path().to_str().unwrap()).expect("first run");
        let mtime_before = std::fs::metadata(schema_version_path(temp.path()))
            .expect("metadata")
            .modified()
            .expect("modified time");

        // Second run should detect current >= max and do nothing — neither
        // run a migration nor rewrite the version file.
        apply_pending_migrations(temp.path().to_str().unwrap())
            .expect("second run must be a no-op");

        let mtime_after = std::fs::metadata(schema_version_path(temp.path()))
            .expect("metadata")
            .modified()
            .expect("modified time");
        assert_eq!(
            read_version_file(temp.path()),
            CURRENT_SCHEMA_VERSION.to_string()
        );
        assert_eq!(
            mtime_before, mtime_after,
            ".schema_version must not be rewritten when no migrations are pending"
        );
    }

    #[test]
    fn rejects_schema_version_higher_than_runners_max() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(schema_version_path(temp.path()), "999")
            .expect("seed schema version");
        let err = apply_pending_migrations(temp.path().to_str().unwrap())
            .expect_err("must reject downgrade");
        let msg = format!("{err}");
        assert!(
            msg.contains("999"),
            "error should name the on-disk version: {msg}"
        );
        assert!(
            msg.contains("downgrade") || msg.contains("higher"),
            "error should explain the refusal: {msg}"
        );
    }

    /// Test-only registry containing a deliberately failing migration to
    /// verify per-step version-write semantics: migrations 1 and 2 succeed
    /// (writing the version to 2 along the way), migration 3 fails, and
    /// the schema version stays at 2 — the last successful step.
    #[test]
    fn failing_migration_leaves_schema_at_last_successful_version() {
        fn ok(_: &Path) -> Result<(), Error> {
            Ok(())
        }
        fn fail(_: &Path) -> Result<(), Error> {
            Err(Error::Other("synthetic failure".into()))
        }
        let migrations: &[(u32, MigrationFn)] = &[(1, ok), (2, ok), (3, fail), (4, ok)];

        let temp = tempfile::tempdir().expect("tempdir");
        let err = apply_migrations_to(temp.path().to_str().unwrap(), migrations)
            .expect_err("must surface migration failure");
        assert!(format!("{err}").contains("synthetic failure"), "{err}");
        // After the failure, the schema version reflects the highest
        // migration that actually succeeded — 2 here.
        assert_eq!(read_version_file(temp.path()), "2");
    }

    #[test]
    fn skips_migrations_already_recorded_at_or_below_current_version() {
        fn ok(_: &Path) -> Result<(), Error> {
            Ok(())
        }
        fn must_not_run(_: &Path) -> Result<(), Error> {
            panic!("migration ran when current version should have skipped it");
        }
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(schema_version_path(temp.path()), "5")
            .expect("seed schema version");
        // Versions 1..=5 must be skipped because current = 5; 6 must run.
        let migrations: &[(u32, MigrationFn)] =
            &[(1, must_not_run), (5, must_not_run), (6, ok)];
        apply_migrations_to(temp.path().to_str().unwrap(), migrations)
            .expect("migrations must succeed");
        assert_eq!(read_version_file(temp.path()), "6");
    }

    // ───────────────────── 0003 verified-reset migration ─────────────────────

    use chrono::TimeZone;

    fn write_user_json(data_dir: &Path, user: &User) {
        let user_dir = data_dir.join("users").join(user.id.to_string());
        std::fs::create_dir_all(&user_dir).expect("create user dir");
        let json = serde_json::to_string(user).expect("serialize user");
        std::fs::write(user_dir.join("user.json"), json).expect("write user.json");
    }

    fn read_user_json(data_dir: &Path, id: Uuid) -> User {
        let path = data_dir
            .join("users")
            .join(id.to_string())
            .join("user.json");
        let raw = std::fs::read_to_string(&path).expect("read user.json");
        serde_json::from_str(&raw).expect("parse user.json")
    }

    fn make_user(id: Uuid, username: &str, is_admin: bool, emails: Vec<UserEmail>) -> User {
        User {
            id,
            is_admin,
            username: username.into(),
            full_name: username.into(),
            emails,
            password_hash: "hash".into(),
            api_key: Uuid::new_v4(),
            logins: vec![],
            oauth_identities: vec![],
            deleted_at: None,
            created_at: Utc::now(),
            last_sign_in_at: None,
            avatar_updated_at: None,
        }
    }

    fn verified_email(email: &str, primary: bool) -> UserEmail {
        UserEmail {
            email: email.into(),
            is_primary: primary,
            verified: true,
            verified_at: Some(Utc.with_ymd_and_hms(2026, 1, 1, 0, 0, 0).unwrap()),
        }
    }

    /// Pre/post snapshot test: 1 admin + 1 OAuth-verified + 2 plain users.
    /// After migration: admin and OAuth-verified emails stay verified;
    /// the other two are flipped to verified = false / verified_at = None.
    #[test]
    fn migrate_0003_resets_verified_except_admin_and_oauth_matched() {
        let temp = tempfile::tempdir().expect("tempdir");
        let data_dir = temp.path();

        let admin_id = Uuid::new_v4();
        let admin = make_user(
            admin_id,
            "admin",
            true,
            vec![verified_email("admin@example.com", true)],
        );

        let oauth_id = Uuid::new_v4();
        let mut oauth_user = make_user(
            oauth_id,
            "oauth_alice",
            false,
            vec![verified_email("alice@gmail.com", true)],
        );
        oauth_user.oauth_identities.push(OAuthIdentity::new(
            "google".into(),
            "google-sub-1".into(),
            Some("alice@gmail.com".into()),
        ));

        let plain1_id = Uuid::new_v4();
        let plain1 = make_user(
            plain1_id,
            "bob",
            false,
            vec![verified_email("bob@example.com", true)],
        );

        let plain2_id = Uuid::new_v4();
        let plain2 = make_user(
            plain2_id,
            "carol",
            false,
            vec![verified_email("carol@example.com", true)],
        );

        write_user_json(data_dir, &admin);
        write_user_json(data_dir, &oauth_user);
        write_user_json(data_dir, &plain1);
        write_user_json(data_dir, &plain2);

        migrate_0003_user_emails_verified_reset(data_dir).expect("migration must succeed");

        // Admin email — untouched.
        let admin_after = read_user_json(data_dir, admin_id);
        assert!(admin_after.emails[0].verified);
        assert!(admin_after.emails[0].verified_at.is_some());

        // OAuth-matched email — untouched.
        let oauth_after = read_user_json(data_dir, oauth_id);
        assert!(oauth_after.emails[0].verified);
        assert!(oauth_after.emails[0].verified_at.is_some());

        // Plain users — flipped.
        for id in [plain1_id, plain2_id] {
            let after = read_user_json(data_dir, id);
            assert!(
                !after.emails[0].verified,
                "plain user email should be flipped to verified=false"
            );
            assert!(
                after.emails[0].verified_at.is_none(),
                "plain user verified_at should be cleared"
            );
        }
    }

    /// Idempotency: re-running the migration on already-migrated data
    /// produces the same result. Verifies that the second invocation
    /// doesn't accidentally re-flip already-flipped rows or re-clear
    /// already-cleared timestamps in a destructive way.
    #[test]
    fn migrate_0003_is_idempotent_when_re_run() {
        let temp = tempfile::tempdir().expect("tempdir");
        let data_dir = temp.path();

        let plain_id = Uuid::new_v4();
        let plain = make_user(
            plain_id,
            "dave",
            false,
            vec![verified_email("dave@example.com", true)],
        );
        write_user_json(data_dir, &plain);

        migrate_0003_user_emails_verified_reset(data_dir).expect("first run");
        let after_first = read_user_json(data_dir, plain_id);
        assert!(!after_first.emails[0].verified);
        assert!(after_first.emails[0].verified_at.is_none());

        // Capture the user.json mtime before the second run.
        let user_path = data_dir.join("users").join(plain_id.to_string()).join("user.json");
        let mtime_before = std::fs::metadata(&user_path)
            .expect("metadata")
            .modified()
            .expect("modified time");

        migrate_0003_user_emails_verified_reset(data_dir).expect("second run");
        let after_second = read_user_json(data_dir, plain_id);
        assert!(!after_second.emails[0].verified);
        assert!(after_second.emails[0].verified_at.is_none());

        // The second run shouldn't have rewritten the file — values were
        // already correct, so `mutated` stayed false.
        let mtime_after = std::fs::metadata(&user_path)
            .expect("metadata")
            .modified()
            .expect("modified time");
        assert_eq!(
            mtime_before, mtime_after,
            "idempotent re-run should not rewrite an already-flipped user.json"
        );
    }

    /// Empty users directory → migration succeeds without error.
    #[test]
    fn migrate_0003_empty_users_dir_is_no_op() {
        let temp = tempfile::tempdir().expect("tempdir");
        // Don't create users/ at all.
        migrate_0003_user_emails_verified_reset(temp.path())
            .expect("must succeed on missing users dir");

        // Also verify "users dir exists but is empty" path.
        std::fs::create_dir_all(temp.path().join("users")).expect("mkdir");
        migrate_0003_user_emails_verified_reset(temp.path())
            .expect("must succeed on empty users dir");
    }

    /// User with multiple emails: only the OAuth-matched one stays verified;
    /// other emails are flipped, even on the same user.
    #[test]
    fn migrate_0003_per_email_carve_out_within_one_user() {
        let temp = tempfile::tempdir().expect("tempdir");
        let data_dir = temp.path();

        let id = Uuid::new_v4();
        let mut user = make_user(
            id,
            "multi",
            false,
            vec![
                verified_email("primary@gmail.com", true),
                verified_email("secondary@example.com", false),
                verified_email("tertiary@example.com", false),
            ],
        );
        // OAuth provider knows about primary@gmail.com; the other two are
        // self-asserted plain emails.
        user.oauth_identities.push(OAuthIdentity::new(
            "google".into(),
            "sub-multi".into(),
            Some("primary@gmail.com".into()),
        ));
        write_user_json(data_dir, &user);

        migrate_0003_user_emails_verified_reset(data_dir).expect("migration must succeed");

        let after = read_user_json(data_dir, id);
        // primary@gmail.com — OAuth-matched, stays verified.
        assert!(after.emails[0].verified);
        assert!(after.emails[0].verified_at.is_some());
        // The other two — flipped.
        for i in 1..=2 {
            assert!(
                !after.emails[i].verified,
                "non-OAuth email at index {i} should be flipped"
            );
            assert!(after.emails[i].verified_at.is_none());
        }
    }
}
