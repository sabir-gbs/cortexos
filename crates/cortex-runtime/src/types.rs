//! Runtime domain types.
//!
//! Defines the app lifecycle state machine, instance tracking,
//! and lifecycle event types used across the runtime crate.

use cortex_core::{AppId, AppInstanceId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// AppState
// ---------------------------------------------------------------------------

/// Current state of an app instance in the lifecycle state machine.
///
/// Valid transitions:
/// - `Starting` -> `Running` (started successfully)
/// - `Starting` -> `Crashed` (failed to start)
/// - `Running` -> `Suspended` (OS suspends for resources)
/// - `Running` -> `Stopped` (user closes or app exits)
/// - `Running` -> `Crashed` (unhandled error)
/// - `Suspended` -> `Running` (resume)
/// - `Suspended` -> `Stopped` (kill while suspended)
/// - `Crashed` -> `Starting` (auto-retry, once only)
/// - `Crashed` -> `Stopped` (user dismisses or retry failed)
/// - `Stopped` -> `Starting` (re-launch)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    /// The app is being loaded and initialized.
    Starting,
    /// The app is running and interactive.
    Running,
    /// The app has been suspended by the OS (e.g. for resource management).
    Suspended,
    /// The app is in the process of stopping.
    Stopping,
    /// The app has been stopped and resources freed.
    Stopped,
    /// The app crashed due to an unhandled error.
    Crashed,
}

impl AppState {
    /// Returns `true` if this state represents an active (non-terminal) instance.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Starting | Self::Running | Self::Suspended | Self::Stopping
        )
    }

    /// Returns `true` if this state represents a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped | Self::Crashed)
    }

    /// Attempt a valid state transition. Returns `Ok(next)` on success,
    /// or `Err(self)` if the transition is not allowed.
    pub fn transition_to(self, next: AppState) -> std::result::Result<AppState, AppState> {
        match (self, next) {
            // Starting -> Running
            (AppState::Starting, AppState::Running) => Ok(AppState::Running),
            // Starting -> Crashed
            (AppState::Starting, AppState::Crashed) => Ok(AppState::Crashed),
            // Running -> Suspended
            (AppState::Running, AppState::Suspended) => Ok(AppState::Suspended),
            // Running -> Stopped
            (AppState::Running, AppState::Stopped) => Ok(AppState::Stopped),
            // Running -> Crashed
            (AppState::Running, AppState::Crashed) => Ok(AppState::Crashed),
            // Suspended -> Running
            (AppState::Suspended, AppState::Running) => Ok(AppState::Running),
            // Suspended -> Stopped
            (AppState::Suspended, AppState::Stopped) => Ok(AppState::Stopped),
            // Crashed -> Starting (auto-retry)
            (AppState::Crashed, AppState::Starting) => Ok(AppState::Starting),
            // Crashed -> Stopped
            (AppState::Crashed, AppState::Stopped) => Ok(AppState::Stopped),
            // Stopped -> Starting (re-launch)
            (AppState::Stopped, AppState::Starting) => Ok(AppState::Starting),
            // Any state -> Stopping (graceful shutdown initiated)
            (_, AppState::Stopping) => Ok(AppState::Stopping),
            // Stopping -> Stopped
            (AppState::Stopping, AppState::Stopped) => Ok(AppState::Stopped),
            // All other transitions are invalid
            _ => Err(self),
        }
    }
}

// ---------------------------------------------------------------------------
// AppInstance
// ---------------------------------------------------------------------------

/// A running (or previously running) app instance.
///
/// Tracks the full lifecycle of a single invocation of an application,
/// from launch through to stop or crash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInstance {
    /// Unique instance identifier.
    pub instance_id: AppInstanceId,
    /// The app being instantiated.
    pub app_id: AppId,
    /// The user who launched the app.
    pub user_id: UserId,
    /// Current state of the instance.
    pub state: AppState,
    /// Window manager identifier for this instance's window, if assigned.
    pub window_id: Option<String>,
    /// ISO 8601 timestamp when the instance was launched.
    pub launched_at: Option<Timestamp>,
    /// ISO 8601 timestamp when the instance was stopped or crashed.
    pub stopped_at: Option<Timestamp>,
}

impl AppInstance {
    /// Create a new `AppInstance` in the `Starting` state.
    pub fn new(instance_id: AppInstanceId, app_id: AppId, user_id: UserId) -> Self {
        Self {
            instance_id,
            app_id,
            user_id,
            state: AppState::Starting,
            window_id: None,
            launched_at: None,
            stopped_at: None,
        }
    }
}

// ---------------------------------------------------------------------------
// AppLifecycleEvent
// ---------------------------------------------------------------------------

/// Events emitted during app lifecycle transitions.
///
/// These events are broadcast via the command bus so that other subsystems
/// (shell, notifications, observability) can react to app state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "event", content = "payload")]
pub enum AppLifecycleEvent {
    /// An app instance was launched.
    Launched {
        /// The app that was launched.
        app_id: AppId,
        /// The new instance identifier.
        instance_id: AppInstanceId,
        /// The user who initiated the launch.
        user_id: UserId,
    },
    /// A running app instance was suspended.
    Suspended {
        /// The instance that was suspended.
        instance_id: AppInstanceId,
        /// The app identifier.
        app_id: AppId,
    },
    /// A suspended app instance was resumed.
    Resumed {
        /// The instance that was resumed.
        instance_id: AppInstanceId,
        /// The app identifier.
        app_id: AppId,
    },
    /// An app instance was stopped (gracefully or by user).
    Stopped {
        /// The instance that was stopped.
        instance_id: AppInstanceId,
        /// The app identifier.
        app_id: AppId,
    },
    /// An app instance crashed.
    Crashed {
        /// The instance that crashed.
        instance_id: AppInstanceId,
        /// The app identifier.
        app_id: AppId,
        /// The reason for the crash.
        reason: String,
    },
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- AppState transition tests ------------------------------------------

    #[test]
    fn app_state_starting_to_running() {
        assert_eq!(
            AppState::Starting.transition_to(AppState::Running),
            Ok(AppState::Running)
        );
    }

    #[test]
    fn app_state_starting_to_crashed() {
        assert_eq!(
            AppState::Starting.transition_to(AppState::Crashed),
            Ok(AppState::Crashed)
        );
    }

    #[test]
    fn app_state_running_to_stopped() {
        assert_eq!(
            AppState::Running.transition_to(AppState::Stopped),
            Ok(AppState::Stopped)
        );
    }

    #[test]
    fn app_state_running_to_crashed() {
        assert_eq!(
            AppState::Running.transition_to(AppState::Crashed),
            Ok(AppState::Crashed)
        );
    }

    #[test]
    fn app_state_running_to_suspended() {
        assert_eq!(
            AppState::Running.transition_to(AppState::Suspended),
            Ok(AppState::Suspended)
        );
    }

    #[test]
    fn app_state_suspended_to_running() {
        assert_eq!(
            AppState::Suspended.transition_to(AppState::Running),
            Ok(AppState::Running)
        );
    }

    #[test]
    fn app_state_suspended_to_stopped() {
        assert_eq!(
            AppState::Suspended.transition_to(AppState::Stopped),
            Ok(AppState::Stopped)
        );
    }

    #[test]
    fn app_state_crashed_to_starting() {
        assert_eq!(
            AppState::Crashed.transition_to(AppState::Starting),
            Ok(AppState::Starting)
        );
    }

    #[test]
    fn app_state_crashed_to_stopped() {
        assert_eq!(
            AppState::Crashed.transition_to(AppState::Stopped),
            Ok(AppState::Stopped)
        );
    }

    #[test]
    fn app_state_stopped_to_starting() {
        assert_eq!(
            AppState::Stopped.transition_to(AppState::Starting),
            Ok(AppState::Starting)
        );
    }

    #[test]
    fn app_state_invalid_transition_running_to_starting() {
        assert_eq!(
            AppState::Running.transition_to(AppState::Starting),
            Err(AppState::Running)
        );
    }

    #[test]
    fn app_state_invalid_transition_stopped_to_running() {
        assert_eq!(
            AppState::Stopped.transition_to(AppState::Running),
            Err(AppState::Stopped)
        );
    }

    #[test]
    fn app_state_any_to_stopping() {
        for state in [
            AppState::Starting,
            AppState::Running,
            AppState::Suspended,
            AppState::Stopped,
            AppState::Crashed,
        ] {
            assert_eq!(
                state.transition_to(AppState::Stopping),
                Ok(AppState::Stopping)
            );
        }
    }

    #[test]
    fn app_state_stopping_to_stopped() {
        assert_eq!(
            AppState::Stopping.transition_to(AppState::Stopped),
            Ok(AppState::Stopped)
        );
    }

    // -- AppState helper tests ---------------------------------------------

    #[test]
    fn app_state_is_active() {
        assert!(AppState::Starting.is_active());
        assert!(AppState::Running.is_active());
        assert!(AppState::Suspended.is_active());
        assert!(AppState::Stopping.is_active());
        assert!(!AppState::Stopped.is_active());
        assert!(!AppState::Crashed.is_active());
    }

    #[test]
    fn app_state_is_terminal() {
        assert!(AppState::Stopped.is_terminal());
        assert!(AppState::Crashed.is_terminal());
        assert!(!AppState::Starting.is_terminal());
        assert!(!AppState::Running.is_terminal());
        assert!(!AppState::Suspended.is_terminal());
        assert!(!AppState::Stopping.is_terminal());
    }

    // -- AppInstance tests -------------------------------------------------

    #[test]
    fn app_instance_new_starts_in_starting_state() {
        let instance = AppInstance::new(
            AppInstanceId(uuid::Uuid::new_v7(uuid::timestamp::Timestamp::now(
                uuid::NoContext,
            ))),
            AppId("com.cortexos.calculator".into()),
            UserId(uuid::Uuid::new_v7(uuid::timestamp::Timestamp::now(
                uuid::NoContext,
            ))),
        );

        assert_eq!(instance.state, AppState::Starting);
        assert!(instance.window_id.is_none());
        assert!(instance.launched_at.is_none());
        assert!(instance.stopped_at.is_none());
    }

    #[test]
    fn app_instance_construction() {
        let instance_id = AppInstanceId(uuid::Uuid::new_v7(uuid::timestamp::Timestamp::now(
            uuid::NoContext,
        )));
        let app_id = AppId("com.cortexos.notes".into());
        let user_id = UserId(uuid::Uuid::new_v7(uuid::timestamp::Timestamp::now(
            uuid::NoContext,
        )));
        let launched_at = "2026-03-30T12:00:00Z".to_string();

        let instance = AppInstance {
            instance_id: instance_id.clone(),
            app_id: app_id.clone(),
            user_id: user_id.clone(),
            state: AppState::Running,
            window_id: Some("win-123".to_string()),
            launched_at: Some(launched_at.clone()),
            stopped_at: None,
        };

        assert_eq!(instance.instance_id, instance_id);
        assert_eq!(instance.app_id, app_id);
        assert_eq!(instance.user_id, user_id);
        assert_eq!(instance.state, AppState::Running);
        assert_eq!(instance.window_id.as_deref(), Some("win-123"));
        assert_eq!(instance.launched_at.as_deref(), Some(launched_at.as_str()));
        assert!(instance.stopped_at.is_none());
    }

    // -- AppLifecycleEvent serialization tests -----------------------------

    #[test]
    fn app_lifecycle_event_launched_serialization() {
        let event = AppLifecycleEvent::Launched {
            app_id: AppId("com.cortexos.calculator".into()),
            instance_id: AppInstanceId(
                uuid::Uuid::parse_str("01968c8a-0000-7000-8000-000000000001").unwrap(),
            ),
            user_id: UserId(uuid::Uuid::parse_str("01968c8a-0000-7000-8000-000000000002").unwrap()),
        };

        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains("\"event\":\"launched\""));
        assert!(json.contains("\"com.cortexos.calculator\""));

        let deserialized: AppLifecycleEvent =
            serde_json::from_str(&json).expect("should deserialize");
        assert!(matches!(deserialized, AppLifecycleEvent::Launched { .. }));
    }

    #[test]
    fn app_lifecycle_event_stopped_serialization() {
        let event = AppLifecycleEvent::Stopped {
            instance_id: AppInstanceId(uuid::Uuid::new_v7(uuid::timestamp::Timestamp::now(
                uuid::NoContext,
            ))),
            app_id: AppId("com.cortexos.notes".into()),
        };

        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains("\"event\":\"stopped\""));

        let roundtrip: AppLifecycleEvent = serde_json::from_str(&json).expect("should deserialize");
        assert!(matches!(roundtrip, AppLifecycleEvent::Stopped { .. }));
    }

    #[test]
    fn app_lifecycle_event_crashed_serialization() {
        let event = AppLifecycleEvent::Crashed {
            instance_id: AppInstanceId(uuid::Uuid::new_v7(uuid::timestamp::Timestamp::now(
                uuid::NoContext,
            ))),
            app_id: AppId("com.cortexos.terminal".into()),
            reason: "unhandled exception in main loop".into(),
        };

        let json = serde_json::to_string(&event).expect("should serialize");
        assert!(json.contains("\"event\":\"crashed\""));
        assert!(json.contains("unhandled exception in main loop"));

        let roundtrip: AppLifecycleEvent = serde_json::from_str(&json).expect("should deserialize");
        assert!(matches!(roundtrip, AppLifecycleEvent::Crashed { .. }));
    }

    #[test]
    fn app_lifecycle_event_suspended_resumed_roundtrip() {
        let iid = AppInstanceId(uuid::Uuid::new_v7(uuid::timestamp::Timestamp::now(
            uuid::NoContext,
        )));
        let aid = AppId("com.cortexos.media".into());

        for event in [
            AppLifecycleEvent::Suspended {
                instance_id: iid.clone(),
                app_id: aid.clone(),
            },
            AppLifecycleEvent::Resumed {
                instance_id: iid.clone(),
                app_id: aid.clone(),
            },
        ] {
            let json = serde_json::to_string(&event).expect("serialize");
            let back: AppLifecycleEvent = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(json, serde_json::to_string(&back).unwrap());
        }
    }

    // -- AppState serde tests ----------------------------------------------

    #[test]
    fn app_state_serde_roundtrip() {
        for state in [
            AppState::Starting,
            AppState::Running,
            AppState::Suspended,
            AppState::Stopping,
            AppState::Stopped,
            AppState::Crashed,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let back: AppState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, back);
        }
    }
}
