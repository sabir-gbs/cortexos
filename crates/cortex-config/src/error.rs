//! Configuration error types.

/// Result alias for configuration operations.
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Errors that can occur during configuration loading or validation.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// A required configuration value is missing.
    #[error("missing configuration: {0}")]
    Missing(String),
    /// A configuration value failed validation.
    #[error("invalid configuration: {0}")]
    Validation(String),
    /// An IO error occurred reading configuration.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    /// An unexpected internal error.
    #[error("internal error: {0}")]
    Internal(String),
}
