//! Notification service trait.

use crate::error::Result;
use crate::types::Notification;
use cortex_core::{NotificationId, UserId};

/// Notification service trait.
///
/// Provides create, list, mark-read, and dismiss operations for user
/// notifications. All methods are async and return `Send` futures.
pub trait NotifyService: Send + Sync {
    /// Create a new notification.
    fn create(
        &self,
        notification: Notification,
    ) -> impl std::future::Future<Output = Result<Notification>> + Send;

    /// List notifications for a user.
    fn list(
        &self,
        user_id: &UserId,
        limit: Option<u32>,
    ) -> impl std::future::Future<Output = Result<Vec<Notification>>> + Send;

    /// Mark a notification as read.
    fn mark_read(
        &self,
        id: &NotificationId,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Dismiss a notification.
    fn dismiss(&self, id: &NotificationId) -> impl std::future::Future<Output = Result<()>> + Send;
}
