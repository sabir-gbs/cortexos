//! CortexOS database access layer.
//!
//! Provides connection pooling, query helpers, and the migration runner.
//! SQLite is the default backend, bundled via rusqlite.

pub mod error;
pub mod migration;
pub mod types;

pub use error::{DbError, Result};
pub use migration::run_migrations;
pub use types::Pool;
