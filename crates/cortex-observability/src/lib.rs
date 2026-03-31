//! CortexOS observability, logging, and telemetry.
//!
//! All subsystems log through this crate. No subsystem implements its
//! own logging infrastructure. Structured JSON logs are emitted to stdout.

pub mod audit;
pub mod error;
pub mod logging;
pub mod types;

pub use audit::{AuditEvent, AuditLogger};
pub use error::{ObservabilityError, Result};
pub use logging::init_logging;
pub use types::{ComponentHealth, HealthReport, HealthStatus, LogEntry};
