//! Window manager error types.

/// Errors produced by window manager operations.
#[derive(Debug, thiserror::Error)]
pub enum WmError {
    #[error("window not found: {0}")]
    WindowNotFound(String),
    #[error("workspace not found: {0}")]
    WorkspaceNotFound(String),
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
    #[error("database error: {0}")]
    Db(#[from] cortex_db::error::DbError),
}

/// Result type alias for window manager operations.
pub type Result<T> = std::result::Result<T, WmError>;
