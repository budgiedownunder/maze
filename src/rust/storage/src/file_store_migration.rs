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
/// Real FileStore migrations register at 3 and above (added in subsequent
/// steps).
const MIGRATIONS: &[(u32, MigrationFn)] = &[(1, no_op_migration), (2, no_op_migration)];

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
}
