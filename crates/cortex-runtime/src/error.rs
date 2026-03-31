//! Runtime error types.

/// Result alias for runtime operations.
pub type Result<T> = std::result::Result<T, RuntimeError>;

/// Errors that can occur during app runtime operations.
#[derive(Debug, thiserror::Error)]
pub enum RuntimeError {
    /// The requested app was not found in the registry.
    #[error("app not found: {0}")]
    AppNotFound(String),

    /// An app instance crashed during execution.
    #[error("app {app_id} crashed: {reason}")]
    AppCrashed {
        /// The app identifier.
        app_id: String,
        /// The crash reason or error message.
        reason: String,
    },

    /// The app manifest is invalid or missing required fields.
    #[error("invalid manifest: {0}")]
    ManifestInvalid(String),

    /// The app is already running and cannot be launched again.
    #[error("app already running: {0}")]
    AlreadyRunning(String),

    /// The app instance is not currently running.
    #[error("app not running: {0}")]
    NotRunning(String),

    /// The user does not have permission for the requested operation.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// An unexpected internal error.
    #[error("internal runtime error")]
    Internal,
}
