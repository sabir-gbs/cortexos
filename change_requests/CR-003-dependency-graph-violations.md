# CR-003: Dependency Graph Violations Break The Spec 01 Architecture Contract

Severity: Critical

## Problem

The workspace’s dependency-graph enforcement tool reports forbidden internal crate edges. This means the actual architecture does not currently satisfy the foundational layering contract from spec 01.

## Evidence

`cargo run --bin check-deps` reports 11 violations:

- `cortex-api` depends on `cortex-wm`
- `cortex-observability` depends on `cortex-db`
- `cortex-wm` depends on `cortex-core`
- `cortex-wm` depends on `cortex-db`
- `cortex-server` depends on `cortex-api`
- `cortex-server` depends on `cortex-auth`
- `cortex-server` depends on `cortex-config`
- `cortex-server` depends on `cortex-core`
- `cortex-server` depends on `cortex-db`
- `cortex-server` depends on `cortex-notify`
- `cortex-server` depends on `cortex-wm`

## Expected Contract

[docs/specs/01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md) defines an explicit acyclic dependency model and says violations detected by CI block merge.

## Requested Change

Resolve the mismatch in one of two ways:

1. refactor the crate boundaries so the actual edges match the documented graph, or
2. if the graph changed intentionally, update spec 01 and the dependency checker together in the same change

This must not be handled by silently loosening only the checker or only the docs.

## Verification

- `cargo run --bin check-deps` passes with zero violations
- the updated dependency graph is reflected consistently in:
  - spec 01
  - the checker
  - crate manifests

## Affected Files

- [docs/specs/01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [tools/check-deps/src/main.rs](/home/sabir/projects/cortexos/tools/check-deps/src/main.rs)
- [Cargo.toml](/home/sabir/projects/cortexos/Cargo.toml)
