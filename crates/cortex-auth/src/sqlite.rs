//! SQLite-backed implementation of [`AuthService`].

use std::future::Future;
use std::pin::Pin;

use chrono::Utc;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use cortex_core::{SessionId, SessionToken, UserId};

use crate::error::{AuthError, Result};
use crate::service::AuthService;
use crate::types::{LoginRequest, ProfileUpdate, Session, UserProfile};

// ---------------------------------------------------------------------------
// Password hashing helpers
// ---------------------------------------------------------------------------

/// Generate a random 32-byte hex salt.
fn generate_salt() -> String {
    let bytes: [u8; 32] = rand_bytes();
    hex_encode(&bytes)
}

/// Hash `salt + password` with SHA-256 and return the hex digest.
fn hash_password(salt: &str, password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex_encode(&result)
}

/// Build the stored hash string: `{salt}${hex_hash}`.
fn encode_stored_hash(salt: &str, hash: &str) -> String {
    format!("{salt}${hash}")
}

/// Verify a plaintext password against a stored `{salt}${hash}` value.
fn verify_password(stored: &str, plaintext: &str) -> bool {
    let (salt, expected_hash) = match stored.split_once('$') {
        Some(pair) => pair,
        None => return false,
    };
    let candidate = hash_password(salt, plaintext);
    constant_time_eq(expected_hash.as_bytes(), candidate.as_bytes())
}

/// Encode `data` as a lowercase hex string.
fn hex_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for b in data {
        use std::fmt::Write;
        write!(s, "{b:02x}").unwrap();
    }
    s
}

/// Generate 32 random bytes using `uuid::Uuid` as the entropy source.
fn rand_bytes() -> [u8; 32] {
    let mut buf = [0u8; 32];
    // Four UUIDv4s give us 64 bytes of entropy; copy the first 32.
    for chunk in buf.chunks_exact_mut(16) {
        chunk.copy_from_slice(Uuid::now_v7().as_bytes());
    }
    buf
}

// ---------------------------------------------------------------------------
// Constant-time comparison (to avoid timing attacks on password hashes)
// ---------------------------------------------------------------------------

mod constant_time_eq {
    /// Constant-time byte comparison.
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut result: u8 = 0;
        for (x, y) in a.iter().zip(b.iter()) {
            result |= x ^ y;
        }
        result == 0
    }
}

use constant_time_eq::constant_time_eq;

// ---------------------------------------------------------------------------
// SqliteAuthService
// ---------------------------------------------------------------------------

/// SQLite-backed authentication service.
pub struct SqliteAuthService {
    pool: cortex_db::Pool,
    session_ttl_secs: u64,
}

impl SqliteAuthService {
    /// Create a new service backed by the given connection pool.
    ///
    /// `session_ttl_secs` controls how long sessions remain valid.
    pub fn new(pool: cortex_db::Pool, session_ttl_secs: u64) -> Self {
        Self {
            pool,
            session_ttl_secs,
        }
    }

    /// Bootstrap helper: create a new user.
    ///
    /// This method is intentionally **not** on the [`AuthService`] trait
    /// because user provisioning is an admin-only concern.
    pub fn create_user(
        &self,
        username: &str,
        password: &str,
        display_name: &str,
    ) -> Result<UserId> {
        let user_id = UserId(Uuid::now_v7());
        let salt = generate_salt();
        let hash = hash_password(&salt, password);
        let stored = encode_stored_hash(&salt, &hash);

        self.pool
            .write(|conn| {
                conn.execute(
                    "INSERT INTO users (user_id, username, display_name, password_hash) VALUES (?1, ?2, ?3, ?4)",
                    rusqlite::params![user_id.0.to_string(), username, display_name, stored],
                )
                .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                Ok(())
            })
            .map_err(|e| AuthError::Internal(e.to_string()))?;

        Ok(user_id)
    }
}

// ---------------------------------------------------------------------------
// AuthService implementation
// ---------------------------------------------------------------------------

impl AuthService for SqliteAuthService {
    fn login(
        &self,
        req: LoginRequest,
    ) -> Pin<Box<dyn Future<Output = Result<Session>> + Send + '_>> {
        Box::pin(async move {
            req.validate()?;

            // Look up user by username
            let row: Option<(String, String)> = self
                .pool
                .read(|conn| {
                    Ok(conn
                        .query_row(
                            "SELECT user_id, password_hash FROM users WHERE username = ?1",
                            rusqlite::params![req.username],
                            |row| Ok((row.get(0)?, row.get(1)?)),
                        )
                        .ok())
                })
                .map_err(|e| AuthError::Internal(e.to_string()))?;

            let (user_id_str, stored_hash) = match row {
                Some(r) => r,
                None => return Err(AuthError::InvalidCredentials),
            };

            let user_id = Uuid::parse_str(&user_id_str)
                .map(UserId)
                .map_err(|e| AuthError::Internal(format!("invalid user_id uuid: {e}")))?;

            // Verify password
            if !verify_password(&stored_hash, &req.password) {
                return Err(AuthError::InvalidCredentials);
            }

            // Create session
            let session = Session {
                session_id: SessionId(Uuid::now_v7()),
                user_id,
                token: SessionToken(Uuid::now_v7().to_string()),
                created_at: Utc::now().to_rfc3339(),
                expires_at: (Utc::now() + chrono::Duration::seconds(self.session_ttl_secs as i64))
                    .to_rfc3339(),
            };

            self.pool
                .write(|conn| {
                    conn.execute(
                        "INSERT INTO sessions (session_id, user_id, token, created_at, expires_at, last_active_at) \
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![
                            session.session_id.0.to_string(),
                            session.user_id.0.to_string(),
                            session.token.0,
                            session.created_at,
                            session.expires_at,
                            session.created_at,
                        ],
                    )
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                    Ok(())
                })
                .map_err(|e| AuthError::Internal(e.to_string()))?;

            Ok(session)
        })
    }

    fn logout(
        &self,
        token: &SessionToken,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let token = token.clone();
        Box::pin(async move {
            self.pool
                .write(|conn| {
                    conn.execute(
                        "DELETE FROM sessions WHERE token = ?1",
                        rusqlite::params![token.0],
                    )
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                    Ok(())
                })
                .map_err(|e| AuthError::Internal(e.to_string()))?;

            Ok(())
        })
    }

    fn validate_session(
        &self,
        token: &SessionToken,
    ) -> Pin<Box<dyn Future<Output = Result<Session>> + Send + '_>> {
        let token = token.clone();
        Box::pin(async move {
            let row: Option<(String, String, String, String, String)> = self
                .pool
                .read(|conn| {
                    Ok(conn
                        .query_row(
                            "SELECT session_id, user_id, created_at, expires_at, last_active_at \
                             FROM sessions WHERE token = ?1",
                            rusqlite::params![token.0],
                            |row| {
                                Ok((
                                    row.get(0)?,
                                    row.get(1)?,
                                    row.get(2)?,
                                    row.get(3)?,
                                    row.get(4)?,
                                ))
                            },
                        )
                        .ok())
                })
                .map_err(|e| AuthError::Internal(e.to_string()))?;

            let (session_id_str, user_id_str, created_at, expires_at, _last_active_at) = match row {
                Some(r) => r,
                None => return Err(AuthError::SessionInvalid),
            };

            // Check expiry
            let expires = chrono::DateTime::parse_from_rfc3339(&expires_at)
                .map_err(|e| AuthError::Internal(format!("cannot parse expires_at: {e}")))?;
            if Utc::now() > expires {
                return Err(AuthError::SessionExpired);
            }

            let session = Session {
                session_id: SessionId(
                    Uuid::parse_str(&session_id_str)
                        .map_err(|e| AuthError::Internal(format!("invalid session_id: {e}")))?,
                ),
                user_id: UserId(
                    Uuid::parse_str(&user_id_str)
                        .map_err(|e| AuthError::Internal(format!("invalid user_id: {e}")))?,
                ),
                token,
                created_at,
                expires_at,
            };

            // Update last_active_at
            let now = Utc::now().to_rfc3339();
            self.pool
                .write(|conn| {
                    conn.execute(
                        "UPDATE sessions SET last_active_at = ?1 WHERE session_id = ?2",
                        rusqlite::params![now, session.session_id.0.to_string()],
                    )
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                    Ok(())
                })
                .map_err(|e| AuthError::Internal(e.to_string()))?;

            Ok(session)
        })
    }

    fn get_profile(
        &self,
        user_id: &UserId,
    ) -> Pin<Box<dyn Future<Output = Result<UserProfile>> + Send + '_>> {
        let uid_str = user_id.0.to_string();
        let user_id = user_id.clone();
        Box::pin(async move {
            let row: Option<(String, String, String)> = self
                .pool
                .read(|conn| {
                    Ok(conn
                        .query_row(
                            "SELECT username, display_name, created_at FROM users WHERE user_id = ?1",
                            rusqlite::params![uid_str],
                            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
                        )
                        .ok())
                })
                .map_err(|e| AuthError::Internal(e.to_string()))?;

            let (username, display_name, created_at) = match row {
                Some(r) => r,
                None => return Err(AuthError::UserNotFound),
            };

            Ok(UserProfile {
                user_id,
                username,
                display_name,
                created_at,
            })
        })
    }

    fn update_profile(
        &self,
        user_id: &UserId,
        update: ProfileUpdate,
    ) -> Pin<Box<dyn Future<Output = Result<UserProfile>> + Send + '_>> {
        let user_id = user_id.clone();
        let uid_str = user_id.0.to_string();
        Box::pin(async move {
            // Apply update if there is one
            if let Some(ref display_name) = update.display_name {
                let dn = display_name.clone();
                self.pool
                    .write(|conn| {
                        conn.execute(
                            "UPDATE users SET display_name = ?1, updated_at = datetime('now') \
                             WHERE user_id = ?2",
                            rusqlite::params![dn, uid_str],
                        )
                        .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                        Ok(())
                    })
                    .map_err(|e| AuthError::Internal(e.to_string()))?;
            }

            // Fetch and return updated profile
            self.get_profile(&user_id).await
        })
    }

    fn change_password(
        &self,
        user_id: &UserId,
        current: &str,
        new: &str,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        let uid_str = user_id.0.to_string();
        let current = current.to_string();
        let new = new.to_string();
        Box::pin(async move {
            // Validate new password strength
            if new.len() < 8 {
                return Err(AuthError::PasswordTooWeak { min_length: 8 });
            }

            // Fetch current password hash
            let stored_hash: Option<String> = self
                .pool
                .read(|conn| {
                    Ok(conn
                        .query_row(
                            "SELECT password_hash FROM users WHERE user_id = ?1",
                            rusqlite::params![uid_str],
                            |row| row.get(0),
                        )
                        .ok())
                })
                .map_err(|e| AuthError::Internal(e.to_string()))?;

            let stored = match stored_hash {
                Some(h) => h,
                None => return Err(AuthError::UserNotFound),
            };

            // Verify current password
            if !verify_password(&stored, &current) {
                return Err(AuthError::InvalidCredentials);
            }

            // Hash and store new password
            let salt = generate_salt();
            let hash = hash_password(&salt, &new);
            let new_stored = encode_stored_hash(&salt, &hash);

            self.pool
                .write(|conn| {
                    conn.execute(
                        "UPDATE users SET password_hash = ?1, updated_at = datetime('now') \
                         WHERE user_id = ?2",
                        rusqlite::params![new_stored, uid_str],
                    )
                    .map_err(|e| cortex_db::DbError::Query(e.to_string()))?;
                    Ok(())
                })
                .map_err(|e| AuthError::Internal(e.to_string()))?;

            Ok(())
        })
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AuthError;

    /// Helper: create an in-memory database with migrations applied.
    fn setup() -> SqliteAuthService {
        let pool = cortex_db::Pool::open_in_memory().unwrap();
        cortex_db::run_migrations(&pool).unwrap();
        SqliteAuthService::new(pool, 3600) // 1-hour TTL
    }

    // -- create_user + login success ----------------------------------------

    #[tokio::test]
    async fn create_user_and_login_success() {
        let svc = setup();
        let uid = svc.create_user("alice", "password123", "Alice").unwrap();

        let session = svc
            .login(LoginRequest {
                username: "alice".to_string(),
                password: "password123".to_string(),
            })
            .await
            .unwrap();

        assert_eq!(session.user_id, uid);
        assert!(!session.token.0.is_empty());
    }

    // -- login with wrong password ------------------------------------------

    #[tokio::test]
    async fn login_wrong_password() {
        let svc = setup();
        svc.create_user("bob", "correctpass", "Bob").unwrap();

        let result = svc
            .login(LoginRequest {
                username: "bob".to_string(),
                password: "wrongpass".to_string(),
            })
            .await;

        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    // -- login with nonexistent user ----------------------------------------

    #[tokio::test]
    async fn login_nonexistent_user() {
        let svc = setup();

        let result = svc
            .login(LoginRequest {
                username: "nobody".to_string(),
                password: "anything".to_string(),
            })
            .await;

        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    // -- logout invalidates session -----------------------------------------

    #[tokio::test]
    async fn logout_invalidates_session() {
        let svc = setup();
        svc.create_user("carol", "password123", "Carol").unwrap();

        let session = svc
            .login(LoginRequest {
                username: "carol".to_string(),
                password: "password123".to_string(),
            })
            .await
            .unwrap();

        // Validate works before logout
        let valid = svc.validate_session(&session.token).await.unwrap();
        assert_eq!(valid.session_id, session.session_id);

        // Logout
        svc.logout(&session.token).await.unwrap();

        // Validate fails after logout
        let result = svc.validate_session(&session.token).await;
        assert!(matches!(result, Err(AuthError::SessionInvalid)));
    }

    // -- validate_session returns session -----------------------------------

    #[tokio::test]
    async fn validate_session_returns_session() {
        let svc = setup();
        svc.create_user("dave", "password123", "Dave").unwrap();

        let session = svc
            .login(LoginRequest {
                username: "dave".to_string(),
                password: "password123".to_string(),
            })
            .await
            .unwrap();

        let validated = svc.validate_session(&session.token).await.unwrap();
        assert_eq!(validated.user_id, session.user_id);
        assert_eq!(validated.token.0, session.token.0);
    }

    // -- validate_session expired session -----------------------------------

    #[tokio::test]
    async fn validate_session_expired() {
        // Create service with a very short TTL (0 seconds -> already expired)
        let pool = cortex_db::Pool::open_in_memory().unwrap();
        cortex_db::run_migrations(&pool).unwrap();
        let svc = SqliteAuthService::new(pool, 0);

        svc.create_user("eve", "password123", "Eve").unwrap();

        let session = svc
            .login(LoginRequest {
                username: "eve".to_string(),
                password: "password123".to_string(),
            })
            .await
            .unwrap();

        let result = svc.validate_session(&session.token).await;
        assert!(matches!(result, Err(AuthError::SessionExpired)));
    }

    // -- get_profile ---------------------------------------------------------

    #[tokio::test]
    async fn get_profile_returns_user_data() {
        let svc = setup();
        let uid = svc
            .create_user("frank", "password123", "Frank Castle")
            .unwrap();

        let profile = svc.get_profile(&uid).await.unwrap();
        assert_eq!(profile.user_id, uid);
        assert_eq!(profile.username, "frank");
        assert_eq!(profile.display_name, "Frank Castle");
    }

    // -- update_profile ------------------------------------------------------

    #[tokio::test]
    async fn update_profile_changes_display_name() {
        let svc = setup();
        let uid = svc.create_user("grace", "password123", "Grace").unwrap();

        let updated = svc
            .update_profile(
                &uid,
                ProfileUpdate {
                    display_name: Some("Grace Hopper".to_string()),
                },
            )
            .await
            .unwrap();

        assert_eq!(updated.display_name, "Grace Hopper");
    }

    // -- change_password success ---------------------------------------------

    #[tokio::test]
    async fn change_password_success() {
        let svc = setup();
        let uid = svc.create_user("heidi", "oldpassword", "Heidi").unwrap();

        // Change password
        svc.change_password(&uid, "oldpassword", "newpassword123")
            .await
            .unwrap();

        // Login with new password should succeed
        let session = svc
            .login(LoginRequest {
                username: "heidi".to_string(),
                password: "newpassword123".to_string(),
            })
            .await
            .unwrap();
        assert!(!session.token.0.is_empty());
    }

    // -- change_password wrong current password ------------------------------

    #[tokio::test]
    async fn change_password_wrong_current() {
        let svc = setup();
        let uid = svc.create_user("ivan", "correctpass", "Ivan").unwrap();

        let result = svc
            .change_password(&uid, "wrongpass", "newpassword123")
            .await;

        assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    }

    // -- change_password too weak new password -------------------------------

    #[tokio::test]
    async fn change_password_too_weak() {
        let svc = setup();
        let uid = svc.create_user("judy", "password123", "Judy").unwrap();

        let result = svc.change_password(&uid, "password123", "short").await;

        assert!(matches!(
            result,
            Err(AuthError::PasswordTooWeak { min_length: 8 })
        ));
    }
}
