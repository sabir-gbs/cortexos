# FINAL_DELTA_CR-001: Doc Baseline Still Not Fully Reconciled

> **Status: Resolved (2026-03-31).** This CR is now archival. All 8 current-state docs (CLAUDE.md, AGENTS.md, MASTER_IMPLEMENTATION_BRIEF.md, open-task-list.md, IMPLEMENTATION_PHASES.md, INDEX.md, FOLLOW_UP_INDEX, FINAL_AUDIT_INDEX) agree on 544 frontend tests, 381 Rust tests, and "3 Playwright frontend shell smoke tests".

> **Status: Resolved (2026-03-31).** This CR is now archival. All 8 current-state docs now agree on 544 frontend tests and "3 Playwright frontend shell smoke tests" wording.

Severity: Medium

## Problem

The latest report claims all status docs agree, but they still do not match on exact counts and exact E2E wording.

## Evidence

- [CLAUDE.md](/home/sabir/projects/cortexos/CLAUDE.md)
  - says `pnpm test` passes with `539 tests`
  - says E2E is “configured in `e2e/`”
  - says `pnpm e2e` “requires dev server”
- [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md)
  - still says frontend tests pass with `539 tests`
  - says Playwright covers the “shell login flow”
- [FOLLOW_UP_INDEX_2026-03-31.md](/home/sabir/projects/cortexos/change_requests/FOLLOW_UP_INDEX_2026-03-31.md)
  - still says `539 tests`
  - still frames Playwright as configured shell/API coverage
- [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md), [open-task-list.md](/home/sabir/projects/cortexos/open-task-list.md), and [FINAL_AUDIT_INDEX_2026-03-31.md](/home/sabir/projects/cortexos/change_requests/FINAL_AUDIT_INDEX_2026-03-31.md)
  - now say `554` tests and green E2E

## Requested Change

- normalize all current-state docs to the same test count and same gate wording
- distinguish “configured” from “passing”
- distinguish archival audit docs from current-state status docs

## Related CRs

- [FINAL_AUDIT_CR-002](./FINAL_AUDIT_CR-002-status-docs-and-audit-layers-are-internally-inconsistent.md)
