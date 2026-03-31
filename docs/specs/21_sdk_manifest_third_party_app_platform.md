# SPEC 21: SDK, Manifest, and Third-Party App Platform

**Spec ID:** 21
**Status:** Implementation-grade
**Risk Level:** Standard
**Crate:** cortex-sdk
**Last Updated:** 2026-03-30

---

## 1. Purpose

This specification defines the CortexOS SDK and third-party application platform. It establishes how third-party apps are described (manifest), packaged, installed, sandboxed, and updated. It defines what APIs are available to third-party apps, what operations are restricted, and how the platform ensures app safety through sandboxing and permission enforcement.

---

## 2. Scope

- Application manifest schema (all fields, validation rules, versioning)
- SDK API surface (filesystem, settings, notifications, AI abstraction, clipboard, search)
- SDK restrictions (prohibited operations)
- Application packaging format
- Application installation and uninstallation flow
- Application update flow
- Application sandboxing enforcement
- Developer documentation requirements
- Application review and verification process

---

## 3. Out of Scope

- First-party application development (first-party apps use internal APIs, not the SDK)
- IDE or developer tooling for building apps
- App store frontend or marketplace UI design
- Revenue sharing, billing, or payment processing for paid apps
- Cross-device app synchronization
- App extension system (apps extending other apps)

---

## 4. Objectives

1. Provide a comprehensive, well-documented SDK that enables third-party developers to build CortexOS applications.
2. Enforce strict sandboxing so third-party apps cannot compromise system stability or user privacy.
3. Define a clear manifest format that declares app metadata, permissions, and capabilities upfront.
4. Provide a deterministic installation, update, and uninstallation flow.
5. Enable AI capabilities for third-party apps through a safe abstraction layer.
6. Establish a review/verification process that can be enforced in future versions.

---

## 5. User-Visible Behavior

### 5.1 App Installation

- User downloads a `.cortex-app` package file.
- Opening the file launches the app installer.
- The installer displays:
  - App name, icon, version, author
  - Description
  - Requested permissions (each listed with risk level)
  - Minimum OS version compatibility
- User clicks "Install" to proceed.
- If any permission is High-risk, a separate confirmation is required for each.
- After installation, the app appears in the app launcher.
- Installation progress is shown with a progress bar.

### 5.2 App Uninstallation

- User navigates to Settings > Apps, selects the app, and clicks "Uninstall".
- Confirmation dialog: "Uninstall {app_name}? All app data will be removed."
- User confirms, uninstallation proceeds.
- All app files, data, settings, and permissions are removed.
- A brief "Uninstalling..." progress indicator is shown.

### 5.3 App Updates

- When an update is available, a notification appears: "{app_name} has an update available (v{old} -> v{new})."
- User can update immediately or dismiss.
- If the update requests new permissions, a dialog lists the new permissions for review.
- Update is applied atomically: the old version remains functional until the new version is fully verified.
- If verification fails, the old version is retained and an error is shown.

### 5.4 Permission Prompts

- When a third-party app first requests a permissioned resource, a prompt appears:
  - "{app_name} wants to access {permission_description}."
  - Buttons: "Allow", "Deny"
- If the user denies, the app receives a permission-denied error.
- If the user allows, the permission is remembered for future use.

---

## 6. System Behavior

### 6.1 Sandbox Enforcement

- Each third-party app runs in a sandboxed environment with the following constraints:
  - Filesystem access is restricted to the app's data directory: `$XDG_DATA_HOME/cortexos/apps/{app_id}/`.
  - Network access is gated by the `network` permission. Without it, all network calls return an error.
  - The app cannot access other apps' data directories.
  - The app cannot read or write system configuration files.
  - The app cannot execute arbitrary processes or shell commands.
  - The app cannot access hardware directly (GPU, camera, microphone) without explicit capability declarations.
- Sandboxing is enforced at the API layer: the SDK routing layer validates every request against the app's declared permissions before forwarding to the system service.

### 6.2 Manifest Validation

- On installation, the manifest is validated against the schema.
- All required fields must be present and valid.
- Unknown fields are ignored but logged at `warn` level.
- If validation fails, installation is aborted with a clear error message indicating which field(s) failed and why.

### 6.3 App Lifecycle

```
Installed -> Running -> Suspended -> Running
                   |-> Stopped
Stopped -> (uninstalled or launched again)
Installed -> (updated) -> Installed (new version)
```

- **Installed**: App is present on disk but not running.
- **Running**: App process is active and handling requests.
- **Suspended**: App process is paused (backgrounded). Retains state in memory.
- **Stopped**: App process is terminated. State is lost unless the app persisted it.

### 6.4 AI Abstraction Layer

- Third-party apps access AI capabilities through `cortex-sdk` AI API.
- Apps specify their AI needs in the manifest: `capabilities.ai = true`.
- Apps never see API keys or provider details.
- AI requests go through the standard AI-action permission pipeline defined in SPEC 20.
- Apps can register AI actions through `@cortexos/ai-client` hooks (SPEC 19).

---

## 7. Architecture

```
+-----------------------------------------------------------+
|                     Third-Party App                        |
|                                                            |
|  +------------------------------------------------------+ |
|  |              cortex-sdk (linked into app)             | |
|  |                                                      | |
|  |  +---------------+  +-------------+  +------------+ | |
|  |  | FileSystem    |  | Settings    |  | AI         | | |
|  |  | API           |  | API         |  | Abstraction| | |
|  |  +-------+-------+  +------+------+  +-----+------+ | |
|  |          |                  |               |        | |
|  |  +-------v------------------v---------------v------+ | |
|  |  |           SDK Permission Gate                   | | |
|  |  +----------------------+-------------------------+ | |
|  +-------------------------|---------------------------+ |
|                            |                              |
+----------------------------|-----------------------------+
                             |
         +-------------------v-------------------+
         |          SDK Routing Layer            |
         |  (validates permissions, routes to    |
         |   appropriate system service)         |
         +-------------------+-------------------+
                             |
    +------------+-----------+-----------+------------+
    |            |           |           |            |
+---v---+  +----v----+  +---v----+  +--v-----+  +--v------+
|cortex |  | cortex  |  |cortex  |  |cortex  |  |cortex   |
|-files |  |-settings|  |-notify |  |-ai-    |  |-clipboard|
|       |  |         |  |        |  |surface |  |         |
+-------+  +---------+  +--------+  +--------+  +---------+
```

### Component Responsibilities

- **cortex-sdk**: Library linked into third-party apps. Provides typed API clients for all permitted system services.
- **SDK Permission Gate**: Validates every API call against the app's manifest-declared permissions. Returns `PermissionDenied` if the app lacks the required permission.
- **SDK Routing Layer**: System-side service that receives API calls from apps, re-validates permissions, and routes to the appropriate system service.

---

## 8. Data Model

### 8.1 App Manifest

```rust
struct AppManifest {
    /// Unique identifier for the app. Must be reverse-domain format.
    /// Example: "com.example.text-editor"
    /// Pattern: ^[a-z0-9]+(\.[a-z0-9]+)*\.[a-z0-9-]+$
    /// Maximum length: 128 characters
    id: String,

    /// Human-readable application name.
    /// Maximum length: 64 characters
    /// Must not be empty.
    name: String,

    /// Semantic version string. Must follow semver (MAJOR.MINOR.PATCH).
    /// Pattern: ^\d+\.\d+\.\d+$
    version: String,

    /// Entry point: the main HTML file or WASM module to load.
    /// Must reference a file within the package.
    /// Example: "index.html" or "main.wasm"
    entry_point: String,

    /// Human-readable description of the application.
    /// Maximum length: 512 characters
    description: String,

    /// Path to the app icon within the package.
    /// Must be a PNG, SVG, or ICO file.
    /// Recommended size: 128x128 pixels.
    icon: String,

    /// Permissions requested by the app.
    permissions: Vec<AppPermission>,

    /// Capabilities declared by the app.
    capabilities: AppCapabilities,

    /// Minimum OS version required.
    /// Pattern: ^\d+\.\d+\.\d+$
    min_os_version: String,

    /// Author information.
    author: AuthorInfo,

    /// Optional: Category for app store organization.
    category: Option<AppCategory>,

    /// Optional: Homepage URL.
    homepage_url: Option<String>,

    /// Optional: Support/contact URL or email.
    support_url: Option<String>,

    /// Optional: License identifier (SPDX format preferred).
    license: Option<String>,
}

struct AuthorInfo {
    /// Author or organization name.
    /// Maximum length: 64 characters
    name: String,

    /// Optional: Author email address.
    email: Option<String>,

    /// Optional: Author website URL.
    url: Option<String>,
}

struct AppPermission {
    /// Permission name. Must be one of the defined permission types.
    permission: PermissionType,

    /// Human-readable reason why the app needs this permission.
    /// Shown to the user during installation.
    /// Maximum length: 256 characters
    reason: String,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum PermissionType {
    /// Read files in the app's sandbox directory.
    FilesRead,
    /// Write files in the app's sandbox directory.
    FilesWrite,
    /// Access the system clipboard.
    Clipboard,
    /// Display system notifications.
    Notifications,
    /// Access AI capabilities (through abstraction layer).
    Ai,
    /// Access user settings (app-scoped only).
    Settings,
    /// Access system search functionality.
    Search,
    /// Access network (HTTP/HTTPS outbound).
    Network,
    /// Access hardware features (camera, microphone, GPS).
    HardwareCamera,
    HardwareMicrophone,
    HardwareLocation,
}

struct AppCapabilities {
    /// Whether the app uses AI features.
    ai: bool,  // default: false

    /// Whether the app needs background execution.
    background_execution: bool,  // default: false

    /// Whether the app needs to run at system startup.
    autostart: bool,  // default: false

    /// Maximum file size the app may create (in bytes). 0 = unlimited.
    max_file_size: u64,  // default: 104857600 (100MB)

    /// Whether the app provides a system tray icon.
    system_tray: bool,  // default: false

    /// File type associations the app can handle.
    file_handlers: Vec<FileHandler>,  // default: empty
}

struct FileHandler {
    /// File extension (e.g., "txt", "md").
    extension: String,
    /// MIME type (e.g., "text/plain").
    mime_type: String,
    /// Human-readable label (e.g., "Text Document").
    label: String,
}

#[derive(Clone, Copy)]
enum AppCategory {
    Productivity,
    Communication,
    Development,
    Media,
    Education,
    Finance,
    Health,
    Games,
    Utilities,
    Other,
}
```

### 8.2 App Package

```rust
struct AppPackage {
    /// The manifest for this package.
    manifest: AppManifest,

    /// CRC32 checksums for all files in the package.
    checksums: HashMap<String, u32>,

    /// Signature of the manifest (Ed25519).
    /// Required for review-verified apps. Optional for sideloaded apps.
    signature: Option<Vec<u8>>,
}

/// On-disk format: a .cortex-app file is a ZIP archive with the following structure:
///
/// /manifest.json          - The AppManifest serialized as JSON
/// /signature.bin          - Optional Ed25519 signature of manifest.json
/// /checksums.json         - CRC32 checksums for all files
/// /icon.png               - App icon (as referenced in manifest)
/// /{entry_point}          - The main entry point file
/// /assets/                - Additional assets
/// /...                    - Other files referenced by the app
```

### 8.3 Installed App Record

```rust
struct InstalledApp {
    manifest: AppManifest,
    install_path: PathBuf,
    data_path: PathBuf,
    installed_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    install_source: InstallSource,
    granted_permissions: Vec<PermissionType>,
    state: AppState,
}

enum InstallSource {
    AppStore,
    Sideload,
    System,
}

enum AppState {
    Installed,
    Running { pid: u32, started_at: DateTime<Utc> },
    Suspended { pid: u32, started_at: DateTime<Utc>, suspended_at: DateTime<Utc> },
    Stopped,
}
```

### 8.4 Invariants

1. Every installed app has a valid manifest that passed schema validation.
2. An app's `id` is unique across all installed apps (no two apps can share an `id`).
3. An app's data directory is only accessible to that app (enforced by the SDK routing layer).
4. An app cannot request permissions at runtime that are not declared in its manifest.
5. Package checksums are verified on installation and on every app launch.
6. Manifest `min_os_version` is checked before installation. Installation is rejected if the current OS version is lower.
7. App `id` format is validated against the reverse-domain regex on installation.
8. Entry point file must exist in the package.
9. Icon file must exist in the package and be a valid image (PNG, SVG, or ICO).

---

## 9. Public Interfaces

### 9.1 SDK API (for third-party apps)

```rust
/// Filesystem API (sandboxed to app's data directory)
trait SdkFileSystem {
    /// Read a file from the app's sandbox.
    /// Requires: FilesRead permission
    fn read_file(&self, path: &str) -> Result<Vec<u8>>;

    /// Write a file to the app's sandbox.
    /// Requires: FilesWrite permission
    /// Enforces: max_file_size from capabilities
    fn write_file(&self, path: &str, data: &[u8]) -> Result<()>;

    /// List files in a directory within the app's sandbox.
    /// Requires: FilesRead permission
    fn list_dir(&self, path: &str) -> Result<Vec<DirEntry>>;

    /// Delete a file from the app's sandbox.
    /// Requires: FilesWrite permission
    fn delete_file(&self, path: &str) -> Result<()>;

    /// Check if a file exists in the app's sandbox.
    /// Requires: FilesRead permission
    fn file_exists(&self, path: &str) -> Result<bool>;

    /// Get metadata for a file in the app's sandbox.
    /// Requires: FilesRead permission
    fn file_metadata(&self, path: &str) -> Result<FileMetadata>;
}

struct DirEntry {
    name: String,
    is_directory: bool,
    size: u64,
    modified: DateTime<Utc>,
}

struct FileMetadata {
    size: u64,
    created: DateTime<Utc>,
    modified: DateTime<Utc>,
    is_readonly: bool,
}

/// Settings API (app-scoped)
trait SdkSettings {
    /// Get a setting value for this app.
    /// Requires: Settings permission
    fn get(&self, key: &str) -> Result<Option<serde_json::Value>>;

    /// Set a setting value for this app.
    /// Requires: Settings permission
    fn set(&self, key: &str, value: serde_json::Value) -> Result<()>;

    /// Delete a setting for this app.
    /// Requires: Settings permission
    fn delete(&self, key: &str) -> Result<()>;

    /// List all settings for this app.
    /// Requires: Settings permission
    fn list(&self) -> Result<HashMap<String, serde_json::Value>>;
}

/// Notifications API
trait SdkNotifications {
    /// Show a system notification.
    /// Requires: Notifications permission
    fn show_notification(&self, notification: Notification) -> Result<()>;

    /// Cancel a previously shown notification.
    fn cancel_notification(&self, notification_id: &str) -> Result<()>;
}

struct Notification {
    id: String,
    title: String,
    body: String,
    icon: Option<String>,
    actions: Vec<NotificationAction>,
}

struct NotificationAction {
    id: String,
    label: String,
}

/// AI Abstraction API
trait SdkAi {
    /// Send a prompt to the AI and receive a response.
    /// Requires: Ai permission
    /// The prompt and response go through the AI-action permission pipeline.
    fn prompt(&self, request: AiRequest) -> Result<AiResponse>;

    /// Register an AI action hook for this app.
    /// Requires: Ai permission
    fn register_ai_action(&self, action: AiActionRegistration) -> Result<()>;

    /// Stream a prompt response.
    /// Requires: Ai permission
    fn stream_prompt(&self, request: AiRequest) -> Result<mpsc::Receiver<StreamEvent>>;
}

struct AiRequest {
    prompt: String,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
}

struct AiResponse {
    text: String,
    tokens_used: u32,
    provider: String,
    model: String,
}

struct AiActionRegistration {
    action_id: String,
    label: String,
    description: String,
    category: String,
    requires_confirmation: bool,
}

/// Clipboard API
trait SdkClipboard {
    /// Read the system clipboard.
    /// Requires: Clipboard permission
    fn read(&self) -> Result<ClipboardContent>;

    /// Write to the system clipboard.
    /// Requires: Clipboard permission
    fn write(&self, content: ClipboardContent) -> Result<()>;
}

enum ClipboardContent {
    Text(String),
    Html(String),
}

/// Search API
trait SdkSearch {
    /// Perform a search query.
    /// Requires: Search permission
    /// Results are scoped to the app's own indexed data.
    fn search(&self, query: &str, options: SearchOptions) -> Result<Vec<SearchResult>>;
}

struct SearchOptions {
    limit: u32,  // default: 10
    offset: u32, // default: 0
}

struct SearchResult {
    id: String,
    title: String,
    snippet: String,
    relevance: f32,
}

/// Network API
trait SdkNetwork {
    /// Perform an HTTP request.
    /// Requires: Network permission
    fn fetch(&self, request: HttpRequest) -> Result<HttpResponse>;
}

struct HttpRequest {
    url: String,
    method: HttpMethod,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>,
    timeout_ms: u32, // default: 30000
}

enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

struct HttpResponse {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}
```

### 9.2 Package Manager API (for the installer/settings UI)

```rust
trait PackageManager {
    /// Install an app from a package file.
    fn install(&self, package_path: &PathBuf) -> Result<InstalledApp>;

    /// Uninstall an app by ID.
    fn uninstall(&self, app_id: &str) -> Result<()>;

    /// Update an app from a new package file.
    fn update(&self, package_path: &PathBuf) -> Result<InstalledApp>;

    /// Get information about an installed app.
    fn get_installed_app(&self, app_id: &str) -> Result<InstalledApp>;

    /// List all installed apps.
    fn list_installed(&self) -> Vec<InstalledApp>;

    /// Validate a package file without installing.
    fn validate_package(&self, package_path: &PathBuf) -> Result<AppManifest>;

    /// Launch an installed app.
    fn launch(&self, app_id: &str) -> Result<()>;

    /// Stop a running app.
    fn stop(&self, app_id: &str) -> Result<()>;
}
```

---

## 10. Internal Interfaces

### 10.1 Sandbox Enforcement Layer

```rust
trait SandboxEnforcer {
    /// Validate that a file path is within the app's sandbox.
    fn validate_path(&self, app_id: &str, path: &str) -> Result<PathBuf>;

    /// Validate that an API call is permitted for the app.
    fn validate_permission(&self, app_id: &str, permission: PermissionType) -> Result<()>;

    /// Get the app's data directory.
    fn app_data_dir(&self, app_id: &str) -> PathBuf;

    /// Get the app's install directory.
    fn app_install_dir(&self, app_id: &str) -> PathBuf;
}
```

### 10.2 Manifest Validator

```rust
trait ManifestValidator {
    /// Validate a manifest against the schema.
    /// Returns a list of validation errors (empty if valid).
    fn validate(&self, manifest: &AppManifest) -> Vec<ValidationError>;

    /// Validate a package's checksums.
    fn validate_checksums(&self, package_path: &PathBuf) -> Result<()>;

    /// Validate the package signature (if present).
    fn validate_signature(&self, package_path: &PathBuf) -> Result<SignatureStatus>;
}

struct ValidationError {
    field: String,
    message: String,
    severity: ValidationSeverity,
}

enum ValidationSeverity {
    Error,    // Blocks installation
    Warning,  // Logged but allows installation
}

enum SignatureStatus {
    Valid { signer: String },
    Invalid,
    NotSigned,
}
```

### 10.3 App Runtime

```rust
trait AppRuntime {
    /// Start an app process.
    fn start_app(&self, app_id: &str) -> Result<u32>; // returns PID

    /// Stop an app process.
    fn stop_app(&self, app_id: &str) -> Result<()>;

    /// Suspend an app process.
    fn suspend_app(&self, app_id: &str) -> Result<()>;

    /// Resume a suspended app process.
    fn resume_app(&self, app_id: &str) -> Result<()>;

    /// Get the state of an app.
    fn get_app_state(&self, app_id: &str) -> Result<AppState>;

    /// List all running apps.
    fn list_running(&self) -> Vec<(String, AppState)>;
}
```

---

## 11. State Management

### 11.1 Persistent State

| Data | Location | Format |
|---|---|---|
| Installed app records | `cortex-db` / app registry tables | SQLite |
| App manifests | `cortex-db` / manifest tables, with package copy on disk | SQLite + JSON artifact |
| App packages (extracted) | `$XDG_DATA_HOME/cortexos/apps/{app_id}/package/` | Directory tree |
| App data (sandboxed) | `$XDG_DATA_HOME/cortexos/apps/{app_id}/data/` | Directory tree |
| App settings | `cortex-db` / app settings tables | SQLite |
| Granted permissions | `cortex-db` / policy grant tables | SQLite |

### 11.2 In-Memory State

| State Key | Type | Persistence |
|---|---|---|
| `running_apps` | `HashMap<String, AppState>` | Not persisted |
| `manifest_cache` | `HashMap<String, AppManifest>` | Loaded from persistent store on demand |
| `permission_cache` | `HashMap<String, Vec<PermissionType>>` | Loaded from persistent store, refreshed on change |

### 11.3 State Invariants

- The app registry tables are the single source of truth for installed apps.
- App data directories are created on installation and removed on uninstallation.
- Registry mutations are transactional.
- Running app state is reconstructed from the process table on system startup (apps are not auto-started unless they declare `autostart: true`).

---

## 12. Failure Modes and Error Handling

### 12.1 Package Corruption

- **Trigger**: ZIP file cannot be extracted, checksums do not match.
- **Detection**: Checksum verification during installation.
- **Behavior**: Abort installation. Show error: "The app package is corrupted. Please re-download and try again."
- **Recovery**: User must obtain a new copy of the package.

### 12.2 Manifest Validation Failure

- **Trigger**: Manifest missing required fields, invalid format, or version mismatch.
- **Detection**: Schema validation during installation.
- **Behavior**: Abort installation. Show specific validation errors (field name and reason).
- **Recovery**: User must obtain a corrected package from the developer.

### 12.3 Insufficient OS Version

- **Trigger**: App's `min_os_version` is greater than the current OS version.
- **Detection**: Version comparison during installation.
- **Behavior**: Abort installation. Show error: "This app requires CortexOS {min_os_version} or later. You are running {current_version}."
- **Recovery**: User must update CortexOS.

### 12.4 Duplicate App ID

- **Trigger**: An app with the same `id` is already installed.
- **Detection**: Registry lookup during installation.
- **Behavior**: If the version is higher, offer to update. If the version is the same or lower, show error: "{app_name} is already installed."
- **Recovery**: User can uninstall the existing version first or use the update flow.

### 12.5 Sandbox Violation

- **Trigger**: App attempts to access a resource outside its sandbox.
- **Detection**: SDK Permission Gate or SandboxEnforcer rejects the request.
- **Behavior**: Return `PermissionDenied` error to the app. Log the violation at `warn` level.
- **Recovery**: The app must handle the error. Repeated violations (more than 10 in 60 seconds) trigger an automatic app suspension and a notification to the user.

### 12.6 App Crash

- **Trigger**: App process terminates unexpectedly.
- **Detection**: Process monitor detects PID disappearance.
- **Behavior**: Log the crash. Show notification: "{app_name} has crashed." Offer "Restart" and "Report" buttons.
- **Recovery**: User can restart the app. Crash reports are stored at `$XDG_DATA_HOME/cortexos/apps/{app_id}/crash_logs/`.

### 12.7 Update Failure

- **Trigger**: New package fails validation or checksum verification.
- **Detection**: Validation during update.
- **Behavior**: Abort update. Old version remains installed and functional. Show error with details.
- **Recovery**: User can try the update again or continue using the current version.

---

## 13. Security and Permissions

### 13.1 Permission Model

- Permissions are declared in the manifest at install time.
- Permissions are granted by the user during installation.
- Permissions cannot be escalated at runtime.
- Permissions can be revoked by the user from Settings > Apps at any time.
- If a permission is revoked, the app receives `PermissionDenied` for calls requiring that permission until the user re-grants it.

### 13.2 Sandboxing Rules

1. **Filesystem**: App can only read/write within `$XDG_DATA_HOME/cortexos/apps/{app_id}/`. Path traversal attacks (`../`) are prevented by canonicalizing and validating paths.
2. **Network**: Without `Network` permission, all outbound network requests are blocked.
3. **Process isolation**: Apps cannot signal or communicate with other app processes directly. All inter-app communication must go through the SDK routing layer.
4. **Memory**: Apps run in separate processes. Memory is isolated by the OS.
5. **Capabilities**: Background execution, autostart, and system tray require explicit capability declarations.

### 13.3 Package Signing

- Packages can be signed with Ed25519 signatures.
- In v1, signing is optional for sideloaded apps.
- If a signature is present, it is verified against the manifest.
- Future versions may require signatures for all apps installed from the app store.

---

## 14. Performance Requirements

| Metric | Requirement |
|---|---|
| Manifest validation time | Less than 50ms |
| Package installation time (10MB package) | Less than 5 seconds |
| App launch time | Less than 2 seconds (cold start), less than 500ms (warm start) |
| SDK API call overhead | Less than 1ms (permission check + routing) |
| App uninstallation time | Less than 3 seconds |
| Package checksum verification (10MB) | Less than 2 seconds |

---

## 15. Accessibility Requirements

- Installation dialogs must be keyboard-navigable.
- Permission lists must be readable by screen readers, with each permission announced with its name, risk level, and reason.
- App launcher must support keyboard navigation for app selection.
- Update notifications must be announced by screen readers.
- Error messages during installation must use `role="alert"` for announcement.

---

## 16. Observability and Logging

### 16.1 Log Events

| Event | Level | Fields |
|---|---|---|
| `sdk_app_installed` | info | app_id, version, source, permissions_granted |
| `sdk_app_uninstalled` | info | app_id, data_removed (bool) |
| `sdk_app_updated` | info | app_id, old_version, new_version, new_permissions |
| `sdk_app_launched` | info | app_id, version |
| `sdk_app_stopped` | info | app_id, reason (user/crash/suspend) |
| `sdk_app_crashed` | error | app_id, exit_code, crash_log_path |
| `sdk_permission_denied` | warn | app_id, permission, api_call |
| `sdk_sandbox_violation` | warn | app_id, attempted_path, api_call |
| `sdk_manifest_validation_failed` | error | field, message, severity |
| `sdk_package_integrity_failed` | error | app_id, expected_checksum, actual_checksum |
| `sdk_api_call` | debug | app_id, api, duration_ms |

### 16.2 Metrics

- `sdk_installed_apps` (gauge)
- `sdk_running_apps` (gauge)
- `sdk_app_installs_total` (counter, labels: source)
- `sdk_app_crashes_total` (counter, labels: app_id)
- `sdk_permission_denials_total` (counter, labels: app_id, permission)
- `sdk_api_call_duration_ms` (histogram, labels: api, app_id)

---

## 17. Testing Requirements

### 17.1 Unit Tests

- Manifest validation: valid manifests pass, every invalid field is caught (missing, wrong format, too long, bad pattern).
- Checksum verification: correct checksums pass, tampered files fail.
- Signature verification: valid signatures pass, invalid signatures fail, unsigned packages handled correctly.
- Sandbox path validation: paths within sandbox pass, path traversal (`../`, `/absolute/`) blocked.
- Permission checking: granted permissions pass, ungranted permissions denied, unknown permissions rejected.

### 17.2 Integration Tests

- Full installation flow: validate package -> extract -> register -> create sandbox -> grant permissions -> app appears in launcher.
- Full uninstallation flow: stop app -> remove files -> remove registry entry -> remove permissions -> app gone from launcher.
- Update flow: install v1 -> install v2 -> verify v2 running, v1 removed, data preserved.
- Update with new permissions: install v1 -> install v2 with extra permission -> permission prompt shown -> user grants/denies.
- App launch: launch -> verify running -> verify sandbox enforcement -> stop.
- Crash recovery: launch -> kill process -> verify crash logged -> verify restart works.

### 17.3 Security Tests

- Sandbox escape attempts: app tries to read/write outside sandbox via path traversal, symlinks, absolute paths. All must be blocked.
- Permission escalation: app tries to call an API without the required permission. Must be denied.
- Manifest tampering: modify manifest after installation. Verify app fails to launch with checksum mismatch.
- Inter-app data access: app A tries to read app B's data directory. Must be denied.

### 17.4 Performance Tests

- Install 10 apps in sequence: verify each takes less than 5 seconds.
- Launch 5 apps simultaneously: verify all launch within 3 seconds.
- 1000 SDK API calls in sequence: verify total overhead less than 1 second.

---

## 18. Acceptance Criteria

- [ ] App manifest schema is fully defined with all required and optional fields.
- [ ] Manifest validation rejects invalid manifests with specific error messages.
- [ ] Package format (.cortex-app ZIP) is defined and documented.
- [ ] Installation flow validates manifest, checksums, version, and permissions.
- [ ] Installation prompts user for permission review with risk levels.
- [ ] Uninstallation removes all app files, data, settings, and permissions.
- [ ] Update flow preserves user data and prompts for new permissions.
- [ ] Update is atomic: old version preserved if update fails.
- [ ] SDK provides typed APIs for: FileSystem, Settings, Notifications, AI, Clipboard, Search, Network.
- [ ] Every SDK API call is validated against the app's declared permissions.
- [ ] Apps cannot access resources outside their declared permissions.
- [ ] Sandbox restricts file access to the app's data directory.
- [ ] Path traversal attacks are prevented.
- [ ] Apps cannot execute arbitrary processes or shell commands.
- [ ] Apps cannot access other apps' data.
- [ ] Apps cannot modify system settings.
- [ ] Apps cannot access hardware without explicit capability declarations.
- [ ] Package checksums are verified on installation and launch.
- [ ] App review/verification flow is defined (even if not enforced in v1).
- [ ] Developer documentation requirements are specified.
- [ ] All installation and permission dialogs are keyboard-navigable.
- [ ] App crashes are detected, logged, and reported to the user.
- [ ] Sandbox violations are logged and trigger suspension after repeated offenses.

---

## 19. Build Order and Dependencies
**Layer 14**. Depends on: 04 (permissions), 09 (app runtime), 10 (command bus)

### 19.1 Crate Dependencies

```
cortex-sdk depends on:
  - cortex-core (shared types)
  - cortex-db (persistent storage)
  - cortex-files (filesystem abstraction, used internally by sandbox)
  - cortex-settings (settings storage for app settings)
  - cortex-auth (authentication for signed packages)
  - cortex-policy (permission framework)
  - serde / serde_json (serialization)
  - zip (package extraction)
  - chrono (timestamps)
  - uuid (identifiers)
  - tokio (async runtime)
  - tracing (logging)
  - ed25519-dalek (package signature verification)
  - crc32fast (checksum verification)
  - semver (version parsing and comparison)
  - regex (manifest field validation)
```

### 19.2 Build Order

1. `cortex-core` (shared types)
2. `cortex-auth` (authentication primitives)
3. `cortex-policy` (permission framework)
4. `cortex-db` (persistent storage)
5. `cortex-files` (filesystem abstraction)
6. `cortex-settings` (settings infrastructure)
7. `cortex-ai` (for AI abstraction)
8. `@cortexos/ai-client` (for AI action hooks)
9. `cortex-sdk` (this crate)

---

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- First-party app development tooling.
- App marketplace or store UI.
- Revenue or payment processing.
- Cross-device app sync.
- App extension mechanism (apps extending other apps).
- Plugin system within apps.
- Hot reloading or live code updates.
- Custom theme or visual customization for apps.

### 20.2 Anti-Patterns

- **Direct system access**: Apps must never call system APIs directly. All access must go through the SDK.
- **Permission bypass**: No code path in the SDK allows an API call to bypass permission checking.
- **Hardcoded paths**: App sandbox paths must be computed dynamically, never hardcoded.
- **Unvalidated manifests**: No app may be installed without passing manifest validation.
- **Unchecked packages**: No app may be installed without checksum verification.
- **Data leakage**: Apps must not be able to access another app's data through any SDK API.
- **Silent permission changes**: Permission changes (grant/revoke) must always be logged and visible in Settings.
- **Auto-update without consent**: App updates must never be applied without user confirmation, even for security patches.
- **Side-loading bypass**: Sideloaded apps must go through the same validation and permission flow as app store apps.

---

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 File Structure

```
cortex-sdk/
  src/
    lib.rs
    manifest/
      mod.rs
      schema.rs          # AppManifest, AuthorInfo, AppPermission, AppCapabilities, etc.
      validator.rs        # Manifest validation logic
    package/
      mod.rs
      reader.rs           # Package reading and extraction
      writer.rs           # Package creation (for developer tooling)
      checksum.rs         # CRC32 checksum generation and verification
      signature.rs        # Ed25519 signature verification
    sandbox/
      mod.rs
      enforcer.rs         # SandboxEnforcer: path validation and permission checks
      path_resolver.rs    # App sandbox path computation
    api/
      mod.rs
      filesystem.rs       # SdkFileSystem implementation
      settings.rs         # SdkSettings implementation
      notifications.rs    # SdkNotifications implementation
      ai.rs               # SdkAi implementation (abstraction layer)
      clipboard.rs        # SdkClipboard implementation
      search.rs           # SdkSearch implementation
      network.rs          # SdkNetwork implementation
    manager/
      mod.rs
      installer.rs        # Package installation flow
      uninstaller.rs      # App uninstallation flow
      updater.rs          # App update flow
      registry.rs         # Installed app registry management
    runtime/
      mod.rs
      launcher.rs         # App launch logic
      monitor.rs          # Process monitoring and crash detection
      lifecycle.rs        # App lifecycle state management
    review/
      mod.rs
      verification.rs     # App review/verification flow (defined, not enforced in v1)
    error.rs              # SDK-specific error types
    constants.rs          # Default values and limits
```

### 21.2 Implementation Order

1. **Phase 1 - Manifest Schema and Validation** (`manifest/schema.rs`, `manifest/validator.rs`):
   - Define all manifest types with `Serialize`/`Deserialize`.
   - Implement validation for every field (regex patterns, length limits, required fields).
   - Write exhaustive tests: valid manifests, every type of invalid field.
   - Use `semver::Version` for version parsing and comparison.

2. **Phase 2 - Package Handling** (`package/reader.rs`, `package/checksum.rs`, `package/signature.rs`):
   - Implement ZIP extraction with path sanitization (no absolute paths, no `../` traversal).
   - Implement CRC32 checksum generation and verification.
   - Implement Ed25519 signature verification.
   - Test with valid packages, tampered packages, unsigned packages.

3. **Phase 3 - Sandbox Enforcement** (`sandbox/enforcer.rs`, `sandbox/path_resolver.rs`):
   - Implement path resolution: app data dir, app install dir.
   - Implement path validation: canonicalize, verify within sandbox.
   - Implement permission checking: load granted permissions, check against required.
   - Test path traversal, symlink attacks, direct absolute paths.

4. **Phase 4 - SDK API Surface** (`api/`):
   - Implement each API trait: FileSystem, Settings, Notifications, AI, Clipboard, Search, Network.
   - Each implementation calls SandboxEnforcer for permission checks before routing.
   - FileSystem API enforces max_file_size from capabilities.
   - AI API routes through `cortex-ai` and `@cortexos/ai-client` with app context.
   - Test each API with permission granted and denied.

5. **Phase 5 - Package Manager** (`manager/`):
   - Installer: validate -> extract -> register -> sandbox -> permissions.
   - Uninstaller: stop -> remove files -> remove registry -> remove permissions.
   - Updater: validate new -> backup old -> extract new -> verify -> swap -> cleanup old.
   - Registry: transactional load/save of installed-app records.
   - Test full install, uninstall, update flows.

6. **Phase 6 - App Runtime** (`runtime/`):
   - Launcher: create process with sandbox environment.
   - Monitor: watch PID, detect crashes, log crash events.
   - Lifecycle: track state transitions (installed -> running -> suspended -> stopped).
   - Test launch, stop, crash detection, restart.

7. **Phase 7 - Review Flow** (`review/verification.rs`):
   - Define the verification process as a stub for v1.
   - Verification steps defined: automated checks (manifest, checksums, known vulnerability patterns), manual review (future).
   - In v1, all apps are treated as "unverified". The infrastructure exists but is not enforced.

### 21.3 Key Implementation Notes

- Package extraction must sanitize all file paths: reject entries with absolute paths, `..` components, or paths outside the intended directory.
- The SDK API layer must be a separate library that apps link against. System services (routing layer) must not be linkable by apps.
- Installed-app registry mutations must be transactional.
- Permission checks must use a cached in-memory copy of granted permissions, refreshed from disk on change (inotify).
- The AI abstraction API must not expose provider details, API keys, or model names to apps. Apps specify only the prompt and parameters.
- All timestamps must use UTC (`chrono::Utc`).
- Error types must implement `std::error::Error` and provide human-readable messages.
- The manifest validator must use a `match` or visitor pattern that requires handling every field, so adding new fields causes a compile error until the validator is updated.

### 21.4 Configuration Defaults

```rust
const DEFAULT_MAX_FILE_SIZE: u64 = 104_857_600; // 100MB
const DEFAULT_SANDBOX_BASE_PATH: &str = "apps";
const APP_REGISTRY_SCHEMA: &str = "app_registry";
const MANIFEST_FILE: &str = "manifest.json";
const SIGNATURE_FILE: &str = "signature.bin";
const CHECKSUMS_FILE: &str = "checksums.json";
const PACKAGE_EXTENSION: &str = ".cortex-app";
const MAX_SANDBOX_VIOLATIONS_BEFORE_SUSPEND: u32 = 10;
const SANDBOX_VIOLATION_WINDOW_SECS: u64 = 60;
const MAX_MANIFEST_ID_LENGTH: usize = 128;
const MAX_APP_NAME_LENGTH: usize = 64;
const MAX_DESCRIPTION_LENGTH: usize = 512;
const MAX_PERMISSION_REASON_LENGTH: usize = 256;
const MAX_AUTHOR_NAME_LENGTH: usize = 64;
const DEFAULT_NETWORK_TIMEOUT_MS: u32 = 30_000;
const DEFAULT_SEARCH_LIMIT: u32 = 10;
```

### 21.5 Developer Documentation Requirements

The following documentation must be produced for third-party developers:

1. **Getting Started Guide**: How to set up a development environment, create a minimal app, and run it.
2. **Manifest Reference**: Complete field-by-field documentation with examples and validation rules.
3. **SDK API Reference**: Every API method with parameters, return types, required permissions, and error cases.
4. **Packaging Guide**: How to create a `.cortex-app` package with correct structure and checksums.
5. **Permission Guide**: How to declare, request, and handle permissions.
6. **Sandboxing Guide**: What the sandbox restricts and how to work within it.
7. **AI Integration Guide**: How to use the AI abstraction API and register AI action hooks.
8. **Testing Guide**: How to test apps locally, including sandbox simulation.
9. **App Review Process**: Steps for submitting apps for review (future: enforced; v1: informational).

Each document must include code examples, common patterns, and troubleshooting sections.
