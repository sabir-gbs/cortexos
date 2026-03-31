# FOLLOW_UP_CR-002: Cookie/Session Remediation Is Not Wired End-To-End

> **Status: Resolved (2026-03-31).** This CR is now archival. Cookie/session auth is wired end-to-end: login sets Set-Cookie, authenticate checks cookies first (with documented bearer-token fallback for SDK consumers), WS checks cookies, logout clears cookie.

Severity: Critical

## Problem

The remediation moved the frontend and docs toward a cookie-based session model, but the live server still authenticates protected routes through Authorization headers and still accepts WebSocket query-token auth. The login route also does not set the session cookie even though the route layer defines a cookie helper.

## Evidence

Frontend/docs now assume cookie auth:

- [apps/desktop-shell/src/api.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/api.ts): uses `credentials: "include"` and no bearer token
- [apps/desktop-shell/src/bus.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/bus.ts): opens `/ws` without token query params
- [crates/cortex-api/src/routes/auth.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/routes/auth.rs): defines `SESSION_COOKIE_NAME` and `session_cookie_value()`

Live server still uses the old auth path:

- [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs): `authenticate()` extracts bearer token from `Authorization`
- [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs): `ws_handler()` still accepts `?token=...`
- [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs): `auth_login_handler()` just returns `to_response(...)`; it does not set `Set-Cookie`

## Why This Matters

This leaves the repo in a split-brain auth state:

- docs say cookie auth
- frontend assumes cookie auth
- backend still expects bearer/query token auth

That means the live system is not actually remediated even though unit tests and mocks may pass.

## Requested Change

- wire login/logout/profile/WS auth through the cookie/session contract end to end
- remove or explicitly deprecate the bearer/query-token fallback paths if cookie auth is canonical
- add integration tests that exercise the live server auth boundary rather than mocked frontend helpers

## Related Earlier CRs

- [CR-005](./CR-005-session-auth-and-websocket-auth-drift.md)
- [CR-004](./CR-004-api-route-versioning-and-transport-drift.md)

## Pattern Gap

- `PCR-006` was not fully completed because the server-side trust boundary still exposes the older auth mechanism
