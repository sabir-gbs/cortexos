# Phase 3 Persistence And Observability Checklist

Goal:

- establish canonical persistence and observability foundations before higher services depend on them

Use with:

- [11_virtual_filesystem_and_storage_abstraction.md](/home/sabir/projects/cortexos/docs/specs/11_virtual_filesystem_and_storage_abstraction.md)
- [14_observability_logging_telemetry.md](/home/sabir/projects/cortexos/docs/specs/14_observability_logging_telemetry.md)
- [22_admin_diagnostics_recovery.md](/home/sabir/projects/cortexos/docs/specs/22_admin_diagnostics_recovery.md)

## Scope Of This PR

In scope:

- `cortex-db`
- `cortex-observability`
- baseline migration application flow
- audit/log storage primitives

Out of scope:

- admin UI
- files service behavior
- app-specific persistence

## PR Success Criteria

- canonical SQLite access layer exists
- migrations can be applied deterministically
- structured logging exists
- searchable log mirror contract is defined
- audit persistence utilities exist for later phases

## Work Order

### Step 1: `cortex-db`

- [ ] define pool/connection ownership
- [ ] define migration runner
- [ ] define transaction helpers
- [ ] add integration tests for migration apply/rollback sequencing

### Step 2: `cortex-observability`

- [ ] define structured log format
- [ ] define subscriber initialization
- [ ] define log mirror retention boundary
- [ ] define audit append contract
- [ ] add tests for log serialization and retention behavior

## Validation Checklist

- [ ] no structured state is pushed into ad hoc file storage
- [ ] no subsystem invents its own logging path
- [ ] migration failures are covered by tests
- [ ] log fields are stable and typed

## Stop Conditions

- stop if any spec still implies a conflicting source of truth for structured state
- stop if log/query ownership becomes split between admin and observability
