//! Core error types shared across all CortexOS crates.
//!
//! Implements the error taxonomy from spec 02, section 8.2.
//! Every API endpoint returns errors in the universal CortexError format.

use serde::{Deserialize, Serialize};

/// Result alias for core operations.
pub type Result<T> = std::result::Result<T, CoreError>;

/// Error categories matching spec 02 section 8.1.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    Transient,
    Permanent,
    Auth,
    Policy,
    NotFound,
    Conflict,
    RateLimit,
    QuotaExceeded,
    ProviderUnavailable,
    Timeout,
    Validation,
}

/// Canonical error codes from spec 02 section 8.2.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    // Transient
    #[serde(rename = "TRANS_001")]
    Trans001,
    // Permanent
    #[serde(rename = "PERM_001")]
    Perm001,
    // Auth
    #[serde(rename = "AUTH_001")]
    Auth001,
    #[serde(rename = "AUTH_002")]
    Auth002,
    // Policy
    #[serde(rename = "POL_001")]
    Pol001,
    #[serde(rename = "POL_002")]
    Pol002,
    // NotFound
    #[serde(rename = "NF_001")]
    Nf001,
    #[serde(rename = "NF_002")]
    Nf002,
    // Conflict
    #[serde(rename = "CONF_001")]
    Conf001,
    #[serde(rename = "CONF_002")]
    Conf002,
    // RateLimit
    #[serde(rename = "RL_001")]
    Rl001,
    // QuotaExceeded
    #[serde(rename = "QE_001")]
    Qe001,
    #[serde(rename = "QE_002")]
    Qe002,
    // ProviderUnavailable
    #[serde(rename = "PU_001")]
    Pu001,
    #[serde(rename = "PU_002")]
    Pu002,
    // Timeout
    #[serde(rename = "TM_001")]
    Tm001,
    #[serde(rename = "TM_002")]
    Tm002,
    // Validation
    #[serde(rename = "VAL_001")]
    Val001,
    #[serde(rename = "VAL_002")]
    Val002,
    #[serde(rename = "VAL_003")]
    Val003,
}

impl ErrorCode {
    /// Returns the error category for this code.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Trans001 => ErrorCategory::Transient,
            Self::Perm001 => ErrorCategory::Permanent,
            Self::Auth001 | Self::Auth002 => ErrorCategory::Auth,
            Self::Pol001 | Self::Pol002 => ErrorCategory::Policy,
            Self::Nf001 | Self::Nf002 => ErrorCategory::NotFound,
            Self::Conf001 | Self::Conf002 => ErrorCategory::Conflict,
            Self::Rl001 => ErrorCategory::RateLimit,
            Self::Qe001 | Self::Qe002 => ErrorCategory::QuotaExceeded,
            Self::Pu001 | Self::Pu002 => ErrorCategory::ProviderUnavailable,
            Self::Tm001 | Self::Tm002 => ErrorCategory::Timeout,
            Self::Val001 | Self::Val002 | Self::Val003 => ErrorCategory::Validation,
        }
    }

    /// Returns the HTTP status code for this error code.
    pub fn http_status(&self) -> u16 {
        match self {
            Self::Trans001 => 503,
            Self::Perm001 => 500,
            Self::Auth001 => 401,
            Self::Auth002 => 403,
            Self::Pol001 | Self::Pol002 => 403,
            Self::Nf001 | Self::Nf002 => 404,
            Self::Conf001 | Self::Conf002 => 409,
            Self::Rl001 => 429,
            Self::Qe001 | Self::Qe002 => 429,
            Self::Pu001 | Self::Pu002 => 502,
            Self::Tm001 | Self::Tm002 => 504,
            Self::Val001 | Self::Val002 => 400,
            Self::Val003 => 422,
        }
    }

    /// Whether the client should retry this error.
    pub fn retryable(&self) -> bool {
        matches!(
            self,
            Self::Trans001 | Self::Rl001 | Self::Pu001 | Self::Pu002 | Self::Tm001 | Self::Tm002
        )
    }

    /// Returns the canonical code string (e.g. "TRANS_001").
    pub fn code_str(&self) -> &'static str {
        match self {
            Self::Trans001 => "TRANS_001",
            Self::Perm001 => "PERM_001",
            Self::Auth001 => "AUTH_001",
            Self::Auth002 => "AUTH_002",
            Self::Pol001 => "POL_001",
            Self::Pol002 => "POL_002",
            Self::Nf001 => "NF_001",
            Self::Nf002 => "NF_002",
            Self::Conf001 => "CONF_001",
            Self::Conf002 => "CONF_002",
            Self::Rl001 => "RL_001",
            Self::Qe001 => "QE_001",
            Self::Qe002 => "QE_002",
            Self::Pu001 => "PU_001",
            Self::Pu002 => "PU_002",
            Self::Tm001 => "TM_001",
            Self::Tm002 => "TM_002",
            Self::Val001 => "VAL_001",
            Self::Val002 => "VAL_002",
            Self::Val003 => "VAL_003",
        }
    }
}

/// Universal error response format from spec 02 section 8.1.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexError {
    pub error: CortexErrorBody,
}

/// Inner body of the canonical error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexErrorBody {
    pub code: String,
    pub category: ErrorCategory,
    pub message: String,
    pub details: serde_json::Value,
    pub retryable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
}

/// Standard success response envelope from spec 02 section 8.3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

/// Metadata included in success responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub request_id: String,
    pub timestamp: String,
}

/// Foundation error type for cross-cutting failures.
///
/// This is the internal error representation. The API layer converts
/// these into the canonical [`CortexError`] response format.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// A required resource was not found.
    #[error("not found: {0}")]
    NotFound(String),
    /// Input validation failed.
    #[error("validation failed: {0}")]
    Validation(String),
    /// An unexpected internal error occurred.
    #[error("internal error: {0}")]
    Internal(String),
}

impl CoreError {
    /// Convert to a canonical CortexError for API responses.
    pub fn to_cortex_error(&self, _request_id: &str) -> CortexError {
        let (code, message) = match self {
            Self::NotFound(msg) => (ErrorCode::Nf001, msg.clone()),
            Self::Validation(msg) => (ErrorCode::Val001, msg.clone()),
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
    fn error_code_category_mapping() {
        assert_eq!(ErrorCode::Trans001.category(), ErrorCategory::Transient);
        assert_eq!(ErrorCode::Auth001.category(), ErrorCategory::Auth);
        assert_eq!(ErrorCode::Pol001.category(), ErrorCategory::Policy);
        assert_eq!(ErrorCode::Nf001.category(), ErrorCategory::NotFound);
        assert_eq!(ErrorCode::Val001.category(), ErrorCategory::Validation);
    }

    #[test]
    fn error_code_http_status() {
        assert_eq!(ErrorCode::Trans001.http_status(), 503);
        assert_eq!(ErrorCode::Auth001.http_status(), 401);
        assert_eq!(ErrorCode::Nf001.http_status(), 404);
        assert_eq!(ErrorCode::Conf001.http_status(), 409);
        assert_eq!(ErrorCode::Val003.http_status(), 422);
    }

    #[test]
    fn error_code_retryable() {
        assert!(ErrorCode::Trans001.retryable());
        assert!(ErrorCode::Pu001.retryable());
        assert!(ErrorCode::Tm001.retryable());
        assert!(!ErrorCode::Auth001.retryable());
        assert!(!ErrorCode::Val001.retryable());
        assert!(!ErrorCode::Pol001.retryable());
    }

    #[test]
    fn core_error_to_cortex_error() {
        let err = CoreError::NotFound("user not found".to_string());
        let cortex_err = err.to_cortex_error("req-123");
        assert_eq!(cortex_err.error.code, "NF_001");
        assert_eq!(cortex_err.error.category, ErrorCategory::NotFound);
        assert_eq!(cortex_err.error.message, "user not found");
        assert!(!cortex_err.error.retryable);
    }

    #[test]
    fn success_response_serialization() {
        let resp = SuccessResponse {
            data: 42,
            meta: Some(ResponseMeta {
                request_id: "req-1".to_string(),
                timestamp: "2026-03-30T00:00:00Z".to_string(),
            }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"data\":42"));
        assert!(json.contains("\"request_id\":\"req-1\""));
    }

    #[test]
    fn cortex_error_serialization() {
        let err = CortexError {
            error: CortexErrorBody {
                code: "RL_001".to_string(),
                category: ErrorCategory::RateLimit,
                message: "rate limit exceeded".to_string(),
                details: serde_json::Value::Null,
                retryable: true,
                retry_after_ms: Some(1000),
            },
        };
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("\"retry_after_ms\":1000"));
        assert!(json.contains("\"retryable\":true"));
    }

    #[test]
    fn all_error_codes_have_valid_status() {
        let codes = [
            ErrorCode::Trans001,
            ErrorCode::Perm001,
            ErrorCode::Auth001,
            ErrorCode::Auth002,
            ErrorCode::Pol001,
            ErrorCode::Pol002,
            ErrorCode::Nf001,
            ErrorCode::Nf002,
            ErrorCode::Conf001,
            ErrorCode::Conf002,
            ErrorCode::Rl001,
            ErrorCode::Qe001,
            ErrorCode::Qe002,
            ErrorCode::Pu001,
            ErrorCode::Pu002,
            ErrorCode::Tm001,
            ErrorCode::Tm002,
            ErrorCode::Val001,
            ErrorCode::Val002,
            ErrorCode::Val003,
        ];
        for code in &codes {
            let status = code.http_status();
            assert!(
                (400..600).contains(&status),
                "invalid status {status} for {code:?}"
            );
        }
    }
}
