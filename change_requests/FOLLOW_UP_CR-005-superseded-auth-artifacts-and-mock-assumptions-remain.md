# FOLLOW_UP_CR-005: Superseded Auth Artifacts And Mock Assumptions Remain

> **Status: Resolved (2026-03-31).** This CR is now archival. `App.tsx.bak` has been deleted. `token` has been removed from `LoginResponse` and all mock test fixtures. `localStorage.clear()` removed from desktop-shell test setup. Dead `ShellState` interface removed. The original evidence below describes the pre-remediation state.

Severity: Medium

## Problem

The remediation left behind stale artifacts and stale test assumptions from the previous auth model.

## Evidence

Stale source artifact:

- [apps/desktop-shell/src/App.tsx.bak](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx.bak): still contains the old localStorage/token-based auth flow

Stale test assumptions:

- [apps/desktop-shell/src/__tests__/DesktopShell.test.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/__tests__/DesktopShell.test.tsx): mocked login response still includes `token`
- [apps/desktop-shell/src/__tests__/DesktopShell.test.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/__tests__/DesktopShell.test.tsx): test setup still clears `localStorage`
- [apps/desktop-shell/src/__tests__/setup.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/__tests__/setup.ts): still installs a `localStorage` mock for the shell test environment

## Why This Matters

These artifacts make the repo harder to reason about and can mislead future edits or cause the old auth model to leak back in.

## Requested Change

- remove stale backup/source artifacts from superseded implementations
- update tests to reflect the current auth contract precisely
- keep `localStorage` mocks only where they are still needed for non-auth behavior, and document that scope

## Related Earlier CRs

- [CR-005](./CR-005-session-auth-and-websocket-auth-drift.md)
- [CR-012](./CR-012-false-completion-ledger-and-unindexed-doc-gap.md)
