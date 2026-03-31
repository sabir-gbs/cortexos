# Claude Code Single Prompt

Use the following single prompt in Claude Code with GLM-5.1 as the coding model.

```text
You are implementing the entire CortexOS project from this repository, but you must do it strictly phase by phase and strictly from the documented contracts.

Start by reading and treating these documents as authoritative in this order:

1. /home/sabir/projects/cortexos/DOCUMENTATION_INDEX.md
2. /home/sabir/projects/cortexos/MASTER_IMPLEMENTATION_BRIEF.md
3. /home/sabir/projects/cortexos/AGENTS.md
4. /home/sabir/projects/cortexos/CLAUDE.md
5. /home/sabir/projects/cortexos/docs/specs/00_master_spec.md
6. /home/sabir/projects/cortexos/docs/specs/01_repository_toolchain_engineering_conventions.md
7. /home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md
8. /home/sabir/projects/cortexos/docs/specs/appendix_d_coding_agent_guardrails.md
9. /home/sabir/projects/cortexos/docs/specs/spec_audit_2026-03-30.md

Then use these planning documents:

- /home/sabir/projects/cortexos/IMPLEMENTATION_KICKOFF.md
- /home/sabir/projects/cortexos/IMPLEMENTATION_PHASES.md
- /home/sabir/projects/cortexos/PHASE1_BOOTSTRAP_CHECKLIST.md
- /home/sabir/projects/cortexos/IMPLEMENTATION_PHASE_CHECKLISTS.md
- /home/sabir/projects/cortexos/PHASE2_RUST_FOUNDATION_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE3_PERSISTENCE_AND_OBSERVABILITY_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE4_IDENTITY_AND_POLICY_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE5_SETTINGS_AND_FILESYSTEM_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE6_AI_RUNTIME_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE7_COMMAND_BUS_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE8_WINDOW_MANAGER_AND_SHELL_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE9_APP_RUNTIME_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE10_CORE_SYSTEM_SERVICES_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE11_THEME_AND_ACCESSIBILITY_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE12_FIRST_PARTY_APPS_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE13_GAMES_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE14_AI_UX_AND_SAFETY_UX_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE15_SDK_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE16_ADMIN_CHECKLIST.md
- /home/sabir/projects/cortexos/PHASE17_RELEASE_GATES_CHECKLIST.md

Rules:

- Do not hallucinate missing code, workspaces, crates, apps, or scripts as already existing.
- Follow the exact phase order from Phase 1 through Phase 17.
- Do not skip ahead if the lower layer is incomplete.
- Treat specs as implementation contracts.
- Child specs are authoritative inside their subsystem scope.
- For external libraries, frameworks, SDKs, and platform APIs, always consult Context7 and current official documentation before implementing or revising usage.
- Do not rely on model memory for API signatures, configuration keys, migration patterns, or framework best practices when Context7 can provide current docs.
- Prefer the current supported mechanism over legacy or deprecated patterns, and if a legacy pattern must be used, explain why explicitly.
- Never bypass server-side auth, policy, typed command/event contracts, or canonical storage ownership.
- Never make the client authoritative for auth, permissions, or structured persistence.
- Keep event names, shortcut values, and settings keys aligned with the docs.
- If you encounter a true spec ambiguity or cross-subsystem contradiction, stop and ask for a documentation decision instead of guessing.

Execution instructions:

1. Begin with Phase 1 only.
2. Read the Phase 1 checklist and implement only that phase.
3. Run the relevant verification commands for that phase.
4. Report exactly what was completed, what failed, and whether Phase 1 is truly done.
5. Only after Phase 1 is complete, continue to Phase 2, then Phase 3, and so on.
6. At every phase, use the corresponding phase checklist file as the execution boundary.
7. Keep code, tests, and docs aligned as you go.

Output style:

- Be explicit about the current phase.
- Name the exact docs you are using.
- Explain any blocker as a documentation or contract issue, not as a guess.
- Do not claim a phase is complete unless its checklist and Definition of Done are satisfied.

Start now with Phase 1.
```
