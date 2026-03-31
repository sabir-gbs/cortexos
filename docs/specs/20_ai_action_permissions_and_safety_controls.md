# SPEC 20: AI Action Permissions and Safety Controls

**Spec ID:** 20
**Status:** Implementation-grade
**Risk Level:** HIGH-RISK
**Owner Crate:** `cortex-policy` (AI action guard), with audit persistence in `cortex-observability`
**Last Updated:** 2026-03-29

---

## 1. Purpose

This specification defines the permission model, safety controls, and audit trail for all AI actions in CortexOS. It establishes what resources AI can read, modify, and create; what user confirmations are required for each action category; how permissions are granted and revoked; and how every AI action is logged for accountability. This is a security-critical specification: no AI action may bypass the controls defined here.

---

## 2. Scope

- AI read permissions: user messages, selected text, file contents, app state, conversation history
- AI write permissions: file contents, clipboard, app state (all with explicit user confirmation)
- AI create permissions: new files, new notes, drafts (with varying confirmation levels)
- Required user confirmations for destructive and modifying actions
- Action schemas with typed parameters and risk levels
- Risk level classification: Low, Medium, High
- Permission revocation controls accessible from Settings
- Audit trail: logging of every AI action with full metadata
- Explicit anti-patterns preventing unsafe mutation shortcuts

---

## 3. Out of Scope

- General OS permission system (covered in separate permission framework spec)
- AI provider authentication and API key management (covered in SPEC 06 and Settings)
- AI model safety and content filtering (handled by providers)
- Network-level security (TLS, certificate pinning)
- User account and session management
- Filesystem-level access control lists

---

## 4. Objectives

1. Ensure every AI action is governed by explicit, enforceable permission rules.
2. Require user confirmation for any action that modifies, overwrites, or deletes user data.
3. Classify all AI actions into risk levels with corresponding confirmation requirements.
4. Maintain a complete, immutable audit trail of every AI action.
5. Allow users to revoke AI permissions at any time without restarting the system.
6. Prevent any circumvention of the permission system by AI, apps, or system components.
7. Ensure AI actions are never silent: every action is visible in the audit log.

---

## 5. User-Visible Behavior

### 5.1 Permission Prompts

- When an AI action requires a permission that has not been granted, the user sees a permission prompt dialog:
  - Title: "AI Permission Request"
  - Body: "{app_name} wants to {action_description} on {resource_description}."
  - Risk level indicator: a colored badge (green for Low, yellow for Medium, red for High)
  - Buttons: "Allow" (primary), "Deny" (secondary), "Always allow for this app" (tertiary)
- "Always allow for this app" bypasses future prompts for the same `(app_id, action_type, resource_type)` triple, but the action is still logged.

### 5.2 Confirmation Dialogs

- For Medium-risk actions, a brief confirmation is shown:
  - Title: "Confirm AI Action"
  - Description: "{action_type} will {effect_description}."
  - Buttons: "Confirm" (primary), "Cancel" (secondary)
- For High-risk actions, a detailed confirmation is shown:
  - Title: "Confirm Destructive AI Action"
  - Description: "{action_type} will {effect_description}. This action cannot be undone."
  - Preview: Full diff or description of what will change
  - Buttons: "Apply" (primary, requires two clicks or hold-for-1-second), "Cancel" (secondary)
  - The "Apply" button for High-risk actions has a 1-second hold requirement to prevent accidental confirmation.

### 5.3 Revocation UI

- Settings > AI > Permissions shows a list of all granted permissions grouped by app.
- Each entry shows: app name, granted permissions, risk level, grant date.
- User can revoke individual permissions or all permissions for an app.
- Revocation takes effect immediately (no restart required).
- A revoked permission triggers the permission prompt again on next use.

### 5.4 Audit Log Viewer

- Settings > AI > Activity shows the audit log.
- Entries are displayed in reverse chronological order.
- Each entry shows: timestamp, app name, action type, target resource, risk level, outcome (success/denied/failed).
- The log can be filtered by: app, action type, risk level, date range, outcome.
- The log can be exported as JSON.

---

## 6. System Behavior

### 6.1 Permission Check Flow

Every AI action goes through the following flow:

```
1. Action initiated (by user or app)
2. Action schema validated (must match a known action type)
3. Risk level determined from action schema
4. Permission check:
   a. Has the user granted permission for (app_id, action_type, resource_type)?
   b. If not, show permission prompt.
   c. If denied, log and abort.
5. Confirmation check:
   a. Is the risk level Low? Skip confirmation.
   b. Is the risk level Medium? Show brief confirmation.
   c. Is the risk level High? Show detailed confirmation.
   d. If cancelled, log and abort.
6. Execute action.
7. Log outcome to audit trail.
8. Return result.
```

### 6.2 Permission Grant Storage

- AI-action permission grants are stored in SQLite via `cortex-db`.
- Canonical columns:
  - `app_id`
  - `action_type`
  - `resource_type`
  - `risk_level`
  - `granted_at`
  - `source`
- The `source` field indicates how the permission was granted: `user_prompt` or `settings`.
- Permissions are loaded into memory on startup and invalidated when the underlying policy records change.

### 6.3 Audit Trail Storage

- Audit entries are stored in an append-only SQLite table owned by `cortex-observability`.
- Retention is enforced by a background purge job using `ai.audit_retention_days` (default: 90).
- Audit rows are immutable after insert from the perspective of user-facing behavior.

---

## 7. Architecture

```
+-----------------------------------------------------------+
|        AI action guard (`cortex-policy`)                  |
|                                                            |
|  +------------------------+  +-------------------------+  |
|  | Permission Guard       |  | Audit Logger            |  |
|  | (check + prompt)       |  | (append-only log)       |  |
|  +-----------+------------+  +------------+------------+  |
|              |                             |               |
|  +-----------v------------+  +------------v------------+  |
|  | Permission Store       |  | Audit Store             |  |
|  | (grants + revocation)  |  | (append-only SQLite)    |  |
|  +-----------+------------+  +-------------------------+  |
|              |                                             |
|  +-----------v------------------------------------------+  |
|  |              Action Schema Validator                  |  |
|  +-----------+------------------------------------------+  |
|              |                                             |
|  +-----------v------------------------------------------+  |
|  |              Risk Classifier                         |  |
|  +------------------------------------------------------+  |
|                                                            |
+-----------------------------------------------------------+
         |
         | (called by)
         v
+---------------------------+
|   cortex-ai /             |
|   @cortexos/ai-client     |
|   (initiates actions)     |
+---------------------------+
         |
         | (reads/writes via)
         v
+---------------------------+
|   cortex-files            |
|   (file access)           |
+---------------------------+
```

---

## 8. Data Model

### 8.1 Action Schema

Every AI action must conform to the following schema:

```rust
struct AiAction {
    action_id: Uuid,
    action_type: AiActionType,
    target_resource: TargetResource,
    parameters: HashMap<String, serde_json::Value>,
    risk_level: RiskLevel,
    initiator: Initiator,
    timestamp: DateTime<Utc>,
}

enum AiActionType {
    // Read operations
    TextRead,
    FileRead,
    AppStateRead,
    ConversationHistoryRead,

    // Modify operations
    FileModify,
    ClipboardWrite,
    AppStateModify,

    // Create operations
    FileCreate,
    NoteCreate,
    DraftCreate,

    // Delete operations
    FileDelete,

    // Generate operations (read-only output, no modification)
    TextGenerate,
    TextSummarize,
    TextTranslate,
    TextExplain,
    TextRewrite,
}

enum TargetResource {
    SelectedText { length: u32 },
    File { path: PathBuf },
    Files { paths: Vec<PathBuf> },
    Clipboard,
    AppState { app_id: String, state_key: String },
    Conversation { conversation_id: Uuid },
    None, // for actions with no specific target (e.g., general chat)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RiskLevel {
    Low,    // Read-only, no confirmation
    Medium, // Create/append, brief confirmation
    High,   // Overwrite/delete/modify, detailed confirmation
}

struct Initiator {
    user_id: String,
    app_id: String,
    source: InitiatorSource,
}

enum InitiatorSource {
    UserAction,        // Direct user interaction
    AppHook,           // App-registered AI action
    AssistantPanel,    // Global assistant panel
}
```

### 8.2 Risk Level Classification

The risk level is determined by the action type according to the following immutable mapping:

| Action Type | Risk Level | Confirmation |
|---|---|---|
| `TextRead` | Low | None |
| `FileRead` | Low | None |
| `AppStateRead` | Low | None |
| `ConversationHistoryRead` | Low | None |
| `TextGenerate` | Low | None |
| `TextSummarize` | Low | None |
| `TextTranslate` | Low | None |
| `TextExplain` | Low | None |
| `DraftCreate` | Low | None (drafts are non-destructive) |
| `NoteCreate` | Medium | Brief confirmation |
| `FileCreate` | Medium | Brief confirmation |
| `ClipboardWrite` | Medium | Brief confirmation |
| `TextRewrite` | Medium | Brief confirmation (result shown, user chooses to apply) |
| `FileModify` | High | Detailed confirmation with diff preview |
| `AppStateModify` | High | Detailed confirmation |
| `FileDelete` | High | Detailed confirmation |

### 8.3 Permission Grant

```rust
struct PermissionGrant {
    app_id: String,
    action_type: AiActionType,
    resource_type: ResourceType,
    risk_level: RiskLevel,
    granted_at: DateTime<Utc>,
    source: GrantSource,
}

enum ResourceType {
    Text,
    File,
    Clipboard,
    AppState,
    Conversation,
    None,
}

enum GrantSource {
    UserPrompt,  // Granted via permission dialog
    Settings,    // Granted via Settings UI
}
```

### 8.4 Audit Entry

```rust
struct AuditEntry {
    entry_id: Uuid,
    timestamp: DateTime<Utc>,
    user_id: String,
    app_id: String,
    action_type: AiActionType,
    target_resource: TargetResource,
    parameters: HashMap<String, serde_json::Value>,
    risk_level: RiskLevel,
    outcome: AuditOutcome,
    provider: String,
    model: String,
    duration_ms: u64,
    tokens_used: u32,
}

enum AuditOutcome {
    Success,
    DeniedByUser,
    DeniedByPermission,
    Failed { error: String },
    Cancelled,
}
```

### 8.5 Invariants

1. No `AiAction` may be executed without a corresponding `AuditEntry` being written first (write audit, then execute).
2. `RiskLevel` mapping is static and cannot be overridden by apps or configuration.
3. Every `PermissionGrant` has exactly one `(app_id, action_type, resource_type)` key; duplicates are updated, not accumulated.
4. `AuditEntry.outcome` is never null; it is always one of the defined variants.
5. Audit entries are never deleted except by the 90-day rotation.
6. The audit record is never modified after insertion (append-only semantics).
7. Permission grants are loaded atomically on startup; partial reads are rejected.

---

## 9. Public Interfaces

### 9.1 Permission Guard API

```rust
trait PermissionGuard {
    /// Check if an action is permitted and perform confirmation if needed.
    /// Returns PermissionResult::Granted if the action may proceed.
    /// Returns PermissionResult::Denied if the user or system denied it.
    /// This method may show UI dialogs (permission prompt, confirmation dialog).
    fn authorize_action(
        &self,
        action: &AiAction,
    ) -> Result<PermissionResult>;

    /// Check if a permission is already granted without showing any UI.
    /// Used for pre-checking before initiating an action.
    fn is_permission_granted(
        &self,
        app_id: &str,
        action_type: &AiActionType,
        resource_type: &ResourceType,
    ) -> bool;

    /// Grant a permission programmatically (used by Settings UI).
    fn grant_permission(
        &self,
        grant: PermissionGrant,
    ) -> Result<()>;

    /// Revoke a permission.
    fn revoke_permission(
        &self,
        app_id: &str,
        action_type: &AiActionType,
        resource_type: &ResourceType,
    ) -> Result<()>;

    /// Revoke all permissions for an app.
    fn revoke_all_permissions(&self, app_id: &str) -> Result<()>;

    /// List all granted permissions, optionally filtered by app.
    fn list_permissions(&self, app_id: Option<&str>) -> Vec<PermissionGrant>;
}

enum PermissionResult {
    Granted,
    Denied { reason: DenialReason },
}

enum DenialReason {
    UserDenied,
    PermissionNotGranted,
    RiskLevelTooHigh,
}
```

### 9.2 Audit Logger API

```rust
trait AuditLogger {
    /// Log an audit entry. This must be called before executing the action.
    fn log_action(&self, entry: AuditEntry) -> Result<()>;

    /// Query audit entries with filters.
    fn query_entries(&self, filter: AuditFilter) -> Result<Vec<AuditEntry>>;

    /// Export audit entries as JSON string.
    fn export_entries(&self, filter: AuditFilter) -> Result<String>;

    /// Get audit statistics.
    fn get_stats(&self, since: DateTime<Utc>) -> Result<AuditStats>;
}

struct AuditFilter {
    app_id: Option<String>,
    action_type: Option<AiActionType>,
    risk_level: Option<RiskLevel>,
    outcome: Option<AuditOutcome>,
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    limit: u32,  // default: 100
}

struct AuditStats {
    total_actions: u64,
    actions_by_risk_level: HashMap<RiskLevel, u64>,
    actions_by_outcome: HashMap<String, u64>,
    unique_apps: u32,
    total_tokens_used: u64,
    average_duration_ms: f64,
}
```

### 9.3 Action Schema Validator API

```rust
trait ActionSchemaValidator {
    /// Validate that an action conforms to its schema.
    /// Returns Ok(()) if valid, Err with details if invalid.
    fn validate(&self, action: &AiAction) -> Result<()>;

    /// Determine the risk level for an action type.
    fn classify_risk(&self, action_type: &AiActionType) -> RiskLevel;

    /// Check if an action type requires confirmation.
    fn requires_confirmation(&self, action_type: &AiActionType) -> bool;

    /// Get the confirmation type required.
    fn confirmation_type(&self, action_type: &AiActionType) -> ConfirmationType;
}

enum ConfirmationType {
    None,
    Brief,
    Detailed,
}
```

---

## 10. Internal Interfaces

### 10.1 Permission Store (internal)

```rust
trait PermissionStore {
    fn load(&self) -> Result<PermissionFile>;
    fn save(&self, file: &PermissionFile) -> Result<()>;
    fn find_grant(&self, app_id: &str, action_type: &AiActionType, resource_type: &ResourceType) -> Option<PermissionGrant>;
    fn upsert_grant(&self, grant: PermissionGrant) -> Result<()>;
    fn remove_grant(&self, app_id: &str, action_type: &AiActionType, resource_type: &ResourceType) -> Result<()>;
    fn remove_all_grants(&self, app_id: &str) -> Result<()>;
    fn list_grants(&self, app_id: Option<&str>) -> Vec<PermissionGrant>;
}

struct PermissionFile {
    version: u32,
    grants: Vec<PermissionGrant>,
}
```

### 10.2 Audit Store (internal)

```rust
trait AuditStore {
    /// Append an audit entry to the append-only store.
    fn append(&self, entry: &AuditEntry) -> Result<()>;

    /// Read entries from a date range.
    fn read_range(&self, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<AuditEntry>>;

    /// Rotate files older than retention_days.
    fn rotate(&self, retention_days: u32) -> Result<u32>;

    /// Get the current day's file path.
    fn current_file_path(&self) -> PathBuf;
}
```

### 10.3 UI Dialog Service (internal)

```rust
trait PermissionDialogService {
    /// Show a permission prompt dialog.
    fn show_permission_prompt(
        &self,
        app_name: &str,
        action_description: &str,
        resource_description: &str,
        risk_level: RiskLevel,
    ) -> PermissionPromptResult;

    /// Show a brief confirmation dialog.
    fn show_brief_confirmation(
        &self,
        action_type: &str,
        effect_description: &str,
    ) -> ConfirmationResult;

    /// Show a detailed confirmation dialog with preview.
    fn show_detailed_confirmation(
        &self,
        action_type: &str,
        effect_description: &str,
        preview: ConfirmationPreview,
    ) -> ConfirmationResult;
}

enum PermissionPromptResult {
    Allow,
    Deny,
    AlwaysAllow,
}
```

---

## 11. State Management

### 11.1 In-Memory State

| State Key | Type | Persistence |
|---|---|---|
| `granted_permissions` | `HashMap<(String, AiActionType, ResourceType), PermissionGrant>` | Persisted to SQLite grant records |
| `active_authorizations` | `HashMap<Uuid, AiAction>` | Not persisted (tracks in-flight actions) |
| `audit_buffer` | `Vec<AuditEntry>` | Flushed to SQLite every 1 second or when buffer reaches 10 entries |

### 11.2 Persistence Invariants

- Permission grant writes are committed transactionally.
- Audit entries are flushed to SQLite at minimum every 1 second.
- On graceful shutdown, the audit buffer is flushed immediately.
- On crash, at most 10 audit entries may be lost (the buffer size). This is acceptable because the write-ahead audit for High-risk actions is flushed immediately before execution.
- High-risk actions flush the audit buffer synchronously before execution begins.

### 11.3 Concurrency

- `granted_permissions` is protected by a `RwLock`. Reads do not block other reads. Writes acquire an exclusive lock.
- `active_authorizations` is protected by a `Mutex`. Only one authorization flow can be in progress per action at a time.
- `audit_buffer` is protected by a `Mutex`. The background flush task acquires the lock every 1 second or when signaled.

---

## 12. Failure Modes and Error Handling

### 12.1 Permission Store Unavailable

- **Trigger**: Cannot read or write AI-action permission grant records.
- **Behavior**: Deny all actions that require permission checks. Show a persistent notification: "AI permissions are unavailable. AI actions that require permissions are disabled."
- **Log**: Error level with full storage error details.
- **Recovery**: User must resolve the storage issue. A "Repair Permissions" button is offered in Settings > AI > Permissions that rebuilds the grant store from an empty state after explicit confirmation.

### 12.2 Audit Store Unavailable

- **Trigger**: Cannot write to the AI audit store.
- **Behavior**: AI actions continue to function, but a warning notification is shown: "AI action logging is unavailable. Actions will proceed but will not be recorded."
- **Log**: Error level with storage error details.
- **Recovery**: Automatic retry on next action. If 10 consecutive failures, disable AI actions entirely and show a persistent error.

### 12.3 Concurrent Permission Modification

- **Trigger**: User revokes a permission while an action using that permission is in progress.
- **Behavior**: The in-progress action completes. The revocation takes effect for the next action.
- **Rationale**: Revoking mid-execution could leave files in a corrupt state. Better to let the current action complete.

### 12.4 Malformed Permission File

- **Trigger**: Permission grant records fail integrity checks or cannot be decoded.
- **Behavior**: Deny all permissioned actions. Show notification with repair option.
- **Recovery**: "Repair Permissions" rebuilds the grant store from an empty state after explicit confirmation.

### 12.5 Audit Entry Write Failure

- **Trigger**: Disk write fails for an audit entry.
- **Behavior**: For High-risk actions, abort the action (audit is required before execution). For Low/Medium-risk actions, proceed but log the audit failure at `error` level.
- **Recovery**: Automatic retry on next flush cycle.

---

## 13. Security and Permissions

### 13.1 Security Principles

1. **Principle of least privilege**: AI actions receive only the minimum permissions necessary.
2. **Explicit consent**: Every new permission requires explicit user consent.
3. **Auditability**: Every action is logged with full context.
4. **Non-repudiation**: Audit entries cannot be deleted by users or apps within the retention period.
5. **Fail-closed**: If the permission system is unavailable, permissioned actions are denied rather than allowed.

### 13.2 Permission Inheritance

- Permissions are not inherited. Each `(app_id, action_type, resource_type)` triple must be granted independently.
- Granting `FileRead` for a specific file does not grant `FileRead` for all files.
- Granting `FileRead` does not grant `FileModify`. These are separate permissions.

### 13.3 Permission Scope

- Permissions can be scoped to:
  - **Global**: Applies to all resources of the type (e.g., all files).
  - **Path prefix**: Applies to files under a specific directory (e.g., `/home/user/Documents/`).
  - **Specific resource**: Applies to a single resource (e.g., a specific file).
- In v1, only Global scope is supported. Path prefix and specific resource scoping are deferred to v2.

### 13.4 Cross-App Data Access

- An app's AI actions cannot access data belonging to another app unless:
  1. The user explicitly grants cross-app permission from Settings.
  2. The target app explicitly exposes the data through a public API.
- Cross-app permission grants are stored with both `source_app_id` and `target_app_id`.

---

## 14. Performance Requirements

| Metric | Requirement |
|---|---|
| Permission check latency (in-memory) | Less than 1ms |
| Permission check latency (with UI prompt) | User-dependent (not measured) |
| Audit entry write latency | Less than 10ms (buffered), less than 50ms (synchronous flush) |
| Audit query latency (1000 entries) | Less than 100ms |
| Permission file load time | Less than 50ms |
| Revocation effectiveness latency | Less than 100ms (takes effect within 100ms of revocation) |

---

## 15. Accessibility Requirements

- Permission prompts must announce the requesting app, action, and risk level via screen reader.
- Risk level indicators must use both color and text label (not color alone).
- Confirmation dialogs must be fully keyboard-navigable.
- The hold-to-confirm interaction for High-risk actions must have a keyboard equivalent: pressing Enter twice within 1 second.
- Audit log viewer must support keyboard navigation and screen reader row-by-row reading.
- Permission revocation list must be keyboard-navigable with focus management after revocation.

---

## 16. Observability and Logging

### 16.1 Log Events

| Event | Level | Fields |
|---|---|---|
| `ai_permission_granted` | info | app_id, action_type, resource_type, risk_level, source |
| `ai_permission_revoked` | info | app_id, action_type, resource_type |
| `ai_permission_check_passed` | debug | app_id, action_type, resource_type |
| `ai_permission_check_denied` | warn | app_id, action_type, resource_type, reason |
| `ai_action_authorized` | info | action_id, app_id, action_type, risk_level, confirmation_type |
| `ai_action_denied` | info | action_id, app_id, action_type, risk_level, denial_reason |
| `ai_audit_entry_written` | debug | entry_id, action_type, outcome |
| `ai_audit_flush_failed` | error | error_message, buffer_size |
| `ai_permission_store_error` | error | operation, error_message |
| `ai_permission_repaired` | warn | operation, backup_path |

### 16.2 Metrics

- `ai_permission_checks_total` (counter, labels: action_type, outcome)
- `ai_permission_grants_total` (counter, labels: risk_level, source)
- `ai_permission_revocations_total` (counter)
- `ai_audit_entries_total` (counter, labels: action_type, risk_level, outcome)
- `ai_audit_flush_duration_ms` (histogram)
- `ai_active_permissions` (gauge, labels: risk_level)

---

## 17. Testing Requirements

### 17.1 Unit Tests

- Risk level classification: every `AiActionType` maps to the correct `RiskLevel`.
- Action schema validation: valid actions pass, invalid actions (missing required fields, unknown types) are rejected.
- Permission grant/revoke lifecycle: grant, check, revoke, check denied.
- Permission store: load, save, find, upsert, remove operations.
- Audit store: append, read range, rotation.
- Audit buffer: flush on timer, flush on buffer full, flush on shutdown.
- Confirmation type determination: each risk level maps to the correct confirmation type.

### 17.2 Integration Tests

- Full authorization flow: initiate action -> permission check -> confirmation -> audit -> execute.
- Permission prompt: denied by user -> action does not execute, audit entry records denial.
- Revocation: grant permission, execute action (succeeds), revoke, execute again (denied).
- Audit rotation: create entries across multiple days, rotate with 1-day retention, verify old entries deleted.
- Concurrent access: two threads checking permissions simultaneously, one thread revoking while another checks.
- Crash recovery: kill process during audit buffer flush, restart, verify no High-risk actions executed without audit.

### 17.3 Security Tests

- Anti-pattern verification: attempt each forbidden action (see Section 20.2), verify it is blocked.
- Permission bypass attempts: app tries to execute High-risk action without permission, verify denial.
- Audit integrity: verify audit entries cannot be modified after writing.
- Permission grant tampering: modify stored grant records or integrity metadata externally, verify the system detects and rejects malformed state.
- Cross-app access: app A tries to access app B's data without cross-app permission, verify denial.

### 17.4 Performance Tests

- 1000 permission checks in sequence: verify all complete within 1 second total.
- Write 1000 audit entries: verify all persisted and queryable.
- Grant 100 permissions, list them: verify query completes within 50ms.

---

## 18. Acceptance Criteria

- [ ] Every AI action type has a defined risk level and confirmation requirement.
- [ ] Low-risk actions (read-only) execute without confirmation.
- [ ] Medium-risk actions (create/append) show brief confirmation dialog.
- [ ] High-risk actions (overwrite/delete/modify) show detailed confirmation with preview and hold-to-confirm.
- [ ] Permission prompts appear for first-time actions and can be remembered with "Always allow".
- [ ] Users can revoke permissions from Settings > AI > Permissions.
- [ ] Revocation takes effect within 100ms without restart.
- [ ] Every AI action produces an audit entry with all required fields.
- [ ] Audit entries include: timestamp, user_id, app_id, action_type, target_resource, risk_level, outcome, provider, model.
- [ ] High-risk actions are logged to audit before execution (write-ahead).
- [ ] Audit entries are append-only and cannot be modified.
- [ ] Audit files rotate after the configured retention period (default: 90 days).
- [ ] AI cannot bypass the permission system.
- [ ] AI cannot execute arbitrary code.
- [ ] AI cannot access files without going through cortex-files.
- [ ] AI cannot modify system settings without user confirmation.
- [ ] AI actions are always visible in the audit log.
- [ ] AI cannot self-elevate permissions.
- [ ] AI cannot access other apps' data without explicit cross-app permission.
- [ ] Permission store failure causes fail-closed behavior (actions denied).
- [ ] Audit store failure shows warning but allows Low/Medium-risk actions to proceed.
- [ ] Audit store failure blocks High-risk actions until logging is restored.
- [ ] All permission and confirmation dialogs are keyboard-navigable.
- [ ] Risk level indicators use both color and text labels.

---

## 19. Build Order and Dependencies
**Layer 13**. Depends on: 04 (permissions), 06 (AI runtime), 14 (observability)

### 19.1 Crate Dependencies

```text
AI action guard depends on:
  - cortex-core (shared types)
  - cortex-db (persistent storage)
  - cortex-files (file access abstraction for AI actions)
  - cortex-settings (settings infrastructure for revocation UI)
  - cortex-observability (audit persistence)
  - @cortexos/ui-components (dialogs rendered by UI layers)
  - serde / serde_json
  - chrono
  - uuid
  - tokio
  - tracing
```

### 19.2 Build Order

1. `cortex-core` (shared types)
2. `cortex-db` (persistent storage)
3. `cortex-files` (filesystem abstraction)
4. `cortex-settings` (settings infrastructure)
5. `cortex-observability` (audit persistence)
6. `cortex-policy` (this subsystem owner)

Note: the AI-action guard must be available before spec 19 integration, because the surface layer calls the permission guard for every action.

---

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Fine-grained resource-level permissions in v1 (only global scope is supported).
- Real-time permission monitoring dashboard.
- Permission templates or presets.
- AI-driven permission recommendations.
- Role-based access control (RBAC) for AI actions.
- Permission synchronization across devices.
- Encrypted audit log at rest in v1.

### 20.2 Anti-Patterns (MUST BE PREVENTED)

The following patterns are explicitly forbidden and must be technically prevented:

1. **AI cannot bypass the permission system**: There is no code path that allows an AI action to execute without passing through the `PermissionGuard`. The `PermissionGuard::authorize_action` call is mandatory in every code path that leads to action execution.

2. **AI cannot execute arbitrary code**: The `AiActionType` enum is closed (not extensible by apps). There is no "custom code execution" action type. Apps cannot register arbitrary executable actions as AI actions.

3. **AI cannot access files without going through cortex-files**: All file operations in AI actions use `cortex-files` API. Direct host-filesystem access from the AI layer is prohibited by architecture.

4. **AI cannot modify system settings without user confirmation**: `AppStateModify` is classified as High-risk. It always requires detailed confirmation. There is no setting or configuration that can downgrade this risk level.

5. **AI actions are never silent**: The audit logger is called on every action, regardless of outcome (success, denied, failed). The audit call is in the same function as the action execution and cannot be bypassed by conditional logic.

6. **AI cannot self-elevate permissions**: Permission grants can only be created through:
   - User interaction with the permission prompt dialog.
   - User interaction with the Settings UI.
   There is no API that allows an app or AI component to grant itself permissions. The `PermissionGuard::grant_permission` method requires a `GrantSource` that is verified to be `UserPrompt` or `Settings`.

7. **AI cannot access other apps' data without explicit cross-app permission**: The permission check includes `app_id` in its key. A permission granted to `app_id=A` does not allow access to data owned by `app_id=B`. Cross-app access requires a separate permission grant with both `source_app_id` and `target_app_id`.

8. **Risk levels cannot be overridden by apps or configuration**: The `classify_risk` function returns a static mapping. There is no configuration key, API parameter, or app-level override that can change an action's risk level.

9. **Confirmation dialogs cannot be skipped by apps**: The confirmation flow is controlled by the `PermissionGuard`, not by the calling app. Apps cannot pass a "skip_confirmation" flag. (Exception: the global `ai.auto_apply_low_risk_actions` setting, which only affects explicitly eligible Low-risk inline actions after permission checks have already passed.)

10. **Audit entries cannot be deleted by users or apps within the retention period**: The audit store provides no delete API. The only deletion mechanism is the rotation task, which is triggered by age, not by user or app action.

---

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 File Structure

```
crates/cortex-policy/src/ai_actions/
    lib.rs
    guard.rs              # PermissionGuard implementation
    validator.rs          # ActionSchemaValidator implementation
    classifier.rs         # RiskClassifier: static mapping of action types to risk levels
    confirmation.rs       # Confirmation flow logic (brief and detailed)
    store/
      mod.rs
      permission_store.rs # PermissionStore: load/save/find/upsert/remove in SQLite
      audit_store.rs      # AuditStore: append/read/rotate via cortex-observability
    models/
      mod.rs
      action.rs           # AiAction, AiActionType, TargetResource, RiskLevel, Initiator
      permission.rs       # PermissionGrant, ResourceType, GrantSource
      audit.rs            # AuditEntry, AuditOutcome, AuditFilter, AuditStats
    dialog/
      mod.rs
      permission_prompt.rs    # Permission prompt dialog UI logic
      brief_confirmation.rs   # Brief confirmation dialog UI logic
      detailed_confirmation.rs # Detailed confirmation with preview and hold
    settings.rs           # Permission-related settings keys
    error.rs              # Error types for permission operations
    anti_pattern_guard.rs # Compile-time and runtime checks for anti-patterns
```

### 21.2 Implementation Order

1. **Phase 1 - Models** (`models/action.rs`, `models/permission.rs`, `models/audit.rs`):
   - Define all data types from Section 8.
   - Implement `Serialize`/`Deserialize` for all types.
   - Write unit tests for type construction and validation.

2. **Phase 2 - Classifier** (`classifier.rs`):
   - Implement the static risk level mapping.
   - Write exhaustive tests: every `AiActionType` variant maps to the correct `RiskLevel`.
   - The mapping must be implemented as a `match` statement that the compiler enforces as exhaustive.

3. **Phase 3 - Validator** (`validator.rs`):
   - Implement `ActionSchemaValidator`.
   - Validate: action_type is known, target_resource is compatible with action_type, parameters are present for the action type.
   - Test: valid actions pass, every invalid combination is caught.

4. **Phase 4 - Stores** (`store/permission_store.rs`, `store/audit_store.rs`):
   - Implement `PermissionStore` with transactional SQLite writes.
   - Implement `AuditStore` with append-only inserts and retention sweeps.
   - Test: load, save, find, upsert, remove, append, read_range, rotate.
   - Test crash recovery: write partial data, verify load handles it.

5. **Phase 5 - Guard** (`guard.rs`):
   - Implement `PermissionGuard` using the store, classifier, and validator.
   - The authorization flow must be a single method that orchestrates all checks.
   - Integration tests: full flow for Low, Medium, High-risk actions.
   - Test denial paths: permission denied, confirmation cancelled.

6. **Phase 6 - Dialogs** (`dialog/`):
   - Implement permission prompt, brief confirmation, detailed confirmation.
   - Detailed confirmation must include hold-to-confirm with 1-second timer.
   - Keyboard accessibility: Tab navigation, Enter to confirm, Escape to cancel.
   - Test dialog display and result handling.

7. **Phase 7 - Anti-Pattern Guard** (`anti_pattern_guard.rs`):
   - Implement runtime checks that verify anti-patterns are not violated:
     - No direct host-filesystem mutation path for permission or audit persistence.
     - No "skip_confirmation" parameter in any public API.
     - No delete method on audit store (except rotation).
     - Permission grant source is verified.
   - Document these checks in code comments.

8. **Phase 8 - Settings Integration** (`settings.rs`):
   - Define settings keys: `ai.audit_retention_days`, permission UI preferences.
   - Integrate with `cortex-settings` for loading/saving.

### 21.3 Key Implementation Notes

- Use `RwLock` for `granted_permissions` to allow concurrent reads during permission checks.
- Use `Mutex` for `audit_buffer` to prevent interleaved writes.
- The audit flush task runs as a `tokio::spawn` background task with a 1-second interval timer.
- For High-risk actions, call `audit_store.append` synchronously (with `block_on` if necessary) before executing the action. This ensures the audit entry exists even if the process crashes during execution.
- Permission and audit writes must be committed transactionally through the owning storage layer.
- The `classify_risk` function must use a Rust `match` statement that covers every `AiActionType` variant. Adding a new variant must cause a compile error until the risk level is assigned.
- All error types must implement `std::error::Error` and `std::fmt::Display` with human-readable messages.
- Use `tracing::instrument` on all public API methods for observability.

### 21.4 Configuration Defaults

```rust
const DEFAULT_AUDIT_RETENTION_DAYS: u32 = 90;
const AUDIT_BUFFER_FLUSH_INTERVAL_SECS: u64 = 1;
const AUDIT_BUFFER_MAX_SIZE: usize = 10;
const HIGH_RISK_HOLD_DURATION_MS: u64 = 1000;
const AI_PERMISSION_SCHEMA_VERSION: u32 = 1;
```

### 21.5 Critical Safety Checklist

Before marking this crate as complete, verify:

- [ ] Every public API method that could execute an AI action calls `PermissionGuard::authorize_action`.
- [ ] No code path exists that bypasses `PermissionGuard`.
- [ ] The `AiActionType` enum is non-exhaustive to apps (only CortexOS can add variants).
- [ ] No direct host-filesystem code path exists for AI permission or audit persistence.
- [ ] The `grant_permission` method verifies the `GrantSource` is user-initiated.
- [ ] The audit store has no delete method accessible to apps.
- [ ] The risk classifier uses an exhaustive match with no wildcard arm.
- [ ] High-risk actions flush audit synchronously before execution.
- [ ] Permission and audit persistence are transactional.
- [ ] All anti-patterns from Section 20.2 have corresponding test cases that verify they are blocked.
