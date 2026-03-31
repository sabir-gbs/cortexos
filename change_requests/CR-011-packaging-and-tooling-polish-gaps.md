# CR-011: Packaging And Tooling Polish Gaps

Severity: Low

## Problem

A small number of packaging/tooling details are not yet clean even where build paths otherwise work.

## Evidence

`pnpm -r build` completes, but `@cortexos/sdk` emits an exports warning because the `types` condition appears after `import` and `require`:

- [packages/sdk/package.json](/home/sabir/projects/cortexos/packages/sdk/package.json)

## Expected Contract

Published package metadata should be warning-free where practical, especially for core SDK packages.

## Requested Change

- clean up `exports` ordering and any related package metadata warnings
- treat packaging warnings as part of release hardening before claiming completion

## Verification

- package build emits no avoidable metadata warnings for `@cortexos/sdk`

## Affected Files

- [packages/sdk/package.json](/home/sabir/projects/cortexos/packages/sdk/package.json)
