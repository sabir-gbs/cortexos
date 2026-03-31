# Spec 04: Permissions, Policy, and Trust Model

**Status:** Implementation-grade
**Owner crate:** `cortex-policy`
**Depends on:** `cortex-core`, `cortex-db`, `cortex-auth`, `cortex-settings`
**Required by:** `cortex-api`, `cortex-runtime`, `cortex-ai`, `cortex-files`, `cortex-notify`, all first-party apps, all third-party apps

---

## 1. Purpose

Define the permission model, policy enforcement engine, and trust boundaries for CortexOS. This subsystem is the sole authority for access control decisions across the entire operating system. Every privileged operation -- file access, network requests, clipboard reads, AI dispatch, notification delivery, system introspection, and device interaction -- must be authorized by `cortex-policy` before execution.

The trust model is built on a single principle: the client is never trusted for authorization. All permission checks execute server-side. The browser frontend is an untrusted rendering surface. Apps (both first-party and third-party) operate within permission boundaries enforced by the server. No app may escalate its own permissions, silently degrade security, or bypass a denied operation.

This is a HIGH-RISK spec. Errors in permission enforcement lead to data exfiltration, unauthorized file access, or uncontrolled AI requests. Every section of this document must be implemented exactly as written.

---

## 2. Scope

**In scope for this spec:**

- Permission categories: FileSystem, Clipboard, Network, Notification, AI, System, Device
- Resource/action model: every permission is a structured pair of resource URI and action verb
- App grants: apps request permissions via their manifest, the user approves at install time or at first use
- AI grants: specific permissions governing what the AI runtime may do on the user's behalf
- File access grants: path-based with wildcard support
- Clipboard grants: read/write with mandatory user prompt
- Grant types: one-time (single-use, consumed after the operation) and persistent (survives across sessions)
- Revocation semantics: immediate effect upon revocation, app is notified
- Deny behavior: operation fails with `PermissionDenied` error, no silent fallback, no partial success
- Audit requirements: every permission check is logged to the observability subsystem
- Server-side enforcement: all checks run on the server; the client is never trusted for authorization
- Policy storage: grant records persisted in `cortex-db`
- Permission prompt UI contract: the interface the policy engine uses to request user approval

**Owned by this subsystem:**

- Permission schema definition and validation
- Grant storage and retrieval
- Permission check evaluation engine
- Revocation engine
- Audit event emission for all permission decisions
- Permission prompt dispatch to the frontend
- Default policy configuration

---

## 3. Out of Scope

- **Authentication and identity:** Owned by `cortex-auth` (spec 03). `cortex-policy` receives a verified user ID; it does not verify identity.
- **Settings storage:** Owned by `cortex-settings` (spec 05). Policy-related settings (e.g., default permission behavior) are stored there.
- **App manifest parsing:** Owned by `cortex-runtime` (spec 09). The runtime extracts requested permissions from the manifest and registers them with `cortex-policy`.
- **Filesystem implementation:** Owned by `cortex-files` (spec 11). `cortex-policy` authorizes file operations but does not implement them.
- **Network request execution:** Owned by the network subsystem. `cortex-policy` authorizes outbound requests but does not make them.
- **AI request dispatch:** Owned by `cortex-ai` (spec 06). `cortex-policy` authorizes AI operations but does not execute them.
- **Notification delivery:** Owned by `cortex-notify` (spec 13). `cortex-policy` authorizes notification posting but does not deliver them.
- **UI rendering of permission prompts:** Owned by `apps/desktop-shell` (spec 07). The shell renders the prompt UI based on data provided by `cortex-policy`.

---

## 4. Objectives

1. Provide a unified permission model that covers all privileged operations in CortexOS: file access, clipboard, network, notifications, AI, system, and device.
2. Enforce all permission checks server-side. The client is an untrusted rendering surface and must never be the authority for any access control decision.
3. Support two grant types: one-time grants (consumed after a single operation) and persistent grants (survive across sessions until explicitly revoked).
4. Require explicit user approval for every permission grant. No permission is granted silently, by default, or by convention.
5. Support immediate revocation with synchronous notification to the affected app.
6. Log every permission check (grant, deny, prompt) to the audit subsystem for observability and forensics.
7. Provide path-based file access control with wildcard support for flexible scoping.
8. Govern AI operations with dedicated permissions that reflect the unique trust requirements of AI actions (file reads, file writes, clipboard access, command execution).
9. Ensure that permission denial is always explicit and never results in silent fallback or degraded behavior that the user did not consent to.

---

## 5. User-Visible Behavior

### 5.1 Permission Prompts at Install Time

When a user installs an app (first-party or third-party), the installer screen displays a list of requested permissions extracted from the app manifest. Each permission is shown with:

- A human-readable description (e.g., "Read files in your Documents folder" for `file://documents/read`)
- The permission category icon (file, clipboard, network, notification, AI, system, device)
- A toggle or checkbox for each permission, defaulting to OFF

The user must explicitly approve each permission. The app cannot be installed with zero approved permissions if the manifest declares required permissions. If the user declines a required permission, the installation is blocked with a message explaining that the permission is required.

Optional permissions (marked as `optional` in the manifest) can be declined without blocking installation.

### 5.2 Permission Prompts at First Use

When an app attempts an operation for which it has no grant (neither persistent nor one-time), the system displays a permission prompt dialog:

- Dialog title: "{App Name} wants to {action description}"
- Dialog body: "{App Name} is requesting permission to {detailed description}."
- Buttons: "Allow Once", "Always Allow", "Deny"
- The dialog is modal and blocks the operation until the user responds
- The dialog cannot be dismissed without a choice (no close button, no click-outside-to-dismiss)

If the user selects "Allow Once", the operation proceeds, the one-time grant is consumed, and the next identical operation will prompt again.

If the user selects "Always Allow", a persistent grant is created and stored. Future operations matching the same permission will be automatically authorized without prompting.

If the user selects "Deny", the operation fails immediately with a `PermissionDenied` error returned to the app. The app receives the error and must handle it. No retry is automatic.

### 5.3 Permission Management in Settings

The Settings app provides a "Privacy & Permissions" section (linking to `apps/settings-app` under the `privacy.*` namespace). This section displays:

- A list of all installed apps
- For each app, a list of granted permissions with their status (persistent, one-time history count, last used timestamp)
- A "Revoke" button next to each persistent grant
- A "Revoke All" button to remove all grants for an app
- A toggle to block all future permission prompts for an app (effectively auto-deny)

When the user revokes a permission:
- The grant is immediately removed from the policy store
- The affected app receives a `PermissionsRevoked` notification via the command bus
- The app must immediately stop performing the revoked operation
- Any in-progress operation covered by the revoked permission is cancelled and returns `PermissionDenied`

### 5.4 AI Permission Prompts

AI permissions are prompted separately and carry additional disclosure:

- `ai://file/read`: "Allow {App Name} to read file contents through AI features?"
- `ai://file/write`: "Allow {App Name} to write or modify files through AI features?"
- `ai://clipboard/read`: "Allow {App Name} to read your clipboard through AI features?"
- `ai://command/execute`: "Allow {App Name} to execute system commands through AI features?"

These prompts always display a warning badge and additional text: "AI features may send data to external providers. Review your AI privacy settings before granting."

The `ai://command/execute` permission is always one-time. There is no "Always Allow" option for command execution. Every AI-initiated command execution requires explicit user approval.

### 5.5 Clipboard Access Prompts

Clipboard permissions (`clipboard://system/read` and `clipboard://system/write`) always prompt the user at first use, even if the user previously granted a persistent permission. The persistent grant eliminates the prompt, but the settings UI shows a warning that clipboard access is active.

When an app reads the clipboard with a valid grant, the system displays a transient notification: "{App Name} read your clipboard." This notification is not dismissible by the app; it is a system-level observability feature.

---

## 6. System Behavior

### 6.1 Permission Check Flow

Every privileged operation in CortexOS follows this flow:

```
1. App requests a privileged operation via the command bus or API
2. The service handling the operation calls cortex-policy::check(permission, app_id, user_id)
3. cortex-policy evaluates:
   a. Is there a persistent grant matching (app_id, permission)?
      -> YES: log GRANT, return Allow
   b. Is there a one-time grant matching (app_id, permission)?
      -> YES: consume the grant, log GRANT, return Allow
   c. Neither grant exists:
      -> Dispatch a permission prompt to the frontend via the prompt channel
      -> Wait for user response (blocking, with a 120-second timeout)
      -> If user approves "Allow Once": create one-time grant, consume it, log GRANT, return Allow
      -> If user approves "Always Allow": create persistent grant, log GRANT, return Allow
      -> If user denies: log DENY, return Deny(PermissionDenied)
      -> If timeout: log DENY_TIMEOUT, return Deny(PermissionDenied)
4. If Allow: the service proceeds with the operation
5. If Deny: the service returns PermissionDenied to the app
```

### 6.2 Permission Check for AI Operations

AI operations add a second gate. The check sequence is:

```
1. cortex-ai receives a request from app_id with feature tag
2. cortex-ai calls cortex-policy::check("ai://completion/request", app_id, user_id)
   -> If denied: return PermissionDenied immediately
3. If the AI request involves file context:
   cortex-ai calls cortex-policy::check("ai://file/read", app_id, user_id)
   -> If denied: the AI request proceeds without file context (not blocked entirely)
4. If the AI request involves writing files:
   cortex-ai calls cortex-policy::check("ai://file/write", app_id, user_id)
   -> If denied: the AI request is rejected with PermissionDenied for the write portion
5. If the AI request involves clipboard context:
   cortex-ai calls cortex-policy::check("ai://clipboard/read", app_id, user_id)
   -> If denied: the AI request proceeds without clipboard context (not blocked entirely)
6. If the AI request involves command execution:
   cortex-ai calls cortex-policy::check("ai://command/execute", app_id, user_id)
   -> This permission is ALWAYS one-time, no persistent option
   -> If denied: the command is not executed, the AI response notes the denial
```

### 6.3 Grant Storage and Resolution

Grants are stored in `cortex-db` with the following resolution rules:

1. **Exact match first:** If a grant exists for the exact (app_id, permission) pair, use it.
2. **Wildcard match second:** For file permissions, check wildcard grants. A grant for `file://documents/**` matches `file://documents/read` and `file://documents/write` for any path under `documents/`.
3. **No inheritance:** A grant for `file://documents/read` does not imply `file://documents/write`. Each action requires its own grant.
4. **Category-level grants are not supported:** A grant for `file://*/read` (all files, read) is not valid. Grants must specify at minimum a top-level directory.

### 6.4 Revocation Flow

```
1. User revokes permission for (app_id, permission) in Settings
2. cortex-policy deletes the grant record from cortex-db
3. cortex-policy emits a PermissionsRevoked event on the command bus:
   { app_id, permissions: [permission], revoked_at: timestamp }
4. The event is delivered synchronously to the app's message queue
5. Any in-progress operation that depends on the revoked permission is cancelled:
   - The operation's check result is re-evaluated on next access
   - Since the grant is removed, the re-evaluation returns Deny
6. The app must handle the PermissionDenied error
```

### 6.5 Prompt Timeout

Permission prompts have a 120-second timeout. If the user does not respond within 120 seconds, the prompt is dismissed and the permission check returns `Deny(PermissionDenied)`. The app receives the denial. The timeout is logged as `DENY_TIMEOUT`.

---

## 7. Architecture

### 7.1 Crate Layout

```
cortex-policy/
  src/
    lib.rs              -- public re-exports
    permission.rs       -- Permission struct, PermissionUri, ActionType
    categories.rs       -- FileSystem, Clipboard, Network, Notification, AI, System, Device
    grant.rs            -- Grant record, GrantType (OneTime, Persistent)
    engine.rs           -- check(), evaluate(), prompt dispatch
    revocation.rs       -- revoke(), revoke_all_for_app(), notification dispatch
    audit.rs            -- audit event emission for every permission decision
    prompt.rs           -- PermissionPrompt struct, prompt channel interface
    store.rs            -- PolicyStore trait for grant persistence
    error.rs            -- PolicyError enum
    validation.rs       -- permission URI validation, wildcard matching
    defaults.rs         -- default policy configuration
```

### 7.2 Dependency Direction

```
cortex-core (types, error macros)
    |
cortex-db (storage interface)
    |
cortex-policy
    |
    +---> cortex-auth (user identity resolution, read-only)
    +---> cortex-settings (reads default permission behavior settings)
    +---> cortex-observability (audit event emission)
```

`cortex-policy` depends on `cortex-core`, `cortex-db`, `cortex-auth`, `cortex-settings`. It does not depend on `cortex-ai`, `cortex-files`, `cortex-runtime`, or any app crate.

### 7.3 Prompt Channel

The prompt channel is an asynchronous communication channel between the server-side policy engine and the frontend permission prompt UI:

- The engine sends a `PermissionPromptRequest` through the channel
- The frontend (desktop shell) receives the request and renders the prompt dialog
- The frontend sends a `PermissionPromptResponse` back through the channel
- The channel is implemented as a tokio MPSC (multi-producer, single-consumer) pair per user session
- The engine awaits the response with a 120-second timeout

---

## 8. Data Model

### 8.1 Permission

```rust
/// A structured permission identifier.
///
/// Format: "{category}://{resource}/{action}"
/// Examples:
///   "file://documents/read"
///   "clipboard://system/write"
///   "ai://completion/request"
///   "network://external/connect"
///   "notification://system/post"
///   "system://runtime/info"
///   "device://camera/access"
struct Permission {
    /// The permission category.
    category: PermissionCategory,

    /// The resource being accessed. For file permissions, this is a path prefix.
    /// For other categories, this identifies the specific resource (e.g., "system" for clipboard).
    resource: String,

    /// The action being performed on the resource.
    action: Action,
}

enum PermissionCategory {
    FileSystem,
    Clipboard,
    Network,
    Notification,
    AI,
    System,
    Device,
}

enum Action {
    Read,
    Write,
    Execute,
    Access,
    Connect,
    Post,
    Request,
    Delete,
}
```

### 8.2 Permission URI Syntax

Every permission is represented as a URI string with the following grammar:

```
permission_uri := category "://" resource "/" action

category       := "file" | "clipboard" | "network" | "notification" | "ai" | "system" | "device"
resource       := path_segment ( "/" path_segment )*
action         := "read" | "write" | "execute" | "access" | "connect" | "post" | "request" | "delete"
path_segment   := (alphanumeric | "-" | "_" | ".")+
wildcard       := "**"  (matches zero or more path segments)
```

Examples:

| Permission URI | Category | Resource | Action |
|----------------|----------|----------|--------|
| `file://documents/read` | FileSystem | documents | Read |
| `file://documents/**/read` | FileSystem | documents (with wildcard) | Read |
| `file://pictures/write` | FileSystem | pictures | Write |
| `clipboard://system/read` | Clipboard | system | Read |
| `clipboard://system/write` | Clipboard | system | Write |
| `network://external/connect` | Network | external | Connect |
| `notification://system/post` | Notification | system | Post |
| `ai://completion/request` | AI | completion | Request |
| `ai://file/read` | AI | file | Read |
| `ai://file/write` | AI | file | Write |
| `ai://clipboard/read` | AI | clipboard | Read |
| `ai://command/execute` | AI | command | Execute |
| `system://runtime/info` | System | runtime | Access |
| `device://camera/access` | Device | camera | Access |

### 8.3 Grant Record

```rust
/// A permission grant associating an app with a permitted action.
struct Grant {
    /// Unique grant identifier (UUIDv7).
    id: GrantId,

    /// The app this grant belongs to.
    app_id: AppId,

    /// The user who authorized this grant.
    user_id: UserId,

    /// The permission that was granted.
    permission: Permission,

    /// The type of grant.
    grant_type: GrantType,

    /// Whether this grant has been consumed (only relevant for OneTime grants).
    consumed: bool,

    /// When the grant was created.
    created_at: DateTime<Utc>,

    /// When the grant was last used (updated on each successful check).
    last_used_at: Option<DateTime<Utc>,

    /// How many times this grant has been used.
    use_count: u64,
}

enum GrantType {
    /// Consumed after a single operation. The grant record is marked as consumed
    /// and will not match future permission checks.
    OneTime,

    /// Persists across sessions until explicitly revoked.
    Persistent,
}
```

### 8.4 Permission Prompt Request

```rust
/// Request sent to the frontend to display a permission prompt.
struct PermissionPromptRequest {
    /// Unique request ID for correlation.
    id: Uuid,

    /// The app requesting the permission.
    app_id: AppId,

    /// Human-readable app name (for display).
    app_name: String,

    /// The permission being requested.
    permission: Permission,

    /// Human-readable description of the permission.
    description: String,

    /// Whether "Always Allow" is available for this permission.
    /// Always false for ai://command/execute.
    always_allow_available: bool,

    /// Timestamp when the request was created (for timeout tracking).
    created_at: DateTime<Utc>,
}

/// Response from the frontend after user interaction.
struct PermissionPromptResponse {
    /// The request ID this response corresponds to.
    request_id: Uuid,

    /// The user's decision.
    decision: PromptDecision,
}

enum PromptDecision {
    /// User approved for this one operation.
    AllowOnce,

    /// User approved for all future operations (creates persistent grant).
    AlwaysAllow,

    /// User denied the request.
    Deny,
}
```

### 8.5 Invariants

1. Every privileged operation must call `cortex-policy::check()` before execution. There are no exceptions for first-party apps.
2. A `OneTime` grant is consumed atomically: the check and consumption happen in a single transaction.
3. A `Persistent` grant survives server restarts because it is stored in `cortex-db`.
4. Revocation is immediate: once the grant record is deleted from the store, the next permission check for that grant will fail.
5. `ai://command/execute` is always one-time. Persistent grants for this permission are never created.
6. Permission URIs are validated at grant creation time. Invalid URIs are rejected with a validation error.
7. A grant's `app_id` must correspond to an installed app. Grants for uninstalled apps are invalid and are cleaned up during app uninstallation.
8. A grant's `user_id` must match the current authenticated user. Cross-user grants are not supported in the single-user model.
9. Wildcard grants for file permissions match only within the specified path hierarchy. `file://documents/**/read` matches `file://documents/read` and `file://documents/subfolder/read` but not `file://pictures/read`.

---

## 9. Public Interfaces

### 9.1 HTTP API Endpoints

| Method | Path | Auth Required | Description |
|--------|------|---------------|-------------|
| GET | `/api/v1/policy/grants` | Yes | List all grants for the current user |
| GET | `/api/v1/policy/grants/{app_id}` | Yes | List grants for a specific app |
| POST | `/api/v1/policy/grants` | Yes | Create a grant (user approval) |
| DELETE | `/api/v1/policy/grants/{grant_id}` | Yes | Revoke a specific grant |
| DELETE | `/api/v1/policy/grants/app/{app_id}` | Yes | Revoke all grants for an app |
| GET | `/api/v1/policy/check` | Yes | Check if a permission is granted (returns allow/deny without prompting) |
| POST | `/api/v1/policy/prompt/respond` | Yes | Respond to a pending permission prompt |

### 9.2 Request/Response Types

```typescript
// GET /api/v1/policy/grants
interface GrantListResponse {
  grants: GrantRecord[];
}

interface GrantRecord {
  id: string;
  app_id: string;
  app_name: string;
  permission: string;        // e.g., "file://documents/read"
  grant_type: "one_time" | "persistent";
  consumed: boolean;
  created_at: string;        // ISO 8601
  last_used_at: string | null;
  use_count: number;
}

// POST /api/v1/policy/grants
interface CreateGrantRequest {
  app_id: string;
  permission: string;
  grant_type: "one_time" | "persistent";
}

// DELETE /api/v1/policy/grants/{grant_id}
// No request body. Returns 204 on success.

// DELETE /api/v1/policy/grants/app/{app_id}
// No request body. Returns 204 on success.

// GET /api/v1/policy/check?app_id={app_id}&permission={permission}
interface PermissionCheckResponse {
  allowed: boolean;
  grant_type: "one_time" | "persistent" | null;  // null if denied
  grant_id: string | null;                        // null if denied
}

// POST /api/v1/policy/prompt/respond
interface PromptResponseRequest {
  request_id: string;
  decision: "allow_once" | "always_allow" | "deny";
}
```

### 9.3 Rust Public API

```rust
/// The primary interface for permission checking.
/// Used by all service crates (cortex-ai, cortex-files, cortex-notify, etc.)
pub trait PolicyEngine: Send + Sync {
    /// Check whether an app has permission to perform an action.
    /// This is a synchronous check that does NOT prompt the user.
    /// Returns Allow if a matching grant exists, Deny otherwise.
    fn check(&self, permission: &Permission, app_id: &AppId, user_id: &UserId)
        -> PermissionDecision;

    /// Check and prompt if necessary.
    /// If no grant exists, dispatches a prompt to the user and awaits a response.
    /// Returns the final decision after user interaction or timeout.
    fn check_or_prompt(&self, permission: &Permission, app_id: &AppId, user_id: &UserId)
        -> PermissionDecision;

    /// Create a grant explicitly (used when user approves via API).
    fn create_grant(&self, grant: NewGrant) -> Result<Grant, PolicyError>;

    /// Revoke a specific grant.
    fn revoke_grant(&self, grant_id: &GrantId, user_id: &UserId) -> Result<(), PolicyError>;

    /// Revoke all grants for an app.
    fn revoke_all_for_app(&self, app_id: &AppId, user_id: &UserId) -> Result<usize, PolicyError>;

    /// List all grants for a user, optionally filtered by app.
    fn list_grants(&self, user_id: &UserId, app_id: Option<&AppId>)
        -> Result<Vec<Grant>, PolicyError>;

    /// Check whether an app is blocked from prompting (auto-deny).
    fn is_prompt_blocked(&self, app_id: &AppId, user_id: &UserId) -> bool;
}

enum PermissionDecision {
    Allow { grant_id: GrantId, grant_type: GrantType },
    Deny { reason: DenyReason },
}

enum DenyReason {
    NoGrant,
    GrantConsumed,
    GrantRevoked,
    PromptDenied,
    PromptTimeout,
    PromptBlocked,
}

struct NewGrant {
    app_id: AppId,
    user_id: UserId,
    permission: Permission,
    grant_type: GrantType,
}
```

---

## 10. Internal Interfaces

### 10.1 Policy Store Trait

```rust
/// Internal storage abstraction for grant records. Implemented by cortex-db backends.
pub trait PolicyStore: Send + Sync {
    /// Insert a new grant record.
    fn insert_grant(&self, grant: Grant) -> Result<Grant, PolicyError>;

    /// Get a grant by ID.
    fn get_grant(&self, grant_id: &GrantId) -> Result<Option<Grant>, PolicyError>;

    /// Find a matching grant for (app_id, user_id, permission).
    /// For file permissions, performs wildcard matching.
    /// Returns the most specific matching grant, or None.
    fn find_grant(&self, app_id: &AppId, user_id: &UserId, permission: &Permission)
        -> Result<Option<Grant>, PolicyError>;

    /// Mark a one-time grant as consumed.
    fn consume_grant(&self, grant_id: &GrantId) -> Result<(), PolicyError>;

    /// Delete a specific grant.
    fn delete_grant(&self, grant_id: &GrantId) -> Result<(), PolicyError>;

    /// Delete all grants for an app.
    fn delete_all_for_app(&self, app_id: &AppId, user_id: &UserId)
        -> Result<usize, PolicyError>;

    /// List all grants for a user, optionally filtered by app.
    fn list_grants(&self, user_id: &UserId, app_id: Option<&AppId>)
        -> Result<Vec<Grant>, PolicyError>;

    /// Delete all grants for an app (used during app uninstallation).
    fn delete_all_grants_for_app(&self, app_id: &AppId) -> Result<usize, PolicyError>;

    /// List all apps that have prompt blocking enabled.
    fn list_blocked_apps(&self, user_id: &UserId) -> Result<Vec<AppId>, PolicyError>;

    /// Check if an app is prompt-blocked.
    fn is_prompt_blocked(&self, app_id: &AppId, user_id: &UserId) -> Result<bool, PolicyError>;

    /// Set or unset prompt blocking for an app.
    fn set_prompt_blocked(&self, app_id: &AppId, user_id: &UserId, blocked: bool)
        -> Result<(), PolicyError>;
}
```

### 10.2 Prompt Channel

```rust
/// Internal interface for dispatching permission prompts to the frontend.
pub trait PromptChannel: Send + Sync {
    /// Send a permission prompt request and await the response.
    /// Blocks for up to 120 seconds.
    async fn dispatch_prompt(
        &self,
        request: PermissionPromptRequest,
    ) -> Result<PromptDecision, PolicyError>;
}
```

### 10.3 Audit Emitter

```rust
/// Internal interface for emitting permission audit events.
pub trait PermissionAuditEmitter: Send + Sync {
    /// Emit an audit event for a permission decision.
    fn emit(&self, event: PermissionAuditEvent);
}

struct PermissionAuditEvent {
    timestamp: DateTime<Utc>,
    user_id: UserId,
    app_id: AppId,
    permission: Permission,
    decision: AuditDecision,
    grant_id: Option<GrantId>,
    prompt_shown: bool,
    response_time_ms: Option<u64>,
}

enum AuditDecision {
    Granted,
    Denied,
    Consumed,
    Revoked,
    PromptTimeout,
}
```

### 10.4 Wildcard Matcher

```rust
/// Internal utility for matching permission URIs with wildcards.
pub struct WildcardMatcher;

impl WildcardMatcher {
    /// Check if a granted permission pattern matches a requested permission.
    /// Wildcard "**" matches zero or more path segments in the resource.
    /// Actions must match exactly.
    ///
    /// Examples:
    ///   grant "file://documents/**/read" matches "file://documents/read" -> true
    ///   grant "file://documents/**/read" matches "file://documents/sub/file/read" -> true
    ///   grant "file://documents/**/read" matches "file://documents/write" -> false (action mismatch)
    ///   grant "file://documents/**/read" matches "file://pictures/read" -> false (resource mismatch)
    pub fn matches(grant_permission: &Permission, request_permission: &Permission) -> bool;
}
```

---

## 11. State Management

### 11.1 State Location

| State | Storage | Persistence |
|-------|---------|-------------|
| Grant records | cortex-db (SQLite) | Persistent |
| Prompt block list | cortex-db (SQLite) | Persistent |
| Pending prompts | In-memory (per-session) | Volatile (lost on server restart, which cancels pending prompts with denial) |
| Permission check cache | None | No caching. Every check hits the store. |
| Audit events | cortex-observability | Persistent (via observability subsystem) |

### 11.2 Default Configuration

| Setting | Default | Description |
|---------|---------|-------------|
| `privacy.auto_deny_unknown_apps` | `false` | When true, permission prompts for apps not in the grant store are auto-denied without prompting |
| `privacy.prompt_timeout_seconds` | `120` | Seconds before a pending prompt times out and is denied |
| `privacy.clipboard_notify_on_read` | `true` | Show a notification when any app reads the clipboard |
| `privacy.max_grants_per_app` | `100` | Maximum number of persistent grants per app |

### 11.3 Concurrent Access

Grant records are stored in SQLite with WAL mode. Concurrent permission checks from multiple apps are safe because:

1. Read operations (grant lookup) do not block each other.
2. Write operations (grant creation, consumption, revocation) use row-level transactions.
3. One-time grant consumption is atomic: the check-and-consume happens within a single transaction, preventing double-use.

### 11.4 Cleanup

- Consumed one-time grants are retained for 7 days for audit purposes, then deleted by a background task.
- Grants for uninstalled apps are deleted immediately during app uninstallation (called by `cortex-runtime`).
- Prompt block list entries for uninstalled apps are deleted during app uninstallation.

---

## 12. Failure Modes and Error Handling

### 12.1 Error Taxonomy

```rust
enum PolicyError {
    /// The requested permission is not granted.
    PermissionDenied { reason: DenyReason },

    /// The permission URI is malformed or invalid.
    InvalidPermission { permission: String, reason: String },

    /// The grant record could not be found.
    GrantNotFound { grant_id: GrantId },

    /// The app ID does not correspond to an installed app.
    AppNotFound { app_id: AppId },

    /// The user ID does not match any known user.
    UserNotFound { user_id: UserId },

    /// The prompt timed out without user response.
    PromptTimeout { request_id: Uuid },

    /// The prompt channel is not available (no frontend connected).
    PromptChannelUnavailable,

    /// A storage error occurred in cortex-db.
    StorageError { source: DbError },

    /// Validation error on input data.
    ValidationError { field: String, reason: String },

    /// The requested operation would exceed the maximum grants per app.
    GrantLimitExceeded { app_id: AppId, max: u32 },
}
```

### 12.2 Failure Scenarios

| Scenario | Behavior |
|----------|----------|
| App has no grant for the requested permission | Dispatch prompt. If prompt blocked, return `PermissionDenied` with `PromptBlocked`. |
| One-time grant already consumed | Return `PermissionDenied` with `GrantConsumed`. |
| Grant revoked mid-operation | The next permission check for that grant fails. In-progress operations that have already passed their check are not retroactively cancelled (the check is a point-in-time decision). |
| Prompt timeout (120 seconds) | Return `PermissionDenied` with `PromptTimeout`. Log the timeout event. |
| Frontend disconnected during prompt | Return `PermissionDenied` with `PromptChannelUnavailable`. This is treated as a denial, not an error that retries. |
| Database unavailable during check | Return 503 Service Unavailable. Do not fall back to allowing the operation. Do not cache previous grant decisions. |
| Invalid permission URI in grant request | Return 400 with `InvalidPermission` error explaining the validation failure. |
| Attempt to create persistent grant for `ai://command/execute` | Return 400 with `ValidationError`. This permission only supports one-time grants. |
| Grant limit exceeded for an app | Return 429 with `GrantLimitExceeded`. The user must revoke existing grants before new ones can be created. |
| Wildcard pattern in grant does not match any valid path | Return 400 with `ValidationError`. Wildcard grants must specify at least one directory level. |

### 12.3 Recovery Behavior

- **After server restart:** All grants persist in `cortex-db`. Pending prompts are lost and treated as denials. Apps that were awaiting prompt responses receive `PermissionDenied`.
- **After database corruption:** `cortex-db` is responsible for recovery. `cortex-policy` reports storage errors and does not attempt auto-repair. If the grant store is corrupted, all permission checks fail with `StorageError`, which means all privileged operations are blocked until recovery.
- **After app crash during prompt:** The prompt response channel is per-session. If the app crashes, the prompt remains open and awaits user response. The prompt is not tied to the app's lifecycle.

---

## 13. Security and Permissions

### 13.1 Server-Side Enforcement

All permission checks execute on the server. The client (browser) is an untrusted rendering surface. The following rules are absolute:

1. The client never makes authorization decisions. The client only renders the permission prompt UI and forwards the user's response to the server.
2. The client never stores grants. Grants are stored server-side in `cortex-db`.
3. The client never bypasses a permission check. If the server returns `PermissionDenied`, the client must display the error. The client must not attempt the operation through an alternative path.
4. API endpoints that require permission enforcement perform the check server-side before executing the operation. The client's API call is the trigger, not the authority.

### 13.2 First-Party App Parity

First-party apps have no elevated privileges. They go through the same permission check flow as third-party apps. The only difference is that first-party app manifests may declare permissions that are pre-approved during OS installation (handled by the first-boot setup flow), but these are still stored as grants in the policy store and can be revoked by the user.

### 13.3 Permission Escalation Prevention

1. Apps cannot modify their own grants. Grants are created only through the user-facing prompt flow or the Settings UI.
2. Apps cannot request permissions not declared in their manifest. If an app attempts an operation requiring a permission not in its manifest, the operation is denied with `PermissionDenied` and a warning is logged.
3. Apps cannot escalate from read to write. A grant for `file://documents/read` does not imply any access for `file://documents/write`.
4. Apps cannot delegate permissions. An app with `ai://file/read` cannot pass that access to another app.

### 13.4 Audit Completeness

Every permission check emits an audit event, regardless of outcome:

| Decision | Audit Level | Fields |
|----------|-------------|--------|
| Granted (from existing grant) | INFO | user_id, app_id, permission, grant_id, grant_type |
| Granted (from prompt approval) | INFO | user_id, app_id, permission, grant_id, grant_type, prompt_response_time_ms |
| Denied (no grant, prompt denied) | WARN | user_id, app_id, permission, prompt_response_time_ms |
| Denied (prompt timeout) | WARN | user_id, app_id, permission |
| Denied (prompt blocked) | WARN | user_id, app_id, permission |
| Denied (consumed) | INFO | user_id, app_id, permission, grant_id |
| Revoked | INFO | user_id, app_id, permission, grant_id |

### 13.5 Sensitive Data in Permissions

- Permission URIs are not sensitive. They are logged in audit events.
- The content of files, clipboard data, and network payloads are not part of the permission system and must not be logged by `cortex-policy`.
- Grant records contain app IDs and permission URIs but not the data accessed under the grant.

---

## 14. Performance Requirements

| Operation | Maximum Latency (p99) |
|-----------|----------------------|
| Permission check (grant lookup, no prompt) | 2ms |
| Permission check with prompt (including user interaction) | User-dependent (bounded by 120s timeout) |
| Grant creation | 5ms |
| Grant revocation | 5ms |
| List grants for a user | 20ms (for up to 500 grants) |
| List grants for an app | 10ms (for up to 100 grants) |
| Wildcard matching | 1ms per pattern |
| Cleanup of consumed grants (batch) | 100ms for 1000 records |

### 14.1 No Caching Rationale

Permission checks are not cached. Every privileged operation performs a live check against the grant store. This ensures that revocation is immediate: the moment a grant is deleted, the next check for that grant fails. Caching would introduce a window during which a revoked grant could still be used.

The performance target of 2ms for a grant lookup is achievable with an indexed SQLite query by (app_id, user_id, category, resource, action). The grant table must have a composite index on these columns.

### 14.2 Batch Operations

Grant listing and revocation support batch operations to avoid N+1 queries when the Settings UI loads or when an app is uninstalled.

---

## 15. Accessibility Requirements

### 15.1 Permission Prompt Dialog

- The dialog title and body text must have a minimum contrast ratio of 4.5:1.
- All three buttons ("Allow Once", "Always Allow", "Deny") must be keyboard-focusable.
- Tab order: "Allow Once" -> "Always Allow" (if available) -> "Deny".
- The dialog must trap focus: Tab and Shift+Tab cycle within the dialog buttons.
- Screen readers must announce the dialog title and body when it appears (via `aria-modal="true"` and `role="dialog"`).
- Button labels must be descriptive: "Allow Once", "Always Allow", "Deny". Do not use "OK" or "Cancel".
- Color is not the sole indicator of permission status. Icons and text labels accompany any color coding.

### 15.2 Settings Permissions Section

- The list of grants is navigable via keyboard (arrow keys).
- Each grant entry has an accessible name describing the app and permission.
- The "Revoke" button is announced with the grant it affects (e.g., "Revoke file read access for Notes App").
- Search within the permissions list is keyboard-accessible.

---

## 16. Observability and Logging

### 16.1 Log Events

| Event | Level | Fields |
|-------|-------|--------|
| Permission granted (existing grant) | INFO | user_id, app_id, permission, grant_id, grant_type |
| Permission granted (prompt approved) | INFO | user_id, app_id, permission, grant_id, grant_type, response_time_ms |
| Permission denied (no grant) | WARN | user_id, app_id, permission |
| Permission denied (prompt denied) | WARN | user_id, app_id, permission, response_time_ms |
| Permission denied (prompt timeout) | WARN | user_id, app_id, permission |
| Permission denied (prompt blocked) | WARN | user_id, app_id, permission |
| Permission denied (consumed) | INFO | user_id, app_id, permission, grant_id |
| Grant created | INFO | user_id, app_id, permission, grant_id, grant_type |
| Grant revoked | INFO | user_id, app_id, permission, grant_id |
| All grants revoked for app | INFO | user_id, app_id, count |
| Clipboard read notification | INFO | user_id, app_id |
| Permission prompt blocked for app | WARN | user_id, app_id |

### 16.2 Metrics

| Metric | Type | Labels |
|--------|------|--------|
| policy_check_total | Counter | decision (granted/denied), category |
| policy_check_duration_ms | Histogram | category |
| policy_prompt_total | Counter | decision (allow_once/always_allow/deny/timeout) |
| policy_prompt_response_time_ms | Histogram | |
| policy_active_grants | Gauge | grant_type, category |
| policy_revocations_total | Counter | |
| policy_consumed_grants_total | Counter | category |

### 16.3 Sensitive Data

The following data is NEVER logged:
- Contents of files accessed under a grant
- Clipboard contents
- Network request payloads
- AI request or response contents

The following data IS logged:
- Permission URIs (e.g., "file://documents/read")
- App IDs
- Grant IDs
- Decision outcomes
- Timestamps

---

## 17. Testing Requirements

### 17.1 Unit Tests

| Test | Description |
|------|-------------|
| `test_permission_uri_parse_valid` | Parse valid URIs for all categories and verify components |
| `test_permission_uri_parse_invalid` | Reject malformed URIs: empty, missing parts, invalid category |
| `test_permission_uri_wildcard_parse` | Parse URIs with `**` wildcards in resource path |
| `test_grant_creation_persistent` | Create a persistent grant, verify it is stored and retrievable |
| `test_grant_creation_one_time` | Create a one-time grant, verify it is stored with consumed=false |
| `test_grant_consumption` | Create one-time grant, check once (allow), check again (deny with GrantConsumed) |
| `test_grant_persistence_across_checks` | Create persistent grant, check multiple times, verify all succeed |
| `test_grant_revocation` | Create persistent grant, revoke it, verify next check denies |
| `test_revocation_notifies_app` | Revoke grant, verify PermissionsRevoked event is emitted |
| `test_wildcard_match_exact` | Grant for `file://documents/**/read` matches `file://documents/read` |
| `test_wildcard_match_subpath` | Grant for `file://documents/**/read` matches `file://documents/sub/file/read` |
| `test_wildcard_no_cross_resource` | Grant for `file://documents/**/read` does not match `file://pictures/read` |
| `test_wildcard_action_strict` | Grant for `file://documents/**/read` does not match `file://documents/write` |
| `test_ai_command_always_onetime` | Verify persistent grant for `ai://command/execute` is rejected |
| `test_prompt_blocked_auto_deny` | Block prompts for app, verify check returns Deny(PromptBlocked) |
| `test_prompt_timeout` | Verify prompt times out after configured duration |
| `test_check_no_grant_no_prompt` | Verify `check()` returns Deny without dispatching a prompt |
| `test_check_or_prompt_dispatches` | Verify `check_or_prompt()` dispatches a prompt when no grant exists |
| `test_audit_event_emitted_on_grant` | Verify audit event is emitted when a permission is granted |
| `test_audit_event_emitted_on_deny` | Verify audit event is emitted when a permission is denied |
| `test_max_grants_per_app_limit` | Verify grant creation fails when per-app limit is reached |
| `test_first_party_app_no_bypass` | Verify first-party apps go through the same check flow |
| `test_manifest_permission_enforcement` | Verify app cannot use permissions not in its manifest |

### 17.2 Integration Tests

| Test | Description |
|------|-------------|
| `test_full_grant_lifecycle` | Create grant -> check (allow) -> revoke -> check (deny) |
| `test_onetime_grant_lifecycle` | Create one-time grant -> check (allow, consumed) -> check (deny) |
| `test_prompt_flow_allow_once` | No grant -> prompt -> Allow Once -> check (allow) -> check (deny, consumed) |
| `test_prompt_flow_always_allow` | No grant -> prompt -> Always Allow -> check (allow) -> check (allow, persistent) |
| `test_prompt_flow_deny` | No grant -> prompt -> Deny -> check (deny) |
| `test_prompt_flow_timeout` | No grant -> prompt -> timeout -> check (deny) |
| `test_revocation_during_operation` | Grant exists -> check (allow) -> revoke -> next check (deny) |
| `test_multi_app_isolation` | Grant for app A does not allow app B to use the same permission |
| `test_wildcard_file_grants` | Create wildcard grant -> verify matches multiple subpaths |
| `test_cleanup_consumed_grants` | Create one-time grants -> consume -> run cleanup -> verify consumed grants older than 7 days are deleted |
| `test_app_uninstall_cleans_grants` | Create grants for app -> uninstall app -> verify all grants deleted |
| `test_ai_permission_chain` | Test the AI permission chain: completion request, file read, file write, clipboard read, command execute |
| `test_clipboard_read_notification` | Grant clipboard read -> read clipboard -> verify notification emitted |

### 17.3 Security Tests

| Test | Description |
|------|-------------|
| `test_no_client_side_bypass` | Verify that modifying client-side state does not bypass server-side checks |
| `test_no_grant_escalation_read_to_write` | Verify file read grant does not allow file write |
| `test_no_cross_app_grant_usage` | Verify app A cannot use app B's grants |
| `test_no_permission_bypass_via_api` | Verify calling the API directly without a session results in 401 |
| `test_revoke_all_for_app` | Create multiple grants for an app -> revoke all -> verify all are gone |
| `test_concurrent_grant_consumption` | Two concurrent checks for the same one-time grant: only one succeeds |
| `test_no_undeclared_permission_usage` | App with manifest declaring only file read attempts network connect -> denied |
| `test_audit_log_completeness` | Perform 100 permission checks, verify all 100 have audit events |

---

## 18. Acceptance Criteria

### 18.1 Functional Checklist

- [ ] All seven permission categories (FileSystem, Clipboard, Network, Notification, AI, System, Device) are implemented and enforceable
- [ ] Permission URIs are validated according to the specified grammar
- [ ] Persistent grants survive server restarts
- [ ] One-time grants are consumed after a single successful check
- [ ] `ai://command/execute` only supports one-time grants (persistent creation is rejected)
- [ ] Wildcard file grants match subpaths correctly
- [ ] Wildcard grants do not match across different top-level directories
- [ ] Permission prompts are dispatched to the frontend when no grant exists
- [ ] Prompt responses (Allow Once, Always Allow, Deny) create or deny grants correctly
- [ ] Prompt timeout (120 seconds) results in denial
- [ ] Grant revocation is immediate: the next permission check after revocation fails
- [ ] Revocation emits a `PermissionsRevoked` event on the command bus
- [ ] First-party apps go through the same permission check flow as third-party apps
- [ ] Apps cannot use permissions not declared in their manifest
- [ ] The maximum grants-per-app limit is enforced
- [ ] Consumed grants are cleaned up after 7 days by the background task
- [ ] Grants for uninstalled apps are deleted during uninstallation
- [ ] AI permission chain is enforced (completion request, file read, file write, clipboard, command)
- [ ] Clipboard reads emit a notification to the user
- [ ] All permission checks emit audit events
- [ ] Permission check results never include sensitive data (file contents, clipboard data)
- [ ] The Settings app can list, revoke, and block permissions for all installed apps

### 18.2 Performance Checklist

- [ ] Permission check (grant lookup) completes in under 2ms at p99
- [ ] Grant creation completes in under 5ms at p99
- [ ] Grant revocation completes in under 5ms at p99
- [ ] Listing grants for a user (up to 500) completes in under 20ms at p99
- [ ] Wildcard matching completes in under 1ms per pattern at p99

### 18.3 Security Checklist

- [ ] All permission checks execute server-side
- [ ] No permission check result is cached (every check hits the store)
- [ ] No grant can be created without user approval (prompt or Settings UI)
- [ ] `ai://command/execute` never has persistent grants
- [ ] First-party apps have no elevated privileges
- [ ] Audit events are emitted for every permission decision
- [ ] Concurrent one-time grant consumption is atomic (only one consumer succeeds)
- [ ] Revocation takes effect immediately (next check fails)
- [ ] The prompt channel is secured per user session (no cross-session prompt injection)

---

## 19. Build Order and Dependencies
**Layer 3**. Depends on: 01, 02 (core architecture), 03 (identity/user context)

### 19.1 Build Sequence

`cortex-policy` must be built after:
1. `cortex-core` (shared types, error macros)
2. `cortex-db` (storage interface, SQLite backend)
3. `cortex-auth` (user identity types, UserId)
4. `cortex-settings` (reads privacy settings, default permission behavior)

`cortex-policy` must be built before:
1. `cortex-api` (HTTP route registration for policy endpoints)
2. `cortex-ai` (AI permission checks)
3. `cortex-files` (file access permission checks)
4. `cortex-notify` (notification permission checks)
5. `cortex-runtime` (app sandboxing, manifest validation)
6. `apps/desktop-shell` (permission prompt UI rendering)
7. `apps/settings-app` (permission management UI)
8. All first-party and third-party apps (they consume the permission check API)

### 19.2 External Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| uuid | 1.x | UUIDv7 generation for grant IDs |
| serde | 1.x | Serialization/deserialization |
| serde_json | 1.x | JSON serialization |
| chrono | 0.4 | Timestamp handling |
| regex | 1.x | Permission URI validation and wildcard matching |
| tokio | 1.x | Async runtime for prompt channel |
| tracing | 0.1 | Structured logging for audit events |

---

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Role-based access control (RBAC): CortexOS is a single-user OS. There are no roles or permission inheritance hierarchies.
- Mandatory access control (MAC) or Bell-LaPadula models: Not applicable to the single-user browser OS context.
- Content-based access control: Permissions govern access to resources and actions, not to specific content within resources. The policy engine does not inspect file contents.
- Encryption or data-level protection: Permissions control whether an operation is allowed. Data at rest and in transit is protected by other subsystems.
- Cross-user permission sharing: Not applicable in single-user mode.
- Permission delegation or proxying: Apps cannot delegate their permissions to other apps.
- Time-based or context-based permission conditions (e.g., "only allow during work hours"): Not in v1.

### 20.2 Anti-Patterns

1. **NEVER trust the client for authorization.** All permission checks run server-side. The client is an untrusted rendering surface.
2. **NEVER cache permission check results.** Caching introduces a window during which revocation is not effective.
3. **NEVER silently fallback when a permission is denied.** The operation must fail with `PermissionDenied`. No degraded mode, no partial success.
4. **NEVER allow first-party apps to bypass the permission system.** First-party apps use the same check flow as third-party apps.
5. **NEVER allow apps to create their own grants.** Grants are created only through the user-facing prompt flow or the Settings UI.
6. **NEVER allow persistent grants for `ai://command/execute`.** This permission is always one-time.
7. **NEVER omit audit logging for a permission check.** Every check, regardless of outcome, must emit an audit event.
8. **NEVER log sensitive data in audit events.** File contents, clipboard data, network payloads, and AI request/response bodies are never included in audit logs.
9. **NEVER allow permission escalation.** A read grant never implies write access. A scoped grant never implies broader access.
10. **NEVER allow undeclared permissions.** If an app attempts an operation requiring a permission not in its manifest, deny the operation and log a warning.
11. **NEVER use "OK"/"Cancel" in permission prompts.** Use explicit labels: "Allow Once", "Always Allow", "Deny".
12. **NEVER allow the prompt dialog to be dismissed without a choice.** No close button, no click-outside-to-dismiss. The user must choose.

---

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership Reminder

You are implementing `cortex-policy`. You own:
- Permission schema definition and URI parsing
- Grant storage, retrieval, and lifecycle (creation, consumption, revocation)
- Permission check evaluation engine (`check()` and `check_or_prompt()`)
- Wildcard matching for file permissions
- Prompt channel interface for user approval
- Audit event emission for all permission decisions
- Default policy configuration

You do NOT own:
- Authentication/user identity (`cortex-auth`)
- Settings storage (`cortex-settings`)
- Filesystem operations (`cortex-files`)
- AI request dispatch (`cortex-ai`)
- Notification delivery (`cortex-notify`)
- App runtime/manifest parsing (`cortex-runtime`)
- HTTP server setup (`cortex-api`)
- Permission prompt UI rendering (`apps/desktop-shell`)

### 21.2 Recommended Crate Structure

```
crates/cortex-policy/
  Cargo.toml
  src/
    lib.rs              -- pub mod for all modules
    permission.rs       -- Permission, PermissionCategory, Action, URI parsing
    categories.rs       -- category definitions and helpers
    grant.rs            -- Grant, GrantId, GrantType, NewGrant
    engine.rs           -- PolicyEngine trait and implementation
    revocation.rs       -- revoke(), revoke_all_for_app(), event dispatch
    audit.rs            -- PermissionAuditEmitter, PermissionAuditEvent
    prompt.rs           -- PermissionPromptRequest/Response, PromptChannel trait
    store.rs            -- PolicyStore trait (implemented by cortex-db)
    error.rs            -- PolicyError, DenyReason
    validation.rs       -- URI validation, wildcard matching
    defaults.rs         -- default configuration values
```

### 21.3 What Can Be Stubbed Initially

- The prompt channel can be initially stubbed with an in-memory channel that auto-approves all requests. This allows other subsystems to be tested before the frontend prompt UI is implemented. The stub must log a warning on every auto-approval: "Prompt channel stub: auto-approving permission {permission} for app {app_id}".
- Audit event emission can be initially stubbed with `tracing::info!` calls. The full structured audit pipeline can be connected once `cortex-observability` is ready.
- Wildcard matching can initially support only `**` (double-star) wildcards. More complex patterns (single-star, character classes) are not in v1.

### 21.4 What Must Be Real in v1

- Permission URI parsing and validation (all categories, all actions)
- Grant storage in `cortex-db` (SQLite)
- Persistent and one-time grant types
- One-time grant consumption (atomic check-and-consume)
- Grant revocation with immediate effect
- `PermissionsRevoked` event emission on revocation
- `ai://command/execute` always-one-time enforcement
- Wildcard matching for file permissions (`**` patterns)
- Prompt dispatch and response handling
- Prompt timeout (120 seconds)
- Audit event emission for every permission check
- Server-side enforcement: no client-side authorization logic
- First-party app parity: no bypass paths
- Manifest permission validation (apps cannot use undeclared permissions)

### 21.5 What Cannot Be Inferred

- The prompt timeout is exactly 120 seconds. Do not use a different value.
- `ai://command/execute` is always one-time. There is no "Always Allow" option. Do not add one.
- One-time grant consumption must be atomic within a single database transaction. Do not implement a two-phase check-then-consume pattern.
- The maximum grants per app is 100. Do not use a different limit.
- Consumed grants are retained for 7 days for audit, then deleted. Do not delete immediately or retain indefinitely.
- Permission URIs follow the exact grammar specified in section 8.2. Do not invent alternative formats.
- The "Deny" button label is always "Deny". Do not use "Cancel", "No", or "Reject".
- Clipboard read notifications are always emitted, regardless of the grant type. Do not suppress them for persistent grants.

### 21.6 Stop Conditions

The subsystem is considered done when:
1. All unit tests pass (see section 17.1)
2. All integration tests pass (see section 17.2)
3. All security tests pass (see section 17.3)
4. The acceptance criteria checklist (section 18) is fully verified
5. No compiler warnings in `cortex-policy`
6. `cargo clippy --all-targets` produces no warnings for `cortex-policy`
7. `cargo audit` shows no known vulnerabilities in `cortex-policy` dependencies
8. Manual test: install an app, grant a permission, verify the operation succeeds, revoke the permission, verify the operation fails
9. Manual test: attempt `ai://command/execute` twice, verify the second attempt prompts again (one-time enforcement)

### 21.7 Testing Gates

Before marking this subsystem as complete, verify:
- `cargo test -p cortex-policy` passes with 100% of listed tests present and passing
- `cargo test -p cortex-policy -- --nocapture` shows audit events in log output (no sensitive data)
- Manual test: create a persistent file grant, verify reads succeed, revoke, verify reads fail immediately
- Manual test: create a one-time grant, verify first use succeeds and second use fails
- Manual test: block prompts for an app, verify auto-deny behavior
- Manual test: verify the Settings app can list and revoke all grants for any installed app
