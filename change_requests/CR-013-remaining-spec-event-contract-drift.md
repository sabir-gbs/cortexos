# CR-013: Remaining Spec/Event-Name Drift Still Exists Across Shell, WM, Bus, And Notifications Docs

Severity: High

## Problem

The earlier reconciliation/audit documents claim event naming drift was resolved, but at least one important cross-spec contradiction still remains in the current docs and is reflected inconsistently in the implementation.

## Evidence

Notifications event contract drift:

- [docs/specs/10_system_command_bus_and_event_model.md](/home/sabir/projects/cortexos/docs/specs/10_system_command_bus_and_event_model.md): `notification.created`, `notification.dismissed`
- [docs/specs/13_notifications_service.md](/home/sabir/projects/cortexos/docs/specs/13_notifications_service.md): `notification.created`, `notification.dismissed`
- [docs/specs/07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md): includes `notification.read` and `notification.all_read`
- [apps/desktop-shell/src/App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx): handles `notification.read` and `notification.all_read`

Window-manager event/command drift:

- [docs/specs/08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md): uses `workspace.changed` and `wm.workspace.activate`
- [docs/specs/07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md): uses `wm.desktop.changed`, `wm.focus.changed`, and `wm.desktop.switch`

Audit overstatement:

- [docs/specs/spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md): says event naming drift was resolved

## Expected Contract

Cross-subsystem bus/event names must be canonical and identical across:

- the shell spec
- the window manager spec
- the bus/event spec
- the notifications spec
- implementation code
- tests

## Requested Change

- reconcile the remaining event-name and command-name differences
- update the stale audit language that says this class of drift is fully resolved
- align desktop-shell implementation and tests to the canonical names once decided

## Verification

- one canonical name exists for each shell/WM/notification event and command
- specs 07, 08, 10, and 13 agree exactly
- desktop-shell implementation and tests subscribe/emit only the canonical names

## Affected Files

- [docs/specs/07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md)
- [docs/specs/08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md)
- [docs/specs/10_system_command_bus_and_event_model.md](/home/sabir/projects/cortexos/docs/specs/10_system_command_bus_and_event_model.md)
- [docs/specs/13_notifications_service.md](/home/sabir/projects/cortexos/docs/specs/13_notifications_service.md)
- [docs/specs/spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md)
- [apps/desktop-shell/src/App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx)
