//! Authentication error types.

/// Result alias for auth operations.
pub type Result<T> = std::result::Result<T, AuthError>;

/// Errors that can occur during authentication operations.
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    /// The provided credentials are invalid.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// The session has expired and is no longer valid.
    #[error("session expired")]
    SessionExpired,

    /// The session token is invalid or does not exist.
    #[error("session invalid")]
    SessionInvalid,

    /// The user is not authorized to perform this action.
    #[error("unauthorized")]
    Unauthorized,

    /// The new password does not meet strength requirements.
    #[error("password too weak: must be at least {min_length} characters")]
    PasswordTooWeak {
        /// Minimum password length required.
        min_length: usize,
    },

    /// The requested user was not found.
    #[error("user not found")]
    UserNotFound,

    /// An unexpected internal error occurred.
    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(
            AuthError::InvalidCredentials.to_string(),
            "invalid credentials"
        );
        assert_eq!(AuthError::SessionExpired.to_string(), "session expired");
        assert_eq!(AuthError::SessionInvalid.to_string(), "session invalid");
        assert_eq!(AuthError::Unauthorized.to_string(), "unauthorized");
        assert_eq!(
            AuthError::PasswordTooWeak { min_length: 8 }.to_string(),
            "password too weak: must be at least 8 characters"
        );
        assert_eq!(AuthError::UserNotFound.to_string(), "user not found");
        assert_eq!(
            AuthError::Internal("database unreachable".to_string()).to_string(),
            "internal error: database unreachable"
        );
    }

    #[test]
    fn result_alias_ok() {
        let ok: Result<i32> = Ok(42);
        assert!(ok.is_ok());
        assert_eq!(ok.as_ref().unwrap(), &42);
    }

    #[test]
    fn result_alias_err() {
        let err: Result<i32> = Err(AuthError::UserNotFound);
        assert!(err.is_err());
    }
}
