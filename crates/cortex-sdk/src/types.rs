//! SDK type definitions for third-party app development.
//!
//! These types define the manifest schema and permission model for
//! third-party apps per spec 21.

use serde::{Deserialize, Serialize};

/// Current manifest schema version.
pub const MANIFEST_SCHEMA_VERSION: &str = "1.0.0";

/// An app manifest describing a third-party application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppManifest {
    /// Unique app identifier (reverse-domain format, e.g., "com.example.myapp").
    pub app_id: String,
    /// Human-readable app name.
    pub name: String,
    /// App version in semver format.
    pub version: String,
    /// Brief description of the app.
    pub description: String,
    /// Entry point relative to the app root.
    pub entry_point: String,
    /// Permissions required by the app.
    pub permissions: Vec<AppPermission>,
    /// App icon path relative to the app root (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Minimum CortexOS version required (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_platform_version: Option<String>,
    /// App category for organization.
    #[serde(default)]
    pub category: AppCategory,
}

/// App categories for organization in the app launcher.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AppCategory {
    #[default]
    Utility,
    Productivity,
    Entertainment,
    Games,
    Development,
    Education,
    Social,
    Other,
}

/// A permission declaration in an app manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPermission {
    /// The permission name (e.g., "ai.chat", "files.read").
    pub name: String,
    /// Whether the permission is required or optional.
    pub required: bool,
    /// Human-readable reason why the permission is needed.
    pub reason: String,
}

/// Schema definition for validating app manifests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSchema {
    /// The schema version.
    pub version: String,
}

impl Default for ManifestSchema {
    fn default() -> Self {
        Self {
            version: MANIFEST_SCHEMA_VERSION.to_string(),
        }
    }
}

impl AppManifest {
    /// Validate the manifest for required fields and format.
    pub fn validate(&self) -> crate::error::Result<()> {
        if self.app_id.is_empty() {
            return Err(crate::error::SdkError::Validation(
                "app_id cannot be empty".to_string(),
            ));
        }
        if self.name.is_empty() {
            return Err(crate::error::SdkError::Validation(
                "name cannot be empty".to_string(),
            ));
        }
        if self.entry_point.is_empty() {
            return Err(crate::error::SdkError::Validation(
                "entry_point cannot be empty".to_string(),
            ));
        }
        if self.version.is_empty() {
            return Err(crate::error::SdkError::Validation(
                "version cannot be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_manifest() {
        let manifest = AppManifest {
            app_id: "com.example.test".to_string(),
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            description: "A test app".to_string(),
            entry_point: "index.html".to_string(),
            permissions: vec![],
            icon: None,
            min_platform_version: None,
            category: AppCategory::default(),
        };
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn manifest_rejects_empty_app_id() {
        let manifest = AppManifest {
            app_id: String::new(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: String::new(),
            entry_point: "index.html".to_string(),
            permissions: vec![],
            icon: None,
            min_platform_version: None,
            category: AppCategory::default(),
        };
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn manifest_serialization_roundtrip() {
        let manifest = AppManifest {
            app_id: "com.example.test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "A test".to_string(),
            entry_point: "index.html".to_string(),
            permissions: vec![AppPermission {
                name: "files.read".to_string(),
                required: true,
                reason: "Need to read files".to_string(),
            }],
            icon: Some("icon.png".to_string()),
            min_platform_version: None,
            category: AppCategory::Productivity,
        };
        let json = serde_json::to_string(&manifest).unwrap();
        let deserialized: AppManifest = serde_json::from_str(&json).unwrap();
        assert_eq!(manifest.app_id, deserialized.app_id);
        assert_eq!(manifest.permissions.len(), deserialized.permissions.len());
    }
}
