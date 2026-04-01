# ISSUE-007: Minimized window remains focused and focus does not transfer

Severity: High

Status: Open

## Summary

Window minimization is only partially correct today.

The minimize action hides the window visually, but the backend keeps the minimized window marked as focused and does not promote another visible window. This violates the window-manager spec and leaves the frontend with an inconsistent focus model:

- the minimized window is hidden
- the minimized window is still logically focused
- no other visible window becomes focused

## Spec Contract

Per [08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md):

- "Exactly one window is focused at a time (or none if all minimized)"
- "Minimized windows lose focus; focus moves to next topmost window"
- "User minimizes | Window shrinks to taskbar icon, no longer visible"

The current implementation satisfies the visibility part, but not the focus-transfer rules.

## Live Evidence

### Browser behavior

Against the live shell at `http://localhost:5173`:

1. login succeeds
2. open multiple windows, e.g. `Settings` and `Terminal`
3. minimize the currently focused window
4. the minimized window disappears visually
5. but focus does not move to the next visible topmost window as required

### Direct API reproduction

Open two windows, then minimize the currently focused one.

Before minimize:

```json
[
  {
    "title": "Terminal",
    "state": "normal",
    "focused": true,
    "z_index": 9
  },
  {
    "title": "Settings",
    "state": "normal",
    "focused": false,
    "z_index": 8
  }
]
```

Minimize response:

```json
{
  "title": "Terminal",
  "state": "minimized",
  "focused": true,
  "z_index": 9
}
```

After minimize:

```json
[
  {
    "title": "Terminal",
    "state": "minimized",
    "focused": true,
    "z_index": 9
  },
  {
    "title": "Settings",
    "state": "normal",
    "focused": false,
    "z_index": 8
  }
]
```

This is the core defect: a minimized window remains focused.

## Code Evidence

Server-side state transition:

- [sqlite.rs](/home/sabir/projects/cortexos/crates/cortex-wm/src/sqlite.rs)
  - `minimize_window()` calls `update_state(window_id, "minimized")`
  - `update_state()` only changes the `state` column
  - it does not clear `focused`
  - it does not promote the next eligible visible window in the workspace

Frontend propagation:

- [App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx)
  - `handleWindowMinimize()` applies the server response directly into local state

Taskbar rendering:

- [Taskbar.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/components/Taskbar.tsx)
  - taskbar focus indication depends on `windows[].focused`

So the shell is not inventing bad focus locally; it is rendering the inconsistent backend state it receives.

## User Impact

This creates several user-visible problems:

1. the focused app indicator can remain attached to a hidden minimized window
2. the next visible window is not promoted as active
3. keyboard interactions after minimize may target the wrong logical window state
4. the desktop can violate the spec rule that focus is either on one visible window or on none when all are minimized

## Recommended Fix

Fix this in the window-manager service first, not just in the frontend.

Preferred behavior:

1. when minimizing a focused window:
   - clear its `focused` flag
   - find the next topmost non-closed, non-minimized window in the same workspace
   - promote that window to `focused = true`
2. when minimizing a non-focused window:
   - do not disturb the currently focused visible window
3. when all windows in the workspace are minimized:
   - no window should be focused

Frontend follow-through:

- confirm the shell state stays aligned with the backend response
- confirm the taskbar focus indicator updates correctly after minimize

## Required Verification

1. open at least two windows on the same workspace
2. minimize the focused window
3. confirm:
   - minimized window is hidden
   - minimized window is not focused
   - next topmost visible window becomes focused
4. minimize all visible windows
5. confirm no window remains focused
6. restore a minimized window from the taskbar
7. confirm it becomes visible and focused again
8. add regression tests at the window-manager layer and desktop-shell integration layer

