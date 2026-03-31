//! Notifications API routes.
//!
//! Framework-agnostic handler functions for the notification system.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_core::SuccessResponse;
use cortex_notify::Notification;
use cortex_notify::NotifyService;
use serde::{Deserialize, Serialize};

/// Response body for a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub notification_id: String,
    pub user_id: String,
    pub title: String,
    pub body: String,
    pub category: String,
    pub is_read: bool,
    pub created_at: String,
    pub dismissed_at: Option<String>,
}

/// List notifications for a user.
pub async fn list(
    state: &AppState,
    user_id: &str,
) -> Result<SuccessResponse<Vec<NotificationResponse>>> {
    let uid = cortex_core::UserId(
        uuid::Uuid::parse_str(user_id)
            .map_err(|e| ApiError::BadRequest(format!("invalid user_id: {e}")))?,
    );

    let notifications = state
        .notify
        .list(&uid, None)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    let items = notifications
        .into_iter()
        .map(|n| NotificationResponse {
            notification_id: n.notification_id.0.to_string(),
            user_id: n.user_id.0.to_string(),
            title: n.title,
            body: n.body,
            category: n.category,
            is_read: n.is_read,
            created_at: n.created_at,
            dismissed_at: n.dismissed_at,
        })
        .collect();

    Ok(SuccessResponse {
        data: items,
        meta: None,
    })
}

/// Create a notification.
pub async fn create(
    state: &AppState,
    req: Notification,
) -> Result<SuccessResponse<NotificationResponse>> {
    let notification = state
        .notify
        .create(req)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: NotificationResponse {
            notification_id: notification.notification_id.0.to_string(),
            user_id: notification.user_id.0.to_string(),
            title: notification.title,
            body: notification.body,
            category: notification.category,
            is_read: notification.is_read,
            created_at: notification.created_at,
            dismissed_at: notification.dismissed_at,
        },
        meta: None,
    })
}

/// Mark a notification as read.
pub async fn mark_read(state: &AppState, notification_id: &str) -> Result<SuccessResponse<()>> {
    let nid = cortex_core::NotificationId(
        uuid::Uuid::parse_str(notification_id)
            .map_err(|e| ApiError::BadRequest(format!("invalid notification_id: {e}")))?,
    );

    state.notify.mark_read(&nid).await.map_err(|e| match e {
        cortex_notify::NotifyError::NotFound(_) => {
            ApiError::NotFound(format!("notification not found: {notification_id}"))
        }
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// Dismiss a notification.
pub async fn dismiss(state: &AppState, notification_id: &str) -> Result<SuccessResponse<()>> {
    let nid = cortex_core::NotificationId(
        uuid::Uuid::parse_str(notification_id)
            .map_err(|e| ApiError::BadRequest(format!("invalid notification_id: {e}")))?,
    );

    state.notify.dismiss(&nid).await.map_err(|e| match e {
        cortex_notify::NotifyError::NotFound(_) => {
            ApiError::NotFound(format!("notification not found: {notification_id}"))
        }
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}
