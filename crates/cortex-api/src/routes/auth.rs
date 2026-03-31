//! Authentication API routes.
//!
//! Framework-agnostic handler functions for login, logout, profile
//! management, and password changes.

use crate::app_state::AppState;
use crate::error::{ApiError, Result};
use cortex_auth::types::{LoginRequest, ProfileUpdate};
use cortex_auth::AuthService;
use cortex_core::{SessionToken, SuccessResponse, UserId};
use serde::{Deserialize, Serialize};

/// Response body for a successful login.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub session_id: String,
    pub user_id: String,
    pub expires_at: String,
}

/// The cookie name used for the session token.
pub const SESSION_COOKIE_NAME: &str = "cortex_session";

/// Build a Set-Cookie header value for the session token.
pub fn session_cookie_value(token: &str) -> String {
    format!("{SESSION_COOKIE_NAME}={token}; HttpOnly; Secure; SameSite=Strict; Path=/")
}

/// Response body for profile queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileResponse {
    pub user_id: String,
    pub username: String,
    pub display_name: String,
    pub created_at: String,
}

/// Login handler.
pub async fn login(state: &AppState, req: LoginRequest) -> Result<SuccessResponse<LoginResponse>> {
    let session = state.auth.login(req).await.map_err(|e| match e {
        cortex_auth::AuthError::InvalidCredentials => {
            ApiError::Unauthorized("invalid credentials".to_string())
        }
        cortex_auth::AuthError::PasswordTooWeak { min_length } => {
            ApiError::BadRequest(format!("password must be at least {min_length} characters"))
        }
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: LoginResponse {
            session_id: session.session_id.0.to_string(),
            user_id: session.user_id.0.to_string(),
            expires_at: session.expires_at,
        },
        meta: None,
    })
}

/// Logout handler.
pub async fn logout(state: &AppState, token: &str) -> Result<SuccessResponse<()>> {
    state
        .auth
        .logout(&SessionToken(token.to_string()))
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// Get profile handler.
pub async fn get_profile(
    state: &AppState,
    user_id: &UserId,
) -> Result<SuccessResponse<ProfileResponse>> {
    let profile = state.auth.get_profile(user_id).await.map_err(|e| match e {
        cortex_auth::AuthError::UserNotFound => ApiError::NotFound("user not found".to_string()),
        _ => ApiError::Internal(e.to_string()),
    })?;

    Ok(SuccessResponse {
        data: ProfileResponse {
            user_id: profile.user_id.0.to_string(),
            username: profile.username,
            display_name: profile.display_name,
            created_at: profile.created_at,
        },
        meta: None,
    })
}

/// Update profile handler.
pub async fn update_profile(
    state: &AppState,
    user_id: &UserId,
    update: ProfileUpdate,
) -> Result<SuccessResponse<ProfileResponse>> {
    let profile = state
        .auth
        .update_profile(user_id, update)
        .await
        .map_err(|e| ApiError::Internal(e.to_string()))?;

    Ok(SuccessResponse {
        data: ProfileResponse {
            user_id: profile.user_id.0.to_string(),
            username: profile.username,
            display_name: profile.display_name,
            created_at: profile.created_at,
        },
        meta: None,
    })
}

/// Change password handler.
pub async fn change_password(
    state: &AppState,
    user_id: &UserId,
    current: &str,
    new: &str,
) -> Result<SuccessResponse<()>> {
    state
        .auth
        .change_password(user_id, current, new)
        .await
        .map_err(|e| match e {
            cortex_auth::AuthError::InvalidCredentials => {
                ApiError::Unauthorized("current password is incorrect".to_string())
            }
            cortex_auth::AuthError::PasswordTooWeak { min_length } => {
                ApiError::BadRequest(format!("password must be at least {min_length} characters"))
            }
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(SuccessResponse {
        data: (),
        meta: None,
    })
}

/// Validate session and return the user ID.
pub async fn validate_session(state: &AppState, token: &str) -> Result<UserId> {
    let session = state
        .auth
        .validate_session(&SessionToken(token.to_string()))
        .await
        .map_err(|e| match e {
            cortex_auth::AuthError::SessionInvalid | cortex_auth::AuthError::SessionExpired => {
                ApiError::Unauthorized("invalid or expired session".to_string())
            }
            _ => ApiError::Internal(e.to_string()),
        })?;

    Ok(session.user_id)
}

/// "Who am I?" handler -- returns the profile of the currently authenticated user.
/// Used by the desktop shell to obtain user info after a cookie-based login.
pub async fn me(state: &AppState, token: &str) -> Result<SuccessResponse<ProfileResponse>> {
    let user_id = validate_session(state, token).await?;
    get_profile(state, &user_id).await
}
