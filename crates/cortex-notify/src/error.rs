//! Notification error types.

/// Result alias for notification operations.
pub type Result<T> = std::result::Result<T, NotifyError>;

/// Errors that can occur during notification operations.
#[derive(Debug, thiserror::Error)]
pub enum NotifyError {
    /// The requested notification was not found.
    #[error("not found: {0}")]
    NotFound(String),
    /// The caller does not have permission for this operation.
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    /// An unexpected internal error.
    #[error("internal error")]
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display() {
        let err = NotifyError::NotFound("notification abc".to_string());
        assert_eq!(format!("{err}"), "not found: notification abc");
    }

    #[test]
    fn permission_denied_display() {
        let err = NotifyError::PermissionDenied("user cannot dismiss".to_string());
        assert_eq!(format!("{err}"), "permission denied: user cannot dismiss");
    }

    #[test]
    fn internal_display() {
        let err = NotifyError::Internal;
        assert_eq!(format!("{err}"), "internal error");
    }
}
