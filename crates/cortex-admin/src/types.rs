//! Admin domain types (SPEC 22 §8).

use cortex_core::Timestamp;
use cortex_observability::{ComponentHealth, HealthStatus};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Existing types
// ---------------------------------------------------------------------------

/// Overall system health summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Aggregated system health status.
    pub status: HealthStatus,
    /// Per-component health details.
    pub components: Vec<ComponentHealth>,
    /// System uptime in seconds.
    pub uptime_secs: u64,
}

/// A diagnostic report capturing the full system state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    /// ISO 8601 timestamp when the report was generated.
    pub generated_at: Timestamp,
    /// System health at the time of the report.
    pub health: SystemHealth,
    /// Number of active user sessions.
    pub active_sessions: u64,
    /// Number of running app instances.
    pub running_apps: u64,
    /// Database size in bytes.
    pub db_size_bytes: u64,
}

// ---------------------------------------------------------------------------
// Metrics (SPEC 22 §8.1)
// ---------------------------------------------------------------------------

/// A single timestamped metric sample.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSample {
    /// ISO 8601 timestamp.
    pub timestamp: Timestamp,
    /// CPU usage as a percentage (0.0 – 100.0).
    pub cpu_percent: f32,
    /// Bytes of RAM currently in use.
    pub memory_used_bytes: u64,
    /// Total physical RAM in bytes.
    pub memory_total_bytes: u64,
    /// Bytes of swap currently in use.
    pub swap_used_bytes: u64,
    /// Total swap space in bytes.
    pub swap_total_bytes: u64,
    /// Number of running app instances.
    pub running_app_count: u32,
    /// Errors observed since last sample.
    pub error_count: u32,
}

/// A disk-usage sample for a single mount point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskMetricSample {
    /// ISO 8601 timestamp.
    pub timestamp: Timestamp,
    /// Mount point path (e.g. `/`).
    pub mount_point: String,
    /// Bytes used on the filesystem.
    pub used_bytes: u64,
    /// Total bytes on the filesystem.
    pub total_bytes: u64,
}

// ---------------------------------------------------------------------------
// Running app info (SPEC 22 §8.2)
// ---------------------------------------------------------------------------

/// Runtime state of an app instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppRuntimeState {
    Running,
    Suspended,
}

/// Information about a currently running app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppRunningInfo {
    /// Application identifier (e.g. `com.cortexos.settings`).
    pub app_id: String,
    /// Human-readable application name.
    pub app_name: String,
    /// Process identifier.
    pub pid: u32,
    /// CPU usage percentage.
    pub cpu_percent: f32,
    /// Memory used by the process in bytes.
    pub memory_used_bytes: u64,
    /// Runtime state.
    pub state: AppRuntimeState,
    /// ISO 8601 timestamp when the app started.
    pub started_at: Timestamp,
}

// ---------------------------------------------------------------------------
// Crash records (SPEC 22 §8.3)
// ---------------------------------------------------------------------------

/// A record of an app crash event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashRecord {
    /// Unique identifier for this crash record.
    pub id: Uuid,
    /// Application identifier.
    pub app_id: String,
    /// Human-readable application name.
    pub app_name: String,
    /// Exit code returned by the process.
    pub exit_code: i32,
    /// Signal that terminated the process, if any.
    pub signal: Option<i32>,
    /// ISO 8601 timestamp of the crash.
    pub crash_time: Timestamp,
    /// Path to the crash log file.
    pub crash_log_path: PathBuf,
    /// Number of automatic restart attempts so far this session.
    pub auto_restart_attempts: u32,
}

// ---------------------------------------------------------------------------
// Session state (SPEC 22 §8.4)
// ---------------------------------------------------------------------------

/// Information about a single app within a persisted session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionApp {
    /// Application identifier.
    pub app_id: String,
    /// Window X position.
    pub window_x: i32,
    /// Window Y position.
    pub window_y: i32,
    /// Window width in pixels.
    pub window_width: u32,
    /// Window height in pixels.
    pub window_height: u32,
    /// Whether this app's window was focused.
    pub was_focused: bool,
}

/// Snapshot of session state for recovery after an unclean shutdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Schema version for future compatibility.
    pub version: u32,
    /// Whether the previous shutdown was clean.
    pub clean_shutdown: bool,
    /// ISO 8601 timestamp when the state was saved.
    pub saved_at: Timestamp,
    /// List of running apps at save time.
    pub apps: Vec<SessionApp>,
}

// ---------------------------------------------------------------------------
// Filesystem check (SPEC 22 §8.5)
// ---------------------------------------------------------------------------

/// Status of a single filesystem check item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsCheckStatus {
    Pass,
    Warning,
    Error,
}

/// A single filesystem check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsCheckItem {
    /// Name of the check performed.
    pub check_name: String,
    /// Path that was checked.
    pub path: String,
    /// Check result status.
    pub status: FsCheckStatus,
    /// Human-readable message about the result.
    pub message: String,
    /// Whether this issue can be automatically repaired.
    pub repairable: bool,
}

/// Aggregate result of a filesystem verification pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsCheckResult {
    /// Individual check results.
    pub items: Vec<FsCheckItem>,
    /// Total number of checks performed.
    pub total_checked: u32,
    /// Number of checks that passed.
    pub passed: u32,
    /// Number of warnings.
    pub warnings: u32,
    /// Number of errors.
    pub errors: u32,
}

impl FsCheckResult {
    /// Create a result from a list of check items, computing the aggregates.
    pub fn from_items(items: Vec<FsCheckItem>) -> Self {
        let total_checked = items.len() as u32;
        let passed = items
            .iter()
            .filter(|i| i.status == FsCheckStatus::Pass)
            .count() as u32;
        let warnings = items
            .iter()
            .filter(|i| i.status == FsCheckStatus::Warning)
            .count() as u32;
        let errors = items
            .iter()
            .filter(|i| i.status == FsCheckStatus::Error)
            .count() as u32;
        Self {
            items,
            total_checked,
            passed,
            warnings,
            errors,
        }
    }
}

// ---------------------------------------------------------------------------
// Diagnostic bundle (SPEC 22 §8.6)
// ---------------------------------------------------------------------------

/// Component included in a diagnostic export.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DiagnosticComponent {
    SystemLogs { hours: u32 },
    RunningApps,
    PermissionGrants,
    AiUsageSummary,
    CrashLogs,
    SystemSettings,
}

/// System information included in a diagnostic bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    /// CortexOS version string.
    pub os_version: String,
    /// Kernel version string.
    pub kernel_version: String,
    /// Machine hostname.
    pub hostname: String,
    /// CPU architecture (e.g. `x86_64`).
    pub architecture: String,
    /// Number of logical CPU cores.
    pub cpu_count: u32,
    /// Total physical memory in bytes.
    pub total_memory_bytes: u64,
    /// System uptime in seconds.
    pub uptime_seconds: u64,
}

/// A diagnostic export bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticBundle {
    /// ISO 8601 timestamp when the bundle was created.
    pub exported_at: Timestamp,
    /// System information at export time.
    pub system_info: SystemInfo,
    /// Components included in the bundle.
    pub included_components: Vec<DiagnosticComponent>,
    /// Total size of the bundle in bytes.
    pub total_size_bytes: u64,
}

// ---------------------------------------------------------------------------
// Export options
// ---------------------------------------------------------------------------

/// Options controlling what to include in a diagnostic export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Include system logs.
    pub include_logs: bool,
    /// Hours of log history to include (default 24).
    pub log_hours: u32,
    /// Include system info.
    pub include_system_info: bool,
    /// Include running app list.
    pub include_running_apps: bool,
    /// Include permission grants.
    pub include_permissions: bool,
    /// Include AI usage summary.
    pub include_ai_usage: bool,
    /// Include crash logs.
    pub include_crash_logs: bool,
    /// Include system settings (redacted).
    pub include_settings: bool,
    /// Output path for the ZIP file.
    pub output_path: PathBuf,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_logs: true,
            log_hours: 24,
            include_system_info: true,
            include_running_apps: true,
            include_permissions: true,
            include_ai_usage: true,
            include_crash_logs: true,
            include_settings: true,
            output_path: PathBuf::from("cortexos-diagnostics.zip"),
        }
    }
}

// ---------------------------------------------------------------------------
// Error counts
// ---------------------------------------------------------------------------

/// Error counts broken down by severity for a time window.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorCountBySeverity {
    /// Number of errors.
    pub errors: u32,
    /// Number of warnings.
    pub warnings: u32,
}

/// Error counts for multiple time windows.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ErrorCounts {
    /// Errors in the last hour.
    pub last_hour: ErrorCountBySeverity,
    /// Errors in the last 6 hours.
    pub last_6_hours: ErrorCountBySeverity,
    /// Errors in the last 24 hours.
    pub last_24_hours: ErrorCountBySeverity,
    /// Errors in the last 7 days.
    pub last_7_days: ErrorCountBySeverity,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_components() -> Vec<ComponentHealth> {
        vec![
            ComponentHealth {
                name: "database".to_string(),
                status: HealthStatus::Healthy,
                latency_ms: Some(5),
            },
            ComponentHealth {
                name: "filesystem".to_string(),
                status: HealthStatus::Degraded("slow disk".to_string()),
                latency_ms: Some(120),
            },
        ]
    }

    #[test]
    fn system_health_construction() {
        let health = SystemHealth {
            status: HealthStatus::Healthy,
            components: sample_components(),
            uptime_secs: 86400,
        };
        assert_eq!(health.uptime_secs, 86400);
        assert_eq!(health.components.len(), 2);
    }

    #[test]
    fn system_health_serde_roundtrip() {
        let health = SystemHealth {
            status: HealthStatus::Healthy,
            components: sample_components(),
            uptime_secs: 3600,
        };
        let json = serde_json::to_string(&health).unwrap();
        let parsed: SystemHealth = serde_json::from_str(&json).unwrap();
        assert_eq!(health.uptime_secs, parsed.uptime_secs);
        assert_eq!(health.components.len(), parsed.components.len());
        assert_eq!(health.status, parsed.status);
    }

    #[test]
    fn diagnostic_report_construction() {
        let report = DiagnosticReport {
            generated_at: "2026-03-30T12:00:00Z".to_string(),
            health: SystemHealth {
                status: HealthStatus::Healthy,
                components: sample_components(),
                uptime_secs: 86400,
            },
            active_sessions: 5,
            running_apps: 3,
            db_size_bytes: 1024 * 1024 * 512,
        };
        assert_eq!(report.active_sessions, 5);
        assert_eq!(report.running_apps, 3);
        assert_eq!(report.db_size_bytes, 536870912);
    }

    #[test]
    fn diagnostic_report_serde_roundtrip() {
        let report = DiagnosticReport {
            generated_at: "2026-03-30T12:00:00Z".to_string(),
            health: SystemHealth {
                status: HealthStatus::Unhealthy("db down".to_string()),
                components: vec![],
                uptime_secs: 100,
            },
            active_sessions: 0,
            running_apps: 0,
            db_size_bytes: 4096,
        };
        let json = serde_json::to_string(&report).unwrap();
        let parsed: DiagnosticReport = serde_json::from_str(&json).unwrap();
        assert_eq!(report.generated_at, parsed.generated_at);
        assert_eq!(report.active_sessions, parsed.active_sessions);
        assert_eq!(report.running_apps, parsed.running_apps);
        assert_eq!(report.db_size_bytes, parsed.db_size_bytes);
    }

    // -- MetricSample tests --

    #[test]
    fn metric_sample_construction() {
        let sample = MetricSample {
            timestamp: "2026-03-30T12:00:00Z".to_string(),
            cpu_percent: 45.2,
            memory_used_bytes: 4_000_000_000,
            memory_total_bytes: 16_000_000_000,
            swap_used_bytes: 0,
            swap_total_bytes: 8_000_000_000,
            running_app_count: 7,
            error_count: 0,
        };
        assert_eq!(sample.cpu_percent, 45.2);
        assert_eq!(sample.running_app_count, 7);
    }

    #[test]
    fn metric_sample_serde_roundtrip() {
        let sample = MetricSample {
            timestamp: "2026-03-30T12:00:00Z".to_string(),
            cpu_percent: 75.0,
            memory_used_bytes: 8_000_000_000,
            memory_total_bytes: 16_000_000_000,
            swap_used_bytes: 1_000_000_000,
            swap_total_bytes: 8_000_000_000,
            running_app_count: 3,
            error_count: 5,
        };
        let json = serde_json::to_string(&sample).unwrap();
        let parsed: MetricSample = serde_json::from_str(&json).unwrap();
        assert_eq!(sample.cpu_percent, parsed.cpu_percent);
        assert_eq!(sample.memory_used_bytes, parsed.memory_used_bytes);
        assert_eq!(sample.error_count, parsed.error_count);
    }

    // -- CrashRecord tests --

    #[test]
    fn crash_record_construction() {
        let record = CrashRecord {
            id: Uuid::now_v7(),
            app_id: "com.cortexos.settings".to_string(),
            app_name: "Settings".to_string(),
            exit_code: 139,
            signal: Some(11),
            crash_time: "2026-03-30T12:00:00Z".to_string(),
            crash_log_path: PathBuf::from("/var/log/cortexos/crash_settings.log"),
            auto_restart_attempts: 2,
        };
        assert_eq!(record.exit_code, 139);
        assert_eq!(record.signal, Some(11));
        assert_eq!(record.auto_restart_attempts, 2);
    }

    #[test]
    fn crash_record_serde_roundtrip() {
        let record = CrashRecord {
            id: Uuid::now_v7(),
            app_id: "com.cortexos.terminal-lite".to_string(),
            app_name: "Terminal".to_string(),
            exit_code: 1,
            signal: None,
            crash_time: "2026-03-30T13:00:00Z".to_string(),
            crash_log_path: PathBuf::from("/tmp/crash.log"),
            auto_restart_attempts: 0,
        };
        let json = serde_json::to_string(&record).unwrap();
        let parsed: CrashRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(record.id, parsed.id);
        assert_eq!(record.app_id, parsed.app_id);
        assert_eq!(record.signal, parsed.signal);
    }

    // -- SessionState tests --

    #[test]
    fn session_state_construction() {
        let state = SessionState {
            version: 1,
            clean_shutdown: false,
            saved_at: "2026-03-30T12:00:00Z".to_string(),
            apps: vec![
                SessionApp {
                    app_id: "com.cortexos.settings".to_string(),
                    window_x: 100,
                    window_y: 100,
                    window_width: 800,
                    window_height: 600,
                    was_focused: true,
                },
                SessionApp {
                    app_id: "com.cortexos.file-manager".to_string(),
                    window_x: 200,
                    window_y: 200,
                    window_width: 900,
                    window_height: 600,
                    was_focused: false,
                },
            ],
        };
        assert!(!state.clean_shutdown);
        assert_eq!(state.apps.len(), 2);
        assert!(state.apps[0].was_focused);
        assert_eq!(state.apps[1].window_width, 900);
    }

    #[test]
    fn session_state_serde_roundtrip() {
        let state = SessionState {
            version: 1,
            clean_shutdown: true,
            saved_at: "2026-03-30T14:00:00Z".to_string(),
            apps: vec![SessionApp {
                app_id: "com.cortexos.notes".to_string(),
                window_x: 50,
                window_y: 50,
                window_width: 900,
                window_height: 600,
                was_focused: true,
            }],
        };
        let json = serde_json::to_string(&state).unwrap();
        let parsed: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.version, parsed.version);
        assert_eq!(state.clean_shutdown, parsed.clean_shutdown);
        assert_eq!(state.apps.len(), parsed.apps.len());
        assert_eq!(state.apps[0].app_id, parsed.apps[0].app_id);
    }

    // -- FsCheckResult tests --

    #[test]
    fn fs_check_result_from_items() {
        let items = vec![
            FsCheckItem {
                check_name: "config_exists".to_string(),
                path: "/etc/cortexos/config.json".to_string(),
                status: FsCheckStatus::Pass,
                message: "OK".to_string(),
                repairable: false,
            },
            FsCheckItem {
                check_name: "app_dir".to_string(),
                path: "/data/cortexos/apps/old-app".to_string(),
                status: FsCheckStatus::Warning,
                message: "Orphaned directory".to_string(),
                repairable: true,
            },
            FsCheckItem {
                check_name: "db_integrity".to_string(),
                path: "/data/cortexos/cortex.db".to_string(),
                status: FsCheckStatus::Error,
                message: "Corrupt table".to_string(),
                repairable: false,
            },
        ];
        let result = FsCheckResult::from_items(items);
        assert_eq!(result.total_checked, 3);
        assert_eq!(result.passed, 1);
        assert_eq!(result.warnings, 1);
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn fs_check_result_empty() {
        let result = FsCheckResult::from_items(vec![]);
        assert_eq!(result.total_checked, 0);
        assert_eq!(result.passed, 0);
        assert_eq!(result.warnings, 0);
        assert_eq!(result.errors, 0);
    }

    // -- DiagnosticBundle / SystemInfo tests --

    #[test]
    fn system_info_construction() {
        let info = SystemInfo {
            os_version: "0.1.0".to_string(),
            kernel_version: "6.19.9".to_string(),
            hostname: "cortexos-dev".to_string(),
            architecture: "x86_64".to_string(),
            cpu_count: 8,
            total_memory_bytes: 16_000_000_000,
            uptime_seconds: 86400,
        };
        assert_eq!(info.cpu_count, 8);
        assert_eq!(info.architecture, "x86_64");
    }

    #[test]
    fn diagnostic_bundle_construction() {
        let bundle = DiagnosticBundle {
            exported_at: "2026-03-30T12:00:00Z".to_string(),
            system_info: SystemInfo {
                os_version: "0.1.0".to_string(),
                kernel_version: "6.19.9".to_string(),
                hostname: "cortexos-dev".to_string(),
                architecture: "x86_64".to_string(),
                cpu_count: 8,
                total_memory_bytes: 16_000_000_000,
                uptime_seconds: 3600,
            },
            included_components: vec![
                DiagnosticComponent::SystemLogs { hours: 24 },
                DiagnosticComponent::RunningApps,
                DiagnosticComponent::SystemSettings,
            ],
            total_size_bytes: 1024 * 1024,
        };
        assert_eq!(bundle.included_components.len(), 3);
        assert_eq!(bundle.total_size_bytes, 1048576);
    }

    // -- ExportOptions tests --

    #[test]
    fn export_options_default() {
        let opts = ExportOptions::default();
        assert!(opts.include_logs);
        assert_eq!(opts.log_hours, 24);
        assert!(opts.include_system_info);
        assert!(opts.include_settings);
    }

    // -- AppRuntimeState tests --

    #[test]
    fn app_runtime_state_serde() {
        let running = serde_json::to_string(&AppRuntimeState::Running).unwrap();
        assert_eq!(running, "\"running\"");
        let suspended = serde_json::to_string(&AppRuntimeState::Suspended).unwrap();
        assert_eq!(suspended, "\"suspended\"");
    }

    // -- ErrorCounts tests --

    #[test]
    fn error_counts_default() {
        let counts = ErrorCounts::default();
        assert_eq!(counts.last_hour.errors, 0);
        assert_eq!(counts.last_6_hours.warnings, 0);
    }
}
