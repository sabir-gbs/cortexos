# CR-006: Manifest Schema, App Manifests, And Validator Tooling Are Out Of Sync

Severity: High

## Problem

The validator tool does not actually validate `manifest.json` content, and the current app manifests appear not to satisfy the documented SDK manifest contract.

## Evidence

Spec contract:

- [docs/specs/01_repository_toolchain_engineering_conventions.md](/home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md): CI stage 6 is manifest validation against schema
- [docs/specs/21_sdk_manifest_third_party_app_platform.md](/home/sabir/projects/cortexos/docs/specs/21_sdk_manifest_third_party_app_platform.md): manifest fields such as `id`, `version`, `entry_point`, `description`, `icon`, `permissions`, `capabilities`, `min_os_version`, `author`
- [packages/sdk/src/types/manifest.ts](/home/sabir/projects/cortexos/packages/sdk/src/types/manifest.ts): TypeScript manifest contract also requires `entry_point` and typed `permissions`

Actual tooling:

- [tools/validate-manifests.sh](/home/sabir/projects/cortexos/tools/validate-manifests.sh) only checks for:
  - `package.json`
  - `tsconfig.json`
  - `src/index.ts(x)` or `src/main.ts(x)`

Actual manifest issues observed:

- app manifests have `id`
- `.app_id` is null everywhere
- `entry_point` is null everywhere in the current manifest files sampled
- `permissions` shape is not the SDK `AppPermission[]` shape from [packages/sdk/src/types/manifest.ts](/home/sabir/projects/cortexos/packages/sdk/src/types/manifest.ts)

Examples:

- [apps/calculator-app/manifest.json](/home/sabir/projects/cortexos/apps/calculator-app/manifest.json)
- [apps/notes-app/manifest.json](/home/sabir/projects/cortexos/apps/notes-app/manifest.json)
- [apps/games/chess/manifest.json](/home/sabir/projects/cortexos/apps/games/chess/manifest.json)

## Expected Contract

Manifest schema, manifest files, SDK types, and validation tooling must agree on one canonical shape.

## Requested Change

- implement real manifest schema validation
- align all first-party app manifests to the canonical schema
- ensure validator failures block CI when a manifest is incomplete or malformed
- clarify whether first-party manifests intentionally use a different contract from third-party SDK manifests; if so, document both schemas explicitly

## Verification

- validator fails on malformed manifest content
- validator passes on all real app manifests
- manifest shape is consistent across:
  - spec 21
  - SDK types
  - actual `manifest.json` files
  - installer/runtime expectations

## Affected Files

- [tools/validate-manifests.sh](/home/sabir/projects/cortexos/tools/validate-manifests.sh)
- [packages/sdk/src/types/manifest.ts](/home/sabir/projects/cortexos/packages/sdk/src/types/manifest.ts)
- [docs/specs/21_sdk_manifest_third_party_app_platform.md](/home/sabir/projects/cortexos/docs/specs/21_sdk_manifest_third_party_app_platform.md)
- [apps/calculator-app/manifest.json](/home/sabir/projects/cortexos/apps/calculator-app/manifest.json)
- [apps/notes-app/manifest.json](/home/sabir/projects/cortexos/apps/notes-app/manifest.json)
