# AGENTS.md

This file defines how coding agents should operate in the CortexOS repository.

## Current Repository Reality

- This repository contains both specs and implementation.
- **Rust workspace**: 17 crates in `crates/` (cortex-admin, cortex-ai, cortex-api, cortex-auth, cortex-config, cortex-core, cortex-db, cortex-files, cortex-notify, cortex-observability, cortex-policy, cortex-runtime, cortex-sdk, cortex-search, cortex-server, cortex-settings, cortex-wm).
- **Frontend workspace**: desktop shell + apps in `apps/`, 3 shared packages in `packages/` (ai-client, game-framework, sdk).
- **CI**: `.github/workflows/ci.yml` is in place.
- Specs under `docs/specs/` remain the authoritative design contract. Implementation must follow specs, not the other way around.
- When modifying existing code, consult the owning spec first. When adding new code, ensure it conforms to the relevant spec sections.

### What is implemented

- All 17 Rust crates are scaffolded with tests and core logic.
- All frontend apps and packages are scaffolded with tests.
- Cookie/session auth wired end-to-end (login, authenticate, WS, logout).
- Files API REST-style routes (`GET/PUT /api/v1/files/{path}`) matching text-editor-app contract.
- HTTP integration tests in `tests/e2e` covering the auth boundary.
- Manifest validation is real and checks spec 21 fields.
- `/api/v1/...` routing is present.
- Event/workspace naming reconciled (`wm.workspace.changed` is canonical).

### Quality-gate status (as of 2026-03-31)

**Passing:**
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace` (all tests green)
- `cargo run --bin check-deps` (17 crates, 0 violations)
- `cargo audit` (190 crate dependencies, 0 vulnerabilities)
- `cargo fmt --all -- --check`
- `pnpm -r typecheck`
- `pnpm test` (544 tests across all packages)
- `pnpm -r build`
- `bash tools/validate-manifests.sh` (14 manifests, 0 errors)
- `pnpm e2e` (3 Playwright frontend shell smoke tests — login screen rendering, form error handling, index page)

**Known pre-existing items (non-blocking):**
- `pnpm lint` has 50+ pre-existing warnings (0 errors in desktop-shell)
- `pnpm format:check` has 193 pre-existing formatting issues
- Some first-party apps still contain placeholder or local-only behavior

See `change_requests/INDEX.md` for the full audit history.

## Primary Sources of Truth

Read these before making meaningful changes:

1. `DOCUMENTATION_INDEX.md`
2. `MASTER_IMPLEMENTATION_BRIEF.md`
3. `docs/specs/00_master_spec.md`
4. `docs/specs/01_repository_toolchain_engineering_conventions.md`
5. `docs/specs/appendix_c_definition_of_done.md`
6. `docs/specs/appendix_d_coding_agent_guardrails.md`
7. `CLAUDE.md`

If a task is subsystem-specific, also read the owning spec for that subsystem before implementing.

## Project Intent

CortexOS is specified as an AI-native browser operating system with:

- Rust on the server/backend side
- TypeScript/React on the browser/client side
- a Cargo workspace for backend crates
- a pnpm workspace for frontend apps/packages
- a browser-rendered desktop shell
- a strict server-side security model
- a typed command bus as the main client/server interface

The specs remain the canonical design contract; implementation must match them.

## Operating Rules

### 1. Spec-first implementation

- Do not start implementation from assumptions.
- Read the relevant spec sections completely before changing code or adding structure.
- If specs conflict, the more specific child spec wins within its scope.
- If the spec is missing, ambiguous, or underdefines a security-sensitive change, stop and ask.
- For external libraries, frameworks, SDKs, and platform APIs, use Context7 and current official docs as the default reference before coding. Do not rely on model memory for current API signatures or recommended patterns.

### 2. Do not invent infrastructure that does not exist

- The Rust workspace, pnpm workspace, and CI are established. Do not claim additional infrastructure (e.g., E2E test harness, deployment pipelines) exists unless you verify it in the repo.
- When adding new scaffolding, follow spec 01 exactly unless the user requests a deviation.

### 3. Preserve architecture boundaries

- Client is never the source of truth for auth, authorization, or permissions.
- Policy checks must not be bypassed.
- First-party apps must not gain hidden privileges.
- AI provider logic must remain adapter-isolated, not embedded in core routing logic.
- No untyped command-bus events.
- Do not introduce deprecated or legacy API usage when Context7 or current official docs indicate a newer supported pattern.

### 4. Treat specs as implementation contracts

- `docs/specs/01_repository_toolchain_engineering_conventions.md` is authoritative for repo layout, naming, toolchain, testing, CI, scripts, and commit conventions.
- `docs/specs/appendix_c_definition_of_done.md` defines done.
- `docs/specs/appendix_d_coding_agent_guardrails.md` defines hard rules and stop conditions.

## Expected Target Structure

When creating the real repo scaffolding, align to the planned monorepo shape from spec 01:

- root Cargo workspace
- root pnpm workspace
- `crates/` for Rust backend crates
- `apps/` for TypeScript apps
- `tests/` for integration and E2E coverage
- `tools/` for repo validation utilities
- `scripts/` for setup/dev/migration helpers

Build in the order defined by spec 01, section 21.

## Naming Conventions

Use the spec-defined naming rules:

- Rust crates: `cortex-<domain>`
- Rust files/functions/modules: `snake_case`
- Rust types/enums: `PascalCase`
- directories: `kebab-case`
- TypeScript functions: `camelCase`
- app/package names: `kebab-case`
- WebSocket events: `domain.action`
- API routes: `/api/v1/<domain>/<action>`
- SQL migrations: `NNNN_description_in_snake_case.up.sql` and `.down.sql`
- commits: Conventional Commits, `type(scope): description`

## Implementation Priorities

When bootstrapping the codebase, prefer this order:

1. Root workspace/toolchain files
2. Foundation crates
3. Service crates in dependency order
4. API crate
5. Frontend app scaffolds
6. Tooling, CI, and scripts

Follow spec 01 section 21.3 for file creation priority.

## Quality Bar

Every meaningful implementation should aim to satisfy the relevant spec plus the project DoD:

- formatting passes
- linting passes
- tests accompany the change
- public APIs are documented
- error paths are handled
- security rules are enforced server-side
- docs/specs are updated if behavior changes

Do not mark work complete because the UI looks finished.

## Testing Expectations

As the real workspace is created, agents should add:

- unit tests for public behavior
- integration tests for cross-crate behavior
- E2E coverage for user flows where applicable
- regression tests for bug fixes

Tests should verify behavior, not internal implementation details.

## Stop Conditions

Pause and ask the user before proceeding when the task involves:

- ambiguous or missing spec guidance
- security uncertainty
- cross-subsystem contract changes
- new external dependencies
- breaking public-interface changes
- database migration design
- permission model changes

## Practical Guidance For This Repo Today

- Most near-term work will likely be one of:
  - refining specs to match evolving requirements
  - extending or fixing existing crate/app implementations against their specs
  - adding missing test coverage or tightening quality gates
  - tightening agent guidance and contributor workflow docs
- For documentation-only work, keep docs aligned with the spec hierarchy and avoid introducing contradictions.
- For code changes, prefer modifying existing implementations over creating new files, but do not violate the required structure in spec 01.

## When Unsure

- Check the relevant subsystem spec.
- Check spec 01 for repo/tooling conventions.
- Check appendix C for done.
- Check appendix D for hard rules and stop conditions.
- Prefer explicit notes about assumptions over silent guesses.
