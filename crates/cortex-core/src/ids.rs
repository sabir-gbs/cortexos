//! Shared ID types for all CortexOS crates.
//!
//! All ID types are newtype wrappers around [`uuid::Uuid`] using UUIDv7 for
//! time-ordered, globally unique identifiers.

use serde::{Deserialize, Serialize};

/// Unique identifier for a user.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub uuid::Uuid);

/// Unique identifier for a session.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub uuid::Uuid);

/// Unique identifier for a session token (stored opaque).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionToken(pub String);

/// Unique identifier for a file or directory node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FileId(pub uuid::Uuid);

/// Unique identifier for an installed app.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppId(pub String);

/// Unique identifier for an app instance.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppInstanceId(pub uuid::Uuid);

/// Unique identifier for a settings namespace key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SettingsNamespaceId(pub String);

/// Unique identifier for a notification.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationId(pub uuid::Uuid);

/// ISO 8601 datetime string used in all API contracts.
pub type Timestamp = String;
