# Phase 9 App Runtime Checklist

Goal:

- make the platform capable of hosting apps with a consistent lifecycle contract

Use with:

- [09_app_runtime_and_app_lifecycle.md](/home/sabir/projects/cortexos/docs/specs/09_app_runtime_and_app_lifecycle.md)
- [08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md)

## Scope Of This PR

In scope:

- app lifecycle state
- manifest loading at runtime
- launch/stop/crash semantics

Out of scope:

- third-party app SDK flows
- admin recovery UI

## PR Success Criteria

- app lifecycle is explicit and observable
- runtime emits canonical lifecycle events
- manifest rules are enforced consistently

## Work Order

- [ ] implement lifecycle state transitions
- [ ] implement manifest loading/validation at runtime boundary
- [ ] implement crash and stop events
- [ ] add tests for lifecycle transitions and invalid manifest cases

## Validation Checklist

- [ ] runtime does not grant hidden privileges
- [ ] manifest handling is centralized

## Stop Conditions

- stop if lifecycle semantics diverge across first-party vs future third-party apps
