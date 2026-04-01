Read these documents first:

1. /home/sabir/projects/cortexos/audit_issues/issues-list-index.md
2. /home/sabir/projects/cortexos/audit_issues/ISSUE-007-minimized-window-remains-focused-and-focus-does-not-transfer.md
3. /home/sabir/projects/cortexos/audit_issues/ISSUE-002-successful-app-launches-can-leak-orphan-runtime-instances.md
4. /home/sabir/projects/cortexos/audit_issues/ISSUE-004-launch-error-reporting-obscures-the-real-backend-failure.md

Then reconcile the fix against the owning specs before coding, especially:

- /home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md
- /home/sabir/projects/cortexos/docs/specs/08_window_manager.md
- /home/sabir/projects/cortexos/docs/specs/09_app_runtime_and_app_lifecycle.md
- /home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md
- /home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md
- /home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md

Your job is to fix the minimize/focus contract cleanly without regressing the currently working desktop launch and restore paths.

Current live defect:

- minimizing a window hides it
- but the backend still returns the minimized window as `focused: true`
- no other visible topmost window is promoted to focused
- this violates the window-manager spec

Key evidence:

- `crates/cortex-wm/src/sqlite.rs`
  - `minimize_window()` delegates to `update_state(window_id, "minimized")`
  - `update_state()` changes `state` only and does not repair focus
- the shell applies the server response directly, so taskbar focus state follows the bad backend state

Required behavior:

1. minimizing a focused window must:
   - clear focus from that window
   - promote the next topmost eligible visible window in the same workspace
2. minimizing a non-focused window must not disturb the current focused visible window
3. if all windows are minimized, no window should remain focused
4. restoring a minimized window from the taskbar must make it visible and focused again

Rules:

- Fix the source-of-truth logic in the window-manager service first.
- Do not patch only the frontend.
- Use Context7 and current official docs for any external APIs if needed.
- Do not mark the issue resolved unless live browser behavior and backend state both match the spec.

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
- open at least two windows on the same workspace
- minimize the focused one
- confirm:
  - minimized window disappears
  - minimized window is no longer focused
  - next visible topmost window becomes focused
- minimize all windows
- confirm no window remains focused
- restore a minimized window from the taskbar
- confirm it becomes visible and focused

When finished, report:

- which files changed
- exactly how focus reassignment works after minimize
- exact command results
- live browser verification results
- any remaining limitations
