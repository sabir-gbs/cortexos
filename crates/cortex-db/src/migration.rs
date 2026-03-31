//! Migration runner for SQLite schema management.
//!
//! Applies numbered SQL migrations in order. Tracks applied migrations
//! in the `_schema_version` table.

use crate::error::{DbError, Result};
use crate::types::Pool;

/// Built-in migrations embedded at compile time.
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "0001_create_schema_version",
        include_str!("../migrations/0001_create_schema_version.up.sql"),
    ),
    (
        "0002_create_users",
        include_str!("../migrations/0002_create_users.up.sql"),
    ),
    (
        "0003_create_sessions",
        include_str!("../migrations/0003_create_sessions.up.sql"),
    ),
    (
        "0004_create_permissions",
        include_str!("../migrations/0004_create_permissions.up.sql"),
    ),
    (
        "0005_create_settings",
        include_str!("../migrations/0005_create_settings.up.sql"),
    ),
    (
        "0006_create_files",
        include_str!("../migrations/0006_create_files.up.sql"),
    ),
    (
        "0007_create_notifications",
        include_str!("../migrations/0007_create_notifications.up.sql"),
    ),
    (
        "0008_create_apps",
        include_str!("../migrations/0008_create_apps.up.sql"),
    ),
    (
        "0009_create_ai_config",
        include_str!("../migrations/0009_create_ai_config.up.sql"),
    ),
    (
        "0010_create_audit_log",
        include_str!("../migrations/0010_create_audit_log.up.sql"),
    ),
    (
        "0011_create_search_index",
        include_str!("../migrations/0011_create_search_index.up.sql"),
    ),
    (
        "0012_create_command_bus_events",
        include_str!("../migrations/0012_create_command_bus_events.up.sql"),
    ),
];

/// Run all pending migrations against the database.
pub fn run_migrations(pool: &Pool) -> Result<()> {
    pool.write(|conn| {
        // Ensure schema_version table exists
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _schema_version (
                migration_id TEXT PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (datetime('now'))
            );",
        )
        .map_err(|e| DbError::Migration(format!("cannot create schema_version: {e}")))?;

        for (id, sql) in MIGRATIONS {
            // Check if already applied
            let applied: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM _schema_version WHERE migration_id = ?1",
                    [id],
                    |row| row.get(0),
                )
                .map_err(|e| DbError::Migration(format!("migration check failed: {e}")))?;

            if applied {
                continue;
            }

            tracing::info!(migration = id, "applying migration");
            conn.execute_batch(sql)
                .map_err(|e| DbError::Migration(format!("migration {id} failed: {e}")))?;

            conn.execute(
                "INSERT INTO _schema_version (migration_id) VALUES (?1)",
                [id],
            )
            .map_err(|e| DbError::Migration(format!("cannot record migration {id}: {e}")))?;
        }

        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_migrations_on_fresh_db() {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();

        // Verify all migrations were recorded
        pool.read(|conn| {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM _schema_version", [], |row| row.get(0))
                .map_err(|e| DbError::Query(e.to_string()))?;
            assert_eq!(count, MIGRATIONS.len() as i64);
            Ok(())
        })
        .unwrap();
    }

    #[test]
    fn migrations_are_idempotent() {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        // Running again should succeed without error
        run_migrations(&pool).unwrap();
    }
}
