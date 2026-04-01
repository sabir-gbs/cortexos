# ISSUE-005: Desktop Icon Launch UX Is Misaligned With User Expectations

Severity: Medium

## What Happens

Desktop icons launch on double-click or `Enter`, but not on single click. From a user perspective, “clicking an app does nothing,” which matches the reported behavior.

## Evidence

- [DesktopIconGrid.tsx](/home/sabir/projects/cortexos/apps/desktop-shell/src/components/DesktopIconGrid.tsx)
  - launch is bound to `onDoubleClick`
  - keyboard launch is bound to `Enter`
  - single click only begins selection/drag interaction

Compounding factors:

- there is no strong selected-state or affordance documented in this path
- `Trash` is rendered as an icon with `app_id: null`, so interaction also appears to do nothing there by design

## Why It Matters

Even after backend fixes, desktop interaction may still feel broken to users if the launch gesture is not obvious.

## Recommended Fix

- explicitly decide whether the product wants:
  - single-click to select + double-click to open, or
  - single-click to open
- if double-click remains the design:
  - add a clear selected state
  - ensure first click produces visible feedback
  - consider tooltip/help text or onboarding cue
- if single-click is the intended product behavior, update the desktop icon interaction contract and implementation accordingly

