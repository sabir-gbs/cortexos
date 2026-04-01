# Implementation Phase Checklists

This document provides detailed execution checklists for Phases 2 through 17. Many phases are now partially or fully scaffolded with real implementation code; these checklists serve as a Definition of Done reference rather than a purely forward-looking plan.

Use alongside:

- [IMPLEMENTATION_PHASES.md](/home/sabir/projects/cortexos/docs/guides/IMPLEMENTATION_PHASES.md)
- [IMPLEMENTATION_KICKOFF.md](/home/sabir/projects/cortexos/docs/guides/IMPLEMENTATION_KICKOFF.md)
- [PHASE1_BOOTSTRAP_CHECKLIST.md](/home/sabir/projects/cortexos/docs/checklists/PHASE1_BOOTSTRAP_CHECKLIST.md)

These are planning and sequencing documents only. They do not authorize implementation shortcuts or codegen outside the spec contracts.

## Phase 2: Rust Foundation Checklist

Primary scope:

- `cortex-core`
- `cortex-config`
- foundational `cortex-sdk` types

Read first:

- [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md)
- [02_core_architecture_and_runtime_boundaries.md](/home/sabir/projects/cortexos/docs/specs/02_core_architecture_and_runtime_boundaries.md)
- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)

Checklist:

- [ ] Define shared IDs, base enums, common error contracts, and core traits in `cortex-core`
- [ ] Define the cross-cutting client/server and service boundary types without leaking subsystem-specific logic into `cortex-core`
- [ ] Define config-loading precedence and environment/file loading rules in `cortex-config`
- [ ] Keep `cortex-sdk` to foundational shared types only at this stage
- [ ] Ensure crate ownership boundaries match the spec table in master spec section 21.2
- [ ] Ensure no service crate logic is pulled prematurely into the foundation layer
- [ ] Document all exported public types and traits with rustdoc
- [ ] Add unit tests for config precedence, parsing, and shared type invariants

Must be real:

- shared error taxonomy
- core identifiers
- config model and load path
- compile-safe foundational traits

May stay stubbed:

- feature-level service behavior
- subsystem implementations that belong to later phases

Exit criteria:

- later crates can depend on these types without redefining them
- no dependency inversion is needed when Phase 3 starts

## Phase 3: Persistence And Observability Checklist

Primary scope:

- `cortex-db`
- `cortex-observability`

Read first:

- [11_virtual_filesystem_and_storage_abstraction.md](/home/sabir/projects/cortexos/docs/specs/11_virtual_filesystem_and_storage_abstraction.md)
- [14_observability_logging_telemetry.md](/home/sabir/projects/cortexos/docs/specs/14_observability_logging_telemetry.md)
- [22_admin_diagnostics_recovery.md](/home/sabir/projects/cortexos/docs/specs/22_admin_diagnostics_recovery.md)

Checklist:

- [ ] Implement migration runner ownership and schema version tracking in `cortex-db`
- [ ] Define canonical SQLite access patterns and transaction helpers
- [ ] Define structured log event format and subscriber initialization in `cortex-observability`
- [ ] Define the searchable log mirror contract used by admin tooling
- [ ] Define retention boundaries for logs, audit records, and persisted observability data
- [ ] Ensure no child spec still implies shadow persistence outside the canonical model
- [ ] Define audit-table ownership and generic append-only persistence utilities
- [ ] Add integration tests for migration application and rollback behavior
- [ ] Add tests for log serialization and retention-rule enforcement

Must be real:

- migration runner
- SQLite-backed persistence entrypoint
- structured logging contract
- audit persistence primitives

May stay stubbed:

- advanced admin queries
- full-text search optimization details

Exit criteria:

- later services have one canonical place to persist structured state
- admin/log consumers can rely on observability-owned records

## Phase 4: Identity And Policy Checklist

Primary scope:

- `cortex-auth`
- `cortex-policy`

Read first:

- [03_identity_authentication_sessions_user_profiles.md](/home/sabir/projects/cortexos/docs/specs/03_identity_authentication_sessions_user_profiles.md)
- [04_permissions_policy_trust_model.md](/home/sabir/projects/cortexos/docs/specs/04_permissions_policy_trust_model.md)
- [20_ai_action_permissions_and_safety_controls.md](/home/sabir/projects/cortexos/docs/specs/20_ai_action_permissions_and_safety_controls.md)

Checklist:

- [ ] Define session issuance, validation, revocation, and persistence rules
- [ ] Ensure cookie/session model matches the server-authoritative security contract
- [ ] Define user/profile models only to the extent required for auth and session ownership
- [ ] Define policy-check interfaces for files, settings, AI actions, SDK calls, and app runtime operations
- [ ] Ensure first-party apps do not receive hidden permission bypasses
- [ ] Define audit hooks for policy grants, denials, revocations, and high-risk actions
- [ ] Define error taxonomy for unauthorized, forbidden, expired, and invalid session states
- [ ] Add tests for session invalidation, expired sessions, and denied permission paths

Must be real:

- session contract
- auth enforcement
- policy engine interface
- denial and audit paths

May stay stubbed:

- richer user profile UX
- less critical account management flows

Exit criteria:

- later services can call auth/policy as infrastructure, not reimplement them

## Phase 5: Settings And Filesystem Checklist

Primary scope:

- `cortex-settings`
- `cortex-files`

Read first:

- [05_settings_service_and_settings_app.md](/home/sabir/projects/cortexos/docs/specs/05_settings_service_and_settings_app.md)
- [11_virtual_filesystem_and_storage_abstraction.md](/home/sabir/projects/cortexos/docs/specs/11_virtual_filesystem_and_storage_abstraction.md)
- [appendix_a_required_ai_settings_fields.md](/home/sabir/projects/cortexos/docs/specs/appendix_a_required_ai_settings_fields.md)

Checklist:

- [ ] Implement namespace-aware settings model and validation pipeline
- [ ] Ensure all AI settings keys and ranges from appendix A are represented canonically
- [ ] Define `settings.changed` event semantics and propagation path
- [ ] Define settings read/write authorization through policy
- [ ] Define filesystem metadata vs blob-content handling per reconciled storage rules
- [ ] Ensure file operations route through `cortex-files`, never direct host access by apps
- [ ] Define persistence semantics for file metadata, content blobs, exports, and crash artifacts
- [ ] Add tests for settings validation, invalid updates, and namespace isolation
- [ ] Add tests for filesystem path safety and metadata/content consistency

Must be real:

- settings validation and persistence
- canonical AI settings registry usage
- file service boundaries

May stay stubbed:

- extensive settings UI polish
- advanced import/export utilities

Exit criteria:

- later phases can safely depend on settings and files without redefining policy or storage rules

## Phase 6: AI Runtime Checklist

Primary scope:

- `cortex-ai`

Read first:

- [06_ai_runtime_provider_registry_preferred_llm_model_routing.md](/home/sabir/projects/cortexos/docs/specs/06_ai_runtime_provider_registry_preferred_llm_model_routing.md)
- [appendix_a_required_ai_settings_fields.md](/home/sabir/projects/cortexos/docs/specs/appendix_a_required_ai_settings_fields.md)
- [19_ai_system_surfaces_and_ux.md](/home/sabir/projects/cortexos/docs/specs/19_ai_system_surfaces_and_ux.md)

Checklist:

- [ ] Define provider adapter trait and provider registry model
- [ ] Implement deterministic routing precedence exactly as specified
- [ ] Implement `NoProviderConfigured` as a deterministic error path
- [ ] Ensure fallback behavior only occurs after a selected provider fails
- [ ] Define timeout handling for connect, first-token, total, and stream-idle cases
- [ ] Define audit and budget integration points
- [ ] Ensure provider-specific code remains isolated in adapters
- [ ] Ensure `Zhipu` support remains represented in provider enums and settings contracts
- [ ] Add tests for routing precedence, no-provider, fallback behavior, and timeout handling

Must be real:

- routing engine
- provider adapter abstraction
- timeout model
- audit hooks

May stay stubbed:

- long-tail provider adapters
- full model catalog sync and caching optimization

Exit criteria:

- AI runtime can process or reject requests deterministically without any UI dependency

## Phase 7: Command Bus Checklist

Primary scope:

- typed command/event bus

Read first:

- [10_system_command_bus_and_event_model.md](/home/sabir/projects/cortexos/docs/specs/10_system_command_bus_and_event_model.md)
- [02_core_architecture_and_runtime_boundaries.md](/home/sabir/projects/cortexos/docs/specs/02_core_architecture_and_runtime_boundaries.md)

Checklist:

- [ ] Define typed Rust command and event interfaces
- [ ] Define typed TypeScript client subscription contract
- [ ] Implement durable idempotency record semantics
- [ ] Define dead-letter persistence and retry semantics
- [ ] Normalize event names to canonical registry values only
- [ ] Ensure no `serde_json::Value`-style untyped transport escapes leak into public contracts
- [ ] Define permission interception points before command dispatch
- [ ] Add tests for duplicate command replay, restart-safe idempotency, and event fan-out

Must be real:

- typed commands and events
- command dispatch
- durable idempotency
- dead-letter handling

May stay stubbed:

- some low-priority producers/consumers

Exit criteria:

- later realtime subsystems can rely on one transport model and one schema discipline

## Phase 8: Window Manager And Shell Checklist

Primary scope:

- `cortex-runtime` window management slice
- `apps/desktop-shell`

Read first:

- [07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md)
- [08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md)
- [15_accessibility_input_keyboard_system.md](/home/sabir/projects/cortexos/docs/specs/15_accessibility_input_keyboard_system.md)

Checklist:

- [ ] Define window state, workspace state, and lifecycle event contracts
- [ ] Ensure window mutations use command-bus commands, not ad hoc client REST
- [ ] Limit HTTP usage to bootstrap/snapshot reads where specified
- [ ] Ensure shell shortcuts use canonical values only
- [ ] Ensure shell subscribes only to canonical event names
- [ ] Define focus, z-order, minimize/maximize, and workspace switching semantics
- [ ] Add tests for window state transitions and shell event handling
- [ ] Add accessibility checks for focus and keyboard navigation at shell level

Must be real:

- shell/window integration
- event-driven state updates
- bootstrap/snapshot contract

May stay stubbed:

- richer desktop polish
- non-critical shell customizations

Exit criteria:

- a minimal desktop is operable and bound to canonical runtime contracts

## Phase 9: App Runtime Checklist

Primary scope:

- `cortex-runtime`
- `cortex-api` wiring for app lifecycle

Read first:

- [09_app_runtime_and_app_lifecycle.md](/home/sabir/projects/cortexos/docs/specs/09_app_runtime_and_app_lifecycle.md)
- [08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md)
- [21_sdk_manifest_third_party_app_platform.md](/home/sabir/projects/cortexos/docs/specs/21_sdk_manifest_third_party_app_platform.md)

Checklist:

- [ ] Define app launch, suspend, stop, crash, and restart semantics
- [ ] Ensure manifest loading and lifecycle ownership are centralized
- [ ] Ensure runtime emits canonical lifecycle events
- [ ] Ensure app runtime does not bypass declared permissions
- [ ] Define single-instance and multi-instance behavior where specified
- [ ] Add tests for launch/stop/crash flows and manifest validation at runtime

Must be real:

- runtime lifecycle model
- manifest handling
- lifecycle events

May stay stubbed:

- advanced warm-start or optimization behavior

Exit criteria:

- platform can host apps in a consistent, observable way

## Phase 10: Core System Services Checklist

Primary scope:

- `cortex-search`
- `cortex-notify`
- observability-consuming service paths

Read first:

- [12_search_indexing_global_command_palette.md](/home/sabir/projects/cortexos/docs/specs/12_search_indexing_global_command_palette.md)
- [13_notifications_service.md](/home/sabir/projects/cortexos/docs/specs/13_notifications_service.md)
- [14_observability_logging_telemetry.md](/home/sabir/projects/cortexos/docs/specs/14_observability_logging_telemetry.md)

Checklist:

- [ ] Implement search indexing and retrieval with reconciled storage assumptions
- [ ] Ensure command palette shortcut is `Ctrl+Space`
- [ ] Implement notification event model using canonical names
- [ ] Ensure logs used by system services flow through observability-owned paths
- [ ] Add tests for search indexing correctness and notification lifecycle behavior

Must be real:

- search core service
- notification core service
- canonical event usage

May stay stubbed:

- advanced ranking
- long-tail notification presentation polish

Exit criteria:

- apps and shell can consume search and notification services through stable contracts

## Phase 11: Theme And Accessibility Checklist

Primary scope:

- `@cortexos/theme`
- `@cortexos/ui-components`
- accessibility integration

Read first:

- [15_accessibility_input_keyboard_system.md](/home/sabir/projects/cortexos/docs/specs/15_accessibility_input_keyboard_system.md)
- [16_theme_design_tokens_ui_system.md](/home/sabir/projects/cortexos/docs/specs/16_theme_design_tokens_ui_system.md)

Checklist:

- [ ] Define theme tokens and shared UI contract
- [ ] Define focus, ARIA, keyboard, contrast, and motion behaviors centrally
- [ ] Ensure accessibility settings wire through the settings service
- [ ] Ensure shared UI does not conflict with shell shortcut and focus rules
- [ ] Add tests or audits for contrast, focus, and keyboard traversal

Must be real:

- token contract
- shared accessibility behavior
- shared UI primitives

May stay stubbed:

- non-essential theme variants
- final visual refinements

Exit criteria:

- first-party apps can build on consistent theme and accessibility primitives

## Phase 12: First-Party Apps Checklist

Primary scope:

- first-party apps from spec 17

Read first:

- [17_first_party_core_apps_parent.md](/home/sabir/projects/cortexos/docs/specs/17_first_party_core_apps_parent.md)
- [appendix_b_minimum_first_party_app_list.md](/home/sabir/projects/cortexos/docs/specs/appendix_b_minimum_first_party_app_list.md)
- individual app specs `17a` through `17g`

Checklist:

- [ ] Implement apps in spec-defined order unless there is a documented reason to change it
- [ ] Ensure every app has an explicit manifest and declared permissions
- [ ] Ensure apps use system services, not direct host filesystem/network access
- [ ] Ensure app-level keyboard shortcuts do not conflict with canonical system shortcuts
- [ ] Add app-specific tests for core flows and error states

Must be real:

- manifests
- explicit permission declarations
- service-based integrations

May stay stubbed:

- non-core app enhancements
- optional convenience features outside v1

Exit criteria:

- first-party apps validate the platform without special privilege exceptions

## Phase 13: Games Checklist

Primary scope:

- game platform and bundled games

Read first:

- [18_games_platform_parent.md](/home/sabir/projects/cortexos/docs/specs/18_games_platform_parent.md)
- individual game specs `18a` through `18e`

Checklist:

- [ ] Ensure games remain normal app-platform consumers
- [ ] Ensure save/load flows route through canonical file services
- [ ] Ensure Tetris uses the documented spawn, lock-delay, DAS/ARR, and top-out rules
- [ ] Ensure Chess uses the documented draw-state rules
- [ ] Add deterministic logic tests for game-rule correctness

Must be real:

- platform-consistent app integration
- game-rule correctness for documented mechanics

May stay stubbed:

- optional polish outside spec scope

Exit criteria:

- games conform to their specs without hidden gameplay assumptions

## Phase 14: AI UX And Safety UX Checklist

Primary scope:

- `@cortexos/ai-client`
- AI surface UX
- AI action confirmation/safety UX

Read first:

- [19_ai_system_surfaces_and_ux.md](/home/sabir/projects/cortexos/docs/specs/19_ai_system_surfaces_and_ux.md)
- [20_ai_action_permissions_and_safety_controls.md](/home/sabir/projects/cortexos/docs/specs/20_ai_action_permissions_and_safety_controls.md)

Checklist:

- [ ] Ensure assistant shortcut is `Ctrl+Shift+A`
- [ ] Ensure conversation state is server-authoritative
- [ ] Ensure low-risk auto-apply uses the canonical setting key
- [ ] Ensure risky actions still flow through `cortex-policy`
- [ ] Ensure AI action audit and permission grant data use canonical persistence paths
- [ ] Add tests for permission prompts, denial paths, and conversation persistence behavior

Must be real:

- AI client contract
- safety/confirmation flows
- canonical persistence and policy integration

May stay stubbed:

- extra surface polish
- non-core assistant affordances

Exit criteria:

- user-facing AI is functional without violating runtime, policy, or storage rules

## Phase 15: SDK Checklist

Primary scope:

- `cortex-sdk`

Read first:

- [21_sdk_manifest_third_party_app_platform.md](/home/sabir/projects/cortexos/docs/specs/21_sdk_manifest_third_party_app_platform.md)
- [09_app_runtime_and_app_lifecycle.md](/home/sabir/projects/cortexos/docs/specs/09_app_runtime_and_app_lifecycle.md)
- [20_ai_action_permissions_and_safety_controls.md](/home/sabir/projects/cortexos/docs/specs/20_ai_action_permissions_and_safety_controls.md)

Checklist:

- [ ] Define manifest schema and validation path
- [ ] Define install, update, uninstall, and registry flows using canonical persistence
- [ ] Ensure third-party apps are sandboxed and policy-gated
- [ ] Ensure SDK AI hooks align with reconciled AI ownership
- [ ] Ensure third-party apps do not gain privileges unavailable to first-party apps
- [ ] Add tests for manifest validation, install lifecycle, and permission enforcement

Must be real:

- manifest contract
- install/update/uninstall contract
- sandbox and permission enforcement

May stay stubbed:

- marketplace UX
- advanced verification workflows beyond v1

Exit criteria:

- third-party app platform exists as a constrained public layer, not a side path around platform rules

## Phase 16: Admin Checklist

Primary scope:

- `cortex-admin`

Read first:

- [22_admin_diagnostics_recovery.md](/home/sabir/projects/cortexos/docs/specs/22_admin_diagnostics_recovery.md)
- [14_observability_logging_telemetry.md](/home/sabir/projects/cortexos/docs/specs/14_observability_logging_telemetry.md)
- [20_ai_action_permissions_and_safety_controls.md](/home/sabir/projects/cortexos/docs/specs/20_ai_action_permissions_and_safety_controls.md)

Checklist:

- [ ] Ensure admin aggregates subsystem-owned data instead of creating shadow stores
- [ ] Ensure host metrics come from the admin-owned collector
- [ ] Ensure logs and AI audit data come from observability-owned records
- [ ] Ensure session recovery and metrics history use canonical SQLite-backed persistence
- [ ] Define recovery, export, and reset flows using documented state ownership boundaries
- [ ] Add tests for recovery behavior, export redaction, and collector/store integration

Must be real:

- metrics collector ownership
- diagnostics data aggregation
- recovery and export behavior

May stay stubbed:

- non-essential admin polish

Exit criteria:

- admin tooling can inspect and recover the system without contradicting platform ownership boundaries

## Phase 17: Release Gates Checklist

Primary scope:

- E2E, CI hardening, coverage, release validation

Read first:

- [23_release_readiness_qa_acceptance_framework.md](/home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md)
- [appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)

Checklist:

- [ ] Ensure every subsystem can prove its Definition of Done
- [ ] Ensure workspace-wide fmt, clippy, build, test, and typecheck gates are enforced
- [ ] Ensure E2E coverage exists for critical flows
- [ ] Ensure coverage thresholds are measured and enforced
- [ ] Ensure security, accessibility, and observability gates are represented in CI/release validation
- [ ] Ensure release validation is measurable rather than narrative

Must be real:

- CI gates
- E2E harness
- release validation contract

May stay stubbed:

- none that would undermine a release-readiness claim

Exit criteria:

- the project can demonstrate readiness through passing gates, not documentation alone

## Recommended Next Documentation Step

The planning stack is now:

- [IMPLEMENTATION_KICKOFF.md](/home/sabir/projects/cortexos/docs/guides/IMPLEMENTATION_KICKOFF.md)
- [IMPLEMENTATION_PHASES.md](/home/sabir/projects/cortexos/docs/guides/IMPLEMENTATION_PHASES.md)
- [PHASE1_BOOTSTRAP_CHECKLIST.md](/home/sabir/projects/cortexos/docs/checklists/PHASE1_BOOTSTRAP_CHECKLIST.md)
- [IMPLEMENTATION_PHASE_CHECKLISTS.md](/home/sabir/projects/cortexos/docs/guides/IMPLEMENTATION_PHASE_CHECKLISTS.md)

If you want one more documentation pass, the next high-value step would be to add PR-sized checklists for Phases 2 through 6 individually, similar in granularity to Phase 1.
