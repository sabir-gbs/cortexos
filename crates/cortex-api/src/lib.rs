//! CortexOS API server.
//!
//! The top-level composition layer that wires all services together,
//! exposes HTTP and WebSocket endpoints, and manages the server lifecycle.
//! This crate depends on all other crates and no crate depends on it.

pub mod app_state;
pub mod bus;
pub mod error;
pub mod middleware;
pub mod routes;
pub mod ws;

pub use app_state::AppState;
pub use error::{ApiError, Result};
