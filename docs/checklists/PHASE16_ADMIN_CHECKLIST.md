# Phase 16 Admin Checklist

Goal:

- add diagnostics and recovery on top of existing subsystem-owned data

Use with:

- [22_admin_diagnostics_recovery.md](/home/sabir/projects/cortexos/docs/specs/22_admin_diagnostics_recovery.md)
- [14_observability_logging_telemetry.md](/home/sabir/projects/cortexos/docs/specs/14_observability_logging_telemetry.md)

## Scope Of This PR

In scope:

- `cortex-admin`
- metrics collector
- recovery/export/admin query flows

Out of scope:

- remote admin
- speculative self-healing behavior

## PR Success Criteria

- admin uses subsystem-owned data instead of shadow state
- host metrics ownership is respected
- logs and AI audit queries come from observability-owned stores

## Work Order

- [ ] implement host metrics collector ownership model
- [ ] implement diagnostic views against canonical data sources
- [ ] implement recovery/session/export flows against canonical persistence
- [ ] add tests for export redaction, recovery behavior, and collector integration

## Validation Checklist

- [ ] no admin-specific shadow persistence model is created

## Stop Conditions

- stop if admin behavior requires redefining ownership of logs, metrics, or audit records
