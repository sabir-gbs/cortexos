# Implementation Kickoff

> **Note (2026-03-31):** Implementation has begun. The repo now contains a Rust workspace (17 crates), pnpm workspace (desktop shell + apps, 3 shared packages), and CI. This document is retained for its scaffold order and phase logic. For current status, see [DOCUMENTATION_INDEX.md](/home/sabir/projects/cortexos/DOCUMENTATION_INDEX.md) and [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md).

This document converts the reconciled spec set into a practical scaffold order and execution checklist for Claude Code and GLM-5.1.

It was written when the repository was still spec-only:

- docs existed
- the target Rust and TypeScript workspaces now exist (17 crates, desktop shell, and first-party apps). See change_requests/INDEX.md for current status.
- implementation was to follow the specs, not invent missing infrastructure

The workspaces now exist. The phase order and gate criteria below remain valid as a reference.

Primary inputs:

- [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md)
- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)
- [appendix_d_coding_agent_guardrails.md](/home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md)
- [spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md)

## 1. Operating Rules For Claude Code + GLM-5.1

Before any implementation batch:

- read the owning spec and all directly referenced appendices
- treat child specs as authoritative inside their scope
- stop if a change would alter a cross-subsystem contract without a spec edit
- do not claim a workspace, CI, migrations, scripts, or apps exist until they are created
- do not skip server-side auth, policy, or typed bus contracts for speed

Execution mode:

- Claude Code should do scaffolding, edits, tests, and integration wiring
- GLM-5.1 should be used against the same spec contracts for implementation assistance, code generation, and review prompts
- both agents must follow the same build order and stop conditions

## 2. Non-Negotiable Gates

No phase is complete unless all applicable items pass:

- spec behavior implemented, not just rendered
- errors and edge cases handled
- required tests added
- observability added
- accessibility covered for UI work
- no placeholder core logic
- no untyped command/event payloads
- no client-side auth or authorization as source of truth

Use [appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md) as the binary pass/fail gate.

## 3. Scaffold Order

Follow this order unless the specs are updated:

1. Repository bootstrap
2. Rust foundation crates
3. Persistence and observability
4. Identity and policy
5. Settings and filesystem
6. AI runtime
7. Command bus
8. Window manager and shell
9. App runtime
10. Notifications, search, observability consumers
11. Theme and accessibility
12. First-party apps
13. Games
14. AI UX and AI safety UX integration
15. SDK
16. Admin tools
17. Release gates and CI hardening

This order is derived from [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md) and [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md).

## 4. Phase-by-Phase Kickoff Checklist

### Phase 0: Freeze The Contracts

Entry criteria:

- use the reconciled specs as the baseline
- do not start implementation from older audit findings

Checklist:

- [ ] Treat [spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md) as current
- [ ] Freeze platform contracts in specs `00`, `01`, `05`, `06`, `08`, `10`, `14`, `19`, `20`, `21`, `22`
- [ ] Decide whether the low-priority Zhipu/GLM runtime example is needed before implementation

Output:

- stable spec baseline for scaffolding

### Phase 1: Repository Bootstrap

Owning spec:

- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)

Create first:

- `Cargo.toml`
- `rust-toolchain.toml`
- `package.json`
- `pnpm-workspace.yaml`
- `.node-version`
- `.gitignore`
- `.github/workflows/ci.yml`
- `crates/`
- `apps/`
- `tests/`
- `tools/`
- `scripts/`

Checklist:

- [ ] root Cargo workspace exists with planned members
- [ ] root pnpm workspace exists
- [ ] directory skeleton matches spec 01
- [ ] setup and dev scripts exist
- [ ] CI file exists with placeholder but valid stages
- [ ] baseline validation tools exist

Must be real:

- workspace manifests
- pinned toolchain files
- basic scripts
- baseline CI definition

May stay stubbed:

- most subsystem code
- app internals
- non-critical CI jobs beyond syntax/build/test skeletons

Exit criteria:

- `cargo build` succeeds on empty scaffold crates
- workspace-level frontend install/config is valid

### Phase 2: Rust Foundation

Owning specs:

- [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md)
- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [02_core_architecture_and_runtime_boundaries.md](/home/sabir/projects/cortexos/docs/specs/02_core_architecture_and_runtime_boundaries.md)

Create in this order:

1. `cortex-core`
2. `cortex-config`
3. `cortex-sdk` types only

Checklist:

- [ ] core IDs, shared errors, and base traits exist
- [ ] config loading exists with deterministic precedence
- [ ] crate boundaries match spec ownership
- [ ] no circular internal dependencies

Must be real:

- shared error taxonomy
- typed shared models used by later crates
- config loading contract

May stay stubbed:

- feature-specific implementations behind traits

Exit criteria:

- later service crates can depend on these without refactor pressure

### Phase 3: Persistence And Observability

Create in this order:

1. `cortex-db`
2. `cortex-observability`

Checklist:

- [ ] baseline SQLite migration exists
- [ ] DB pool and migration runner exist
- [ ] structured logging setup exists
- [ ] searchable log mirror contract exists
- [ ] audit and retention primitives exist

Must be real:

- SQLite-backed canonical state primitives
- migration runner
- structured log ingestion

May stay stubbed:

- advanced dashboards
- long-tail admin queries

Exit criteria:

- downstream crates can persist authoritative state without inventing local storage

### Phase 4: Identity And Policy

Create in this order:

1. `cortex-auth`
2. `cortex-policy`

Checklist:

- [ ] session model uses server-authoritative cookies
- [ ] permission checks are server-side
- [ ] policy hooks exist for files, settings, AI actions, SDK calls
- [ ] audit trail integration exists for security-sensitive actions

Must be real:

- authentication boundaries
- permission enforcement entry points
- denial/error types

May stay stubbed:

- less critical profile UX
- non-core permission-admin UI

Exit criteria:

- no subsequent subsystem needs to bypass auth or policy to make progress

### Phase 5: Settings And Filesystem

Create in parallel if useful:

1. `cortex-settings`
2. `cortex-files`

Checklist:

- [ ] settings schema includes full AI registry from appendix A
- [ ] settings API and `settings.changed` flow are implemented
- [ ] filesystem metadata/content split matches reconciled storage model
- [ ] no app gets direct host filesystem access

Must be real:

- canonical settings read/write path
- namespace validation
- filesystem service boundaries

May stay stubbed:

- non-critical settings UI polish
- import/export conveniences

Exit criteria:

- AI runtime and apps can read settings and files through stable service APIs

### Phase 6: AI Runtime

Create:

1. `cortex-ai`

Checklist:

- [ ] provider registry exists
- [ ] routing precedence matches spec 06
- [ ] `NoProviderConfigured` path works deterministically
- [ ] provider adapters are isolated from core routing logic
- [ ] Zhipu is supported at the provider enum/settings level
- [ ] audit logging and budget hooks exist

Must be real:

- routing engine
- adapter trait
- timeout model
- fallback behavior

May stay stubbed:

- all provider adapters beyond the first one or two
- non-critical model metadata enrichment

Exit criteria:

- AI requests can execute or fail deterministically without UI-specific assumptions

### Phase 7: Command Bus

Create:

1. typed command/event bus in `cortex-core` or its owned bus module

Checklist:

- [ ] typed Rust command/event APIs exist
- [ ] typed TypeScript client contract exists
- [ ] durable idempotency records exist
- [ ] dead-letter persistence exists
- [ ] event names follow canonical registry

Must be real:

- typed schemas
- command dispatch
- event publish/subscribe
- durable idempotency path

May stay stubbed:

- some lower-priority event producers

Exit criteria:

- realtime state changes no longer need ad hoc transport logic

### Phase 8: Window Manager And Shell

Create in this order:

1. `cortex-runtime` window-management slice
2. `apps/desktop-shell`

Checklist:

- [ ] window mutations go through bus commands
- [ ] HTTP is limited to bootstrap/snapshot reads where specified
- [ ] shell subscribes to canonical events
- [ ] keyboard shortcuts match reconciled values

Must be real:

- window state model
- shell/window event integration
- taskbar/dock basics

May stay stubbed:

- richer visual polish
- advanced workspace UX details

Exit criteria:

- a minimal desktop session can render and react to window lifecycle events

### Phase 9: App Runtime

Create:

1. remaining `cortex-runtime` app lifecycle pieces
2. `cortex-api` integration wiring begins

Checklist:

- [ ] app launch/stop/crash state exists
- [ ] manifest loading works for first-party apps
- [ ] lifecycle events are emitted
- [ ] runtime never grants hidden first-party privileges

Exit criteria:

- the platform can host first-party apps through the same declared runtime path

### Phase 10: Core System Services

Create in parallel:

1. `cortex-search`
2. `cortex-notify`
3. observability consumers in API/admin paths as needed

Checklist:

- [ ] search index persistence matches reconciled storage rules
- [ ] command palette shortcut is `Ctrl+Space`
- [ ] notification events use canonical names
- [ ] admin/log queries depend on observability service rather than bespoke storage

### Phase 11: Theme And Accessibility

Create:

1. `@cortexos/theme`
2. `@cortexos/ui-components`
3. accessibility wiring

Checklist:

- [ ] token system exists
- [ ] accessibility settings are wired through the settings service
- [ ] keyboard/focus/ARIA patterns exist in shared components

Exit criteria:

- first-party apps do not each invent accessibility and theme behavior

### Phase 12: First-Party Apps

Recommended order from the specs:

1. `clock-utils-app`
2. `calculator-app`
3. `terminal-lite-app`
4. `media-viewer-app`
5. `notes-app`
6. `file-manager-app`
7. `text-editor-app`

Checklist for each app:

- [ ] manifest exists
- [ ] declared permissions are explicit
- [ ] app uses service APIs, not direct host/network access
- [ ] tests cover app-specific core flows

### Phase 13: Games

Apps may be built in any order, but keep the parent platform contract first:

1. games platform shell
2. solitaire
3. minesweeper
4. snake
5. tetris
6. chess

Special checks:

- [ ] Tetris uses the reconciled spawn/lock-delay/top-out rules
- [ ] Chess includes draw rules beyond stalemate/checkmate

### Phase 14: AI UX And Safety UX

Create in this order:

1. `@cortexos/ai-client`
2. AI surface flows from spec 19
3. AI action safety and confirmation UX from spec 20

Checklist:

- [ ] assistant shortcut is `Ctrl+Shift+A`
- [ ] conversation persistence is server-authoritative
- [ ] low-risk auto-apply uses the canonical setting key
- [ ] all risky actions still flow through `cortex-policy`

### Phase 15: SDK

Create:

1. `cortex-sdk` implementation

Checklist:

- [ ] manifest schema exists
- [ ] install/update/uninstall flow uses canonical storage and policy paths
- [ ] third-party apps have no hidden privileges
- [ ] SDK AI hooks use the reconciled AI client/runtime ownership model

### Phase 16: Admin

Create:

1. `cortex-admin`

Checklist:

- [ ] admin tools aggregate existing subsystem data rather than invent new stores
- [ ] host metrics come from the admin-owned collector
- [ ] logs and AI audit data query the observability-owned store
- [ ] recovery/session state uses SQLite-backed canonical storage

### Phase 17: Release Gates

Create:

1. `tests/e2e`
2. coverage/reporting flow
3. full CI hardening

Checklist:

- [ ] build, fmt, clippy, tests, coverage, audit, and E2E gates exist
- [ ] per-subsystem DoD can be proven
- [ ] release checklist aligns to spec 23

## 5. Recommended First PR Sequence

If you want implementation to start with minimal thrash, use this PR order:

1. repo bootstrap only
2. `cortex-core` + `cortex-config`
3. `cortex-db` + baseline migration
4. `cortex-observability`
5. `cortex-auth`
6. `cortex-policy`
7. `cortex-settings`
8. `cortex-files`
9. `cortex-ai`
10. typed command bus
11. `cortex-runtime`
12. `cortex-api`
13. `apps/desktop-shell` + settings app shell
14. search + notifications
15. theme + accessibility shared packages
16. first-party apps
17. AI client surfaces
18. SDK
19. admin
20. games
21. release gates and CI tightening

## 6. What Claude Code Should Verify On Every Batch

- [ ] which spec owns this change
- [ ] which lower-layer dependencies must exist first
- [ ] which event names, setting keys, and routes are canonical
- [ ] whether the state being added is canonical SQLite state or filesystem blob/export state
- [ ] whether tests belong at unit, integration, E2E, or all three
- [ ] whether the change introduces a new public contract that must be documented

## 7. What GLM-5.1 Should Be Prompted To Do

Use GLM-5.1 for bounded, spec-referenced tasks. Good prompts:

- implement spec section X in crate/package Y
- generate tests for the error paths and acceptance criteria in spec X
- review this diff against spec X and appendix C
- extract all required event names, setting keys, and invariants from spec X

Do not use it with underspecified prompts like:

- build the whole OS
- make the shell work however you think is best
- choose a storage model
- infer missing permission behavior

## 8. Immediate Next Step

If implementation starts now, the correct first task is:

1. scaffold Phase 1 exactly to spec 01
2. keep all service crates minimal but compilable
3. avoid feature logic until the workspace, toolchain, migrations, scripts, and CI skeleton are in place

> **Note:** Phase 1 (repository bootstrap) is already complete. These instructions are retained for reference and for fresh-clone scenarios. Check change_requests/INDEX.md for the current phase status.

That is the lowest-risk starting point for both Claude Code and GLM-5.1.
