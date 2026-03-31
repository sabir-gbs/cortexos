//! Observability types.

use serde::{Deserialize, Serialize};

/// Health check status for a subsystem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// The subsystem is healthy and operational.
    Healthy,
    /// The subsystem is degraded but functional.
    Degraded(String),
    /// The subsystem is unavailable.
    Unhealthy(String),
}

/// Structured log entry for audit and observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// ISO 8601 timestamp.
    pub timestamp: String,
    /// Log level (trace, debug, info, warn, error).
    pub level: String,
    /// The target module that produced this log.
    pub target: String,
    /// The log message.
    pub message: String,
    /// Additional structured fields.
    #[serde(flatten)]
    pub fields: serde_json::Value,
}

/// Component health report for the health endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall system health.
    pub status: HealthStatus,
    /// Per-component health status.
    pub components: Vec<ComponentHealth>,
    /// ISO 8601 timestamp of this report.
    pub checked_at: String,
}

/// Health of a single component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component name (e.g., "database", "filesystem").
    pub name: String,
    /// Component health status.
    pub status: HealthStatus,
    /// Optional response time in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency_ms: Option<u64>,
}
