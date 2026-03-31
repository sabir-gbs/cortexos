# Phase 5 Settings And Filesystem Checklist

Goal:

- implement canonical settings and file access services that later phases can safely consume

Use with:

- [05_settings_service_and_settings_app.md](/home/sabir/projects/cortexos/docs/specs/05_settings_service_and_settings_app.md)
- [11_virtual_filesystem_and_storage_abstraction.md](/home/sabir/projects/cortexos/docs/specs/11_virtual_filesystem_and_storage_abstraction.md)
- [appendix_a_required_ai_settings_fields.md](/home/sabir/projects/cortexos/docs/specs/appendix_a_required_ai_settings_fields.md)

## Scope Of This PR

In scope:

- `cortex-settings`
- `cortex-files`
- canonical settings validation and persistence
- canonical file metadata/content handling

Out of scope:

- rich settings app UI
- high-level AI UX
- file-manager app behavior

## PR Success Criteria

- settings namespaces and validation rules are implemented
- appendix A AI settings are fully represented
- file metadata and content ownership follow the reconciled storage model
- apps still have no direct host filesystem access

## Work Order

### Step 1: `cortex-settings`

- [ ] define namespace-aware settings API
- [ ] define validation rules and effective timing behavior
- [ ] implement `settings.changed` event contract
- [ ] add tests for invalid writes, scope separation, and AI-setting completeness

### Step 2: `cortex-files`

- [ ] define file metadata persistence
- [ ] define blob/content storage boundary
- [ ] define export/artifact exceptions explicitly
- [ ] add tests for path safety, metadata consistency, and permission enforcement

## Validation Checklist

- [ ] no AI settings keys are missing from implementation contracts
- [ ] no file content is forced into the wrong persistence model
- [ ] service APIs, not direct access, remain the only supported path

## Stop Conditions

- stop if any new settings key is introduced without appendix A coverage
