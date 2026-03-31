//! CortexOS authentication, sessions, and user profiles.
//!
//! Handles user identity, login/logout, session management,
//! and password hashing. All security-sensitive logic is server-side only.

pub mod error;
pub mod service;
pub mod sqlite;
pub mod types;

pub use error::{AuthError, Result};
pub use service::AuthService;
pub use sqlite::SqliteAuthService;
pub use types::{LoginRequest, ProfileUpdate, Session, UserProfile};
