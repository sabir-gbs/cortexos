# Phase 7 Command Bus Checklist

Goal:

- establish the typed realtime transport contract for the platform

Use with:

- [10_system_command_bus_and_event_model.md](/home/sabir/projects/cortexos/docs/specs/10_system_command_bus_and_event_model.md)
- [02_core_architecture_and_runtime_boundaries.md](/home/sabir/projects/cortexos/docs/specs/02_core_architecture_and_runtime_boundaries.md)

## Scope Of This PR

In scope:

- typed commands and events
- durable idempotency
- dead-letter behavior
- command dispatch and subscription contracts

Out of scope:

- every eventual producer/consumer
- app-level UI behavior

## PR Success Criteria

- typed command/event contracts exist on Rust and TypeScript sides
- durable idempotency resolves replay/restart concerns
- no untyped payload escapes remain in public transport contracts

## Work Order

- [ ] implement typed bus interfaces
- [ ] implement durable idempotency records
- [ ] implement dead-letter persistence and retry handling
- [ ] enforce canonical event names
- [ ] define policy interception before dispatch
- [ ] add tests for duplicate command replay, restart-safe behavior, and subscriber failure handling

## Validation Checklist

- [ ] no `any`/untyped payloads at public bus boundaries
- [ ] no memory-only idempotency assumption remains as the sole guarantee

## Stop Conditions

- stop if any subsystem requests a custom event naming scheme outside the canonical registry
