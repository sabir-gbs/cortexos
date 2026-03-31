//! Database error types.

/// Result alias for database operations.
pub type Result<T> = std::result::Result<T, DbError>;

/// Errors that can occur during database operations.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// A query returned no results when at least one was expected.
    #[error("not found: {0}")]
    NotFound(String),
    /// A database migration failed.
    #[error("migration error: {0}")]
    Migration(String),
    /// A connection pool error.
    #[error("connection error: {0}")]
    Connection(String),
    /// A query execution error.
    #[error("query error: {0}")]
    Query(String),
    /// An unexpected internal error.
    #[error("internal error: {0}")]
    Internal(String),
}
