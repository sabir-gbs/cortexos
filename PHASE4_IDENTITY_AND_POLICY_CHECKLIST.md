# Phase 4 Identity And Policy Checklist

Goal:

- establish authentication, sessions, and permission enforcement before resource services expand

Use with:

- [03_identity_authentication_sessions_user_profiles.md](/home/sabir/projects/cortexos/docs/specs/03_identity_authentication_sessions_user_profiles.md)
- [04_permissions_policy_trust_model.md](/home/sabir/projects/cortexos/docs/specs/04_permissions_policy_trust_model.md)
- [20_ai_action_permissions_and_safety_controls.md](/home/sabir/projects/cortexos/docs/specs/20_ai_action_permissions_and_safety_controls.md)

## Scope Of This PR

In scope:

- `cortex-auth`
- `cortex-policy`
- session model
- permission-check interfaces

Out of scope:

- full profile UX
- AI UI
- third-party SDK platform

## PR Success Criteria

- session model is server-authoritative
- permission checks are server-side
- first-party apps have no hidden bypass
- denial and audit paths are defined and tested

## Work Order

### Step 1: `cortex-auth`

- [ ] define session issuance and validation
- [ ] define expiration/revocation model
- [ ] define auth error taxonomy
- [ ] add tests for expired, invalid, and revoked sessions

### Step 2: `cortex-policy`

- [ ] define permission-check interfaces used by later services
- [ ] define grant, revoke, deny, and audit hook paths
- [ ] ensure policy can cover files, settings, AI actions, runtime, and SDK paths
- [ ] add tests for allowed/denied transitions

## Validation Checklist

- [ ] no client-side auth is treated as authoritative
- [ ] no subsystem can bypass policy by design
- [ ] audit hooks exist for sensitive actions

## Stop Conditions

- stop if a later spec requires a special privilege path not modeled in policy
