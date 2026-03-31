//! Files error types.

/// Result alias for file operations.
pub type Result<T> = std::result::Result<T, FilesError>;

/// Errors that can occur during file operations.
#[derive(Debug, thiserror::Error)]
pub enum FilesError {
    /// The file or directory was not found.
    #[error("not found: {0}")]
    NotFound(String),

    /// Permission denied for the operation.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// The requested path violates sandboxing rules.
    #[error("path violation: {0}")]
    PathViolation(String),

    /// The operation would exceed a quota limit.
    #[error("quota exceeded: {0}")]
    QuotaExceeded(String),

    /// An I/O error occurred during the operation.
    #[error("I/O error: {0}")]
    IoError(String),

    /// An unexpected internal error.
    #[error("internal error")]
    Internal,
}
