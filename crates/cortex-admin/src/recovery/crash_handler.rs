//! Crash handler for recording and managing app crashes (SPEC 22 §10.3).
//!
//! Records app crash events to a SQLite table, tracks auto-restart attempts,
//! and enforces a maximum of 3 restarts per app per session.

use cortex_db::Pool;
use uuid::Uuid;

use crate::error::{AdminError, Result};
use crate::types::CrashRecord;

/// Maximum auto-restart attempts per app per session (SPEC 22 §5.6.1).
const MAX_AUTO_RESTART_ATTEMPTS: u32 = 3;

/// Convert a database error into an AdminError.
fn db_err(e: cortex_db::DbError) -> AdminError {
    AdminError::CrashError(e.to_string())
}

/// Handles crash event recording, auto-restart logic, and crash log management.
pub struct CrashHandler {
    pool: Pool,
    max_restart_attempts: u32,
}

impl CrashHandler {
    /// Create a new crash handler with the default max restart attempts (3).
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            max_restart_attempts: MAX_AUTO_RESTART_ATTEMPTS,
        }
    }

    /// Create a crash handler with a custom max restart attempts value.
    pub fn with_max_attempts(pool: Pool, max: u32) -> Self {
        Self {
            pool,
            max_restart_attempts: max,
        }
    }

    /// Ensure the crash_records table exists.
    pub fn init_schema(&self) -> Result<()> {
        self.pool
            .write(|conn| {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS admin_crash_records (
                         id                      TEXT PRIMARY KEY,
                         app_id                  TEXT NOT NULL,
                         app_name                TEXT NOT NULL,
                         exit_code               INTEGER NOT NULL,
                         signal                  INTEGER,
                         crash_time              TEXT NOT NULL,
                         crash_log_path          TEXT NOT NULL,
                         auto_restart_attempts   INTEGER NOT NULL DEFAULT 0
                     );",
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_err)?;
        Ok(())
    }

    /// Record a new crash event.
    pub fn record_crash(
        &self,
        app_id: &str,
        app_name: &str,
        exit_code: i32,
        signal: Option<i32>,
        crash_log_path: &str,
    ) -> Result<CrashRecord> {
        let id = Uuid::now_v7();
        let crash_time = chrono::Utc::now().to_rfc3339();

        // Check existing restart attempts for this app.
        let existing_attempts = self.get_max_restart_attempts(app_id);

        let record = CrashRecord {
            id,
            app_id: app_id.to_string(),
            app_name: app_name.to_string(),
            exit_code,
            signal,
            crash_time: crash_time.clone(),
            crash_log_path: crash_log_path.into(),
            auto_restart_attempts: existing_attempts,
        };

        self.pool
            .write(|conn| {
                conn.execute(
                    "INSERT INTO admin_crash_records \
                     (id, app_id, app_name, exit_code, signal, crash_time, crash_log_path, auto_restart_attempts) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        id.to_string(),
                        app_id,
                        app_name,
                        exit_code,
                        signal,
                        crash_time,
                        crash_log_path,
                        existing_attempts,
                    ],
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_err)?;

        tracing::error!(
            app_id,
            exit_code,
            signal,
            crash_log_path,
            "app crash recorded"
        );
        Ok(record)
    }

    /// Get the max auto-restart attempts for an app.
    fn get_max_restart_attempts(&self, app_id: &str) -> u32 {
        self.pool
            .read(|conn| {
                let val: u32 = conn
                    .query_row(
                        "SELECT COALESCE(MAX(auto_restart_attempts), 0) \
                         FROM admin_crash_records WHERE app_id = ?1",
                        rusqlite::params![app_id],
                        |row| row.get::<_, u32>(0),
                    )
                    .unwrap_or(0);
                Ok(val)
            })
            .map_err(|e| {
                tracing::warn!(app_id, error = %e, "failed to query restart attempts");
                e
            })
            .unwrap_or(0)
    }

    /// Check whether auto-restart should be attempted for the given app.
    pub fn should_auto_restart(&self, app_id: &str) -> bool {
        let attempts = self.get_max_restart_attempts(app_id);
        attempts < self.max_restart_attempts
    }

    /// Increment the auto-restart counter for an app after a restart attempt.
    pub fn increment_restart_attempts(&self, app_id: &str) -> Result<()> {
        self.pool
            .write(|conn| {
                conn.execute(
                    "UPDATE admin_crash_records \
                     SET auto_restart_attempts = auto_restart_attempts + 1 \
                     WHERE app_id = ?1",
                    rusqlite::params![app_id],
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_err)?;
        Ok(())
    }

    /// Get all crash records for the current session, most recent first.
    pub fn get_crash_history(&self) -> Result<Vec<CrashRecord>> {
        let records = self
            .pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, app_id, app_name, exit_code, signal, \
                         crash_time, crash_log_path, auto_restart_attempts \
                         FROM admin_crash_records ORDER BY crash_time DESC",
                    )
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                let rows = stmt
                    .query_map([], |row| {
                        let id_str: String = row.get(0)?;
                        Ok(CrashRecord {
                            id: Uuid::parse_str(&id_str).unwrap_or(Uuid::nil()),
                            app_id: row.get(1)?,
                            app_name: row.get(2)?,
                            exit_code: row.get(3)?,
                            signal: row.get(4)?,
                            crash_time: row.get(5)?,
                            crash_log_path: row.get::<_, String>(6)?.into(),
                            auto_restart_attempts: row.get(7)?,
                        })
                    })
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_err)?;
        Ok(records)
    }

    /// Clear all crash history.
    pub fn clear_crash_history(&self) -> Result<()> {
        self.pool
            .write(|conn| {
                conn.execute("DELETE FROM admin_crash_records", [])
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_err)?;
        tracing::info!("crash history cleared");
        Ok(())
    }

    /// Get the crash log path for an app.
    pub fn crash_log_path(&self, app_id: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("/var/log/cortexos/crash_{app_id}.log"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_db::run_migrations;

    fn test_pool() -> Pool {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        pool
    }

    #[test]
    fn record_crash_creates_record() {
        let pool = test_pool();
        let handler = CrashHandler::new(pool);
        handler.init_schema().unwrap();

        let record = handler
            .record_crash(
                "com.cortexos.settings",
                "Settings",
                139,
                Some(11),
                "/var/log/cortexos/crash_settings.log",
            )
            .unwrap();

        assert_eq!(record.app_id, "com.cortexos.settings");
        assert_eq!(record.exit_code, 139);
        assert_eq!(record.signal, Some(11));
        assert_eq!(record.auto_restart_attempts, 0);
    }

    #[test]
    fn crash_history_returns_records() {
        let pool = test_pool();
        let handler = CrashHandler::new(pool);
        handler.init_schema().unwrap();

        handler
            .record_crash("com.cortexos.settings", "Settings", 1, None, "/tmp/a.log")
            .unwrap();
        handler
            .record_crash(
                "com.cortexos.terminal-lite",
                "Terminal",
                139,
                Some(11),
                "/tmp/b.log",
            )
            .unwrap();

        let history = handler.get_crash_history().unwrap();
        assert_eq!(history.len(), 2);
    }

    #[test]
    fn auto_restart_allowed_under_limit() {
        let pool = test_pool();
        let handler = CrashHandler::with_max_attempts(pool, 3);
        handler.init_schema().unwrap();

        assert!(handler.should_auto_restart("com.cortexos.settings"));

        for _ in 0..3 {
            handler
                .record_crash("com.cortexos.settings", "Settings", 1, None, "/tmp/a.log")
                .unwrap();
            handler
                .increment_restart_attempts("com.cortexos.settings")
                .unwrap();
        }

        assert!(!handler.should_auto_restart("com.cortexos.settings"));
    }

    #[test]
    fn clear_crash_history() {
        let pool = test_pool();
        let handler = CrashHandler::new(pool);
        handler.init_schema().unwrap();

        handler
            .record_crash("com.cortexos.settings", "Settings", 1, None, "/tmp/a.log")
            .unwrap();
        assert_eq!(handler.get_crash_history().unwrap().len(), 1);

        handler.clear_crash_history().unwrap();
        assert!(handler.get_crash_history().unwrap().is_empty());
    }

    #[test]
    fn crash_log_path_returns_expected_path() {
        let pool = test_pool();
        let handler = CrashHandler::new(pool);
        let path = handler.crash_log_path("com.cortexos.settings");
        assert_eq!(
            path.to_str().unwrap(),
            "/var/log/cortexos/crash_com.cortexos.settings.log"
        );
    }
}
