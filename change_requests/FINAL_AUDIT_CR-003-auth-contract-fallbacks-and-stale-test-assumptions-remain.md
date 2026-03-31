# FINAL_AUDIT_CR-003: Auth Contract Fallbacks And Stale Test Assumptions Remain

> **Status: Resolved (2026-03-31).** This CR is now archival. Bearer and query-token fallbacks are explicitly documented as transitional compatibility paths in `main.rs`. DesktopShell tests updated: `localStorage.clear()` removed, test descriptions reflect cookie-based auth model. E2E login test asserts precise error message.

Severity: High

## Problem

Cookie/session auth has been added, but the repo still retains legacy compatibility paths and stale test assumptions from the older token-centric model. That means the remediation is incomplete if the intended contract is a clean cookie-authoritative boundary.

## Evidence

Legacy compatibility still present in the live server:

- [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs)
  - `auth_logout_handler` accepts `extract_cookie_token(...).or_else(|| extract_token(...))`
  - WebSocket auth still supports query-token input through `WsParams { token: Option<String> }`
  - auth helpers still include `extract_token`

Stale auth-era test assumptions remain:

- [apps/desktop-shell/src/__tests__/DesktopShell.test.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/__tests__/DesktopShell.test.tsx)
  - still clears `localStorage` in every test
  - still uses wording like “shows login screen when no token is stored”
  - still uses comments like “Simulate cookie-based session by triggering login,” reflecting a partially transitioned mental model
- [e2e/tests/shell.spec.ts](/home/sabir/projects/cortexos/e2e/tests/shell.spec.ts)
  - treats either desktop success or backend failure/error as acceptable after login form submission, which weakens the value of the test as a contract check

## Why This Matters

The remediation was supposed to remove stale auth-model drift. Leaving both the old and new paths in place makes it harder to reason about the intended boundary and weakens future audits.

## Requested Change

- decide explicitly whether bearer-header and query-token paths are still supported or should be removed
- if they remain intentionally, document them as compatibility behavior instead of calling the migration complete
- update unit and E2E tests so they assert the current auth contract precisely rather than carrying legacy token-era assumptions

## Related Existing CRs

- [FOLLOW_UP_CR-002](./FOLLOW_UP_CR-002-cookie-auth-remediation-not-wired-end-to-end.md)
- [FOLLOW_UP_CR-005](./FOLLOW_UP_CR-005-superseded-auth-artifacts-and-mock-assumptions-remain.md)
- [CR-005](./CR-005-session-auth-and-websocket-auth-drift.md)

## Pattern Gap

- `PCR-006` Security Boundary Erosion Through Client Authority remains partially unresolved
- `PCR-007` Placeholder Or Mock Logic In Core User Flows remains partially unresolved
