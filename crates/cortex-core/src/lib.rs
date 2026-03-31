//! CortexOS foundation crate.
//!
//! Shared types, error taxonomy, ID newtypes, and foundational traits
//! used across all CortexOS crates. This crate has zero internal dependencies.
//!
//! Implements the error taxonomy from spec 02, section 8.2, and the
//! shared type contracts referenced by all downstream crates.

pub mod error;
pub mod ids;
pub mod types;

pub use error::{
    CoreError, CortexError, CortexErrorBody, ErrorCategory, ErrorCode, ResponseMeta, Result,
    SuccessResponse,
};
pub use ids::Timestamp;
pub use ids::{
    AppId, AppInstanceId, FileId, NotificationId, SessionId, SessionToken, SettingsNamespaceId,
    UserId,
};
pub use types::{AiProvider, ResolvedProvider};
