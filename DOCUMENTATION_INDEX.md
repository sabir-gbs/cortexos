# Documentation Index

> **Status: Working document.** Gate claims were last verified on 2026-03-31. Re-run gates before trusting completion marks.

This is the single index page for the entire CortexOS documentation stack.

Use this page when you want one place that links:

- top-level agent guidance
- the master implementation brief
- the full spec set
- the implementation planning documents
- the phase-by-phase checklists
- the single Claude Code prompt

## Start Here

If you are starting implementation or directing an agent, read in this order:

1. [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md)
2. [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md)
3. [CLAUDE.md](/home/sabir/projects/cortexos/CLAUDE.md)
4. [CLAUDE_CODE_SINGLE_PROMPT.md](/home/sabir/projects/cortexos/CLAUDE_CODE_SINGLE_PROMPT.md)

## Top-Level Guidance

- [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md): top-level execution reference for implementing all of CortexOS
- [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md): agent operating rules for this repository
- [CLAUDE.md](/home/sabir/projects/cortexos/CLAUDE.md): Claude Code repository guidance
- [CLAUDE_CODE_SINGLE_PROMPT.md](/home/sabir/projects/cortexos/CLAUDE_CODE_SINGLE_PROMPT.md): copy-paste single prompt for Claude Code using GLM-5.1

## Core Specs

- [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md)
- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [02_core_architecture_and_runtime_boundaries.md](/home/sabir/projects/cortexos/docs/specs/02_core_architecture_and_runtime_boundaries.md)
- [03_identity_authentication_sessions_user_profiles.md](/home/sabir/projects/cortexos/docs/specs/03_identity_authentication_sessions_user_profiles.md)
- [04_permissions_policy_trust_model.md](/home/sabir/projects/cortexos/docs/specs/04_permissions_policy_trust_model.md)
- [05_settings_service_and_settings_app.md](/home/sabir/projects/cortexos/docs/specs/05_settings_service_and_settings_app.md)
- [06_ai_runtime_provider_registry_preferred_llm_model_routing.md](/home/sabir/projects/cortexos/docs/specs/06_ai_runtime_provider_registry_preferred_llm_model_routing.md)
- [07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md)
- [08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md)
- [09_app_runtime_and_app_lifecycle.md](/home/sabir/projects/cortexos/docs/specs/09_app_runtime_and_app_lifecycle.md)
- [10_system_command_bus_and_event_model.md](/home/sabir/projects/cortexos/docs/specs/10_system_command_bus_and_event_model.md)
- [11_virtual_filesystem_and_storage_abstraction.md](/home/sabir/projects/cortexos/docs/specs/11_virtual_filesystem_and_storage_abstraction.md)
- [12_search_indexing_global_command_palette.md](/home/sabir/projects/cortexos/docs/specs/12_search_indexing_global_command_palette.md)
- [13_notifications_service.md](/home/sabir/projects/cortexos/docs/specs/13_notifications_service.md)
- [14_observability_logging_telemetry.md](/home/sabir/projects/cortexos/docs/specs/14_observability_logging_telemetry.md)
- [15_accessibility_input_keyboard_system.md](/home/sabir/projects/cortexos/docs/specs/15_accessibility_input_keyboard_system.md)
- [16_theme_design_tokens_ui_system.md](/home/sabir/projects/cortexos/docs/specs/16_theme_design_tokens_ui_system.md)

## First-Party Apps Specs

- [17_first_party_core_apps_parent.md](/home/sabir/projects/cortexos/docs/specs/17_first_party_core_apps_parent.md)
- [17a_calculator_app.md](/home/sabir/projects/cortexos/docs/specs/17a_calculator_app.md)
- [17b_text_editor_app.md](/home/sabir/projects/cortexos/docs/specs/17b_text_editor_app.md)
- [17c_notes_app.md](/home/sabir/projects/cortexos/docs/specs/17c_notes_app.md)
- [17d_file_manager_app.md](/home/sabir/projects/cortexos/docs/specs/17d_file_manager_app.md)
- [17e_media_viewer_app.md](/home/sabir/projects/cortexos/docs/specs/17e_media_viewer_app.md)
- [17f_terminal_lite_app.md](/home/sabir/projects/cortexos/docs/specs/17f_terminal_lite_app.md)
- [17g_clock_and_utility_apps.md](/home/sabir/projects/cortexos/docs/specs/17g_clock_and_utility_apps.md)

## Games Specs

- [18_games_platform_parent.md](/home/sabir/projects/cortexos/docs/specs/18_games_platform_parent.md)
- [18a_solitaire.md](/home/sabir/projects/cortexos/docs/specs/18a_solitaire.md)
- [18b_minesweeper.md](/home/sabir/projects/cortexos/docs/specs/18b_minesweeper.md)
- [18c_snake.md](/home/sabir/projects/cortexos/docs/specs/18c_snake.md)
- [18d_tetris_like_puzzle_game.md](/home/sabir/projects/cortexos/docs/specs/18d_tetris_like_puzzle_game.md)
- [18e_chess_or_checkers.md](/home/sabir/projects/cortexos/docs/specs/18e_chess_or_checkers.md)

## Higher-Layer Platform Specs

- [19_ai_system_surfaces_and_ux.md](/home/sabir/projects/cortexos/docs/specs/19_ai_system_surfaces_and_ux.md)
- [20_ai_action_permissions_and_safety_controls.md](/home/sabir/projects/cortexos/docs/specs/20_ai_action_permissions_and_safety_controls.md)
- [21_sdk_manifest_third_party_app_platform.md](/home/sabir/projects/cortexos/docs/specs/21_sdk_manifest_third_party_app_platform.md)
- [22_admin_diagnostics_recovery.md](/home/sabir/projects/cortexos/docs/specs/22_admin_diagnostics_recovery.md)
- [23_release_readiness_qa_acceptance_framework.md](/home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md)

## Appendices And Meta Docs

- [appendix_a_required_ai_settings_fields.md](/home/sabir/projects/cortexos/docs/specs/appendix_a_required_ai_settings_fields.md)
- [appendix_b_minimum_first_party_app_list.md](/home/sabir/projects/cortexos/docs/specs/appendix_b_minimum_first_party_app_list.md)
- [appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)
- [appendix_d_coding_agent_guardrails.md](/home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md)
- [cortex-prompt.md](/home/sabir/projects/cortexos/docs/specs/cortex-prompt.md)
- [spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md)

## Implementation Planning Docs

- [IMPLEMENTATION_KICKOFF.md](/home/sabir/projects/cortexos/IMPLEMENTATION_KICKOFF.md): execution order, operating rules, and phase entry logic
- [IMPLEMENTATION_PHASES.md](/home/sabir/projects/cortexos/IMPLEMENTATION_PHASES.md): all phases from Phase 0 through Phase 17
- [IMPLEMENTATION_PHASE_CHECKLISTS.md](/home/sabir/projects/cortexos/IMPLEMENTATION_PHASE_CHECKLISTS.md): detailed checklist summary for Phases 2 through 17

## Change Requests And Audit

- [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md): post-implementation audit findings, severity ratings, and triage order

## Phase Checklists

### Phase 1

- [PHASE1_BOOTSTRAP_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE1_BOOTSTRAP_CHECKLIST.md)

### Phase 2 Through Phase 17

- [PHASE2_RUST_FOUNDATION_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE2_RUST_FOUNDATION_CHECKLIST.md)
- [PHASE3_PERSISTENCE_AND_OBSERVABILITY_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE3_PERSISTENCE_AND_OBSERVABILITY_CHECKLIST.md)
- [PHASE4_IDENTITY_AND_POLICY_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE4_IDENTITY_AND_POLICY_CHECKLIST.md)
- [PHASE5_SETTINGS_AND_FILESYSTEM_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE5_SETTINGS_AND_FILESYSTEM_CHECKLIST.md)
- [PHASE6_AI_RUNTIME_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE6_AI_RUNTIME_CHECKLIST.md)
- [PHASE7_COMMAND_BUS_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE7_COMMAND_BUS_CHECKLIST.md)
- [PHASE8_WINDOW_MANAGER_AND_SHELL_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE8_WINDOW_MANAGER_AND_SHELL_CHECKLIST.md)
- [PHASE9_APP_RUNTIME_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE9_APP_RUNTIME_CHECKLIST.md)
- [PHASE10_CORE_SYSTEM_SERVICES_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE10_CORE_SYSTEM_SERVICES_CHECKLIST.md)
- [PHASE11_THEME_AND_ACCESSIBILITY_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE11_THEME_AND_ACCESSIBILITY_CHECKLIST.md)
- [PHASE12_FIRST_PARTY_APPS_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE12_FIRST_PARTY_APPS_CHECKLIST.md)
- [PHASE13_GAMES_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE13_GAMES_CHECKLIST.md)
- [PHASE14_AI_UX_AND_SAFETY_UX_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE14_AI_UX_AND_SAFETY_UX_CHECKLIST.md)
- [PHASE15_SDK_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE15_SDK_CHECKLIST.md)
- [PHASE16_ADMIN_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE16_ADMIN_CHECKLIST.md)
- [PHASE17_RELEASE_GATES_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE17_RELEASE_GATES_CHECKLIST.md)

## Working Notes (Non-Authoritative)

- [open-task-list.md](/home/sabir/projects/cortexos/open-task-list.md): working task-tracking notes. **Non-authoritative.** See [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md) for current status.

## Recommended Navigation Paths

If you are:

- starting the project: read [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md), then [PHASE1_BOOTSTRAP_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE1_BOOTSTRAP_CHECKLIST.md)
- auditing architecture: start with [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md), [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md), and [spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md)
- checking current issues: see [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md)
- implementing a subsystem: read the owning spec, then the matching phase checklist
- prompting Claude Code: use [CLAUDE_CODE_SINGLE_PROMPT.md](/home/sabir/projects/cortexos/CLAUDE_CODE_SINGLE_PROMPT.md)

## Maintenance Rule

When a new top-level planning or guidance document is added, this index should be updated in the same documentation pass.
