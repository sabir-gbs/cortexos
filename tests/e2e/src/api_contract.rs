//! API contract validation tests.
//! Verifies error responses match the canonical CortexError format.

#[test]
fn cortex_error_matches_spec_format() {
    use cortex_core::{CortexError, CortexErrorBody, ErrorCategory, ErrorCode};

    let error = CortexError {
        error: CortexErrorBody {
            code: ErrorCode::Auth001.code_str().to_string(),
            category: ErrorCategory::Auth,
            message: "not authenticated".to_string(),
            details: serde_json::Value::Null,
            retryable: false,
            retry_after_ms: None,
        },
    };

    let json = serde_json::to_string(&error).unwrap();
    assert!(json.contains("\"code\":\"AUTH_001\""));
    assert!(json.contains("\"category\":\"auth\""));
    assert!(json.contains("\"retryable\":false"));
}

#[test]
fn all_error_codes_are_documented() {
    use cortex_core::ErrorCode;
    // Verify every error code has a non-empty code_str
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
        assert!(!code.code_str().is_empty());
        assert!(code.http_status() >= 400);
        assert!(code.http_status() < 600);
    }
}
