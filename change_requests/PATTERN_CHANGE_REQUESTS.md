# CortexOS Pattern Change Requests

This document captures the repeatable failure patterns found during the CortexOS audit. These are not replacements for the file-specific CRs in [INDEX.md](./INDEX.md). They are the pattern-level guidance for identifying and fixing the same classes of problems elsewhere in the repo.

## PCR-001: Stale Control-Plane Documentation

### Pattern

Top-level guidance documents continue to describe an older repository state after the repo has materially changed.

### How To Identify

- compare top-level docs against the actual filesystem
- look for phrases like:
  - "spec-first"
  - "codebase not yet established"
  - "workspaces/crates/apps do not yet exist"
- verify whether those claims still match the repository tree

### Typical Evidence

- root docs say the repo is mostly docs-only
- `Cargo.toml`, `pnpm-workspace.yaml`, `crates/`, `apps/`, `.github/`, `tools/` already exist

### Required Fix

- update the docs to reflect actual repo reality
- separate:
  - implemented
  - partial
  - planned
- add a current-status note where agents begin work

### Prevention

- any PR that adds a new workspace, crate, app, CI workflow, or major subsystem must update top-level guidance docs in the same PR

### Related CRs

- [CR-001](./CR-001-documentation-reality-drift.md)
- [CR-010](./CR-010-repository-inventory-and-phase-claim-drift.md)

## PCR-002: False Completion Ledger

### Pattern

A checklist, task list, or status doc claims a subsystem is complete or verified, but the current repo no longer satisfies the claimed checks.

### How To Identify

- search for words like:
  - "verified"
  - "complete"
  - "all gates pass"
  - "no stubs/placeholders"
- re-run the claimed commands instead of trusting the document
- ensure any status ledger is indexed and clearly authoritative

### Typical Evidence

- a task list says `cargo clippy` or `pnpm typecheck` passes
- the same commands fail when re-run

### Required Fix

- correct or remove the stale completion claims
- mark the doc as non-authoritative if it is only a working note
- do not allow top-level status docs to float outside the main documentation index

### Prevention

- require status docs to include:
  - date
  - exact command set
  - commit/branch context
  - whether the status is historical or current

### Related CRs

- [CR-012](./CR-012-false-completion-ledger-and-unindexed-doc-gap.md)
- [CR-002](./CR-002-release-gates-and-ci-mismatch.md)

## PCR-003: Non-Executable Quality Gates

### Pattern

The project documents strong quality gates, but the actual scripts, CI jobs, or package-level commands are partial, fake, or missing.

### How To Identify

- compare spec-required gates with:
  - root scripts
  - package scripts
  - CI workflow jobs
- search for scripts like:
  - `"lint": "echo lint"`
  - `"format:check": "echo format:check"`
- verify that all documented commands are actually run in CI

### Typical Evidence

- CI omits manifest validation, E2E, or tests that the docs claim are required
- package-level lint/format scripts are no-ops

### Required Fix

- make the gates real, or lower the docs to the actual gate set
- do not leave “green” scripts that only echo success
- align spec 01, spec 23, root scripts, and CI workflow together

### Prevention

- every documented gate should be traceable to:
  - a real script
  - a CI job
  - a failure mode that blocks merge

### Related CRs

- [CR-002](./CR-002-release-gates-and-ci-mismatch.md)
- [CR-008](./CR-008-frontend-typecheck-and-test-harness-gap.md)
- [CR-011](./CR-011-packaging-and-tooling-polish-gaps.md)

## PCR-004: Tool Validates Proxy Artifacts Instead Of The Real Contract

### Pattern

A validation tool exists and passes, but it validates the wrong thing.

### How To Identify

- read the validator implementation, not just its output
- compare:
  - what the spec says is being validated
  - what the script actually checks
- look for validators that check file presence but not schema/content

### Typical Evidence

- manifest validator only checks `package.json`, `tsconfig.json`, and entry files
- it does not validate actual `manifest.json` schema fields

### Required Fix

- change validators to enforce the real contract
- ensure malformed data actually fails validation
- align the validator with the canonical schema/type definitions

### Prevention

- every validator should have negative tests with known-bad fixtures
- CI should include at least one test that proves the validator rejects invalid input

### Related CRs

- [CR-006](./CR-006-manifest-schema-and-validation-gap.md)
- [CR-002](./CR-002-release-gates-and-ci-mismatch.md)

## PCR-005: Canonical Contract Drift Across Docs, Code, And Tests

### Pattern

One public contract exists in multiple places, but names or shapes diverge across specs, code, manifests, or tests.

### How To Identify

- search the same contract term across:
  - subsystem specs
  - parent specs
  - audit docs
  - implementation
  - tests
- focus on:
  - event names
  - command names
  - route paths
  - manifest fields
  - IDs

### Typical Evidence

- one spec says `workspace.changed`, another says `wm.desktop.changed`
- one doc says `/api/v1/...`, code uses `/api/...`
- manifests, SDK types, and validators expect different field shapes

### Required Fix

- nominate one canonical source
- reconcile every consumer to the same names and shapes
- update stale audit docs that claim the drift is already resolved

### Prevention

- add contract-inventory tables for canonical names
- add tests that assert route/event/schema names from a shared source where possible

### Related CRs

- [CR-004](./CR-004-api-route-versioning-and-transport-drift.md)
- [CR-006](./CR-006-manifest-schema-and-validation-gap.md)
- [CR-013](./CR-013-remaining-spec-event-contract-drift.md)

## PCR-006: Security Boundary Erosion Through Client Authority

### Pattern

The implementation moves trust-sensitive state or auth mechanics into the browser client in ways the specs explicitly disallow.

### How To Identify

- search for:
  - `localStorage` token storage
  - bearer tokens in frontend API helpers
  - query-string auth tokens
  - browser-owned permission or session state
- compare those flows with the auth/policy specs

### Typical Evidence

- `localStorage` stores session token
- WebSocket uses `?token=...`
- login response returns token body when the spec says cookie session

### Required Fix

- move authority back to the server-side boundary
- make browser state cache-only, not authoritative
- eliminate token transport patterns that contradict the security model

### Prevention

- require explicit security review when:
  - auth transport changes
  - session persistence changes
  - permission-state ownership changes

### Related CRs

- [CR-005](./CR-005-session-auth-and-websocket-auth-drift.md)
- [CR-004](./CR-004-api-route-versioning-and-transport-drift.md)

## PCR-007: Placeholder Or Mock Logic In Core User Flows

### Pattern

Core app or platform behavior remains backed by placeholder UI, demo data, or browser-local mock storage even though the subsystem is being presented as implemented.

### How To Identify

- search for:
  - `placeholder`
  - `simulate`
  - hardcoded data structures that stand in for services
  - `localStorage` in first-party app persistence paths
- compare those paths against the owning subsystem specs

### Typical Evidence

- shell window says “App content renders here”
- file manager browses a hardcoded object
- text editor saves to `localStorage` instead of the filesystem service

### Required Fix

- replace placeholders in core flows with real service-backed behavior
- if a placeholder is intentional for non-core polish, mark it clearly and keep it out of completion claims

### Prevention

- DoD checks should explicitly fail a phase if core logic remains placeholder-backed

### Related CRs

- [CR-007](./CR-007-first-party-app-and-shell-placeholder-gaps.md)
- [CR-009](./CR-009-command-bus-and-runtime-implementation-gap.md)

## PCR-008: Silent Fallback Masks Corruption Or Contract Failure

### Pattern

Core code silently defaults on invalid data instead of surfacing an error.

### How To Identify

- search for:
  - `unwrap_or_default()`
  - broad catch-and-ignore patterns
  - “default on parse failure” behavior
- inspect persistence and API boundary code first

### Typical Evidence

- invalid UUIDs from SQLite are coerced to default values
- serialization errors are swallowed into empty/default values

### Required Fix

- replace lossy defaults with explicit error propagation or structured failure handling
- add tests for malformed persisted data and malformed payloads

### Prevention

- ban silent defaulting in persistence and auth paths unless explicitly justified and documented

### Related CRs

- [CR-009](./CR-009-command-bus-and-runtime-implementation-gap.md)
- [CR-002](./CR-002-release-gates-and-ci-mismatch.md)

## PCR-009: Layering And Boundary Violations

### Pattern

Crates or modules bypass the documented dependency graph or architectural layering.

### How To Identify

- run the dependency checker
- inspect any “integration convenience” crate that starts depending on too many internals
- compare actual dependencies to spec 01’s allowed edges

### Typical Evidence

- `check-deps` reports forbidden internal crate dependencies

### Required Fix

- refactor the implementation or revise the graph contract explicitly
- do not simply weaken the checker without updating the architecture docs

### Prevention

- keep dependency checks in CI
- require architecture review when adding new internal crate edges

### Related CRs

- [CR-003](./CR-003-dependency-graph-violations.md)

## PCR-010: Planning/Inventory Drift After Real Implementation Arrives

### Pattern

Planning docs remain written as if the repo is pre-bootstrap even after large parts of the system are implemented.

### How To Identify

- compare phase docs/checklists with the current repository inventory
- look for plans that still instruct contributors to create things that already exist
- check whether all top-level docs are indexed and status-scoped

### Typical Evidence

- implementation brief says to begin at Phase 1 bootstrap
- repo already contains many phase outputs
- undocumented top-level docs exist outside the index

### Required Fix

- convert planning docs into living status docs, or clearly mark them as historical planning artifacts
- keep the index complete

### Prevention

- after major implementation milestones, run a documentation reality pass before claiming completion

### Related CRs

- [CR-010](./CR-010-repository-inventory-and-phase-claim-drift.md)
- [CR-012](./CR-012-false-completion-ledger-and-unindexed-doc-gap.md)

## Recommended Usage

Use this document as a review checklist whenever Claude Code or another agent claims a subsystem or the full repo is “done.” The fastest pattern checks are:

1. Re-run the gates instead of trusting status docs.
2. Compare top-level guidance docs against the actual repo tree.
3. Search for placeholder/mock/local-only behavior in core flows.
4. Compare canonical contracts across specs, code, tests, and validators.
5. Re-check auth/session and dependency boundaries before accepting a completion claim.
