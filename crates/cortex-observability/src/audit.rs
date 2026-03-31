//! Audit log persistence.
//!
//! Writes structured audit events to the `audit_log` SQLite table.
//! Every security-relevant action (login, permission change, setting
//! mutation, file access) should be recorded through this module.

use crate::error::{ObservabilityError, Result};

/// An audit event ready to be persisted.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEvent {
    /// Unique event identifier.
    pub event_id: String,
    /// Event type (e.g. "auth.login", "policy.grant", "file.write").
    pub event_type: String,
    /// The user who triggered the event.
    pub user_id: Option<String>,
    /// The app involved (if applicable).
    pub app_id: Option<String>,
    /// Arbitrary JSON details about the event.
    pub details: serde_json::Value,
}

/// Append-only audit logger backed by SQLite.
pub struct AuditLogger {
    pool: cortex_db::Pool,
}

impl AuditLogger {
    /// Create a new audit logger writing to the given database pool.
    pub fn new(pool: cortex_db::Pool) -> Self {
        Self { pool }
    }

    /// Append an audit event to the log.
    ///
    /// This is an append-only operation; events cannot be modified or
    /// deleted after insertion.
    pub fn append(&self, event: &AuditEvent) -> Result<()> {
        let details_json = serde_json::to_string(&event.details)
            .map_err(|e| ObservabilityError::Internal(format!("cannot serialize details: {e}")))?;

        self.pool
            .write(|conn| {
                conn.execute(
                    "INSERT INTO audit_log (event_id, event_type, user_id, app_id, details_json) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        event.event_id,
                        event.event_type,
                        event.user_id,
                        event.app_id,
                        details_json,
                    ],
                )
                .map_err(|e| {
                    cortex_db::DbError::Internal(format!("audit log insert failed: {e}"))
                })?;
                Ok(())
            })
            .map_err(|e| ObservabilityError::Internal(format!("audit write failed: {e}")))
    }

    /// Query recent audit events, ordered by creation time descending.
    pub fn recent(&self, limit: u32) -> Result<Vec<AuditEvent>> {
        self.pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT event_id, event_type, user_id, app_id, details_json \
                         FROM audit_log ORDER BY created_at DESC LIMIT ?1",
                    )
                    .map_err(|e| cortex_db::DbError::Internal(format!("prepare failed: {e}")))?;

                let rows = stmt
                    .query_map([limit], |row| {
                        let event_id: String = row.get(0)?;
                        let event_type: String = row.get(1)?;
                        let user_id: Option<String> = row.get(2)?;
                        let app_id: Option<String> = row.get(3)?;
                        let details_json: String = row.get(4)?;
                        Ok((event_id, event_type, user_id, app_id, details_json))
                    })
                    .map_err(|e| cortex_db::DbError::Internal(format!("query failed: {e}")))?;

                let mut events = Vec::new();
                for row_result in rows {
                    let (event_id, event_type, user_id, app_id, details_json) = row_result
                        .map_err(|e| {
                            cortex_db::DbError::Internal(format!("row read failed: {e}"))
                        })?;
                    let details: serde_json::Value =
                        serde_json::from_str(&details_json).unwrap_or(serde_json::json!({}));
                    events.push(AuditEvent {
                        event_id,
                        event_type,
                        user_id,
                        app_id,
                        details,
                    });
                }
                Ok(events)
            })
            .map_err(|e| ObservabilityError::Internal(format!("audit read failed: {e}")))
    }

    /// Count total audit events.
    pub fn count(&self) -> Result<u64> {
        self.pool
            .read(|conn| {
                let count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM audit_log", [], |row| row.get(0))
                    .map_err(|e| cortex_db::DbError::Internal(format!("count failed: {e}")))?;
                Ok(count as u64)
            })
            .map_err(|e| ObservabilityError::Internal(format!("audit count failed: {e}")))
    }

    /// Query events for a specific user.
    pub fn for_user(&self, user_id: &str, limit: u32) -> Result<Vec<AuditEvent>> {
        self.pool
            .read(|conn| {
                let mut stmt = conn
                    .prepare(
                        "SELECT event_id, event_type, user_id, app_id, details_json \
                         FROM audit_log WHERE user_id = ?1 \
                         ORDER BY created_at DESC LIMIT ?2",
                    )
                    .map_err(|e| cortex_db::DbError::Internal(format!("prepare failed: {e}")))?;

                let rows = stmt
                    .query_map(rusqlite::params![user_id, limit], |row| {
                        let event_id: String = row.get(0)?;
                        let event_type: String = row.get(1)?;
                        let user_id: Option<String> = row.get(2)?;
                        let app_id: Option<String> = row.get(3)?;
                        let details_json: String = row.get(4)?;
                        Ok((event_id, event_type, user_id, app_id, details_json))
                    })
                    .map_err(|e| cortex_db::DbError::Internal(format!("query failed: {e}")))?;

                let mut events = Vec::new();
                for row_result in rows {
                    let (event_id, event_type, user_id, app_id, details_json) = row_result
                        .map_err(|e| {
                            cortex_db::DbError::Internal(format!("row read failed: {e}"))
                        })?;
                    let details: serde_json::Value =
                        serde_json::from_str(&details_json).unwrap_or(serde_json::json!({}));
                    events.push(AuditEvent {
                        event_id,
                        event_type,
                        user_id,
                        app_id,
                        details,
                    });
                }
                Ok(events)
            })
            .map_err(|e| ObservabilityError::Internal(format!("audit user query failed: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_db() -> cortex_db::Pool {
        let pool = cortex_db::Pool::open_in_memory().unwrap();
        cortex_db::run_migrations(&pool).unwrap();
        pool
    }

    #[test]
    fn append_and_count() {
        let pool = setup_db();
        let logger = AuditLogger::new(pool);
        assert_eq!(logger.count().unwrap(), 0);

        logger
            .append(&AuditEvent {
                event_id: "evt-001".to_string(),
                event_type: "auth.login".to_string(),
                user_id: Some("user-1".to_string()),
                app_id: None,
                details: serde_json::json!({"method": "password"}),
            })
            .unwrap();

        assert_eq!(logger.count().unwrap(), 1);
    }

    #[test]
    fn recent_returns_events_ordered() {
        let pool = setup_db();
        let logger = AuditLogger::new(pool);

        for i in 0..5 {
            logger
                .append(&AuditEvent {
                    event_id: format!("evt-{i:03}"),
                    event_type: "test.event".to_string(),
                    user_id: Some("user-1".to_string()),
                    app_id: None,
                    details: serde_json::json!({"seq": i}),
                })
                .unwrap();
        }

        let events = logger.recent(3).unwrap();
        assert_eq!(events.len(), 3);
        // Most recent first (evt-004, evt-003, evt-002)
        assert_eq!(events[0].event_id, "evt-004");
        assert_eq!(events[1].event_id, "evt-003");
        assert_eq!(events[2].event_id, "evt-002");
    }

    #[test]
    fn for_user_filters_correctly() {
        let pool = setup_db();
        let logger = AuditLogger::new(pool);

        logger
            .append(&AuditEvent {
                event_id: "evt-a".to_string(),
                event_type: "test".to_string(),
                user_id: Some("alice".to_string()),
                app_id: None,
                details: serde_json::json!({}),
            })
            .unwrap();
        logger
            .append(&AuditEvent {
                event_id: "evt-b".to_string(),
                event_type: "test".to_string(),
                user_id: Some("bob".to_string()),
                app_id: None,
                details: serde_json::json!({}),
            })
            .unwrap();

        let alice = logger.for_user("alice", 10).unwrap();
        assert_eq!(alice.len(), 1);
        assert_eq!(alice[0].event_id, "evt-a");

        let bob = logger.for_user("bob", 10).unwrap();
        assert_eq!(bob.len(), 1);
        assert_eq!(bob[0].event_id, "evt-b");
    }

    #[test]
    fn details_serialization_roundtrip() {
        let pool = setup_db();
        let logger = AuditLogger::new(pool);

        let details = serde_json::json!({
            "action": "file.write",
            "path": "docs/readme.md",
            "bytes": 1024
        });

        logger
            .append(&AuditEvent {
                event_id: "evt-detail".to_string(),
                event_type: "file.write".to_string(),
                user_id: Some("user-1".to_string()),
                app_id: Some("com.example.editor".to_string()),
                details: details.clone(),
            })
            .unwrap();

        let events = logger.recent(1).unwrap();
        assert_eq!(events[0].details, details);
        assert_eq!(events[0].app_id, Some("com.example.editor".to_string()));
    }
}
