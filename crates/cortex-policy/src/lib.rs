//! CortexOS permission policy engine.
//!
//! Manages permission grants, evaluates policy decisions, and enforces
//! access control for all system operations. No subsystem bypasses this
//! engine for authorization checks.

pub mod error;
pub mod service;
pub mod sqlite;
pub mod types;

pub use error::{PolicyError, Result};
pub use service::PolicyService;
pub use sqlite::SqlitePolicyService;
pub use types::{Permission, PermissionGrant, PermissionKind};
