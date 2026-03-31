# CR-002: Release Gates Are Not Passing And CI/Workspace Enforcement Is Incomplete

Severity: Critical

## Problem

The repository does not currently satisfy its own documented quality and release gates, and the CI workflow is not enforcing the full documented gate set from spec 01 and spec 23.

## Evidence

Observed command results:

- `cargo test --workspace`: passed
- `cargo clippy --workspace --all-targets -- -D warnings`: failed
- `pnpm -r typecheck`: failed
- `cargo run --bin check-deps`: failed

Representative failures:

- [crates/cortex-wm/src/sqlite.rs](/home/sabir/projects/cortexos/crates/cortex-wm/src/sqlite.rs): unused imports, useless `format!`, redundant closures, `new_without_default`
- [crates/cortex-admin/src/recovery/session.rs](/home/sabir/projects/cortexos/crates/cortex-admin/src/recovery/session.rs): unused import
- [apps/notes-app/src/__tests__/App.test.tsx](/home/sabir/projects/cortexos/apps/notes-app/src/__tests__/App.test.tsx): missing `describe`, `test`, `expect` type globals during `tsc --noEmit`

CI mismatch observed in [ci.yml](/home/sabir/projects/cortexos/.github/workflows/ci.yml):

- runs Rust format/clippy/build/test
- runs frontend build and typecheck
- runs dependency check
- does not run:
  - workspace frontend tests
  - manifest validation
  - E2E tests
  - `cargo audit`
  - documented smoke/release gates

Workspace script quality is also weaker than the docs imply:

- many frontend packages use no-op scripts like `"lint": "echo lint"` and `"format:check": "echo format:check"`

Examples:

- [apps/desktop-shell/package.json](/home/sabir/projects/cortexos/apps/desktop-shell/package.json)
- [apps/notes-app/package.json](/home/sabir/projects/cortexos/apps/notes-app/package.json)
- [packages/ai-client/package.json](/home/sabir/projects/cortexos/packages/ai-client/package.json)

## Expected Contract

Per:

- [docs/specs/01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [docs/specs/23_release_readiness_qa_acceptance_framework.md](/home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md)
- [docs/specs/appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)

the project should not be treated as complete or releasable unless the documented gates actually pass and CI enforces them.

## Requested Change

Bring the implementation and CI into alignment with the documented gates:

- make `cargo clippy --workspace --all-targets --all-features -- -D warnings` pass
- make `pnpm -r typecheck` pass
- replace frontend no-op lint/format scripts with real enforcement or explicitly downgrade the docs if that is intentional
- extend CI to run the documented missing checks or revise the specs/planning docs to match the actual gate set
- add an explicit “completion claims require gate results” note to the top-level docs

## Verification

- all required CI commands pass on the current branch
- CI runs the same gate set the docs claim is mandatory
- no frontend package ships fake quality scripts where the docs imply real enforcement

## Affected Files

- [docs/specs/01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [docs/specs/23_release_readiness_qa_acceptance_framework.md](/home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md)
- [docs/specs/appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)
- [ci.yml](/home/sabir/projects/cortexos/.github/workflows/ci.yml)
- [package.json](/home/sabir/projects/cortexos/package.json)
