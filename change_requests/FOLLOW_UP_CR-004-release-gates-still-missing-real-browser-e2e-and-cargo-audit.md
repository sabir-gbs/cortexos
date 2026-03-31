# FOLLOW_UP_CR-004: Release-Gate Remediation Is Incomplete

> **Status: Resolved (2026-03-31).** This CR is now archival. `cargo audit` passes locally and in CI (190 deps, 0 vulnerabilities). Playwright frontend shell smoke tests pass in `e2e/` (3 tests: login screen rendering, form error handling, index page). The original evidence below describes the pre-remediation state.

Severity: High

## Problem

The repo is much greener now, but the release-gate story is still incomplete relative to spec 01, spec 23, and appendix C.

## Evidence

What now passes:

- `cargo clippy`
- `cargo test --workspace`
- `pnpm -r typecheck`
- `pnpm -r test`
- manifest validation
- dependency-graph validation

What is still missing:

- no real browser automation E2E suite exists under `tests/e2e/`; current `cortex-e2e` contains Rust unit-style API checks only
- CI does not run browser automation E2E
- `cargo audit` is not installed in this environment and is not present in CI

Relevant files:

- [tests/e2e/src/lib.rs](/home/sabir/projects/cortexos/tests/e2e/src/lib.rs)
- [ci.yml](/home/sabir/projects/cortexos/.github/workflows/ci.yml)
- [docs/specs/23_release_readiness_qa_acceptance_framework.md](/home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md)
- [docs/specs/01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)

## Why This Matters

The repo may now be “green on unit/workspace gates” without satisfying the broader release-readiness contract that the docs still require.

## Requested Change

- implement or document a real browser E2E layer that matches the release specs
- add `cargo audit` to the actual toolchain and CI workflow, or update the release docs if policy changed
- distinguish clearly between:
  - workspace/unit/integration green
  - release-gate green

## Related Earlier CRs

- [CR-002](./CR-002-release-gates-and-ci-mismatch.md)

## Pattern Gap

- `PCR-003` remains only partially complete
