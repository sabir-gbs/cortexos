//! SDK error types.

/// Result alias for SDK operations.
pub type Result<T> = std::result::Result<T, SdkError>;

/// Errors that can occur in SDK operations.
#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    /// A required resource was not found.
    #[error("not found: {0}")]
    NotFound(String),
    /// Validation of an SDK input failed.
    #[error("validation failed: {0}")]
    Validation(String),
    /// An unexpected internal error.
    #[error("internal error: {0}")]
    Internal(String),
}
