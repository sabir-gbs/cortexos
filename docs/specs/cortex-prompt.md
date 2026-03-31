You are operating inside an OpenSpec-driven engineering workflow.

Your task is to generate a complete OpenSpec-ready specification system for a product called CortexOS.

You must produce the entire spec set from start to finish.

Do not produce a summary.
Do not produce a lightweight PRD.
Do not produce marketing copy.
Do not stop at an outline.
Do not ask clarifying questions.
Do not leave critical sections underspecified.
Do not write vague requirements that force future interpretation.
Do not use “TBD” unless absolutely unavoidable. When a default is needed, choose a reasonable implementation default and state it explicitly.

The output must be optimized for:
1. Claude Code as the primary implementation agent
2. GLM-5.1 as an additional model that may later consume the specs
3. OpenSpec-style change/spec workflows
4. minimal rework during coding
5. deterministic subsystem boundaries
6. implementation in a real repository

The goal is to create an authoritative, implementation-grade spec pack for CortexOS that can be placed into an OpenSpec workflow and then implemented subsystem by subsystem.

==================================================
PRODUCT
==================================================

Product name: CortexOS

Definition:
CortexOS is an AI-native browser operating system that provides a browser-rendered desktop shell, windowed apps, a virtual filesystem, system services, core built-in apps, bundled games, and a first-class AI runtime integrated into the OS itself.

Core principle:
AI is a system layer, not a bolt-on feature.

Critical product requirement:
In Settings, the user must be able to choose their Preferred LLM at the OS level.

The OS must also support:
- per-app AI model overrides
- per-feature AI model overrides
- provider registry
- model capability metadata
- routing/fallback behavior
- privacy and permission controls for AI
- auditability for AI actions
- cost/budget controls where applicable

The OS must still remain functional even if no AI provider is configured.

==================================================
IMPLEMENTATION ASSUMPTIONS
==================================================

These are defaults unless a strong reason exists to refine them.

Primary implementation language:
- Rust stable for all backend/system/runtime/orchestration/storage/policy/AI layers

Browser/UI language:
- TypeScript for browser-facing UI surfaces

Architecture assumptions:
- browser-based desktop shell
- Rust backend services
- typed APIs
- typed manifests
- typed event contracts
- monorepo
- Cargo workspace
- strong subsystem boundaries
- security-sensitive logic server-side only
- no provider lock-in in core AI runtime
- no client-side authorization as source of truth

Recommended Rust crates:
- cortex-core
- cortex-config
- cortex-db
- cortex-api
- cortex-auth
- cortex-policy
- cortex-settings
- cortex-ai
- cortex-files
- cortex-search
- cortex-notify
- cortex-observability
- cortex-runtime
- cortex-sdk
- cortex-admin

Recommended app structure:
- apps/desktop-shell
- apps/settings-app
- apps/calculator-app
- apps/text-editor-app
- apps/notes-app
- apps/file-manager-app
- apps/media-viewer-app
- apps/terminal-lite-app
- apps/clock-utils-app
- apps/games/solitaire
- apps/games/minesweeper
- apps/games/snake
- apps/games/tetris
- apps/games/chess

You may improve structure if needed, but do not radically diverge without explicitly justifying it.

==================================================
OPENSpec WORKFLOW REQUIREMENT
==================================================

Write the output so it can be used directly in an OpenSpec-driven repository.

That means:
- the master document must define the overall spec system
- each major subsystem must be a separate spec file
- each spec must be implementation-grade
- each spec must define exact scope, boundaries, interfaces, data models, failure modes, and acceptance criteria
- each spec must be suitable for future OpenSpec “apply” or implementation flows
- each spec must reduce room for interpretation by Claude Code or GLM-5.1
- every subsystem spec must explicitly say what it owns and what it does not own
- every subsystem spec must include build order and dependencies
- every subsystem spec must include explicit implementation instructions for Claude Code / Codex-style coding agents

Treat these specs as authoritative engineering documents, not discussion notes.

==================================================
MODEL-OPTIMIZED WRITING RULES
==================================================

Because this output may later be consumed by Claude Code and GLM-5.1, use the following writing discipline:

1. Prefer deterministic language over conversational explanation.
2. Prefer explicit defaults over open-ended choice.
3. Prefer exact subsystem ownership over blended responsibilities.
4. Define invariants explicitly.
5. Define anti-patterns explicitly.
6. Define acceptance criteria as checklists where useful.
7. Define data structures, enums, and interfaces in a concrete but implementation-agnostic way.
8. When giving suggested API routes or Rust trait names, keep them stable and obvious.
9. Avoid stylistic filler.
10. Avoid aspirational product language.
11. Avoid “smart,” “intuitive,” “seamless,” or “powerful” unless behavior is concretely defined.
12. Do not compress critical edge cases.
13. Do not skip error handling or recovery behavior.
14. Do not skip testing requirements.
15. Do not skip security or permission enforcement details.
16. Do not make hidden assumptions across subsystem boundaries.

==================================================
REQUIRED FILE SET
==================================================

Generate the full spec system as separate files using this file set unless a clearly better numbering adjustment is necessary:

00_master_spec.md
01_repository_toolchain_engineering_conventions.md
02_core_architecture_and_runtime_boundaries.md
03_identity_authentication_sessions_user_profiles.md
04_permissions_policy_trust_model.md
05_settings_service_and_settings_app.md
06_ai_runtime_provider_registry_preferred_llm_model_routing.md
07_desktop_shell.md
08_window_manager.md
09_app_runtime_and_app_lifecycle.md
10_system_command_bus_and_event_model.md
11_virtual_filesystem_and_storage_abstraction.md
12_search_indexing_global_command_palette.md
13_notifications_service.md
14_observability_logging_telemetry.md
15_accessibility_input_keyboard_system.md
16_theme_design_tokens_ui_system.md
17_first_party_core_apps_parent.md
17a_calculator_app.md
17b_text_editor_app.md
17c_notes_app.md
17d_file_manager_app.md
17e_media_viewer_app.md
17f_terminal_lite_app.md
17g_clock_and_utility_apps.md
18_games_platform_parent.md
18a_solitaire.md
18b_minesweeper.md
18c_snake.md
18d_tetris_like_puzzle_game.md
18e_chess_or_checkers.md
19_ai_system_surfaces_and_ux.md
20_ai_action_permissions_and_safety_controls.md
21_sdk_manifest_third_party_app_platform.md
22_admin_diagnostics_recovery.md
23_release_readiness_qa_acceptance_framework.md
appendix_a_required_ai_settings_fields.md
appendix_b_minimum_first_party_app_list.md
appendix_c_definition_of_done.md
appendix_d_coding_agent_guardrails.md

==================================================
MASTER SPEC REQUIREMENTS
==================================================

The master spec must:
- define CortexOS at the product level
- define product goals
- define product non-goals
- define system-wide engineering assumptions
- define the spec-authoring contract
- define coding-agent guardrails
- define subsystem dependency order
- define canonical build order
- define conflict resolution between parent spec and child specs
- define how specific subspecs override broader parent wording only within their declared scope
- define completion criteria for each subsystem spec
- reference every child spec by number and title
- state that child specs are authoritative within their owned domain

==================================================
MANDATORY SECTION TEMPLATE
==================================================

Every major spec and every significant subspec must include these sections in this order:

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

If a smaller app or game spec does not need deep detail in one section, still include the section and provide concise but concrete content.

==================================================
GLOBAL PRODUCT REQUIREMENTS
==================================================

The spec pack must cover all of the following:

Core platform:
- repository and engineering conventions
- core architecture and runtime boundaries
- identity/authentication/sessions/user profiles
- permissions/policy/trust model
- settings service and settings app
- AI runtime/provider registry/preferred LLM/model routing
- desktop shell
- window manager
- app runtime and app lifecycle
- command bus and event model
- virtual filesystem and storage abstraction
- search/indexing/global command palette
- notifications
- observability/logging/telemetry
- accessibility/input/keyboard system
- theme/design tokens/UI system
- SDK/manifest/third-party app platform
- admin/diagnostics/recovery
- release readiness/QA/acceptance framework

First-party apps:
- Settings
- Calculator
- Text Editor
- Notes
- File Manager
- Media Viewer
- Terminal-lite
- Clock utilities

Games:
- Solitaire
- Minesweeper
- Snake
- Tetris-like puzzle game
- Chess or Checkers

AI surfaces:
- global assistant surface
- app assistant surfaces
- selected text/file AI actions
- AI action permissions and safety controls

==================================================
HIGH-RISK SPEC EXPANSION REQUIREMENT
==================================================

The following specs must be especially detailed and must include concrete contracts, invariants, and implementation guidance:

1. 01 Repository, Toolchain, Engineering Conventions
Include:
- repo structure
- workspace layout
- crate ownership
- dependency rules
- schema ownership
- naming conventions
- CI requirements
- testing conventions
- local development conventions
- command conventions
- migration conventions

2. 02 Core Architecture and Runtime Boundaries
Include:
- trusted vs untrusted layers
- client vs server responsibilities
- source-of-truth mapping
- service ownership map
- communication patterns
- degraded mode behavior
- restore behavior
- error taxonomy
- boundary rules

3. 04 Permissions, Policy, and Trust Model
Include:
- permission categories
- resource/action model
- app grants
- AI grants
- file access grants
- clipboard grants
- one-time vs persistent grants
- revocation semantics
- deny behavior
- audit requirements
- server-side enforcement rules

4. 05 Settings Service and Settings App
Include:
- settings namespaces
- schema validation
- defaults resolution order
- effective settings resolution rules
- settings API surface
- AI settings fields
- import/export rules if included
- settings app IA and behavior

5. 06 AI Runtime, Provider Registry, Preferred LLM, and Model Routing
Include:
- provider abstraction
- provider adapter contract
- model capability metadata
- routing precedence
- task types
- fallback chain behavior
- timeout rules
- structured output handling
- tool-call handling
- budget policy
- logging and audit model
- context attachment rules
- no provider lock-in

6. 09 App Runtime and App Lifecycle
Include:
- app manifest contract
- app lifecycle states
- launch rules
- restore rules
- single-instance vs multi-instance rules
- crash containment
- capability declaration
- app identity and version compatibility

7. 11 Virtual Filesystem and Storage Abstraction
Include:
- file/directory model
- file identity vs path
- metadata invariants
- content store abstraction
- save/open semantics
- move/rename rules
- trash/restore rules
- conflict behavior
- quotas
- file associations

8. 19 AI System Surfaces and UX
Include:
- global assistant panel
- app assistant hooks
- selected text/file AI actions
- provider/model disclosure
- confirmation expectations
- failure behavior when model/provider unavailable

9. 20 AI Action Permissions and Safety Controls
Include:
- what AI can read
- what AI can modify
- what AI can create
- required user confirmations
- action schemas
- revocation controls
- audit trail requirements
- explicit anti-patterns preventing unsafe mutation shortcuts

==================================================
CRITICAL AI SETTINGS REQUIREMENT
==================================================

The Settings and AI Runtime specs must explicitly include these fields and behaviors:

Required settings fields:
- ai.preferred_provider
- ai.preferred_model
- ai.fallback_enabled
- ai.fallback_chain
- ai.per_app_overrides
- ai.per_feature_overrides
- ai.privacy_mode
- ai.allow_file_access
- ai.allow_clipboard_access
- ai.budget_policy
- ai.show_model_disclosure

Required behavior:
- Preferred LLM is a true OS-level setting
- routing precedence must be deterministic
- fallback behavior must be defined
- no provider-specific shortcuts in core logic
- AI context access must be permission-gated
- AI action audit trails are mandatory for sensitive actions

==================================================
REPOSITORY-READY OUTPUT REQUIREMENT
==================================================

Write the output as if it will be saved directly into an OpenSpec-compatible repository under a specs/docs area.

For every file:
- begin with a clear delimiter
- include the exact filename
- then include the full file content

Use this output format:

=== FILE: 00_master_spec.md ===
[full content]

=== FILE: 01_repository_toolchain_engineering_conventions.md ===
[full content]

Continue this exact structure until the full spec pack is complete.

==================================================
CLAUDE CODE / GLM-5.1 IMPLEMENTATION INSTRUCTIONS
==================================================

In the final section of each spec, titled exactly:
Implementation Instructions for Claude Code / Codex

Include:
- exact subsystem ownership reminders
- recommended Rust crate(s)
- recommended frontend app/package placement if applicable
- what is allowed to be stubbed initially
- what must be real in v1
- what cannot be inferred
- stop conditions for marking the subsystem done
- testing gates for that subsystem

Write these instructions so that:
- Claude Code can execute them directly in implementation phases
- GLM-5.1 can parse them without losing critical structure
- OpenSpec change/apply flows can use them cleanly

==================================================
ANTI-REWORK RULES
==================================================

The spec pack must explicitly prevent:
- hidden coupling between services
- frontend-owned authorization
- provider lock-in
- direct app bypass of file/policy/settings services
- hidden privileges for first-party apps
- ambiguous lifecycle behavior
- silent conflict resolution
- ad hoc permission prompts
- untyped event payloads
- incomplete “UI-only” definitions of done

==================================================
QUALITY BAR
==================================================

The output must be:
- full
- internally consistent
- implementation-grade
- ready for AI coding workflows
- explicit enough to reduce rework
- specific enough to support subsystem-by-subsystem implementation

Do not stop early.
Do not return only a plan.
Do not omit smaller specs.
Do not compress critical specs into shallow summaries.

==================================================
DEFINITION OF DONE
==================================================

You are done only when:
- the master spec is complete
- every required file exists
- every major file includes the mandatory sections
- the AI-native platform requirements are fully integrated
- Preferred LLM behavior is fully specified
- core apps are fully covered
- games are covered
- guardrails are covered
- the result is implementation-grade

Generate the entire CortexOS OpenSpec-ready spec pack now.
