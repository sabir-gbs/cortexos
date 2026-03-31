# 09 — App Runtime and App Lifecycle

## 1. Purpose
Define the app runtime that manages the lifecycle of all applications in CortexOS, including first-party and third-party apps. Establishes the manifest contract, sandboxing rules, crash containment, and the guarantee that first-party apps receive no hidden privileges.

## 2. Scope
- App manifest schema and validation
- App lifecycle state machine
- Launch, suspend, resume, stop, crash recovery
- Single-instance vs multi-instance enforcement
- App sandboxing and capability isolation
- App version compatibility checking
- First-party vs third-party parity

## 3. Out of Scope
- Specific app implementations (owned by specs 17a–17g, 18a–18e)
- Window management (owned by spec 08)
- Inter-app communication protocol (owned by spec 10)
- Permission enforcement logic (owned by spec 04)

## 4. Objectives
1. Every app, first-party or third-party, follows the same lifecycle and permission model.
2. A crashed app never affects other apps or the OS shell.
3. App manifests are the single source of truth for an app's identity, capabilities, and requirements.
4. The runtime can restore previously running apps after an OS restart.

## 5. User-Visible Behavior

| Action | Outcome |
|---|---|
| User opens an app from launcher | App window appears, app transitions to Running |
| User closes an app window | App transitions to Stopped, resources freed |
| App crashes | Error state shown in window, other apps unaffected, auto-retry once |
| OS restarts | Previously running apps restored automatically |
| User opens already-running single-instance app | Existing window focused instead of new instance |

## 6. System Behavior

### 6.1 App Manifest Schema

```typescript
interface AppManifest {
  id: string;                  // Reverse-DNS: "com.cortexos.calculator"
  name: string;                // Display name: "Calculator"
  version: string;             // Semver: "1.0.0"
  entry_point: string;         // URL: "/apps/calculator-app/index.html"
  description: string;
  icon: string;                // URL to icon asset
  category: AppCategory;
  single_instance: boolean;    // true = only one window allowed
  min_os_version: string;      // Semver: "0.1.0"
  capabilities: Capability[];  // What the app can do
  permissions_needed: string[]; // e.g. ["file://documents/read", "clipboard://system/write"]
  author: string;
  homepage_url?: string;
}

type AppCategory =
  | "utilities" | "productivity" | "media" | "games" | "development"
  | "system" | "education" | "other";

type Capability =
  | "FileSystem"    // Read/write files via cortex-files
  | "Network"       // Make HTTP requests
  | "AI"            // Use AI runtime
  | "Clipboard"     // Read/write clipboard
  | "Notifications" // Send notifications
  | "Settings"      // Read settings (write requires permission)
  | "SystemInfo"    // Read OS version, device info
  | "Search"        // Register searchable items
  | "Themes";       // React to theme changes
```

### 6.2 Lifecycle State Machine

```
                    ┌──────────────────────────────────┐
                    │                                  │
  NotInstalled ──→ Installed ──→ Starting ──→ Running ─┤
                                  ↑           │  │     │
                                  │           │  ↓     │
                                  │    Suspended ←┘     │
                                  │           │         │
                                  │           ↓         │
                                  ├← Stopped ←┘         │
                                  │                     │
                                  └← Crashed ←──────────┘
                                         │
                                         ↓
                                     Uninstalled
```

**Valid transitions:**
- NotInstalled → Installed (install)
- Installed → Starting (launch)
- Starting → Running (started successfully)
- Starting → Crashed (failed to start)
- Running → Suspended (OS suspends for resources)
- Running → Stopped (user closes or app exits)
- Running → Crashed (unhandled error)
- Suspended → Running (resume)
- Suspended → Stopped (kill while suspended)
- Crashed → Starting (auto-retry, once only)
- Crashed → Stopped (after failed auto-retry, or user dismisses)
- Stopped → Starting (re-launch)
- Installed → Uninstalled (uninstall)

### 6.3 Launch Sequence
1. App launch requested (from launcher, file association, or restore)
2. Runtime loads and validates manifest
3. Check `min_os_version` compatibility → reject if incompatible
4. Check if `single_instance` and already running → focus existing window instead
5. Evaluate requested permissions against cortex-policy → grant/deny
6. Create sandboxed execution context (iframe or web worker)
7. Set up command bus connection for the app
8. Load `entry_point` URL into context
9. App sends "ready" signal → transition to Running
10. Emit `AppStarted` event

### 6.4 Crash Containment
- Each app runs in its own isolated context (sandboxed iframe)
- App crash = uncaught error in app context
- On crash: error boundary catches, shows error state in window
- Auto-retry: transition Crashed → Starting (once only, with 2s delay)
- If retry fails: stay Crashed, show "App crashed" with "Restart" button
- Other apps completely unaffected — no shared mutable state

### 6.5 Restore After OS Restart
1. Before shutdown: persist list of Running apps with their state
2. On startup: read persisted list, attempt to restore each
3. Restore = re-launch with saved state passed to app
4. Apps that fail restore go to Crashed state

## 7. Architecture

```
┌──────────────────────────────────────────┐
│           cortex-runtime (Rust)          │
│                                          │
│  ┌────────────────────────────────────┐  │
│  │         Manifest Registry          │  │
│  │  (load, validate, store manifests) │  │
│  └────────────────┬───────────────────┘  │
│                   │                      │
│  ┌────────────────┴───────────────────┐  │
│  │         Lifecycle Manager          │  │
│  │  (state machine, transitions,      │  │
│  │   launch, stop, crash recovery)    │  │
│  └────────────────┬───────────────────┘  │
│                   │                      │
│  ┌────────────────┴───────────────────┐  │
│  │         Sandbox Manager            │  │
│  │  (create isolated contexts,        │  │
│  │   enforce boundaries)              │  │
│  └────────────────────────────────────┘  │
└──────────────────────────────────────────┘
```

## 8. Data Model

```rust
struct AppRecord {
    id: String,                    // Manifest ID
    manifest: AppManifest,
    state: AppState,
    installed_at: chrono::DateTime<chrono::Utc>,
    last_launched_at: Option<chrono::DateTime<chrono::Utc>>,
    launch_count: u32,
    crash_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum AppState {
    NotInstalled,
    Installed,
    Starting,
    Running,
    Suspended,
    Stopped,
    Crashed { error: String, auto_retried: bool },
    Uninstalled,
}

struct AppInstance {
    instance_id: String,           // UUID per instance
    app_id: String,                // Manifest ID
    window_id: String,             // Window manager ID
    sandbox_context_id: String,    // Browser context ID
    started_at: chrono::DateTime<chrono::Utc>,
    state: AppState,
    granted_permissions: Vec<String>,
}
```

## 9. Public Interfaces

### REST API
```
GET    /api/v1/apps                      → List installed apps
GET    /api/v1/apps/{id}                 → Get app record
POST   /api/v1/apps/{id}/launch          → Launch app
POST   /api/v1/apps/{id}/stop            → Stop app
POST   /api/v1/apps/{id}/suspend         → Suspend app
POST   /api/v1/apps/{id}/resume          → Resume app
DELETE /api/v1/apps/{id}                 → Uninstall app
GET    /api/v1/apps/{id}/instances       → List running instances
```

### WebSocket Events
```
app.launched   → { app_id, instance_id, window_id }
app.stopped    → { app_id, instance_id }
app.crashed    → { app_id, instance_id, error }
app.suspended  → { app_id, instance_id }
app.resumed    → { app_id, instance_id }
```

## 10. Internal Interfaces
- Loads manifests from filesystem or bundled resources
- Checks permissions via `cortex-policy` trait
- Emits events via command bus (spec 10)
- Notifies window manager (spec 08) of new windows needed
- Persists state via `cortex-db`

## 11. State Management
- App records stored in SQLite (`apps` table)
- Running instances tracked in memory (lost on crash, recovered from persisted snapshot)
- App state snapshot taken on graceful shutdown
- Snapshot includes: running apps, their window positions, unsaved state references

## 12. Failure Modes and Error Handling

| Failure | Handling |
|---|---|
| Manifest validation fails | Reject install with VAL_003 |
| min_os_version incompatible | Reject launch with VAL_003 |
| Permission denied | Launch anyway with reduced capabilities, log warning |
| App fails to start | Transition to Crashed, auto-retry once |
| Auto-retry fails | Stay Crashed, show error in window |
| Sandbox creation fails | Transition to Crashed, log at ERROR |
| Persistence write fails | Log at ERROR, continue in-memory |

## 13. Security and Permissions
- Apps can only access capabilities declared in their manifest
- Apps can only communicate via command bus — no direct IPC
- App contexts are sandboxed (no access to other app DOMs, state, or cookies)
- First-party apps follow identical permission model as third-party (INV-09-1)
- No app can request elevated permissions at runtime beyond manifest declaration
- Permission grants are auditable

## 14. Performance Requirements

| Metric | Target |
|---|---|
| App launch (cold) | < 2s |
| App launch (warm, cached) | < 500ms |
| App stop | < 200ms |
| State snapshot on shutdown | < 1s total |
| Crash detection and recovery | < 3s |

## 15. Accessibility Requirements
- App launcher is keyboard-navigable
- Running apps listed in taskbar with accessible names
- Crash notifications are ARIA live announcements
- Alt+Tab window switching is accessible

## 16. Observability and Logging
- Every lifecycle transition logged at INFO: `{app_id, old_state, new_state, instance_id}`
- Crashes logged at ERROR with stack trace (if available)
- Launch failures logged at WARN
- Metrics: `app_launch_duration_ms`, `app_crash_count`, `apps_running_count`

## 17. Testing Requirements
- Unit: lifecycle state machine (all valid/invalid transitions)
- Unit: manifest validation (missing fields, invalid semver, unknown capabilities)
- Integration: launch → verify sandbox created → verify permissions checked → verify event emitted
- Integration: crash → verify auto-retry → verify other apps unaffected
- E2E: install app, launch, use, close, restart OS, verify restore

## 18. Acceptance Criteria
- [ ] App manifest validates all required fields
- [ ] Lifecycle state machine enforces only valid transitions
- [ ] Single-instance apps: second launch focuses existing window
- [ ] Crashed app auto-retries once, then shows error state
- [ ] Other apps unaffected by any single app crash
- [ ] First-party apps have no hidden privileges (same permission checks)
- [ ] App state restored after OS restart
- [ ] min_os_version incompatibility blocks launch with clear message
- [ ] All lifecycle transitions emit events

## 19. Build Order and Dependencies
**Layer 8**. Depends on: 01, 02, 04 (permissions), 05 (settings), 10 (command bus)

## 20. Non-Goals and Anti-Patterns
- No hot-reloading of app code (v1)
- No app sandboxing beyond browser iframe isolation (v1)
- No background app execution (apps stop when closed)
- NEVER grant first-party apps hidden capabilities
- NEVER allow apps to directly access other apps' state
- NEVER skip permission checks for first-party apps
- NEVER silently recover from crashes without user visibility

## 21. Implementation Instructions for Claude Code / Codex
1. Define AppManifest, AppState, AppInstance structs in cortex-runtime.
2. Implement manifest loader and validator (required fields, semver parsing).
3. Implement lifecycle state machine with transition validation.
4. Implement launch sequence: manifest → permission check → sandbox create → load entry → wait for ready.
5. Implement crash detection and auto-retry logic.
6. Implement state persistence for restore.
7. Write tests: every state transition, manifest validation, crash recovery.
