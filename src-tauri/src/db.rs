use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION: i64 = 1;

#[derive(Clone, Debug)]
pub struct DbState {
    pub path: PathBuf,
}

pub fn now() -> String {
    Utc::now().to_rfc3339()
}

pub fn open(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Unable to create Gradia data directory: {e}"))?;
    }
    let connection =
        Connection::open(path).map_err(|e| format!("Unable to open Gradia database: {e}"))?;
    configure(&connection)?;
    migrate(&connection)?;
    seed_defaults(&connection)?;
    Ok(connection)
}

#[cfg(test)]
pub fn open_memory() -> Result<Connection, String> {
    let connection = Connection::open_in_memory().map_err(|e| e.to_string())?;
    configure(&connection)?;
    migrate(&connection)?;
    seed_defaults(&connection)?;
    Ok(connection)
}

pub fn configure(connection: &Connection) -> Result<(), String> {
    connection
        .execute_batch(
            "PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;
             PRAGMA synchronous = NORMAL;",
        )
        .map_err(|e| format!("Unable to configure SQLite: {e}"))?;
    Ok(())
}

pub fn migrate(connection: &Connection) -> Result<(), String> {
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| format!("Unable to read database version: {e}"))?;
    if version < 1 {
        connection
            .execute_batch(include_str!("../migrations/0001_init.sql"))
            .map_err(|e| format!("Unable to apply Gradia migration 1: {e}"))?;
        connection
            .pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|e| format!("Unable to store database version: {e}"))?;
    }
    Ok(())
}

fn seed_defaults(connection: &Connection) -> Result<(), String> {
    let count: i64 = connection
        .query_row("SELECT COUNT(*) FROM grading_policies", [], |row| {
            row.get(0)
        })
        .map_err(|e| e.to_string())?;
    if count > 0 {
        return Ok(());
    }

    let timestamp = now();
    connection
        .execute(
            "INSERT INTO grading_policies(name, description, is_default, version, created_at, updated_at)
             VALUES (?1, ?2, 1, 1, ?3, ?3)",
            params![
                "Starter percentage policy",
                "Editable institute-independent starter policy",
                timestamp
            ],
        )
        .map_err(|e| format!("Unable to seed starter policy: {e}"))?;
    let policy_id = connection.last_insert_rowid();
    let bands = [
        (80.0, 100.0, "A+", 4.0, "Pass", "#22c55e"),
        (75.0, 79.999_999, "A", 3.75, "Pass", "#4ade80"),
        (70.0, 74.999_999, "B+", 3.5, "Pass", "#84cc16"),
        (65.0, 69.999_999, "B", 3.25, "Pass", "#a3e635"),
        (60.0, 64.999_999, "C+", 3.0, "Pass", "#eab308"),
        (55.0, 59.999_999, "C", 2.75, "Pass", "#f59e0b"),
        (50.0, 54.999_999, "D+", 2.5, "Pass", "#fb923c"),
        (45.0, 49.999_999, "D", 2.25, "Pass", "#f97316"),
        (0.0, 44.999_999, "F", 0.0, "Fail", "#ef4444"),
    ];
    for (index, band) in bands.iter().enumerate() {
        connection
            .execute(
                "INSERT INTO grade_bands(
                    policy_id, min_percent, max_percent, min_inclusive, max_inclusive,
                    grade_label, grade_point, result_label, color_hex, order_index
                 ) VALUES (?1, ?2, ?3, 1, 1, ?4, ?5, ?6, ?7, ?8)",
                params![
                    policy_id,
                    band.0,
                    band.1,
                    band.2,
                    band.3,
                    band.4,
                    band.5,
                    index as i64
                ],
            )
            .map_err(|e| format!("Unable to seed grade band: {e}"))?;
    }
    Ok(())
}

pub fn audit(
    connection: &Connection,
    entity_type: &str,
    entity_id: Option<i64>,
    action: &str,
    old_json: Option<&str>,
    new_json: Option<&str>,
    reason: Option<&str>,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO audit_log(entity_type, entity_id, action, old_json, new_json, reason, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entity_type,
                entity_id,
                action,
                old_json,
                new_json,
                reason,
                now()
            ],
        )
        .map_err(|e| format!("Unable to write audit history: {e}"))?;
    Ok(())
}

pub fn default_policy_id(connection: &Connection) -> Result<Option<i64>, String> {
    connection
        .query_row(
            "SELECT id FROM grading_policies WHERE is_default = 1 ORDER BY id LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migration_is_idempotent_and_foreign_keys_are_enabled() {
        let connection = open_memory().expect("database");
        migrate(&connection).expect("second migration");
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        let foreign_keys: i64 = connection
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        assert_eq!(foreign_keys, 1);
        let policy_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM grading_policies", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(policy_count, 1);
    }
}
