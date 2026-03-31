# CR-001: Documentation Reality Drift And Stale Repository-State Guidance

Severity: High

## Problem

Several top-level guidance documents still describe CortexOS as a mostly spec-only repository without a real Rust workspace, pnpm workspace, apps, crates, CI, or tooling. That is no longer true. This stale guidance will actively mislead the next coding pass and can cause agents to ignore or overwrite real implementation.

## Evidence

Current docs still state or imply the old repo state:

- [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md)
- [CLAUDE.md](/home/sabir/projects/cortexos/CLAUDE.md)
- [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md)
- [IMPLEMENTATION_KICKOFF.md](/home/sabir/projects/cortexos/IMPLEMENTATION_KICKOFF.md)

Examples observed:

- [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md): says the repository is currently spec-first and that the only substantive assets are `CLAUDE.md` and `docs/specs/*.md`
- [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md): says the Rust workspace, pnpm workspace, backend crates, frontend apps, and CI do not yet materially exist
- [CLAUDE.md](/home/sabir/projects/cortexos/CLAUDE.md): says the codebase is not yet established

Actual repository state now includes:

- root `Cargo.toml`
- root `package.json`
- `pnpm-workspace.yaml`
- `.github/workflows/ci.yml`
- multiple backend crates under `crates/`
- multiple apps under `apps/`
- packages under `packages/`
- tests under `tests/`
- tooling under `tools/`

## Expected Contract

Repository guidance must reflect actual repository reality. Agent instructions are part of the effective control plane of the repo and cannot remain stale once real code exists.

## Requested Change

Update the top-level guidance docs to:

- describe the repository as implementation-bearing, not spec-only
- distinguish clearly between:
  - what is implemented
  - what is partial or placeholder
  - what remains planned
- remove instructions that assume the workspaces or crates do not exist
- add an “implementation status caveat” that points agents to run the real gates before trusting completion claims

## Verification

- Top-level docs no longer claim the repo lacks workspaces/apps/crates/CI when those artifacts exist
- An agent reading only `DOCUMENTATION_INDEX.md`, `MASTER_IMPLEMENTATION_BRIEF.md`, `AGENTS.md`, and `CLAUDE.md` would correctly understand the current implementation-bearing state

## Affected Files

- [AGENTS.md](/home/sabir/projects/cortexos/AGENTS.md)
- [CLAUDE.md](/home/sabir/projects/cortexos/CLAUDE.md)
- [MASTER_IMPLEMENTATION_BRIEF.md](/home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md)
- [IMPLEMENTATION_KICKOFF.md](/home/sabir/projects/cortexos/IMPLEMENTATION_KICKOFF.md)
- [DOCUMENTATION_INDEX.md](/home/sabir/projects/cortexos/DOCUMENTATION_INDEX.md)
