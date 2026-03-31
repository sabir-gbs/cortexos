# CR-005: Session Authentication Model Drifts From The Spec And Weakens The Trust Boundary

Severity: Critical

## Problem

The implementation is using bearer tokens in response bodies, browser `localStorage`, Authorization headers, and WebSocket query-string tokens, while the reconciled auth spec defines a server-side session store with HTTP-only secure cookies.

## Evidence

Spec contract:

- [docs/specs/03_identity_authentication_sessions_user_profiles.md](/home/sabir/projects/cortexos/docs/specs/03_identity_authentication_sessions_user_profiles.md): login flow sets `cortex_session=<token>` as an HTTP-only, Secure, SameSite=Strict cookie; session validation extracts the token from the cookie
- [docs/specs/spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md): WebSocket authentication conflict with the session-cookie model was listed as resolved

Implementation drift:

- [crates/cortex-api/src/routes/auth.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/routes/auth.rs): login response includes `token` in the JSON body
- [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs): authenticated routes extract bearer tokens from headers
- [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs): WebSocket handshake accepts `?token=...`
- [apps/desktop-shell/src/App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx): stores token and user in `localStorage`
- [apps/desktop-shell/src/api.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/api.ts): sends `Authorization: Bearer <token>`
- [apps/desktop-shell/src/bus.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/bus.ts): connects to `/ws?token=...`

## Expected Contract

- browser should not persist the session token in `localStorage`
- session validation should be cookie-based per spec
- WebSocket auth should not rely on query-string token transport if the resolved contract is cookie/session based

## Requested Change

Reconcile auth/session handling end to end:

- eliminate browser-side token persistence in `localStorage` for auth state
- stop returning the raw session token to the client if the cookie model remains authoritative
- move HTTP auth enforcement to the session cookie path or revise the docs if a bearer-token model is now the intended design
- remove query-string token auth from the WebSocket path if the cookie-based model remains authoritative

## Verification

- auth/session flows match spec 03 exactly, or the spec is updated explicitly and consistently
- desktop shell tests no longer assume stored auth tokens in `localStorage`
- WebSocket handshake no longer requires a query parameter token under the cookie model

## Affected Files

- [docs/specs/03_identity_authentication_sessions_user_profiles.md](/home/sabir/projects/cortexos/docs/specs/03_identity_authentication_sessions_user_profiles.md)
- [crates/cortex-api/src/routes/auth.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/routes/auth.rs)
- [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs)
- [apps/desktop-shell/src/App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx)
- [apps/desktop-shell/src/api.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/api.ts)
- [apps/desktop-shell/src/bus.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/bus.ts)
