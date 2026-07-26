use crate::db::{self, DbState};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::State;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub format: String,
    pub version: u32,
    pub product: String,
    #[serde(default)]
    pub app_version: Option<String>,
    pub created_at: String,
    pub database_sha256: String,
}

fn hash(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn validate_gradia_extension(path: &Path) -> Result<(), String> {
    let valid = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("gradia"));
    if valid {
        Ok(())
    } else {
        Err("Gradia database transfers must use the .gradia file extension.".into())
    }
}

pub fn create_backup(database_path: &Path, output_path: &Path) -> Result<BackupManifest, String> {
    validate_gradia_extension(output_path)?;
    if !database_path.exists() {
        return Err("Gradia database does not exist yet.".into());
    }
    let connection = db::open(database_path)?;
    connection
        .execute_batch("PRAGMA wal_checkpoint(FULL);")
        .map_err(|e| e.to_string())?;
    drop(connection);
    let database = fs::read(database_path).map_err(|e| format!("Unable to read database: {e}"))?;
    let manifest = BackupManifest {
        format: "gradia-backup".into(),
        version: 1,
        product: "Gradia".into(),
        app_version: Some(env!("CARGO_PKG_VERSION").into()),
        created_at: Utc::now().to_rfc3339(),
        database_sha256: hash(&database),
    };
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let file = File::create(output_path).map_err(|e| format!("Unable to create backup: {e}"))?;
    let mut writer = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer
        .start_file("manifest.json", options)
        .map_err(|e| e.to_string())?;
    writer
        .write_all(
            serde_json::to_string_pretty(&manifest)
                .map_err(|e| e.to_string())?
                .as_bytes(),
        )
        .map_err(|e| e.to_string())?;
    writer
        .start_file("gradia.db", options)
        .map_err(|e| e.to_string())?;
    writer.write_all(&database).map_err(|e| e.to_string())?;
    writer.finish().map_err(|e| e.to_string())?;
    Ok(manifest)
}

fn read_backup(path: &Path) -> Result<(BackupManifest, Vec<u8>), String> {
    validate_gradia_extension(path)?;
    let file = File::open(path).map_err(|e| format!("Unable to open backup: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid Gradia backup: {e}"))?;
    let manifest: BackupManifest = {
        let mut entry = archive
            .by_name("manifest.json")
            .map_err(|_| "Backup is missing manifest.json.".to_string())?;
        let mut content = String::new();
        entry
            .read_to_string(&mut content)
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&content).map_err(|e| format!("Invalid backup manifest: {e}"))?
    };
    if manifest.format != "gradia-backup" || manifest.version != 1 {
        return Err("Unsupported Gradia backup format or version.".into());
    }
    let database = {
        let mut entry = archive
            .by_name("gradia.db")
            .map_err(|_| "Backup is missing gradia.db.".to_string())?;
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
        bytes
    };
    if hash(&database) != manifest.database_sha256 {
        return Err("Backup checksum verification failed.".into());
    }
    Ok((manifest, database))
}

pub fn restore_backup(database_path: &Path, backup_path: &Path) -> Result<BackupManifest, String> {
    let (manifest, database) = read_backup(backup_path)?;
    let parent = database_path
        .parent()
        .ok_or("Gradia database path has no parent directory.")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let temp_path: PathBuf = parent.join("gradia.restore.tmp.db");
    fs::write(&temp_path, database)
        .map_err(|e| format!("Unable to stage restored database: {e}"))?;
    {
        let connection = db::open(&temp_path)?;
        let integrity: String = connection
            .query_row("PRAGMA integrity_check", [], |row| row.get(0))
            .map_err(|e| e.to_string())?;
        if integrity != "ok" {
            return Err(format!(
                "Restored database failed integrity check: {integrity}"
            ));
        }
    }
    fs::rename(&temp_path, database_path)
        .or_else(|_| {
            fs::copy(&temp_path, database_path)?;
            fs::remove_file(&temp_path)
        })
        .map_err(|e| format!("Unable to replace Gradia database: {e}"))?;
    db::open(database_path)?;
    Ok(manifest)
}

#[tauri::command]
pub fn export_backup(
    state: State<'_, DbState>,
    output_path: String,
) -> Result<BackupManifest, String> {
    create_backup(&state.path, Path::new(&output_path))
}

#[tauri::command]
pub fn import_backup(
    state: State<'_, DbState>,
    backup_path: String,
) -> Result<BackupManifest, String> {
    restore_backup(&state.path, Path::new(&backup_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::params;
    use tempfile::tempdir;

    #[test]
    fn backup_round_trip_preserves_rows() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("gradia.db");
        let backup = directory.path().join("backup.gradia");
        let restored = directory.path().join("restored.db");
        let connection = db::open(&source).unwrap();
        connection
            .execute(
                "INSERT INTO semesters(season, session, is_active, created_at)
                 VALUES ('Fall', '2025-2026', 1, ?1)",
                params![db::now()],
            )
            .unwrap();
        drop(connection);
        create_backup(&source, &backup).unwrap();
        restore_backup(&restored, &backup).unwrap();
        let connection = db::open(&restored).unwrap();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM semesters WHERE season='Fall' AND session='2025-2026'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn transfer_rejects_wrong_extension_and_tampered_database() {
        let directory = tempdir().unwrap();
        let source = directory.path().join("gradia.db");
        let wrong_extension = directory.path().join("transfer.zip");
        let transfer = directory.path().join("transfer.gradia");
        let restored = directory.path().join("restored.db");
        db::open(&source).unwrap();

        assert!(create_backup(&source, &wrong_extension)
            .unwrap_err()
            .contains(".gradia"));

        let manifest = BackupManifest {
            format: "gradia-backup".into(),
            version: 1,
            product: "Gradia".into(),
            app_version: Some(env!("CARGO_PKG_VERSION").into()),
            created_at: Utc::now().to_rfc3339(),
            database_sha256: hash(b"expected database"),
        };
        let file = File::create(&transfer).unwrap();
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        writer.start_file("manifest.json", options).unwrap();
        writer
            .write_all(serde_json::to_string(&manifest).unwrap().as_bytes())
            .unwrap();
        writer.start_file("gradia.db", options).unwrap();
        writer.write_all(b"not a database").unwrap();
        writer.finish().unwrap();

        let error = restore_backup(&restored, &transfer).unwrap_err();
        assert!(error.contains("checksum"));
        assert!(!restored.exists());
    }
}
