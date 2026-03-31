# Master Implementation Brief

> **Current status (2026-03-31):** Implementation is in progress. The Rust workspace (17 crates) and frontend workspace (desktop shell + apps, 3 shared packages) are scaffolded. All primary quality gates pass: `cargo clippy`, `cargo test --workspace`, `cargo run --bin check-deps`, `cargo audit`, `pnpm test/typecheck/build`, `pnpm e2e`, and manifest validation. Cookie/session auth is wired end-to-end, Files API routes match the frontend contract, HTTP integration tests cover the auth boundary, and Playwright frontend shell smoke tests pass (3 tests). See `change_requests/INDEX.md` for the current audit history.

This is the master implementation reference for starting CortexOS in Claude Code with GLM-5.1 as the coding LLM.

It is the top-level document to reference when asking the agent to begin implementation of the entire project.

For the complete link map to every spec, planning doc, and checklist, use [DOCUMENTATION_INDEX.md](/home/sabir/projects/cortexos/DOCUMENTATION_INDEX.md).

## Purpose

This brief exists to do three things:

1. tell the agent what CortexOS is and what stage the repository is in
2. define the authoritative document stack and decision hierarchy
3. define the required implementation sequence, stop conditions, and completion rules

## Current Repository State

CortexOS is an implementation-bearing repository.

What exists:

- documentation under `docs/specs/`
- agent guidance files
- implementation planning documents
- **17 Rust crates** in `crates/` (cortex-admin, cortex-ai, cortex-api, cortex-auth, cortex-config, cortex-core, cortex-db, cortex-files, cortex-notify, cortex-observability, cortex-policy, cortex-runtime, cortex-sdk, cortex-search, cortex-server, cortex-settings, cortex-wm)
- **Desktop shell + frontend apps** in `apps/`, **3 shared packages** in `packages/`
- **CI workflow**: `.github/workflows/ci.yml`
- Frontend builds and tests pass (544 tests)
- **All primary quality gates pass** (clippy, test, check-deps, audit, typecheck, test, build, manifest validation)
- Cookie/session auth wired end-to-end; Files API routes match frontend contract
- HTTP integration tests cover the auth boundary
- Playwright frontend shell smoke tests pass (3 tests: login screen rendering, form error handling, index page)

What now passes (as of 2026-03-31):

- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace` (all tests green)
- `cargo run --bin check-deps` (17 crates, 0 violations)
- `cargo audit` (190 crate dependencies, 0 vulnerabilities)
- `cargo fmt --all -- --check`
- `pnpm typecheck`
- `pnpm test` (544 tests)
- `pnpm -r build`
- `pnpm e2e` (3 Playwright frontend shell smoke tests)
- `bash tools/validate-manifests.sh` (14 manifests, 0 errors)

Known pre-existing items (non-blocking):

- `pnpm lint` has 50+ pre-existing warnings (0 errors in desktop-shell)
- `pnpm format:check` has 193 pre-existing formatting issues
- Some first-party apps still contain placeholder or local-only behavior

For the current change-request audit status, see [change_requests/INDEX.md](/home/sabir/projects/cortexos/change_requests/INDEX.md).

The agent must verify claims against actual repo state rather than relying on documentation alone.

## Product Intent

CortexOS is an AI-native browser operating system with:

- Rust on the backend/service side
- TypeScript/React on the browser/client side
- server-authoritative auth, policy, and persistence
- typed command bus for realtime commands/events
- HTTP/REST for auth, bootstrap, CRUD, and admin/snapshot operations
- SQLite as canonical structured state storage
- filesystem/object storage only for blobs, packages, exports, and crash artifacts

## Authoritative Document Hierarchy

Use this order when interpreting the project:

1. [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md)
2. [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md)
3. [CLAUDE.md](/home/sabir/projects/cortexos/CLAUDE.md)
4. [docs/specs/00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md)
5. [docs/specs/01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
6. subsystem-owning specs under `docs/specs/`
7. [docs/specs/appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)
8. [docs/specs/appendix_d_coding_agent_guardrails.md](/home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md)
9. implementation planning documents listed below

Conflict rule:

- master spec and spec 01 govern global architecture and repo conventions
- a child spec is authoritative within its declared subsystem scope
- if a true conflict remains, stop and request a documentation decision before coding further

## Planning Document Stack

These planning docs convert the specs into execution order:

- [IMPLEMENTATION_KICKOFF.md](/home/sabir/projects/cortexos/IMPLEMENTATION_KICKOFF.md)
- [IMPLEMENTATION_PHASES.md](/home/sabir/projects/cortexos/IMPLEMENTATION_PHASES.md)
- [PHASE1_BOOTSTRAP_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE1_BOOTSTRAP_CHECKLIST.md)
- [IMPLEMENTATION_PHASE_CHECKLISTS.md](/home/sabir/projects/cortexos/IMPLEMENTATION_PHASE_CHECKLISTS.md)
- `PHASE2_...` through `PHASE17_...` checklist files at repo root

## Reconciled Baseline

Before coding, assume the following are already decided in the documentation:

- transport model is hybrid:
  - HTTP/REST for auth, bootstrap, CRUD, admin, and snapshot reads
  - typed command bus over WebSocket for realtime commands, events, and streaming
- structured persistent state is SQLite-authoritative
- logs are written to stdout and mirrored into a bounded searchable observability store
- AI runtime is optional and deterministic when no provider is configured
- `Zhipu` is a first-class runtime provider enum
- assistant shortcut is `Ctrl+Shift+A`
- command palette shortcut is `Ctrl+Space`
- canonical event names are already reconciled
- window manager client mutations go through the command bus, not ad hoc client REST

Reference:

- [docs/specs/spec_audit_2026-03-30.md](/home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md)

## Required Phase Order

The implementation order is:

1. Phase 0: freeze contracts
2. Phase 1: repository bootstrap
3. Phase 2: Rust foundation
4. Phase 3: persistence and observability
5. Phase 4: identity and policy
6. Phase 5: settings and filesystem
7. Phase 6: AI runtime
8. Phase 7: command bus
9. Phase 8: window manager and shell
10. Phase 9: app runtime
11. Phase 10: core system services
12. Phase 11: theme and accessibility
13. Phase 12: first-party apps
14. Phase 13: games
15. Phase 14: AI UX and safety UX
16. Phase 15: SDK
17. Phase 16: admin
18. Phase 17: release gates

Do not skip ahead if the lower-layer phase is not truly complete.

## Execution Rules

The agent must:

- work phase by phase
- use the matching checklist file for the current phase
- keep changes bounded to the current phase unless a required dependency forces a small adjacent edit
- use Context7 as the default source for current library/framework/API documentation before writing or revising code that depends on external libraries, SDKs, or platform APIs
- prefer current official docs and Context7-backed examples over model memory for library usage details, method names, signatures, configuration, and migration patterns
- implement tests with each phase where the specs require them
- update docs if behavior changes or if a new setting/event/public contract is introduced
- avoid placeholder logic in core paths once a phase is declared complete

The agent must not:

- bypass `cortex-policy`
- invent direct app filesystem or external network access
- use untyped command/event payloads
- make the browser client authoritative for auth, permissions, or persistent state
- silently change spec-defined shortcuts, events, or ownership boundaries
- rely on stale remembered API usage when Context7 or current official documentation is available
- introduce deprecated, legacy, or superseded library mechanisms without a documented reason

## Stop Conditions

Stop and ask for a documentation decision if:

- a spec is still ambiguous for a security-sensitive behavior
- a required cross-subsystem contract is missing
- a new public interface is needed that no current spec owns
- a dependency edge would violate spec 01
- a phase cannot be completed without changing a lower-layer contract

## Definition Of Done

No phase is complete because code compiles or the UI renders.

A phase is complete only when:

- spec behavior is implemented
- edge cases and failure modes are handled
- tests required by the specs are added and passing
- observability is in place
- accessibility is covered for UI work
- documentation stays aligned with behavior

Reference:

- [docs/specs/appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)

## How To Start

The correct starting point for a fresh implementation run is:

1. read this brief
2. read [PHASE1_BOOTSTRAP_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE1_BOOTSTRAP_CHECKLIST.md)
3. implement only Phase 1 first
4. verify Phase 1 gates
5. then continue to [PHASE2_RUST_FOUNDATION_CHECKLIST.md](/home/sabir/projects/cortexos/PHASE2_RUST_FOUNDATION_CHECKLIST.md), and so on

## Expected Working Style

For each phase, the agent should:

- summarize the current phase goal
- name the docs it is using
- implement the smallest complete batch that satisfies the phase checklist
- run the relevant verification commands
- report what was completed, what remains, and whether the phase is truly done

## Completion Marker

The full CortexOS build is complete only when all phases through Phase 17 are done and the release gates pass.
