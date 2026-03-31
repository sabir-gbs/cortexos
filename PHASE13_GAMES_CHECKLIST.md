# Phase 13 Games Checklist

Goal:

- implement bundled games as standard platform consumers

Use with:

- [18_games_platform_parent.md](/home/sabir/projects/cortexos/docs/specs/18_games_platform_parent.md)
- game specs `18a` through `18e`

## Scope Of This PR

In scope:

- games platform shell
- bundled game implementations
- game save/load via platform services

Out of scope:

- non-specified multiplayer or AI opponents

## PR Success Criteria

- games behave as app-platform consumers
- documented gameplay rules are implemented where the specs are precise

## Work Order

- [ ] implement games platform scaffolding
- [ ] implement games in chosen order
- [ ] verify save/load uses canonical file services
- [ ] verify Tetris mechanics match reconciled rule details
- [ ] verify Chess draw handling matches reconciled rule details
- [ ] add deterministic tests for core game rules

## Validation Checklist

- [ ] no game introduces special storage or runtime exceptions

## Stop Conditions

- stop if a game requires rules not defined in its spec and those rules would affect correctness materially
