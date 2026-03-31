# FINAL_DELTA_CR-002: Playwright Scope Is Green But Narrower Than Claimed

> **Status: Resolved (2026-03-31).** This CR is now archival. Docs now accurately describe Playwright as "frontend shell smoke tests" (3 tests, no backend required). Backend-integrated auth coverage lives in Rust HTTP integration tests.

Severity: Medium

## Problem

`pnpm e2e` now passes, but the suite no longer proves the backend-integrated behavior that some docs and completion text imply.

## Evidence

- [e2e/tests/api-health.spec.ts](/home/sabir/projects/cortexos/e2e/tests/api-health.spec.ts)
  - now checks only that the frontend dev server serves the index page
  - it no longer validates `/api/v1/health`
- [e2e/tests/shell.spec.ts](/home/sabir/projects/cortexos/e2e/tests/shell.spec.ts)
  - verifies login screen rendering
  - verifies login submission fails cleanly when backend is unavailable
  - does not verify successful authenticated shell bootstrap against a live backend
- [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md)
  - says Playwright covers the “shell login flow”
- [FOLLOW_UP_CR-004](./FOLLOW_UP_CR-004-release-gates-still-missing-real-browser-e2e-and-cargo-audit.md)
  - archival header says Playwright has 3 passing tests covering shell login flow

## Requested Change

- either narrow the docs to describe Playwright as frontend-shell smoke coverage
- or expand Playwright so it actually provisions the backend and verifies a successful login/bootstrap path

## Related CRs

- [FINAL_AUDIT_CR-001](./FINAL_AUDIT_CR-001-playwright-e2e-is-configured-but-not-green.md)
- [FOLLOW_UP_CR-004](./FOLLOW_UP_CR-004-release-gates-still-missing-real-browser-e2e-and-cargo-audit.md)
