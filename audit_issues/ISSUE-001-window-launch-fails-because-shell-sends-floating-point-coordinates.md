# ISSUE-001: Window Launch Fails Because The Shell Sends Floating-Point Coordinates

Severity: Critical

## What Happens

Desktop app launch fails even though login succeeds and the desktop shell loads.

Visible symptom:

- double-clicking `Files`, `Terminal`, or `Settings` does not open an app window
- the shell eventually surfaces a generic launch failure message

## Reproduction

1. Open `http://localhost:5173`
2. Log in as `admin`
3. Double-click a desktop app icon such as `Files`

## Evidence

Live browser trace showed:

- `POST /api/v1/apps/launch` returns `200`
- `POST /api/v1/wm/windows` returns `422`

Observed backend message:

- `Failed to deserialize the JSON body into the target type: x: invalid type: floating point ..., expected i32`

Code path causing it:

- [App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx)
  - launch path uses:
  - `x: 100 + Math.random() * 200`
  - `y: 50 + Math.random() * 150`
- [types.rs](/home/sabir/projects/cortexos/crates/cortex-wm/src/types.rs)
  - `OpenWindowRequest` expects integer-compatible coordinates
- [main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs)
  - `wm_open_window_handler` deserializes directly into `OpenWindowRequest`

## Why It Matters

This is the first blocking bug in the launch flow. No desktop app window can be created from normal shell interaction while this mismatch exists.

## Recommended Fix

- make the shell send integer coordinates for `openWindow`
- use deterministic integer rounding before serialization
- add a frontend unit test that verifies `openWindow` receives integer `x` and `y`
- add an integration test proving a launched app can successfully create a window through the live HTTP path

