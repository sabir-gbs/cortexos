//! Default values and limits for admin operations (SPEC 22 §21.4).

/// Default metrics collection interval in seconds.
pub const DEFAULT_METRICS_INTERVAL_SECS: u64 = 5;

/// Ring buffer capacity: 1 hour of data at 5-second intervals.
pub const METRICS_RING_BUFFER_CAPACITY: usize = 720;

/// Hours of metric history persisted to SQLite.
pub const METRICS_DISK_RETENTION_HOURS: u32 = 24;

/// Aggregation interval for flushing metrics to disk (seconds).
pub const METRICS_AGGREGATION_INTERVAL_SECS: u64 = 60;

/// Session state save interval in seconds.
pub const SESSION_SAVE_INTERVAL_SECS: u64 = 60;

/// Crash log retention period in days.
pub const CRASH_LOG_RETENTION_DAYS: u32 = 30;

/// Maximum automatic restart attempts per app per session.
pub const MAX_AUTO_RESTART_ATTEMPTS: u32 = 3;

/// Maximum log entries displayed in dashboard.
pub const DASHBOARD_MAX_LOG_ENTRIES: u32 = 10_000;

/// Default log query limit.
pub const DASHBOARD_DEFAULT_LOG_LIMIT: u32 = 1000;

/// Default log hours included in diagnostic export.
pub const DEFAULT_EXPORT_LOG_HOURS: u32 = 24;

/// Maximum log hours allowed in export (7 days).
pub const MAX_EXPORT_LOG_HOURS: u32 = 168;

/// Text the user must type to confirm factory reset.
pub const FACTORY_RESET_CONFIRMATION_TEXT: &str = "RESET";

/// Safe mode flag file name.
pub const SAFE_MODE_FLAG_FILE: &str = "safe_mode";

/// Shift key detection window at boot in seconds.
pub const SAFE_MODE_SHIFT_WINDOW_SECS: u64 = 3;

/// Diagnostic export file name prefix.
pub const DIAGNOSTIC_FILE_PREFIX: &str = "cortexos-diagnostics-";

/// Diagnostic export file extension.
pub const DIAGNOSTIC_FILE_EXTENSION: &str = ".zip";

/// Redacted value placeholder for sensitive data.
pub const REDACTED_VALUE: &str = "[REDACTED]";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_buffer_capacity_matches_one_hour() {
        // 3600 seconds / 5 second interval = 720 entries
        assert_eq!(
            METRICS_RING_BUFFER_CAPACITY,
            (3600 / DEFAULT_METRICS_INTERVAL_SECS) as usize
        );
    }

    #[test]
    fn max_export_hours_is_seven_days() {
        assert_eq!(MAX_EXPORT_LOG_HOURS, 24 * 7);
    }

    #[test]
    fn reset_confirmation_text() {
        assert_eq!(FACTORY_RESET_CONFIRMATION_TEXT, "RESET");
    }
}
