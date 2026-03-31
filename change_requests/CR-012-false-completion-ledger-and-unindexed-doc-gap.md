# CR-012: `open-task-list.md` Is Unindexed And Makes False Completion Claims

Severity: High

## Problem

There is a top-level documentation file that is not linked from the primary documentation index and currently asserts completion/pass status that is contradicted by the current repository.

## Evidence

[open-task-list.md](/home/sabir/projects/cortexos/open-task-list.md) claims, among other things:

- `cargo clippy --workspace` passes
- `pnpm typecheck` passes
- `pnpm test` passes
- “everything through the following has been verified”
- “no stubs/placeholders”

These claims conflict with current audit evidence:

- `cargo clippy --workspace --all-targets -- -D warnings` fails
- `pnpm -r typecheck` fails in `apps/notes-app`
- the repo still contains documented placeholder/stub behavior in core user-facing paths

The file is also absent from [DOCUMENTATION_INDEX.md](/home/sabir/projects/cortexos/DOCUMENTATION_INDEX.md), which means there is no clear control-plane status for whether it is authoritative, stale, historical, or private scratch documentation.

## Expected Contract

Top-level docs that assert completion or verification status must be:

- accurate
- indexed if authoritative
- clearly marked as non-authoritative if they are merely local planning notes

## Requested Change

- either remove or correct the false completion claims in `open-task-list.md`
- add the file to the documentation index if it is intended to remain authoritative
- otherwise label it explicitly as a non-authoritative working note

## Verification

- no top-level doc claims gate success when the gates currently fail
- the documentation index either includes the file or the file clearly states it is non-authoritative

## Affected Files

- [open-task-list.md](/home/sabir/projects/cortexos/open-task-list.md)
- [DOCUMENTATION_INDEX.md](/home/sabir/projects/cortexos/DOCUMENTATION_INDEX.md)
