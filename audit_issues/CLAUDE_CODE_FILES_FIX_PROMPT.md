Read these documents first:

1. /home/sabir/projects/cortexos/audit_issues/issues-list-index.md
2. /home/sabir/projects/cortexos/audit_issues/ISSUE-006-file-manager-root-path-contract-mismatch.md
3. /home/sabir/projects/cortexos/audit_issues/ISSUE-002-successful-app-launches-can-leak-orphan-runtime-instances.md
4. /home/sabir/projects/cortexos/audit_issues/ISSUE-004-launch-error-reporting-obscures-the-real-backend-failure.md

Then reconcile the fix against the owning specs before coding, especially:

- /home/sabir/projects/cortexos/docs/specs/00_master_spec.md
- /home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md
- /home/sabir/projects/cortexos/docs/specs/09_app_runtime_and_app_lifecycle.md
- /home/sabir/projects/cortexos/docs/specs/11_files_service_and_virtual_filesystem.md
- /home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md
- /home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md
- /home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md

Your job is to fix the remaining live `Files` app failure without regressing the now-working desktop app-launch path.

Current live state:

- desktop login works
- app windows open
- `Settings` and `Terminal` render inside their windows
- `Files` opens and renders, but immediately shows `Error: Failed to load directory: 400 Bad Request`
- direct backend evidence shows `GET /api/v1/files/list?path=/` fails with:
  - `invalid path: path violation: path must be relative, not absolute`

Priority:

1. resolve ISSUE-006 cleanly at the contract level
2. keep orphan-instance handling correct if any launch/open failure still occurs
3. keep frontend error reporting specific and actionable

Required rules:

- Use Context7 and current official docs for external libraries/frameworks/APIs.
- Do not guess the Files API contract; verify the backend and owning spec first.
- Prefer one canonical root-path representation across frontend and backend.
- Do not mark the issue resolved unless the live browser flow works.

Required outcomes:

- opening `Files` from the desktop loads the root directory successfully
- navigating into at least one folder and back works
- the frontend and backend agree on the root-path representation
- regression coverage is added for the root-path case

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
- open `Files`
- confirm the root directory loads without a 400
- confirm folder navigation works

When finished, report:

- exactly what root-path contract was chosen
- which files changed
- exact command results
- live browser verification results
- any remaining limitations
