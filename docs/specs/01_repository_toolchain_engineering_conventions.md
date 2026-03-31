# Spec 01: Repository, Toolchain, and Engineering Conventions

**Status:** Implementation-grade
**Owner crate:** Cross-cutting (no single owner; enforced by CI and code review)
**Depends on:** None (this is the foundational convention spec)
**Required by:** All other specs, all crates, all apps

---

## 1. Purpose

Establish the authoritative repository structure, build toolchain configuration, naming conventions, testing conventions, dependency rules, CI requirements, and local development workflows for the CortexOS project. This document is the single source of truth for how the codebase is organized, how crates relate to each other, how code is validated before merge, and how engineers set up and run the project locally.

Every engineer, contributor, and automated agent (including Claude Code and Codex) must follow these conventions without exception. Violations detected by CI block merge. Violations not caught by CI must be corrected during code review.

---

## 2. Scope

**In scope for this spec:**

- Repository layout: monorepo directory structure, Cargo workspace configuration
- Crate inventory: every crate, its directory path, and its one-sentence purpose
- Dependency graph rules: which crates may depend on which, and which may not
- Naming conventions: crate names, Rust type names, API route patterns, event names, file names, directory names
- Build toolchain: Rust version, TypeScript version, required toolchain components
- CI pipeline: stages, commands, failure policies
- Testing conventions: where tests live, how they are categorized, naming patterns
- Local development workflow: how to build, run, test, and debug
- Migration conventions: SQL migration file naming and ordering
- Code formatting and linting rules
- Commit message format
- Branch naming conventions

**Owned by this spec:**

- All structural decisions about the repository
- All CI/CD configuration
- All toolchain version pinning

---

## 3. Out of Scope

- **Application behavior:** What each crate does internally is defined by its own spec (specs 02 through 23 and appendices).
- **API request/response schemas:** Defined per-subsystem in their respective specs.
- **Deployment and release procedures:** Owned by spec 23 (Release Readiness, QA, and Acceptance Framework).
- **Design tokens and visual style:** Owned by spec 16 (Theme/Design Tokens).
- **Specific library choices within crates:** Each spec recommends libraries; this spec only governs the version of the language toolchains (Rust, TypeScript) and the build system.
- **Git hosting provider configuration:** This spec defines branch naming and commit format, not GitHub/GitLab-specific settings like branch protection rules (those are documented in the wiki).
- **IDE or editor configuration:** Contributors may use any editor. Editor config files (`.vscode/`, `.idea/`) are gitignored.

---

## 4. Objectives

1. Provide a deterministic, reproducible build that produces the same artifacts given the same source commit, regardless of the developer's machine.
2. Enforce a strict, acyclic dependency graph between crates so that no crate is coupled to a crate it does not need.
3. Guarantee that every PR passes formatting, linting, type-checking, unit tests, integration tests, and clippy before merge.
4. Make onboarding trivial: a new contributor runs one command to build and one command to run the full project.
5. Ensure that naming is consistent across Rust code, TypeScript code, API routes, WebSocket events, and file paths so that any name can be predicted from any other.
6. Ensure that database schema changes are always forward-only, versioned, and reversible in case of emergency.

---

## 5. User-Visible Behavior

This spec governs infrastructure and conventions that are not directly visible to the end user. However, the following developer-facing behaviors arise from these conventions:

### 5.1 Build Commands

- Running `cargo build` from the repository root builds all Rust crates in dependency order.
- Running `cargo build -p cortex-api` builds only the API crate and its transitive dependencies.
- Running `pnpm --filter desktop-shell dev` starts the desktop shell in development mode with hot module replacement.
- Running `docker-compose up` starts optional external dependencies (SQLite is embedded by default and does not require Docker).

### 5.2 Test Commands

- `cargo test` runs all unit tests and integration tests across the workspace.
- `cargo test -p cortex-auth` runs tests for a single crate.
- `pnpm --filter desktop-shell test` runs TypeScript tests for the desktop shell.
- `cargo test --test e2e` runs end-to-end tests that start the server and drive browser automation.

### 5.3 CI Feedback

- Every PR receives a pass/fail status for each CI stage within 15 minutes of opening or updating.
- CI failures include the exact command, the exact error output, and a link to the relevant section of this spec.

---

## 6. System Behavior

### 6.1 Workspace Resolution

The Cargo workspace is defined in the root `Cargo.toml`. The workspace contains every Rust crate under `/crates/`. Apps under `/apps/` are TypeScript projects and are not members of the Cargo workspace, but they are part of the monorepo.

The root `Cargo.toml` uses `[workspace]` with `members = ["crates/*"]`. It does NOT use `exclude`. Every directory under `/crates/` must contain a valid `Cargo.toml` that is a workspace member.

### 6.2 TypeScript Workspace

The root `package.json` and `pnpm-workspace.yaml` declare `/apps/*` as workspace members. Each app under `/apps/` has its own `package.json` with its own dependencies and scripts. Apps do not depend on each other at the package level.

### 6.3 Dependency Resolution Order

When building the full workspace, Cargo resolves dependencies in topological order. `cortex-core` builds first (zero internal dependencies). Service crates build next. `cortex-api` builds last (depends on all service crates). The build must complete with zero warnings treated as errors in CI (see section 12 for how warnings are handled during local development).

### 6.4 CI Pipeline Execution

The CI pipeline runs stages sequentially. A stage failure stops the pipeline. The stages are:

1. **Format check:** `cargo fmt --check` and `pnpm format:check`
2. **Lint:** `cargo clippy --all-targets --all-features -- -D warnings` and `pnpm lint`
3. **Type check:** `pnpm typecheck` across all TypeScript apps
4. **Build:** `cargo build` and `pnpm build`
5. **Unit and integration tests:** `cargo test` and `pnpm test`
6. **Manifest validation:** a custom script that validates all `manifest.json` files in `/apps/*/` against the app manifest schema
7. **Dependency graph validation:** `cargo run --bin check-deps` validates that no crate has forbidden dependencies per section 8.4
8. **E2E tests:** `cargo test --test e2e` (runs only on PRs targeting main, not on draft PRs)

---

## 7. Architecture

### 7.1 Repository Directory Tree

```
cortexos/
├── Cargo.toml                  # Workspace root (members = ["crates/*"])
├── Cargo.lock                  # Checked in, never gitignored
├── package.json                # Root package.json for pnpm workspace
├── pnpm-workspace.yaml         # Declares apps/* as workspace members
├── pnpm-lock.yaml              # Checked in, never gitignored
├── docker-compose.yml           # Optional: Postgres, Redis, etc.
├── rust-toolchain.toml          # Pins Rust version and components
├── .node-version                # Pins Node.js version
├── clippy.toml                  # Workspace-wide clippy configuration
├── rustfmt.toml                 # Workspace-wide rustfmt configuration
├── tsconfig.base.json           # Shared TypeScript configuration for all apps
├── Makefile                     # Convenience make targets
├── .cargo/
│   └── config.toml              # Cargo build configuration
├── .github/
│   └── workflows/
│       └── ci.yml               # CI pipeline definition
├── docs/
│   └── specs/                   # Specification documents (this file lives here)
├── crates/
│   ├── cortex-core/             # Shared types, error types, no internal deps
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       └── ids.rs           # All ID newtypes (UserId, SessionId, etc.)
│   ├── cortex-config/           # Configuration loading
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       └── loader.rs
│   ├── cortex-db/               # Database layer (SQLite default)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── error.rs
│   │   │   ├── pool.rs
│   │   │   └── migration.rs
│   │   └── migrations/          # Numbered SQL migration files
│   │       ├── 0001_create_schema_version.up.sql
│   │       ├── 0001_create_schema_version.down.sql
│   │       ├── 0002_create_users.up.sql
│   │       ├── 0002_create_users.down.sql
│   │       └── ...
│   ├── cortex-api/              # HTTP/WS API server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── routes/
│   │       │   ├── mod.rs
│   │       │   ├── auth.rs
│   │       │   ├── files.rs
│   │       │   ├── settings.rs
│   │       │   ├── ai.rs
│   │       │   ├── apps.rs
│   │       │   ├── notifications.rs
│   │       │   ├── search.rs
│   │       │   └── admin.rs
│   │       ├── ws.rs            # WebSocket handler
│   │       └── middleware/
│   │           ├── auth.rs
│   │           ├── cors.rs
│   │           └── logging.rs
│   ├── cortex-auth/             # Authentication, sessions, user profiles
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       ├── service.rs       # AuthService trait + impl
│   │       ├── session.rs
│   │       └── password.rs
│   ├── cortex-policy/           # Permissions, grants, enforcement
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       ├── service.rs       # PolicyService trait + impl
│   │       └── evaluator.rs
│   ├── cortex-settings/         # Settings service, schema validation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       ├── service.rs       # SettingsService trait + impl
│   │       └── schema.rs
│   ├── cortex-ai/               # AI runtime, provider adapters, routing
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       ├── service.rs       # AiService trait + impl
│   │       ├── router.rs
│   │       └── providers/
│   │           ├── mod.rs
│   │           ├── openai.rs
│   │           ├── anthropic.rs
│   │           └── local.rs
│   ├── cortex-files/            # Virtual filesystem, storage abstraction
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       ├── service.rs       # FilesService trait + impl
│   │       └── storage.rs
│   ├── cortex-search/           # Search indexing, command palette
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       ├── service.rs       # SearchService trait + impl
│   │       └── indexer.rs
│   ├── cortex-notify/           # Notification service
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       └── service.rs       # NotifyService trait + impl
│   ├── cortex-observability/    # Logging, metrics, tracing
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       ├── logging.rs
│   │       ├── metrics.rs
│   │       └── health.rs
│   ├── cortex-runtime/          # App lifecycle, sandboxing
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       ├── service.rs       # RuntimeService trait + impl
│   │       ├── lifecycle.rs
│   │       └── sandbox.rs
│   ├── cortex-sdk/              # Third-party app SDK types
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       └── manifest.rs
│   └── cortex-admin/            # Diagnostics, recovery
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── error.rs
│           ├── types.rs
│           ├── service.rs       # AdminService trait + impl
│           └── diagnostics.rs
├── apps/
│   ├── desktop-shell/           # TypeScript desktop shell
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   ├── manifest.json
│   │   └── src/
│   │       ├── index.tsx
│   │       ├── App.tsx
│   │       ├── components/
│   │       ├── hooks/
│   │       ├── services/        # API client layer
│   │       └── styles/
│   ├── settings-app/            # Settings application
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   ├── manifest.json
│   │   └── src/
│   ├── calculator-app/          # Calculator application
│   │   ├── package.json
│   │   ├── tsconfig.json
│   │   ├── manifest.json
│   │   └── src/
│   ├── text-editor-app/         # Text editor application
│   ├── notes-app/               # Notes application
│   ├── file-manager-app/        # File manager application
│   ├── media-viewer-app/        # Media viewer application
│   ├── terminal-lite-app/       # Terminal-lite application
│   ├── clock-utils-app/         # Clock and utility application
│   └── games/
│       ├── solitaire/           # Solitaire game
│       │   ├── package.json
│       │   ├── tsconfig.json
│       │   ├── manifest.json
│       │   └── src/
│       ├── minesweeper/         # Minesweeper game
│       ├── snake/               # Snake game
│       ├── tetris/              # Tetris-like puzzle game
│       └── chess/               # Chess game
├── tests/
│   └── e2e/                     # End-to-end test suite
│       ├── main.rs
│       ├── auth_tests.rs
│       ├── files_tests.rs
│       └── app_lifecycle_tests.rs
├── tools/
│   ├── check-deps.rs            # Dependency graph validator
│   ├── check-api-types.rs       # Rust-TypeScript type sync validator
│   ├── generate-api-types.rs    # Generate TypeScript types from Rust types
│   └── validate-manifests.sh    # App manifest validation script
├── scripts/
│   ├── dev.sh                   # Local development startup script
│   ├── setup.sh                 # First-time environment setup
│   └── migrate.sh               # Database migration runner
└── specs/ -> docs/specs/        # Symlink for convenience
```

### 7.2 Crate Inventory and Purpose

| Crate | Directory | Purpose | Internal Dependencies |
|---|---|---|---|
| `cortex-core` | `/crates/cortex-core/` | Shared types, error types, domain primitives. The foundation crate with zero internal dependencies. | None |
| `cortex-config` | `/crates/cortex-config/` | Loads and validates configuration from files, environment variables, and CLI flags. | `cortex-core` |
| `cortex-db` | `/crates/cortex-db/` | Database access layer. Schema migrations, query helpers, connection pooling. SQLite by default. | `cortex-core`, `cortex-config` |
| `cortex-auth` | `/crates/cortex-auth/` | User identity, authentication, session management, user profiles. | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-policy` | `/crates/cortex-policy/` | Permission model, grant management, enforcement engine. | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-settings` | `/crates/cortex-settings/` | Settings CRUD, schema validation, per-user and system-level settings resolution. | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-ai` | `/crates/cortex-ai/` | AI provider adapters (OpenAI, Anthropic, local), routing, prompt management, streaming. | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-files` | `/crates/cortex-files/` | Virtual filesystem, storage backends, file metadata, directory operations. | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-search` | `/crates/cortex-search/` | Full-text search indexing, command palette search, result ranking. | `cortex-core`, `cortex-config`, `cortex-db`, `cortex-files` |
| `cortex-notify` | `/crates/cortex-notify/` | Notification creation, delivery, read status, notification center backend. | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-observability` | `/crates/cortex-observability/` | Structured logging, metrics collection, distributed tracing, health checks. | `cortex-core`, `cortex-config` |
| `cortex-runtime` | `/crates/cortex-runtime/` | App lifecycle management, sandboxing, process isolation, manifest parsing. | `cortex-core`, `cortex-config`, `cortex-db`, `cortex-policy`, `cortex-sdk` |
| `cortex-sdk` | `/crates/cortex-sdk/` | Types and helpers for third-party app developers. Public API surface. | `cortex-core` |
| `cortex-admin` | `/crates/cortex-admin/` | System diagnostics, database integrity checks, backup/restore, recovery tools. | `cortex-core`, `cortex-config`, `cortex-db`, `cortex-observability` |
| `cortex-api` | `/crates/cortex-api/` | HTTP and WebSocket API server. The aggregation layer that wires all services together. | All crates above |

### 7.3 Workspace Root Cargo.toml

The root `Cargo.toml` must use `[workspace]` with explicit member listing:

```toml
[workspace]
resolver = "2"
members = [
    "crates/cortex-core",
    "crates/cortex-config",
    "crates/cortex-db",
    "crates/cortex-api",
    "crates/cortex-auth",
    "crates/cortex-policy",
    "crates/cortex-settings",
    "crates/cortex-ai",
    "crates/cortex-files",
    "crates/cortex-search",
    "crates/cortex-notify",
    "crates/cortex-observability",
    "crates/cortex-runtime",
    "crates/cortex-sdk",
    "crates/cortex-admin",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
rust-version = "1.85"

[workspace.dependencies]
# Workspace-internal crates
cortex-core = { path = "crates/cortex-core" }
cortex-config = { path = "crates/cortex-config" }
cortex-db = { path = "crates/cortex-db" }
cortex-auth = { path = "crates/cortex-auth" }
cortex-policy = { path = "crates/cortex-policy" }
cortex-settings = { path = "crates/cortex-settings" }
cortex-ai = { path = "crates/cortex-ai" }
cortex-files = { path = "crates/cortex-files" }
cortex-search = { path = "crates/cortex-search" }
cortex-notify = { path = "crates/cortex-notify" }
cortex-observability = { path = "crates/cortex-observability" }
cortex-runtime = { path = "crates/cortex-runtime" }
cortex-sdk = { path = "crates/cortex-sdk" }
cortex-admin = { path = "crates/cortex-admin" }

# External dependencies (shared versions)
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
uuid = { version = "1.0", features = ["v7", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
anyhow = "1.0"
```

### 7.4 Frontend Workspace Configuration

`pnpm-workspace.yaml`:

```yaml
packages:
  - "apps/*"
  - "apps/games/*"
```

Each app under `apps/` is an independent TypeScript project with its own `package.json` and `tsconfig.json` extending `tsconfig.base.json`. Apps do not depend on each other at the package level. Shared UI components and type definitions, if needed, live in `apps/shared/` as a proper workspace package, but each app must be buildable independently.

### 7.5 Standard Crate Internal Structure

Every crate under `crates/` must follow this internal structure:

```
crates/cortex-<name>/
├── Cargo.toml
├── src/
│   ├── lib.rs                  # Public API surface, re-exports
│   ├── error.rs                # Crate-specific error types
│   ├── types.rs                # Public types for this crate's domain
│   ├── service.rs              # Service trait + implementation (for service crates)
│   └── <module>.rs             # Additional internal modules
├── tests/
│   ├── mod.rs                  # Integration test module root
│   └── <scenario>.rs           # Named integration test scenarios
└── benches/                    # Benchmarks (optional)
    └── <bench_name>.rs
```

---

## 8. Data Model

This spec does not define the data model for any subsystem. Each subsystem spec defines its own tables and schemas. However, this spec defines the conventions that all data models must follow.

### 8.1 Migration File Naming

SQL migrations live in `/crates/cortex-db/migrations/`. Each migration consists of a pair of files following this naming pattern:

```
NNNN_description_in_snake_case.up.sql
NNNN_description_in_snake_case.down.sql
```

Where `NNNN` is a zero-padded four-digit sequence number: `0001`, `0002`, etc.

Examples:
```
0001_create_schema_version.up.sql
0001_create_schema_version.down.sql
0002_create_users_table.up.sql
0002_create_users_table.down.sql
0003_create_sessions_table.up.sql
0003_create_sessions_table.down.sql
0004_add_avatar_column_to_users.up.sql
0004_add_avatar_column_to_users.down.sql
```

### 8.2 Migration Rules

- Migrations are forward-only in normal operation. The `.down.sql` file exists for emergency rollback only.
- Migrations are applied in order by sequence number.
- No two migration files may share the same sequence number.
- Each migration must be idempotent where practical (use `IF NOT EXISTS` for table creation).
- Migrations must not modify or delete data from previous migrations in a way that breaks the down migration.
- All new tables use the following standard columns:
  - `id TEXT PRIMARY KEY` (UUID v7, stored as text)
  - `created_at TEXT NOT NULL` (ISO 8601 UTC timestamp)
  - `updated_at TEXT NOT NULL` (ISO 8601 UTC timestamp)
- Foreign keys use `REFERENCES` with `ON DELETE CASCADE` unless the owning spec states otherwise.

### 8.3 Table Naming

Table names are `snake_case` plural: `users`, `sessions`, `settings`, `files`, `notifications`. Junction tables are `snake_case` with the two entity names joined by underscores: `user_roles`, `app_permissions`.

### 8.4 Dependency Rules

The following dependency graph is authoritative. A crate may only depend on crates listed in its "Allowed Dependencies" column. The `check-deps` tool enforces this at CI time.

| Crate | Allowed Dependencies |
|---|---|
| `cortex-core` | External crates only (serde, uuid, chrono, thiserror, etc.). Zero internal crate dependencies. |
| `cortex-config` | `cortex-core` |
| `cortex-db` | `cortex-core`, `cortex-config` |
| `cortex-observability` | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-sdk` | `cortex-core` |
| `cortex-auth` | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-policy` | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-settings` | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-ai` | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-files` | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-search` | `cortex-core`, `cortex-config`, `cortex-db`, `cortex-files` |
| `cortex-notify` | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-wm` | `cortex-core`, `cortex-config`, `cortex-db` |
| `cortex-runtime` | `cortex-core`, `cortex-config`, `cortex-db`, `cortex-policy`, `cortex-sdk` |
| `cortex-admin` | `cortex-core`, `cortex-config`, `cortex-db`, `cortex-observability` |
| `cortex-api` | ALL crates above (cortex-api is the composition layer) |
| `cortex-server` | `cortex-api`, `cortex-core`, `cortex-config`, `cortex-db`, `cortex-auth`, `cortex-notify`, `cortex-wm` |

**Critical invariant:** No crate may depend on `cortex-api`. The API layer is a top-level composition boundary that wires all services together. No circular dependencies are allowed. The dependency graph forms a DAG with `cortex-core` at the root and `cortex-api` at the top.

**Cross-service communication rule:** When service A (e.g., `cortex-auth`) needs to trigger behavior in service B (e.g., `cortex-notify` to send a notification), this must be done through the `cortex-api` composition layer, never through a direct crate dependency. Services communicate through events and commands mediated by the API layer.

**Permitted cross-service read-only exceptions:** The following cross-service dependencies are explicitly permitted because they represent read-only lookups, not orchestration:
- `cortex-search` depends on `cortex-files` (reads file content for indexing)
- `cortex-runtime` depends on `cortex-policy` (checks permissions before launching apps)
- `cortex-runtime` depends on `cortex-sdk` (uses manifest types for app loading)
- `cortex-admin` depends on `cortex-observability` (reads metrics for diagnostics)

### 8.5 Crate Ownership Map

Each crate owns exactly one domain. Ownership means the crate defines the types, traits, validation logic, and business rules for that domain. Other crates must use the owning crate's public API to interact with that domain.

| Crate | Domain Ownership |
|---|---|
| `cortex-core` | Foundational types, error taxonomy, ID newtypes, shared traits, serialization helpers |
| `cortex-config` | Application configuration loading, environment variable parsing, config validation |
| `cortex-db` | Database connection pool, query execution, migration runner, schema versioning |
| `cortex-api` | HTTP server, WebSocket server, request routing, request/response types for API boundary |
| `cortex-auth` | User identity, authentication, session tokens, password hashing, OAuth flows |
| `cortex-policy` | Permission definitions, policy evaluation, grant storage, enforcement engine |
| `cortex-settings` | Settings namespaces, schema validation, defaults resolution, effective settings computation |
| `cortex-ai` | AI provider abstraction, provider registry, model capability metadata, routing, fallback, budget |
| `cortex-files` | Virtual filesystem, file identity, directory tree, content storage, trash, quotas |
| `cortex-search` | Full-text indexing, search query parsing, result ranking, command palette |
| `cortex-notify` | Notification creation, delivery, read state, notification preferences |
| `cortex-observability` | Structured logging, metrics collection, telemetry, health checks |
| `cortex-runtime` | App lifecycle management, app manifest loading, sandboxing, process isolation |
| `cortex-sdk` | SDK types shared with third-party app developers, manifest schema, capability types |
| `cortex-admin` | Admin diagnostics, system health, recovery operations, database status |

### 8.6 Schema Ownership Rules

Type definitions live in the crate that owns the domain. Cross-crate type references follow these rules:

- **cortex-core** defines all ID types (`UserId`, `SessionId`, `FileId`, `AppId`, `SettingsNamespaceId`, `NotificationId`, etc.) as newtype wrappers around `uuid::Uuid`.
- **cortex-core** defines the shared error taxonomy and foundational traits.
- Domain-specific types (e.g., `UserProfile`, `FileNode`, `AiProviderConfig`) live in their owning crate's `types.rs`.
- **cortex-api** defines request/response types that may compose types from multiple crates, but never redefines them. API types use `From` impls to convert between domain types and API types.
- **cortex-sdk** mirrors a subset of types that are safe for third-party consumption. SDK types are versioned independently.

---

## 9. Public Interfaces

### 9.1 API Route Naming Convention

All HTTP API routes follow the pattern `/api/v1/{domain}/{action}` or `/api/v1/{domain}/{id}/{action}`.

Rules:
- Domain is a lowercase, hyphen-separated noun: `auth`, `files`, `settings`, `ai`, `apps`, `notifications`, `search`, `admin`.
- Action is a lowercase verb or noun: `login`, `logout`, `session`, `profile`, `upload`, `download`, `query`, `invoke`.
- Resource IDs are path parameters: `/api/v1/files/{file_id}`, `/api/v1/apps/{app_id}`.
- Query parameters are `snake_case`: `?page_size=20`, `?sort_by=created_at`.
- Request and response bodies use `camelCase` for JSON keys (TypeScript convention) but `snake_case` for query parameters (HTTP convention).

Illustrative examples (child specs own the authoritative endpoint inventory):

```
POST   /api/v1/auth/login                    # Authenticate user
POST   /api/v1/auth/logout                   # End session
GET    /api/v1/auth/session                  # Get current session info
GET    /api/v1/auth/profile                  # Get user profile
PUT    /api/v1/auth/profile                  # Update user profile
PUT    /api/v1/auth/password                 # Change password

GET    /api/v1/files                         # List files in directory
POST   /api/v1/files/upload                  # Upload file(s)
GET    /api/v1/files/{file_id}               # Get file metadata
GET    /api/v1/files/{file_id}/content       # Download file content
PUT    /api/v1/files/{file_id}               # Update file metadata
DELETE /api/v1/files/{file_id}               # Delete file
POST   /api/v1/files/{file_id}/move          # Move file to new path

GET    /api/v1/settings/{namespace}/{key}    # Get setting by namespaced key
PUT    /api/v1/settings/{namespace}/{key}    # Update setting
DELETE /api/v1/settings/{namespace}/{key}    # Reset setting to default

POST   /api/v1/ai/chat                      # Send chat message to AI
POST   /api/v1/ai/complete                  # Request non-streaming completion
POST   /api/v1/ai/stream                    # Request streaming completion
GET    /api/v1/ai/providers                  # List configured AI providers
GET    /api/v1/ai/models                    # List available AI models

GET    /api/v1/apps                          # List installed apps
POST   /api/v1/apps/{app_id}/launch         # Launch an app
DELETE /api/v1/apps/{app_id}                 # Uninstall an app
GET    /api/v1/apps/{app_id}/manifest        # Get app manifest

GET    /api/v1/notifications                 # List notifications
PUT    /api/v1/notifications/{id}/dismiss    # Dismiss notification
POST   /api/v1/notifications/{id}/action/{action} # Execute action

GET    /api/v1/search?q={query}              # Execute search query

GET    /api/v1/health                        # Health check endpoint
GET    /api/v1/admin/diagnostics             # System diagnostics
POST   /api/v1/admin/backup                  # Trigger database backup
POST   /api/v1/admin/restore                 # Restore from backup
```

### 9.2 WebSocket Event Naming Convention

WebSocket events follow the pattern `domain.action` (dot-separated, lowercase).

Rules:
- Domain matches the API route domain.
- Action is a past-tense verb or state descriptor.
- Client-to-server events use the same pattern (the server distinguishes direction by connection context).

Canonical cross-cutting event names:

```
# File events
file.created
file.modified
file.deleted
file.moved

# App events
app.launched
app.stopped
app.focused
app.minimized
app.maximized
app.restored
app.crashed

# Notification events
notification.created
notification.read
notification.dismissed

# Settings events
settings.changed

# AI events
ai.token                     # Streaming token from AI provider
ai.response.complete
ai.response.error

# Session events
session.expired
auth.logged_out

# System events
system.shutdown
system.startup
```

### 9.3 WebSocket Endpoint

The single WebSocket endpoint is `/ws`. All real-time events flow through this connection. The connection requires an authenticated session established via the same HTTP-only session cookie used by the HTTP API. Query-parameter session tokens are not permitted. See spec 02 for protocol details.

### 9.4 TypeScript Package Naming

Each app under `/apps/` has a `package.json` with a `name` field matching its directory name: `desktop-shell`, `settings-app`, `calculator-app`, etc. Package names use `kebab-case`.

### 9.5 Crate Public API Convention

Every crate exposes its public API through `lib.rs`. The `lib.rs` file must:

1. Re-export all public types from `types.rs`.
2. Re-export the crate's error type and `Result` alias from `error.rs`.
3. Re-export the service trait from `service.rs` (for service crates).
4. Provide a `new()` or `init()` function for service initialization.

Example `lib.rs` for a service crate:

```rust
//! CortexOS authentication service.

pub mod error;
pub mod types;
pub mod service;
pub mod session;
pub mod password;

// Re-exports
pub use types::{UserProfile, SessionToken, LoginRequest, ProfileUpdate};
pub use error::{AuthError, Result};
pub use service::AuthService;

/// Initialize the auth service with the given database pool.
pub async fn init(pool: cortex_db::Pool) -> Result<AuthService> {
    AuthService::new(pool).await
}
```

---

## 10. Internal Interfaces

### 10.1 Crate-to-Crate Trait Interfaces

Each service crate exposes its functionality through a Rust trait defined in the crate's `src/service.rs`. The trait is the contract. The concrete implementation is a struct that satisfies the trait. Traits are named `{Domain}Service`.

Examples:

```rust
// cortex-auth/src/service.rs
pub trait AuthService: Send + Sync {
    async fn login(&self, req: LoginRequest) -> Result<Session, AuthError>;
    async fn logout(&self, session_id: &str) -> Result<(), AuthError>;
    async fn validate_session(&self, token: &str) -> Result<Session, AuthError>;
    async fn get_profile(&self, user_id: &str) -> Result<UserProfile, AuthError>;
    async fn update_profile(&self, user_id: &str, update: ProfileUpdate) -> Result<UserProfile, AuthError>;
    async fn change_password(&self, user_id: &str, current: &str, new: &str) -> Result<(), AuthError>;
}
```

```rust
// cortex-files/src/service.rs
pub trait FilesService: Send + Sync {
    async fn list(&self, path: &str) -> Result<Vec<FileEntry>, FilesError>;
    async fn read(&self, file_id: &str) -> Result<FileContent, FilesError>;
    async fn write(&self, path: &str, content: &[u8]) -> Result<FileEntry, FilesError>;
    async fn delete(&self, file_id: &str) -> Result<(), FilesError>;
    async fn move_file(&self, file_id: &str, dest_path: &str) -> Result<FileEntry, FilesError>;
    async fn get_metadata(&self, file_id: &str) -> Result<FileMetadata, FilesError>;
}
```

```rust
// cortex-settings/src/service.rs
pub trait SettingsService: Send + Sync {
    async fn get(&self, key: &str, user_id: Option<&str>) -> Result<SettingValue, SettingsError>;
    async fn set(&self, key: &str, value: SettingValue, user_id: Option<&str>) -> Result<(), SettingsError>;
    async fn delete(&self, key: &str, user_id: Option<&str>) -> Result<(), SettingsError>;
    async fn list_all(&self, user_id: Option<&str>) -> Result<Vec<SettingEntry>, SettingsError>;
    async fn resolve_effective(&self, key: &str, user_id: &str) -> Result<SettingValue, SettingsError>;
}
```

```rust
// cortex-ai/src/service.rs
pub trait AiService: Send + Sync {
    async fn chat(&self, req: ChatRequest) -> Result<ChatResponse, AiError>;
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, AiError>;
    async fn list_providers(&self) -> Result<Vec<ProviderInfo>, AiError>;
    async fn list_models(&self) -> Result<Vec<ModelInfo>, AiError>;
    async fn stream_chat(&self, req: ChatRequest) -> Result<ChatStream, AiError>;
}
```

### 10.2 Error Types

Each crate defines its own error enum in `src/error.rs`. The error enum implements `std::fmt::Display`, `std::error::Error`, and `serde::Serialize` (for API responses). All error variants use `PascalCase`.

Error enums are named `{Domain}Error`:
- `cortex-auth` defines `AuthError`
- `cortex-files` defines `FilesError`
- `cortex-settings` defines `SettingsError`
- `cortex-ai` defines `AiError`
- `cortex-policy` defines `PolicyError`
- `cortex-search` defines `SearchError`
- `cortex-notify` defines `NotifyError`
- `cortex-runtime` defines `RuntimeError`
- `cortex-admin` defines `AdminError`

The `cortex-api` crate maps domain errors to HTTP status codes using a central `map_error` function. Service crates do not know about HTTP status codes.

Standard error variants that every service crate must include:

```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Validation failed: {0}")]
    Validation(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Internal error: {0}")]
    Internal(String),
    // Domain-specific variants follow...
}
```

### 10.3 Configuration Interface

`cortex-config` exposes a single `AppConfig` struct that is constructed once at startup and passed via `Arc<AppConfig>` to every service that needs it. No service reads configuration from environment variables or files directly; all configuration is funneled through `cortex-config`.

### 10.4 Event Contract Internal Interface

Events are defined as Rust structs in the crate that owns the domain. Each event struct implements `cortex_core::Event` (a marker trait with serialization requirements). Event definitions are in the owning crate's `types.rs`. Event handler registrations and dispatch are wired in `cortex-api`.

---

## 11. State Management

### 11.1 Repository State

- `Cargo.lock` and `pnpm-lock.yaml` are checked into the repository. They must never be added to `.gitignore`. Reproducible builds depend on lock files.
- The Rust toolchain version is pinned in `rust-toolchain.toml` at the repository root:

```toml
[toolchain]
channel = "1.85.0"
components = ["rustfmt", "clippy"]
```

- The Node.js version is pinned in `.node-version` at the repository root: `22.12.0`.
- The pnpm version is pinned in `package.json` under `packageManager`: `"packageManager": "pnpm@9.15.0"`.

### 11.2 Build State

- The `target/` directory (Cargo build artifacts) is gitignored.
- The `node_modules/` directories are gitignored.
- The `dist/` directories in each app are gitignored.
- No build artifacts are committed to the repository.

### 11.3 No Generated Code in Version Control

Generated files (TypeScript API types from Rust, generated documentation) must not be committed. They are regenerated during build and CI. The generation scripts are committed. The `.gitignore` must exclude generated output paths.

Exception: If a generation step is expensive and a conscious decision is made to commit generated output, this must be documented in the crate's `README.md` with the regeneration command.

### 11.4 Runtime State (Developer Machine)

- SQLite database files are stored in a configurable data directory (default: `./data/cortex.db`).
- Large local artifacts that are not modeled as relational state use service-owned directories under `./data/` (for example file blobs, imported packages, crash dumps, and exported diagnostics).
- Structured logs are emitted to stdout and may additionally be mirrored into a local searchable observability store for admin tooling.
- All runtime-state directories are gitignored.

---

## 12. Failure Modes and Error Handling

### 12.1 CI Failure Handling

When CI fails, it fails loudly and completely. There are no "soft" CI failures.

- **Format check failure:** The contributor runs `cargo fmt` or `pnpm format` locally and commits the result. CI does not auto-format.
- **Clippy failure:** The contributor fixes the clippy warning. Warnings are treated as errors in CI (`-D warnings`). During local development, clippy warnings are warnings (not errors) to allow incremental progress.
- **Type check failure:** The contributor fixes TypeScript type errors. CI runs `pnpm typecheck` which uses `tsc --noEmit`.
- **Build failure:** The contributor fixes compilation errors.
- **Test failure:** The contributor fixes the failing test or the code that the test exercises.
- **Manifest validation failure:** The contributor fixes the app manifest JSON to conform to the schema.
- **Dependency graph violation:** The `check-deps` tool reports the exact crate, the forbidden dependency, the allowed dependency list, and a suggested alternative.

### 12.2 Build Failure Recovery

If the build fails due to a dependency resolution error (e.g., crates.io is unavailable), CI retries up to 2 times with a 30-second delay between retries. This is configured in the CI workflow file, not in the codebase.

### 12.3 Dependency Rule Violation

If `check-deps` detects a forbidden dependency, CI fails with a message specifying:
- The crate that has the violation.
- The forbidden dependency.
- The allowed dependency list for that crate.
- Suggested alternatives (e.g., "Move shared types to cortex-core" or "Use event dispatch for cross-service communication at the API layer").

### 12.4 Development Environment Failure

If a new contributor cannot build the project after following the setup instructions, that is a bug in this spec or in the setup tooling. The contributor files an issue and the project maintainer fixes the instructions or the tooling within 24 hours.

---

## 13. Security and Permissions

### 13.1 Secrets in the Repository

No secrets are committed to the repository. This includes:
- API keys (AI provider keys, SSO client secrets)
- Database passwords (even though SQLite is embedded, if Postgres is used via Docker)
- JWT signing keys
- Any file whose name contains `.env`, `.secret`, `.key`, `.pem`, or `.p12`

The `.gitignore` file MUST contain entries for these patterns:
```
.env
.env.*
*.pem
*.p12
*.key
secrets/
```

### 13.2 Dependency Auditing

- CI runs `cargo audit` on every PR to check for known vulnerabilities in Rust dependencies. If `cargo audit` reports a vulnerability with a known fix, the PR is blocked until the dependency is updated.
- CI runs `pnpm audit` on every PR for the same purpose in the TypeScript workspace.
- `cargo-deny` is configured in `deny.toml` to ban known vulnerable crates, enforce license compatibility, and ban duplicate dependency versions where possible. CI runs `cargo deny check` on every PR.

### 13.3 Supply Chain

- All Rust dependencies must come from crates.io. No git dependencies except during initial evaluation (must be replaced before merge).
- All TypeScript dependencies must come from npmjs.com via pnpm. No `file:` or `link:` protocol dependencies except for workspace packages.

### 13.4 Unsafe Code

- No crate may use `unsafe` code without a safety comment explaining the invariant.
- `#![deny(unsafe_code)]` is set at the workspace level.
- Crates that need `unsafe` must explicitly `#![allow(unsafe_code)]` with a documented reason.

### 13.5 Frontend Security

- Frontend apps must never contain hardcoded API keys or secrets.
- All API calls use credentials from the session, never from configuration.
- Content Security Policy headers are set by `cortex-api`.

---

## 14. Performance Requirements

### 14.1 Build Time Budgets

- Full workspace build (clean, from scratch): under 5 minutes on a CI runner with 4 vCPUs and 16 GB RAM.
- Incremental build (one crate changed): under 30 seconds on a developer machine with 8 vCPUs and 32 GB RAM.
- Frontend full build (all apps): under 60 seconds.
- TypeScript type check (all apps): under 15 seconds.
- Full test suite: under 10 minutes.

### 14.2 Binary Size

- The `cortex-api` release binary must be under 50 MB (stripped, optimized for size).
- Individual app bundles (after Vite/esbuild) must each be under 5 MB uncompressed, 500 KB gzipped.

### 14.3 CI Time Budget

- PR feedback on fast-fail stages (format, lint, type check): under 2 minutes.
- The full CI pipeline (all stages, including e2e): under 15 minutes for a typical PR.

---

## 15. Accessibility Requirements

This spec governs tooling and conventions, not user-facing UI. However, the following accessibility-adjacent requirements apply:

- All test output must be machine-readable (TAP, JUnit XML, or JSON) so that CI dashboards and accessibility auditing tools can consume it.
- Commit messages and PR descriptions must be written in plain English, avoiding jargon that would be opaque to contributors using screen readers or translation tools.
- All frontend apps must use semantic HTML elements.
- All interactive elements in frontend apps must be keyboard-navigable by default.
- ESLint accessibility rules (`eslint-plugin-jsx-a11y` or equivalent) must be enabled and pass in CI.

---

## 16. Observability and Logging

### 16.1 Build Observability

CI emits structured logs for each stage. Each stage logs:
- Stage name
- Start time (ISO 8601)
- End time (ISO 8601)
- Duration in seconds
- Exit code
- Key output (last 20 lines on failure)

### 16.2 Local Development Logging

When running `cargo run` locally, the server emits structured logs to stdout in JSON format using the `tracing` crate. The log level is configurable via the `RUST_LOG` environment variable (default: `cortex=info`).

Log format in development (human-readable):
```
2026-03-29T14:30:00.123Z INFO cortex_api::server: Server listening on 0.0.0.0:8080
```

Log format in production (JSON):
```json
{"timestamp":"2026-03-29T14:30:00.123Z","level":"INFO","module":"cortex_api::server","message":"Server listening on 0.0.0.0:8080"}
```

### 16.3 Test Logging

Test output is captured by default. Failed tests print their output. The `--nocapture` flag can be used during local development to see test output in real time. Test output includes pass/fail counts per crate.

---

## 17. Testing Requirements

### 17.1 Test Categories

| Category | Location | Purpose | Naming Convention |
|---|---|---|---|
| Unit tests | `src/**/module.rs` within `#[cfg(test)] mod tests` | Test a single function or small module in isolation | `fn test_<thing>_<scenario>_<expected>()` |
| Integration tests | `/crates/<crate>/tests/*.rs` | Test a crate's public API from the outside | `fn test_<scenario>()` |
| E2E tests | `/tests/e2e/*.rs` | Test the full system: server + browser | `fn test_<user_story>()` |
| TypeScript unit tests | `/apps/<app>/src/**/*.test.ts` | Test frontend components and logic | `describe('Component', () => { it('should ...', ...) })` |

### 17.2 Unit Test Conventions

- Unit tests live in the same file as the code they test, inside a `#[cfg(test)] mod tests` block at the bottom of the file.
- Unit tests may NOT touch the database, the filesystem, or the network. They test pure logic only.
- Unit tests may use mocks for dependencies via trait objects.
- Test function names follow the pattern `<thing>_<scenario>_<expected_result>`.

Examples:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_valid_input_succeeds() { /* ... */ }

    #[test]
    fn password_verify_wrong_password_fails() { /* ... */ }

    #[test]
    fn session_token_expired_returns_unauthorized() { /* ... */ }
}
```

### 17.3 Integration Test Conventions

- Integration tests live in `/crates/<crate>/tests/` and use the crate's public API.
- Integration tests MAY use an in-memory SQLite database (created fresh per test).
- Integration tests are organized one file per domain: `tests/auth_tests.rs`, `tests/files_tests.rs`, etc.
- Integration tests must use real database implementations. Do not mock the database in integration tests.
- External HTTP services (AI providers, OAuth) may be mocked using a mock HTTP server (e.g., `wiremock` or `mockito` crates).
- Never mock the system under test. Mock only its collaborators.

### 17.4 E2E Test Conventions

- E2E tests live in `/tests/e2e/` at the repository root.
- E2E tests start the full server, open a headless browser, and simulate user actions.
- E2E tests use a dedicated test database that is wiped before each test run.
- E2E tests are tagged with `#[ignore]` by default and run with `cargo test --test e2e -- --ignored` or via a dedicated CI stage.
- E2E tests use Playwright for browser automation.

### 17.5 TypeScript Test Conventions

- Tests use Vitest or Jest (decided per app at creation time; the desktop shell uses Vitest).
- Component tests use a virtual DOM (e.g., Testing Library).
- No test makes real HTTP requests to the backend. All backend communication is mocked.

### 17.6 Test Coverage Expectation

- Service crates (`cortex-auth`, `cortex-policy`, etc.) must have at least 80% line coverage for their public APIs as measured by `cargo-llvm-cov`.
- The desktop shell must have at least 70% line coverage for non-trivial components.
- Coverage is measured in CI and reported as a comment on the PR.
- Coverage regression (a PR that decreases coverage by more than 2%) blocks merge.
- Every error variant in a crate's error enum must have at least one test that triggers it.

---

## 18. Acceptance Criteria

This spec is "accepted" when all of the following are true:

1. The repository contains every directory listed in section 7.1, even if some directories are initially empty (with a `.gitkeep` file).
2. The root `Cargo.toml` defines a workspace with all crates as members and the workspace builds successfully with `cargo build`.
3. Every crate in section 7.2 has a `Cargo.toml` with the correct name, version (`0.1.0`), and dependencies matching the table in section 8.4.
4. Every crate has a `src/lib.rs` that exports at minimum the crate's error type and, for service crates, its service trait.
5. `cargo fmt --check` passes on the entire workspace.
6. `cargo clippy --all-targets --all-features -- -D warnings` passes on the entire workspace.
7. `cargo test` passes on the entire workspace (even if tests are trivial initially).
8. The root `package.json` and `pnpm-workspace.yaml` exist and configure the TypeScript workspace.
9. The `docker-compose.yml` exists and documents optional dependencies.
10. `rust-toolchain.toml` exists and pins the Rust version.
11. `.node-version` exists and pins the Node.js version.
12. The `.gitignore` file contains all entries listed in section 13.1 plus `target/`, `node_modules/`, `dist/`, `data/`, and `logs/`.
13. The CI workflow file (`.github/workflows/ci.yml`) implements all stages listed in section 6.4.
14. At least one migration file exists in `/crates/cortex-db/migrations/` (a baseline migration that creates the schema version table).
15. The `tools/validate-manifests.sh` script exists and validates app manifests against the schema.
16. The `tools/check-deps.rs` binary exists and validates the dependency graph.
17. `scripts/dev.sh` starts the backend server and prints its bind address.
18. `scripts/setup.sh` completes successfully on a clean clone.
19. No crate has a circular dependency (run `cargo tree --duplicates` to verify).
20. No secrets in source code or configuration files.

---

## 19. Build Order and Dependencies
**Layer 0**. No dependencies (foundation layer)

### 19.1 Rust Build Order

Cargo determines build order automatically based on the dependency graph. The logical build order from leaves to root is:

```
Layer 0: cortex-core                              (no internal deps)
Layer 1: cortex-config, cortex-sdk                 (depend on cortex-core only)
Layer 2: cortex-db, cortex-observability            (depend on cortex-core + cortex-config)
Layer 3: cortex-auth, cortex-policy, cortex-settings, cortex-ai, cortex-files, cortex-search, cortex-notify, cortex-runtime, cortex-admin
                                                    (depend on cortex-core + cortex-config + cortex-db + permitted cross-service deps)
Layer 4: cortex-api                                (depends on all crates above)
```

### 19.2 Implementation Order for Initial Development

1. `cortex-core` -- foundational types, IDs, error taxonomy, traits
2. `cortex-config` -- configuration loading
3. `cortex-db` -- database pool, migration runner, query helpers
4. `cortex-observability` -- logging infrastructure
5. `cortex-auth` -- authentication, sessions
6. `cortex-policy` -- permission system
7. `cortex-settings` -- settings service
8. `cortex-files` -- virtual filesystem
9. `cortex-ai` -- AI runtime
10. `cortex-runtime` -- app lifecycle, event bus
11. `cortex-search` -- search indexing
12. `cortex-notify` -- notifications
13. `cortex-sdk` -- SDK types
14. `cortex-admin` -- diagnostics
15. `cortex-api` -- API server wiring everything together
16. Frontend apps -- built in parallel after the API layer is stable

### 19.3 TypeScript Build Order

Apps under `/apps/` are independent of each other. Each app builds independently. The desktop shell should be implemented first since it is the primary user-facing surface, followed by the settings app and then utility apps.

### 19.4 External Toolchain Dependencies

| Tool | Version | Purpose | Required |
|---|---|---|---|
| Rust | 1.85.0+ | Backend compilation | Yes |
| Node.js | 22.12.0+ | Frontend tooling | Yes |
| pnpm | 9.15.0+ | Package management | Yes |
| Docker + Docker Compose | Latest stable | Optional dependencies (Postgres, Redis) | No |
| SQLite | 3.40+ (bundled via rusqlite) | Default database | Bundled |
| cargo-audit | Latest | Vulnerability scanning | CI only |
| cargo-deny | Latest | License and ban checks | CI only |
| cargo-llvm-cov | Latest | Coverage measurement | CI only |
| Playwright | Latest | E2E browser automation | CI + E2E dev |

---

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- **Monorepo tooling beyond Cargo and pnpm:** This project does not use Bazel, Nx, Turborepo, or any other monorepo orchestration tool. Cargo and pnpm are sufficient.
- **Custom build scripts:** No `build.rs` files that generate Rust code from external sources. If code generation is needed, it is done as a separate tool run manually and the tool itself is committed.
- **Multi-language backend:** The backend is Rust. There is no Python, Go, or Java component. The frontend is TypeScript. No other languages are used.
- **Cross-compilation in CI:** CI builds for the host platform only (linux/amd64). Cross-compilation for other targets is a future concern.
- **Performance benchmarks in CI:** Benchmark tests are welcome in the codebase but are not run in CI. They are run manually before releases.
- **IDE or editor configuration:** Contributors may use any editor. Editor config files are gitignored.

### 20.2 Anti-Patterns

The following patterns are explicitly forbidden:

1. **Using `unwrap()` in production code.** All `Result` values must be explicitly handled with `?`, `match`, `map_err`, or a custom error conversion. The only exception is test code, where `unwrap()` is permitted.

2. **Committing generated code.** Generated files (build artifacts, lock file changes from `cargo update` that are not dependency upgrades) must not be committed as part of a feature PR. Lock file changes are committed only as part of a dedicated "update dependencies" PR.

3. **Using `pub` on items that should be private.** Every `pub` annotation is a commitment to a stable API. Internal functions should be `pub(crate)` or private. Service traits should be `pub`, but their implementations should be `pub(crate)`.

4. **Glob imports in production code.** `use some_crate::*` is forbidden except in test modules. Production code must enumerate every imported item.

5. **Pinning to a specific commit hash in Cargo.toml.** All dependencies must use semver version specifiers (e.g., `serde = "1.0"`). No git dependencies with `rev = "abc123"`.

6. **Bypassing CI.** No `--no-verify` commits. No force pushes to main. No merging PRs with failing CI.

7. **Adding a dependency on a crate that is already covered by an existing dependency.** If `cortex-auth` needs a type from `cortex-files`, that type should be moved to `cortex-core` rather than adding a dependency edge between the two service crates.

8. **Storing test data in the source tree outside of test fixtures.** Test data (database snapshots, large binary files) belongs in `/tests/fixtures/` and must be gitignored if larger than 1 MB.

9. **Using `expect("message")` in production code.** If a panic is genuinely unreachable, use `unreachable!()` with a comment explaining why. Otherwise, propagate errors. `expect()` is permitted in test code.

10. **Creating a new crate without adding it to this spec.** Every `cortex-*` crate must be listed in section 7.2 with its purpose and dependencies. If a new crate is needed, this spec is updated first (or simultaneously with the crate creation).

11. **Circular crate dependencies.** No two crates may depend on each other, directly or transitively. The `check-deps` tool enforces this.

12. **God crates.** No crate may accumulate responsibilities outside its domain. If a crate needs types from another domain, it imports them, not redefines them.

13. **Shared mutable global state.** No crate may use `lazy_static` or `once_cell` for mutable global state. All state is managed through service instances initialized in `cortex-api`.

14. **Type redefinition.** If a type is defined in crate A, crate B must import it, not define its own version. The only exception is API request/response types in `cortex-api` that wrap domain types.

15. **Direct database access from non-db crates.** Only `cortex-db` executes SQL. All other crates use `cortex-db`'s public API.

16. **Frontend apps importing each other.** Apps are independent. Shared code goes to `apps/shared/`.

17. **Dynamic typing at API boundaries.** All API payloads are statically typed on both Rust and TypeScript sides. No `any` types in TypeScript API code. No `serde_json::Value` in Rust API request/response types except for explicitly extensible fields.

---

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Repository Initialization

When implementing this specification, perform the following steps in order:

1. **Create the directory structure.** Create every directory listed in section 7.1. Place a `.gitkeep` file in each empty directory so Git tracks the directory.

2. **Create the root `Cargo.toml`.** Write the workspace Cargo.toml exactly as specified in section 7.3.

3. **Create each crate's `Cargo.toml`.** For each crate in section 7.2, create `crates/<crate-name>/Cargo.toml` with the correct name, version (inherited from workspace), edition (inherited from workspace), and dependencies (using `workspace = true` for shared deps).

4. **Create each crate's `src/lib.rs`.** Each lib.rs must contain:
   - A doc comment describing the crate's purpose.
   - A `pub mod error;` declaration.
   - A `pub mod types;` declaration.
   - A `pub mod service;` declaration (for service crates only).
   - Re-exports of the service trait and error type at the crate root.

5. **Create each crate's `src/error.rs`.** Define the crate's error enum with at least these variants:
   - `NotFound(String)` -- resource not found
   - `Validation(String)` -- input validation failure
   - `Internal(String)` -- unexpected internal error
   - Additional variants as appropriate for the domain (e.g., `Unauthorized` for `cortex-auth`).

6. **Create each crate's `src/types.rs`.** Define placeholder types for the domain. These are stubs that compile but have no real logic.

7. **Create each crate's `src/service.rs`.** Define the service trait for the crate with stub methods. The implementation struct returns `todo!()` for each method.

8. **Create the root `package.json` and `pnpm-workspace.yaml`:**

```json
{
  "name": "cortexos",
  "private": true,
  "packageManager": "pnpm@9.15.0",
  "scripts": {
    "dev": "pnpm --filter desktop-shell dev",
    "build": "pnpm -r build",
    "test": "pnpm -r test",
    "lint": "pnpm -r lint",
    "typecheck": "pnpm -r typecheck",
    "format": "pnpm -r format",
    "format:check": "pnpm -r format:check"
  }
}
```

```yaml
packages:
  - "apps/*"
  - "apps/games/*"
```

9. **Create `rust-toolchain.toml`:**

```toml
[toolchain]
channel = "1.85.0"
components = ["rustfmt", "clippy"]
```

10. **Create `.node-version`:**
```
22.12.0
```

11. **Create `.gitignore`:**
```
/target/
node_modules/
dist/
.env
.env.*
*.pem
*.p12
*.key
secrets/
data/
logs/
tests/fixtures/*.db
tests/fixtures/*.db-*
.vscode/
.idea/
```

12. **Create `docker-compose.yml`** with commented-out optional services (PostgreSQL, Redis) and documentation about when to use them.

13. **Create the CI workflow file** at `.github/workflows/ci.yml` implementing all stages from section 6.4.

14. **Create the baseline database migration** at `crates/cortex-db/migrations/0001_create_schema_version.up.sql` and its `.down.sql` counterpart. The schema version table is used to track which migrations have been applied.

15. **Create `tools/validate-manifests.sh`** that iterates over `/apps/*/manifest.json` and validates each against a JSON schema.

16. **Create `tools/check-deps.rs`** that parses every `Cargo.toml` in `crates/` and validates that no crate has dependencies outside its allowed list per section 8.4.

17. **Create `scripts/dev.sh`** that starts the backend (`cargo run`) and frontend (`pnpm dev`) concurrently.

18. **Create `scripts/setup.sh`** that checks for Rust, Node.js, and pnpm, installs them if missing, and runs `cargo build` and `pnpm install`.

19. **Create `scripts/migrate.sh`** that runs database migrations.

20. **Create `Makefile`** with convenience targets: `make build`, `make test`, `make lint`, `make dev`, `make setup`.

### 21.2 Validation Checklist

After implementation, verify:

- `cargo build` succeeds from the repository root.
- `cargo test` succeeds (all tests pass, even if they are stubs).
- `cargo fmt --check` passes.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- `pnpm install` succeeds.
- `pnpm typecheck` passes.
- No crate has a circular dependency (run `cargo tree --duplicates` to verify).
- Every crate listed in section 7.2 exists with a valid `Cargo.toml` and `src/lib.rs`.
- `.gitignore` contains all required entries.
- `rust-toolchain.toml` pins the Rust version.
- `.node-version` pins the Node.js version.
- `tools/check-deps.rs` validates the dependency graph correctly.
- `tools/validate-manifests.sh` validates app manifests correctly.

### 21.3 File Creation Priority

Create files in this order to avoid broken intermediate states:

1. Root files: `Cargo.toml`, `package.json`, `pnpm-workspace.yaml`, `rust-toolchain.toml`, `.node-version`, `.gitignore`
2. `cortex-core` (foundation crate, no deps)
3. `cortex-config`, `cortex-sdk` (depend on `cortex-core` only)
4. `cortex-db`, `cortex-observability` (depend on `cortex-core` + `cortex-config`)
5. Remaining service crates (depend on `cortex-core` + `cortex-config` + `cortex-db` + permitted deps)
6. `cortex-api` (leaf crate, depends on all)
7. App scaffolds (TypeScript, independent)
8. CI, Docker, tooling files
9. Scripts

### 21.4 What May Be Stubbed Initially

- `tools/check-api-types` and `tools/generate-api-types` may be stub binaries that exit 0 until API types are defined.
- Frontend apps may be minimal `package.json` + `tsconfig.json` + empty `src/index.ts` files.
- Integration test files may be empty test modules that compile but contain no tests.
- The initial migration (`0001_create_schema_version.up.sql`) may create only the `_migrations` table.

### 21.5 What Must Be Real in v1

- The workspace must compile (`cargo build --workspace` succeeds).
- The dependency graph must be enforced (`check-deps` passes).
- The CI pipeline must run all gates.
- `cargo fmt` and `cargo clippy` must pass.
- `pnpm install` and `pnpm build` must succeed.
- The migration runner in `cortex-db` must apply migrations.
- `scripts/dev.sh` must start the backend server and print its bind address.
- `scripts/setup.sh` must complete on a clean clone.

### 21.6 Stop Conditions

This specification is considered fully implemented when:

1. `cargo build --workspace` succeeds.
2. `cargo test --workspace` succeeds (all tests pass, even if there are zero tests).
3. `cargo clippy --workspace --all-targets -- -D warnings` passes.
4. `cargo fmt --all -- --check` passes.
5. `cargo run --bin check-deps` passes.
6. `pnpm install && pnpm build` succeeds.
7. `pnpm typecheck` passes.
8. Every directory in section 7.1 exists.
9. Every crate has a compilable `lib.rs` with at minimum a placeholder struct or trait.
10. The CI pipeline runs all stages and passes on the initial commit.
11. `scripts/setup.sh` completes successfully on a clean clone.
12. `scripts/dev.sh` starts the backend and frontend successfully.

### 21.7 Naming Quick Reference

When creating new files, types, or identifiers, use this reference:

| What | Convention | Example |
|---|---|---|
| Crate name | `cortex-<domain>` (kebab-case) | `cortex-auth` |
| Rust module file | `snake_case.rs` | `session_manager.rs` |
| Rust struct | `PascalCase` | `SessionToken` |
| Rust enum | `PascalCase` | `AuthError` |
| Rust enum variant | `PascalCase` | `InvalidCredentials` |
| Rust function | `snake_case` | `validate_session()` |
| Rust constant | `SCREAMING_SNAKE_CASE` | `MAX_SESSION_DURATION_SECS` |
| Rust type alias | `PascalCase` | `type UserId = String;` |
| SQL table | `snake_case` plural | `users`, `sessions` |
| SQL column | `snake_case` | `created_at`, `user_id` |
| SQL migration file | `NNNN_description.up.sql` | `0001_create_users_table.up.sql` |
| API route | `/api/v1/<domain>/<action>` | `/api/v1/auth/login` |
| WebSocket event | `domain.action` | `file.created` |
| TypeScript file | `camelCase.tsx` or `kebab-case.tsx` | `taskBar.tsx` or `task-bar.tsx` |
| TypeScript component | `PascalCase` | `TaskBar` |
| TypeScript function | `camelCase` | `handleLogin()` |
| TypeScript constant | `SCREAMING_SNAKE_CASE` or `camelCase` | `API_BASE_URL` |
| Directory name | `kebab-case` | `desktop-shell/` |
| App manifest | `manifest.json` | `/apps/calculator-app/manifest.json` |
| Commit message | `type(scope): description` | `feat(auth): add session refresh endpoint` |

### 21.8 Commit Message Format

Every commit message follows the Conventional Commits format:

```
type(scope): imperative description

Optional body explaining why, not what.
```

Types: `feat`, `fix`, `refactor`, `test`, `docs`, `chore`, `ci`, `perf`, `style`, `build`

Scopes: crate names (`auth`, `files`, `settings`, `api`, etc.) or `workspace` for cross-cutting changes.

Examples:
```
feat(auth): add session refresh endpoint
fix(files): resolve race condition in concurrent writes
test(settings): add integration tests for schema validation
chore(workspace): pin Rust toolchain to 1.85.0
ci: add manifest validation stage
refactor(core): move UserId type from cortex-auth to cortex-core
```
