# FOLLOW_UP_CR-001: Status And Audit Docs Were Not Refreshed After Major Remediation Progress

> **Status: Resolved (2026-03-31).** This CR is now archival. All status docs (AGENTS.md, MASTER_IMPLEMENTATION_BRIEF.md, CLAUDE.md, open-task-list.md, INDEX.md) now agree on the same current-state baseline.

Severity: Medium

## Problem

Several status documents still claim that major gates are failing even though they now pass. The repo moved forward, but the status/audit layer was not refreshed accordingly.

## Evidence

Current gate results from this audit:

- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passes
- `cargo run --bin check-deps`: passes
- `pnpm -r typecheck`: passes
- `cargo test --workspace`: passes

But these docs still say failures remain:

- [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md)
- [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md)
- [open-task-list.md](/home/sabir/projects/cortexos/open-task-list.md)
- [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md)

## Why This Matters

This is the inverse of the original stale-doc problem. Now the docs understate progress and can cause wasted rework or incorrect triage.

## Requested Change

- refresh all status docs to reflect the new passing gate baseline
- preserve historical context, but clearly mark historical findings as superseded
- update the original change-request index with a superseded/resolved status for items that were actually fixed

## Related Earlier CRs

- [CR-001](./CR-001-documentation-reality-drift.md)
- [CR-010](./CR-010-repository-inventory-and-phase-claim-drift.md)
- [CR-012](./CR-012-false-completion-ledger-and-unindexed-doc-gap.md)
