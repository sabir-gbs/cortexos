# Phase 1 Bootstrap Checklist

This is the first implementation batch for CortexOS.

Goal:

- create the repository/workspace/toolchain skeleton defined by spec 01
- keep the batch small enough for one focused PR
- leave the repo in a compilable, verifiable state for Phase 2

Use this with:

- [IMPLEMENTATION_KICKOFF.md](/home/sabir/projects/cortexos/docs/guides/IMPLEMENTATION_KICKOFF.md)
- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)
- [appendix_d_coding_agent_guardrails.md](/home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md)

## Scope Of This PR

In scope:

- root workspace files
- directory skeleton
- minimal crate manifests and `src/lib.rs` stubs
- minimal frontend app manifests and `tsconfig` stubs
- baseline migration
- basic tools/scripts
- CI skeleton

Out of scope:

- real subsystem logic
- real API routes
- real UI implementation
- full provider integrations
- full test coverage

## Current Repo Reality

Before starting this PR, the repo only has documentation and agent guidance. The following do not exist yet and must be created:

- Cargo workspace
- pnpm workspace
- `crates/`
- `apps/`
- `tests/`
- `tools/`
- `scripts/`
- CI workflow

## PR Success Criteria

This PR is successful when all of the following are true:

- `cargo build --workspace` succeeds
- `cargo test --workspace` succeeds
- `cargo fmt --all -- --check` succeeds
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` succeeds
- `pnpm install` succeeds
- `pnpm build` succeeds
- `pnpm typecheck` succeeds
- `cargo run --bin check-deps` succeeds
- `scripts/setup.sh` completes on a clean clone
- `scripts/dev.sh` starts backend and frontend entrypoints without immediate failure

## File Creation Order

Create files in this order. Do not jump ahead.

### Step 1: Root Files

Create:

- `Cargo.toml`
- `package.json`
- `pnpm-workspace.yaml`
- `rust-toolchain.toml`
- `.node-version`
- `.gitignore`
- `Makefile`
- `docker-compose.yml`
- `tsconfig.base.json`

Checklist:

- [ ] Cargo workspace uses `members = ["crates/*"]`
- [ ] `package.json` uses `pnpm@9.15.0`
- [ ] `pnpm-workspace.yaml` includes `apps/*` and `apps/games/*`
- [ ] Rust pinned to `1.85.0`
- [ ] Node pinned to `22.12.0`
- [ ] `.gitignore` includes required build, env, secret, data, and logs entries

### Step 2: Repository Directories

Create directories from spec 01 and add `.gitkeep` only where needed:

- `.github/workflows/`
- `crates/`
- `apps/`
- `apps/games/`
- `apps/shared/`
- `tests/e2e/`
- `tools/`
- `scripts/`

Create crate directories:

- `crates/cortex-core/`
- `crates/cortex-config/`
- `crates/cortex-db/`
- `crates/cortex-observability/`
- `crates/cortex-auth/`
- `crates/cortex-policy/`
- `crates/cortex-settings/`
- `crates/cortex-ai/`
- `crates/cortex-files/`
- `crates/cortex-search/`
- `crates/cortex-notify/`
- `crates/cortex-runtime/`
- `crates/cortex-sdk/`
- `crates/cortex-admin/`
- `crates/cortex-api/`

Create app directories:

- `apps/desktop-shell/`
- `apps/settings-app/`
- `apps/calculator-app/`
- `apps/text-editor-app/`
- `apps/notes-app/`
- `apps/file-manager-app/`
- `apps/media-viewer-app/`
- `apps/terminal-lite-app/`
- `apps/clock-utils-app/`
- `apps/games/solitaire/`
- `apps/games/minesweeper/`
- `apps/games/snake/`
- `apps/games/tetris/`
- `apps/games/chess/`

Checklist:

- [ ] every required top-level directory exists
- [ ] every required crate directory exists
- [ ] every required first-party app directory exists

### Step 3: Minimal Rust Crates

For every crate, create:

- `Cargo.toml`
- `src/lib.rs`
- `src/error.rs`
- `src/types.rs`
- `src/service.rs` for service crates

Minimum crate contract:

- crate-level rustdoc
- exported error type
- exported placeholder types
- exported service trait where applicable
- no `unwrap()`
- compiles cleanly

Checklist:

- [ ] `cortex-core` created first
- [ ] `cortex-config` and `cortex-sdk` created second
- [ ] `cortex-db` and `cortex-observability` created third
- [ ] remaining service crates created after their allowed dependencies exist
- [ ] `cortex-api` created last

Stub policy:

- placeholder types are allowed
- trait methods may return typed placeholder errors
- avoid `todo!()` if it breaks clippy or tests

### Step 4: Baseline Database Files

Create:

- `crates/cortex-db/migrations/0001_create_schema_version.up.sql`
- `crates/cortex-db/migrations/0001_create_schema_version.down.sql`

Checklist:

- [ ] baseline migration creates schema version tracking table only
- [ ] migration runner crate structure exists in `cortex-db`

### Step 5: Minimal Frontend Workspace

For each app, create:

- `package.json`
- `tsconfig.json`
- `src/index.ts` or `src/main.ts`

For shared frontend infra, create:

- `apps/shared/package.json`
- `apps/shared/tsconfig.json`
- `apps/shared/src/index.ts`

Checklist:

- [ ] app package names match directory names
- [ ] apps build independently
- [ ] no app imports another app directly
- [ ] shared code is isolated to `apps/shared`

Stub policy:

- empty render/bootstrap files are acceptable
- no UI logic required in this PR

### Step 6: Tooling Files

Create:

- `tools/validate-manifests.sh`
- `tools/check-deps.rs`

Minimum behavior:

- `validate-manifests.sh` scans app manifests and exits non-zero on missing/invalid files
- `check-deps.rs` walks crate manifests and validates dependency graph rules at a basic level

Checklist:

- [ ] both tools are executable or runnable as appropriate
- [ ] both are wired into CI or documented in scripts/Makefile

### Step 7: Scripts

Create:

- `scripts/setup.sh`
- `scripts/dev.sh`
- `scripts/migrate.sh`

Minimum behavior:

- `setup.sh` verifies toolchain presence and runs bootstrap commands
- `dev.sh` starts backend and frontend entrypoints
- `migrate.sh` invokes the migration path

Checklist:

- [ ] scripts use `bash`
- [ ] scripts fail fast on errors
- [ ] scripts print useful status lines

### Step 8: CI

Create:

- `.github/workflows/ci.yml`

Minimum stages:

- Rust format
- Rust lint
- Rust build
- Rust test
- frontend install
- frontend build
- frontend typecheck
- dependency graph check

Checklist:

- [ ] CI references commands that actually exist in the repo
- [ ] no fake jobs pointing at missing scripts

## Recommended Sub-Batches Inside The PR

If the work is too large for one uninterrupted pass, still keep it as one logical PR but do it in this sequence:

1. root files and directories
2. `cortex-core`
3. `cortex-config` and `cortex-sdk`
4. `cortex-db` and migration
5. remaining crate stubs
6. frontend app stubs
7. tools and scripts
8. CI and final validation

## Validation Checklist Before Marking The PR Done

### Root

- [ ] root workspace files exist
- [ ] lockfiles are present if generated and not ignored
- [ ] `.gitignore` does not ignore lockfiles

### Rust

- [ ] every crate has valid manifest and source files
- [ ] no circular dependencies
- [ ] workspace builds
- [ ] workspace tests pass
- [ ] fmt passes
- [ ] clippy passes

### Frontend

- [ ] every app has package manifest and tsconfig
- [ ] pnpm workspace installs
- [ ] builds pass
- [ ] typecheck passes

### Tooling

- [ ] `check-deps` runs
- [ ] `validate-manifests.sh` runs
- [ ] scripts are executable
- [ ] Makefile targets map to real commands

### CI

- [ ] CI file exists
- [ ] CI stages correspond to available commands
- [ ] no placeholder stage names without commands

## Stop Conditions

Stop and fix before merging if any of these happen:

- workspace file layout diverges from spec 01
- crate names diverge from canonical names
- frontend app names diverge from canonical names
- dependency graph requires a forbidden edge
- scripts/CI refer to commands that do not exist
- stub code introduces warnings or failing gates

## Handoff To Phase 2

Do not start real subsystem logic until this PR is green.

Phase 2 starts with:

1. `cortex-core`
2. `cortex-config`
3. `cortex-db`

That next PR should implement real shared types, config loading, and migration runner behavior rather than more empty scaffolding.
