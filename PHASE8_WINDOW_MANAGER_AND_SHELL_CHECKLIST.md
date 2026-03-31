# Phase 8 Window Manager And Shell Checklist

Goal:

- build the interactive desktop surface on top of the typed bus and canonical runtime state

Use with:

- [07_desktop_shell.md](/home/sabir/projects/cortexos/docs/specs/07_desktop_shell.md)
- [08_window_manager.md](/home/sabir/projects/cortexos/docs/specs/08_window_manager.md)
- [15_accessibility_input_keyboard_system.md](/home/sabir/projects/cortexos/docs/specs/15_accessibility_input_keyboard_system.md)

## Scope Of This PR

In scope:

- window state model
- shell integration
- canonical shortcut handling
- snapshot/bootstrap reads

Out of scope:

- rich visual polish
- full app ecosystem

## PR Success Criteria

- realtime window mutations use the bus
- shell subscribes to canonical events only
- bootstrap/snapshot reads are limited to documented HTTP usage
- shell focus and keyboard behavior are accessible

## Work Order

- [ ] define window and workspace state
- [ ] implement canonical window lifecycle commands/events
- [ ] wire shell subscriptions
- [ ] verify `Ctrl+Space` and `Ctrl+Shift+A` are not redefined locally
- [ ] add tests for shell response to window events and focus changes

## Validation Checklist

- [ ] no legacy client REST mutation path is reintroduced
- [ ] taskbar/workspace behavior matches spec-owned events

## Stop Conditions

- stop if shell behavior requires inventing new window contracts not in specs 07/08
