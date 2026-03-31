//! App runtime API routes.
//!
//! Framework-agnostic handler functions for app lifecycle management.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_core::{AppId, AppInstanceId, SuccessResponse, UserId};
use cortex_runtime::RuntimeService;
use serde::{Deserialize, Serialize};

/// Response body for an app instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInstanceResponse {
    pub instance_id: String,
    pub app_id: String,
    pub user_id: String,
    pub state: String,
    pub window_id: Option<String>,
    pub launched_at: Option<String>,
    pub stopped_at: Option<String>,
}

/// Launch request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchRequest {
    pub app_id: String,
}

/// Launch a new app instance.
pub async fn launch(
    state: &AppState,
    user_id: &UserId,
    req: LaunchRequest,
) -> Result<SuccessResponse<AppInstanceResponse>> {
    let app_id = AppId(req.app_id);
    let instance = state
        .runtime
        .launch(&app_id, user_id)
        .await
        .map_err(|e| match e {
            cortex_runtime::RuntimeError::AppNotFound(id) => {
                ApiError::NotFound(format!("app not found: {id}"))
            }
            cortex_runtime::RuntimeError::AlreadyRunning(id) => {
                ApiError::BadRequest(format!("app already running: {id}"))
            }
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: instance_to_response(&instance),
        meta: None,
    })
}

/// Stop a running app instance.
pub async fn stop(state: &AppState, instance_id: &str) -> Result<SuccessResponse<()>> {
    let iid = AppInstanceId(
        uuid::Uuid::parse_str(instance_id)
            .map_err(|e| ApiError::BadRequest(format!("invalid instance_id: {e}")))?,
    );

    state.runtime.stop(&iid).await.map_err(|e| match e {
        cortex_runtime::RuntimeError::NotRunning(id) => {
            ApiError::BadRequest(format!("app not running: {id}"))
        }
        cortex_runtime::RuntimeError::AppNotFound(id) => {
            ApiError::NotFound(format!("instance not found: {id}"))
        }
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// Suspend a running app instance.
pub async fn suspend(state: &AppState, instance_id: &str) -> Result<SuccessResponse<()>> {
    let iid = AppInstanceId(
        uuid::Uuid::parse_str(instance_id)
            .map_err(|e| ApiError::BadRequest(format!("invalid instance_id: {e}")))?,
    );

    state.runtime.suspend(&iid).await.map_err(|e| match e {
        cortex_runtime::RuntimeError::NotRunning(id) => {
            ApiError::BadRequest(format!("app not running: {id}"))
        }
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// Resume a suspended app instance.
pub async fn resume(
    state: &AppState,
    instance_id: &str,
) -> Result<SuccessResponse<AppInstanceResponse>> {
    let iid = AppInstanceId(
        uuid::Uuid::parse_str(instance_id)
            .map_err(|e| ApiError::BadRequest(format!("invalid instance_id: {e}")))?,
    );

    let instance = state.runtime.resume(&iid).await.map_err(|e| match e {
        cortex_runtime::RuntimeError::NotRunning(id) => {
            ApiError::BadRequest(format!("app not suspended: {id}"))
        }
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: instance_to_response(&instance),
        meta: None,
    })
}

/// Get the state of an app instance.
pub async fn get_state(state: &AppState, instance_id: &str) -> Result<SuccessResponse<String>> {
    let iid = AppInstanceId(
        uuid::Uuid::parse_str(instance_id)
            .map_err(|e| ApiError::BadRequest(format!("invalid instance_id: {e}")))?,
    );

    let app_state = state
        .runtime
        .get_state(&iid)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let state_str = serde_json::to_string(&app_state)
        .map_err(|e| ApiError::Internal(format!("state serialization: {e}")))?;

    // Remove surrounding quotes from the JSON string representation.
    let state_str = state_str.trim_matches('"').to_string();

    Ok(SuccessResponse {
        data: state_str,
        meta: None,
    })
}

/// List running app instances for a user.
pub async fn list_running(
    state: &AppState,
    user_id: &UserId,
) -> Result<SuccessResponse<Vec<AppInstanceResponse>>> {
    let instances = state
        .runtime
        .list_running(user_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items = instances.iter().map(instance_to_response).collect();

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}

fn instance_to_response(i: &cortex_runtime::AppInstance) -> AppInstanceResponse {
    AppInstanceResponse {
        instance_id: i.instance_id.0.to_string(),
        app_id: i.app_id.0.clone(),
        user_id: i.user_id.0.to_string(),
        state: serde_json::to_string(&i.state)
            .unwrap_or_default()
            .trim_matches('"')
            .to_string(),
        window_id: i.window_id.clone(),
        launched_at: i.launched_at.clone(),
        stopped_at: i.stopped_at.clone(),
    }
}
