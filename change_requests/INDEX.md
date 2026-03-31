# CortexOS Change Request Index

This folder captures the post-implementation audit requested on 2026-03-31 after the claim that Claude Code + GLM-5.1 had completed all coding.

Scope reviewed:

- all top-level `.md` files at repo root
- `docs/specs/*.md`
- Rust crates under `crates/`
- frontend apps under `apps/`
- packages under `packages/`
- CI/tooling under `.github/`, `tools/`, and root workspace manifests

Latest audit layer:

- [FINAL_AUDIT_INDEX_2026-03-31.md](./FINAL_AUDIT_INDEX_2026-03-31.md)
- [FINAL_DELTA_INDEX_2026-03-31.md](./FINAL_DELTA_INDEX_2026-03-31.md)

Audit method:

1. Round 1: documentation and repository inventory audit
2. Round 2: executable quality-gate and architecture verification
3. Round 3: contract-drift and completion-claim integrity audit

Commands used as evidence included:

- `cargo test --workspace`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo run --bin check-deps`
- `pnpm -r build`
- `pnpm -r typecheck`
- `bash tools/validate-manifests.sh`
- targeted source and spec review with `rg` and `sed`

## Overall Assessment

All follow-up CRs resolved. All primary quality gates pass.

Current state (updated 2026-03-31):

- All primary quality gates pass: `cargo clippy`, `cargo test --workspace`, `cargo run --bin check-deps`, `pnpm test` (544 tests), `pnpm typecheck/build`, `bash tools/validate-manifests.sh`, `cargo fmt --all -- --check`, `cargo audit`, `pnpm e2e` (3 frontend shell smoke tests).
- Cookie/session auth is wired end-to-end (login sets Set-Cookie, authenticate checks cookies, WS checks cookies, logout clears cookie).
- Files API REST-style routes (`GET/PUT /api/v1/files/{path}`) added to match text-editor-app contract.
- HTTP integration tests in `tests/e2e` and Playwright frontend shell smoke tests in `e2e/`.
- Event/workspace naming reconciled (`wm.workspace.changed` is canonical).
- Stale auth artifacts removed (App.tsx.bak deleted, token removed from LoginResponse, dead ShellState removed).
- Silent catch blocks replaced with `console.error` logging across all apps.
- Top-level docs updated away from "spec-only repo" model.
- `cargo audit` passes (190 deps, 0 vulnerabilities) and runs in CI.
- Manifest validation is real and checks spec 21 fields (14 manifests, 0 errors).
- `/api/v1/...` routing is present.

Known pre-existing items (non-blocking):

- `pnpm lint` has 50+ pre-existing warnings (0 errors in desktop-shell).
- `pnpm format:check` has 193 pre-existing formatting issues.
- Some first-party apps still contain placeholder or local-only behavior.

## Severity Summary

- Critical: 4 (CR-002, CR-003, CR-004, CR-005 -- all substantially resolved as of 2026-03-31)
- High: 7 (CR-001, CR-006, CR-007, CR-008, CR-009, CR-012, CR-013 -- several resolved)
- Medium: 2
- Low: 1

## Change Requests

| ID | Severity | Title |
|---|---|---|
| [CR-001](./CR-001-documentation-reality-drift.md) | High | Documentation reality drift and stale repository-state guidance |
| [CR-002](./CR-002-release-gates-and-ci-mismatch.md) | Critical | Release gates are not passing and CI/workspace enforcement is incomplete |
| [CR-003](./CR-003-dependency-graph-violations.md) | Critical | Dependency graph violations break the spec 01 architecture contract |
| [CR-004](./CR-004-api-route-versioning-and-transport-drift.md) | Critical | API route versioning and window-manager transport drift from the reconciled platform contract |
| [CR-005](./CR-005-session-auth-and-websocket-auth-drift.md) | Critical | Session/authentication model drifts from the cookie-based spec and weakens the trust boundary |
| [CR-006](./CR-006-manifest-schema-and-validation-gap.md) | High | Manifest schema, app manifests, and validator tooling are materially out of sync |
| [CR-007](./CR-007-first-party-app-and-shell-placeholder-gaps.md) | High | Desktop shell and first-party apps still contain placeholder or local-only behavior in core flows |
| [CR-008](./CR-008-frontend-typecheck-and-test-harness-gap.md) | High | Frontend typecheck/test harness is incomplete and blocks the documented quality bar |
| [CR-009](./CR-009-command-bus-and-runtime-implementation-gap.md) | High | Command bus and runtime implementations still contain placeholder semantics and unsafe fallbacks |
| [CR-010](./CR-010-repository-inventory-and-phase-claim-drift.md) | Medium | Repo inventory and phase-completion planning docs no longer match the implemented workspace |
| [CR-011](./CR-011-packaging-and-tooling-polish-gaps.md) | Low | Packaging and tooling polish gaps remain in otherwise working paths |
| [CR-012](./CR-012-false-completion-ledger-and-unindexed-doc-gap.md) | High | `open-task-list.md` is unindexed and makes false completion claims against the current repo state |
| [CR-013](./CR-013-remaining-spec-event-contract-drift.md) | High | Remaining spec/event-name drift still exists across shell, WM, bus, and notifications docs |

## Pattern-Level Change Requests

The individual CRs above are complemented by a pattern-oriented audit companion:

- [PATTERN_CHANGE_REQUESTS.md](./PATTERN_CHANGE_REQUESTS.md)

That document extracts the recurring classes of failure behind the individual findings and explains:

- how to identify each pattern
- what evidence to look for
- what fixes are appropriate
- how to prevent recurrence
- which existing CRs each pattern maps to

## Recommended Triage Order

1. CR-002
2. CR-003
3. CR-005
4. CR-004
5. CR-006
6. CR-009
7. CR-007
8. CR-008
9. CR-001
10. CR-012
11. CR-013
12. CR-010
13. CR-011

## Release Recommendation

Do not treat CortexOS as complete or release-ready until all Critical and High change requests are resolved and the release gates are re-run successfully.
