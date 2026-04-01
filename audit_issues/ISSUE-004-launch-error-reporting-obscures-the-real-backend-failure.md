# ISSUE-004: Launch Error Reporting Obscures The Real Backend Failure

Severity: Medium

## What Happens

The frontend does not surface the actual window-creation failure cleanly. Instead, it logs JSON parse noise and shows a generic message such as `Unprocessable Entity`.

## Evidence

Observed browser console output:

- `Failed to parse error response body: SyntaxError: Unexpected token 'F', "Failed to "... is not valid JSON`

Relevant code:

- [api.ts](/home/sabir/projects/cortexos/apps/desktop-shell/src/api.ts)
  - error parsing assumes JSON first
  - plain-text framework errors fall back poorly

Observed user-facing impact:

- low-signal message like `Failed to launch Terminal: Unprocessable Entity`
- the real actionable reason is hidden:
  - floating-point `x`
  - expected `i32`

## Why It Matters

This slows debugging and creates the impression that “nothing happens” rather than exposing the concrete contract failure.

## Recommended Fix

- make backend validation errors consistently return canonical JSON error bodies
- make the frontend gracefully fall back to plain-text error bodies when JSON parsing fails
- include field-level validation details in the surfaced error message where safe

