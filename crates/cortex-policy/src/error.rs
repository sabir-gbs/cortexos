//! Policy error types.

/// Result alias for policy operations.
pub type Result<T> = std::result::Result<T, PolicyError>;

/// Errors that can occur during policy evaluation.
#[derive(Debug, thiserror::Error)]
pub enum PolicyError {
    /// The requested permission is not granted.
    #[error("permission denied: {permission}")]
    PermissionDenied {
        /// The permission that was denied.
        permission: String,
    },

    /// A policy rule was violated.
    #[error("policy violation: {reason}")]
    PolicyViolation {
        /// Why the policy was violated.
        reason: String,
    },

    /// The requested grant does not exist.
    #[error("grant not found")]
    GrantNotFound,

    /// An unexpected internal error.
    #[error("internal policy error: {0}")]
    Internal(String),
}
