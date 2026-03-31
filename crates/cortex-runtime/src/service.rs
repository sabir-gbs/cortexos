//! Runtime service trait.
//!
//! Defines the async interface for managing the full app lifecycle:
//! launch, stop, suspend, resume, and state queries.

use crate::error::Result;
use crate::types::{AppInstance, AppState};
use cortex_core::{AppId, AppInstanceId, UserId};

/// Runtime service trait.
///
/// All app lifecycle operations go through this trait. Implementations
/// must be `Send + Sync` and return `Send` futures so the service can
/// be shared across async tasks.
pub trait RuntimeService: Send + Sync {
    /// Launch a new instance of an app for the given user.
    ///
    /// Returns the new [`AppInstance`] in the `Starting` state. The caller
    /// should observe a subsequent [`AppState::Running`] transition via
    /// lifecycle events or by polling [`get_state`](Self::get_state).
    fn launch(
        &self,
        app_id: &AppId,
        user_id: &UserId,
    ) -> impl std::future::Future<Output = Result<AppInstance>> + Send;

    /// Stop a running app instance.
    ///
    /// Transitions the instance to `Stopping` and then `Stopped`,
    /// releasing all associated resources.
    fn stop(
        &self,
        instance_id: &AppInstanceId,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Get the current [`AppState`] of an app instance.
    fn get_state(
        &self,
        instance_id: &AppInstanceId,
    ) -> impl std::future::Future<Output = Result<AppState>> + Send;

    /// List all active app instances for the given user.
    ///
    /// Returns instances in any non-terminal state (`Starting`, `Running`,
    /// `Suspended`, `Stopping`).
    fn list_running(
        &self,
        user_id: &UserId,
    ) -> impl std::future::Future<Output = Result<Vec<AppInstance>>> + Send;

    /// Suspend a running app instance.
    ///
    /// Transitions from `Running` to `Suspended`. The app's execution
    /// context is preserved but not actively running.
    fn suspend(
        &self,
        instance_id: &AppInstanceId,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Resume a suspended app instance.
    ///
    /// Transitions from `Suspended` back to `Running` and returns
    /// the updated [`AppInstance`].
    fn resume(
        &self,
        instance_id: &AppInstanceId,
    ) -> impl std::future::Future<Output = Result<AppInstance>> + Send;
}
