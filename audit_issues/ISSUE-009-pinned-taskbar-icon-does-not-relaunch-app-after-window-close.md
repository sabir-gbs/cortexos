# ISSUE-009: Pinned taskbar icon does not relaunch app after window close

Severity: High

Status: Open

## Summary

If a user closes an active app window with the `X` button and then interacts with that app's pinned taskbar icon, nothing useful happens. The app is not relaunched.

This is a shell taskbar behavior gap:

- when an app instance/window exists, taskbar click restores or focuses it
- when no instance/window exists, the taskbar does not launch the app

For pinned apps, this leaves the icon visible but non-functional as an app entry point after close.

## Live Reproduction

At `http://localhost:5173` with `admin / uglyducks22!`:

1. open `Settings` from the desktop
2. confirm the `Settings` window is visible
3. click the window `X` button to close it
4. confirm the pinned `Settings` taskbar icon is still present
5. double-click the taskbar `Settings` icon
6. no `Settings` window reopens

Observed result:

- window visible after launch: `true`
- window visible after close: `false`
- taskbar icon still present: `true`
- window visible after taskbar double-click: `false`

## Code Evidence

Taskbar button behavior:

- [Taskbar.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/components/Taskbar.tsx)
  - if `item.instanceId` exists, calls `onAppClick(instanceId)`
  - otherwise calls `onLauncherClick()`

Relevant code path:

```ts
onClick={() => {
  if (item.instanceId) onAppClick(item.instanceId);
  else onLauncherClick();
}}
```

Taskbar app click handler:

- [App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx)
  - `handleTaskbarAppClick(instanceId)` only focuses/restores an existing matching window
  - if no matching window exists, it does nothing

Relevant behavior:

- finds a window by `instance_id`
- restores if minimized
- focuses if found
- no fallback relaunch path

Pinned taskbar population:

- [types.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/types.ts)
  - `settings-app` is pinned by default in `shell.taskbar.pinned_apps`

## Why This Happens

The taskbar currently mixes two concepts:

1. pinned app shortcut
2. running app instance handle

But it only has correct behavior for the running-instance case.

For pinned-but-not-running apps, taskbar click currently falls back to `onLauncherClick()` rather than launching the pinned app directly. On double-click, this can just toggle the launcher open/closed or otherwise appear to do nothing useful.

## Spec Implications

[07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md) explicitly defines:

- pinned and running app icons both appear on the taskbar
- clicking a running app icon focuses the app window

The spec is less explicit about pinned-but-not-running click behavior, but a pinned app icon that does not launch the app is a broken UX contract for a taskbar shortcut.

This also conflicts with the audit/history intent that taskbar should be a real app entry point, not just a passive indicator.

## User Impact

1. pinned taskbar icons are unreliable as launch affordances
2. users must go back to the desktop or launcher to reopen an app they just closed
3. double-clicking the taskbar icon appears broken or ignored
4. the taskbar violates normal desktop-shell expectations

## Recommended Fix

Make pinned taskbar icons launch the app when there is no active/running instance to restore.

Preferred behavior:

1. if app has a live window:
   - focus it
   - restore if minimized
2. if app has no live instance/window:
   - launch a new instance directly from the taskbar icon
3. if app is single-instance and a hidden/stale runtime exists:
   - reconcile against the runtime/window contract and choose one deterministic recovery path

Implementation likely requires:

- passing `appId` into the taskbar app click path, not just `instanceId`
- adding a relaunch fallback when no restorable window exists
- clarifying single-click vs double-click behavior for taskbar icons

## Required Verification

1. open a pinned app from the desktop
2. close it with `X`
3. click the pinned taskbar icon
4. confirm the app relaunches
5. repeat for at least `Settings` and `Terminal`
6. verify running-instance focus/restore still works for already-open windows
7. add regression coverage for the closed-then-taskbar-relaunch path

