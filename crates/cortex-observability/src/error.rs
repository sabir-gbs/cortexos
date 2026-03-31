//! Observability error types.

/// Result alias for observability operations.
pub type Result<T> = std::result::Result<T, ObservabilityError>;

/// Errors that can occur in observability operations.
#[derive(Debug, thiserror::Error)]
pub enum ObservabilityError {
    /// An unexpected internal error.
    #[error("internal error: {0}")]
    Internal(String),
}
