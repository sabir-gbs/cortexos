//! Settings service trait.

use crate::error::Result;
use crate::types::{SettingEntry, SettingValue};

/// Settings service trait.
///
/// All settings reads and writes go through this trait. Namespace strings are
/// used to partition settings by owning subsystem (e.g. "ai.providers",
/// "ai.routing"). A subsystem may only write to namespaces it owns, but may
/// read from any namespace.
pub trait SettingsService: Send + Sync {
    /// Get a single setting by namespace and key.
    fn get(
        &self,
        namespace: &str,
        key: &str,
    ) -> impl std::future::Future<Output = Result<SettingEntry>> + Send;

    /// Set a setting value. Creates the setting if it does not exist.
    fn set(
        &self,
        namespace: &str,
        key: &str,
        value: SettingValue,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Delete a setting, reverting to its default (or removing it entirely).
    fn delete(
        &self,
        namespace: &str,
        key: &str,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// List all settings within a namespace.
    fn list_all(
        &self,
        namespace: &str,
    ) -> impl std::future::Future<Output = Result<Vec<SettingEntry>>> + Send;
}
