# Phase 14 AI UX And Safety UX Checklist

Goal:

- build user-facing AI surfaces on top of the completed runtime and policy layers

Use with:

- [19_ai_system_surfaces_and_ux.md](/home/sabir/projects/cortexos/docs/specs/19_ai_system_surfaces_and_ux.md)
- [20_ai_action_permissions_and_safety_controls.md](/home/sabir/projects/cortexos/docs/specs/20_ai_action_permissions_and_safety_controls.md)

## Scope Of This PR

In scope:

- `@cortexos/ai-client`
- assistant UX
- action confirmation/safety UX

Out of scope:

- new provider routing logic
- policy bypasses

## PR Success Criteria

- assistant shortcut is canonical
- conversation state is server-authoritative
- AI actions flow through policy and audit paths

## Work Order

- [ ] implement AI client interfaces for UX layer
- [ ] implement conversation UX against canonical persistence
- [ ] implement confirmation and denial flows for AI actions
- [ ] implement low-risk auto-apply wiring against the canonical setting key
- [ ] add tests for deny/confirm/auto-apply boundaries

## Validation Checklist

- [ ] no client cache becomes source of truth
- [ ] no risky action bypasses `cortex-policy`

## Stop Conditions

- stop if UX wants to redefine runtime routing or policy semantics
