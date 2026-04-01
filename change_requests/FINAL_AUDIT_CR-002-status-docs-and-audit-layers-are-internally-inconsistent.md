# FINAL_AUDIT_CR-002: Status Docs And Audit Layers Are Internally Inconsistent

> **Status: Resolved (2026-03-31).** This CR is now archival. All top-level status docs (AGENTS.md, MASTER_IMPLEMENTATION_BRIEF.md, CLAUDE.md, open-task-list.md, INDEX.md, FOLLOW_UP_INDEX) agree on the same current-state baseline. Individual FOLLOW_UP_CR docs have archival headers.

Severity: High

## Problem

The repo now contains conflicting claims about current status. Some docs still say browser E2E and `cargo audit` are missing, while other docs say they are complete. This makes the documentation set unreliable as an execution reference.

## Evidence

Still-stale status docs:

- [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md)
  - says `cargo audit` is not yet run in CI
  - says no real browser automation E2E exists
- [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/docs/guides/MASTER_IMPLEMENTATION_BRIEF.md)
  - says `cargo audit` is not yet in CI
  - says no real browser E2E exists
- [open-task-list.md](/home/sabir/projects/cortexos/docs/status/open-task-list.md)
  - repeats the same outdated claims

Newer docs claiming completion:

- [CLAUDE.md](/home/sabir/projects/cortexos/CLAUDE.md)
  - says `cargo audit` passes
  - says Playwright E2E exists in `e2e/`
- [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md)
  - says all follow-up CRs are resolved
- [change_requests/FOLLOW_UP_INDEX_2026-03-31.md](/home/sabir/projects/cortexos/change_requests/FOLLOW_UP_INDEX_2026-03-31.md)
  - says all follow-up CRs are fully resolved

Archive drift also remains:

- [FOLLOW_UP_CR-004](./FOLLOW_UP_CR-004-release-gates-still-missing-real-browser-e2e-and-cargo-audit.md)
  - still describes the pre-remediation state
- [FOLLOW_UP_CR-005](./FOLLOW_UP_CR-005-superseded-auth-artifacts-and-mock-assumptions-remain.md)
  - still cites the removed `App.tsx.bak` file as live evidence

## Why This Matters

The user asked for a single authoritative documentation stack to guide Claude Code. That stack cannot be trusted if key status docs disagree about what is actually done.

## Requested Change

- reconcile all top-level status docs to the same current-state baseline
- clearly distinguish:
  - live current-state docs
  - historical archived audit docs
- avoid “resolved” wording in summary indexes unless the executable evidence is still current

## Related Existing CRs

- [FOLLOW_UP_CR-001](./FOLLOW_UP_CR-001-status-docs-not-refreshed-after-remediation.md)
- [CR-001](./CR-001-documentation-reality-drift.md)
- [CR-012](./CR-012-false-completion-ledger-and-unindexed-doc-gap.md)

## Pattern Gap

- `PCR-002` False Completion Ledger remains partially unresolved
- `PCR-005` Canonical Contract Drift Across Docs, Code, And Tests remains partially unresolved
