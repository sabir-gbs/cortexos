# FOLLOW_UP_CR-003: Live Files API Contract Still Drifts From First-Party App Usage

> **Status: Resolved (2026-03-31).** This CR is now archival. REST-style file routes (`GET/PUT /api/v1/files/{path}`) added to match text-editor-app contract. File-manager-app uses query-style `/api/v1/files/list` which also has a matching server route.

Severity: Critical

## Problem

At least one first-party app now uses a live API contract that the server does not implement. This was hidden by mock-heavy app tests.

## Evidence

Text Editor now calls:

- `GET /api/v1/files/{path}`
- `PUT /api/v1/files/{path}`

See:

- [apps/text-editor-app/src/App.tsx](/home/sabir/projects/cortexos/apps/text-editor-app/src/App.tsx)

But the server exposes:

- `GET /api/v1/files/list`
- `GET /api/v1/files/read`
- `POST /api/v1/files/write`
- `DELETE /api/v1/files/delete`
- `POST /api/v1/files/move`

See:

- [crates/cortex-server/src/main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs)

This means Text Editor’s real file load/save path does not match the live backend route contract.

## Why This Matters

This is a live integration break in a core first-party app, and it demonstrates that the current test strategy does not sufficiently exercise real client/server contracts.

## Requested Change

- reconcile the files API contract between first-party apps and the live server
- add integration coverage that exercises the real file API from app-facing code
- audit other first-party apps for similar “mock-green, live-broken” route mismatches

## Related Earlier CRs

- [CR-007](./CR-007-first-party-app-and-shell-placeholder-gaps.md)
- [CR-004](./CR-004-api-route-versioning-and-transport-drift.md)

## Pattern Gap

- `PCR-005` and `PCR-007` were not fully completed
- new under-documented pattern: mock-heavy tests can hide live API drift
