//! Window manager domain types.

use cortex_core::{AppInstanceId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

/// Unique window identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub String);

impl Default for WindowId {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowId {
    /// Generate a new unique window ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7().to_string())
    }
}

/// Unique workspace identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkspaceId(pub String);

impl Default for WorkspaceId {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkspaceId {
    /// Generate a new unique workspace ID.
    pub fn new() -> Self {
        Self(uuid::Uuid::now_v7().to_string())
    }
}

/// Window state in the window lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WindowState {
    /// Window is visible and interactive.
    Normal,
    /// Window is minimized to the taskbar.
    Minimized,
    /// Window covers the entire screen.
    Maximized,
    /// Window is closed and removed.
    Closed,
}

/// A managed window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Window {
    /// Unique window identifier.
    pub id: WindowId,
    /// The app instance this window belongs to.
    pub instance_id: AppInstanceId,
    /// The user who owns this window.
    pub user_id: UserId,
    /// The workspace this window belongs to.
    pub workspace_id: WorkspaceId,
    /// Window title.
    pub title: String,
    /// Current state of the window.
    pub state: WindowState,
    /// X position in pixels.
    pub x: i32,
    /// Y position in pixels.
    pub y: i32,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Z-order (higher = on top).
    pub z_index: u32,
    /// Whether this window has focus.
    pub focused: bool,
    /// ISO 8601 timestamp when the window was created.
    pub created_at: Timestamp,
    /// ISO 8601 timestamp when the window was last updated.
    pub updated_at: Timestamp,
}

/// A workspace (virtual desktop).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique workspace identifier.
    pub id: WorkspaceId,
    /// The user who owns this workspace.
    pub user_id: UserId,
    /// Workspace display name.
    pub name: String,
    /// 0-based index for ordering.
    pub index: u32,
    /// Whether this is the active workspace.
    pub active: bool,
    /// ISO 8601 timestamp when the workspace was created.
    pub created_at: Timestamp,
}

/// Request to open a new window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenWindowRequest {
    /// The app instance this window belongs to.
    pub instance_id: String,
    /// Window title.
    pub title: String,
    /// Initial X position.
    pub x: i32,
    /// Initial Y position.
    pub y: i32,
    /// Initial width.
    pub width: u32,
    /// Initial height.
    pub height: u32,
    /// Target workspace ID (defaults to active workspace).
    pub workspace_id: Option<String>,
}

/// Request to move a window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveWindowRequest {
    /// New X position.
    pub x: i32,
    /// New Y position.
    pub y: i32,
}

/// Request to resize a window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResizeWindowRequest {
    /// New width.
    pub width: u32,
    /// New height.
    pub height: u32,
}

/// Request to create a new workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWorkspaceRequest {
    /// Workspace display name.
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_id_generates_unique() {
        let a = WindowId::new();
        let b = WindowId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn workspace_id_generates_unique() {
        let a = WorkspaceId::new();
        let b = WorkspaceId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn window_state_serde_roundtrip() {
        for state in [
            WindowState::Normal,
            WindowState::Minimized,
            WindowState::Maximized,
            WindowState::Closed,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let back: WindowState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, back);
        }
    }

    #[test]
    fn open_window_request_deserialization() {
        let json = r#"{"instance_id":"inst-1","title":"Test","x":10,"y":20,"width":800,"height":600,"workspace_id":null}"#;
        let req: OpenWindowRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.instance_id, "inst-1");
        assert_eq!(req.title, "Test");
        assert!(req.workspace_id.is_none());
    }
}
