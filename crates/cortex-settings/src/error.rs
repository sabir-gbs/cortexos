//! Settings error types.

/// Result alias for settings operations.
pub type Result<T> = std::result::Result<T, SettingsError>;

/// Errors that can occur during settings operations.
#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    /// The requested setting was not found.
    #[error("setting not found: {0}")]
    NotFound(String),

    /// The setting value failed validation.
    #[error("validation failed: {0}")]
    Validation(String),

    /// Namespace-related error (unknown namespace, access denied, etc.).
    #[error("namespace error: {0}")]
    Namespace(String),

    /// An unexpected internal error.
    #[error("internal settings error")]
    Internal(#[source] Box<dyn std::error::Error + Send + Sync>),
}
