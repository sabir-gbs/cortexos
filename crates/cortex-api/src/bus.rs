//! Typed command/event bus for CortexOS.
//!
//! Implements the canonical event names from spec 10. All events are
//! typed Rust structs with Serialize/Deserialize.
//!
//! Idempotency and event persistence are backed by SQLite so that
//! duplicates are rejected across server restarts and failed events
//! are captured for dead-letter recovery.

use cortex_core::Timestamp;
use cortex_db::error::DbError;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

// === Canonical event names ===
//
// Naming convention (per CR-013 reconciliation):
//   - WM events:   window.created, window.updated, window.closed, window.focused,
//                  wm.workspace.changed, wm.focus.changed
//   - App events:  app.launched, app.stopped, app.crashed, ...
//   - Notify events: notification.created, notification.dismissed, notification.read, notification.all_read
//   - Settings:    settings.changed
//   - File events: file.created, file.modified, file.deleted, ...

pub const EVENT_APP_LAUNCHED: &str = "app.launched";
pub const EVENT_APP_STOPPED: &str = "app.stopped";
pub const EVENT_APP_CRASHED: &str = "app.crashed";
pub const EVENT_APP_SUSPENDED: &str = "app.suspended";
pub const EVENT_APP_RESUMED: &str = "app.resumed";
pub const EVENT_FILE_CREATED: &str = "file.created";
pub const EVENT_FILE_MODIFIED: &str = "file.modified";
pub const EVENT_FILE_DELETED: &str = "file.deleted";
pub const EVENT_SETTINGS_CHANGED: &str = "settings.changed";
pub const EVENT_NOTIFICATION_CREATED: &str = "notification.created";
pub const EVENT_NOTIFICATION_DISMISSED: &str = "notification.dismissed";
pub const EVENT_NOTIFICATION_READ: &str = "notification.read";
pub const EVENT_NOTIFICATION_ALL_READ: &str = "notification.all_read";
pub const EVENT_WINDOW_CREATED: &str = "window.created";
pub const EVENT_WINDOW_UPDATED: &str = "window.updated";
pub const EVENT_WINDOW_CLOSED: &str = "window.closed";
pub const EVENT_WINDOW_FOCUSED: &str = "window.focused";
pub const EVENT_WM_WORKSPACE_CHANGED: &str = "wm.workspace.changed";
pub const EVENT_WM_FOCUS_CHANGED: &str = "wm.focus.changed";
pub const EVENT_AI_CHAT_STARTED: &str = "ai.chat_started";
pub const EVENT_AI_CHAT_COMPLETED: &str = "ai.chat_completed";
pub const EVENT_AI_CHAT_STREAMING: &str = "ai.chat_streaming";
pub const EVENT_SYSTEM_HEALTH: &str = "system.health";
pub const EVENT_SESSION_CREATED: &str = "session.created";
pub const EVENT_SESSION_DESTROYED: &str = "session.destroyed";

/// Canonical WebSocket channels from spec 02 section 9.2.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Channel {
    System,
    Apps,
    Files,
    Ai,
    Notifications,
    Settings,
}

/// Typed bus event envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusEvent {
    /// Canonical event name.
    pub event: String,
    /// Channel this event belongs to.
    pub channel: Channel,
    /// Event-specific payload (typed per event).
    pub payload: serde_json::Value,
    /// ISO 8601 timestamp.
    pub timestamp: Timestamp,
    /// Optional correlation ID for tracing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

/// Client -> Server command envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusCommand {
    /// Client-generated message ID for correlation.
    pub id: String,
    /// Command type name.
    pub command: String,
    /// Command payload (typed per command).
    pub payload: serde_json::Value,
}

/// Server -> Client message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusMessage {
    /// Message ID (echoes client command ID or server-generated).
    pub id: String,
    /// Message type.
    pub message_type: BusMessageType,
    /// Channel (for events).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<Channel>,
    /// Payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    /// Error (for error message type).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<cortex_core::CortexError>,
}

/// Message type discriminator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BusMessageType {
    Event,
    CommandResponse,
    Error,
    Pong,
}

/// Durable idempotency record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyRecord {
    /// The command ID that was processed.
    pub command_id: String,
    /// When this record was created.
    pub processed_at: Timestamp,
}

/// Status of a persisted event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventStatus {
    Published,
    Failed,
    DeadLetter,
}

impl EventStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Published => "published",
            Self::Failed => "failed",
            Self::DeadLetter => "dead_letter",
        }
    }
}

/// A dead-lettered event that failed processing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterEvent {
    /// Database row ID.
    pub id: i64,
    /// Original command ID.
    pub command_id: String,
    /// Event name.
    pub event_name: String,
    /// Channel.
    pub channel: String,
    /// Payload JSON.
    pub payload: String,
    /// Correlation ID if any.
    pub correlation_id: Option<String>,
    /// Original timestamp.
    pub timestamp: String,
    /// Error message.
    pub error_message: String,
    /// When it was processed.
    pub processed_at: String,
}

/// Command bus with publish/subscribe semantics and SQLite-backed durability.
pub struct CommandBus {
    tx: broadcast::Sender<BusEvent>,
    pool: cortex_db::Pool,
}

/// Errors produced by the command bus.
#[derive(Debug, thiserror::Error)]
pub enum BusError {
    #[error("duplicate command: {0}")]
    Duplicate(String),
    #[error("database error: {0}")]
    Db(#[from] DbError),
    #[error("publish failed: {0}")]
    PublishFailed(String),
}

impl CommandBus {
    /// Create a new command bus backed by the given database pool.
    ///
    /// The broadcast channel has a capacity of 256 entries.
    /// Idempotency state is persisted to `command_bus_idempotency`.
    pub fn new(pool: cortex_db::Pool) -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx, pool }
    }

    /// Publish an event to all subscribers and persist it to SQLite.
    ///
    /// If the idempotency check detects a duplicate command, it returns
    /// `BusError::Duplicate`. Otherwise the event is recorded with status
    /// `published` and broadcast to in-process subscribers.
    pub fn publish(&self, event: BusEvent, command_id: &str) -> Result<(), BusError> {
        let event_name = event.event.clone();
        let channel =
            serde_json::to_string(&event.channel).unwrap_or_else(|_| "\"system\"".to_string());
        let payload = event.payload.to_string();
        let correlation_id = event.correlation_id.clone();
        let timestamp = event.timestamp.clone();

        // Durable idempotency + event insert in one transaction.
        self.pool.write(|conn| {
            // Check for duplicate
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM command_bus_idempotency WHERE command_id = ?1",
                    [command_id],
                    |row| row.get(0),
                )
                .map_err(|e| DbError::Query(e.to_string()))?;

            if exists {
                return Err(DbError::Query(format!(
                    "duplicate command: {command_id}"
                )));
            }

            // Insert idempotency record
            conn.execute(
                "INSERT INTO command_bus_idempotency (command_id, event_name, channel) VALUES (?1, ?2, ?3)",
                rusqlite::params![command_id, event_name, channel],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

            // Insert event record
            conn.execute(
                "INSERT INTO command_bus_events (command_id, event_name, channel, payload, correlation_id, timestamp, status) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'published')",
                rusqlite::params![command_id, event_name, channel, payload, correlation_id, timestamp],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

            Ok(())
        }).map_err(|e| {
            if e.to_string().contains("duplicate command") {
                BusError::Duplicate(command_id.to_string())
            } else {
                BusError::Db(e)
            }
        })?;

        // Broadcast to in-process subscribers
        if let Err(e) = self.tx.send(event) {
            tracing::warn!(target: "command_bus", "no active subscribers for event: {e}");
        }

        Ok(())
    }

    /// Publish an event but capture failures as dead letters instead of
    /// returning an error. Returns the event status.
    pub fn publish_or_dead_letter(&self, event: BusEvent, command_id: &str) -> EventStatus {
        match self.publish(event.clone(), command_id) {
            Ok(()) => EventStatus::Published,
            Err(BusError::Duplicate(_)) => EventStatus::Published, // already processed
            Err(e) => {
                let error_msg = e.to_string();
                tracing::error!(
                    target: "command_bus",
                    command_id,
                    error = %error_msg,
                    "event publish failed, recording dead letter"
                );
                if let Err(de) = self.record_dead_letter(&event, command_id, &error_msg) {
                    tracing::error!(target: "command_bus", "failed to record dead letter: {de}");
                }
                EventStatus::DeadLetter
            }
        }
    }

    /// Record a failed event as a dead letter.
    fn record_dead_letter(
        &self,
        event: &BusEvent,
        command_id: &str,
        error_message: &str,
    ) -> Result<(), BusError> {
        let event_name = event.event.clone();
        let channel =
            serde_json::to_string(&event.channel).unwrap_or_else(|_| "\"system\"".to_string());
        let payload = event.payload.to_string();
        let correlation_id = event.correlation_id.clone();
        let timestamp = event.timestamp.clone();

        self.pool.write(|conn| {
            // Try to insert idempotency record if not there yet
            let exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) > 0 FROM command_bus_idempotency WHERE command_id = ?1",
                    [command_id],
                    |row| row.get(0),
                )
                .map_err(|e| DbError::Query(e.to_string()))?;

            if !exists {
                conn.execute(
                    "INSERT INTO command_bus_idempotency (command_id, event_name, channel) VALUES (?1, ?2, ?3)",
                    rusqlite::params![command_id, event_name, channel],
                )
                .map_err(|e| DbError::Query(e.to_string()))?;
            }

            // Upsert event as dead_letter
            conn.execute(
                "INSERT INTO command_bus_events (command_id, event_name, channel, payload, correlation_id, timestamp, status, error_message) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'dead_letter', ?7)",
                rusqlite::params![command_id, event_name, channel, payload, correlation_id, timestamp, error_message],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

            Ok(())
        })?;

        Ok(())
    }

    /// Subscribe to events. Returns a receiver.
    pub fn subscribe(&self) -> broadcast::Receiver<BusEvent> {
        self.tx.subscribe()
    }

    /// Check idempotency without publishing. Returns true if duplicate.
    pub fn check_idempotency(&self, command_id: &str) -> bool {
        self.pool
            .read(|conn| {
                let exists: bool = conn
                    .query_row(
                        "SELECT COUNT(*) > 0 FROM command_bus_idempotency WHERE command_id = ?1",
                        [command_id],
                        |row| row.get(0),
                    )
                    .map_err(|e| DbError::Query(e.to_string()))?;
                Ok(exists)
            })
            .unwrap_or(false)
    }

    /// Record a processed command (inserts into idempotency table only).
    pub fn record_processed(
        &self,
        command_id: &str,
        event_name: &str,
        channel: &str,
    ) -> Result<(), BusError> {
        self.pool.write(|conn| {
            conn.execute(
                "INSERT OR IGNORE INTO command_bus_idempotency (command_id, event_name, channel) VALUES (?1, ?2, ?3)",
                rusqlite::params![command_id, event_name, channel],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;
            Ok(())
        })?;
        Ok(())
    }

    /// Retrieve all dead-lettered events for inspection / replay.
    pub fn list_dead_letters(&self) -> Result<Vec<DeadLetterEvent>, BusError> {
        self.pool.read(|conn| {
            let mut stmt = conn
                .prepare(
                    "SELECT id, command_id, event_name, channel, payload, correlation_id, timestamp, error_message, processed_at \
                     FROM command_bus_events WHERE status = 'dead_letter' ORDER BY processed_at DESC",
                )
                .map_err(|e| DbError::Query(e.to_string()))?;

            let rows = stmt
                .query_map([], |row| {
                    Ok(DeadLetterEvent {
                        id: row.get(0)?,
                        command_id: row.get(1)?,
                        event_name: row.get(2)?,
                        channel: row.get(3)?,
                        payload: row.get(4)?,
                        correlation_id: row.get(5)?,
                        timestamp: row.get(6)?,
                        error_message: row.get(7)?,
                        processed_at: row.get(8)?,
                    })
                })
                .map_err(|e| DbError::Query(e.to_string()))?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row.map_err(|e| DbError::Query(e.to_string()))?);
            }
            Ok(result)
        }).map_err(BusError::Db)
    }

    /// Replay a dead-lettered event by command_id — resets its status
    /// to `published` and re-broadcasts it.
    pub fn replay_dead_letter(&self, command_id: &str) -> Result<(), BusError> {
        let event = self.pool.write(|conn| {
            let (event_name, channel_str, payload_str, correlation_id, timestamp): (
                String, String, String, Option<String>, String,
            ) = conn
                .query_row(
                    "SELECT event_name, channel, payload, correlation_id, timestamp \
                     FROM command_bus_events WHERE command_id = ?1 AND status = 'dead_letter'",
                    [command_id],
                    |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
                )
                .map_err(|e| DbError::Query(e.to_string()))?;

            // Update status
            conn.execute(
                "UPDATE command_bus_events SET status = 'published', error_message = NULL WHERE command_id = ?1",
                [command_id],
            )
            .map_err(|e| DbError::Query(e.to_string()))?;

            Ok::<_, DbError>(BusEvent {
                event: event_name,
                channel: serde_json::from_str(&channel_str).unwrap_or(Channel::System),
                payload: serde_json::from_str(&payload_str).unwrap_or(serde_json::json!({})),
                timestamp,
                correlation_id,
            })
        })?;

        if let Err(e) = self.tx.send(event) {
            tracing::warn!(target: "command_bus", "replay broadcast failed: {e}");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_db::migration::run_migrations;

    fn test_pool() -> cortex_db::Pool {
        let pool = cortex_db::Pool::open_in_memory().unwrap();
        run_migrations(&pool).unwrap();
        pool
    }

    fn make_event(name: &str) -> BusEvent {
        BusEvent {
            event: name.to_string(),
            channel: Channel::Apps,
            payload: serde_json::json!({"app_id": "com.cortexos.notes"}),
            timestamp: chrono::Utc::now().to_rfc3339(),
            correlation_id: None,
        }
    }

    #[test]
    fn bus_event_serialization() {
        let event = BusEvent {
            event: EVENT_APP_LAUNCHED.to_string(),
            channel: Channel::Apps,
            payload: serde_json::json!({"app_id": "com.cortexos.notes"}),
            timestamp: "2026-03-30T00:00:00Z".to_string(),
            correlation_id: Some("req-123".to_string()),
        };
        let json = serde_json::to_string(&event).unwrap();
        let roundtrip: BusEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.event, EVENT_APP_LAUNCHED);
        assert_eq!(roundtrip.channel, Channel::Apps);
    }

    #[test]
    fn bus_command_serialization() {
        let cmd = BusCommand {
            id: "cmd-1".to_string(),
            command: "app.launch".to_string(),
            payload: serde_json::json!({"app_id": "com.cortexos.calculator"}),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let roundtrip: BusCommand = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip.id, "cmd-1");
        assert_eq!(roundtrip.command, "app.launch");
    }

    #[test]
    fn bus_message_serialization() {
        let msg = BusMessage {
            id: "msg-1".to_string(),
            message_type: BusMessageType::Event,
            channel: Some(Channel::Apps),
            payload: Some(serde_json::json!({"test": true})),
            error: None,
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"message_type\":\"event\""));
    }

    #[test]
    fn channel_serialization() {
        assert_eq!(
            serde_json::to_string(&Channel::System).unwrap(),
            "\"system\""
        );
        assert_eq!(serde_json::to_string(&Channel::Apps).unwrap(), "\"apps\"");
        assert_eq!(serde_json::to_string(&Channel::Files).unwrap(), "\"files\"");
    }

    #[test]
    fn publish_persists_and_rejects_duplicate() {
        let pool = test_pool();
        let bus = CommandBus::new(pool.clone());

        let event = make_event(EVENT_APP_LAUNCHED);
        bus.publish(event, "cmd-100").unwrap();

        // Duplicate should fail
        let event2 = make_event(EVENT_APP_LAUNCHED);
        let result = bus.publish(event2, "cmd-100");
        assert!(result.is_err());
        match result {
            Err(BusError::Duplicate(id)) => assert_eq!(id, "cmd-100"),
            _ => panic!("expected Duplicate error"),
        }
    }

    #[test]
    fn check_idempotency() {
        let pool = test_pool();
        let bus = CommandBus::new(pool);

        assert!(!bus.check_idempotency("cmd-200"));
        bus.record_processed("cmd-200", "test.event", "system")
            .unwrap();
        assert!(bus.check_idempotency("cmd-200"));
    }

    #[test]
    fn dead_letter_recording_and_retrieval() {
        let pool = test_pool();
        let bus = CommandBus::new(pool);

        // Publish one successfully
        let event = make_event(EVENT_APP_LAUNCHED);
        let status = bus.publish_or_dead_letter(event, "cmd-300");
        assert_eq!(status, EventStatus::Published);

        // Verify no dead letters
        let dead = bus.list_dead_letters().unwrap();
        assert!(dead.is_empty());
    }

    #[test]
    fn replay_dead_letter() {
        let pool = test_pool();
        let bus = CommandBus::new(pool.clone());

        let event = make_event(EVENT_FILE_CREATED);
        bus.publish(event.clone(), "cmd-400").unwrap();

        // Manually insert a dead letter to simulate failure
        pool.write(|conn| {
            conn.execute(
                "INSERT INTO command_bus_idempotency (command_id, event_name, channel) VALUES ('cmd-dl-1', 'test.event', '\"system\"')",
                [],
            ).map_err(|e| DbError::Query(e.to_string()))?;
            conn.execute(
                "INSERT INTO command_bus_events (command_id, event_name, channel, payload, timestamp, status, error_message) \
                 VALUES ('cmd-dl-1', 'test.event', '\"system\"', '{}', '2026-03-30T00:00:00Z', 'dead_letter', 'something broke')",
                [],
            ).map_err(|e| DbError::Query(e.to_string()))?;
            Ok(())
        }).unwrap();

        let dead = bus.list_dead_letters().unwrap();
        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].command_id, "cmd-dl-1");
        assert_eq!(dead[0].error_message, "something broke");

        // Replay it
        bus.replay_dead_letter("cmd-dl-1").unwrap();

        let dead = bus.list_dead_letters().unwrap();
        assert!(dead.is_empty());
    }

    #[test]
    fn canonical_event_names_are_consistent() {
        assert!(EVENT_APP_LAUNCHED.starts_with("app."));
        assert!(EVENT_FILE_CREATED.starts_with("file."));
        assert!(EVENT_SETTINGS_CHANGED.starts_with("settings."));
        assert!(EVENT_NOTIFICATION_CREATED.starts_with("notification."));
        assert!(EVENT_WINDOW_CREATED.starts_with("window."));
        assert!(EVENT_AI_CHAT_STARTED.starts_with("ai."));
        assert!(EVENT_SYSTEM_HEALTH.starts_with("system."));
        assert!(EVENT_SESSION_CREATED.starts_with("session."));
    }

    #[test]
    fn subscribe_receives_events() {
        let pool = test_pool();
        let bus = CommandBus::new(pool);

        let mut rx = bus.subscribe();
        let event = make_event(EVENT_APP_LAUNCHED);
        bus.publish(event, "cmd-500").unwrap();

        let received = rx.try_recv().unwrap();
        assert_eq!(received.event, EVENT_APP_LAUNCHED);
    }
}
