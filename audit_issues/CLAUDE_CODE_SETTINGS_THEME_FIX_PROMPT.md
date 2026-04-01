Read these documents first:

1. /home/sabir/projects/cortexos/audit_issues/issues-list-index.md
2. /home/sabir/projects/cortexos/audit_issues/ISSUE-008-settings-app-theme-controls-are-local-only-and-do-not-update-shell-theme.md
3. /home/sabir/projects/cortexos/audit_issues/ISSUE-007-minimized-window-remains-focused-and-focus-does-not-transfer.md
4. /home/sabir/projects/cortexos/audit_issues/ISSUE-006-file-manager-root-path-contract-mismatch.md

Then reconcile the fix against the owning specs before coding, especially:

- /home/sabir/projects/cortexos/docs/specs/05_settings_service_and_settings_app.md
- /home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md
- /home/sabir/projects/cortexos/docs/specs/10_system_command_bus_and_event_model.md
- /home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md
- /home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md
- /home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md

Your job is to make the standalone `Settings` app a real client of the canonical settings system for theme changes, instead of a local-only mock.

Current defect:

- changing theme from the shell-owned settings surface works
- changing theme from the standalone `Settings` app window does not update the shell theme
- `apps/settings-app/src/App.tsx` currently uses isolated local React state and does not call the settings API or notify the shell

Required outcomes:

1. the `Settings` app reads the canonical theme setting on load
2. changing theme from the `Settings` app updates the canonical setting
3. the desktop shell theme changes immediately without reload
4. theme state persists and stays consistent across the shell-owned settings surface and the standalone `Settings` app

Rules:

- Do not keep the current local-only theme mock as the source of truth.
- Do not introduce a second independent theme state.
- Use Context7 and current official docs for external APIs/libraries if needed.
- Reconcile with the existing shell settings flow rather than bypassing it.

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
- open the standalone `Settings` app window from the desktop
- switch dark/light mode from inside that window
- confirm the desktop shell `data-theme` changes immediately
- open the shell-owned settings surface and confirm it reflects the same value
- reload and confirm the chosen theme persists

When finished, report:

- which files changed
- how the standalone settings app now reads/writes canonical settings
- how shell theme propagation works
- exact command results
- live browser verification results
- any remaining limitations
