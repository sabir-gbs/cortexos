# CortexOS Live E2E Issues Index

This index records the issues found during a live audit against the running app at `http://localhost:5173` on 2026-03-31 using the provided credentials:

- user: `admin`
- password: `uglyducks22!`

Audit scope:

- login to the real desktop shell
- interact with visible desktop icons
- trace launch, window-creation, and render paths
- confirm behavior with targeted HTTP checks against the live backend

## Summary

The original live desktop launch flow was broken by multiple stacked issues. The workspace now includes fixes for the launch path, but one app-level contract gap remains:

1. the shell launch path previously sent floating-point `x`/`y` values into `openWindow`, which the server rejected
2. app launch could succeed before window creation, leaking orphan runtime instances
3. iframe-served apps previously emitted root-relative asset URLs, causing blank windows in the desktop shell
4. frontend error handling could hide the real backend cause
5. desktop icon interaction still relies on double-click, which may be surprising but is currently spec-aligned
6. the `Files` app now renders, but its initial root-path request still violates the Files API contract and returns `400`

## Issues

| ID | Severity | Title |
|---|---|---|
| [ISSUE-001](./ISSUE-001-window-launch-fails-because-shell-sends-floating-point-coordinates.md) | Critical | Window launch fails because the shell sends floating-point coordinates |
| [ISSUE-002](./ISSUE-002-successful-app-launches-can-leak-orphan-runtime-instances.md) | High | Successful app launches can leak orphan runtime instances when window creation fails |
| [ISSUE-003](./ISSUE-003-window-iframes-point-to-non-routable-or-wrong-app-urls.md) | Critical | Window iframes point to non-routable or wrong app URLs |
| [ISSUE-004](./ISSUE-004-launch-error-reporting-obscures-the-real-backend-failure.md) | Medium | Launch error reporting obscures the real backend failure |
| [ISSUE-005](./ISSUE-005-desktop-icon-launch-ux-is-misaligned-with-user-expectations.md) | Medium | Desktop icon launch UX is misaligned with user expectations |
| [ISSUE-006](./ISSUE-006-file-manager-root-path-contract-mismatch.md) | High | File Manager uses an absolute root path that the Files API rejects |

## Recommended Fix Order

1. [ISSUE-006](./ISSUE-006-file-manager-root-path-contract-mismatch.md)
2. [ISSUE-002](./ISSUE-002-successful-app-launches-can-leak-orphan-runtime-instances.md)
3. [ISSUE-004](./ISSUE-004-launch-error-reporting-obscures-the-real-backend-failure.md)
4. [ISSUE-005](./ISSUE-005-desktop-icon-launch-ux-is-misaligned-with-user-expectations.md)

## Primary Evidence Files

- [App.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/App.tsx)
- [types.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/types.ts)
- [WindowFrame.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/components/WindowFrame.tsx)
- [api.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/api.ts)
- [wm.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/routes/wm.rs)
- [main.rs](/home/sabir/projects/cortexos/crates/cortex-server/src/main.rs)
- [files.rs](/home/sabir/projects/cortexos/crates/cortex-api/src/routes/files.rs)
- [sqlite.rs](/home/sabir/projects/cortexos/crates/cortex-wm/src/sqlite.rs)
- [sqlite.rs](/home/sabir/projects/cortexos/crates/cortex-runtime/src/sqlite.rs)
