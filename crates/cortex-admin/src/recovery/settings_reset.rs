//! Settings reset functionality (SPEC 22 §5.6.4).

use cortex_db::Pool;

use crate::error::{AdminError, Result};

/// Handles resetting app-specific and system-wide settings.
pub struct SettingsReset {
    pool: Pool,
}

impl SettingsReset {
    /// Create a new settings reset handler.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Reset a specific app's settings to defaults.
    ///
    /// App settings are stored with the app's ID as the namespace.
    /// This deletes all settings rows in that namespace.
    pub fn reset_app_settings(&self, app_id: &str) -> Result<u64> {
        let namespace = app_id;
        let deleted = self
            .pool
            .write(|conn| {
                conn.execute(
                    "DELETE FROM settings WHERE namespace = ?1",
                    rusqlite::params![namespace],
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(|_| AdminError::Internal)?;

        tracing::warn!(app_id, deleted, "app settings reset");
        Ok(deleted as u64)
    }

    /// Reset all system settings to defaults.
    ///
    /// System settings use the "system" namespace. This deletes all rows
    /// in the system namespace.
    pub fn reset_system_settings(&self) -> Result<u64> {
        let deleted = self
            .pool
            .write(|conn| {
                conn.execute("DELETE FROM settings WHERE namespace = 'system'", [])
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(|_| AdminError::Internal)?;

        tracing::warn!(deleted, "system settings reset to defaults");
        Ok(deleted as u64)
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

    fn insert_setting(pool: &Pool, namespace: &str, key: &str, value: &str) {
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO settings (namespace, key, value) VALUES (?1, ?2, ?3)",
                rusqlite::params![namespace, key, value],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))
        })
        .unwrap();
    }

    #[test]
    fn reset_app_settings_deletes_only_app_namespace() {
        let pool = test_pool();
        insert_setting(&pool, "com.cortexos.settings", "theme", "dark");
        insert_setting(&pool, "com.cortexos.settings", "lang", "en");
        insert_setting(&pool, "system", "version", "0.1.0");

        let reset = SettingsReset::new(pool);
        let deleted = reset.reset_app_settings("com.cortexos.settings").unwrap();
        assert_eq!(deleted, 2);
    }

    #[test]
    fn reset_system_settings_deletes_system_namespace() {
        let pool = test_pool();
        insert_setting(&pool, "system", "version", "0.1.0");
        insert_setting(&pool, "system", "locale", "en-US");
        insert_setting(&pool, "com.cortexos.settings", "theme", "dark");

        let reset = SettingsReset::new(pool);
        let deleted = reset.reset_system_settings().unwrap();
        assert_eq!(deleted, 2);
    }

    #[test]
    fn reset_nonexistent_app_returns_zero() {
        let pool = test_pool();
        let reset = SettingsReset::new(pool);
        let deleted = reset.reset_app_settings("com.nonexistent").unwrap();
        assert_eq!(deleted, 0);
    }
}
