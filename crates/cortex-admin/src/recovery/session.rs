//! Session state manager (SPEC 22 §10.2).
//!
//! Saves and loads session state so that running apps can be restored after
//! an unclean shutdown. Session state is persisted to the database and
//! includes the list of running apps with their window positions.

use cortex_db::Pool;

use crate::error::{AdminError, Result};
use crate::types::SessionState;

/// Convert a database error into an AdminError.
fn db_err(e: cortex_db::DbError) -> AdminError {
    AdminError::SessionError(e.to_string())
}

/// Manages session state persistence and recovery.
pub struct SessionManager {
    pool: Pool,
}

impl SessionManager {
    /// Create a new session manager backed by the given pool.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Ensure the session_state table exists.
    pub fn init_schema(&self) -> Result<()> {
        self.pool
            .write(|conn| {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS admin_session_state (
                         key   TEXT PRIMARY KEY,
                         value TEXT NOT NULL
                     );",
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_err)?;
        Ok(())
    }

    /// Save the current session state atomically.
    pub fn save_session(&self, state: &SessionState) -> Result<()> {
        let json = serde_json::to_string(state).map_err(|e| {
            AdminError::SessionError(format!("failed to serialize session state: {e}"))
        })?;

        self.pool
            .write(|conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO admin_session_state (key, value) VALUES ('current', ?1)",
                    rusqlite::params![json],
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_err)?;

        tracing::debug!(
            app_count = state.apps.len(),
            clean_shutdown = state.clean_shutdown,
            "session state saved"
        );
        Ok(())
    }

    /// Load the last saved session state.
    pub fn load_session(&self) -> Result<Option<SessionState>> {
        let result: Option<String> = self
            .pool
            .read(|conn| {
                conn.query_row(
                    "SELECT value FROM admin_session_state WHERE key = 'current'",
                    [],
                    |row| row.get::<_, String>(0),
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_err)
            .ok();

        match result {
            Some(json) => {
                let state: SessionState = serde_json::from_str(&json).map_err(|e| {
                    tracing::warn!(error = %e, "failed to parse session state, treating as no session");
                    AdminError::SessionError(format!("corrupt session state: {e}"))
                })?;
                Ok(Some(state))
            }
            None => Ok(None),
        }
    }

    /// Mark the session as cleanly shut down.
    pub fn mark_clean_shutdown(&self) -> Result<()> {
        let mut state = self.load_session().unwrap_or_else(|_| {
            Some(SessionState {
                version: 1,
                clean_shutdown: false,
                saved_at: chrono::Utc::now().to_rfc3339(),
                apps: vec![],
            })
        });

        if let Some(ref mut state) = state {
            state.clean_shutdown = true;
            state.saved_at = chrono::Utc::now().to_rfc3339();
            self.save_session(state)?;
        }

        Ok(())
    }

    /// Clear saved session state.
    pub fn clear_session(&self) -> Result<()> {
        self.pool
            .write(|conn| {
                conn.execute("DELETE FROM admin_session_state WHERE key = 'current'", [])
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_err)?;

        tracing::info!("saved session state cleared");
        Ok(())
    }

    /// Check if the previous session ended cleanly. Returns `true` if no
    /// saved state exists (a clean boot with no prior state is treated as
    /// clean).
    pub fn was_clean_shutdown(&self) -> Result<bool> {
        match self.load_session()? {
            Some(state) => Ok(state.clean_shutdown),
            None => Ok(true),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SessionApp;
    use cortex_db::run_migrations;

    fn test_pool() -> Pool {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        pool
    }

    #[test]
    fn save_and_load_roundtrip() {
        let pool = test_pool();
        let mgr = SessionManager::new(pool);
        mgr.init_schema().unwrap();

        let state = SessionState {
            version: 1,
            clean_shutdown: false,
            saved_at: "2026-03-30T12:00:00Z".to_string(),
            apps: vec![
                SessionApp {
                    app_id: "com.cortexos.settings".to_string(),
                    window_x: 100,
                    window_y: 100,
                    window_width: 800,
                    window_height: 600,
                    was_focused: true,
                },
                SessionApp {
                    app_id: "com.cortexos.file-manager".to_string(),
                    window_x: 200,
                    window_y: 200,
                    window_width: 900,
                    window_height: 600,
                    was_focused: false,
                },
            ],
        };

        mgr.save_session(&state).unwrap();
        let loaded = mgr.load_session().unwrap().unwrap();
        assert_eq!(loaded.version, 1);
        assert!(!loaded.clean_shutdown);
        assert_eq!(loaded.apps.len(), 2);
        assert_eq!(loaded.apps[0].app_id, "com.cortexos.settings");
        assert!(loaded.apps[0].was_focused);
        assert!(!loaded.apps[1].was_focused);
    }

    #[test]
    fn load_returns_none_when_empty() {
        let pool = test_pool();
        let mgr = SessionManager::new(pool);
        mgr.init_schema().unwrap();
        assert!(mgr.load_session().unwrap().is_none());
    }

    #[test]
    fn mark_clean_shutdown() {
        let pool = test_pool();
        let mgr = SessionManager::new(pool);
        mgr.init_schema().unwrap();

        let state = SessionState {
            version: 1,
            clean_shutdown: false,
            saved_at: "2026-03-30T12:00:00Z".to_string(),
            apps: vec![SessionApp {
                app_id: "com.cortexos.notes".to_string(),
                window_x: 0,
                window_y: 0,
                window_width: 900,
                window_height: 600,
                was_focused: true,
            }],
        };
        mgr.save_session(&state).unwrap();
        assert!(!mgr.was_clean_shutdown().unwrap());

        mgr.mark_clean_shutdown().unwrap();
        assert!(mgr.was_clean_shutdown().unwrap());

        let loaded = mgr.load_session().unwrap().unwrap();
        assert!(loaded.clean_shutdown);
        assert_eq!(loaded.apps.len(), 1);
    }

    #[test]
    fn clear_session() {
        let pool = test_pool();
        let mgr = SessionManager::new(pool);
        mgr.init_schema().unwrap();

        let state = SessionState {
            version: 1,
            clean_shutdown: false,
            saved_at: "2026-03-30T12:00:00Z".to_string(),
            apps: vec![],
        };
        mgr.save_session(&state).unwrap();
        assert!(mgr.load_session().unwrap().is_some());

        mgr.clear_session().unwrap();
        assert!(mgr.load_session().unwrap().is_none());
    }

    #[test]
    fn was_clean_shutdown_true_when_no_state() {
        let pool = test_pool();
        let mgr = SessionManager::new(pool);
        mgr.init_schema().unwrap();
        assert!(mgr.was_clean_shutdown().unwrap());
    }
}
