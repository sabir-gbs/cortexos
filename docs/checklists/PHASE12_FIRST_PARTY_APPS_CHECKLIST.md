# Phase 12 First-Party Apps Checklist

Goal:

- validate the platform through first-party app implementation before exposing the SDK publicly

Use with:

- [17_first_party_core_apps_parent.md](/home/sabir/projects/cortexos/docs/specs/17_first_party_core_apps_parent.md)
- [appendix_b_minimum_first_party_app_list.md](/home/sabir/projects/cortexos/docs/specs/appendix_b_minimum_first_party_app_list.md)
- app specs `17a` through `17g`

## Scope Of This PR

In scope:

- first-party app manifests
- app-specific service usage
- app-specific tests

Out of scope:

- third-party app platform

## PR Success Criteria

- each app follows the manifest and permission model
- each app uses platform services rather than direct host/network paths
- system shortcut conflicts are avoided

## Work Order

- [ ] implement apps in documented order unless a spec-backed reason changes it
- [ ] validate manifest and declared permissions for each app
- [ ] ensure service integrations are explicit
- [ ] add app-level tests for core flows and failures

## Validation Checklist

- [ ] no hidden first-party privileges
- [ ] no direct host filesystem or external network access from app code

## Stop Conditions

- stop if an app requires a capability not modeled by existing service/policy contracts
