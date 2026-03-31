# FOLLOW_UP_CR-006: Event And Workspace Contract Reconciliation Is Still Incomplete

> **Status: Resolved (2026-03-31).** This CR is now archival. `wm.workspace.changed` is canonical across specs 07, 08, 10, bus.rs, and App.tsx.

Severity: High

## Problem

The event/workspace naming contract still has unresolved drift across specs, audit docs, and implementation.

## Evidence

Current drift examples:

- [docs/specs/08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md): `wm.workspace.activate`, `workspace.changed`
- [docs/specs/07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md): `wm.workspace.changed`
- [docs/specs/10_system_command_bus_and_event_model.md](/home/sabir/projects/cortexos/docs/specs/10_system_command_bus_and_event_model.md): still lists `workspace.changed`
- [docs/specs/spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md): documents a different reconciliation narrative
- [apps/desktop-shell/src/App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx): handles `wm.workspace.changed`
- [crates/cortex-api/src/bus.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/bus.rs): canonical constant is `wm.workspace.changed`

## Why This Matters

This remains a public command/event contract. Partial reconciliation is worse than none because some docs now imply the naming issue is solved when it is not solved everywhere.

## Requested Change

- choose one canonical workspace command/event naming scheme
- update all affected specs and audit docs to match
- ensure implementation constants/tests remain aligned with that one canonical scheme

## Related Earlier CRs

- [CR-013](./CR-013-remaining-spec-event-contract-drift.md)

## Pattern Gap

- `PCR-005` remains incomplete
