//! Notification domain types.

use cortex_core::{NotificationId, Timestamp, UserId};
use serde::{Deserialize, Serialize};

/// A notification delivered to a user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Unique notification identifier.
    pub notification_id: NotificationId,
    /// The user this notification is for.
    pub user_id: UserId,
    /// The notification title.
    pub title: String,
    /// The notification body text.
    pub body: String,
    /// The notification category (e.g. "system", "app", "alert").
    pub category: String,
    /// Whether the notification has been read.
    pub is_read: bool,
    /// When the notification was created.
    pub created_at: Timestamp,
    /// When the notification was dismissed, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dismissed_at: Option<String>,
}

/// Actions that can be performed on a notification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationAction {
    /// Create a new notification.
    Create,
    /// Mark a notification as read.
    MarkRead,
    /// Dismiss a notification.
    Dismiss,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_core::NotificationId;
    use uuid::Uuid;

    #[test]
    fn notification_construction() {
        let notif = Notification {
            notification_id: NotificationId(Uuid::now_v7()),
            user_id: UserId(Uuid::now_v7()),
            title: "Test".to_string(),
            body: "Hello world".to_string(),
            category: "system".to_string(),
            is_read: false,
            created_at: "2026-03-30T12:00:00Z".to_string(),
            dismissed_at: None,
        };
        assert!(!notif.is_read);
        assert!(notif.dismissed_at.is_none());
        assert_eq!(notif.category, "system");
    }

    #[test]
    fn notification_with_dismissed_at() {
        let notif = Notification {
            notification_id: NotificationId(Uuid::now_v7()),
            user_id: UserId(Uuid::now_v7()),
            title: "Alert".to_string(),
            body: "Dismissed".to_string(),
            category: "alert".to_string(),
            is_read: true,
            created_at: "2026-03-30T12:00:00Z".to_string(),
            dismissed_at: Some("2026-03-30T13:00:00Z".to_string()),
        };
        assert!(notif.dismissed_at.is_some());
    }

    #[test]
    fn notification_action_serde_roundtrip() {
        let actions = vec![
            NotificationAction::Create,
            NotificationAction::MarkRead,
            NotificationAction::Dismiss,
        ];
        for action in &actions {
            let json = serde_json::to_string(action).unwrap();
            let parsed: NotificationAction = serde_json::from_str(&json).unwrap();
            assert_eq!(*action, parsed);
        }
    }

    #[test]
    fn notification_action_serde_values() {
        assert_eq!(
            serde_json::to_string(&NotificationAction::Create).unwrap(),
            "\"create\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationAction::MarkRead).unwrap(),
            "\"mark_read\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationAction::Dismiss).unwrap(),
            "\"dismiss\""
        );
    }

    #[test]
    fn notification_serde_roundtrip() {
        let notif = Notification {
            notification_id: NotificationId(Uuid::now_v7()),
            user_id: UserId(Uuid::now_v7()),
            title: "Test".to_string(),
            body: "Body".to_string(),
            category: "app".to_string(),
            is_read: false,
            created_at: "2026-03-30T12:00:00Z".to_string(),
            dismissed_at: None,
        };
        let json = serde_json::to_string(&notif).unwrap();
        let parsed: Notification = serde_json::from_str(&json).unwrap();
        assert_eq!(notif.title, parsed.title);
        assert_eq!(notif.body, parsed.body);
        assert_eq!(notif.category, parsed.category);
    }
}
