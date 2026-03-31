//! Window Manager for CortexOS.
//!
//! Manages window lifecycle (open, close, minimize, resize, focus, move),
//! workspace switching, and z-ordering. All state is persisted to SQLite.

pub mod error;
pub mod service;
pub mod sqlite;
pub mod types;

pub use error::{Result, WmError};
pub use service::WindowManagerService;
pub use sqlite::SqliteWindowManager;
pub use types::*;
