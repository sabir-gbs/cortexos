# Phase 15 SDK Checklist

Goal:

- expose the platform to third-party apps only after first-party use has validated the internals

Use with:

- [21_sdk_manifest_third_party_app_platform.md](/home/sabir/projects/cortexos/docs/specs/21_sdk_manifest_third_party_app_platform.md)

## Scope Of This PR

In scope:

- manifest schema
- install/update/uninstall flows
- sandboxed SDK APIs

Out of scope:

- app marketplace UX
- payment/revenue systems

## PR Success Criteria

- third-party app lifecycle is defined and constrained
- sandboxing and permission enforcement are explicit
- SDK AI hooks align with reconciled AI ownership

## Work Order

- [ ] implement manifest validation contract
- [ ] implement install/update/uninstall registry flows
- [ ] implement sandbox and permission checks
- [ ] implement SDK API surfaces through platform services only
- [ ] add tests for manifest validation, install lifecycle, and denied calls

## Validation Checklist

- [ ] no third-party app gets a side path around policy or runtime contracts

## Stop Conditions

- stop if SDK design depends on capabilities not already validated by the platform
