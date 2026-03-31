//! Settings API routes.
//!
//! Framework-agnostic handler functions for namespace-scoped settings.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_core::SuccessResponse;
use cortex_settings::types::SettingValue;
use cortex_settings::SettingsService;
use serde::{Deserialize, Serialize};

/// Response body for a single setting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingResponse {
    pub namespace: String,
    pub key: String,
    pub value: SettingValue,
    pub updated_at: String,
}

/// Request body for setting a value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetSettingRequest {
    pub namespace: String,
    pub key: String,
    pub value: SettingValue,
}

/// Get a single setting.
pub async fn get(
    state: &AppState,
    namespace: &str,
    key: &str,
) -> Result<SuccessResponse<SettingResponse>> {
    let entry = state
        .settings
        .get(namespace, key)
        .await
        .map_err(|e| match e {
            cortex_settings::SettingsError::NotFound(_) => {
                ApiError::NotFound(format!("setting not found: {namespace}:{key}"))
            }
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: SettingResponse {
            namespace: entry.namespace,
            key: entry.key,
            value: entry.value,
            updated_at: entry.updated_at,
        },
        meta: None,
    })
}

/// Set (upsert) a setting value.
pub async fn set(state: &AppState, req: SetSettingRequest) -> Result<SuccessResponse<()>> {
    state
        .settings
        .set(&req.namespace, &req.key, req.value)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// Delete a setting.
pub async fn delete(state: &AppState, namespace: &str, key: &str) -> Result<SuccessResponse<()>> {
    state
        .settings
        .delete(namespace, key)
        .await
        .map_err(|e| match e {
            cortex_settings::SettingsError::NotFound(_) => {
                ApiError::NotFound(format!("setting not found: {namespace}:{key}"))
            }
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// List all settings in a namespace.
pub async fn list_all(
    state: &AppState,
    namespace: &str,
) -> Result<SuccessResponse<Vec<SettingResponse>>> {
    let entries = state
        .settings
        .list_all(namespace)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items = entries
        .into_iter()
        .map(|e| SettingResponse {
            namespace: e.namespace,
            key: e.key,
            value: e.value,
            updated_at: e.updated_at,
        })
        .collect();

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}
