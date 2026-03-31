//! Policy API routes.
//!
//! Framework-agnostic handler functions for authorization / permission management.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_core::{AppId, SuccessResponse, UserId};
use cortex_policy::PermissionGrant;
use cortex_policy::PermissionKind;
use cortex_policy::PolicyService;
use serde::{Deserialize, Serialize};

/// Response body for a permission check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResponse {
    pub allowed: bool,
}

/// Response body for a permission grant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantResponse {
    pub grant_id: String,
    pub user_id: String,
    pub app_id: String,
    pub permission: String,
    pub granted_at: String,
}

/// Request body for granting a permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrantRequest {
    pub user_id: String,
    pub app_id: String,
    pub permission: String,
}

/// Request body for checking a permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckRequest {
    pub app_id: String,
    pub permission: String,
}

/// Check whether a user holds a specific permission for an app.
pub async fn check_permission(
    state: &AppState,
    user_id: &UserId,
    req: CheckRequest,
) -> Result<SuccessResponse<PermissionCheckResponse>> {
    let app_id = AppId(req.app_id);
    let allowed = state
        .policy
        .check_permission(user_id, &app_id, &req.permission)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: PermissionCheckResponse { allowed },
        meta: None,
    })
}

/// Grant a permission to a user for an app.
pub async fn grant(state: &AppState, req: GrantRequest) -> Result<SuccessResponse<GrantResponse>> {
    let permission: PermissionKind = req
        .permission
        .parse()
        .map_err(|e: String| ApiError::BadRequest(e))?;

    let user_id = UserId(
        uuid::Uuid::parse_str(&req.user_id)
            .map_err(|e| ApiError::BadRequest(format!("invalid user_id: {e}")))?,
    );
    let app_id = AppId(req.app_id);
    let grant_id = uuid::Uuid::now_v7().to_string();
    let granted_at = chrono::Utc::now().to_rfc3339();

    let grant = PermissionGrant {
        grant_id: grant_id.clone(),
        user_id: user_id.clone(),
        app_id: app_id.clone(),
        permission,
        granted_at: granted_at.clone(),
    };

    state
        .policy
        .grant(grant)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: GrantResponse {
            grant_id,
            user_id: user_id.0.to_string(),
            app_id: app_id.0,
            permission: permission.to_string(),
            granted_at,
        },
        meta: None,
    })
}

/// Revoke a permission grant by ID.
pub async fn revoke(state: &AppState, grant_id: &str) -> Result<SuccessResponse<()>> {
    state.policy.revoke(grant_id).await.map_err(|e| match e {
        cortex_policy::PolicyError::GrantNotFound => {
            ApiError::NotFound(format!("grant not found: {grant_id}"))
        }
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// List all permission grants for a user/app pair.
pub async fn list_grants(
    state: &AppState,
    user_id: &UserId,
    app_id: &str,
) -> Result<SuccessResponse<Vec<GrantResponse>>> {
    let app_id = AppId(app_id.to_string());
    let grants = state
        .policy
        .list_grants(user_id, &app_id)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items = grants
        .into_iter()
        .map(|g| GrantResponse {
            grant_id: g.grant_id,
            user_id: g.user_id.0.to_string(),
            app_id: g.app_id.0,
            permission: g.permission.to_string(),
            granted_at: g.granted_at,
        })
        .collect();

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}
