//! Admin API routes.
//!
//! Framework-agnostic handler functions for system administration.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_admin::AdminService;
use cortex_core::SuccessResponse;
use serde::{Deserialize, Serialize};

/// Health check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub uptime_secs: u64,
    pub components: Vec<ComponentHealthResponse>,
}

/// Component health response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealthResponse {
    pub name: String,
    pub status: String,
    pub latency_ms: Option<u64>,
}

/// Diagnostic information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsResponse {
    pub generated_at: String,
    pub active_sessions: u64,
    pub running_apps: u64,
    pub db_size_bytes: u64,
}

/// Health check handler.
pub async fn health(state: &AppState) -> Result<SuccessResponse<HealthResponse>> {
    let health = state
        .admin
        .health()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: HealthResponse {
            status: serde_json::to_string(&health.status)
                .unwrap_or_default()
                .trim_matches('"')
                .to_string(),
            uptime_secs: health.uptime_secs,
            components: health
                .components
                .into_iter()
                .map(|c| ComponentHealthResponse {
                    name: c.name,
                    status: serde_json::to_string(&c.status)
                        .unwrap_or_default()
                        .trim_matches('"')
                        .to_string(),
                    latency_ms: c.latency_ms,
                })
                .collect(),
        },
        meta: None,
    })
}

/// Diagnostics handler.
pub async fn diagnostics(state: &AppState) -> Result<SuccessResponse<DiagnosticsResponse>> {
    let diag = state
        .admin
        .diagnostics()
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: DiagnosticsResponse {
            generated_at: diag.generated_at,
            active_sessions: diag.active_sessions,
            running_apps: diag.running_apps,
            db_size_bytes: diag.db_size_bytes,
        },
        meta: None,
    })
}

/// Backup handler.
pub async fn backup(state: &AppState, path: &str) -> Result<SuccessResponse<()>> {
    state
        .admin
        .backup(path)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}
