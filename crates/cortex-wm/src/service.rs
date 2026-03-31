//! Window manager service trait.

use crate::error::Result;
use crate::types::*;
use cortex_core::UserId;

/// Async service interface for window management operations.
#[allow(async_fn_in_trait)]
pub trait WindowManagerService {
    /// Open a new window for an app instance.
    async fn open_window(&self, user_id: &UserId, req: OpenWindowRequest) -> Result<Window>;

    /// Close a window by ID.
    async fn close_window(&self, window_id: &str) -> Result<()>;

    /// Minimize a window.
    async fn minimize_window(&self, window_id: &str) -> Result<Window>;

    /// Maximize a window.
    async fn maximize_window(&self, window_id: &str) -> Result<Window>;

    /// Restore a window to normal state.
    async fn restore_window(&self, window_id: &str) -> Result<Window>;

    /// Move a window to new coordinates.
    async fn move_window(&self, window_id: &str, req: MoveWindowRequest) -> Result<Window>;

    /// Resize a window.
    async fn resize_window(&self, window_id: &str, req: ResizeWindowRequest) -> Result<Window>;

    /// Focus a window (unfocusing all others for the user).
    async fn focus_window(&self, window_id: &str) -> Result<Window>;

    /// Get a window by ID.
    async fn get_window(&self, window_id: &str) -> Result<Window>;

    /// List all windows for a user in a workspace.
    async fn list_windows(&self, user_id: &UserId, workspace_id: &str) -> Result<Vec<Window>>;

    /// Create a new workspace.
    async fn create_workspace(
        &self,
        user_id: &UserId,
        req: CreateWorkspaceRequest,
    ) -> Result<Workspace>;

    /// Switch to a workspace (sets it as active, deactivates others).
    async fn switch_workspace(&self, user_id: &UserId, workspace_id: &str) -> Result<Workspace>;

    /// List all workspaces for a user.
    async fn list_workspaces(&self, user_id: &UserId) -> Result<Vec<Workspace>>;

    /// Get the active workspace for a user. Creates a default one if none exist.
    async fn get_active_workspace(&self, user_id: &UserId) -> Result<Workspace>;

    /// Delete a workspace. All windows in it are closed.
    async fn delete_workspace(&self, workspace_id: &str) -> Result<()>;
}
