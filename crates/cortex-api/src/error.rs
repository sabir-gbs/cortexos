//! API error types and HTTP status mapping.

use cortex_core::{CortexError, CortexErrorBody, ErrorCode};

#[cfg(test)]
use cortex_core::ErrorCategory;

/// Result alias for API operations.
pub type Result<T> = std::result::Result<T, ApiError>;

/// Errors that can occur at the API layer.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Authentication required.
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    /// Permission denied.
    #[error("forbidden: {0}")]
    Forbidden(String),
    /// Resource not found.
    #[error("not found: {0}")]
    NotFound(String),
    /// Invalid request.
    #[error("bad request: {0}")]
    BadRequest(String),
    /// Internal server error.
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    /// Convert this API error into the canonical [`CortexError`] response
    /// format used across all CortexOS endpoints.
    pub fn to_cortex_error(&self) -> CortexError {
        let (code, message) = match self {
            Self::Unauthorized(msg) => (ErrorCode::Auth001, msg.clone()),
            Self::Forbidden(msg) => (ErrorCode::Auth002, msg.clone()),
            Self::NotFound(msg) => (ErrorCode::Nf001, msg.clone()),
            Self::BadRequest(msg) => (ErrorCode::Val001, msg.clone()),
            Self::Internal(msg) => (ErrorCode::Perm001, msg.clone()),
        };

        CortexError {
            error: CortexErrorBody {
                code: code.code_str().to_string(),
                category: code.category(),
                message,
                details: serde_json::Value::Null,
                retryable: code.retryable(),
                retry_after_ms: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unauthorized_to_cortex_error() {
        let err = ApiError::Unauthorized("missing token".to_string());
        let cortex_err = err.to_cortex_error();
        assert_eq!(cortex_err.error.code, "AUTH_001");
        assert_eq!(cortex_err.error.category, ErrorCategory::Auth);
        assert_eq!(cortex_err.error.message, "missing token");
        assert!(!cortex_err.error.retryable);
    }

    #[test]
    fn forbidden_to_cortex_error() {
        let err = ApiError::Forbidden("admin only".to_string());
        let cortex_err = err.to_cortex_error();
        assert_eq!(cortex_err.error.code, "AUTH_002");
        assert_eq!(cortex_err.error.category, ErrorCategory::Auth);
        assert_eq!(cortex_err.error.message, "admin only");
    }

    #[test]
    fn not_found_to_cortex_error() {
        let err = ApiError::NotFound("user 42".to_string());
        let cortex_err = err.to_cortex_error();
        assert_eq!(cortex_err.error.code, "NF_001");
        assert_eq!(cortex_err.error.category, ErrorCategory::NotFound);
        assert_eq!(cortex_err.error.message, "user 42");
    }

    #[test]
    fn bad_request_to_cortex_error() {
        let err = ApiError::BadRequest("invalid json".to_string());
        let cortex_err = err.to_cortex_error();
        assert_eq!(cortex_err.error.code, "VAL_001");
        assert_eq!(cortex_err.error.category, ErrorCategory::Validation);
        assert_eq!(cortex_err.error.message, "invalid json");
    }

    #[test]
    fn internal_to_cortex_error() {
        let err = ApiError::Internal("database connection lost".to_string());
        let cortex_err = err.to_cortex_error();
        assert_eq!(cortex_err.error.code, "PERM_001");
        assert_eq!(cortex_err.error.category, ErrorCategory::Permanent);
        assert_eq!(cortex_err.error.message, "database connection lost");
    }

    #[test]
    fn error_display_messages() {
        assert_eq!(
            format!("{}", ApiError::Unauthorized("no auth".to_string())),
            "unauthorized: no auth"
        );
        assert_eq!(
            format!("{}", ApiError::Forbidden("denied".to_string())),
            "forbidden: denied"
        );
        assert_eq!(
            format!("{}", ApiError::NotFound("gone".to_string())),
            "not found: gone"
        );
        assert_eq!(
            format!("{}", ApiError::BadRequest("bad".to_string())),
            "bad request: bad"
        );
        assert_eq!(
            format!("{}", ApiError::Internal("oops".to_string())),
            "internal error: oops"
        );
    }
}
