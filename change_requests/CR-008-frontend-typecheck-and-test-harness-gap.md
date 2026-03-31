# CR-008: Frontend Typecheck And Test Harness Gap

Severity: High

## Problem

The frontend workspace does not yet satisfy the documented quality bar because at least one app’s test files are included in TypeScript compilation without the required test-runner types.

## Evidence

`pnpm -r typecheck` fails in `apps/notes-app` with missing globals:

- `describe`
- `test`
- `expect`

Relevant files:

- [apps/notes-app/src/__tests__/App.test.tsx](/home/sabir/projects/cortexos/apps/notes-app/src/__tests__/App.test.tsx)
- [apps/notes-app/tsconfig.json](/home/sabir/projects/cortexos/apps/notes-app/tsconfig.json)
- [apps/notes-app/package.json](/home/sabir/projects/cortexos/apps/notes-app/package.json)

## Expected Contract

The frontend workspace should typecheck cleanly under the documented build/test pipeline. Test files either need correct test-environment typing or need exclusion from the production TS config.

## Requested Change

- standardize frontend test TypeScript setup across all apps
- ensure Vitest/JSDOM type globals are configured consistently
- confirm that every app with tests can pass `typecheck` and `test` under workspace execution

## Verification

- `pnpm -r typecheck` passes
- frontend test files no longer fail due to missing test-runner globals
- app-level TS config strategy is documented and consistent

## Affected Files

- [apps/notes-app/src/__tests__/App.test.tsx](/home/sabir/projects/cortexos/apps/notes-app/src/__tests__/App.test.tsx)
- [apps/notes-app/tsconfig.json](/home/sabir/projects/cortexos/apps/notes-app/tsconfig.json)
- [apps/notes-app/package.json](/home/sabir/projects/cortexos/apps/notes-app/package.json)
