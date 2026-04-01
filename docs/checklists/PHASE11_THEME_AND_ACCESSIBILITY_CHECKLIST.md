# Phase 11 Theme And Accessibility Checklist

Goal:

- centralize UI tokens and accessibility behavior before large app work accelerates

Use with:

- [15_accessibility_input_keyboard_system.md](/home/sabir/projects/cortexos/docs/specs/15_accessibility_input_keyboard_system.md)
- [16_theme_design_tokens_ui_system.md](/home/sabir/projects/cortexos/docs/specs/16_theme_design_tokens_ui_system.md)

## Scope Of This PR

In scope:

- theme tokens
- shared UI primitives
- accessibility behaviors

Out of scope:

- app-specific visual polish

## PR Success Criteria

- accessibility settings are centrally wired
- shared UI primitives encode focus, keyboard, and ARIA behavior
- theme tokens are stable for downstream apps

## Work Order

- [ ] define theme tokens and shared package boundaries
- [ ] define focus, ARIA, contrast, and motion behavior centrally
- [ ] integrate accessibility settings with settings service expectations
- [ ] add tests/audits for focus and keyboard behavior

## Validation Checklist

- [ ] no app should need to invent its own basic accessibility primitives

## Stop Conditions

- stop if shared UI patterns would force apps to override accessibility defaults routinely
