# ISSUE-002: Successful App Launches Can Leak Orphan Runtime Instances When Window Creation Fails

Severity: High

## What Happens

The shell launches an app instance first and only then requests a window. When window creation fails, the launched runtime instance is left running without a window.

## Evidence

Live flow:

1. `POST /api/v1/apps/launch` succeeds
2. `POST /api/v1/wm/windows` fails
3. the runtime instance remains in `GET /api/v1/apps/running`

Observed running-instance list contained multiple leaked entries with:

- `state: "running"`
- `window_id: null`

Relevant code:

- [App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx)
  - `handleLaunchApp` launches the runtime instance first
  - if `openWindow` fails, there is no rollback via `stopApp`
- [api.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/api.ts)
  - separate `launchApp` and `openWindow` calls

## Why It Matters

Repeated launch attempts accumulate hidden running instances, which can distort taskbar/runtime state and make later recovery harder.

## Recommended Fix

- make launch + window creation atomic at the platform contract level, or
- if window creation fails after launch, immediately call `stopApp(instance_id)` as compensation
- add a regression test asserting that failed window creation does not leave `window_id: null` running instances behind

