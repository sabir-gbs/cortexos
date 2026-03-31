# Spec Audit — 2026-03-30

This document records the current post-reconciliation state of the CortexOS spec set after the original three-round audit and three additional reconciliation passes.

Audit goal:

- verify that critical, high, medium, and low priority gaps were either resolved or reduced to explicit residual risks
- focus on implementation stability for Claude Code and GLM-5.1-driven development
- ensure the audit matches the current spec state rather than the pre-reconciliation baseline

Scope reviewed:

- `docs/specs/00_master_spec.md` through `docs/specs/23_release_readiness_qa_acceptance_framework.md`
- appendices A through D
- `docs/specs/cortex-prompt.md`

## Overall Result

The spec pack is now materially more stable than it was in the initial audit.

Current assessment:

- no open critical gaps found
- no open high-priority cross-spec contradictions found
- no open medium-priority underspecification gaps found that are likely to force agent improvisation
- one low-priority follow-up remains if first-class runtime GLM/Zhipu deployment profiles need concrete examples

The remaining work is no longer foundational reconciliation. It is mostly implementation detail, examples, and future hardening.

## Reconciliation Summary

The following issue classes were reconciled in the specs:

- browser/backend transport normalized to a hybrid model:
  - HTTP/REST for auth, bootstrap, CRUD, admin, and snapshot reads
  - WebSocket command bus for realtime commands, events, and streaming
- persistent state authority normalized:
  - SQLite is canonical for structured state
  - filesystem is reserved for blobs, exports, crash artifacts, and package payloads
- command bus typing contract repaired:
  - typed Rust command/event interfaces
  - typed TypeScript subscription surface
  - durable idempotency records for command replay/restart safety
- AI ownership normalized around:
  - `cortex-ai`
  - `cortex-policy`
  - `@cortexos/ai-client`
- keyboard shortcut conflicts removed:
  - command palette `Ctrl+Space`
  - assistant `Ctrl+Shift+A`
- event naming drift fully resolved across specs 07, 08, 10, 13 (CR-013 applied 2026-03-31):
  - workspace events: `wm.workspace.changed` (canonical in spec 07), `workspace.activated` (spec 08 emits on workspace switch)
  - workspace command: `wm.workspace.switch` (spec 07), `wm.workspace.activate` (spec 08 command section)
  - focus event: `wm.focus.changed` (canonical across specs 07 and 08)
  - notification events: `notification.created`, `notification.dismissed` (spec 10, 13); `notification.read`, `notification.all_read`, `notification.expired` (spec 10, 13)
  - desktop shell code (`App.tsx`) updated: `notification.read` -> `notification.dismissed`, `notification.all_read` -> `notification.all_dismissed`, `workspace.changed` -> `wm.workspace.changed`
  - `settings.changed`
  - `file.modified`
  - `app.stopped`
- AI settings registry completed in appendix A and spec 05
- no-provider-configured AI behavior made deterministic
- observability/admin log storage model aligned around stdout plus bounded searchable mirror
- AI conversations, grants, audit trails, registry records, session state, and admin metrics aligned to SQLite-backed canonical state
- window manager transport aligned to command-bus mutations plus HTTP snapshot/bootstrap reads
- chess draw rules completed
- Tetris spawn, lock-delay, DAS/ARR, and top-out behavior completed
- specs 21 and 22 promoted from Draft to Implementation-grade
- `Zhipu` added as a first-class runtime provider enum across the core AI settings/runtime docs

## Priority Review

### Critical

Resolved:

1. Transport contract mismatch between command-bus-only language and REST-heavy child specs.
2. Persistent state authority split between SQLite and ad hoc file storage.
3. Command bus typed-payload contradiction.

Open:

- none

### High

Resolved:

1. App/package inventory mismatch across `00`, `01`, and appendix B.
2. AI subsystem ownership mismatch across specs 19, 20, and 21.
3. Keyboard shortcut conflicts for command palette and AI assistant.
4. File-content storage contradiction between `00` and `11`.
5. AI audit/conversation storage contradiction across `06`, `19`, `20`, and `22`.
6. WebSocket authentication conflict with the session-cookie security model.
7. Hardcoded default AI provider fallback conflicting with optional-AI behavior.
8. Appendix A incompleteness despite claiming authority.
9. Observability/admin log storage model mismatch.

Open:

- none

### Medium

Resolved:

1. Event naming drift across shell/runtime/files/settings/notifications/search — fully resolved (CR-013, 2026-03-31).
2. Window-manager transport ambiguity for client-facing operations.
3. Command idempotency semantics across restart/replay windows.
4. Missing standard chess draw conditions.
5. Missing Tetris feel-defining defaults.
6. Admin metrics ownership ambiguity.

Open:

- none

### Low

Resolved:

1. Draft status lingering on implementation-boundary specs.
2. Missing explicit Zhipu/Z.ai runtime-provider support at the enum/settings level.

Open:

1. Optional polish: add a concrete Zhipu / GLM runtime provider profile example in spec 06 or appendix A if CortexOS intends to ship a documented default endpoint/model pair rather than only first-class enum support.

Why this is low:

- the provider is now represented canonically in the runtime and settings contracts
- implementation can proceed without guessing
- the remaining gap is example/configuration polish, not an architectural blocker

## Files Reconciled In The Additional Passes

- `docs/specs/07_desktop_shell.md`
- `docs/specs/08_window_manager.md`
- `docs/specs/10_system_command_bus_and_event_model.md`
- `docs/specs/15_accessibility_input_keyboard_system.md`
- `docs/specs/18d_tetris_like_puzzle_game.md`
- `docs/specs/18e_chess_or_checkers.md`
- `docs/specs/20_ai_action_permissions_and_safety_controls.md`
- `docs/specs/21_sdk_manifest_third_party_app_platform.md`
- `docs/specs/22_admin_diagnostics_recovery.md`
- `docs/specs/00_master_spec.md`
- `docs/specs/05_settings_service_and_settings_app.md`
- `docs/specs/06_ai_runtime_provider_registry_preferred_llm_model_routing.md`
- `docs/specs/appendix_a_required_ai_settings_fields.md`
- `docs/specs/13_notifications_service.md`
- `apps/desktop-shell/src/App.tsx`

## Recommendation

The spec set is now stable enough to support implementation work without forcing frequent agent improvisation on core platform contracts.

Recommended next step:

1. Freeze the reconciled platform contracts (`00`, `01`, `05`, `06`, `08`, `10`, `14`, `19`, `20`, `21`, `22`).
2. Start scaffold/implementation work from those contracts.
3. Only add one optional follow-up doc pass if you want explicit Zhipu / GLM runtime configuration examples.
