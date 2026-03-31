//! CortexOS admin diagnostics and recovery (SPEC 22).
//!
//! Provides system health checks, database integrity verification,
//! backup/restore operations, metrics collection, session recovery,
//! crash handling, diagnostic export, safe mode, and factory reset.

pub mod constants;
pub mod error;
pub mod export;
pub mod metrics;
pub mod recovery;
pub mod safe_mode;
pub mod service;
pub mod sqlite;
pub mod types;

pub use error::{AdminError, Result};
pub use service::{AdminService, SessionInfo};
pub use sqlite::SqliteAdminService;
pub use types::{
    AppRunningInfo, AppRuntimeState, CrashRecord, DiagnosticBundle, DiagnosticComponent,
    DiagnosticReport, DiskMetricSample, ErrorCountBySeverity, ErrorCounts, ExportOptions,
    FsCheckItem, FsCheckResult, FsCheckStatus, MetricSample, SessionApp, SessionState,
    SystemHealth, SystemInfo,
};
