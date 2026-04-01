# Implementation Phases

This document is the implementation phase map for CortexOS.

It defines the full execution sequence that Claude Code and GLM-5.1 should follow.

## Current Status (2026-03-31)

Implementation is in progress across all phases. Current state:

- **Rust workspace**: 17 crates scaffolded in `crates/`. Compile errors exist (at minimum `cortex-runtime`); `cargo test --workspace` does not pass.
- **Frontend workspace**: Desktop shell + apps in `apps/`, 3 packages in `packages/`. `pnpm -r build` and `pnpm test` pass (544 tests).
- **CI**: `.github/workflows/ci.yml` exists.
- **Quality gates**: `cargo clippy --workspace` passes. Frontend builds and tests are green.
- **Outstanding issues**: See [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md) for the full audit.

The phase definitions, dependencies, and exit criteria below remain the authoritative reference for what "done" means per phase.

Primary references:

- [IMPLEMENTATION_KICKOFF.md](/home/sabir/projects/cortexos/docs/guides/IMPLEMENTATION_KICKOFF.md)
- [PHASE1_BOOTSTRAP_CHECKLIST.md](/home/sabir/projects/cortexos/docs/checklists/PHASE1_BOOTSTRAP_CHECKLIST.md)
- [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md)
- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)
- [appendix_d_coding_agent_guardrails.md](/home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md)

## Phase Count

There are 18 documented phases in the current implementation plan:

1. Phase 0: Freeze The Contracts
2. Phase 1: Repository Bootstrap
3. Phase 2: Rust Foundation
4. Phase 3: Persistence And Observability
5. Phase 4: Identity And Policy
6. Phase 5: Settings And Filesystem
7. Phase 6: AI Runtime
8. Phase 7: Command Bus
9. Phase 8: Window Manager And Shell
10. Phase 9: App Runtime
11. Phase 10: Core System Services
12. Phase 11: Theme And Accessibility
13. Phase 12: First-Party Apps
14. Phase 13: Games
15. Phase 14: AI UX And Safety UX
16. Phase 15: SDK
17. Phase 16: Admin
18. Phase 17: Release Gates

If you want to count only implementation phases after spec freeze, there are 17 implementation phases: Phases 1 through 17.

## Global Rules

These rules apply to every phase:

- the owning specs must be read before work starts
- no phase may bypass auth, policy, or typed command/event contracts
- server-side state remains authoritative
- SQLite is canonical for structured persistent state unless the specs explicitly assign filesystem/blob storage
- every phase must satisfy the applicable Definition of Done before it is called complete
- no later phase should repair a foundational contract that should have been settled earlier

## Phase 0: Freeze The Contracts

Purpose:

- establish the reconciled spec baseline
- prevent implementation from starting against stale contradictions

Primary specs:

- [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md)
- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md)

Inputs:

- reconciled platform contracts
- refreshed audit report

Outputs:

- stable implementation baseline
- explicit decision on whether any final low-priority doc polish is required before coding starts

Exit criteria:

- no unresolved critical, high, or medium doc conflicts
- spec owners agree the current docs are the implementation source of truth

## Phase 1: Repository Bootstrap

Purpose:

- create the monorepo skeleton required by spec 01

Primary specs:

- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)

Inputs:

- frozen repository structure and toolchain decisions

Outputs:

- root workspace files
- Cargo workspace
- pnpm workspace
- crate/app directories
- tools/scripts/CI skeleton

Dependencies:

- Phase 0 complete

Exit criteria:

- workspace scaffolding exists
- bootstrap validation gates pass

Reference:

- [PHASE1_BOOTSTRAP_CHECKLIST.md](/home/sabir/projects/cortexos/docs/checklists/PHASE1_BOOTSTRAP_CHECKLIST.md)

## Phase 2: Rust Foundation

Purpose:

- define the shared backend primitives the rest of the system depends on

Primary specs:

- [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md)
- [02_core_architecture_and_runtime_boundaries.md](/home/sabir/projects/cortexos/docs/specs/02_core_architecture_and_runtime_boundaries.md)

Primary crates:

- `cortex-core`
- `cortex-config`
- foundational `cortex-sdk` types

Outputs:

- shared IDs, errors, base traits, shared model types
- config loading and config precedence rules

Dependencies:

- Phase 1

Exit criteria:

- later service crates can depend on foundation crates without redefining core types

## Phase 3: Persistence And Observability

Purpose:

- establish canonical persistence and logging/audit plumbing

Primary specs:

- [11_virtual_filesystem_and_storage_abstraction.md](/home/sabir/projects/cortexos/docs/specs/11_virtual_filesystem_and_storage_abstraction.md)
- [14_observability_logging_telemetry.md](/home/sabir/projects/cortexos/docs/specs/14_observability_logging_telemetry.md)

Primary crates:

- `cortex-db`
- `cortex-observability`

Outputs:

- migration runner
- SQLite pool and persistence layer
- structured logging
- searchable log mirror contract
- audit persistence primitives

Dependencies:

- Phase 2

Exit criteria:

- structured state has one canonical persistence path
- downstream subsystems do not need local ad hoc storage

## Phase 4: Identity And Policy

Purpose:

- establish authentication, sessions, and authorization boundaries

Primary specs:

- [03_identity_authentication_sessions_user_profiles.md](/home/sabir/projects/cortexos/docs/specs/03_identity_authentication_sessions_user_profiles.md)
- [04_permissions_policy_trust_model.md](/home/sabir/projects/cortexos/docs/specs/04_permissions_policy_trust_model.md)

Primary crates:

- `cortex-auth`
- `cortex-policy`

Outputs:

- session model
- auth enforcement points
- policy engine interfaces
- audit hooks for permission-sensitive actions

Dependencies:

- Phase 3

Exit criteria:

- no later subsystem needs to invent its own auth or authorization logic

## Phase 5: Settings And Filesystem

Purpose:

- provide canonical settings and file access services

Primary specs:

- [05_settings_service_and_settings_app.md](/home/sabir/projects/cortexos/docs/specs/05_settings_service_and_settings_app.md)
- [11_virtual_filesystem_and_storage_abstraction.md](/home/sabir/projects/cortexos/docs/specs/11_virtual_filesystem_and_storage_abstraction.md)
- [appendix_a_required_ai_settings_fields.md](/home/sabir/projects/cortexos/docs/specs/appendix_a_required_ai_settings_fields.md)

Primary crates:

- `cortex-settings`
- `cortex-files`

Outputs:

- settings read/write path
- namespace and validation rules
- canonical AI settings registry usage
- filesystem metadata/content storage split

Dependencies:

- Phase 4

Exit criteria:

- apps and services can consume settings and files only through sanctioned service paths

## Phase 6: AI Runtime

Purpose:

- build the runtime AI layer before any user-facing AI UX

Primary specs:

- [06_ai_runtime_provider_registry_preferred_llm_model_routing.md](/home/sabir/projects/cortexos/docs/specs/06_ai_runtime_provider_registry_preferred_llm_model_routing.md)
- [appendix_a_required_ai_settings_fields.md](/home/sabir/projects/cortexos/docs/specs/appendix_a_required_ai_settings_fields.md)

Primary crate:

- `cortex-ai`

Outputs:

- provider adapter trait
- deterministic routing
- fallback behavior
- timeout model
- budget hooks
- audit hooks
- no-provider-configured path

Dependencies:

- Phase 5

Exit criteria:

- AI requests can route deterministically without any UI assumptions

## Phase 7: Command Bus

Purpose:

- establish the typed realtime contract used across the platform

Primary specs:

- [10_system_command_bus_and_event_model.md](/home/sabir/projects/cortexos/docs/specs/10_system_command_bus_and_event_model.md)
- [02_core_architecture_and_runtime_boundaries.md](/home/sabir/projects/cortexos/docs/specs/02_core_architecture_and_runtime_boundaries.md)

Primary ownership:

- typed bus implementation in the backend foundation layer

Outputs:

- typed commands
- typed events
- durable idempotency records
- dead-letter path
- canonical event naming

Dependencies:

- Phase 6

Exit criteria:

- realtime commands/events no longer require ad hoc transport behavior

## Phase 8: Window Manager And Shell

Purpose:

- establish the browser-rendered desktop surface on top of the command bus

Primary specs:

- [07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md)
- [08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md)

Primary components:

- `cortex-runtime` window management
- `apps/desktop-shell`

Outputs:

- shell/window state model
- window lifecycle commands and events
- bootstrap snapshot reads
- canonical shortcut behavior

Dependencies:

- Phase 7

Exit criteria:

- a minimal interactive desktop session can render and respond to window changes

## Phase 9: App Runtime

Purpose:

- host first-party apps on a stable lifecycle/runtime layer

Primary specs:

- [09_app_runtime_and_app_lifecycle.md](/home/sabir/projects/cortexos/docs/specs/09_app_runtime_and_app_lifecycle.md)

Primary components:

- `cortex-runtime`
- `cortex-api` integration wiring

Outputs:

- app launch/stop/crash lifecycle
- manifest loading
- runtime event emission
- lifecycle state model

Dependencies:

- Phase 8

Exit criteria:

- apps can be launched through the platform without privileged shortcuts

## Phase 10: Core System Services

Purpose:

- implement the independent services used broadly across apps and shell

Primary specs:

- [12_search_indexing_global_command_palette.md](/home/sabir/projects/cortexos/docs/specs/12_search_indexing_global_command_palette.md)
- [13_notifications_service.md](/home/sabir/projects/cortexos/docs/specs/13_notifications_service.md)
- [14_observability_logging_telemetry.md](/home/sabir/projects/cortexos/docs/specs/14_observability_logging_telemetry.md)

Primary crates:

- `cortex-search`
- `cortex-notify`
- observability consumers as needed

Outputs:

- command palette service path
- notification service path
- admin/search consumers of observability data

Dependencies:

- Phase 9

Exit criteria:

- apps and shell can rely on shared search and notification services

## Phase 11: Theme And Accessibility

Purpose:

- centralize design tokens and accessibility behavior before large app implementation

Primary specs:

- [15_accessibility_input_keyboard_system.md](/home/sabir/projects/cortexos/docs/specs/15_accessibility_input_keyboard_system.md)
- [16_theme_design_tokens_ui_system.md](/home/sabir/projects/cortexos/docs/specs/16_theme_design_tokens_ui_system.md)

Primary packages:

- `@cortexos/theme`
- `@cortexos/ui-components`

Outputs:

- theme tokens
- shared UI primitives
- focus/ARIA/keyboard contracts
- accessibility settings integration

Dependencies:

- Phase 10

Exit criteria:

- first-party apps can use shared UI and accessibility primitives instead of inventing their own

## Phase 12: First-Party Apps

Purpose:

- validate the platform with first-party apps before the SDK

Primary specs:

- [17_first_party_core_apps_parent.md](/home/sabir/projects/cortexos/docs/specs/17_first_party_core_apps_parent.md)
- [17a_calculator_app.md](/home/sabir/projects/cortexos/docs/specs/17a_calculator_app.md)
- [17b_text_editor_app.md](/home/sabir/projects/cortexos/docs/specs/17b_text_editor_app.md)
- [17c_notes_app.md](/home/sabir/projects/cortexos/docs/specs/17c_notes_app.md)
- [17d_file_manager_app.md](/home/sabir/projects/cortexos/docs/specs/17d_file_manager_app.md)
- [17e_media_viewer_app.md](/home/sabir/projects/cortexos/docs/specs/17e_media_viewer_app.md)
- [17f_terminal_lite_app.md](/home/sabir/projects/cortexos/docs/specs/17f_terminal_lite_app.md)
- [17g_clock_and_utility_apps.md](/home/sabir/projects/cortexos/docs/specs/17g_clock_and_utility_apps.md)

Recommended order:

1. clock/utility apps
2. calculator
3. terminal-lite
4. media viewer
5. notes
6. file manager
7. text editor

Outputs:

- first-party app manifests
- first-party app implementations using only platform services

Dependencies:

- Phase 11

Exit criteria:

- core user workflows validate the platform’s lower layers

## Phase 13: Games

Purpose:

- implement bundled games as app-platform consumers

Primary specs:

- [18_games_platform_parent.md](/home/sabir/projects/cortexos/docs/specs/18_games_platform_parent.md)
- [18a_solitaire.md](/home/sabir/projects/cortexos/docs/specs/18a_solitaire.md)
- [18b_minesweeper.md](/home/sabir/projects/cortexos/docs/specs/18b_minesweeper.md)
- [18c_snake.md](/home/sabir/projects/cortexos/docs/specs/18c_snake.md)
- [18d_tetris_like_puzzle_game.md](/home/sabir/projects/cortexos/docs/specs/18d_tetris_like_puzzle_game.md)
- [18e_chess_or_checkers.md](/home/sabir/projects/cortexos/docs/specs/18e_chess_or_checkers.md)

Outputs:

- games platform shell
- individual game apps

Dependencies:

- Phase 12

Special reconciled rules:

- Tetris must use the documented spawn/lock-delay/top-out behavior
- Chess must include the documented draw rules

Exit criteria:

- games run as normal app-platform consumers without special architectural exceptions

## Phase 14: AI UX And Safety UX

Purpose:

- add AI user-facing surfaces on top of the AI runtime and policy layers

Primary specs:

- [19_ai_system_surfaces_and_ux.md](/home/sabir/projects/cortexos/docs/specs/19_ai_system_surfaces_and_ux.md)
- [20_ai_action_permissions_and_safety_controls.md](/home/sabir/projects/cortexos/docs/specs/20_ai_action_permissions_and_safety_controls.md)

Primary components:

- `@cortexos/ai-client`
- AI surface integration
- AI action confirmation and safety flows

Outputs:

- assistant panel
- conversation UX
- action confirmation UX
- low-risk auto-apply behavior tied to canonical settings

Dependencies:

- Phase 13

Exit criteria:

- AI UX is functional without bypassing `cortex-policy` or inventing client-authoritative state

## Phase 15: SDK

Purpose:

- expose the platform to third-party apps only after first-party usage has validated the internals

Primary specs:

- [21_sdk_manifest_third_party_app_platform.md](/home/sabir/projects/cortexos/docs/specs/21_sdk_manifest_third_party_app_platform.md)

Primary crate:

- `cortex-sdk`

Outputs:

- manifest schema
- install/update/uninstall flow
- sandboxing and permission-enforced APIs
- third-party app lifecycle integration

Dependencies:

- Phase 14

Exit criteria:

- the public app platform exists without hidden first-party exceptions

## Phase 16: Admin

Purpose:

- add diagnostics, recovery, and admin views after the underlying data sources exist

Primary specs:

- [22_admin_diagnostics_recovery.md](/home/sabir/projects/cortexos/docs/specs/22_admin_diagnostics_recovery.md)

Primary crate:

- `cortex-admin`

Outputs:

- diagnostics dashboards
- log/event inspection
- permission audit
- AI usage stats
- recovery flows

Dependencies:

- Phase 15

Special reconciled rules:

- host metrics are collected by the admin-owned collector
- logs and AI audit data come from observability-owned stores
- session state and metrics history use canonical SQLite-backed persistence

Exit criteria:

- admin aggregates existing subsystem data rather than creating shadow systems

## Phase 17: Release Gates

Purpose:

- convert the implemented system into a release-verifiable product

Primary specs:

- [23_release_readiness_qa_acceptance_framework.md](/home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md)
- [appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)

Outputs:

- E2E suite
- coverage enforcement
- CI hardening
- release validation artifacts

Dependencies:

- Phase 16

Exit criteria:

- release gates are measurable and passable
- the project can prove readiness rather than asserting it

## Completion Status

Implementation status (as of 2026-03-31):

- All phases have been scaffolded: Rust crates, frontend apps, packages, and CI are in place.
- Frontend builds and tests pass. Rust workspace has compile errors that block `cargo test --workspace`.
- No phase can be considered complete by the project's own Definition of Done until quality gates pass and spec drift is resolved.
- See [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md) for outstanding issues.

Documentation status:

- all phases are documented with ownership, dependencies, and exit criteria
- Phase 1 has a dedicated execution checklist
- Phases 2 through 17 have dedicated checklists at the repo root
