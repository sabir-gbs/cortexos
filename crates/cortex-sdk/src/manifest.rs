//! Manifest validation for third-party apps.
//!
//! Validates app manifests per spec 21 requirements.

use crate::types::AppManifest;

#[cfg(test)]
use crate::types::{AppCategory, AppPermission};

/// Validation errors for manifest checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestValidationError {
    /// app_id is not in reverse-domain format.
    InvalidAppId(String),
    /// version is not valid semver.
    InvalidVersion(String),
    /// entry_point is missing or empty.
    InvalidEntryPoint(String),
    /// Permission name is not recognized.
    InvalidPermissionName(String),
    /// Schema version mismatch.
    SchemaVersionMismatch { expected: String, found: String },
}

impl std::fmt::Display for ManifestValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidAppId(id) => write!(f, "invalid app_id format: {id}"),
            Self::InvalidVersion(v) => write!(f, "invalid semver version: {v}"),
            Self::InvalidEntryPoint(ep) => write!(f, "invalid entry_point: {ep}"),
            Self::InvalidPermissionName(p) => write!(f, "unknown permission: {p}"),
            Self::SchemaVersionMismatch { expected, found } => {
                write!(
                    f,
                    "schema version mismatch: expected {expected}, found {found}"
                )
            }
        }
    }
}

/// Known permission names that third-party apps can request.
pub const KNOWN_PERMISSIONS: &[&str] = &[
    "ai.chat",
    "ai.action",
    "files.read",
    "files.write",
    "settings.read",
    "settings.write",
    "notifications.read",
    "notifications.write",
    "apps.list",
    "apps.launch",
];

/// Validate a full app manifest.
pub fn validate_manifest(
    manifest: &AppManifest,
) -> std::result::Result<(), Vec<ManifestValidationError>> {
    let mut errors = Vec::new();

    // Validate app_id format (reverse-domain: at least one dot, no spaces)
    if !manifest.app_id.contains('.') || manifest.app_id.contains(' ') || manifest.app_id.is_empty()
    {
        errors.push(ManifestValidationError::InvalidAppId(
            manifest.app_id.clone(),
        ));
    }

    // Validate version (basic semver: X.Y.Z)
    let version_parts: Vec<&str> = manifest.version.split('.').collect();
    if version_parts.len() != 3 || version_parts.iter().any(|p| p.parse::<u32>().is_err()) {
        errors.push(ManifestValidationError::InvalidVersion(
            manifest.version.clone(),
        ));
    }

    // Validate entry_point
    if manifest.entry_point.is_empty() {
        errors.push(ManifestValidationError::InvalidEntryPoint(
            manifest.entry_point.clone(),
        ));
    }

    // Validate permissions
    for perm in &manifest.permissions {
        if !KNOWN_PERMISSIONS.contains(&perm.name.as_str()) {
            errors.push(ManifestValidationError::InvalidPermissionName(
                perm.name.clone(),
            ));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Install lifecycle for third-party apps.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InstallState {
    /// Manifest downloaded, pending validation.
    PendingValidation,
    /// Validated, waiting for user approval.
    PendingApproval,
    /// User approved, installing.
    Installing,
    /// Installation complete.
    Installed,
    /// Installation failed.
    Failed(String),
    /// Marked for removal.
    Uninstalling,
}

/// Registry entry for an installed app.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppRegistryEntry {
    /// The app manifest.
    pub manifest: AppManifest,
    /// Current install state.
    pub state: InstallState,
    /// Installation timestamp.
    pub installed_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_manifest_passes() {
        let manifest = AppManifest {
            app_id: "com.example.test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "A test app".to_string(),
            entry_point: "index.html".to_string(),
            permissions: vec![AppPermission {
                name: "ai.chat".to_string(),
                required: true,
                reason: "Need AI".to_string(),
            }],
            icon: None,
            min_platform_version: None,
            category: AppCategory::Utility,
        };
        assert!(validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn invalid_app_id_detected() {
        let manifest = AppManifest {
            app_id: "no-dots".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "".to_string(),
            entry_point: "index.html".to_string(),
            permissions: vec![],
            icon: None,
            min_platform_version: None,
            category: AppCategory::Utility,
        };
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ManifestValidationError::InvalidAppId(_))));
    }

    #[test]
    fn invalid_version_detected() {
        let manifest = AppManifest {
            app_id: "com.example.test".to_string(),
            name: "Test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            entry_point: "index.html".to_string(),
            permissions: vec![],
            icon: None,
            min_platform_version: None,
            category: AppCategory::Utility,
        };
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn unknown_permission_detected() {
        let manifest = AppManifest {
            app_id: "com.example.test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            description: "".to_string(),
            entry_point: "index.html".to_string(),
            permissions: vec![AppPermission {
                name: "network.access".to_string(),
                required: false,
                reason: "Need network".to_string(),
            }],
            icon: None,
            min_platform_version: None,
            category: AppCategory::Utility,
        };
        let result = validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn known_permissions_list() {
        assert!(KNOWN_PERMISSIONS.contains(&"ai.chat"));
        assert!(KNOWN_PERMISSIONS.contains(&"files.read"));
        assert!(KNOWN_PERMISSIONS.contains(&"apps.launch"));
    }

    #[test]
    fn install_state_serde() {
        let states = vec![
            InstallState::PendingValidation,
            InstallState::Installed,
            InstallState::Failed("timeout".to_string()),
        ];
        for state in &states {
            let json = serde_json::to_string(&state).unwrap();
            let roundtrip: InstallState = serde_json::from_str(&json).unwrap();
            assert_eq!(*state, roundtrip);
        }
    }
}
