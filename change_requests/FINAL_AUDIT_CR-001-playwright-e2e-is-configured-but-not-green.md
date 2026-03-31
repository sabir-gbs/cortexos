# FINAL_AUDIT_CR-001: Playwright E2E Is Configured But Not Green

> **Status: Resolved (2026-03-31).** This CR is now archival. All 3 Playwright frontend shell smoke tests pass consistently. Locators are precise (`getByRole`, `getByLabel`). Index page test validates frontend dev server response. Login error test asserts exact error message content in `role="alert"`.

Severity: Critical

## Problem

The repo now contains Playwright browser tests and a CI job, but the E2E gate is not actually passing in the local repo state. The completion report therefore overstates release-gate completeness.

## Evidence

Command result:

- `pnpm e2e` exits non-zero

Observed failures:

- [e2e/tests/api-health.spec.ts](/home/sabir/projects/cortexos/e2e/tests/api-health.spec.ts)
  - `GET /api/v1/health` fails through the frontend proxy
  - Playwright run shows Vite proxy `ECONNREFUSED`
- [e2e/tests/shell.spec.ts](/home/sabir/projects/cortexos/e2e/tests/shell.spec.ts)
  - “shows login screen when not authenticated” fails because `getByText(/login|sign in/i)` resolves to multiple elements

Supporting configuration drift:

- [e2e/playwright.config.ts](/home/sabir/projects/cortexos/e2e/playwright.config.ts)
  - starts only `pnpm --filter desktop-shell dev`
  - does not start the backend server required by `/api/v1/health`

## Why This Matters

“Configured” is not the same as “release-gate ready.” Until the browser E2E path is self-contained and green, the repo does not satisfy the completion claim made in the final report.

## Requested Change

- make `pnpm e2e` green in a clean local run
- ensure the Playwright setup provisions all required services, not just the frontend dev server
- tighten test assertions so they verify the intended UI state without ambiguous locators
- do not mark browser E2E complete until the executable gate is consistently passing

## Related Existing CRs

- [FOLLOW_UP_CR-004](./FOLLOW_UP_CR-004-release-gates-still-missing-real-browser-e2e-and-cargo-audit.md)
- [CR-002](./CR-002-release-gates-and-ci-mismatch.md)

## Pattern Gap

- `PCR-003` Non-Executable Quality Gates remains only partially resolved
