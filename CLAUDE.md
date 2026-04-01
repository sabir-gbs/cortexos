# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Status

This is a new project. Development commands will be added as the project develops.

The repository currently contains:

- **17 Rust crates** in `crates/`: cortex-admin, cortex-ai, cortex-api, cortex-auth, cortex-config, cortex-core, cortex-db, cortex-files, cortex-notify, cortex-observability, cortex-policy, cortex-runtime, cortex-sdk, cortex-search, cortex-server, cortex-settings, cortex-wm
- **Desktop shell + frontend apps** in `apps/`: ai-assistant, calculator-app, clock-utils-app, desktop-shell, file-manager-app, games-platform-app, media-viewer-app, notes-app, settings-app, shared, terminal-lite-app, text-editor-app, and the games monorepo (snake, minesweeper, solitaire, tetris, chess)
- **3 shared packages** in `packages/`: ai-client, game-framework, sdk
- **CI workflow**: `.github/workflows/ci.yml`

Current quality-gate status (as of 2026-03-31):

- Frontend: `pnpm -r build` passes, `pnpm test` passes (544 tests), `pnpm -r typecheck` passes
- Rust: `cargo test --workspace` passes (all tests green), `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes
- `cargo run --bin check-deps` passes (17 crates, 0 violations)
- `cargo fmt --all -- --check` passes
- `bash tools/validate-manifests.sh` passes (14 manifests, 0 errors)
- `cargo audit` passes (190 crate dependencies, 0 vulnerabilities)
- E2E: `pnpm e2e` passes (3 Playwright frontend shell smoke tests — login screen rendering, form error handling, index page)
- See `change_requests/INDEX.md` for the full audit and outstanding issues

## Development Setup

### Prerequisites

- Rust toolchain (see `rust-toolchain.toml` for pinned version)
- Node.js (see `.node-version` for pinned version)
- pnpm (workspace package manager)

### Install

```bash
# Rust workspace
cargo build --workspace

# Frontend workspace
pnpm install
pnpm -r build
```

## Architecture

Use `docs/guides/MASTER_IMPLEMENTATION_BRIEF.md` as the top-level execution reference for building CortexOS.
Use `docs/guides/DOCUMENTATION_INDEX.md` as the one-page link map for every spec, planning doc, and checklist.

## Build & Test Commands

### Rust (backend)

```bash
cargo build --workspace          # Build all crates
cargo test --workspace           # Run all Rust tests
cargo clippy --workspace         # Lint all crates
cargo fmt --all -- --check       # Check formatting
```

### Frontend

```bash
pnpm install                     # Install dependencies
pnpm -r build                    # Build all apps/packages
pnpm test                        # Run all frontend unit tests
pnpm typecheck                   # Type-check all packages
```

### E2E (Playwright browser tests)

```bash
pnpm e2e                         # Run Playwright frontend shell smoke tests (auto-starts dev server)
```

## Available Skills

This project has enhanced capabilities through specialized skills. Prefer using these skills when applicable:

### Code Quality & Conventions
- **`project-conventions`** (auto-applied) - Background knowledge about code patterns, naming conventions, and best practices. Automatically applied when writing or reviewing code.
- **`/gen-test <file>`** - Generate tests for a file following project conventions and patterns
- **`/pr-check`** - Review pull requests against project checklist with comprehensive analysis

### Development Workflows
- **`/commit`** - Create git commits with proper, well-formatted commit messages
- **`/feature-dev`** - End-to-end feature development workflow from planning to implementation
- **`/setup-dev`** - Set up development environment for new contributors with prerequisite checks

### Frontend Development
- **`/frontend-design`** - Create polished, professional UI components (React/Vue/Angular) with distinctive design

### Plugin & Skill Development
- **`/skill-development`** - Create custom skills with proper structure and templates

### When to Use Skills

**Automatic (no invocation needed):**
- `project-conventions` - Applied automatically when you write or review code

**Manual invocation (use `/skill-name`):**
- Creating commits → `/commit`
- Building UI components → `/frontend-design`
- Writing tests → `/gen-test <file>`
- Reviewing PRs → `/pr-check`
- Starting new feature → `/feature-dev`
- Onboarding new developers → `/setup-dev`
- Creating custom skills → `/skill-development`

### Skill Best Practices

- Skills encapsulate expertise and workflows - use them instead of reinventing patterns
- Skills can include templates, scripts, and reference examples
- Some skills run in isolated subagents (like `/pr-check`) for focused analysis
- Skills can be configured to only allow user invocation for safety (like `/gen-test`)
