# ISSUE-008: Settings app theme controls are local-only and do not update shell theme

Severity: High

Status: Open

## Summary

Theme switching works from the shell-owned settings surface, but not from the standalone `Settings` app window.

The root problem is that the standalone `settings-app` is not wired to the centralized settings service or the desktop shell theme contract. It updates only its own local React state, so the app can repaint itself without changing the OS theme, persisting the setting, or notifying the shell.

## User-Visible Behavior

Observed behavior:

- changing light/dark mode from the shell-owned settings control works
- changing light/dark mode from the `Settings` app window does not update the desktop shell theme

This creates a direct inconsistency between two settings surfaces that should be controlling the same canonical OS setting.

## Spec Contract

Per [05_settings_service_and_settings_app.md](/home/sabir/projects/cortexos/docs/specs/05_settings_service_and_settings_app.md):

- "Every configurable value in the OS has exactly one canonical setting key."
- "The Settings app provides the user-facing interface for managing all settings."
- "User changes a setting | Change applied immediately, propagated to all affected subsystems"
- "User changes theme | Theme switches instantly, no page reload"

The current standalone `settings-app` does not meet that contract.

## Code Evidence

Standalone app behavior:

- [App.tsx](/home/sabir/projects/cortexos/apps/settings-app/src/App.tsx)
  - initializes local state with:
    - `theme: 'dark'`
    - `aiProvider: 'OpenAI'`
    - `aiModel: ''`
  - updates via:
    - `setSettings((prev) => ({ ...prev, [key]: value }))`
  - does not call the settings API
  - does not call the shell API
  - does not send a command-bus event
  - does not communicate with the parent shell iframe host

What is missing:

- no `GET /api/v1/settings/...`
- no `POST/PUT /api/v1/settings...`
- no subscription to `settings.changed`
- no handshake with the desktop shell for live theme propagation

By contrast, the shell-owned settings surface is wired to shell state:

- [App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx)
  - maintains canonical shell settings state
  - applies `theme.mode` to the shell root via `data-theme`
  - persists through `api.setSetting("shell", key, value)`

## Why This Happens

There are effectively two different settings implementations:

1. shell overlay/settings surface:
   - updates the shell’s real settings state
   - persists through the settings API
2. standalone `settings-app`:
   - behaves like a local mock/demo UI
   - holds its own isolated `useState`

So the standalone window is not controlling the same canonical source of truth.

## User Impact

This causes several real product problems:

1. users see contradictory behavior between two settings surfaces
2. the Settings app appears broken or deceptive
3. changes made in the app are not persisted
4. other subsystems do not receive propagated changes
5. the repo violates the spec’s "one canonical setting key" rule

## Recommended Fix

The standalone `settings-app` needs to become a real client of the settings service.

Preferred direction:

1. make `settings-app` read effective settings from the canonical settings API on load
2. make it write changes through the canonical settings API
3. make it subscribe to settings changes, or otherwise re-read effective values after writes
4. ensure theme changes propagate immediately to the shell host

Possible implementation paths:

- direct settings API integration inside `settings-app`
- plus a shell/theme propagation mechanism for the iframe-hosted shell

Important constraint:

- do not solve this by creating a second theme state in the app window
- do not treat the standalone settings app as a special local-only exception unless the spec is explicitly changed

## Required Verification

1. open the `Settings` app window from the desktop
2. change theme from dark to light and back
3. confirm the desktop shell theme changes immediately
4. refresh/reload and confirm the chosen theme persists
5. confirm the shell-owned settings surface and the standalone settings app stay in sync
6. add regression coverage for the standalone settings-app theme flow

