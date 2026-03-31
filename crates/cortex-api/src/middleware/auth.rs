//! Authentication middleware.
//!
//! Framework-agnostic session validation and user extraction.
//! Called by the HTTP layer before route handlers that require auth.

use crate::app_state::AppState;
use crate::error::ApiError;
use cortex_auth::AuthService;
use cortex_core::{SessionToken, UserId};

/// Outcome of authenticating a request.
#[derive(Debug, Clone)]
pub struct AuthContext {
    /// The authenticated user.
    pub user_id: UserId,
    /// The session token that was validated.
    pub session_token: String,
}

/// Validate a session token and return the authenticated context.
///
/// Returns `Ok(AuthContext)` when the token is valid and not expired,
/// or an `ApiError::Unauthorized` when the session is missing or invalid.
pub async fn require_auth(
    state: &AppState,
    token: &str,
) -> std::result::Result<AuthContext, ApiError> {
    if token.is_empty() {
        return Err(ApiError::Unauthorized("missing session token".to_string()));
    }

    let session_token = SessionToken(token.to_string());
    let session = state
        .auth
        .validate_session(&session_token)
        .await
        .map_err(|e| match e {
            cortex_auth::AuthError::InvalidCredentials
            | cortex_auth::AuthError::SessionExpired
            | cortex_auth::AuthError::SessionInvalid
            | cortex_auth::AuthError::Unauthorized
            | cortex_auth::AuthError::UserNotFound => {
                ApiError::Unauthorized("invalid or expired session".to_string())
            }
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(AuthContext {
        user_id: session.user_id,
        session_token: token.to_string(),
    })
}

/// Extract a bearer token from an Authorization header value.
///
/// Accepts the form `Bearer <token>`. Returns `None` if the header is
/// missing, malformed, or uses a different scheme.
pub fn extract_bearer_token(auth_header: &str) -> Option<&str> {
    let stripped = auth_header.strip_prefix("Bearer ")?;
    let token = stripped.trim();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_bearer_valid() {
        assert_eq!(extract_bearer_token("Bearer abc123"), Some("abc123"));
    }

    #[test]
    fn extract_bearer_with_whitespace() {
        assert_eq!(extract_bearer_token("Bearer   abc123  "), Some("abc123"));
    }

    #[test]
    fn extract_bearer_missing_prefix() {
        assert_eq!(extract_bearer_token("abc123"), None);
    }

    #[test]
    fn extract_bearer_wrong_scheme() {
        assert_eq!(extract_bearer_token("Basic abc123"), None);
    }

    #[test]
    fn extract_bearer_empty_token() {
        assert_eq!(extract_bearer_token("Bearer "), None);
    }

    #[test]
    fn extract_bearer_empty_string() {
        assert_eq!(extract_bearer_token(""), None);
    }
}
