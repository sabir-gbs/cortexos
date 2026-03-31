//! CortexOS notification service.
//!
//! Manages notification creation, delivery, read state, and dismissal.

pub mod error;
pub mod service;
pub mod sqlite;
pub mod types;

pub use error::{NotifyError, Result};
pub use service::NotifyService;
pub use sqlite::SqliteNotifyService;
pub use types::{Notification, NotificationAction};
