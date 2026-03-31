//! Admin error types.

/// Result alias for admin operations.
pub type Result<T> = std::result::Result<T, AdminError>;

/// Errors that can occur during admin operations.
#[derive(Debug, thiserror::Error)]
pub enum AdminError {
    /// A requested resource was not found.
    #[error("not found: {0}")]
    NotFound(String),
    /// A backup operation failed.
    #[error("backup failed: {0}")]
    BackupFailed(String),
    /// A restore operation failed.
    #[error("restore failed: {0}")]
    RestoreFailed(String),
    /// A metrics collection operation failed.
    #[error("metrics error: {0}")]
    MetricsError(String),
    /// A session recovery operation failed.
    #[error("session error: {0}")]
    SessionError(String),
    /// A crash handling operation failed.
    #[error("crash handler error: {0}")]
    CrashError(String),
    /// A filesystem check or repair operation failed.
    #[error("filesystem error: {0}")]
    FilesystemError(String),
    /// A diagnostic export operation failed.
    #[error("export failed: {0}")]
    ExportFailed(String),
    /// Factory reset failed.
    #[error("factory reset failed: {0}")]
    FactoryResetFailed(String),
    /// An unexpected internal error.
    #[error("internal error")]
    Internal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_display() {
        let err = AdminError::NotFound("backup file".to_string());
        assert_eq!(format!("{err}"), "not found: backup file");
    }

    #[test]
    fn backup_failed_display() {
        let err = AdminError::BackupFailed("disk full".to_string());
        assert_eq!(format!("{err}"), "backup failed: disk full");
    }

    #[test]
    fn restore_failed_display() {
        let err = AdminError::RestoreFailed("corrupt archive".to_string());
        assert_eq!(format!("{err}"), "restore failed: corrupt archive");
    }

    #[test]
    fn internal_display() {
        let err = AdminError::Internal;
        assert_eq!(format!("{err}"), "internal error");
    }

    #[test]
    fn metrics_error_display() {
        let err = AdminError::MetricsError("collection failed".to_string());
        assert_eq!(format!("{err}"), "metrics error: collection failed");
    }

    #[test]
    fn session_error_display() {
        let err = AdminError::SessionError("corrupt state".to_string());
        assert_eq!(format!("{err}"), "session error: corrupt state");
    }

    #[test]
    fn export_failed_display() {
        let err = AdminError::ExportFailed("zip write".to_string());
        assert_eq!(format!("{err}"), "export failed: zip write");
    }

    #[test]
    fn factory_reset_failed_display() {
        let err = AdminError::FactoryResetFailed("step 5".to_string());
        assert_eq!(format!("{err}"), "factory reset failed: step 5");
    }
}
