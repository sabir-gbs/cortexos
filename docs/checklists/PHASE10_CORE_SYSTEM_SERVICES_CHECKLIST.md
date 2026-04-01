# Phase 10 Core System Services Checklist

Goal:

- implement shared services consumed by shell and apps after runtime foundations are stable

Use with:

- [12_search_indexing_global_command_palette.md](/home/sabir/projects/cortexos/docs/specs/12_search_indexing_global_command_palette.md)
- [13_notifications_service.md](/home/sabir/projects/cortexos/docs/specs/13_notifications_service.md)
- [14_observability_logging_telemetry.md](/home/sabir/projects/cortexos/docs/specs/14_observability_logging_telemetry.md)

## Scope Of This PR

In scope:

- search
- command palette backend/service path
- notifications

Out of scope:

- app-specific search UX
- advanced ranking and optimization

## PR Success Criteria

- command palette backend/service uses canonical shortcut and storage assumptions
- notifications use canonical event names
- observability integration is reused, not reimplemented

## Work Order

- [ ] implement search index ownership and query path
- [ ] implement palette service behavior
- [ ] implement notification creation/dismiss lifecycle
- [ ] add tests for search indexing/query and notification event flow

## Validation Checklist

- [ ] command palette shortcut remains `Ctrl+Space`
- [ ] notification events remain canonical

## Stop Conditions

- stop if any service attempts to bypass the command bus or observability ownership model
