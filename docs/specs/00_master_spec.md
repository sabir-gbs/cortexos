# 00. Master Specification -- CortexOS

**Status:** Implementation-grade
**Owner:** Product-level authority; delegates to child specs for subsystem details
**Version:** 1.0.0

---

## 1. Purpose

This document is the master specification for CortexOS, an AI-native browser operating system. It defines the product, its goals and non-goals, system-wide engineering assumptions, the spec-authoring contract, coding-agent guardrails, the canonical subsystem dependency order, and the relationship between this master spec and every child spec (01 through 23, child specs 17a through 17g, child specs 18a through 18e, and appendices A through D).

This master spec is the top-level authority for product definition and cross-cutting concerns. Each child spec is the authoritative document within its declared subsystem scope. In the event of a conflict, the more specific child spec overrides this master spec only within that child spec's declared scope.

---

## 2. Scope

This master spec covers:

- Product definition and identity for CortexOS.
- Product goals and product non-goals.
- System-wide engineering assumptions (languages, architecture, repository structure).
- The spec-authoring contract: how specs relate to each other, parent-child conflict resolution, and spec authority rules.
- Coding-agent guardrails: rules that every implementation agent (Claude Code, Codex, GLM-5.1, or any equivalent) must follow without exception.
- Subsystem dependency order: the canonical build layers and the order in which subsystems must be implemented.
- A complete index of every child spec by number and title.
- Completion criteria for each subsystem spec.
- Cross-cutting invariants that apply to all subsystems.

This master spec does not cover:

- Implementation details of any individual subsystem. Each subsystem's data model, interfaces, failure modes, and acceptance criteria are defined exclusively in the corresponding child spec.
- Specific UI layouts, component hierarchies, or API endpoint definitions. Those are owned by their respective child specs.

---

## 3. Out of Scope

The following are out of scope for this master spec document and for CortexOS v1 as a whole:

- Mobile operating system features (touch-first UI, mobile-specific sensors, app store distribution for mobile).
- Server operating system features (headless deployment, multi-user concurrent sessions, server clustering).
- Container platform features (Docker/Kubernetes integration, container orchestration).
- Native hardware driver support (GPU drivers, USB device drivers, print drivers).
- Multi-user concurrent sessions with namespace isolation in v1.
- Localization and internationalization beyond English in v1.
- Real-time collaboration features (multi-user simultaneous editing).
- Cloud synchronization of user data in v1.

---

## 4. Objectives

### 4.1 Product Goals

1. **AI-native OS experience**: AI is a system layer, not a bolt-on feature. The user can select their Preferred LLM at the OS level (in Settings), configure per-app and per-feature overrides, and the OS routes AI requests deterministically. The OS remains fully functional even if no AI provider is configured.

2. **Browser-rendered desktop**: The entire desktop environment (shell, taskbar, windows, apps) is rendered in a browser. No native window manager, no native rendering pipeline. The browser is the only display surface.

3. **First-party apps**: CortexOS ships with a complete set of first-party applications: Settings, Calculator, Text Editor, Notes, File Manager, Media Viewer, Terminal-lite, and Clock utilities. These apps demonstrate the platform's capabilities and provide baseline functionality.

4. **Bundled games**: CortexOS ships with five bundled games: Solitaire, Minesweeper, Snake, Tetris-like puzzle game, and Chess or Checkers. Games validate the app runtime, input handling, and rendering pipeline in a non-trivial domain.

5. **Extensible via SDK**: Third-party developers can build apps for CortexOS using a documented SDK, typed manifest schema, and sandboxed runtime. First-party apps have no hidden privileges over third-party apps.

6. **Privacy-first AI**: AI context access is permission-gated. PII can be redacted or hashed before sending to external providers. Budget controls prevent runaway spending. All AI actions produce audit trails.

7. **Deterministic routing**: AI request routing follows a strict precedence order: per-feature override > per-app override > global preferred provider. Fallback chains are user-configurable. No provider lock-in in core logic.

### 4.2 Engineering Objectives

1. Every subsystem has a single owner crate or app directory with clearly defined boundaries.
2. Security-sensitive logic (authentication, authorization, credential storage, AI provider credentials) is server-side only. The browser client is never the source of truth for security decisions.
3. All inter-subsystem communication uses typed interfaces via the command bus (spec 10). No untyped events, no ad-hoc REST calls bypassing the bus.
4. The codebase is a monorepo with a Cargo workspace for Rust crates and a pnpm workspace (or equivalent) for TypeScript packages.
5. Every spec is implementation-grade: deterministic language, concrete data structures, explicit error handling, and binary acceptance criteria.

---

## 5. User-Visible Behavior

### 5.1 First Boot

On first boot, the user sees a setup screen where they create a username, display name, and password. After setup, they are taken directly to the desktop shell. There is no guest mode.

### 5.2 Desktop Shell

The user sees a browser-rendered desktop with a taskbar at the bottom, desktop icons (File Manager, Terminal, Settings, Trash), a system tray with clock, volume, network status, and AI provider status, and the ability to launch apps via the app launcher (Meta key or button) or the command palette (Ctrl+Space).

### 5.3 AI Assistant

When the user opens the AI assistant (Ctrl+Shift+A or per-app button), the assistant panel appears. If no AI provider is configured, the panel shows a prompt to configure one in Settings. When configured, the assistant responds to queries using the user's Preferred LLM, with model disclosure shown by default.

### 5.4 Settings

The Settings app provides control over: user profile, AI provider configuration (preferred LLM, model, fallback chain, privacy mode, budget policy), permissions, theme, accessibility, notifications, and system information.

### 5.5 Apps and Files

All apps open in windowed containers managed by the window manager. Files are stored in a virtual filesystem. The File Manager provides navigation, and file associations open files in the appropriate app (e.g., .txt files in Text Editor, .png files in Media Viewer).

### 5.6 Graceful Degradation

If the AI runtime is unavailable (no provider configured, provider unreachable, budget exceeded), the OS and all apps remain fully functional. AI-specific features are disabled or show a status message. Non-AI functionality is never blocked by AI failures.

---

## 6. System Behavior

### 6.1 Architecture Overview

CortexOS is a client-server application where:

- **Server side**: Rust binaries running on the host machine. Owns all security-sensitive logic, persistent storage, AI provider communication, policy enforcement, and the command bus.
- **Client side**: TypeScript/React application running in a browser. Owns the desktop shell, window rendering, app UIs, and user interaction. Uses a hybrid transport model:
  - HTTP/REST for authentication, bootstrap, coarse-grained CRUD, diagnostics, and artifact download/upload
  - WebSocket command bus for real-time events, streaming, subscriptions, and latency-sensitive interactive operations

### 6.2 Trust Boundary

The browser client is untrusted. Every security-relevant decision (authentication, authorization, permission checks, credential validation) is made on the server. The client renders state provided by the server and sends user actions to the server for validation.

### 6.3 Startup Sequence

1. Browser loads the Desktop Shell entry point.
2. Shell checks health/bootstrap endpoints and determines whether an authenticated session already exists.
3. Shell authenticates the user over HTTP if needed (or shows the login screen).
4. Shell opens the authenticated WebSocket connection to the command bus.
5. Shell fetches settings, app list, and initial state.
6. Shell renders the desktop environment.
7. User interacts with the desktop shell to launch apps, open files, and configure settings.

### 6.4 Degraded Mode

When any backend service is unavailable:
- The desktop shell renders a connection-lost banner.
- The shell retries connection with exponential backoff (1s, 2s, 4s, 8s, max 30s).
- User customizations made during outage are queued and persisted on reconnection.
- Individual apps show feature-specific degraded behavior (e.g., AI panel shows "unavailable", file operations show error toast).

### 6.5 Cross-Cutting Invariants

These invariants apply to every subsystem without exception:

1. **INV-00-1**: No client-side authorization as source of truth. The server validates every security-relevant operation.
2. **INV-00-2**: No untyped event payloads on the command bus. Every event has a defined schema.
3. **INV-00-3**: No hidden first-party privileges. First-party apps go through the same permission model as third-party apps.
4. **INV-00-4**: No provider lock-in. The AI runtime uses a provider abstraction. No provider-specific code in core logic.
5. **INV-00-5**: No hardcoded secrets or credentials in source code.
6. **INV-00-6**: No silent failures. Every error is logged and surfaced to the user where appropriate.
7. **INV-00-7**: No direct filesystem, network, or OS API access from apps. All access is mediated by system services.
8. **INV-00-8**: AI is optional. The OS must remain functional with no AI provider configured.

---

## 7. Architecture

### 7.1 Technology Stack

| Layer | Technology | Notes |
|-------|-----------|-------|
| Backend language | Rust (stable channel) | All backend/system/runtime/orchestration/storage/policy/AI layers |
| Frontend language | TypeScript | All browser-facing UI surfaces |
| Frontend framework | React | Component-based UI |
| Monorepo structure | Cargo workspace (Rust) + pnpm workspace (TypeScript) | Single repository |
| Storage | SQLite (via cortex-db) | Persistent local storage |
| Communication | HTTP/REST + WebSocket command bus | HTTP for auth/bootstrap/CRUD/admin, WebSocket for realtime/streaming/events |
| AI provider protocol | HTTP/REST (provider-specific adapters) | OpenAI, Anthropic, Google, Ollama, Zhipu, Custom |

### 7.2 Rust Crate Map

| Crate | Owner Spec | Purpose |
|-------|-----------|---------|
| `cortex-core` | 02 | Shared types, error macros, foundation primitives |
| `cortex-config` | 01 | Configuration loading and validation |
| `cortex-db` | 02 | Storage interface, SQLite backend |
| `cortex-api` | 02 | HTTP server, route registration, middleware |
| `cortex-auth` | 03 | Identity, authentication, sessions, user profiles |
| `cortex-policy` | 04 | Permissions, policy engine, trust model |
| `cortex-settings` | 05 | Settings service, namespace management, validation |
| `cortex-ai` | 06 | AI runtime, provider registry, model routing, fallback |
| `cortex-files` | 11 | Virtual filesystem, storage abstraction, file operations |
| `cortex-search` | 12 | Search indexing, global command palette |
| `cortex-notify` | 13 | Notifications service |
| `cortex-observability` | 14 | Logging, metrics, telemetry |
| `cortex-runtime` | 09 | App runtime, app lifecycle, sandboxing |
| `cortex-sdk` | 21 | SDK for third-party app development |
| `cortex-admin` | 22 | Admin, diagnostics, recovery |

### 7.3 Frontend Package Map

| Package | Owner Spec | Purpose |
|---------|-----------|---------|
| `apps/desktop-shell` | 07 | Desktop shell UI |
| `apps/settings-app` | 05 | Settings application UI |
| `apps/calculator-app` | 17a | Calculator application |
| `apps/text-editor-app` | 17b | Text editor application |
| `apps/notes-app` | 17c | Notes application |
| `apps/file-manager-app` | 17d | File manager application |
| `apps/media-viewer-app` | 17e | Media viewer application |
| `apps/terminal-lite-app` | 17f | Terminal-lite application |
| `apps/clock-utils-app` | 17g | Clock and utility applications |
| `apps/games/solitaire` | 18a | Solitaire game |
| `apps/games/minesweeper` | 18b | Minesweeper game |
| `apps/games/snake` | 18c | Snake game |
| `apps/games/tetris` | 18d | Tetris-like puzzle game |
| `apps/games/chess` | 18e | Chess or checkers game |
| `@cortexos/ui-components` | 16 | Shared UI component library |
| `@cortexos/runtime-client` | 09 | Frontend client for app runtime |
| `@cortexos/files-client` | 11 | Frontend client for filesystem |
| `@cortexos/ai-client` | 19 | Frontend client for AI surfaces |
| `@cortexos/theme` | 16 | Design token consumer |

### 7.4 Dependency Direction Rules

1. Rust crates may only depend on crates at the same layer or a lower layer (see section 19 for layer definitions). A crate at layer N must not depend on a crate at layer N+k where k > 0.
2. Frontend packages communicate exclusively through the command bus and system service clients. No direct inter-app imports.
3. No circular dependencies between crates or packages.
4. The `cortex-core` crate has zero internal dependencies (only external crate dependencies).

---

## 8. Data Model

### 8.1 Global Cross-Cutting Types

These types are shared across multiple subsystems and are defined here as the authoritative source:

```rust
/// Unique identifier for a user. UUIDv7.
struct UserId(String);

/// Unique identifier for an app instance. UUIDv4.
struct AppInstanceId(String);

/// Unique identifier for a session token. 256-bit random value, base64url-encoded.
struct SessionToken(String);

/// ISO 8601 datetime string (e.g., "2026-03-29T14:30:00Z").
/// Used in all API contracts for consistency.
type Timestamp = String;

/// AI provider enum. The set of supported providers.
enum AiProvider {
    OpenAI,
    Anthropic,
    Google,
    Ollama,
    Zhipu,
    Custom,
}

/// Model routing resolution result.
struct ResolvedProvider {
    provider: AiProvider,
    model: String,  // empty string means provider default
}
```

### 8.2 Settings Namespace Registry

Each subsystem owns a namespace in the settings service. This table defines the namespace ownership:

| Namespace | Owner Spec | Description |
|-----------|-----------|-------------|
| `ai` | 06 | AI provider, model, fallback, privacy, budget settings |
| `shell` | 07 | Desktop shell preferences (background, taskbar, clock) |
| `wm` | 08 | Window manager preferences |
| `accessibility` | 15 | Accessibility settings |
| `theme` | 16 | Theme preferences |
| `system` | 01, 03 | System-level configuration (session timeouts, SSO) |

No subsystem may write to a namespace it does not own. Any subsystem may read from any namespace.

---

## 9. Public Interfaces

### 9.1 Command Bus

The command bus (spec 10) is the authoritative real-time communication channel between the browser client and the Rust backend. All subscriptions, streaming responses, and server-push events flow through it. Child specs may additionally define HTTP endpoints for authentication, bootstrap, coarse-grained CRUD, diagnostics, and file transfer.

### 9.2 HTTP API Surface

The HTTP API (spec 02) is exposed by `cortex-api` and follows these conventions:

- Base path: `/api/v1/`
- Authentication: session cookie (HTTP-only, Secure, SameSite=Strict)
- Content type: JSON for all request/response bodies
- Error format: `{ "error": { "code": string, "message": string } }`
- Versioning: URL-based versioning (`/api/v1/`). Breaking changes require a new version prefix.

### 9.3 Inter-Service Communication

Rust crates communicate via the command bus (in-process, typed commands and events). No crate calls another crate's internal functions directly. All cross-crate communication goes through the public API traits defined in each crate.

---

## 10. Internal Interfaces

### 10.1 Spec System Internal Structure

This master spec and all child specs follow the 21-section template:

1. Purpose
2. Scope
3. Out of Scope
4. Objectives
5. User-Visible Behavior
6. System Behavior
7. Architecture
8. Data Model
9. Public Interfaces
10. Internal Interfaces
11. State Management
12. Failure Modes and Error Handling
13. Security and Permissions
14. Performance Requirements
15. Accessibility Requirements
16. Observability and Logging
17. Testing Requirements
18. Acceptance Criteria
19. Build Order and Dependencies
20. Non-Goals and Anti-Patterns
21. Implementation Instructions for Claude Code / Codex

Every major spec and every significant subspec includes all 21 sections. If a section is not deeply relevant to a given subspec, the section is still included with concise but concrete content.

### 10.2 Spec Authority Rules

1. This master spec (00) defines product-level goals, non-goals, engineering assumptions, guardrails, and the build order.
2. Each child spec (01 through 23, plus appendices) is authoritative within its declared scope.
3. In the event of a conflict between this master spec and a child spec, the child spec overrides this master spec only within the child spec's declared scope.
4. In the event of a conflict between a parent spec (e.g., 17) and its child spec (e.g., 17a), the child spec overrides the parent spec only within the child spec's declared scope.
5. No spec may define behavior in a scope owned by another spec without explicit cross-reference.
6. If two child specs conflict, the conflict must be resolved by updating one or both specs. The master spec does not adjudicate between child specs.

---

## 11. State Management

### 11.1 State Location Rules

| State Category | Storage Location | Example |
|---------------|-----------------|---------|
| User records | `cortex-db` (SQLite) | User identity, credentials |
| Session records | `cortex-db` (SQLite) | Active sessions |
| Settings | `cortex-db` (SQLite) via `cortex-settings` | User preferences |
| File metadata | `cortex-db` (SQLite) via `cortex-files` | Paths, ownership, MIME, versions |
| File content blobs | Filesystem/object storage owned by `cortex-files` | Virtual filesystem content |
| App state | `cortex-db` (SQLite) via `cortex-runtime` | Window geometry, drafts, runtime state |
| AI conversations | `cortex-db` (SQLite) via AI surface layer | Conversation history and metadata |
| AI audit trail | `cortex-db` (SQLite) via `cortex-observability` | AI action logs |
| Search index | `cortex-db` (SQLite FTS) via `cortex-search` | Searchable metadata and text index |
| Ephemeral UI state | Browser memory (React state) | Hover, focus, dropdown open |
| Exported artifacts / crash dumps | Filesystem under owning service | Diagnostic bundles, crash dumps, imported packages |
| AI provider credentials | Server-side only (never sent to client) | API keys |

### 11.2 Server Authority Rule

For all persistent state, the server is the authoritative source. The client fetches state from the server, renders it, and sends mutations to the server. Client-side state is a cache that is reconciled with server state on every connection.

### 11.3 State Isolation

Each app's state is sandboxed under `cortex-runtime://app-state/{app-id}/`. Apps cannot read or write state outside their namespace.

---

## 12. Failure Modes and Error Handling

### 12.1 Cross-Cutting Failure Handling Rules

1. **Never crash silently**: Every unrecoverable error is logged at ERROR level and surfaced to the user with a generic message. Internal error details are never exposed to the user.
2. **Never lose user data**: Destructive operations require confirmation. Unsaved changes are persisted to session state on every mutation (debounced). The system must recover gracefully from browser refresh.
3. **Never block on unavailable services**: If a backend service is unreachable, the affected features degrade gracefully. A banner or status indicator communicates the outage. The user can continue using unaffected features.
4. **Always log errors**: Every error is logged to cortex-observability with structured fields (error code, subsystem, timestamp). No error is swallowed without a log entry.
5. **Always provide recovery**: Where possible, the system provides a recovery action (retry button, fallback provider, reconnection attempt). Dead-end states require a manual workaround documented in the error message.

### 12.2 Error Taxonomy (Cross-Cutting)

| Error Category | HTTP Code | Example |
|---------------|-----------|---------|
| Authentication error | 401 | Invalid or expired session |
| Authorization error | 403 | Permission denied |
| Validation error | 400 | Invalid input |
| Not found | 404 | File, app, or resource not found |
| Conflict | 409 | Duplicate resource |
| Rate limited | 429 | Too many requests |
| Service unavailable | 503 | Backend service unreachable |
| Internal error | 500 | Unexpected server error |

---

## 13. Security and Permissions

### 13.1 Cross-Cutting Security Rules

1. **SEC-00-1**: All authentication and authorization logic runs server-side. The client never makes security decisions.
2. **SEC-00-2**: All API keys and credentials are stored server-side and never sent to the browser client.
3. **SEC-00-3**: All user input is validated at the server boundary, regardless of client-side validation.
4. **SEC-00-4**: Permission checks are enforced by `cortex-policy` on every operation that requires authorization. No subsystem bypasses the policy engine.
5. **SEC-00-5**: First-party apps have no hidden privileges. They declare permissions in their manifest, and the policy engine enforces them identically to third-party apps.
6. **SEC-00-6**: Password hashing uses Argon2id (OWASP-recommended parameters).
7. **SEC-00-7**: Session tokens are opaque, 256-bit random values stored server-side. Tokens are never logged or included in URLs.
8. **SEC-00-8**: All AI actions that read user data or modify state produce audit trail entries in cortex-observability.
9. **SEC-00-9**: No `eval()`, no `new Function()`, no `innerHTML` with untrusted content in any frontend code.
10. **SEC-00-10**: Dependencies are audited via `cargo audit`. No release ships with known vulnerabilities in dependencies.

### 13.2 Permission Model Summary

Permissions are categorized as:

- **Required**: Granted at install time. App cannot function without them.
- **Optional**: Requested at first use. User can grant or deny. App must function (possibly with reduced capability) if denied.
- **AI-specific**: Additional gating for AI context access (file reading, clipboard reading). Subject to both the global AI privacy toggle and per-operation permission checks.

Permission enforcement is always server-side. The client renders UI based on server-provided permission state but never independently decides whether an operation is allowed.

---

## 14. Performance Requirements

### 14.1 System-Wide Performance Targets

| Metric | Target |
|--------|--------|
| Cold startup (first load, no cache) | <= 3000ms |
| Warm startup (cached) | <= 1500ms |
| App launch (click to interactive) | <= 500ms |
| AI request overhead (excluding provider latency) | <= 500ms |
| Window drag/resize | >= 60 fps (16ms per frame) |
| Desktop shell baseline memory (no apps open) | <= 80 MB |
| Settings change to visual update | <= 100ms |
| Command bus message processing latency | <= 5ms per message |

### 14.2 Performance Rules

1. No unbounded memory growth under normal operation.
2. No blocking operations on the main browser thread. Background operations use Web Workers or async patterns.
3. No performance regression greater than 15% from the previous release on any benchmark.
4. Performance targets are measured on modern browsers (Chrome 110+, Firefox 115+, Safari 16.4+, Edge 110+) on modern hardware.

---

## 15. Accessibility Requirements

### 15.1 Cross-Cutting Accessibility Rules

1. **ACC-00-1**: All interactive elements across all apps and the desktop shell are keyboard-navigable.
2. **ACC-00-2**: All interactive elements have visible focus indicators (2px minimum outline, theme-defined color).
3. **ACC-00-3**: All icons have `aria-label` attributes with descriptive text.
4. **ACC-00-4**: Color is never the sole indicator of state or information.
5. **ACC-00-5**: Minimum contrast ratios meet WCAG 2.1 AA: 4.5:1 for normal text, 3:1 for large text.
6. **ACC-00-6**: Focus traps correctly in modal dialogs. Escape closes modals.
7. **ACC-00-7**: Screen reader announcements for state changes via ARIA live regions.
8. **ACC-00-8**: Minimum touch target size for all interactive elements: 44x44px (WCAG 2.1 SC 2.5.5).
9. **ACC-00-9**: UI remains functional when browser text size is increased to 200%.
10. **ACC-00-10**: Respect `prefers-reduced-motion`. Disable non-essential animations when this media query is active.

---

## 16. Observability and Logging

### 16.1 Cross-Cutting Logging Rules

1. All subsystems log through `cortex-observability`. No subsystem implements its own logging infrastructure.
2. Log levels are used consistently:
   - **ERROR**: Failures that affect user-visible behavior and require attention.
   - **WARN**: Degraded behavior, fallback activation, recoverable failures.
   - **INFO**: Significant events (startup, shutdown, user actions, state changes).
   - **DEBUG**: Diagnostic information for development. Not enabled in production by default.
3. No sensitive data (passwords, session tokens, API keys, full file contents, PII) is logged at INFO level or above.
4. Structured log entries include: `subsystem`, `event`, `level`, `timestamp`, and event-specific `payload`. No freeform text logs.
5. All log entries are JSON-serializable.

### 16.2 AI Audit Trail

All AI actions produce audit trail entries that include:

- Timestamp
- User ID
- App ID that initiated the action
- AI provider and model used
- Action type (read, generate, modify)
- Confirmation status (whether user confirmed)
- Outcome (success, failure, rejected)

The audit trail is stored in cortex-observability and is queryable by the admin tools (spec 22).

---

## 17. Testing Requirements

### 17.1 Testing Tiers

| Tier | Scope | Execution Location |
|------|-------|--------------------|
| Unit tests | Per crate / per package | `cargo test` / `pnpm test` |
| Integration tests | Cross-crate boundaries | `tests/integration/` |
| End-to-end tests | Full user flows through browser | `tests/e2e/` (Playwright) |
| Smoke tests | Minimal functionality verification | `tests/smoke/` |
| Performance benchmarks | Startup, latency, memory | `tests/benchmarks/` |
| Accessibility tests | ARIA, keyboard, contrast | Integrated in E2E |

### 17.2 Cross-Cutting Testing Rules

1. Every Rust crate has a `tests` module. Every TypeScript package has a `tests/` directory.
2. Unit tests must be deterministic, isolated, and idempotent.
3. Integration tests must not depend on external services. Use in-memory databases and mock providers.
4. E2E tests must not depend on real AI API keys. Use mock providers.
5. All tests must clean up after execution.
6. Coverage targets are defined per crate in spec 23 (section 8.1).
7. A PR that reduces coverage below the target for its affected crate is blocked.

---

## 18. Acceptance Criteria

### 18.1 Master Spec Acceptance Checklist

- [ ] Product goals (section 4.1) are clearly defined and non-contradictory.
- [ ] Product non-goals (section 3) are explicitly stated.
- [ ] Engineering assumptions (section 4.2) are concrete and actionable.
- [ ] All 35 child specs (01 through 23, 17a through 17g, 18a through 18e, appendices A through D) are referenced by number and title.
- [ ] Subsystem dependency order (section 19) is complete and acyclic.
- [ ] Coding-agent guardrails (section 20) cover all anti-patterns.
- [ ] Conflict resolution rules (section 10.2) are unambiguous.
- [ ] Completion criteria (section 21) are defined for every child spec.
- [ ] Cross-cutting invariants (section 6.5) are explicit and testable.
- [ ] No TBD placeholders. No vague language. No aspirational statements.

### 18.2 Cross-Cutting Acceptance for Every Subsystem

Every subsystem spec must satisfy:

- [ ] All 21 mandatory sections are present and populated.
- [ ] Scope declares what the subsystem owns and what it does not own.
- [ ] Data models include concrete type definitions (Rust structs, TypeScript interfaces).
- [ ] Failure modes are enumerated with specific behavior for each.
- [ ] Security requirements are explicit and enforceable.
- [ ] Performance requirements are measurable.
- [ ] Acceptance criteria are binary (pass/fail) checklists.
- [ ] Build order and dependencies reference prerequisite specs by number.
- [ ] Implementation instructions provide stop conditions and testing gates.

---

## 19. Build Order and Dependencies

### 19.1 Canonical Build Layers

The subsystems must be implemented in this layer order. All subsystems in a layer depend only on subsystems at lower layers (same or lower layer number).

```
Layer 0:  01 Repository / Toolchain / Engineering Conventions
Layer 1:  02 Core Architecture and Runtime Boundaries
Layer 2:  03 Identity / Authentication / Sessions / User Profiles
Layer 3:  04 Permissions / Policy / Trust Model
Layer 4:  05 Settings Service and Settings App, 11 Virtual Filesystem and Storage Abstraction
Layer 5:  06 AI Runtime / Provider Registry / Preferred LLM / Model Routing
Layer 6:  10 System Command Bus and Event Model
Layer 7:  07 Desktop Shell, 08 Window Manager
Layer 8:  09 App Runtime and App Lifecycle
Layer 9:  13 Notifications, 12 Search / Indexing / Global Command Palette, 14 Observability / Logging / Telemetry
Layer 10: 16 Theme / Design Tokens / UI System, 15 Accessibility / Input / Keyboard System
Layer 11: 17 First-Party Core Apps (parent), 17a-17g (individual apps)
Layer 12: 18 Games Platform (parent), 18a-18e (individual games)
Layer 13: 19 AI System Surfaces and UX, 20 AI Action Permissions and Safety Controls
Layer 14: 21 SDK / Manifest / Third-Party App Platform
Layer 15: 22 Admin / Diagnostics / Recovery
Layer 16: 23 Release Readiness / QA / Acceptance Framework
```

### 19.2 Layer Rationale

- **Layer 0**: Repository structure and tooling must exist before any code is written.
- **Layer 1**: Core architecture defines the client-server boundary, service ownership, and communication patterns that all other layers depend on.
- **Layer 2**: Identity is required before any authenticated operation.
- **Layer 3**: Permissions depend on identity (who is asking) but must exist before any resource access.
- **Layer 4**: Settings and filesystem are independent of each other but both depend on permissions.
- **Layer 5**: AI runtime depends on settings (to read provider config) and filesystem (for model config).
- **Layer 6**: Command bus depends on core architecture and provides the communication layer for all higher layers.
- **Layer 7**: Desktop shell and window manager depend on command bus, settings, and theme tokens.
- **Layer 8**: App runtime depends on window manager, command bus, and filesystem.
- **Layer 9**: Notifications, search, and observability are system services that apps consume.
- **Layer 10**: Theme and accessibility are consumed by all UI surfaces but can be developed in parallel with apps.
- **Layer 11**: First-party apps depend on all lower layers being functional.
- **Layer 12**: Games are apps with no special system dependencies beyond what Layer 11 requires.
- **Layer 13**: AI surfaces and safety controls are higher-level features built on the AI runtime.
- **Layer 14**: SDK is the public API for third-party apps, built after all first-party apps validate the platform.
- **Layer 15**: Admin tools aggregate data from all subsystems.
- **Layer 16**: Release readiness validates everything else.

### 19.3 Within-Layer Build Order

Within a layer, subsystems may be built in parallel unless a dependency exists within the layer. Known within-layer dependencies:

- Layer 4: Settings (05) and Filesystem (11) are independent.
- Layer 7: Desktop Shell (07) and Window Manager (08) can be developed in parallel but the Desktop Shell requires Window Manager events for taskbar rendering.
- Layer 9: Notifications (13), Search (12), and Observability (14) are independent.
- Layer 10: Theme (16) should be built before or concurrently with Accessibility (15).
- Layer 11: Apps should be built in the order defined in spec 17 (section 19.2): 17g, 17a, 17f, 17e, 17c, 17d, 17b.
- Layer 12: Games can be built in any order.
- Layer 13: AI Surfaces (19) should be built before AI Safety (20).

---

## 20. Non-Goals and Anti-Patterns

### 20.1 Product Non-Goals

1. CortexOS is not a mobile OS. It targets desktop/laptop browsers with viewports from 1024px to 3840px.
2. CortexOS is not a server OS. It does not support headless deployment, multi-user concurrent sessions, or server clustering.
3. CortexOS is not a container platform. It does not integrate with Docker, Kubernetes, or equivalent.
4. CortexOS does not include native hardware drivers. It runs entirely in the browser.
5. CortexOS does not support multi-user concurrent sessions in v1. Single user per browser session.
6. CortexOS does not include localization or internationalization beyond English in v1.
7. CortexOS does not include real-time collaboration features in v1.
8. CortexOS does not include cloud synchronization of user data in v1.

### 20.2 Anti-Patterns (Coding-Agent Guardrails)

These rules are absolute. No implementation agent (Claude Code, Codex, GLM-5.1, or equivalent) may violate them under any circumstances.

1. **NEVER client-side auth**: The browser client is never the source of truth for authentication or authorization decisions. All security checks are server-side.
2. **NEVER bypass policy**: No code path may skip permission checks in `cortex-policy`. Even first-party apps go through the policy engine.
3. **NEVER provider lock-in**: The AI runtime must never contain provider-specific code in its core routing logic. Provider differences are handled by adapter implementations of a generic trait.
4. **NEVER hidden first-party privileges**: First-party apps must declare all permissions in their manifest. The system must never auto-grant permissions based on app identity.
5. **NEVER silent conflicts**: When state conflicts occur (concurrent writes, stale cache), the conflict must be detected and resolved explicitly. No silent last-write-wins without logging.
6. **NEVER untyped events**: Every command and event on the command bus must have a typed schema defined in the owning spec. No `any` types, no unstructured payloads.
7. **NEVER UI-only definition of done**: A feature is not done when the UI renders. It is done when: backend logic is implemented, error handling covers all failure modes, tests pass at the required coverage level, accessibility is verified, and observability is in place.
8. **NEVER hardcoded secrets**: No API keys, passwords, or credentials may appear in source code, configuration files committed to the repository, or log output.
9. **NEVER eval or innerHTML with untrusted input**: No `eval()`, no `new Function()`, no `innerHTML` assignment with user-provided or AI-generated content.
10. **NEVER direct filesystem or network access from apps**: Apps use system service clients for all filesystem and network access. No direct `fetch()` calls from app code to external services, and no direct host filesystem access.

### 20.3 Spec Authoring Anti-Patterns

1. Do not use "TBD" or "TODO" in specs. Choose a concrete default and state it explicitly.
2. Do not use vague language ("should", "might", "could") for normative requirements. Use "must", "must not", "shall", or declarative statements.
3. Do not compress critical edge cases into summary statements. Enumerate them explicitly.
4. Do not define behavior in a scope owned by another spec without a cross-reference.
5. Do not leave acceptance criteria subjective. All criteria must be binary (pass/fail).

---

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership

This master spec owns:
- Product definition and goals.
- System-wide engineering assumptions.
- The spec-authoring contract and conflict resolution rules.
- Coding-agent guardrails.
- Subsystem dependency order.
- Cross-cutting invariants.
- The complete child spec index.

This master spec does NOT own:
- Any individual subsystem's implementation details.
- Any API endpoint, data model, or interface definition beyond the cross-cutting types in section 8.

### 21.2 Complete Child Spec Index

The following table lists every child spec by number and title. Each child spec is authoritative within its declared scope.

| Number | Title | Owner Crate / Directory |
|--------|-------|------------------------|
| 01 | Repository, Toolchain, Engineering Conventions | Repository root |
| 02 | Core Architecture and Runtime Boundaries | `cortex-core`, `cortex-db`, `cortex-api` |
| 03 | Identity, Authentication, Sessions, User Profiles | `cortex-auth` |
| 04 | Permissions, Policy, Trust Model | `cortex-policy` |
| 05 | Settings Service and Settings App | `cortex-settings`, `apps/settings-app` |
| 06 | AI Runtime, Provider Registry, Preferred LLM, Model Routing | `cortex-ai` |
| 07 | Desktop Shell | `apps/desktop-shell` |
| 08 | Window Manager | `cortex-runtime` (window management portion) |
| 09 | App Runtime and App Lifecycle | `cortex-runtime` |
| 10 | System Command Bus and Event Model | `cortex-core` (bus portion) |
| 11 | Virtual Filesystem and Storage Abstraction | `cortex-files` |
| 12 | Search, Indexing, Global Command Palette | `cortex-search` |
| 13 | Notifications Service | `cortex-notify` |
| 14 | Observability, Logging, Telemetry | `cortex-observability` |
| 15 | Accessibility, Input, Keyboard System | Cross-cutting (UI layer) |
| 16 | Theme, Design Tokens, UI System | `@cortexos/ui-components`, `@cortexos/theme` |
| 17 | First-Party Core Apps (Parent) | -- |
| 17a | Calculator App | `apps/calculator-app` |
| 17b | Text Editor App | `apps/text-editor-app` |
| 17c | Notes App | `apps/notes-app` |
| 17d | File Manager App | `apps/file-manager-app` |
| 17e | Media Viewer App | `apps/media-viewer-app` |
| 17f | Terminal-lite App | `apps/terminal-lite-app` |
| 17g | Clock and Utility Apps | `apps/clock-utils-app` |
| 18 | Games Platform (Parent) | -- |
| 18a | Solitaire | `apps/games/solitaire` |
| 18b | Minesweeper | `apps/games/minesweeper` |
| 18c | Snake | `apps/games/snake` |
| 18d | Tetris-like Puzzle Game | `apps/games/tetris` |
| 18e | Chess or Checkers | `apps/games/chess` |
| 19 | AI System Surfaces and UX | Frontend AI client layer |
| 20 | AI Action Permissions and Safety Controls | `cortex-policy` (AI portion), `cortex-ai` |
| 21 | SDK, Manifest, Third-Party App Platform | `cortex-sdk` |
| 22 | Admin, Diagnostics, Recovery | `cortex-admin` |
| 23 | Release Readiness, QA, Acceptance Framework | `xtask` / CI |
| A | Required AI Settings Fields | Appendix (referenced by specs 05, 06) |
| B | Minimum First-Party App List | Appendix (referenced by spec 17) |
| C | Definition of Done | Appendix (referenced by spec 23) |
| D | Coding Agent Guardrails | Appendix (referenced by this master spec) |

### 21.3 Completion Criteria for Each Subsystem Spec

A subsystem spec is considered complete when:

1. All 21 mandatory sections are present and populated with concrete, deterministic content.
2. Scope explicitly declares what the subsystem owns and what it does not own.
3. Data models include concrete type definitions (Rust structs, TypeScript interfaces, or JSON schemas).
4. Public interfaces are defined with exact signatures (function signatures, API routes, event schemas).
5. Internal interfaces are defined for testability and modularity.
6. State management specifies which state is ephemeral, session-persisted, and permanently persisted, and where each category is stored.
7. Failure modes are enumerated with specific behavior for each mode (detection, user-visible behavior, recovery, logging).
8. Security and permissions requirements are explicit and enforceable.
9. Performance requirements are measurable with specific targets and measurement methods.
10. Accessibility requirements reference WCAG 2.1 AA as the baseline.
11. Observability and logging requirements specify which events are logged, at what level, with what fields, and what data is excluded.
12. Testing requirements specify unit, integration, and E2E test categories with concrete test cases.
13. Acceptance criteria are binary (pass/fail) checklists.
14. Build order and dependencies reference prerequisite specs by number.
15. Non-goals and anti-patterns are explicitly stated.
16. Implementation instructions include: subsystem ownership reminder, recommended crate/package structure, what can be stubbed, what must be real in v1, what cannot be inferred, stop conditions, and testing gates.

### 21.4 What Can Be Stubbed Initially (Master Spec Level)

At the master spec level, the following child specs can be deferred during early implementation:

- Layer 12 (Games): Games are not required for the core OS to function.
- Layer 14 (SDK): Third-party app support is not required for the OS to function.
- Layer 15 (Admin): Admin tools aggregate data from other subsystems and can be built last.

All other layers should be implemented in order.

### 21.5 What Must Be Real in v1

Every spec in the index must have a complete document. Every spec from Layer 0 through Layer 11 must have a working implementation. Layers 12 through 16 must have at minimum working implementations for the items marked as P0 in their respective specs.

### 21.6 What Cannot Be Inferred

1. The exact build layer assignment for each spec is a deliberate choice. Do not reorder layers without updating this master spec.
2. The coding-agent guardrails (section 20.2) are non-negotiable. Do not relax them.
3. The conflict resolution rules (section 10.2) are authoritative. Do not introduce ambiguity.
4. The AI provider enum (`AiProvider`) is the canonical list. Do not add providers without updating specs 06 and A.
5. The cross-cutting invariants (section 6.5) apply everywhere. Do not create exceptions without updating this master spec.

### 21.7 Stop Conditions

The master spec is considered complete when:

1. All 21 sections are present and populated.
2. Every child spec in the index (01 through 23, 17a through 17g, 18a through 18e, appendices A through D) is referenced by number and title.
3. The build layer order (section 19) covers every child spec exactly once.
4. Completion criteria (section 21.3) are defined and applicable to every child spec.
5. Guardrails (section 20.2) cover every identified anti-pattern.
6. No TBD, no vague language, no aspirational statements, no missing sections.

### 21.8 Testing Gates

The master spec itself is tested by verifying:

1. Every child spec file exists in the repository at the expected path (`docs/specs/`).
2. Every child spec file contains all 21 mandatory sections.
3. No two child specs claim ownership of the same subsystem.
4. The build layer graph is acyclic (no layer depends on a higher-numbered layer).
5. Every crate in section 7.2 is assigned to exactly one spec.
6. Cross-references between specs are valid (referenced spec numbers exist).
