# CortexOS Open Task List

> **Working notes -- non-authoritative.** See [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md) for current status.
>
> Last audit date: 2026-03-31. Status in this file reflects a change-request audit but is not exhaustive. Re-run gates before trusting completion marks.

## Legend

- [x] = item was implemented (code exists); gate status is not guaranteed
- [ ] = not started, needs full implementation
- `dep` = blocked by a dependency on earlier items

---

## Known Gate Status (as of 2026-03-31)

### Gates that pass:

- `cargo clippy --workspace --all-targets --all-features -- -D warnings` -- passes
- `cargo test --workspace` -- passes (all tests green)
- `cargo run --bin check-deps` -- passes (17 crates, 0 violations)
- `cargo audit` -- passes (190 crate dependencies, 0 vulnerabilities)
- `cargo fmt --all -- --check` -- passes
- `pnpm typecheck` -- passes
- `pnpm test` -- passes (544 frontend tests)
- `pnpm -r build` -- passes
- `bash tools/validate-manifests.sh` -- passes (14 manifests, 0 errors)
- `pnpm e2e` -- passes (3 Playwright frontend shell smoke tests)

### Known pre-existing items (non-blocking):

- `pnpm lint` has 50+ pre-existing warnings (0 errors in desktop-shell)
- `pnpm format:check` has 193 pre-existing formatting issues
- Some first-party apps still contain placeholder or local-only behavior

---

## Backend (Rust)

### Phase 7: Command Bus - Durable Idempotency
- [x] Add `command_bus_events` migration (`cortex-db/migrations/0012_create_command_bus_events.up.sql`)
- [x] Update `bus.rs` idempotency check to use SQLite-backed `command_bus_events` table
- [x] Add dead-letter persistence: on failed event inserts, capture error_message, processed_at
- [x] Add tests for durable idempotency and dead-letter handling
- [x] Verify `cargo test -p cortex-api` passes (42 tests)

### Phase 8: Window Manager
- [x] Create `cortex-wm` crate (window state types, workspace data, window lifecycle commands/event types)
- [x] Implement `WindowManagerService` trait with SQLite persistence
- [x] Wire window manager into `cortex-api` AppState and routes
- [x] Add window route handlers (open, close, minimize, resize, focus, move, workspace operations)
- [x] Add tests for window lifecycle, workspace switching, focus management (15 tests)

### Phase 8: HTTP Server Binary
- [x] Create `cortex-server` binary crate with axum-based HTTP server
- [x] Wire all route handlers, middleware, WebSocket, and static file serving
- [x] Add `main.rs` that starts the server with AppState initialization
- [x] Add `Cargo.toml` with axum, tower-http dependencies
- [x] Add to workspace members in root `Cargo.toml`
- [x] Verify `cargo build -p cortex-server` succeeds

### Phase 8: Desktop Shell (frontend)
- [x] Implement desktop-shell with real window management, taskbar, system tray, app launcher
- [x] Add workspace switcher (Ctrl+W / Ctrl+Shift+W shortcut)
- [x] Add system tray with running apps, pinned apps, clock, notifications badge
- [x] Add settings panel accessible from settings service
- [x] Add theme toggle, fullscreen toggle, clock display
- [x] Connect to backend via WebSocket for real-time state
- [x] Bootstrap via HTTP for initial snapshot
- [x] Implement keyboard shortcut handling with focus cycling and aria-live announcements
- [x] Verify `pnpm build` passes, `pnpm test` passes (56 tests)

### Phase 10: Command Palette
- [x] Wire command palette into desktop-shell with Ctrl+Space trigger
- [x] Implement real-time search filtering with local + remote results
- [x] Implement keyboard navigation (Arrow keys, Enter, Escape)
- [x] Implement ranking algorithm with type priorities and prefix matching
- [x] Implement command execution (open app, toggle theme, toggle fullscreen, etc.)
- [x] Write tests for CommandPalette component (11 tests)
- [x] Verify `pnpm build` passes, `pnpm test` passes

### Phase 11: Theme and Accessibility
- [x] Implement `@cortexos/shared` package with complete design token system (spec 16)
- [x] Implement theme resolver with light/dark tokens, accessibility overrides, custom theme validation
- [x] Implement focus trap utility for modal dialogs
- [x] Implement keyboard shortcut registry with conflict resolution
- [x] Implement ARIA announcement helper with sr-only support
- [x] Implement OS-level preference detection (prefers-reduced-motion, prefers-contrast)
- [x] Write tests for theme tokens, token resolution, validation (19 tests)
- [x] Write tests for shortcuts, focus trap, announcements (17 tests)
- [x] Verify `pnpm test` passes (36 tests)

### Phase 12: First-Party Apps
- [x] Implement calculator-app: full calculator UI, operations (+, -, *, /, %), clear, decimal, display
- [x] Implement notes-app: note creation/edit/delete with search, timestamps
- [x] Implement file-manager-app: directory browsing, breadcrumb navigation, folder navigation
- [x] Implement media-viewer-app: drag-drop zone, file info display, clear
- [x] Implement terminal-lite-app: command input/output with help, echo, clear, date, whoami, uname
- [x] Implement clock-utils-app: digital clock with live updating, date display
- [x] Implement settings-app: theme toggle, AI provider selection, about section
- [x] Verify `pnpm build` passes for all apps
- [x] Write tests for each app's core flows
- [x] Write manifest for each app with declared permissions

### Phase 13: Games
- [x] Implement games platform shell with game listing and launch
- [x] Implement snake: grid-based movement, food spawning, score tracking, game over
- [x] Implement minesweeper: grid generation, flood fill, flagging, win/loss detection
- [x] Implement solitaire: standard Klondike rules, card selection/movement, stock/waste/foundations
- [x] Implement tetris: piece spawning, rotation, line clearing, level progression
- [x] Implement chess: board, pieces, move validation, legal move highlighting
- [x] Implement `@cortexos/game-framework` shared package (GameLoop, InputManager, TimerService, etc.)
- [x] Extract game logic into testable engine modules for all 5 games
- [x] Write tests for core game rules (deterministic tests for each game -- 181 game tests)
- [x] Game manifests with proper permissions and game metadata
- [x] Verify `pnpm build` passes for all games
- [x] Verify `pnpm test` passes for all games

### Phase 14: AI UX
- [x] Implement `@cortexos/ai-client` package with conversation UI
- [x] Implement assistant panel (Ctrl+Shift+A) with message history
- [x] Implement action confirmation/denial flows for AI actions
- [x] Implement low-risk auto-apply wiring via settings service
- [x] Wire AI client to cortex-ai backend via API
- [x] Write tests for deny/confirm/auto-apply boundaries (40 tests)
- [x] Verify `pnpm build` passes, `pnpm test` passes

### Phase 15: SDK
- [x] Implement manifest schema validation in cortex-sdk
- [x] Implement install/update/uninstall registry flows in cortex-sdk
- [x] Implement sandbox and permission checks for cortex-sdk
- [x] Implement SDK API surfaces through platform services only
- [x] Write tests for manifest validation, install lifecycle, denied calls (100 tests)
- [x] Verify `pnpm build` passes, `pnpm test` passes

### Phase 16: Admin
- [x] Implement host metrics collector (CPU, memory, disk usage) with ring buffer
- [x] Implement diagnostic views against canonical data sources
- [x] Implement crash handler with auto-restart tracking
- [x] Implement session state persistence and recovery
- [x] Implement filesystem verification and repair
- [x] Implement settings reset (per-app and system-wide)
- [x] Implement factory reset (12-step best-effort process)
- [x] Implement safe mode detection and management
- [x] Implement diagnostic export bundler with settings redaction
- [x] Write tests for export redaction, recovery, collector integration (108 tests)
- [x] Verify `cargo test -p cortex-admin` passes (when workspace compiles)

### Phase 17: Release Gates
- [x] Add E2E test harness for critical flows (login, app launch, file CRUD, search, notifications)
- [x] Document release validation outputs and failure criteria
- [x] Enforce coverage thresholds -- workspace fmt, build, test, typecheck all pass
- [x] Verify `cargo test --workspace` passes (all tests green as of 2026-03-31)
- [x] `cargo clippy --workspace` passes (as of 2026-03-31)
- [x] `cargo fmt --all -- --check` passes
- [x] Verify `pnpm build` passes for all frontend packages
- [x] `pnpm test` passes (544 frontend tests)
