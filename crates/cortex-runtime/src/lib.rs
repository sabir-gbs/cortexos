//! CortexOS app runtime and lifecycle management.
//!
//! Manages app loading, launching, sandboxing, and lifecycle events.
//! All app operations are mediated through this crate.
//!
//! # Core types
//!
//! - [`AppState`] - lifecycle state machine for app instances
//! - [`AppInstance`] - tracks a single invocation of an app
//! - [`AppLifecycleEvent`] - events broadcast on state transitions
//!
//! # Service trait
//!
//! - [`RuntimeService`] - async interface for all lifecycle operations
//!
//! # Errors
//!
//! - [`RuntimeError`] - typed errors for runtime operations

pub mod error;
pub mod service;
pub mod sqlite;
pub mod types;

pub use error::{Result, RuntimeError};
pub use service::RuntimeService;
pub use sqlite::SqliteRuntimeService;
pub use types::{AppInstance, AppLifecycleEvent, AppState};
