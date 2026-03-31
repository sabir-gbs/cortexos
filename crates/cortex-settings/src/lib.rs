//! CortexOS settings service.
//!
//! Manages system-level and per-user settings across namespace-owning
//! subsystems. Each subsystem owns its namespace; no cross-namespace writes.
//!
//! # Key types
//!
//! - [`SettingValue`] -- typed setting values (string, number, boolean, JSON object).
//! - [`SettingEntry`] -- a stored key-value pair with namespace and timestamp.
//! - [`SettingNamespace`] -- namespace metadata (name + description).
//! - [`AI_SETTING_NAMESPACES`] -- well-known AI-related namespace constants.
//! - [`SettingsService`] -- async trait for reading and writing settings.

pub mod error;
pub mod service;
pub mod sqlite;
pub mod types;

pub use error::{Result, SettingsError};
pub use service::SettingsService;
pub use sqlite::SqliteSettingsService;
pub use types::{ai_setting_namespaces, SettingEntry, SettingNamespace, SettingValue};
