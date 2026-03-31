# SPEC 22: Admin, Diagnostics, and Recovery

**Spec ID:** 22
**Status:** Implementation-grade
**Risk Level:** Standard
**Crate:** cortex-admin
**Last Updated:** 2026-03-30

---

## 1. Purpose

This specification defines the administrative, diagnostic, and recovery tools in CortexOS. It establishes a system health dashboard, diagnostic utilities, crash and session recovery mechanisms, diagnostic export capabilities, safe mode operation, and factory reset functionality. These tools enable administrators and advanced users to monitor system health, diagnose issues, and recover from failures.

---

## 2. Scope

- System health dashboard: CPU/memory usage, active sessions, running apps, error counts
- Diagnostic tools: log viewer, event inspector, permission audit, AI usage statistics
- Recovery tools: app crash recovery, session recovery, filesystem repair, settings reset
- Diagnostic export: bundling logs and system state for support
- Safe mode: boot with minimal services for troubleshooting
- Factory reset: restore system to initial state with confirmation safeguards

---

## 3. Out of Scope

- Remote administration or remote access tools
- Automated remediation or self-healing systems
- Telemetry or data collection sent to external servers
- Multi-user administration or role-based admin access
- Hardware diagnostics (POST, memory testing, disk health SMART data)
- Network diagnostics (ping, traceroute, DNS resolution)
- Performance profiling tools (CPU profiler, memory profiler)

---

## 4. Objectives

1. Provide real-time visibility into system resource usage and application health.
2. Offer structured diagnostic tools that allow efficient root cause analysis.
3. Implement reliable recovery mechanisms for crashed apps and corrupted sessions.
4. Enable users to export a complete diagnostic bundle for support requests.
5. Provide safe mode to isolate and resolve system-level issues.
6. Offer factory reset with multiple confirmation safeguards to prevent accidental data loss.

---

## 5. User-Visible Behavior

### 5.1 System Health Dashboard

- Accessible from: System Menu > Administration > System Health, or by running `cortex-admin dashboard`.
- The dashboard is a full-screen application divided into panels:
  - **CPU Panel**: Shows current CPU usage as a percentage, with a 60-second sparkline history. Per-core breakdown available on expand.
  - **Memory Panel**: Shows total, used, and available memory in GB, with a 60-second sparkline. Swap usage shown if swap is configured.
  - **Sessions Panel**: Lists active user sessions with session ID, start time, and duration.
  - **Running Apps Panel**: Lists all running apps with app name, PID, CPU usage, memory usage, and status (running/suspended). Sortable by any column.
  - **Error Counts Panel**: Shows error counts in the last 1 hour, 6 hours, 24 hours, and 7 days. Grouped by severity (error, warning).
  - **Disk Usage Panel**: Shows total, used, and available disk space for each mounted filesystem.
  - **Uptime Panel**: Shows system uptime in days, hours, minutes.
- All panels refresh every 5 seconds by default (configurable: 1s, 5s, 10s, 30s).
- Panels are rearrangeable by drag-and-drop.
- The dashboard can be kept open as a background window.

### 5.2 Log Viewer

- Accessible from: System Menu > Administration > Logs, or from the dashboard's "View Logs" button.
- Displays log entries from all system components in reverse chronological order.
- Filter options:
  - By severity: Debug, Info, Warn, Error (multi-select)
  - By component: dropdown list of all registered components
  - By time range: last 15 minutes, 1 hour, 6 hours, 24 hours, custom range
  - By text search: full-text search across log messages
- Each log entry displays: timestamp, severity (color-coded), component name, message.
- Log entries are streamed in real-time when "Live" mode is toggled on.
- Maximum display: 10,000 entries. Older entries are discarded from the UI view but remain in the observability store until retention expires.
- "Copy" button copies selected entries to clipboard.
- "Export" button exports filtered entries to a file.

### 5.3 Event Inspector

- Accessible from: System Menu > Administration > Events.
- Shows system events in a structured table: event name, source, timestamp, payload (expandable JSON).
- Events include: app lifecycle (launch, stop, crash), permission changes, settings changes, AI actions, system warnings.
- Filter by: event type, source component, time range, text search in payload.
- Click an event to see its full payload in a detail panel.

### 5.4 Permission Audit

- Accessible from: System Menu > Administration > Permissions, or Settings > Apps > Permission Audit.
- Shows all granted permissions across all apps in a table: app name, permission type, resource type, granted date, risk level.
- Filter by: app, permission type, risk level.
- "Revoke" button next to each entry (with confirmation).
- "Export" button generates a JSON report of all permissions.

### 5.5 AI Usage Statistics

- Accessible from: System Menu > Administration > AI Usage, or from the dashboard's "AI Stats" button.
- Shows:
  - Total AI requests in the last 24 hours / 7 days / 30 days
  - Requests by provider and model (bar chart)
  - Total tokens consumed by provider
  - Average response time by provider
  - Error rate by provider (percentage)
  - Cost estimate (if user has configured pricing in settings)
  - Top apps by AI usage (table: app name, request count, token count)
- Data is sourced from the audit log (SPEC 20).

### 5.6 Recovery Tools

- Accessible from: System Menu > Administration > Recovery.

#### 5.6.1 App Crash Recovery

- Lists all apps that have crashed in the current session with: app name, crash time, exit code, crash log path.
- "Relaunch" button attempts to restart the app.
- "View Crash Log" opens the crash log in the log viewer.
- "Clear Crash History" removes crash records (with confirmation).
- Automatic crash recovery: if an app declares `crash_recovery: true` in its manifest, the system offers to restart it automatically after a crash (with a notification, not silently). Max 3 auto-restart attempts per app per session.

#### 5.6.2 Session Recovery

- On system startup after an unclean shutdown, a notification appears: "CortexOS did not shut down cleanly. Would you like to restore your previous session?"
- If the user chooses "Restore", previously running apps are relaunched and their windows positioned as they were.
- Session state is saved every 60 seconds into the admin/session_state table in SQLite.
- Session state includes: list of running apps with their window positions and sizes.
- "Clear Saved Session" button in Recovery Tools discards the saved session.

#### 5.6.3 Filesystem Repair

- "Verify Filesystem" button checks for:
  - Missing or corrupted system configuration files
  - Orphaned app data directories (app uninstalled but data remains)
  - AI-action permission grant integrity
  - App registry consistency (all registered apps have intact install directories and package records)
- Results displayed in a list with severity (info, warning, error).
- "Repair All" button attempts to fix all detected issues (with confirmation).
- Individual items can be repaired or dismissed.

#### 5.6.4 Settings Reset

- "Reset App Settings" allows resetting a specific app's settings to defaults.
- Dropdown to select the app. "Reset" button with confirmation.
- "Reset System Settings" resets all CortexOS system settings to factory defaults (distinct from full factory reset). Confirmation required: "This will reset all system settings to their default values. App data will not be affected."

### 5.7 Diagnostic Export

- Accessible from: System Menu > Administration > Export Diagnostics.
- The export dialog shows what will be included:
  - System logs (last 24 hours by default, configurable up to 7 days)
  - System information (OS version, hardware summary, uptime)
  - Running app list
  - Permission grants
  - AI usage summary (no conversation content, only aggregate stats)
  - Crash logs
  - System settings (with sensitive values redacted: API keys, passwords, tokens replaced with "[REDACTED]")
- User selects what to include via checkboxes (all checked by default).
- "Export" button creates a ZIP file at a user-chosen location.
- Export progress shown with a progress bar.
- The ZIP file is named: `cortexos-diagnostics-{YYYY-MM-DD}-{HHmmss}.zip`.
- A preview of the file size is shown before export begins.
- Warning: "This export may contain information about your installed apps and usage patterns. Share it only with trusted support personnel."

### 5.8 Safe Mode

- Accessible by: holding `Shift` during boot, or from System Menu > Administration > Restart in Safe Mode.
- A confirmation dialog appears: "Restart in Safe Mode? Only essential system services will run. Most apps will not be available."
- In safe mode:
  - Only first-party system apps load (Settings, Terminal, File Manager, Administration).
  - No third-party apps are loaded.
  - No background services (no AI providers, no notification daemons).
  - No autostart apps.
  - The desktop background is replaced with a "Safe Mode" indicator (distinct color band at the top of the screen).
  - A persistent notification reads: "CortexOS is running in Safe Mode. Restart to return to normal mode."
- Safe mode is indicated in the system tray and Settings > About.
- To exit safe mode, the user restarts the system normally (no special key required).

### 5.9 Factory Reset

- Accessible from: System Menu > Administration > Factory Reset.
- The factory reset flow has three confirmation stages:
  1. **Information screen**: Explains what will be deleted (all apps, all app data, all settings, all AI conversation history, all user files in CortexOS-managed directories). Lists what will be preserved (nothing - factory reset is total).
  2. **First confirmation**: "Are you sure you want to reset CortexOS to factory defaults? This action cannot be undone. All data will be permanently deleted." Buttons: "Cancel", "Continue" (secondary style).
  3. **Second confirmation**: User must type "RESET" in a text field. The "Reset" button is disabled until the exact text is entered. Buttons: "Cancel", "Reset" (destructive red style).
- After confirmation, a progress screen shows:
  - "Erasing app data..."
  - "Removing apps..."
  - "Resetting settings..."
  - "Clearing AI history..."
  - "Rebuilding default configuration..."
  - "Restarting..."
- The system restarts into a clean state with default settings and no installed third-party apps.
- Factory reset does not affect the operating system kernel or boot loader. It only resets CortexOS user space data.

---

## 6. System Behavior

### 6.1 Health Data Collection

- Host/system metrics are collected by a background collector owned by `cortex-admin`; `cortex-observability` remains the owner of logs and AI audit/usage query surfaces, but not raw host metric sampling.
- The dashboard and admin APIs read host metrics only through this collector/service boundary. No browser client or unrelated app reads `sysinfo` or host process data directly.
- Collection interval: every 5 seconds (configurable: 1s, 5s, 10s, 30s).
- Metrics are stored in a ring buffer in memory (last 1 hour of data) and in SQLite (last 24 hours, aggregated to 1-minute intervals).
- Each metric sample is a timestamped record of: cpu_percent, memory_used_bytes, memory_total_bytes, swap_used_bytes, swap_total_bytes, disk_used_bytes, disk_total_bytes, running_app_count, error_count.

### 6.2 Session State Persistence

- Session state is persisted every 60 seconds.
- Session state writes are transactional.
- On a clean shutdown, the stored session state is explicitly marked as `clean_shutdown: true`.
- On startup, if the stored session state has `clean_shutdown: false`, the recovery prompt appears.

### 6.3 Safe Mode Detection

- Safe mode is determined by a kernel command-line parameter or a flag file.
- Flag file: `$XDG_CONFIG_HOME/cortexos/safe_mode` (presence triggers safe mode).
- The flag file is created by the "Restart in Safe Mode" action and removed on normal boot.
- The boot loader checks for the `Shift` key hold during a 3-second window.

### 6.4 Factory Reset Execution

- Factory reset runs in a dedicated process that is not dependent on the main UI shell.
- Steps (executed in order):
  1. Stop all running apps.
  2. Delete all app data directories: `$XDG_DATA_HOME/cortexos/apps/*/`.
  3. Delete installed-app registry rows and package metadata from SQLite.
  4. Delete AI conversation history rows.
  5. Delete AI-action permission grant rows.
  6. Delete AI audit rows.
  7. Reset all settings rows to defaults.
  8. Delete session state rows.
  9. Delete metrics history rows.
  10. Delete crash logs and crash dump artifacts from service-owned storage.
  11. Rebuild default directory structure.
  12. Restart the system.
- Each step is logged at `info` level.

---

## 7. Architecture

```
+-----------------------------------------------------------+
|                     cortex-admin                           |
|                                                            |
|  +------------------+  +------------------+               |
|  | Health Dashboard |  | Log Viewer       |               |
|  +--------+---------+  +--------+---------+               |
|           |                      |                         |
|  +--------v----------------------v---------+               |
|  |           Admin Controller              |               |
|  +--------+---------+-----------+---------+               |
|           |         |           |                          |
|  +--------v--+  +---v------+  +--v--------+               |
|  | Metrics   |  | Diag     |  | Recovery  |               |
|  | Collector |  | Exporter |  | Manager   |               |
|  +--------+--+  +----+-----+  +--+--------+               |
|           |           |           |                        |
|  +--------v-----------v-----------v--------+               |
|  |           Admin Data Store              |               |
|  +-----------------------------------------+               |
|                                                            |
+-----------------------------------------------------------+
```

### Component Responsibilities

- **Health Dashboard**: UI component rendering metric panels, sparklines, and app lists.
- **Log Viewer**: UI component for streaming, filtering, and searching log entries.
- **Admin Controller**: Orchestrates all admin operations. Routes UI actions to the appropriate service.
- **Metrics Collector**: Background service that samples system metrics at the configured interval.
- **Diagnostic Exporter**: Bundles selected diagnostic data into a ZIP file.
- **Recovery Manager**: Handles crash recovery, session recovery, filesystem repair, and factory reset.
- **Admin Data Store**: Persistence layer for metrics, session state, and crash logs.

---

## 8. Data Model

### 8.1 Metric Sample

```rust
struct MetricSample {
    timestamp: DateTime<Utc>,
    cpu_percent: f32,
    memory_used_bytes: u64,
    memory_total_bytes: u64,
    swap_used_bytes: u64,
    swap_total_bytes: u64,
    running_app_count: u32,
    error_count: u32,
}

struct DiskMetricSample {
    timestamp: DateTime<Utc>,
    mount_point: String,
    used_bytes: u64,
    total_bytes: u64,
}
```

### 8.2 App Running Info

```rust
struct AppRunningInfo {
    app_id: String,
    app_name: String,
    pid: u32,
    cpu_percent: f32,
    memory_used_bytes: u64,
    state: AppRuntimeState,
    started_at: DateTime<Utc>,
}

enum AppRuntimeState {
    Running,
    Suspended,
}
```

### 8.3 Crash Record

```rust
struct CrashRecord {
    id: Uuid,
    app_id: String,
    app_name: String,
    exit_code: i32,
    signal: Option<i32>,
    crash_time: DateTime<Utc>,
    crash_log_path: PathBuf,
    auto_restart_attempts: u32,
}
```

### 8.4 Session State

```rust
struct SessionState {
    version: u32,
    clean_shutdown: bool,
    saved_at: DateTime<Utc>,
    apps: Vec<SessionApp>,
}

struct SessionApp {
    app_id: String,
    window_x: i32,
    window_y: i32,
    window_width: u32,
    window_height: u32,
    was_focused: bool,
}
```

### 8.5 Filesystem Check Result

```rust
struct FsCheckResult {
    items: Vec<FsCheckItem>,
    total_checked: u32,
    passed: u32,
    warnings: u32,
    errors: u32,
}

struct FsCheckItem {
    check_name: String,
    path: String,
    status: FsCheckStatus,
    message: String,
    repairable: bool,
}

enum FsCheckStatus {
    Pass,
    Warning,
    Error,
}
```

### 8.6 Diagnostic Bundle

```rust
struct DiagnosticBundle {
    exported_at: DateTime<Utc>,
    system_info: SystemInfo,
    included_components: Vec<DiagnosticComponent>,
    total_size_bytes: u64,
}

struct SystemInfo {
    os_version: String,
    kernel_version: String,
    hostname: String,
    architecture: String,
    cpu_count: u32,
    total_memory_bytes: u64,
    uptime_seconds: u64,
}

enum DiagnosticComponent {
    SystemLogs { hours: u32 },
    RunningApps,
    PermissionGrants,
    AiUsageSummary,
    CrashLogs,
    SystemSettings,
}
```

### 8.7 Invariants

1. Metric samples are never backdated; timestamps are always monotonically increasing within a collection session.
2. Session state is never saved in an inconsistent state (atomic write).
3. Crash records are appended, never modified or deleted (except by explicit user action).
4. Factory reset must complete all deletion steps or none (if it fails midway, the system enters a "partial reset" state and the user is advised to retry).
5. Safe mode flag file is always removed on normal boot, even if the previous boot was normal.
6. The metrics ring buffer never exceeds 1 hour of data in memory (older samples are discarded from the buffer).

---

## 9. Public Interfaces

### 9.1 Admin Dashboard API

```rust
trait AdminDashboard {
    /// Get current system metrics (latest sample).
    fn get_current_metrics(&self) -> Result<MetricSample>;

    /// Get metric history for a time range.
    fn get_metric_history(
        &self,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<MetricSample>>;

    /// Get current disk metrics for all mounted filesystems.
    fn get_disk_metrics(&self) -> Result<Vec<DiskMetricSample>>;

    /// Get list of currently running apps with resource usage.
    fn get_running_apps(&self) -> Result<Vec<AppRunningInfo>>;

    /// Get error counts for specified time windows.
    fn get_error_counts(&self) -> Result<ErrorCounts>;

    /// Get active user sessions.
    fn get_sessions(&self) -> Result<Vec<SessionInfo>>;

    /// Get system uptime.
    fn get_uptime(&self) -> Result<Duration>;
}

struct ErrorCounts {
    last_hour: ErrorCountBySeverity,
    last_6_hours: ErrorCountBySeverity,
    last_24_hours: ErrorCountBySeverity,
    last_7_days: ErrorCountBySeverity,
}

struct ErrorCountBySeverity {
    errors: u32,
    warnings: u32,
}

struct SessionInfo {
    session_id: String,
    username: String,
    started_at: DateTime<Utc>,
    is_current: bool,
}
```

### 9.2 Diagnostics API

```rust
trait Diagnostics {
    /// Get log entries matching the filter.
    fn query_logs(&self, filter: LogFilter) -> Result<Vec<LogEntry>>;

    /// Stream log entries in real-time.
    fn stream_logs(&self, filter: LogFilter) -> Result<mpsc::Receiver<LogEntry>>;

    /// Get system events matching the filter.
    fn query_events(&self, filter: EventFilter) -> Result<Vec<SystemEvent>>;

    /// Get all permission grants across all apps.
    fn audit_permissions(&self) -> Result<Vec<PermissionAuditEntry>>;

    /// Get AI usage statistics.
    fn get_ai_usage_stats(&self, period: AiUsagePeriod) -> Result<AiUsageStats>;

    /// Export diagnostic bundle to a ZIP file.
    fn export_diagnostics(&self, options: ExportOptions) -> Result<PathBuf>;
}

struct LogFilter {
    severities: Vec<LogLevel>,
    components: Vec<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    text_search: Option<String>,
    limit: u32,  // default: 1000
}

struct LogEntry {
    timestamp: DateTime<Utc>,
    level: LogLevel,
    component: String,
    message: String,
    fields: HashMap<String, String>,
}

#[derive(Clone, Copy)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

struct EventFilter {
    event_types: Vec<String>,
    sources: Vec<String>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    payload_search: Option<String>,
    limit: u32,  // default: 500
}

struct SystemEvent {
    event_id: Uuid,
    event_type: String,
    source: String,
    timestamp: DateTime<Utc>,
    payload: serde_json::Value,
}

struct PermissionAuditEntry {
    app_id: String,
    app_name: String,
    permission: String,
    resource_type: String,
    risk_level: String,
    granted_at: DateTime<Utc>,
}

enum AiUsagePeriod {
    Last24Hours,
    Last7Days,
    Last30Days,
}

struct AiUsageStats {
    total_requests: u64,
    total_tokens: u64,
    by_provider: Vec<ProviderStats>,
    by_app: Vec<AppAiStats>,
    average_response_time_ms: f64,
    error_rate_percent: f64,
}

struct ProviderStats {
    provider: String,
    model: String,
    request_count: u64,
    token_count: u64,
    average_response_time_ms: f64,
    error_count: u64,
}

struct AppAiStats {
    app_id: String,
    app_name: String,
    request_count: u64,
    token_count: u64,
}

struct ExportOptions {
    include_logs: bool,
    log_hours: u32,           // default: 24
    include_system_info: bool,
    include_running_apps: bool,
    include_permissions: bool,
    include_ai_usage: bool,
    include_crash_logs: bool,
    include_settings: bool,
    output_path: PathBuf,
}
```

### 9.3 Recovery API

```rust
trait Recovery {
    /// Get crash history for the current session.
    fn get_crash_history(&self) -> Result<Vec<CrashRecord>>;

    /// Attempt to relaunch a crashed app.
    fn relaunch_app(&self, app_id: &str) -> Result<()>;

    /// Clear crash history.
    fn clear_crash_history(&self) -> Result<()>;

    /// Get the last saved session state.
    fn get_session_state(&self) -> Result<Option<SessionState>>;

    /// Restore the last session (relaunch apps, position windows).
    fn restore_session(&self) -> Result<()>;

    /// Clear saved session state.
    fn clear_session(&self) -> Result<()>;

    /// Run filesystem checks.
    fn verify_filesystem(&self) -> Result<FsCheckResult>;

    /// Repair detected filesystem issues.
    fn repair_filesystem(&self, items: Vec<FsCheckItem>) -> Result<FsCheckResult>;

    /// Reset a specific app's settings.
    fn reset_app_settings(&self, app_id: &str) -> Result<()>;

    /// Reset all system settings to defaults.
    fn reset_system_settings(&self) -> Result<()>;

    /// Execute factory reset. This is irreversible.
    fn factory_reset(&self) -> Result<()>;

    /// Restart the system in safe mode.
    fn restart_safe_mode(&self) -> Result<()>;

    /// Check if the system is currently in safe mode.
    fn is_safe_mode(&self) -> bool;
}
```

---

## 10. Internal Interfaces

### 10.1 Metrics Collector (internal)

```rust
trait MetricsCollector {
    /// Start collecting metrics at the given interval.
    fn start(&self, interval: Duration) -> Result<()>;

    /// Stop collecting metrics.
    fn stop(&self) -> Result<()>;

    /// Get the latest sample from the in-memory ring buffer.
    fn latest(&self) -> Option<MetricSample>;

    /// Get samples from the ring buffer within a time range.
    fn range(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Vec<MetricSample>;

    /// Flush in-memory samples to disk.
    fn flush(&self) -> Result<()>;
}
```

### 10.2 Session Manager (internal)

```rust
trait SessionManager {
    /// Save the current session state.
    fn save_session(&self) -> Result<()>;

    /// Load the last saved session state.
    fn load_session(&self) -> Result<Option<SessionState>>;

    /// Mark the session as cleanly shut down.
    fn mark_clean_shutdown(&self) -> Result<()>;

    /// Restore a saved session.
    fn restore_session(&self, state: &SessionState) -> Result<()>;
}
```

### 10.3 Crash Handler (internal)

```rust
trait CrashHandler {
    /// Record a crash event.
    fn record_crash(&self, app_id: &str, exit_code: i32, signal: Option<i32>) -> Result<CrashRecord>;

    /// Check if an app should be auto-restarted.
    fn should_auto_restart(&self, app_id: &str) -> bool;

    /// Get crash log path for an app.
    fn crash_log_path(&self, app_id: &str) -> PathBuf;

    /// Clean up old crash logs beyond retention period.
    fn cleanup_old_crash_logs(&self, retention_days: u32) -> Result<u32>;
}
```

### 10.4 Diagnostic Bundler (internal)

```rust
trait DiagnosticBundler {
    /// Create a diagnostic ZIP bundle with the specified options.
    fn create_bundle(&self, options: &ExportOptions) -> Result<PathBuf>;

    /// Estimate the size of the bundle before creating it.
    fn estimate_size(&self, options: &ExportOptions) -> Result<u64>;

    /// Redact sensitive values from settings JSON.
    fn redact_settings(&self, settings_json: &str) -> String;
}
```

---

## 11. State Management

### 11.1 Persistent State

| Data | Location | Format | Retention |
|---|---|---|---|
| Metric history | `cortex-db` / admin metrics tables | SQLite + in-memory ring buffer | 24 hours persisted, 1 hour in memory |
| Session state | `cortex-db` / admin session-state tables | SQLite | Overwritten every 60s |
| Crash logs | `$XDG_DATA_HOME/cortexos/admin/crash_logs/` | Text files | 30 days (configurable) |
| Crash records | `cortex-db` / admin crash tables | SQLite | Current session (cleared on clean restart) |
| Safe mode flag | `$XDG_CONFIG_HOME/cortexos/safe_mode` | Empty file | Removed on normal boot |
| Dashboard layout | `$XDG_CONFIG_HOME/cortexos/admin/dashboard_layout.json` | JSON | Persistent |

### 11.2 In-Memory State

| State Key | Type | Persistence |
|---|---|---|
| `metrics_ring_buffer` | `VecDeque<MetricSample>` (max 720 entries for 1 hour at 5s interval) | Flushed to disk periodically |
| `crash_records` | `Vec<CrashRecord>` | Persisted to `crash_records.json` |
| `is_safe_mode` | `bool` | Read from flag file on startup |
| `session_state` | `Option<SessionState>` | Saved every 60 seconds |
| `dashboard_refresh_interval` | `Duration` | Persisted in settings |

### 11.3 Concurrency

- The metrics collector runs as a background `tokio` task.
- Session state saving runs as a periodic `tokio` task.
- Dashboard reads use `RwLock` on the metrics ring buffer (concurrent reads, exclusive writes).
- Factory reset runs in a dedicated process to avoid self-termination issues.

---

## 12. Failure Modes and Error Handling

### 12.1 Metrics Collector Failure

- **Trigger**: Unable to read system stats (e.g., `/proc` filesystem unavailable).
- **Detection**: Collector task returns an error.
- **Behavior**: Log error at `warn` level. Skip the sample. Dashboard shows a "Data unavailable" indicator for the affected metric.
- **Recovery**: Retry on next interval. If 10 consecutive failures, disable the collector and show a persistent warning.

### 12.2 Session State Corruption

- **Trigger**: stored session-state rows cannot be decoded or fail integrity checks.
- **Detection**: Load fails with parse error.
- **Behavior**: Treat as no saved session. Do not offer session recovery on next boot. Log at `warn` level.
- **Recovery**: Overwrite with fresh session state on next save cycle.

### 12.3 Crash During Factory Reset

- **Trigger**: System crash or power loss during factory reset execution.
- **Detection**: On next boot, a partial reset flag is detected.
- **Behavior**: Show a notification: "A factory reset was interrupted. Retry the reset to complete it."
- **Recovery**: User must re-run factory reset. The reset is designed to be idempotent: running it again will clean up any remaining data.

### 12.4 Diagnostic Export Failure

- **Trigger**: Disk full or permission denied when writing the ZIP file.
- **Detection**: ZIP write error.
- **Behavior**: Show error with specific message. Delete the partial ZIP file.
- **Recovery**: User frees disk space or chooses a different location.

### 12.5 Filesystem Repair Failure

- **Trigger**: Unable to repair an issue (e.g., cannot recreate a missing directory).
- **Detection**: Repair operation returns an error.
- **Behavior**: Mark the item as "Repair Failed" in the results list. Show the error message.
- **Recovery**: User can try manual repair or restart the system and retry.

### 12.6 Safe Mode Boot Failure

- **Trigger**: System cannot boot even in safe mode.
- **Detection**: Boot loader timeout or critical service failure.
- **Behavior**: System boots to a minimal recovery console (text-only).
- **Recovery**: User can run diagnostic commands from the console or perform a factory reset from the command line.

---

## 13. Security and Permissions

- The admin dashboard and diagnostic tools require admin-level access.
- Admin access is granted to the primary user account by default. Additional users must be granted admin access explicitly.
- Factory reset requires the user to be logged in and authenticated.
- Diagnostic exports redact sensitive values: API keys, passwords, authentication tokens, and any field matching common secret patterns (key, password, secret, token, auth, credential).
- Crash logs may contain user data (e.g., file content that was being processed). The export dialog warns the user about this.
- Safe mode does not bypass authentication. The user must still log in.
- The metrics collector does not collect or store any user content (only system-level statistics).

---

## 14. Performance Requirements

| Metric | Requirement |
|---|---|
| Dashboard initial load time | Less than 1 second |
| Dashboard refresh latency | Less than 200ms per refresh cycle |
| Metrics collector CPU overhead | Less than 0.5% CPU when collecting at 5-second intervals |
| Metrics collector memory overhead | Less than 10MB for 1-hour ring buffer |
| Log viewer initial load (10,000 entries) | Less than 500ms |
| Log viewer filter apply time | Less than 200ms |
| Diagnostic export (100MB of logs) | Less than 30 seconds |
| Factory reset execution time | Less than 60 seconds |
| Session state save time | Less than 100ms |
| Filesystem check time | Less than 10 seconds |
| Safe mode boot overhead | No more than 2 seconds slower than normal boot |

---

## 15. Accessibility Requirements

- Dashboard panels must be navigable by keyboard (Tab between panels, Enter to expand).
- Sparkline charts must have an accessible text summary: "CPU usage: 45%, range 20%-65% over the last 60 seconds."
- Log viewer must support keyboard scrolling and filtering.
- Severity indicators must use both color and icon/shape (not color alone).
- Factory reset confirmation must be fully keyboard-navigable, including the text input field.
- Export dialog checkboxes must have proper labels for screen readers.
- All progress bars must have `aria-valuenow`, `aria-valuemin`, `aria-valuemax` attributes and text labels.

---

## 16. Observability and Logging

### 16.1 Log Events

| Event | Level | Fields |
|---|---|---|
| `admin_metrics_collected` | debug | cpu_percent, memory_used_bytes, running_apps |
| `admin_metrics_flush` | debug | samples_flushed, file_path |
| `admin_session_saved` | debug | app_count, clean_shutdown |
| `admin_session_restored` | info | apps_restored |
| `admin_crash_recorded` | error | app_id, exit_code, signal, crash_log_path |
| `admin_app_auto_restarted` | info | app_id, attempt_number |
| `admin_crash_recovery_limit` | warn | app_id, max_attempts |
| `admin_fs_check_started` | info | checks_performed |
| `admin_fs_check_complete` | info | total, passed, warnings, errors |
| `admin_fs_repair_attempted` | info | item, success |
| `admin_diagnostic_export_started` | info | components_included |
| `admin_diagnostic_export_complete` | info | output_path, size_bytes |
| `admin_settings_reset` | warn | scope (app/system), target |
| `admin_factory_reset_started` | warn | user_id |
| `admin_factory_reset_step` | info | step, success |
| `admin_factory_reset_complete` | warn | duration_seconds |
| `admin_safe_mode_enabled` | warn | source (key_press/menu) |
| `admin_safe_mode_detected` | info | boot_mode |

### 16.2 Metrics

- `admin_metrics_collection_duration_ms` (histogram)
- `admin_session_save_duration_ms` (histogram)
- `admin_crashes_total` (counter, labels: app_id)
- `admin_auto_restarts_total` (counter, labels: app_id, outcome)
- `admin_diagnostic_exports_total` (counter)
- `admin_factory_resets_total` (counter)

---

## 17. Testing Requirements

### 17.1 Unit Tests

- Metric sample construction and validation (values in expected ranges).
- Session state serialization/deserialization.
- Crash record creation and cleanup.
- Log filter construction and matching (severity, component, time range, text search).
- Diagnostic export options validation (at least one component selected, output path writable).
- Filesystem check result aggregation (pass/warning/error counts).
- Factory reset step ordering (steps must execute in defined order).
- Sensitive value redaction in settings export (API keys, passwords, tokens must be replaced with "[REDACTED]").

### 17.2 Integration Tests

- Full metrics collection cycle: start collector -> wait for samples -> read from ring buffer -> flush to disk -> read from disk.
- Session save/restore cycle: launch apps -> save session -> clear in-memory state -> load session -> verify app list matches.
- Crash handling: simulate app crash -> verify crash record created -> verify auto-restart logic -> verify max restart attempts.
- Filesystem check: create a corrupt state (missing directory, invalid JSON) -> run check -> verify issues detected -> repair -> verify issues resolved.
- Diagnostic export: generate logs and metrics -> export with all options -> verify ZIP structure -> verify sensitive data redacted.
- Factory reset: create user data (apps, settings, AI history) -> execute reset -> verify all data removed -> verify system restarts cleanly.

### 17.3 UI Tests

- Dashboard: panel rendering, refresh behavior, sparkline display, keyboard navigation.
- Log viewer: filter application, live mode toggle, text search, copy/export.
- Recovery tools: crash list display, relaunch button, filesystem check result display.
- Factory reset: three-stage confirmation flow, text input validation, progress display.
- Export dialog: checkbox toggling, size preview, export progress, completion.

### 17.4 Performance Tests

- Metrics collector: verify CPU overhead is less than 0.5% over a 10-minute collection period.
- Dashboard: load dashboard with 50 running apps, verify initial load within 1 second.
- Log viewer: load 10,000 log entries, verify render within 500ms.
- Export: generate 100MB of log data, verify export completes within 30 seconds.

---

## 18. Acceptance Criteria

- [ ] System health dashboard displays CPU, memory, sessions, running apps, errors, disk usage, uptime.
- [ ] Dashboard refreshes at the configured interval (default: 5 seconds).
- [ ] Dashboard panels show 60-second sparkline history for CPU and memory.
- [ ] Running apps panel shows per-app PID, CPU, memory, and status.
- [ ] Error counts panel shows counts for 1h, 6h, 24h, and 7d windows.
- [ ] Log viewer displays log entries with filtering by severity, component, time range, and text search.
- [ ] Log viewer supports live streaming mode.
- [ ] Event inspector shows system events with expandable JSON payload.
- [ ] Permission audit shows all granted permissions across all apps.
- [ ] AI usage statistics show requests, tokens, response times, and error rates by provider.
- [ ] App crash recovery lists crashes and offers relaunch.
- [ ] Auto-restart works for apps declaring crash_recovery (max 3 attempts per session).
- [ ] Session recovery offers to restore apps after unclean shutdown.
- [ ] Session state is saved every 60 seconds.
- [ ] Filesystem verification detects missing files, orphaned directories, and corrupt configuration.
- [ ] Filesystem repair can fix detected issues with confirmation.
- [ ] Settings reset works for individual apps and system-wide.
- [ ] Diagnostic export creates a ZIP with user-selected components.
- [ ] Diagnostic export redacts API keys, passwords, and tokens.
- [ ] Safe mode boots with only first-party apps and no background services.
- [ ] Safe mode is indicated visually (color band, notification, system tray).
- [ ] Factory reset has three-stage confirmation (info, confirm, type "RESET").
- [ ] Factory reset deletes all apps, data, settings, and AI history.
- [ ] Factory reset is idempotent (safe to run again if interrupted).
- [ ] All admin tools are keyboard-navigable.
- [ ] Sparkline charts have accessible text summaries.
- [ ] Severity indicators use both color and icon/shape.
- [ ] All progress bars have accessible attributes.
- [ ] Metrics collector CPU overhead is less than 0.5%.
- [ ] Dashboard initial load is less than 1 second.

---

## 19. Build Order and Dependencies
**Layer 15**. Depends on: 09 (app runtime), 14 (observability), 11 (filesystem)

### 19.1 Crate Dependencies

```
cortex-admin depends on:
  - cortex-core (shared types)
  - cortex-db (persistent state)
  - cortex-files (filesystem operations for repair and reset)
  - cortex-settings (settings reset, reading system configuration)
  - @cortexos/ui-components (UI component library for dashboard, log viewer, dialogs)
  - cortex-sdk (app lifecycle information via PackageManager)
  - cortex-observability (AI usage statistics from audit log)
  - cortex-search (log search functionality)
  - serde / serde_json (serialization)
  - chrono (timestamps)
  - uuid (identifiers)
  - tokio (async runtime, background tasks)
  - tracing (logging and log entry capture)
  - zip (diagnostic export bundling)
  - sysinfo (system metrics: CPU, memory, disk, processes)
```

### 19.2 Build Order

1. `cortex-core` (shared types)
2. `cortex-db` (persistent state)
3. `cortex-files` (filesystem abstraction)
4. `cortex-settings` (settings infrastructure)
5. `cortex-search` (search primitives)
6. `@cortexos/ui-components` (UI components)
7. `cortex-observability` (audit/log access)
8. `cortex-sdk` (app lifecycle)
9. `cortex-admin` (this crate)

---

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Remote administration capabilities.
- Automated self-healing or auto-remediation.
- Telemetry sent to external servers (all diagnostics are local).
- Hardware diagnostics (POST, SMART, memtest).
- Network diagnostics (ping, traceroute, DNS).
- Performance profiling tools.
- Multi-user role-based admin access.
- Scheduled or automatic factory reset.
- Backup and restore of user data (separate feature, not part of admin).

### 20.2 Anti-Patterns

- **Silent factory reset**: Factory reset must never be triggered without explicit user interaction through the three-stage confirmation flow. No API call, script, or automation can bypass this.
- **Unredacted exports**: Diagnostic exports must always redact sensitive values. There is no "raw" export mode.
- **Crash loops**: Auto-restart must be capped at 3 attempts per session. After that, the app must not be restarted automatically to avoid crash loops.
- **Metric data hoarding**: Persisted metrics are limited to 24 hours in the CortexOS-managed store. Older data must be deleted. The system must not accumulate unbounded metric data.
- **Admin access without authentication**: All admin operations that modify system state (reset, repair, factory reset) require the user to be logged in and authenticated.
- **Safe mode bypass of security**: Safe mode does not disable authentication, permission checks, or sandboxing. It only disables third-party app loading and background services.
- **Partial repair without notification**: If a filesystem repair partially fails, the user must be informed of which items failed. Silent partial repair is prohibited.

---

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 File Structure

```
cortex-admin/
  src/
    lib.rs
    controller.rs           # AdminController: central orchestrator
    dashboard/
      mod.rs
      ui.rs                 # Dashboard UI component
      panels.rs             # Individual panel rendering (CPU, Memory, etc.)
      sparkline.rs          # Sparkline chart rendering utility
    metrics/
      mod.rs
      collector.rs          # MetricsCollector implementation
      ring_buffer.rs        # In-memory ring buffer for samples
      store.rs              # SQLite-backed metric persistence
    logs/
      mod.rs
      viewer.rs             # Log viewer UI and filtering
      stream.rs             # Real-time log streaming
    events/
      mod.rs
      inspector.rs          # Event inspector UI and querying
    permissions_audit.rs    # Permission audit view
    ai_stats.rs             # AI usage statistics view
    recovery/
      mod.rs
      crash_handler.rs      # Crash detection, recording, auto-restart
      session.rs            # Session state save/restore
      fs_check.rs           # Filesystem verification and repair
      settings_reset.rs     # App and system settings reset
      factory_reset.rs      # Factory reset execution
    export/
      mod.rs
      bundler.rs            # Diagnostic ZIP bundle creation
      redactor.rs           # Sensitive value redaction
    safe_mode.rs            # Safe mode detection and management
    models/
      mod.rs
      metrics.rs            # MetricSample, DiskMetricSample
      app_info.rs           # AppRunningInfo, CrashRecord
      session.rs            # SessionState, SessionApp
      fs_check.rs           # FsCheckResult, FsCheckItem
      diagnostic.rs         # DiagnosticBundle, SystemInfo, ExportOptions
    error.rs                # Admin-specific error types
    constants.rs            # Default values and limits
```

### 21.2 Implementation Order

1. **Phase 1 - Models and Constants** (`models/`, `constants.rs`, `error.rs`):
   - Define all data types from Section 8.
   - Define all constants (refresh intervals, retention periods, limits).
   - Implement `Serialize`/`Deserialize` for all types.
   - Write unit tests for model construction and validation.

2. **Phase 2 - Metrics Collector** (`metrics/collector.rs`, `metrics/ring_buffer.rs`, `metrics/store.rs`):
   - Implement `MetricsCollector` using `sysinfo` crate for system metrics.
   - Implement ring buffer with configurable capacity (default: 720 entries for 1 hour at 5s).
   - Implement SQLite-backed aggregate persistence with 1-minute rollups and 24-hour retention.
   - Test collection, ring buffer overflow, store flush, aggregation, and retention cleanup.

3. **Phase 3 - Session Manager** (`recovery/session.rs`):
   - Implement `SessionManager` with transactional SQLite writes.
   - Periodic save every 60 seconds via `tokio::spawn`.
   - Mark clean shutdown on graceful exit.
   - Test save/load/restore cycle.

4. **Phase 4 - Crash Handler** (`recovery/crash_handler.rs`):
   - Implement process monitoring via `sysinfo` or PID watching.
   - Record crash events with exit codes and signals.
   - Implement auto-restart logic with 3-attempt cap.
   - Implement crash log capture and storage.
   - Test crash detection, recording, auto-restart, and limit.

5. **Phase 5 - Log Viewer** (`logs/viewer.rs`, `logs/stream.rs`):
   - Implement log querying with filters (severity, component, time, text).
   - Implement real-time streaming via `tracing` subscriber.
   - Test filter logic, live streaming, and result limiting.

6. **Phase 6 - Event Inspector** (`events/inspector.rs`):
   - Implement event querying from the system event bus.
   - Test event filtering and payload display.

7. **Phase 7 - Filesystem Check and Repair** (`recovery/fs_check.rs`):
   - Implement check functions for: config files, app directories, permissions files, registry.
   - Implement repair functions for each check type.
   - Test with pre-created corrupt states.

8. **Phase 8 - Settings Reset** (`recovery/settings_reset.rs`):
   - Implement per-app settings reset.
   - Implement system-wide settings reset.
   - Test with populated settings.

9. **Phase 9 - Diagnostic Export** (`export/bundler.rs`, `export/redactor.rs`):
   - Implement ZIP bundle creation.
   - Implement sensitive value redaction (regex-based: patterns for API keys, passwords, tokens).
   - Implement size estimation before export.
   - Test with sample data, verify redaction.

10. **Phase 10 - Factory Reset** (`recovery/factory_reset.rs`):
    - Implement the 12-step reset process.
    - Implement idempotency (safe to re-run).
    - Implement partial reset detection and recovery.
    - Test with populated system data.

11. **Phase 11 - Safe Mode** (`safe_mode.rs`):
    - Implement safe mode flag file management.
    - Implement boot-time detection.
    - Implement first-party app filtering.
    - Test flag creation, detection, and removal.

12. **Phase 12 - Dashboard UI** (`dashboard/`, `permissions_audit.rs`, `ai_stats.rs`):
    - Implement all dashboard panels.
    - Implement sparkline rendering.
    - Implement permission audit view.
    - Implement AI usage statistics view.
    - Test UI rendering, keyboard navigation, and accessibility.

13. **Phase 13 - Controller** (`controller.rs`):
    - Wire all components together.
    - Implement the `AdminDashboard`, `Diagnostics`, and `Recovery` traits.
    - Integration test full flows.

### 21.3 Key Implementation Notes

- Use `sysinfo` crate for system metrics. Refresh system data on each collection interval, not on each dashboard read.
- The ring buffer implementation should use `VecDeque` with a fixed capacity. When full, the oldest sample is dropped.
- Session state must use transactional writes.
- The crash handler must register as a `tracing` subscriber to capture panic information.
- Diagnostic redaction must use a set of regex patterns that match common sensitive key names. The redaction replaces the value with `"[REDACTED]"` while preserving the key name and JSON structure.
- Factory reset steps must be logged individually. If any step fails, the process continues to the next step (best-effort deletion) but the failure is recorded.
- The metrics disk store must aggregate samples to 1-minute intervals before writing to reduce file size. Each hour file contains at most 60 aggregated samples.
- Safe mode flag file must be created before the restart command is issued and removed early in the boot sequence (before app loading).
- All background tasks (metrics collector, session saver) must handle graceful shutdown via `tokio::CancellationToken`.
- Use `tracing::instrument` on all public API methods.

### 21.4 Configuration Defaults

```rust
const DEFAULT_METRICS_INTERVAL_SECS: u64 = 5;
const METRICS_RING_BUFFER_CAPACITY: usize = 720; // 1 hour at 5-second intervals
const METRICS_DISK_RETENTION_HOURS: u32 = 24;
const METRICS_AGGREGATION_INTERVAL_SECS: u64 = 60;
const SESSION_SAVE_INTERVAL_SECS: u64 = 60;
const CRASH_LOG_RETENTION_DAYS: u32 = 30;
const MAX_AUTO_RESTART_ATTEMPTS: u32 = 3;
const DASHBOARD_MAX_LOG_ENTRIES: u32 = 10_000;
const DASHBOARD_DEFAULT_LOG_LIMIT: u32 = 1000;
const DEFAULT_EXPORT_LOG_HOURS: u32 = 24;
const MAX_EXPORT_LOG_HOURS: u32 = 168; // 7 days
const FACTORY_RESET_CONFIRMATION_TEXT: &str = "RESET";
const SAFE_MODE_FLAG_FILE: &str = "safe_mode";
const SAFE_MODE_SHIFT_WINDOW_SECS: u64 = 3;
const DIAGNOSTIC_FILE_PREFIX: &str = "cortexos-diagnostics-";
const DIAGNOSTIC_FILE_EXTENSION: &str = ".zip";
const REDACTED_VALUE: &str = "[REDACTED]";
```

### 21.5 Sensitive Value Redaction Patterns

The following patterns are used for redaction in diagnostic exports:

```rust
const REDACTION_PATTERNS: &[&str] = &[
    r#"(?i)"([^"]*(?:key|password|secret|token|auth|credential|api_key|apikey|access_token)[^"]*)"\s*:\s*"[^"]*""#,
    r#"(?i)"([^"]*(?:key|password|secret|token|auth|credential|api_key|apikey|access_token)[^"]*)"\s*:\s*\S+"#,
];
```

Each match is replaced with: `"<key_name>": "[REDACTED]"`.
