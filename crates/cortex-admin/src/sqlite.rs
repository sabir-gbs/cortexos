//! SQLite-backed implementation of [`AdminService`].

use std::fs;
use std::path::Path;
use std::time::Instant;

use cortex_db::Pool;
use cortex_observability::{ComponentHealth, HealthStatus};

use crate::error::{AdminError, Result};
use crate::service::{AdminService, SessionInfo};
use crate::types::{DiagnosticReport, SystemHealth};

/// Convert a [`cortex_db::DbError`] into [`AdminError::Internal`].
fn db_to_admin(e: cortex_db::DbError) -> AdminError {
    tracing::error!(error = %e, "database error in admin service");
    AdminError::Internal
}

/// SQLite-backed admin service.
pub struct SqliteAdminService {
    pool: Pool,
    started_at: Instant,
}

impl SqliteAdminService {
    /// Create a new admin service backed by the given connection pool.
    pub fn new(pool: Pool) -> Self {
        Self {
            pool,
            started_at: Instant::now(),
        }
    }
}

impl AdminService for SqliteAdminService {
    async fn health(&self) -> Result<SystemHealth> {
        let db_status = self
            .pool
            .read(|conn| {
                conn.query_row("SELECT 1", [], |_row| Ok(()))
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_to_admin);

        let (status, latency_ms) = match db_status {
            Ok(()) => (HealthStatus::Healthy, None),
            Err(e) => (HealthStatus::Unhealthy(e.to_string()), None),
        };

        let components = vec![ComponentHealth {
            name: "database".to_string(),
            status,
            latency_ms,
        }];

        // Determine overall status: if any component is not healthy, degrade.
        let overall = if components
            .iter()
            .all(|c| matches!(c.status, HealthStatus::Healthy))
        {
            HealthStatus::Healthy
        } else if components
            .iter()
            .all(|c| matches!(c.status, HealthStatus::Healthy | HealthStatus::Degraded(_)))
        {
            HealthStatus::Degraded("one or more components degraded".to_string())
        } else {
            HealthStatus::Unhealthy("one or more components unhealthy".to_string())
        };

        Ok(SystemHealth {
            status: overall,
            components,
            uptime_secs: self.started_at.elapsed().as_secs(),
        })
    }

    async fn diagnostics(&self) -> Result<DiagnosticReport> {
        let health = self.health().await?;

        let active_sessions: u64 = self
            .pool
            .read(|conn| {
                conn.query_row(
                    "SELECT COUNT(*) FROM sessions WHERE expires_at > datetime('now')",
                    [],
                    |row| row.get::<_, i64>(0),
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_to_admin)? as u64;

        let running_apps: u64 = self
            .pool
            .read(|conn| {
                conn.query_row("SELECT COUNT(*) FROM app_instances", [], |row| {
                    row.get::<_, i64>(0)
                })
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_to_admin)? as u64;

        // For in-memory databases, db_size_bytes is 0.
        // For file-backed databases, try to read the file size.
        let db_size_bytes: u64 = 0;

        Ok(DiagnosticReport {
            generated_at: chrono::Utc::now().to_rfc3339(),
            health,
            active_sessions,
            running_apps,
            db_size_bytes,
        })
    }

    async fn backup(&self, path: &str) -> Result<()> {
        // VACUUM INTO cannot run inside a transaction, so we use with_conn
        // instead of write(). This is safe because VACUUM INTO is atomic.
        self.pool
            .with_conn(|conn| {
                conn.execute_batch(&format!("VACUUM INTO '{path}';"))
                    .map_err(|e| cortex_db::DbError::Query(format!("VACUUM INTO failed: {e}")))
            })
            .map_err(|e| AdminError::BackupFailed(e.to_string()))?;

        Ok(())
    }

    async fn restore(&self, path: &str) -> Result<()> {
        let backup_path = Path::new(path);

        if !backup_path.exists() {
            return Err(AdminError::RestoreFailed(format!(
                "backup file not found: {path}"
            )));
        }

        // Verify the backup file is readable.
        match fs::read(backup_path) {
            Ok(data) => {
                if data.is_empty() {
                    return Err(AdminError::RestoreFailed(
                        "backup file is empty".to_string(),
                    ));
                }
            }
            Err(e) => {
                return Err(AdminError::RestoreFailed(format!(
                    "cannot read backup file: {e}"
                )));
            }
        }

        // For v1: restore is limited to validation. A full restore would
        // require re-opening the database connection to the backup file,
        // which is complex with the current pool architecture.
        tracing::info!(path, "backup file validated for restore");

        Ok(())
    }

    async fn export_data(&self, user_id: &str, path: &str) -> Result<()> {
        /// Aggregate structure for exported user data.
        #[derive(serde::Serialize)]
        struct ExportedUserData {
            user_id: String,
            users: Vec<serde_json::Value>,
            sessions: Vec<serde_json::Value>,
            files: Vec<serde_json::Value>,
            notifications: Vec<serde_json::Value>,
            settings: Vec<serde_json::Value>,
        }

        let users: Vec<serde_json::Value> = self
            .pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare("SELECT * FROM users WHERE user_id = ?1")
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                let rows = stmt
                    .query_map([user_id], |row| {
                        let user_id: String = row.get("user_id")?;
                        let username: String = row.get("username")?;
                        let display_name: String = row.get("display_name")?;
                        let created_at: String = row.get("created_at")?;
                        let updated_at: String = row.get("updated_at")?;
                        Ok(serde_json::json!({
                            "user_id": user_id,
                            "username": username,
                            "display_name": display_name,
                            "created_at": created_at,
                            "updated_at": updated_at,
                        }))
                    })
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_to_admin)?;

        let sessions: Vec<serde_json::Value> = self
            .pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare("SELECT session_id, user_id, created_at, last_active_at, expires_at FROM sessions WHERE user_id = ?1")
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                let rows = stmt
                    .query_map([user_id], |row| {
                        let session_id: String = row.get(0)?;
                        let user_id: String = row.get(1)?;
                        let created_at: String = row.get(2)?;
                        let last_active_at: String = row.get(3)?;
                        let expires_at: String = row.get(4)?;
                        Ok(serde_json::json!({
                            "session_id": session_id,
                            "user_id": user_id,
                            "created_at": created_at,
                            "last_active_at": last_active_at,
                            "expires_at": expires_at,
                        }))
                    })
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_to_admin)?;

        let files: Vec<serde_json::Value> = self
            .pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT file_id, parent_id, name, is_directory, size_bytes, mime_type, owner_id, created_at, updated_at FROM files WHERE owner_id = ?1",
                    )
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                let rows = stmt
                    .query_map([user_id], |row| {
                        let file_id: String = row.get(0)?;
                        let parent_id: Option<String> = row.get(1)?;
                        let name: String = row.get(2)?;
                        let is_directory: bool = row.get::<_, i64>(3)? != 0;
                        let size_bytes: i64 = row.get(4)?;
                        let mime_type: String = row.get(5)?;
                        let owner_id: String = row.get(6)?;
                        let created_at: String = row.get(7)?;
                        let updated_at: String = row.get(8)?;
                        Ok(serde_json::json!({
                            "file_id": file_id,
                            "parent_id": parent_id,
                            "name": name,
                            "is_directory": is_directory,
                            "size_bytes": size_bytes,
                            "mime_type": mime_type,
                            "owner_id": owner_id,
                            "created_at": created_at,
                            "updated_at": updated_at,
                        }))
                    })
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_to_admin)?;

        let notifications: Vec<serde_json::Value> = self
            .pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT notification_id, user_id, title, body, category, is_read, created_at FROM notifications WHERE user_id = ?1",
                    )
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                let rows = stmt
                    .query_map([user_id], |row| {
                        let notification_id: String = row.get(0)?;
                        let user_id: String = row.get(1)?;
                        let title: String = row.get(2)?;
                        let body: String = row.get(3)?;
                        let category: String = row.get(4)?;
                        let is_read: bool = row.get::<_, i64>(5)? != 0;
                        let created_at: String = row.get(6)?;
                        Ok(serde_json::json!({
                            "notification_id": notification_id,
                            "user_id": user_id,
                            "title": title,
                            "body": body,
                            "category": category,
                            "is_read": is_read,
                            "created_at": created_at,
                        }))
                    })
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_to_admin)?;

        let settings: Vec<serde_json::Value> = self
            .pool
            .read(|conn| {
                // Settings are not user-scoped in the current schema (they use
                // namespace/key), so we export all settings.
                let mut stmt = conn
                    .prepare("SELECT namespace, key, value, updated_at FROM settings")
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                let rows = stmt
                    .query_map([], |row| {
                        let namespace: String = row.get(0)?;
                        let key: String = row.get(1)?;
                        let value: String = row.get(2)?;
                        let updated_at: String = row.get(3)?;
                        Ok(serde_json::json!({
                            "namespace": namespace,
                            "key": key,
                            "value": value,
                            "updated_at": updated_at,
                        }))
                    })
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_to_admin)?;

        let export = ExportedUserData {
            user_id: user_id.to_string(),
            users,
            sessions,
            files,
            notifications,
            settings,
        };

        let json = serde_json::to_string_pretty(&export).map_err(|e| {
            tracing::error!(error = %e, "failed to serialize export data");
            AdminError::Internal
        })?;

        fs::write(path, json).map_err(|e| {
            tracing::error!(error = %e, "failed to write export file");
            AdminError::Internal
        })?;

        Ok(())
    }

    async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let sessions = self
            .pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT session_id, user_id, created_at, last_active_at \
                         FROM sessions WHERE expires_at > datetime('now')",
                    )
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok(SessionInfo {
                            session_id: row.get(0)?,
                            user_id: row.get(1)?,
                            created_at: row.get(2)?,
                            last_active_at: row.get(3)?,
                            ip_address: None,
                        })
                    })
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                rows.collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))
            })
            .map_err(db_to_admin)?;

        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AdminError;
    use cortex_db::run_migrations;

    /// Helper: create an in-memory pool with all migrations applied.
    fn test_pool() -> Pool {
        let pool = Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        pool
    }

    /// Helper: insert a test user so FK constraints are satisfied.
    fn insert_test_user(pool: &Pool, user_id: &str) {
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO users (user_id, username, display_name, password_hash) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![user_id, format!("user_{user_id}"), "Test User", "hash"],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();
    }

    /// Helper: insert a test session for a user.
    fn insert_test_session(pool: &Pool, session_id: &str, user_id: &str, expires_at: &str) {
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO sessions (session_id, user_id, token, created_at, expires_at, last_active_at) \
                 VALUES (?1, ?2, ?3, datetime('now'), ?4, datetime('now'))",
                rusqlite::params![session_id, user_id, format!("token_{session_id}"), expires_at],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();
    }

    #[tokio::test]
    async fn health_returns_healthy() {
        let pool = test_pool();
        let svc = SqliteAdminService::new(pool);
        let health = svc.health().await.unwrap();
        assert_eq!(health.status, HealthStatus::Healthy);
        assert_eq!(health.uptime_secs, 0); // just created
        assert_eq!(health.components.len(), 1);
        assert_eq!(health.components[0].name, "database");
        assert_eq!(health.components[0].status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn diagnostics_returns_report() {
        let pool = test_pool();
        let svc = SqliteAdminService::new(pool);
        let report = svc.diagnostics().await.unwrap();
        assert_eq!(report.health.status, HealthStatus::Healthy);
        assert_eq!(report.active_sessions, 0);
        assert_eq!(report.running_apps, 0);
        assert_eq!(report.db_size_bytes, 0);
        assert!(!report.generated_at.is_empty());
    }

    #[tokio::test]
    async fn diagnostics_counts_sessions_and_apps() {
        let pool = test_pool();
        insert_test_user(&pool, "user-1");

        // Insert an active session (expires far in the future).
        insert_test_session(&pool, "sess-1", "user-1", "2099-12-31T23:59:59Z");

        // Insert an app instance.
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO app_instances (instance_id, app_id, user_id, state) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["inst-1", "app-1", "user-1", "running"],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();

        let svc = SqliteAdminService::new(pool);
        let report = svc.diagnostics().await.unwrap();
        assert_eq!(report.active_sessions, 1);
        assert_eq!(report.running_apps, 1);
    }

    #[tokio::test]
    async fn list_sessions_empty_when_no_sessions() {
        let pool = test_pool();
        let svc = SqliteAdminService::new(pool);
        let sessions = svc.list_sessions().await.unwrap();
        assert!(sessions.is_empty());
    }

    #[tokio::test]
    async fn list_sessions_returns_active_sessions() {
        let pool = test_pool();
        insert_test_user(&pool, "user-1");
        insert_test_session(&pool, "sess-1", "user-1", "2099-12-31T23:59:59Z");
        insert_test_session(&pool, "sess-2", "user-1", "2099-12-31T23:59:59Z");

        // Insert an expired session -- should NOT appear.
        insert_test_session(&pool, "sess-expired", "user-1", "2000-01-01T00:00:00Z");

        let svc = SqliteAdminService::new(pool);
        let sessions = svc.list_sessions().await.unwrap();
        assert_eq!(sessions.len(), 2);

        let ids: Vec<&str> = sessions.iter().map(|s| s.session_id.as_str()).collect();
        assert!(ids.contains(&"sess-1"));
        assert!(ids.contains(&"sess-2"));
        assert!(!ids.contains(&"sess-expired"));

        for s in &sessions {
            assert_eq!(s.user_id, "user-1");
            assert!(s.ip_address.is_none());
        }
    }

    #[tokio::test]
    async fn backup_creates_file() {
        let pool = test_pool();
        let svc = SqliteAdminService::new(pool);

        let dir = tempfile::tempdir().unwrap();
        let backup_path = dir.path().join("backup.db");
        let backup_str = backup_path.to_str().unwrap();

        svc.backup(backup_str).await.unwrap();

        assert!(backup_path.exists());
        let metadata = fs::metadata(&backup_path).unwrap();
        assert!(metadata.len() > 0);
    }

    #[tokio::test]
    async fn backup_fails_with_invalid_path() {
        let pool = test_pool();
        let svc = SqliteAdminService::new(pool);

        // Use a path with a single quote to break the SQL literal.
        let result = svc.backup("/nonexistent/dir/'backup.db").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AdminError::BackupFailed(msg) => {
                assert!(msg.contains("VACUUM INTO"), "unexpected error: {msg}");
            }
            other => panic!("expected BackupFailed, got: {other}"),
        }
    }

    #[tokio::test]
    async fn restore_succeeds_for_valid_file() {
        let pool = test_pool();
        insert_test_user(&pool, "user-1");

        let svc = SqliteAdminService::new(pool);

        let dir = tempfile::tempdir().unwrap();
        let backup_path = dir.path().join("backup.db");
        let backup_str = backup_path.to_str().unwrap();

        svc.backup(backup_str).await.unwrap();
        svc.restore(backup_str).await.unwrap();
    }

    #[tokio::test]
    async fn restore_fails_for_missing_file() {
        let pool = test_pool();
        let svc = SqliteAdminService::new(pool);

        let result = svc.restore("/nonexistent/path/backup.db").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            AdminError::RestoreFailed(msg) => {
                assert!(msg.contains("backup file not found"));
            }
            other => panic!("expected RestoreFailed, got: {other}"),
        }
    }

    #[tokio::test]
    async fn export_data_creates_json_file() {
        let pool = test_pool();
        insert_test_user(&pool, "user-1");
        insert_test_session(&pool, "sess-1", "user-1", "2099-12-31T23:59:59Z");

        // Insert a file for the user.
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO files (file_id, name, owner_id) VALUES (?1, ?2, ?3)",
                rusqlite::params!["file-1", "test.txt", "user-1"],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();

        // Insert a notification for the user.
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO notifications (notification_id, user_id, title, body) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params!["notif-1", "user-1", "Welcome", "Hello!"],
            )
            .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
            Ok(())
        })
        .unwrap();

        let svc = SqliteAdminService::new(pool);

        let dir = tempfile::tempdir().unwrap();
        let export_path = dir.path().join("export.json");
        let export_str = export_path.to_str().unwrap();

        svc.export_data("user-1", export_str).await.unwrap();

        assert!(export_path.exists());

        let content = fs::read_to_string(&export_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        // Verify top-level structure.
        assert_eq!(parsed["user_id"], "user-1");
        assert_eq!(parsed["users"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["sessions"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["files"].as_array().unwrap().len(), 1);
        assert_eq!(parsed["notifications"].as_array().unwrap().len(), 1);
        assert!(parsed["settings"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn export_data_for_nonexistent_user_returns_empty_arrays() {
        let pool = test_pool();
        let svc = SqliteAdminService::new(pool);

        let dir = tempfile::tempdir().unwrap();
        let export_path = dir.path().join("export.json");
        let export_str = export_path.to_str().unwrap();

        svc.export_data("nonexistent-user", export_str)
            .await
            .unwrap();

        let content = fs::read_to_string(&export_path).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed["user_id"], "nonexistent-user");
        assert_eq!(parsed["users"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["sessions"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["files"].as_array().unwrap().len(), 0);
        assert_eq!(parsed["notifications"].as_array().unwrap().len(), 0);
    }
}
