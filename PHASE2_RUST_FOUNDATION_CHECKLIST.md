# Phase 2 Rust Foundation Checklist

This is the Phase 2 implementation planning document for CortexOS.

Goal:

- define the backend foundation layer that every later subsystem depends on
- keep the work bounded to shared types, shared errors, and config loading
- avoid leaking service-specific logic into the foundation phase

Use with:

- [IMPLEMENTATION_PHASES.md](/home/sabir/projects/cortexos/IMPLEMENTATION_PHASES.md)
- [IMPLEMENTATION_PHASE_CHECKLISTS.md](/home/sabir/projects/cortexos/IMPLEMENTATION_PHASE_CHECKLISTS.md)
- [00_master_spec.md](/home/sabir/projects/cortexos/docs/specs/00_master_spec.md)
- [01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md)
- [02_core_architecture_and_runtime_boundaries.md](/home/sabir/projects/cortexos/docs/specs/02_core_architecture_and_runtime_boundaries.md)

## Scope Of This PR

In scope:

- `cortex-core`
- `cortex-config`
- foundational `cortex-sdk` shared types only

Out of scope:

- DB behavior
- auth/policy behavior
- AI runtime behavior
- app runtime behavior
- API server wiring

## PR Success Criteria

- shared core types exist and compile cleanly
- config loading rules are implemented and tested
- crate ownership boundaries match the specs
- later service crates can import foundation types without redefining them

## Work Order

### Step 1: `cortex-core`

- [ ] define shared IDs, base enums, timestamps, and common result/error primitives
- [ ] define cross-cutting traits that belong in the foundation layer
- [ ] keep subsystem-specific models out of `cortex-core`
- [ ] add rustdoc for all public items
- [ ] add unit tests for core invariants and shared parsing/validation logic

### Step 2: `cortex-config`

- [ ] define config sources and precedence rules
- [ ] define typed config structures for shared runtime configuration only
- [ ] document how defaults, files, and environment overrides interact
- [ ] add tests for config precedence and invalid config cases

### Step 3: foundational `cortex-sdk` types

- [ ] keep this phase limited to shared type definitions needed by later phases
- [ ] do not implement the third-party app platform in this phase
- [ ] ensure no forbidden dependency edges are introduced

## Validation Checklist

- [ ] no later-phase concepts are pulled into foundation crates
- [ ] public types are documented
- [ ] unit tests cover parsing, invariants, and config precedence
- [ ] `check-deps` still passes

## Stop Conditions

- stop if a shared type actually belongs to a later subsystem spec
- stop if config design requires storage, auth, or policy behavior from later phases
