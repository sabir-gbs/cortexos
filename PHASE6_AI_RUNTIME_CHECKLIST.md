# Phase 6 AI Runtime Checklist

Goal:

- implement the runtime AI layer before user-facing AI surfaces are built

Use with:

- [06_ai_runtime_provider_registry_preferred_llm_model_routing.md](/home/sabir/projects/cortexos/docs/specs/06_ai_runtime_provider_registry_preferred_llm_model_routing.md)
- [appendix_a_required_ai_settings_fields.md](/home/sabir/projects/cortexos/docs/specs/appendix_a_required_ai_settings_fields.md)

## Scope Of This PR

In scope:

- `cortex-ai`
- provider adapter abstraction
- deterministic routing
- timeout and fallback behavior

Out of scope:

- AI assistant UI
- AI action confirmation UI

## PR Success Criteria

- provider routing is deterministic
- `NoProviderConfigured` is explicit
- provider-specific code stays in adapters
- audit/budget/timeouts are integrated at runtime level

## Work Order

### Step 1: provider contract

- [ ] define adapter trait and provider registry
- [ ] include `Zhipu` in canonical runtime support

### Step 2: routing engine

- [ ] implement per-feature, per-app, preferred-provider, and fallback precedence
- [ ] implement no-provider path
- [ ] add tests for precedence and fallback behavior

### Step 3: runtime controls

- [ ] implement timeout categories
- [ ] define audit hooks
- [ ] define budget hooks
- [ ] add tests for timeout mapping and retryable vs non-retryable failures

## Validation Checklist

- [ ] no hardcoded default provider exists
- [ ] adapter-specific behavior is isolated
- [ ] routing results match appendix A and spec 06

## Stop Conditions

- stop if a provider integration would force core routing logic to become provider-specific
