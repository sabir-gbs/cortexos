//! Authentication service trait.

use crate::error::Result;
use crate::types::{LoginRequest, ProfileUpdate, Session, UserProfile};
use cortex_core::{SessionToken, UserId};
use std::future::Future;
use std::pin::Pin;

/// Authentication service trait.
///
/// All authentication operations go through this trait. The concrete
/// implementation is constructed in cortex-api and injected where needed.
pub trait AuthService: Send + Sync {
    /// Authenticate a user with username and password, returning a new session.
    fn login(
        &self,
        req: LoginRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Session>> + Send + '_>>;

    /// End a user session identified by its token.
    fn logout(&self, token: &SessionToken)
        -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;

    /// Validate a session token and return the active session.
    fn validate_session(
        &self,
        token: &SessionToken,
    ) -> Pin<Box<dyn Future<Output = Result<Session>> + Send + '_>>;

    /// Get a user's profile by their ID.
    fn get_profile(
        &self,
        user_id: &UserId,
    ) -> Pin<Box<dyn Future<Output = Result<UserProfile>> + Send + '_>>;

    /// Update a user's profile fields.
    fn update_profile(
        &self,
        user_id: &UserId,
        update: ProfileUpdate,
    ) -> Pin<Box<dyn Future<Output = Result<UserProfile>> + Send + '_>>;

    /// Change a user's password after verifying the current password.
    fn change_password(
        &self,
        user_id: &UserId,
        current: &str,
        new: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AuthError;
    use crate::types::{LoginRequest, ProfileUpdate, Session, UserProfile};
    use cortex_core::{SessionToken, UserId};

    /// A minimal stub implementation used solely to verify the trait compiles
    /// and can be used as a trait object.
    struct StubAuthService;

    impl AuthService for StubAuthService {
        fn login(
            &self,
            _req: LoginRequest,
        ) -> Pin<Box<dyn Future<Output = Result<Session>> + Send + '_>> {
            Box::pin(async { Err(AuthError::InvalidCredentials) })
        }

        fn logout(
            &self,
            _token: &SessionToken,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }

        fn validate_session(
            &self,
            _token: &SessionToken,
        ) -> Pin<Box<dyn Future<Output = Result<Session>> + Send + '_>> {
            Box::pin(async { Err(AuthError::SessionInvalid) })
        }

        fn get_profile(
            &self,
            _user_id: &UserId,
        ) -> Pin<Box<dyn Future<Output = Result<UserProfile>> + Send + '_>> {
            Box::pin(async { Err(AuthError::UserNotFound) })
        }

        fn update_profile(
            &self,
            _user_id: &UserId,
            _update: ProfileUpdate,
        ) -> Pin<Box<dyn Future<Output = Result<UserProfile>> + Send + '_>> {
            Box::pin(async { Err(AuthError::UserNotFound) })
        }

        fn change_password(
            &self,
            _user_id: &UserId,
            _current: &str,
            _new: &str,
        ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }
    }

    #[tokio::test]
    async fn stub_login_returns_invalid_credentials() {
        let svc = StubAuthService;
        let req = LoginRequest {
            username: "nobody".to_string(),
            password: "wrong".to_string(),
        };
        let result = svc.login(req).await;
        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    #[tokio::test]
    async fn stub_logout_succeeds() {
        let svc = StubAuthService;
        let token = SessionToken("tok".to_string());
        let result = svc.logout(&token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn stub_validate_session_returns_invalid() {
        let svc = StubAuthService;
        let token = SessionToken("missing".to_string());
        let result = svc.validate_session(&token).await;
        assert!(matches!(result, Err(AuthError::SessionInvalid)));
    }

    #[tokio::test]
    async fn stub_get_profile_returns_not_found() {
        let svc = StubAuthService;
        let uid = UserId(uuid::Uuid::now_v7());
        let result = svc.get_profile(&uid).await;
        assert!(matches!(result, Err(AuthError::UserNotFound)));
    }

    #[tokio::test]
    async fn stub_change_password_succeeds() {
        let svc = StubAuthService;
        let uid = UserId(uuid::Uuid::now_v7());
        let result = svc.change_password(&uid, "old", "new").await;
        assert!(result.is_ok());
    }

    #[test]
    fn trait_is_object_safe() {
        // Verify the trait can be used as a dynamic trait object.
        fn _assert_object_safe(_svc: &dyn AuthService) {}
        _assert_object_safe(&StubAuthService);
    }
}
