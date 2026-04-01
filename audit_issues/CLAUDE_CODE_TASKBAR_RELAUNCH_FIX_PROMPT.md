Read these documents first:

1. /home/sabir/projects/cortexos/audit_issues/issues-list-index.md
2. /home/sabir/projects/cortexos/audit_issues/ISSUE-009-pinned-taskbar-icon-does-not-relaunch-app-after-window-close.md
3. /home/sabir/projects/cortexos/audit_issues/ISSUE-007-minimized-window-remains-focused-and-focus-does-not-transfer.md
4. /home/sabir/projects/cortexos/audit_issues/ISSUE-008-settings-app-theme-controls-are-local-only-and-do-not-update-shell-theme.md

Then reconcile the fix against the owning specs before coding, especially:

- /home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md
- /home/sabir/projects/cortexos/docs/specs/08_window_manager.md
- /home/sabir/projects/cortexos/docs/specs/09_app_runtime_and_app_lifecycle.md
- /home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md
- /home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md
- /home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md

Your job is to make pinned taskbar icons behave as reliable app launch affordances after the last window is closed.

Current defect:

- open a pinned app such as `Settings`
- close it with the `X` button
- the pinned taskbar icon remains visible
- interacting with that icon does not relaunch the app

Current code evidence:

- `apps/desktop-shell/src/components/Taskbar.tsx`
  - taskbar click uses `instanceId` when present
  - otherwise falls back to `onLauncherClick()`
- `apps/desktop-shell/src/App.tsx`
  - `handleTaskbarAppClick(instanceId)` only restores/focuses an existing window
  - it has no relaunch path when no matching live window exists

Required behavior:

1. if the app has a live window, taskbar click should focus/restore it
2. if the app is pinned but not currently open, taskbar click should launch it directly
3. this must work for at least `Settings` and `Terminal`
4. existing running/minimized behavior must not regress

Rules:

- Do not patch this by relying on double-click quirks.
- Keep taskbar semantics deterministic.
- Prefer a clean pinned-app launch path over opening the launcher as a fallback.
- Use Context7 and current official docs for any external APIs/libraries if needed.

Required verification:

- cargo fmt --all -- --check
- cargo clippy --workspace --all-targets --all-features -- -D warnings
- cargo test --workspace
- cargo run --bin check-deps
- cargo audit
- pnpm test
- pnpm -r typecheck
- bash tools/validate-manifests.sh
- pnpm e2e

Additional live verification:

- use http://localhost:5173
- log in with:
  - username: admin
  - password: uglyducks22!
- open `Settings`
- close it with `X`
- click the pinned `Settings` taskbar icon
- confirm the app relaunches
- repeat with `Terminal`
- verify minimized-window restore still works from the taskbar

When finished, report:

- which files changed
- how taskbar click now distinguishes running vs pinned-not-running apps
- exact command results
- live browser verification results
- any remaining limitations
