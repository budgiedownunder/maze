//! On-startup migration for FileStore `user.json` files.
//!
//! Old-shape JSON carried `email: String`. The current shape carries
//! `emails: Vec<UserEmail>` per the multi-email feature. `FileStore::new()`
//! walks the users directory, rewrites any old-shape file in place to the
//! new shape, and saves the original alongside as `user.json.bak`. The
//! conversion is idempotent — running against already-migrated data
//! is a no-op (every file parses straight as the new shape).

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
}
