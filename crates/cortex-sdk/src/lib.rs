//! CortexOS SDK types for third-party app development.
//!
//! Public API surface that third-party app developers use. These types
//! are versioned independently from the internal crate types.

pub mod error;
pub mod manifest;
pub mod types;

pub use error::{Result, SdkError};
pub use manifest::{
    validate_manifest, AppRegistryEntry, InstallState, ManifestValidationError, KNOWN_PERMISSIONS,
};
pub use types::{AppManifest, AppPermission, ManifestSchema};
