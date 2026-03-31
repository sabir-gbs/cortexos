//! WebSocket handler.
//!
//! Manages the single `/ws` WebSocket endpoint for real-time
//! events, subscriptions, and streaming responses.
//!
//! This module defines the handler interface and session management
//! using the typed bus types from [`crate::bus`]. It is framework-agnostic:
//! the actual HTTP integration (axum, actix, etc.) will wrap these types.

use crate::bus::{BusCommand, BusEvent, BusMessage, BusMessageType, Channel, CommandBus};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Identifier for a connected WebSocket session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WsSessionId(pub String);

/// State tracked per connected WebSocket client.
#[derive(Debug)]
pub struct WsSession {
    /// Unique session identifier.
    pub id: WsSessionId,
    /// Authenticated user ID (set after handshake).
    pub user_id: Option<cortex_core::UserId>,
    /// Channels this session is subscribed to.
    pub subscriptions: HashSet<Channel>,
}

impl WsSession {
    /// Create a new unauthenticated session with a random ID.
    pub fn new() -> Self {
        Self {
            id: WsSessionId(uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string()),
            user_id: None,
            subscriptions: HashSet::new(),
        }
    }

    /// Create a session for a specific user.
    pub fn for_user(user_id: cortex_core::UserId) -> Self {
        Self {
            id: WsSessionId(uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext)).to_string()),
            user_id: Some(user_id),
            subscriptions: HashSet::new(),
        }
    }

    /// Subscribe to a channel. Returns true if newly added.
    pub fn subscribe(&mut self, channel: Channel) -> bool {
        self.subscriptions.insert(channel)
    }

    /// Unsubscribe from a channel. Returns true if it was present.
    pub fn unsubscribe(&mut self, channel: &Channel) -> bool {
        self.subscriptions.remove(channel)
    }

    /// Check if this session is subscribed to a given channel.
    pub fn is_subscribed(&self, channel: &Channel) -> bool {
        self.subscriptions.contains(channel)
    }
}

impl Default for WsSession {
    fn default() -> Self {
        Self::new()
    }
}

/// Client-to-server control messages (parsed from text frames).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsClientFrame {
    /// A typed command from the client.
    Command(BusCommand),
    /// Subscribe to a channel.
    Subscribe { channel: Channel },
    /// Unsubscribe from a channel.
    Unsubscribe { channel: Channel },
    /// Keep-alive ping.
    Ping,
}

/// Server-to-client frames sent over the WebSocket.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsServerFrame {
    /// A bus event forwarded to the client.
    Event(BusEvent),
    /// Response to a client command.
    Message(BusMessage),
    /// Keep-alive pong.
    Pong,
    /// Server-initiated error.
    Error { message: String },
}

/// Shared state for all WebSocket connections.
pub struct WsState {
    /// The command bus for pub/sub.
    pub bus: Arc<CommandBus>,
    /// Active sessions.
    pub sessions: Arc<RwLock<Vec<WsSession>>>,
}

impl WsState {
    /// Create new WebSocket state backed by the given command bus.
    pub fn new(bus: Arc<CommandBus>) -> Self {
        Self {
            bus,
            sessions: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Register a new session.
    pub async fn add_session(&self, session: WsSession) {
        let mut sessions = self.sessions.write().await;
        sessions.push(session);
    }

    /// Remove a session by ID.
    pub async fn remove_session(&self, id: &WsSessionId) {
        let mut sessions = self.sessions.write().await;
        sessions.retain(|s| s.id != *id);
    }
}

/// Process a single client frame and return the server response frames.
///
/// This is the core dispatch function. A real HTTP framework integration
/// would call this for each incoming WebSocket text message.
pub fn handle_client_frame(
    session: &mut WsSession,
    frame: WsClientFrame,
    bus: &CommandBus,
) -> Vec<WsServerFrame> {
    match frame {
        WsClientFrame::Ping => {
            vec![WsServerFrame::Pong]
        }

        WsClientFrame::Subscribe { channel } => {
            session.subscribe(channel.clone());
            let msg = BusMessage {
                id: format!(
                    "sub-{}",
                    uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext))
                ),
                message_type: BusMessageType::CommandResponse,
                channel: Some(channel),
                payload: Some(serde_json::json!({"subscribed": true})),
                error: None,
            };
            vec![WsServerFrame::Message(msg)]
        }

        WsClientFrame::Unsubscribe { channel } => {
            session.unsubscribe(&channel);
            let msg = BusMessage {
                id: format!(
                    "unsub-{}",
                    uuid::Uuid::new_v7(uuid::Timestamp::now(uuid::NoContext))
                ),
                message_type: BusMessageType::CommandResponse,
                channel: Some(channel),
                payload: Some(serde_json::json!({"subscribed": false})),
                error: None,
            };
            vec![WsServerFrame::Message(msg)]
        }

        WsClientFrame::Command(cmd) => {
            let response_id = cmd.id.clone();
            let event_name = command_to_event_name(&cmd.command);
            let channel = command_to_channel(&cmd.command);

            // Validate that we recognise the command; unknown commands
            // are rejected early so that callers get immediate feedback.
            if event_name == "unknown" {
                let err = BusMessage {
                    id: response_id,
                    message_type: BusMessageType::Error,
                    channel: Some(channel),
                    payload: None,
                    error: Some(cortex_core::CortexError {
                        error: cortex_core::CortexErrorBody {
                            code: "VAL_001".to_string(),
                            category: cortex_core::ErrorCategory::Validation,
                            message: format!("unknown command: {}", cmd.command),
                            details: serde_json::Value::Null,
                            retryable: false,
                            retry_after_ms: None,
                        },
                    }),
                };
                return vec![WsServerFrame::Message(err)];
            }

            // Route to the appropriate handler based on command type.
            match cmd.command.as_str() {
                // Window-manager commands -- resolved with the corresponding
                // past-tense event (e.g. window.move -> window.updated).
                "window.move" | "window.resize" | "window.focus" | "workspace.switch" => {
                    let resolved_event = wm_command_to_event(&cmd.command);

                    let event = BusEvent {
                        event: resolved_event.to_string(),
                        channel: channel.clone(),
                        payload: cmd.payload.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        correlation_id: Some(response_id.clone()),
                    };

                    let status = bus.publish_or_dead_letter(event, &cmd.id);

                    let payload = match status {
                        crate::bus::EventStatus::Published => {
                            serde_json::json!({
                                "dispatched": true,
                                "event": resolved_event,
                            })
                        }
                        crate::bus::EventStatus::DeadLetter => {
                            serde_json::json!({
                                "dispatched": false,
                                "event": resolved_event,
                                "reason": "dead_lettered",
                            })
                        }
                        _ => {
                            serde_json::json!({
                                "dispatched": false,
                                "event": resolved_event,
                            })
                        }
                    };

                    let ack = BusMessage {
                        id: response_id,
                        message_type: BusMessageType::CommandResponse,
                        channel: Some(channel),
                        payload: Some(payload),
                        error: None,
                    };

                    vec![WsServerFrame::Message(ack)]
                }

                // All other recognised commands (app.*, file.*, ai.*,
                // notification.*, settings.*) are dispatched generically.
                _ => {
                    let event = BusEvent {
                        event: event_name.to_string(),
                        channel: channel.clone(),
                        payload: cmd.payload.clone(),
                        timestamp: chrono::Utc::now().to_rfc3339(),
                        correlation_id: Some(response_id.clone()),
                    };

                    let status = bus.publish_or_dead_letter(event, &cmd.id);

                    let payload = match status {
                        crate::bus::EventStatus::Published => {
                            serde_json::json!({
                                "dispatched": true,
                                "event": event_name,
                            })
                        }
                        crate::bus::EventStatus::DeadLetter => {
                            serde_json::json!({
                                "dispatched": false,
                                "event": event_name,
                                "reason": "dead_lettered",
                            })
                        }
                        _ => {
                            serde_json::json!({
                                "dispatched": false,
                                "event": event_name,
                            })
                        }
                    };

                    let ack = BusMessage {
                        id: response_id,
                        message_type: BusMessageType::CommandResponse,
                        channel: Some(channel),
                        payload: Some(payload),
                        error: None,
                    };

                    vec![WsServerFrame::Message(ack)]
                }
            }
        }
    }
}

/// Map a command name to its corresponding event name.
fn command_to_event_name(command: &str) -> &'static str {
    match command {
        "app.launch" => crate::bus::EVENT_APP_LAUNCHED,
        "app.stop" => crate::bus::EVENT_APP_STOPPED,
        "app.suspend" => crate::bus::EVENT_APP_SUSPENDED,
        "app.resume" => crate::bus::EVENT_APP_RESUMED,
        "file.create" => crate::bus::EVENT_FILE_CREATED,
        "file.modify" => crate::bus::EVENT_FILE_MODIFIED,
        "file.delete" => crate::bus::EVENT_FILE_DELETED,
        "notification.create" => crate::bus::EVENT_NOTIFICATION_CREATED,
        "notification.dismiss" => crate::bus::EVENT_NOTIFICATION_DISMISSED,
        "settings.change" => crate::bus::EVENT_SETTINGS_CHANGED,
        "ai.chat_start" => crate::bus::EVENT_AI_CHAT_STARTED,
        "ai.chat_complete" => crate::bus::EVENT_AI_CHAT_COMPLETED,
        "window.open" => crate::bus::EVENT_WINDOW_CREATED,
        "window.close" => crate::bus::EVENT_WINDOW_CLOSED,
        "window.focus" => crate::bus::EVENT_WINDOW_FOCUSED,
        "window.move" => crate::bus::EVENT_WINDOW_UPDATED,
        "window.resize" => crate::bus::EVENT_WINDOW_UPDATED,
        "workspace.switch" => crate::bus::EVENT_WM_WORKSPACE_CHANGED,
        "wm.workspace.activate" => crate::bus::EVENT_WM_WORKSPACE_CHANGED,
        _ => "unknown",
    }
}

/// Map a WM-specific command to its past-tense event name.
fn wm_command_to_event(command: &str) -> &'static str {
    match command {
        "window.move" => crate::bus::EVENT_WINDOW_UPDATED,
        "window.resize" => crate::bus::EVENT_WINDOW_UPDATED,
        "window.focus" => crate::bus::EVENT_WINDOW_FOCUSED,
        "workspace.switch" => crate::bus::EVENT_WM_WORKSPACE_CHANGED,
        _ => "unknown",
    }
}

/// Map a command name to the channel it belongs to.
///
/// Window and workspace commands currently fall through to the
/// `System` channel alongside any other unrecognised prefix.
fn command_to_channel(command: &str) -> Channel {
    if command.starts_with("app.") {
        Channel::Apps
    } else if command.starts_with("file.") {
        Channel::Files
    } else if command.starts_with("ai.") {
        Channel::Ai
    } else if command.starts_with("notification.") {
        Channel::Notifications
    } else if command.starts_with("settings.") {
        Channel::Settings
    } else {
        Channel::System
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_bus() -> CommandBus {
        let pool = cortex_db::Pool::open_in_memory().unwrap();
        cortex_db::migration::run_migrations(&pool).unwrap();
        CommandBus::new(pool)
    }

    #[test]
    fn ws_client_frame_command_serialization() {
        let frame = WsClientFrame::Command(BusCommand {
            id: "cmd-1".to_string(),
            command: "app.launch".to_string(),
            payload: serde_json::json!({"app_id": "com.cortexos.notes"}),
        });
        let json = serde_json::to_string(&frame).unwrap();
        assert!(json.contains("\"type\":\"command\""));
        assert!(json.contains("\"command\":\"app.launch\""));

        let roundtrip: WsClientFrame = serde_json::from_str(&json).unwrap();
        if let WsClientFrame::Command(cmd) = roundtrip {
            assert_eq!(cmd.id, "cmd-1");
        } else {
            panic!("expected Command variant");
        }
    }

    #[test]
    fn ws_client_frame_subscribe_serialization() {
        let frame = WsClientFrame::Subscribe {
            channel: Channel::Apps,
        };
        let json = serde_json::to_string(&frame).unwrap();
        assert!(json.contains("\"type\":\"subscribe\""));

        let roundtrip: WsClientFrame = serde_json::from_str(&json).unwrap();
        if let WsClientFrame::Subscribe { channel } = roundtrip {
            assert_eq!(channel, Channel::Apps);
        } else {
            panic!("expected Subscribe variant");
        }
    }

    #[test]
    fn ws_server_frame_pong_serialization() {
        let frame = WsServerFrame::Pong;
        let json = serde_json::to_string(&frame).unwrap();
        assert!(json.contains("\"pong\""));
    }

    #[test]
    fn ws_server_frame_event_serialization() {
        let frame = WsServerFrame::Event(BusEvent {
            event: crate::bus::EVENT_APP_LAUNCHED.to_string(),
            channel: Channel::Apps,
            payload: serde_json::json!({"app_id": "com.cortexos.notes"}),
            timestamp: "2026-03-30T00:00:00Z".to_string(),
            correlation_id: None,
        });
        let json = serde_json::to_string(&frame).unwrap();
        assert!(json.contains("\"type\":\"event\""));
        assert!(json.contains("app.launched"));
    }

    #[test]
    fn handle_ping_returns_pong() {
        let mut session = WsSession::new();
        let bus = test_bus();
        let frames = handle_client_frame(&mut session, WsClientFrame::Ping, &bus);
        assert_eq!(frames.len(), 1);
        assert!(matches!(frames[0], WsServerFrame::Pong));
    }

    #[test]
    fn handle_subscribe_adds_channel() {
        let mut session = WsSession::new();
        let bus = test_bus();

        let frames = handle_client_frame(
            &mut session,
            WsClientFrame::Subscribe {
                channel: Channel::Apps,
            },
            &bus,
        );

        assert!(session.is_subscribed(&Channel::Apps));
        assert_eq!(frames.len(), 1);
    }

    #[test]
    fn handle_unsubscribe_removes_channel() {
        let mut session = WsSession::new();
        session.subscribe(Channel::Files);
        assert!(session.is_subscribed(&Channel::Files));

        let bus = test_bus();
        let frames = handle_client_frame(
            &mut session,
            WsClientFrame::Unsubscribe {
                channel: Channel::Files,
            },
            &bus,
        );

        assert!(!session.is_subscribed(&Channel::Files));
        assert_eq!(frames.len(), 1);
    }

    #[test]
    fn handle_command_publishes_event() {
        let mut session = WsSession::new();
        let bus = test_bus();

        let frames = handle_client_frame(
            &mut session,
            WsClientFrame::Command(BusCommand {
                id: "cmd-42".to_string(),
                command: "app.launch".to_string(),
                payload: serde_json::json!({"app_id": "com.cortexos.notes"}),
            }),
            &bus,
        );

        assert_eq!(frames.len(), 1);
        if let WsServerFrame::Message(msg) = &frames[0] {
            assert_eq!(msg.id, "cmd-42");
            assert_eq!(msg.message_type, BusMessageType::CommandResponse);
            let payload = msg.payload.as_ref().expect("payload should be present");
            assert_eq!(payload["dispatched"], true);
            assert_eq!(payload["event"], "app.launched");
        } else {
            panic!("expected Message variant");
        }
    }

    #[test]
    fn handle_wm_command_dispatches() {
        let mut session = WsSession::new();
        let bus = test_bus();

        let frames = handle_client_frame(
            &mut session,
            WsClientFrame::Command(BusCommand {
                id: "cmd-wm-1".to_string(),
                command: "window.move".to_string(),
                payload: serde_json::json!({"window_id": "w-1", "x": 100, "y": 200}),
            }),
            &bus,
        );

        assert_eq!(frames.len(), 1);
        if let WsServerFrame::Message(msg) = &frames[0] {
            assert_eq!(msg.id, "cmd-wm-1");
            assert_eq!(msg.message_type, BusMessageType::CommandResponse);
            let payload = msg.payload.as_ref().expect("payload should be present");
            assert_eq!(payload["dispatched"], true);
            assert_eq!(payload["event"], "window.updated");
        } else {
            panic!("expected Message variant");
        }
    }

    #[test]
    fn handle_unknown_command_returns_error() {
        let mut session = WsSession::new();
        let bus = test_bus();

        let frames = handle_client_frame(
            &mut session,
            WsClientFrame::Command(BusCommand {
                id: "cmd-bad".to_string(),
                command: "foo.bar".to_string(),
                payload: serde_json::json!({}),
            }),
            &bus,
        );

        assert_eq!(frames.len(), 1);
        if let WsServerFrame::Message(msg) = &frames[0] {
            assert_eq!(msg.id, "cmd-bad");
            assert_eq!(msg.message_type, BusMessageType::Error);
            assert!(msg.error.is_some());
        } else {
            panic!("expected Message variant");
        }
    }

    #[test]
    fn session_default_creates_new_id() {
        let s1 = WsSession::default();
        let s2 = WsSession::default();
        assert_ne!(s1.id, s2.id);
        assert!(s1.user_id.is_none());
        assert!(s1.subscriptions.is_empty());
    }

    #[test]
    fn command_to_event_name_mapping() {
        assert_eq!(
            command_to_event_name("app.launch"),
            crate::bus::EVENT_APP_LAUNCHED
        );
        assert_eq!(
            command_to_event_name("app.stop"),
            crate::bus::EVENT_APP_STOPPED
        );
        assert_eq!(
            command_to_event_name("file.create"),
            crate::bus::EVENT_FILE_CREATED
        );
        assert_eq!(
            command_to_event_name("file.delete"),
            crate::bus::EVENT_FILE_DELETED
        );
        assert_eq!(
            command_to_event_name("settings.change"),
            crate::bus::EVENT_SETTINGS_CHANGED
        );
        assert_eq!(
            command_to_event_name("window.move"),
            crate::bus::EVENT_WINDOW_UPDATED
        );
        assert_eq!(
            command_to_event_name("window.resize"),
            crate::bus::EVENT_WINDOW_UPDATED
        );
        assert_eq!(
            command_to_event_name("window.focus"),
            crate::bus::EVENT_WINDOW_FOCUSED
        );
        assert_eq!(
            command_to_event_name("workspace.switch"),
            crate::bus::EVENT_WM_WORKSPACE_CHANGED
        );
        assert_eq!(
            command_to_event_name("wm.workspace.activate"),
            crate::bus::EVENT_WM_WORKSPACE_CHANGED
        );
        assert_eq!(command_to_event_name("unknown.cmd"), "unknown");
    }

    #[test]
    fn command_to_channel_mapping() {
        assert_eq!(command_to_channel("app.launch"), Channel::Apps);
        assert_eq!(command_to_channel("file.create"), Channel::Files);
        assert_eq!(command_to_channel("ai.chat_start"), Channel::Ai);
        assert_eq!(
            command_to_channel("notification.create"),
            Channel::Notifications
        );
        assert_eq!(command_to_channel("settings.change"), Channel::Settings);
        assert_eq!(command_to_channel("system.health"), Channel::System);
        assert_eq!(command_to_channel("window.move"), Channel::System);
        assert_eq!(command_to_channel("workspace.switch"), Channel::System);
    }

    #[tokio::test]
    async fn ws_state_add_remove_session() {
        let bus = Arc::new(test_bus());
        let state = WsState::new(bus);

        let mut session = WsSession::new();
        let session_id = session.id.clone();
        session.subscribe(Channel::Apps);

        state.add_session(session).await;
        {
            let sessions = state.sessions.read().await;
            assert_eq!(sessions.len(), 1);
        }

        state.remove_session(&session_id).await;
        {
            let sessions = state.sessions.read().await;
            assert!(sessions.is_empty());
        }
    }
}
