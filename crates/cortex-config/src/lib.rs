//! CortexOS configuration loading and validation.
//!
//! Loads application configuration from files, environment variables,
//! and CLI flags. All services receive configuration through [`AppConfig`]
//! via `Arc<AppConfig>` — no service reads config directly.
//!
//! Config precedence (highest wins):
//! 1. Environment variables (`CORTEX_*`)
//! 2. Config file (TOML)
//! 3. Built-in defaults

pub mod error;
pub mod types;

pub use error::{ConfigError, Result};
pub use types::AppConfig;
