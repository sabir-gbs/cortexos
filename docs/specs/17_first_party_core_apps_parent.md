# 17. First-Party Core Apps -- Parent Specification

## 1. Purpose

This specification defines the common contract, requirements, and conventions that all first-party CortexOS applications must follow. It serves as the authoritative parent spec for child specs 17a through 17g. Each child spec owns its specific application domain; this document owns the shared rules, patterns, and constraints that unify all first-party apps.

## 2. Scope

This spec covers:

- Definition of what constitutes a "first-party" application in CortexOS.
- App manifest requirements specific to first-party apps.
- Common behavioral requirements: theme support, accessibility, keyboard navigation, permissions, state management, file access, and AI integration.
- Shared UI component library usage and constraints.
- Build, packaging, and deployment conventions for first-party apps.
- The relationship between first-party apps and system services (cortex-policy, cortex-runtime, cortex-files, cortex-ai, cortex-settings, cortex-notify, cortex-search, cortex-observability).

In scope for this spec:
- apps/calculator-app
- apps/text-editor-app
- apps/notes-app
- apps/file-manager-app
- apps/media-viewer-app
- apps/terminal-lite-app
- apps/clock-utils-app

The Settings app (apps/settings-app) is covered by spec 05 and is not governed by this parent spec, although it follows the same first-party conventions where applicable.

## 3. Out of Scope

- Third-party app platform and SDK (spec 21).
- Games (spec 18 series).
- Desktop shell (spec 07).
- Window manager behavior (spec 08).
- Specific application logic, data models, and UI layouts of individual apps (covered by child specs 17a--17g).
- AI runtime internals (spec 06).
- Permission enforcement engine (spec 04).

## 4. Objectives

1. Define a uniform contract so every first-party app behaves consistently with respect to permissions, state, theming, accessibility, and AI integration.
2. Ensure first-party apps have no hidden privileges -- they use the same permission model as third-party apps.
3. Establish a shared UI component library that all first-party apps consume, guaranteeing visual and behavioral consistency.
4. Mandate that every first-party app exposes AI surfaces per spec 19 (AI System Surfaces and UX).
5. Define app manifest requirements that allow the app runtime (spec 09) to load, sandbox, and manage first-party apps deterministically.
6. Prevent ad-hoc patterns that would create coupling between first-party apps and internal system service implementation details.

## 5. User-Visible Behavior

### 5.1 Consistent UX Across First-Party Apps

All first-party apps must present:

- A window title bar managed by the window manager (spec 08), containing the app name and standard window controls (minimize, maximize, close).
- A menu bar or equivalent navigation surface with at minimum: File (if applicable), Edit (if applicable), View, Help, and an AI-assist entry point.
- Keyboard shortcut hints visible in menus.
- Consistent dialog styling (confirmation, alert, file picker) using the shared UI component library.
- Theme responsiveness: apps must render correctly in light, dark, and high-contrast themes without custom theme logic. Apps consume design tokens from spec 16.
- Consistent focus indicators and tab navigation order.

### 5.2 Launch Behavior

- First-party apps appear in the desktop shell app launcher and global command palette (spec 12).
- Each app launches as a windowed process via the app runtime (spec 09).
- Apps respect single-instance or multi-instance rules declared in their manifest (child specs declare which).

### 5.3 AI Integration Surfaces

Per spec 19, every first-party app must:

- Expose an app-level AI assistant surface accessible via a toolbar button or keyboard shortcut (default: `Ctrl+Shift+A`).
- Provide context to the AI runtime when the user activates the AI surface, including the current document/state, selected content, and app identity.
- Support AI-suggested actions relevant to the app domain (e.g., "Summarize this note" in Notes, "Explain this formula" in Calculator).
- Never execute AI-suggested destructive actions without explicit user confirmation.
- Display the model name/provider disclosure when AI is active, per spec 19.

## 6. System Behavior

### 6.1 Permission Model

First-party apps use cortex-policy (spec 04) identically to third-party apps:

- Apps declare required permissions in their manifest under `permissions.required` and `permissions.optional`.
- The system prompts the user for optional permissions at first use.
- Required permissions are granted at install time.
- First-party apps receive no implicit or hidden grants. If a first-party app needs filesystem access, it declares the `files.read` and/or `files.write` permission and the user grants or denies it.
- Permission denials are handled gracefully per section 12.

### 6.2 State Management

All first-party apps persist state through cortex-runtime (spec 09):

- App state is stored in the app's sandboxed storage namespace under `cortex-runtime://app-state/{app-id}/`.
- State includes: window geometry, user preferences (within the app), unsaved document drafts, recent file lists.
- State serialization format: JSON by default, with binary blob support via the storage API.
- Apps must not write state outside their sandboxed namespace.

### 6.3 File Access

First-party apps access files exclusively through cortex-files (spec 11):

- File open/save dialogs are system-provided via cortex-files.
- Apps never access filesystem paths directly; they use file handles returned by cortex-files.
- Temporary files are managed by cortex-files temp storage.
- File type associations are declared in manifest and registered with cortex-files.

### 6.4 Inter-App Communication

First-party apps communicate with each other only through:

- The system command bus (spec 10): typed events and commands.
- cortex-files: file handles and file associations (e.g., double-clicking a text file in File Manager opens Text Editor).
- cortex-search: content indexing for app-specific data.
- Clipboard: via system clipboard service (requires `clipboard.read`/`clipboard.write` permissions).

First-party apps must NOT:
- Import code from other first-party apps.
- Share memory or state outside the cortex-runtime sandbox.
- Make direct network requests bypassing cortex-runtime.

## 7. Architecture

### 7.1 Common Architecture Pattern

Each first-party app follows this architecture:

```
apps/{app-name}/
  Cargo.toml              # Rust crate for any backend logic (if needed)
  package.json            # Frontend package
  manifest.json           # CortexOS app manifest
  src/
    main.ts               # Entry point, registers app with runtime
    App.tsx               # Root component
    components/           # App-specific UI components
    hooks/                # App-specific React hooks
    services/             # App-specific business logic
    types.ts              # App-specific TypeScript types
    ai/
      context.ts          # AI context provider for this app
      actions.ts          # AI action definitions
  tests/
    unit/                 # Unit tests
    integration/          # Integration tests
```

### 7.2 Frontend Stack

- Framework: React (TypeScript).
- Shared UI library: `@cortexos/ui-components` (spec 16).
- State management: React context + useReducer for local state; cortex-runtime API calls for persisted state.
- Styling: CSS custom properties consuming design tokens from spec 16. No inline styles for themable properties.

### 7.3 Backend Logic

First-party apps that require server-side processing (e.g., media decoding, search indexing) implement a thin Rust crate that:

- Registers as a service with cortex-runtime.
- Exposes a typed API via the command bus.
- Performs processing in a sandboxed subprocess if needed.

Apps that are purely frontend (e.g., Calculator) may omit the Rust crate and use only a frontend package.

### 7.4 Dependency Graph

First-party apps depend on:
- `@cortexos/ui-components` (shared UI library)
- `@cortexos/runtime-client` (cortex-runtime frontend client)
- `@cortexos/files-client` (cortex-files frontend client)
- `@cortexos/ai-client` (AI surface client per spec 19)
- `@cortexos/theme` (design token consumer)

First-party apps must NOT depend on:
- Internal implementation details of system crates.
- Direct filesystem, network, or OS APIs.
- Third-party npm packages that duplicate system-provided functionality without explicit approval.

## 8. Data Model

### 8.1 First-Party App Manifest Schema

```typescript
interface FirstPartyAppManifest {
  // Core identity (same as third-party, per spec 09)
  id: string;                     // e.g., "com.cortexos.calculator"
  name: string;                   // Display name
  version: string;                // Semver
  description: string;

  // First-party marker
  firstParty: true;               // Always true for first-party apps
  bundled: true;                  // Always true -- shipped with OS

  // Entry points
  entry: {
    frontend: string;             // Path to frontend entry relative to app root
    backend?: string;             // Path to Rust crate entry (if applicable)
  };

  // Window defaults
  window: {
    defaultWidth: number;
    defaultHeight: number;
    minWidth: number;
    minHeight: number;
    resizable: boolean;
    singleInstance: boolean;      // true = only one window; false = multiple windows
  };

  // Permissions
  permissions: {
    required: string[];           // Permission IDs from spec 04
    optional: string[];           // Permission IDs from spec 04
  };

  // File associations
  fileAssociations?: {
    extension: string;
    mimeType: string;
    role: "viewer" | "editor" | "default";
  }[];

  // AI integration
  ai: {
    surfaces: {
      assistantPanel: boolean;    // Expose AI assistant panel
      contextMenu: boolean;       // Expose AI context menu on selection
      inlineSuggestions: boolean; // Support inline AI suggestions
    };
    contextProviders: string[];   // Named context providers this app exports
    actions: {
      id: string;
      label: string;
      description: string;
      confirmationRequired: boolean;
      destructive: boolean;
    }[];
  };

  // Accessibility
  accessibility: {
    highContrastSupport: true;
    screenReaderSupport: true;
    keyboardNavigation: true;
  };

  // Search integration
  search?: {
    indexableContent: boolean;
    contentProvider?: string;     // Named content provider for cortex-search
  };

  // Category for app launcher grouping
  category: "utilities" | "productivity" | "media" | "development" | "system";
}
```

### 8.2 App State Schema (Common)

```typescript
interface AppStateBase {
  windowGeometry: {
    x: number;
    y: number;
    width: number;
    height: number;
    maximized: boolean;
  };
  recentFiles: string[];          // File handle IDs, max 10
  preferences: Record<string, unknown>;
  lastOpened: string;             // ISO 8601 timestamp
}
```

## 9. Public Interfaces

### 9.1 App Lifecycle Hooks

Each first-party app must implement and export:

```typescript
interface CortexApp {
  mount(container: HTMLElement, runtime: CortexRuntimeClient): void;
  unmount(): Promise<void>;       // Must flush state before resolving
  getState(): AppStateBase;
  setState(state: Partial<AppStateBase>): void;
  onAIContext(): AIContextPayload; // Returns context for AI surfaces
}
```

### 9.2 System Service Interfaces Consumed

All first-party apps consume these public interfaces:

- `cortex-runtime.apps.state` -- Read/write app state.
- `cortex-files.files` -- File CRUD operations.
- `cortex-files.dialogs` -- System file open/save dialogs.
- `cortex-policy.permissions` -- Check and request permissions.
- `cortex-settings.values` -- Read user settings (read-only for most apps).
- `cortex-notify.send` -- Send notifications.
- `cortex-search.index` -- Index app content for global search.
- `cortex-ai.surface` -- AI integration hooks.
- `cortex-observability.log` -- Structured logging.

## 10. Internal Interfaces

### 10.1 Shared UI Component Library

`@cortexos/ui-components` provides:

- `Button`, `IconButton`, `ToggleButton`
- `TextInput`, `TextArea`, `SearchInput`
- `Dialog`, `AlertDialog`, `ConfirmDialog`
- `Menu`, `MenuItem`, `ContextMenu`
- `Toolbar`, `ToolbarButton`, `ToolbarSeparator`
- `TabBar`, `TabPanel`
- `Sidebar`, `SidebarItem`
- `Breadcrumb`
- `ListView`, `GridView`, `ListItem`
- `StatusBar`
- `Tooltip`
- `Slider`, `ProgressBar`, `Spinner`
- `SplitPane`
- `ThemeConsumer` -- Provides current theme tokens

All components accept standard ARIA props and forward refs.

### 10.2 Internal App Service Pattern

Apps with backend logic expose internal commands via the command bus:

```typescript
// Example: text-editor-app backend command
{
  bus: "app:com.cortexos.text-editor",
  command: "detect-encoding",
  payload: { fileHandleId: string },
  response: { encoding: string, confidence: number }
}
```

## 11. State Management

### 11.1 State Layers

Each first-party app manages three layers of state:

1. **Ephemeral state**: React component state. Lost on unmount. Used for transient UI state (hover, focus, dropdown open).
2. **Session state**: Persisted via cortex-runtime for the current session. Survives hot reload. Used for unsaved drafts, undo stacks, scroll position.
3. **Persistent state**: Persisted via cortex-runtime across sessions. Used for preferences, recent files, window geometry.

### 11.2 State Sync Rules

- Apps write persistent state on explicit user action (save, preference change) and on app unmount.
- Apps debounce session state writes to a maximum of one write per 2 seconds.
- Apps must not lose user data if the browser tab refreshes. Unsaved changes must be stored in session state and restored on remount.
- Conflict resolution: last-write-wins for app state. Apps with concurrent editing requirements must implement their own merge logic.

### 11.3 State Size Limits

- Per-app state quota: 50 MB (configurable by admin).
- Single state key value limit: 5 MB.
- Apps exceeding quota receive a quota error and must handle it per section 12.

## 12. Failure Modes and Error Handling

### 12.1 Common Failure Modes

| Failure | Detection | Recovery |
|---------|-----------|----------|
| System service unavailable | API call timeout (5s default) or error response | Display offline banner. Queue actions if possible. Disable affected features gracefully. |
| Permission denied | cortex-policy returns `PermissionDenied` | Show permission explanation dialog with link to Settings > Permissions. Do not crash. |
| File not found | cortex-files returns `FileNotFound` | Show error toast. Offer to create new file or browse. |
| Storage quota exceeded | cortex-runtime returns `QuotaExceeded` | Warn user. Offer to clear cached data. Do not silently discard data. |
| AI provider unavailable | cortex-ai returns `ProviderUnavailable` | Show status indicator in AI panel. Fall back to local-only features. |
| App crash | Runtime detects unhandled exception | Capture error. Show crash recovery dialog with option to restart. Preserve session state. |

### 12.2 Error Presentation Rules

- User-facing errors use AlertDialog from the shared UI library.
- Errors include: short description, suggested action, and a "Details" expandable with the error code.
- Errors are logged via cortex-observability at the appropriate level.
- Network-related errors show a persistent banner (not a modal) until resolved.
- Apps must never show raw stack traces or internal IDs to users.

### 12.3 Data Loss Prevention

- Before any destructive action (close with unsaved changes, delete file), apps must show a confirmation dialog.
- Confirmation dialogs must have the destructive action as a non-default button.
- Apps must support auto-save for draft content where applicable (child specs define cadence).

## 13. Security and Permissions

### 13.1 No Hidden Privileges

First-party apps follow the same permission enforcement as third-party apps:

- cortex-policy evaluates every permission check without special-casing app identity.
- First-party app manifests declare all required permissions explicitly.
- The system does not auto-grant permissions to first-party apps.
- Admin can revoke first-party app permissions identically to third-party.

### 13.2 Permission Declaration

Each child spec lists its app's required and optional permissions. The common baseline:

**Required (all first-party apps):**
- `runtime.state` -- Read/write own app state.
- `runtime.lifecycle` -- Access lifecycle hooks.

**Optional (declared per app):**
- `files.read` / `files.write` -- Filesystem access.
- `clipboard.read` / `clipboard.write` -- Clipboard access.
- `notifications.send` -- Send user notifications.
- `search.index` -- Index content for global search.
- `ai.context` -- Provide context to AI surfaces.
- `ai.invoke` -- Invoke AI actions.

### 13.3 Content Security

- First-party apps must not eval user input or render untrusted HTML without sanitization.
- File content is treated as untrusted input.
- AI-generated content is treated as untrusted input (displayed but not auto-executed).

## 14. Performance Requirements

> **Note:** Performance and functionality verification claims in this section (including "verified over 100 cycles in tests" and similar assertions) are aspirational targets. They should not be treated as passed until backed by test evidence in CI.

### 14.1 Startup

- App window must render first meaningful paint within 500 ms of launch signal.
- App must be fully interactive within 1 second of launch signal.
- App state restoration must not block initial render.

### 14.2 Runtime

- UI must maintain 60 fps during normal interaction (scrolling, typing, resizing).
- Operations on documents/files up to 10 MB must complete without frame drops.
- Background operations (save, index, AI inference) must not block the UI thread.

### 14.3 Memory

- Apps must not exceed 200 MB heap memory under normal usage.
- Apps must release memory when document/file is closed.
- Apps must not leak memory on repeated open/close cycles (verified over 100 cycles in tests).

### 14.4 Bundle Size

- Individual first-party app frontend bundle must not exceed 500 KB gzipped.
- Shared UI library is loaded once by the runtime and not counted against per-app bundle size.

## 15. Accessibility Requirements

All first-party apps must meet the following (WCAG 2.1 AA minimum):

- **Keyboard navigation**: All functionality accessible via keyboard. Tab order follows visual layout. Focus indicators visible and high-contrast.
- **Screen reader**: All interactive elements have ARIA labels. Dynamic content updates announced via ARIA live regions. Role, name, and value exposed for all controls.
- **High contrast**: Correct rendering in high-contrast theme. No information conveyed by color alone.
- **Text sizing**: UI must remain functional when browser text size is increased to 200%.
- **Focus management**: Focus moves logically on dialogs open/close, tab switches, and navigation. Focus trap in modal dialogs.
- **Reduced motion**: Respect `prefers-reduced-motion`. Disable animations when this setting is active.

## 16. Observability and Logging

### 16.1 Structured Logging

All first-party apps log through `cortex-observability.log` with structured entries:

```typescript
{
  app: string;          // App ID
  level: "debug" | "info" | "warn" | "error";
  event: string;        // Event name (e.g., "file.opened", "ai.action.invoked")
  payload: object;      // Event-specific data (no PII, no full file contents)
  timestamp: string;    // ISO 8601
  sessionId: string;    // App session ID
}
```

### 16.2 Required Log Events

Each first-party app must log:

- App launch and unmount.
- Permission grant/deny events.
- File open/save/close (metadata only: file handle ID, size, type -- never content).
- Error events with error code and recovery action taken.
- AI surface open/close and action invocation (action ID, not payload).

### 16.3 Telemetry

- Apps do not send telemetry independently. All telemetry goes through cortex-observability.
- Apps report performance metrics (startup time, operation latency) via cortex-observability metrics API.
- User-facing analytics (feature usage counts) are opt-in and collected by the runtime, not by individual apps.

## 17. Testing Requirements

### 17.1 Unit Tests

- Minimum 80% code coverage per first-party app.
- All exported functions and hooks must have unit tests.
- Edge cases: empty state, maximum data size, concurrent operations, permission denied paths.

### 17.2 Integration Tests

- App lifecycle: launch, interact, unmount, relaunch -- state must persist.
- Permission flow: required permissions granted at install, optional permissions prompted at use, denied permissions handled.
- File operations: open, edit, save, close -- file content integrity verified.
- AI integration: context provided, action invoked, confirmation shown, result displayed.
- Theme switch: app renders correctly in all three themes without reload.

### 17.3 Accessibility Tests

- Automated AX tree validation for all screens.
- Keyboard-only navigation test for all user flows.
- Screen reader announcement test for dynamic content.
- High-contrast rendering test.

### 17.4 Performance Tests

- Startup time measured and verified under 500 ms / 1 s thresholds.
- Memory leak test: open/close 100 cycles, heap growth under 10 MB.
- Large document test: 10 MB file operations without frame drops.

## 18. Acceptance Criteria

A first-party app is accepted when:

- [ ] Manifest validates against the FirstPartyAppManifest schema.
- [ ] All required permissions are declared and granted at install.
- [ ] App launches and renders within performance thresholds.
- [ ] Theme switching works across light, dark, and high-contrast without custom code.
- [ ] All interactive elements are keyboard-navigable and screen-reader accessible.
- [ ] AI surface is accessible and provides correct context.
- [ ] File operations go through cortex-files exclusively.
- [ ] State persists across sessions correctly.
- [ ] Error handling follows section 12 conventions.
- [ ] Logging follows section 16 conventions.
- [ ] Unit test coverage meets 80% threshold.
- [ ] Integration tests for lifecycle, permissions, files, AI, and themes all pass.
- [ ] Accessibility tests pass.
- [ ] Performance tests pass.
- [ ] No direct filesystem, network, or OS API access (all mediated by system services).
- [ ] No hidden privileges or permission bypasses.

## 19. Build Order and Dependencies
**Layer 11**. Depends on: 09 (app runtime), 16 (theme tokens)

### 19.1 Prerequisite Specs (must be implemented first)

1. Spec 02 -- Core Architecture and Runtime Boundaries
2. Spec 04 -- Permissions, Policy, Trust Model
3. Spec 08 -- Window Manager
4. Spec 09 -- App Runtime and App Lifecycle
5. Spec 10 -- System Command Bus and Event Model
6. Spec 11 -- Virtual Filesystem and Storage Abstraction
7. Spec 14 -- Observability, Logging, Telemetry
8. Spec 15 -- Accessibility, Input, Keyboard System
9. Spec 16 -- Theme, Design Tokens, UI System

### 19.2 Build Order Within First-Party Apps

Apps should be implemented in this order (simpler to more complex):

1. **17g -- Clock and Utility Apps**: Simplest UI, no file operations. Validates basic app lifecycle.
2. **17a -- Calculator**: No file operations, minimal state. Validates keyboard input, AI integration pattern.
3. **17f -- Terminal Lite**: No file writes, read-only command layer. Validates command bus integration.
4. **17e -- Media Viewer**: Read-only file access, media decoding. Validates cortex-files read path.
5. **17c -- Notes App**: Rich text editing, auto-save, search indexing. Validates write path, search integration.
6. **17d -- File Manager**: Complex file operations, drag-and-drop, navigation. Validates full cortex-files API.
7. **17b -- Text Editor**: Most complex: syntax highlighting, multi-tab, encoding detection. Validates performance at scale.

### 19.3 In-Repo Dependencies

```
apps/{app-name}/
  depends on:
    crates/cortex-runtime/       (Rust)
    crates/cortex-files/         (Rust)
    crates/cortex-policy/        (Rust)
    packages/ui-components/      (TypeScript)
    packages/runtime-client/     (TypeScript)
    packages/files-client/       (TypeScript)
    packages/ai-client/          (TypeScript)
    packages/theme/              (TypeScript)
```

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- First-party apps are not expected to replace full desktop applications (e.g., Text Editor is not VS Code, Media Viewer is not Photoshop).
- First-party apps are not a testbed for experimental UI patterns. Use the shared component library.
- First-party apps are not exempt from any system policy or permission requirement.

### 20.2 Anti-Patterns

- **Hidden privileges**: Never grant first-party apps permissions not declared in their manifest. Never skip permission checks for first-party apps.
- **Direct filesystem access**: Never read/write files outside cortex-files API.
- **Cross-app imports**: Never import code from another first-party app.
- **Inline theming**: Never hardcode colors, fonts, or spacing. Always consume design tokens.
- **Bypassing state sandbox**: Never read or write state outside the app's cortex-runtime namespace.
- **Custom dialogs**: Never build custom file pickers, permission dialogs, or confirmation dialogs. Use system-provided dialogs.
- **Eval or innerHTML**: Never eval user input or inject untrusted HTML.
- **Silent failure**: Never swallow errors without logging and user notification.
- **Provider-specific code**: Never write AI provider-specific code in an app. Use the abstract AI client.
- **Untyped events**: Never emit or consume untyped events on the command bus.

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership

- Each app is an independent subsystem with its own spec (17a--17g).
- This parent spec owns the shared contract, not any app-specific logic.
- Changes to the shared contract require updating this file and verifying all child specs remain compatible.

### 21.2 Recommended Crate/Package Structure

For each first-party app:

```
apps/{app-name}/
  manifest.json           # Validate against FirstPartyAppManifest schema
  src/                    # TypeScript frontend
  Cargo.toml              # Optional Rust backend (omit if purely frontend)
```

### 21.3 What Can Be Stubbed Initially

- AI action handlers can return placeholder responses in initial implementation.
- Search indexing can be a no-op until cortex-search is integrated.
- Performance optimization (virtualization, lazy loading) can be deferred until functional correctness is verified.

### 21.4 What Must Be Real in V1

- Manifest schema compliance.
- Permission declaration and enforcement (no bypasses).
- File access via cortex-files (no direct paths).
- State persistence via cortex-runtime.
- Theme support via design tokens (all three themes).
- Keyboard navigation for all primary user flows.
- AI surface exposure (panel opens, context is provided).
- Error handling per section 12.
- Logging per section 16.

### 21.5 What Cannot Be Inferred

- Exact permission set per app (defined in child specs).
- Single vs. multi-instance behavior (defined per app in child specs).
- File associations (defined per app in child specs).
- AI context shape (defined per app in child specs).
- Default window dimensions (defined per app in child specs).

### 21.6 Stop Conditions

A first-party app subsystem is done when:

1. All acceptance criteria in section 18 pass.
2. Child spec acceptance criteria pass.
3. `manifest.json` validates against FirstPartyAppManifest schema.
4. No linter warnings, no type errors.
5. All tests pass (unit, integration, accessibility, performance).
6. No direct imports of system crate internals.
7. Code review confirms no anti-patterns from section 20.

### 21.7 Testing Gates

- Pre-merge: unit tests and lint pass.
- Post-merge to main: integration tests and accessibility tests pass.
- Pre-release: performance tests pass.
- Manual QA: keyboard-only navigation, screen reader testing, theme switch testing.
