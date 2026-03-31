//! Window management API routes.
//!
//! Framework-agnostic handler functions for window and workspace operations.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_core::{SuccessResponse, UserId};
use cortex_wm::service::WindowManagerService;
use cortex_wm::types::*;
use serde::{Deserialize, Serialize};

/// Response for a single window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowResponse {
    pub id: String,
    pub instance_id: String,
    pub user_id: String,
    pub workspace_id: String,
    pub title: String,
    pub state: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub z_index: u32,
    pub focused: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl From<Window> for WindowResponse {
    fn from(w: Window) -> Self {
        Self {
            id: w.id.0,
            instance_id: w.instance_id.0.to_string(),
            user_id: w.user_id.0.to_string(),
            workspace_id: w.workspace_id.0,
            title: w.title,
            state: match w.state {
                WindowState::Normal => "normal",
                WindowState::Minimized => "minimized",
                WindowState::Maximized => "maximized",
                WindowState::Closed => "closed",
            }
            .to_string(),
            x: w.x,
            y: w.y,
            width: w.width,
            height: w.height,
            z_index: w.z_index,
            focused: w.focused,
            created_at: w.created_at,
            updated_at: w.updated_at,
        }
    }
}

/// Response for a workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceResponse {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub index: u32,
    pub active: bool,
    pub created_at: String,
}

impl From<Workspace> for WorkspaceResponse {
    fn from(w: Workspace) -> Self {
        Self {
            id: w.id.0,
            user_id: w.user_id.0.to_string(),
            name: w.name,
            index: w.index,
            active: w.active,
            created_at: w.created_at,
        }
    }
}

/// Open a new window.
pub async fn open_window(
    state: &AppState,
    user_id: &UserId,
    req: OpenWindowRequest,
) -> Result<SuccessResponse<WindowResponse>> {
    let win = state
        .wm
        .open_window(user_id, req)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: WindowResponse::from(win),
        meta: None,
    })
}

/// Close a window by ID.
pub async fn close_window(state: &AppState, window_id: &str) -> Result<SuccessResponse<()>> {
    state
        .wm
        .close_window(window_id)
        .await
        .map_err(|e| match e {
            cortex_wm::WmError::WindowNotFound(_) => ApiError::NotFound(e.to_string()),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// Minimize a window.
pub async fn minimize_window(
    state: &AppState,
    window_id: &str,
) -> Result<SuccessResponse<WindowResponse>> {
    let win = state
        .wm
        .minimize_window(window_id)
        .await
        .map_err(|e| match e {
            cortex_wm::WmError::WindowNotFound(_) => ApiError::NotFound(e.to_string()),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: WindowResponse::from(win),
        meta: None,
    })
}

/// Maximize a window.
pub async fn maximize_window(
    state: &AppState,
    window_id: &str,
) -> Result<SuccessResponse<WindowResponse>> {
    let win = state
        .wm
        .maximize_window(window_id)
        .await
        .map_err(|e| match e {
            cortex_wm::WmError::WindowNotFound(_) => ApiError::NotFound(e.to_string()),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: WindowResponse::from(win),
        meta: None,
    })
}

/// Restore a window to normal state.
pub async fn restore_window(
    state: &AppState,
    window_id: &str,
) -> Result<SuccessResponse<WindowResponse>> {
    let win = state
        .wm
        .restore_window(window_id)
        .await
        .map_err(|e| match e {
            cortex_wm::WmError::WindowNotFound(_) => ApiError::NotFound(e.to_string()),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: WindowResponse::from(win),
        meta: None,
    })
}

/// Move a window.
pub async fn move_window(
    state: &AppState,
    window_id: &str,
    req: MoveWindowRequest,
) -> Result<SuccessResponse<WindowResponse>> {
    let win = state
        .wm
        .move_window(window_id, req)
        .await
        .map_err(|e| match e {
            cortex_wm::WmError::WindowNotFound(_) => ApiError::NotFound(e.to_string()),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: WindowResponse::from(win),
        meta: None,
    })
}

/// Resize a window.
pub async fn resize_window(
    state: &AppState,
    window_id: &str,
    req: ResizeWindowRequest,
) -> Result<SuccessResponse<WindowResponse>> {
    let win = state
        .wm
        .resize_window(window_id, req)
        .await
        .map_err(|e| match e {
            cortex_wm::WmError::WindowNotFound(_) => ApiError::NotFound(e.to_string()),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: WindowResponse::from(win),
        meta: None,
    })
}

/// Focus a window.
pub async fn focus_window(
    state: &AppState,
    window_id: &str,
) -> Result<SuccessResponse<WindowResponse>> {
    let win = state
        .wm
        .focus_window(window_id)
        .await
        .map_err(|e| match e {
            cortex_wm::WmError::WindowNotFound(_) => ApiError::NotFound(e.to_string()),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: WindowResponse::from(win),
        meta: None,
    })
}

/// Get a single window by ID.
pub async fn get_window(
    state: &AppState,
    window_id: &str,
) -> Result<SuccessResponse<WindowResponse>> {
    let win = state.wm.get_window(window_id).await.map_err(|e| match e {
        cortex_wm::WmError::WindowNotFound(_) => ApiError::NotFound(e.to_string()),
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: WindowResponse::from(win),
        meta: None,
    })
}

/// List windows for a user in a workspace.
pub async fn list_windows(
    state: &AppState,
    user_id: &UserId,
    workspace_id: &str,
) -> Result<SuccessResponse<Vec<WindowResponse>>> {
    let windows = state
        .wm
        .list_windows(user_id, workspace_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items = windows.into_iter().map(WindowResponse::from).collect();

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}

/// Create a new workspace.
pub async fn create_workspace(
    state: &AppState,
    user_id: &UserId,
    req: CreateWorkspaceRequest,
) -> Result<SuccessResponse<WorkspaceResponse>> {
    let ws = state
        .wm
        .create_workspace(user_id, req)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: WorkspaceResponse::from(ws),
        meta: None,
    })
}

/// Switch active workspace.
pub async fn switch_workspace(
    state: &AppState,
    user_id: &UserId,
    workspace_id: &str,
) -> Result<SuccessResponse<WorkspaceResponse>> {
    let ws = state
        .wm
        .switch_workspace(user_id, workspace_id)
        .await
        .map_err(|e| match e {
            cortex_wm::WmError::WorkspaceNotFound(_) => ApiError::NotFound(e.to_string()),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: WorkspaceResponse::from(ws),
        meta: None,
    })
}

/// List workspaces for a user.
pub async fn list_workspaces(
    state: &AppState,
    user_id: &UserId,
) -> Result<SuccessResponse<Vec<WorkspaceResponse>>> {
    let workspaces = state
        .wm
        .list_workspaces(user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items = workspaces
        .into_iter()
        .map(WorkspaceResponse::from)
        .collect();

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}

/// Get the active workspace for a user.
pub async fn get_active_workspace(
    state: &AppState,
    user_id: &UserId,
) -> Result<SuccessResponse<WorkspaceResponse>> {
    let ws = state
        .wm
        .get_active_workspace(user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: WorkspaceResponse::from(ws),
        meta: None,
    })
}

/// Delete a workspace.
pub async fn delete_workspace(state: &AppState, workspace_id: &str) -> Result<SuccessResponse<()>> {
    state
        .wm
        .delete_workspace(workspace_id)
        .await
        .map_err(|e| match e {
            cortex_wm::WmError::WorkspaceNotFound(_) => ApiError::NotFound(e.to_string()),
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}
