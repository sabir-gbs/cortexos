//! Authentication domain types.

use crate::error::{AuthError, Result};
use cortex_core::{SessionId, SessionToken, Timestamp, UserId};
use serde::{Deserialize, Serialize};

/// Login request payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    /// The user's username.
    pub username: String,
    /// The user's plaintext password (never stored).
    pub password: String,
}

impl LoginRequest {
    /// Validate the login request fields.
    ///
    /// Returns an error if the username or password is empty or only whitespace.
    pub fn validate(&self) -> Result<()> {
        if self.username.trim().is_empty() {
            return Err(AuthError::InvalidCredentials);
        }
        if self.password.trim().is_empty() {
            return Err(AuthError::InvalidCredentials);
        }
        Ok(())
    }
}

/// Active user session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique session identifier.
    pub session_id: SessionId,
    /// The user this session belongs to.
    pub user_id: UserId,
    /// Opaque session token used for authentication.
    pub token: SessionToken,
    /// When the session was created.
    pub created_at: Timestamp,
    /// When the session expires.
    pub expires_at: Timestamp,
}

/// User profile information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    /// The user's unique ID.
    pub user_id: UserId,
    /// Login username.
    pub username: String,
    /// Display name shown in the UI.
    pub display_name: String,
    /// Account creation timestamp.
    pub created_at: Timestamp,
}

/// Profile update payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileUpdate {
    /// New display name.
    pub display_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_core::{SessionId, SessionToken, UserId};

    #[test]
    fn login_request_serde_roundtrip() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "hunter2".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        let decoded: LoginRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.username, "alice");
        assert_eq!(decoded.password, "hunter2");
    }

    #[test]
    fn session_serde_roundtrip() {
        let session = Session {
            session_id: SessionId(uuid::Uuid::now_v7()),
            user_id: UserId(uuid::Uuid::now_v7()),
            token: SessionToken("tok_abc123".to_string()),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            expires_at: "2025-01-02T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&session).unwrap();
        let decoded: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.token.0, "tok_abc123");
    }

    #[test]
    fn user_profile_serde_roundtrip() {
        let profile = UserProfile {
            user_id: UserId(uuid::Uuid::now_v7()),
            username: "bob".to_string(),
            display_name: "Bob Smith".to_string(),
            created_at: "2025-06-15T12:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&profile).unwrap();
        let decoded: UserProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.username, "bob");
        assert_eq!(decoded.display_name, "Bob Smith");
    }

    #[test]
    fn profile_update_serde_roundtrip() {
        let update = ProfileUpdate {
            display_name: Some("New Name".to_string()),
        };
        let json = serde_json::to_string(&update).unwrap();
        let decoded: ProfileUpdate = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.display_name, Some("New Name".to_string()));
    }

    #[test]
    fn profile_update_none_serde_roundtrip() {
        let update = ProfileUpdate { display_name: None };
        let json = serde_json::to_string(&update).unwrap();
        let decoded: ProfileUpdate = serde_json::from_str(&json).unwrap();
        assert!(decoded.display_name.is_none());
    }

    #[test]
    fn login_request_validate_ok() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "secret".to_string(),
        };
        assert!(req.validate().is_ok());
    }

    #[test]
    fn login_request_validate_empty_username() {
        let req = LoginRequest {
            username: "   ".to_string(),
            password: "secret".to_string(),
        };
        assert!(matches!(req.validate(), Err(AuthError::InvalidCredentials)));
    }

    #[test]
    fn login_request_validate_empty_password() {
        let req = LoginRequest {
            username: "alice".to_string(),
            password: "".to_string(),
        };
        assert!(matches!(req.validate(), Err(AuthError::InvalidCredentials)));
    }

    #[test]
    fn login_request_validate_both_empty() {
        let req = LoginRequest {
            username: String::new(),
            password: String::new(),
        };
        assert!(matches!(req.validate(), Err(AuthError::InvalidCredentials)));
    }
}
