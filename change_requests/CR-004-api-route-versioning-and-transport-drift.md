# CR-004: API Route Versioning And Window-Manager Transport Drift

Severity: Critical

## Problem

The implementation reintroduces client-facing REST mutation patterns and unversioned `/api/...` routes in areas where the reconciled docs specify `/api/v1/...` naming and command-bus-first realtime mutations.

## Evidence

Current server routes in [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs) are unversioned:

- `/api/auth/login`
- `/api/settings`
- `/api/search`
- `/api/wm/windows/{id}/move`
- `/api/wm/windows/{id}/resize`
- `/api/wm/windows/{id}/focus`
- `/api/wm/workspaces/{id}/switch`

Desktop shell client code in [apps/desktop-shell/src/api.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/api.ts) performs direct HTTP mutations for:

- opening windows
- moving windows
- resizing windows
- focusing windows
- workspace switching

The reconciled spec language instead says:

- [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md): API routes should follow `/api/v1/<domain>/<action>`
- [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/docs/guides/MASTER_IMPLEMENTATION_BRIEF.md): window manager client mutations go through the command bus, not ad hoc client REST
- [docs/specs/08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md): browser clients issue realtime window mutations through command bus commands; HTTP is limited to bootstrap/snapshot reads

## Expected Contract

- versioned public API naming
- HTTP for auth/bootstrap/CRUD/admin/snapshots
- command bus for realtime WM mutation operations

## Requested Change

Reconcile the implementation to the documented transport model:

- move desktop-shell realtime WM mutations onto the command bus
- reserve HTTP WM endpoints for bootstrap/snapshot/recovery reads if that remains the contract
- align route naming with the documented `/api/v1/...` convention, or update the docs everywhere if unversioned routes are now intentional

## Verification

- desktop shell no longer uses direct HTTP POST mutations for realtime WM operations
- WM routes and specs agree on which operations are HTTP vs command bus
- public route naming is consistent across specs, tests, and implementation

## Affected Files

- [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs)
- [apps/desktop-shell/src/api.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/api.ts)
- [docs/specs/08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md)
- [docs/specs/01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
