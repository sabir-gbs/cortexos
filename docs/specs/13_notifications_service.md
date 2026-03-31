# 13 — Notifications Service

## 1. Purpose
Define the notification system that delivers transient messages to users from apps and system services.

## 2. Scope
- Notification lifecycle (create, display, dismiss, expire, action)
- Notification types, priorities, and queueing
- Notification center UI
- Toast display behavior
- Per-app notification settings
- Do-not-disturb mode

## 3. Out of Scope
- Push notifications (browser-rendered, no push)
- Email/SMS notifications
- Notification sound customization (v1 — single default sound)

## 4. Objectives
1. Notifications never block the user's workflow.
2. Users have full control over which apps can notify.
3. Urgent system notifications always shown regardless of settings.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| App sends notification | Toast appears bottom-right for 5 seconds |
| User clicks notification | Associated action executed |
| User clicks dismiss | Toast removed, notification marked dismissed |
| User opens notification center | History of recent notifications shown |
| User enables DND | Non-urgent notifications suppressed |
| Notification expires | Auto-removed after timeout |

## 6. System Behavior

### 6.1 Notification Lifecycle
```
Created → Queued → Displayed → Dismissed (by user)
                              → Expired (timeout)
                              → ActionTaken (user clicked action)
```

### 6.2 Notification Types and Priority
```rust
enum NotificationType { Info, Warning, Error, Success }
enum Priority { Low, Normal, High, Urgent }
```
- Urgent notifications force display even in DND mode
- High priority shown immediately, queue bypassed
- Normal/Low queued and displayed in order

### 6.3 Queue Rules
- Max 100 pending notifications per user
- FIFO ordering within same priority
- Overflow: oldest Low-priority auto-dismissed
- Max 3 visible toasts simultaneously

### 6.4 Toast Display
- Position: bottom-right corner
- Auto-dismiss: 5 seconds (configurable via `system.notification_timeout_ms`)
- Stack: newest on top, max 3 visible
- Animation: slide-in, fade-out (respects reduce_motion setting)

## 7. Architecture
```
┌─────────────────────────────────┐
│         cortex-notify           │
│  ┌────────────────────────────┐ │
│  │  Notification Manager      │ │
│  │  (create, queue, deliver)  │ │
│  └────────────┬───────────────┘ │
│  ┌────────────┴───────────────┐ │
│  │  Settings Checker          │ │
│  │  (DND, per-app settings)   │ │
│  └────────────┬───────────────┘ │
│  ┌────────────┴───────────────┐ │
│  │  Persistence (SQLite)      │ │
│  │  (last 1000 notifications) │ │
│  └────────────────────────────┘ │
└─────────────────────────────────┘
```

## 8. Data Model
```rust
struct Notification {
    id: String,                    // UUID
    app_id: String,                // Source app
    notification_type: NotificationType,
    priority: Priority,
    title: String,
    body: String,
    icon_url: Option<String>,
    actions: Vec<NotificationAction>,  // Max 3
    state: NotificationState,
    created_at: chrono::DateTime<chrono::Utc>,
    expires_at: Option<chrono::DateTime<chrono::Utc>>,
    displayed_at: Option<chrono::DateTime<chrono::Utc>>,
    dismissed_at: Option<chrono::DateTime<chrono::Utc>>,
}

struct NotificationAction {
    id: String,                    // Action identifier
    label: String,                 // Button text
    action_type: ActionType,
}

enum ActionType {
    Dismiss,                       // Just dismiss
    OpenApp { app_id: String },
    OpenUrl { url: String },
    OpenFile { file_id: String },
    Custom { command: String, payload: serde_json::Value },
}

enum NotificationState {
    Created,
    Queued,
    Displayed,
    Dismissed,
    Expired,
    ActionTaken { action_id: String },
}
```

## 9. Public Interfaces
```
POST   /api/v1/notifications                      → Create notification
GET    /api/v1/notifications                       → List (paginated, last 1000)
GET    /api/v1/notifications/{id}                  → Get single
PUT    /api/v1/notifications/{id}/dismiss          → Dismiss
POST   /api/v1/notifications/{id}/action/{action}  → Execute action
DELETE /api/v1/notifications                       → Clear all (user)
GET    /api/v1/notifications/settings              → Get per-app settings
PUT    /api/v1/notifications/settings/{app_id}     → Update per-app settings
```

### WebSocket Events
```
notification.created   → Notification
notification.dismissed → { notification_id }
notification.read      → { notification_id }
notification.all_read  → {}
notification.expired   → { notification_id }
```

## 10. Internal Interfaces
- Checks cortex-policy for notification permission
- Reads DND and per-app settings from cortex-settings
- Persists via cortex-db
- Emits events via command bus

## 11. State Management
- Notifications stored in SQLite `notifications` table
- Auto-purge: keep last 1000 per user
- Index on `user_id, state, created_at`

## 12. Failure Modes and Error Handling
| Failure | Handling |
|---|---|
| App has notifications disabled | Silently discard, log at DEBUG |
| DND active, non-urgent | Queue but don't display |
| Queue full (100) | Auto-dismiss oldest Low priority |
| Action execution fails | Show error toast, keep notification |

## 13. Security and Permissions
- Apps need `Notification` capability to create notifications
- System notifications (cortex-* apps) always allowed
- Notification content validated: max title 100 chars, body 500 chars
- No executable content in notifications

## 14. Performance Requirements
| Metric | Target |
|---|---|
| Create to displayed | < 200ms |
| List notifications (50) | < 50ms |
| Dismiss | < 50ms |

## 15. Accessibility Requirements
- Toast notifications announced via ARIA live region
- Notification center keyboard-navigable
- Action buttons have clear labels
- Priority conveyed via text, not color alone

## 16. Observability and Logging
- Notification creation logged at DEBUG: {id, app_id, type, priority}
- DND suppression logged at DEBUG
- Action execution logged at INFO: {notification_id, action_id}
- Metrics: notifications_created_total, notifications_per_app

## 17. Testing Requirements
- Unit: queue ordering by priority
- Unit: DND suppression (urgent bypasses)
- Unit: auto-dismiss on overflow
- Integration: create → WebSocket delivery → dismiss
- E2E: app creates notification → user sees toast → clicks action

## 18. Acceptance Criteria
- [ ] Notifications created, displayed, dismissed, expired
- [ ] Queue respects priority ordering
- [ ] DND mode suppresses non-urgent, allows urgent
- [ ] Per-app enable/disable works
- [ ] Max 3 toasts visible simultaneously
- [ ] Action buttons execute correctly
- [ ] Notification center shows history
- [ ] Last 1000 notifications persisted

## 19. Build Order and Dependencies
**Layer 9**. Depends on: 01, 02, 04 (permissions), 05 (settings), 10 (events)

## 20. Non-Goals and Anti-Patterns
- No notification scheduling (v1)
- No notification grouping/stacking (v1)
- No rich media in notifications (text + icon only)
- NEVER allow apps to create notifications without permission
- NEVER display notifications from disabled apps
- NEVER allow notification content to execute code

## 21. Implementation Instructions for Claude Code / Codex
1. Define Notification, NotificationAction, NotificationState types.
2. Implement Notification Manager: create, queue, deliver, dismiss.
3. Implement priority queue with overflow handling.
4. Implement DND and per-app settings check.
5. Implement REST API endpoints.
6. Implement toast UI component with auto-dismiss.
7. Implement notification center UI.
8. Write tests: queue ordering, DND suppression, lifecycle transitions.
