# CR-009: Command Bus And Runtime Implementations Still Contain Placeholder Semantics And Unsafe Fallbacks

Severity: High

## Problem

Several foundational implementation paths still behave like provisional scaffolding rather than completed subsystem logic. This is especially risky in the command bus, runtime, and persistence layers because higher-level correctness depends on them.

## Evidence

Command bus / WebSocket placeholder semantics:

- [crates/cortex-api/src/ws.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/ws.rs): explicitly says “Here we acknowledge receipt and publish a placeholder event.”

Runtime placeholder semantics:

- [crates/cortex-runtime/src/sqlite.rs](/home/sabir/projects/cortexos/crates/cortex-runtime/src/sqlite.rs): `simulate immediate start success`

Unsafe or lossy fallbacks in WM persistence:

- [crates/cortex-wm/src/sqlite.rs](/home/sabir/projects/cortexos/crates/cortex-wm/src/sqlite.rs): invalid UUID parsing falls back via `unwrap_or_default()`, which can silently coerce bad persisted data instead of surfacing corruption/errors

Additional smell:

- [crates/cortex-api/src/routes/ai.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/routes/ai.rs): uses `unwrap_or_default()` in response serialization paths

## Expected Contract

Core subsystem implementations should:

- implement the real command-dispatch semantics required by spec 10
- avoid placeholder acknowledgements in production paths
- fail explicitly on invalid persisted data rather than silently defaulting

Relevant docs:

- [docs/specs/09_app_runtime_and_app_lifecycle.md](/home/sabir/projects/cortexos/docs/specs/09_app_runtime_and_app_lifecycle.md)
- [docs/specs/10_system_command_bus_and_event_model.md](/home/sabir/projects/cortexos/docs/specs/10_system_command_bus_and_event_model.md)
- [docs/specs/08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md)
- [docs/specs/appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)

## Requested Change

- replace placeholder bus acknowledgement logic with real handler dispatch semantics
- remove simulated runtime-success paths from core lifecycle handling
- replace silent persistence fallbacks with explicit error handling
- audit other `unwrap_or_default()` usage in persistence and API boundary code for silent corruption masking

## Verification

- command bus tests cover real dispatch semantics rather than only placeholder acknowledgements
- runtime lifecycle reflects actual state transitions and failures
- corrupted persistence rows fail loudly and observably

## Affected Files

- [crates/cortex-api/src/ws.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/ws.rs)
- [crates/cortex-runtime/src/sqlite.rs](/home/sabir/projects/cortexos/crates/cortex-runtime/src/sqlite.rs)
- [crates/cortex-wm/src/sqlite.rs](/home/sabir/projects/cortexos/crates/cortex-wm/src/sqlite.rs)
- [crates/cortex-api/src/routes/ai.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/routes/ai.rs)
