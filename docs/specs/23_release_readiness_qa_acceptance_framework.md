# Spec 23: Release Readiness, QA, and Acceptance Framework

## 1. Purpose

> **Status note:** This document defines acceptance criteria and release gates for CortexOS. References to browser compatibility verification, permission validation, AI safety validation, and other "verified" claims describe the criteria that must be met before release -- they are not claims that these criteria currently pass. No gate should be treated as passing without test evidence from CI or manual QA.

This specification defines the release readiness criteria, quality assurance strategy, test coverage requirements, acceptance framework, release process, and compatibility requirements for CortexOS. It establishes the gates that must be passed before any release candidate is promoted to a stable release and provides the QA infrastructure that all other subsystem specs reference.

## 2. Scope

This spec covers:

- Release readiness criteria across all subsystems
- QA strategy: unit, integration, and end-to-end testing tiers
- Minimum test coverage targets per crate
- Definition of done at the feature, subsystem, and release levels
- Branch strategy, versioning, changelog, and release notes
- Smoke test suite required before any release
- Performance benchmark targets and measurement methodology
- Browser compatibility requirements
- Accessibility audit gates
- Security review gates for release

## 3. Out of Scope

- Individual subsystem test case designs (those belong in each subsystem spec)
- CI/CD pipeline implementation details (referenced here but defined in spec 01)
- Third-party app QA processes (covered in spec 21)
- Game-specific testing beyond standard app testing
- Load testing at scale beyond single-user browser session
- Localization or internationalization testing (not in v1 scope)

## 4. Objectives

1. Every CortexOS release must pass deterministic, repeatable quality gates before promotion.
2. Quality criteria must be machine-checkable where possible.
3. No release ships with P0 bugs. No release ships with unresolved P1 bugs unless explicitly waived with documented justification.
4. Performance targets are explicit and measurable.
5. Browser compatibility is verified against a defined matrix.
6. The QA framework must be executable by both human QA engineers and automated CI pipelines.
7. All acceptance criteria must be binary (pass/fail) with no subjective judgments.

## 5. User-Visible Behavior

### 5.1 Release Quality Promise

Users of CortexOS can expect:

- No data loss during normal operation or upgrade.
- No unrecoverable state after a crash.
- All first-party apps launch and perform basic operations within defined benchmarks.
- AI features fail gracefully when no provider is configured; they never corrupt state.
- The desktop shell renders and is interactive within the startup time target.
- Settings changes take effect without requiring a full restart.

### 5.2 Version Display

- The About panel in Settings displays: version number (semver), build hash, build date.
- The login screen displays the CortexOS version in the bottom-left corner.

### 5.3 Release Notes

- On first login after an upgrade, a dismissible release notes dialog is shown.
- Release notes are generated from the changelog and filtered for user-visible changes.

## 6. System Behavior

### 6.1 Release Readiness Gates

A release candidate must pass ALL of the following gates in order:

```
Gate 1: Code Quality Gate
  - cargo clippy --all-targets --all-features -- -D warnings passes with zero warnings
  - cargo fmt --all -- --check passes (zero formatting violations)
  - No TODO/FIXME/HACK comments in production code that are not tracked as issues
  - All public APIs have rustdoc comments

Gate 2: Test Gate
  - cargo test --workspace passes (zero test failures)
  - Every crate meets minimum coverage target (see section 8)
  - Integration test suite passes (zero failures)
  - End-to-end test suite for critical user flows passes (zero failures)
  - Smoke test suite passes (zero failures)

Gate 3: Performance Gate
  - Startup time benchmark within target (see section 14)
  - AI request latency benchmark within target
  - UI responsiveness benchmark within target
  - No performance regressions greater than 15% from previous release

Gate 4: Accessibility Gate
  - Keyboard navigation works for all first-party apps
  - Screen reader announces all interactive elements correctly
  - Color contrast ratios meet WCAG 2.1 AA (4.5:1 for normal text, 3:1 for large text)
  - Focus indicators are visible on all interactive elements
  - All images have alt text or aria-labels

Gate 5: Security Gate
  - cargo audit reports zero known vulnerabilities in dependencies
  - No secrets or credentials in source code
  - All API endpoints enforce authentication where required
  - All user inputs are validated at server boundaries
  - AI action audit trail is functional and queryable

Gate 6: Compatibility Gate
  - All smoke tests pass on all target browsers (see section 6.7)

Gate 7: Documentation Gate
  - All public APIs documented
  - Changelog generated and reviewed
  - Release notes drafted
  - Known issues documented

Gate 8: Bug Triage Gate
  - Zero P0 bugs open
  - Zero P1 bugs open (or each P1 has an approved waiver)
  - All P2 bugs triaged and assigned to a release
```

### 6.2 Bug Severity Definitions

| Severity | Definition | Release Blocking |
|----------|-----------|-----------------|
| P0 | Data loss, security vulnerability, system unrecoverable crash | Yes, always |
| P1 | Feature broken in normal use, workaround does not exist | Yes, unless waived |
| P2 | Feature degraded, workaround exists | No, tracked |
| P3 | Cosmetic issue, minor inconvenience | No, tracked |

### 6.3 Waiver Process

- P1 waivers require: bug ID, justification, risk assessment, approving engineer name, and expiry (maximum one subsequent release).
- Waivers are recorded in the release manifest.
- Waivers are listed in release notes under "Known Issues."

### 6.4 QA Strategy Tiers

#### Tier 1: Unit Tests (per crate)

- Every Rust crate must have a corresponding test module.
- Unit tests cover: public API contracts, internal invariants, error paths, boundary conditions.
- Tests run via `cargo test --workspace`.
- Target: execute in under 60 seconds total for the full workspace.

#### Tier 2: Integration Tests

- Integration tests live in `tests/integration/` at the workspace root.
- Each integration test exercises a cross-crate boundary.
- Mandatory integration test categories:
  - Settings service writes -> AI runtime reads (preferred_provider routing)
  - File service creates -> Search service indexes -> Search returns result
  - Auth service authenticates -> Policy service enforces -> App runtime launches
  - AI runtime routes request -> Provider adapter executes -> Response returns
  - Notification service sends -> Desktop shell renders
  - Window manager opens -> App runtime lifecycle -> Window manager closes
- Integration tests may use in-memory databases and mock provider adapters.
- Target: execute in under 5 minutes total.

#### Tier 3: End-to-End Tests

- E2E tests exercise complete user flows through the browser UI.
- E2E tests use a headless browser automation framework (Playwright or equivalent).
- Mandatory E2E test scenarios:
  - E2E-001: User logs in, desktop shell renders, all first-party apps visible in launcher
  - E2E-002: User opens Settings, changes preferred_provider, change persists across page reload
  - E2E-003: User opens File Manager, creates a folder, creates a file, renames file, deletes file
  - E2E-004: User opens Text Editor, types text, saves file, reopens file, content preserved
  - E2E-005: User opens Calculator, performs calculation, result is correct
  - E2E-006: User triggers AI assistant with no provider configured, sees graceful failure message
  - E2E-007: User triggers AI assistant with provider configured, receives response
  - E2E-008: User opens and closes five apps, memory usage does not exceed baseline by more than 50%
  - E2E-009: User uses keyboard only to navigate between three apps
  - E2E-010: User triggers notification, notification appears, user dismisses notification
- Target: execute in under 15 minutes total.

### 6.5 Smoke Tests

The smoke test suite is a minimal subset of tests that MUST pass before any release artifact is promoted:

```
SMOKE-001: cargo build --release succeeds
SMOKE-002: cargo test --workspace succeeds (zero failures)
SMOKE-003: Desktop shell loads in headless browser within 5 seconds
SMOKE-004: Settings app opens and renders all sections
SMOKE-005: File Manager opens and displays root directory
SMOKE-006: AI assistant panel opens (provider not required)
SMOKE-007: User login/logout cycle completes without error
SMOKE-008: No JavaScript console errors on desktop shell load
SMOKE-009: No Rust panic in logs during 60-second idle session
SMOKE-010: cargo clippy passes with zero warnings
```

Smoke tests must execute in under 3 minutes total. Any smoke test failure is an automatic release block.

### 6.6 Branch Strategy

```
main          - stable, always releasable, protected branch
  |
  +-- develop - integration branch for current release cycle
  |     |
  |     +-- feature/NNNN-description - individual feature branches
  |     +-- fix/NNNN-description     - bug fix branches
  |
  +-- release/vX.Y.Z - release candidate branches, branched from develop
  |
  +-- hotfix/vX.Y.Z  - emergency fixes, branched from main
```

Rules:
- `main` is always in a releasable state. Any commit on `main` must pass all gates.
- Feature branches merge into `develop` via pull request requiring one approval and passing CI.
- Release branches are created from `develop` when feature freeze is declared.
- Only bug fixes merge into release branches.
- Hotfix branches merge into both `main` and `develop`.
- No direct commits to `main` or `develop`. All changes via pull request.

### 6.7 Versioning

CortexOS follows Semantic Versioning 2.0.0 (semver):

```
MAJOR.MINOR.PATCH

MAJOR: breaking changes to the app manifest schema, SDK API, or settings schema
MINOR: new features, new first-party apps, new subsystems
PATCH: bug fixes, performance improvements, accessibility fixes
```

Pre-release identifiers:
- `alpha.N`: early development builds, no stability guarantee
- `beta.N`: feature-complete builds, may have known issues
- `rc.N`: release candidates, must pass all gates

Build metadata:
- Build metadata is appended after a plus sign: `v1.0.0+abc1234.20260329`
- Build metadata includes: git commit short hash, build date

### 6.8 Changelog Generation

- Changelogs are auto-generated from conventional commit messages.
- Commit message format:
  ```
  type(scope): description

  [optional body]

  [optional footer with breaking change note]
  ```
- Types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `chore`, `ci`
- Breaking changes must include `BREAKING CHANGE:` in the footer.
- The changelog is generated using `git log` parsing at release time.
- Changelog sections: Breaking Changes, Features, Bug Fixes, Performance, Other.

### 6.9 Browser Compatibility Matrix

| Browser | Minimum Version | Test Priority |
|---------|----------------|--------------|
| Google Chrome | 110+ | P0 (must pass) |
| Mozilla Firefox | 115+ | P0 (must pass) |
| Apple Safari | 16.4+ | P0 (must pass) |
| Microsoft Edge | 110+ | P0 (must pass) |

All four browsers must pass the full smoke test suite. Chrome and Firefox must pass the full E2E test suite. Safari and Edge must pass smoke tests plus E2E-001 through E2E-005.

Required web platform features:
- WebAssembly (MVP)
- ES2022 (top-level await, class fields, private methods)
- CSS Grid and Flexbox
- IndexedDB
- Web Workers
- ResizeObserver
- IntersectionObserver
- Clipboard API (for clipboard access permission)
- File API / Blob API

## 7. Architecture

### 7.1 QA Infrastructure Components

```
+-------------------------------------------+
|           CI Pipeline (spec 01)            |
|  +---------+  +----------+  +----------+  |
|  | Lint &  |  |  Unit &  |  | Security |  |
|  | Format  |  | IntTest  |  |  Audit   |  |
|  +---------+  +----------+  +----------+  |
|       |            |              |        |
|       v            v              v        |
|  +--------------------------------------+  |
|  |         Gate Decision Engine         |  |
|  |  - Evaluates all gates              |  |
|  |  - Produces pass/fail report        |  |
|  |  - Blocks merge on failure          |  |
|  +--------------------------------------+  |
+-------------------------------------------+
                    |
                    v
+-------------------------------------------+
|         E2E Test Runner                   |
|  +-------------+  +-------------------+   |
|  | Headless    |  | Browser Matrix    |   |
|  | Browser     |  | Chrome/Firefox/   |   |
|  | Automation  |  | Safari/Edge       |   |
|  +-------------+  +-------------------+   |
+-------------------------------------------+
                    |
                    v
+-------------------------------------------+
|         Performance Benchmark Runner      |
|  - Startup time measurement              |
|  - AI latency measurement                |
|  - UI responsiveness measurement         |
|  - Comparison against previous release    |
+-------------------------------------------+
```

### 7.2 Test File Organization

```
cortexos/
  crates/
    cortex-core/       src/, tests/
    cortex-config/     src/, tests/
    cortex-db/         src/, tests/
    cortex-api/        src/, tests/
    cortex-auth/       src/, tests/
    cortex-policy/     src/, tests/
    cortex-settings/   src/, tests/
    cortex-ai/         src/, tests/
    cortex-files/      src/, tests/
    cortex-search/     src/, tests/
    cortex-notify/     src/, tests/
    cortex-observability/ src/, tests/
    cortex-runtime/    src/, tests/
    cortex-sdk/        src/, tests/
    cortex-admin/      src/, tests/
  tests/
    integration/       (cross-crate integration tests)
    e2e/               (end-to-end browser tests)
    smoke/             (smoke test suite)
    benchmarks/        (performance benchmarks)
  apps/
    desktop-shell/     src/, tests/
    settings-app/      src/, tests/
    ... (each app has its own tests/)
```

## 8. Data Model

### 8.1 Test Coverage Targets

| Crate / Area | Minimum Line Coverage | Notes |
|-------------|----------------------|-------|
| cortex-core | 80% | Foundation crate, highest bar |
| cortex-auth | 85% | Security-sensitive |
| cortex-policy | 85% | Security-sensitive |
| cortex-settings | 80% | Settings correctness is critical |
| cortex-ai | 80% | AI routing must be deterministic |
| cortex-files | 80% | Data integrity |
| cortex-db | 75% | Storage layer |
| cortex-api | 75% | API surface |
| cortex-config | 80% | Configuration parsing |
| cortex-search | 70% | Search quality hard to cover with unit tests |
| cortex-notify | 70% | UI-dependent |
| cortex-observability | 70% | Logging infra |
| cortex-runtime | 75% | App lifecycle |
| cortex-sdk | 80% | Public API contract |
| cortex-admin | 70% | Admin tools |
| Total Workspace | 75% | Aggregate minimum |

Coverage is measured by `cargo-llvm-cov` or equivalent. Coverage is computed per crate and reported in CI. A PR that reduces coverage below the target for its affected crate is blocked.

### 8.2 Release Manifest

```rust
struct ReleaseManifest {
    version: SemVer,
    build_hash: String,
    build_date: DateTime<Utc>,
    release_date: Option<DateTime<Utc>>,
    release_type: ReleaseType,
    gates_passed: Vec<GateResult>,
    known_issues: Vec<KnownIssue>,
    waivers: Vec<P1Waiver>,
    breaking_changes: Vec<String>,
    changelog: String,
}

enum ReleaseType {
    Alpha { number: u32 },
    Beta { number: u32 },
    ReleaseCandidate { number: u32 },
    Stable,
}

struct GateResult {
    gate_name: String,
    passed: bool,
    timestamp: DateTime<Utc>,
    details: String,
}

struct KnownIssue {
    bug_id: String,
    severity: BugSeverity,
    description: String,
    workaround: Option<String>,
}

struct P1Waiver {
    bug_id: String,
    justification: String,
    risk_assessment: String,
    approved_by: String,
    expires_after_version: SemVer,
}
```

### 8.3 Performance Benchmark Data Model

```rust
struct PerformanceBenchmark {
    benchmark_id: String,
    target: BenchmarkTarget,
    measured_value: f64,
    unit: BenchmarkUnit,
    threshold: f64,
    passed: bool,
}

enum BenchmarkTarget {
    StartupTime,
    AIRequestLatency { provider: String, model: String },
    UIResponsiveness { action: String },
    MemoryUsageBaseline,
    FileOperationLatency { operation: String },
}

enum BenchmarkUnit {
    Milliseconds,
    Megabytes,
    FramesPerSecond,
}
```

## 9. Public Interfaces

### 9.1 QA CLI Commands

```bash
# Run all quality gates
cargo xtask qa --all-gates

# Run specific gate
cargo xtask qa --gate smoke
cargo xtask qa --gate unit
cargo xtask qa --gate integration
cargo xtask qa --gate e2e
cargo xtask qa --gate benchmarks
cargo xtask qa --gate compatibility

# Generate coverage report
cargo xtask qa --coverage

# Generate release manifest
cargo xtask release --manifest --version X.Y.Z

# Generate changelog since last release
cargo xtask release --changelog --since v0.9.0

# Validate release readiness
cargo xtask release --validate --version X.Y.Z
```

### 9.2 CI Integration Points

- On every pull request: Gate 1 (code quality) + Gate 2 (unit tests only)
- On merge to `develop`: Gate 1 + Gate 2 (full, including integration)
- On release branch creation: All eight gates
- Nightly on `develop`: All gates including E2E and performance benchmarks

## 10. Internal Interfaces

### 10.1 Gate Evaluation Engine

The gate evaluation engine is an internal tool that:

1. Runs each gate's checks sequentially.
2. Records pass/fail for each check.
3. Produces a structured report (JSON + human-readable).
4. Returns non-zero exit code if any gate fails.
5. Publishes results to the observability system (spec 14).

### 10.2 Benchmark Runner

The benchmark runner:

1. Executes each benchmark three times.
2. Reports median value.
3. Compares against the defined threshold.
4. Compares against previous release's benchmark data.
5. Flags regressions exceeding 15%.

## 11. State Management

### 11.1 Release State Machine

```
Draft -> FeatureFreeze -> CodeComplete -> ReleaseCandidate -> GatesPassed -> Released
  |           |                |                  |                |
  v           v                v                  v                v
(new        (only bug       (all tests         (RC branch       (tagged,
features    fixes)          must pass)         created)         published)
allowed)
```

### 11.2 Coverage State

- Coverage data is stored per commit in CI artifacts.
- Coverage trends are tracked across releases.
- Coverage reports are published to a dashboard accessible to all contributors.

## 12. Failure Modes and Error Handling

### 12.1 Gate Failure

When a gate fails:
1. The gate evaluation engine stops at the first failure.
2. A detailed failure report is generated with the exact check that failed, expected value, and actual value.
3. The CI pipeline marks the build as failed.
4. The merge is blocked.
5. The failure is logged to the observability system.

### 12.2 Flaky Test Handling

- A test that fails non-deterministically is marked as flaky.
- Flaky tests are tracked in a flaky test registry.
- A flaky test must be fixed or quarantined within 5 business days.
- Quarantined tests are excluded from gate evaluation but still run and reported.
- A release cannot ship with more than 5 quarantined tests.

### 12.3 Benchmark Variance

- If a benchmark shows >15% regression from the previous release, the gate fails.
- If the regression is determined to be environmental (CI resource contention), the benchmark may be re-run up to 3 times.
- If the regression persists, the gate fails and the regression must be investigated.

### 12.4 Browser Compatibility Failure

- If a smoke test fails on a specific browser, that browser is marked as unsupported for the release.
- A release must support at minimum 3 of the 4 target browsers.
- If fewer than 3 browsers pass, the release is blocked.

## 13. Security and Permissions

### 13.1 Release Integrity

- Release artifacts are checksummed (SHA-256).
- Checksums are published alongside the release.
- Git tags are signed.
- The release manifest is immutable once published.

### 13.2 Security Gate Details

The security gate (Gate 5) performs:

1. Dependency audit: `cargo audit` runs against all dependencies.
2. Secret scanning: The repository is scanned for accidentally committed secrets.
3. Permission model validation: All permission checks are verified to be server-side.
4. AI safety validation: AI action audit trail is verified functional.
5. Input validation audit: Spot-check of input validation at all API boundaries.

## 14. Performance Requirements

### 14.1 Startup Time Targets

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Cold start (first load, no cache) | <= 3000ms | Time from page load event to desktop shell fully rendered |
| Warm start (cached) | <= 1500ms | Time from page load event to desktop shell fully rendered |
| App launch (from click to interactive) | <= 500ms | Time from app icon click to app window interactive |
| Settings app open | <= 400ms | Time from click to settings rendered |

### 14.2 AI Request Latency Targets

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| AI assistant panel open | <= 200ms | Time from trigger to panel visible |
| AI request dispatch (after user submits) | <= 100ms | Time from submit to request leaving the client |
| AI request total (excludes provider time) | <= 500ms overhead | CortexOS overhead above raw provider latency |
| Fallback chain (per hop) | <= 300ms | Time to detect failure and dispatch to next provider |
| AI response streaming first token | <= provider_latency + 200ms | Time from submit to first streamed token visible |

Note: Provider latency (the time the external AI provider takes to process a request) is outside CortexOS control and is explicitly excluded from these targets. The targets measure only CortexOS overhead.

### 14.3 UI Responsiveness Targets

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Window drag latency | <= 16ms per frame (60fps) | RequestAnimationFrame timing during drag |
| Window resize latency | <= 16ms per frame (60fps) | RequestAnimationFrame timing during resize |
| Menu open/close animation | <= 150ms total | Time from trigger to animation complete |
| Notification appear | <= 100ms | Time from notification event to visual render |
| File list scroll (1000 items) | <= 16ms per frame | Scroll performance with large lists |
| Theme switch | <= 300ms | Time from theme change to full re-render |

### 14.4 Memory Targets

| Metric | Target |
|--------|--------|
| Desktop shell baseline (no apps open) | <= 80MB |
| Desktop shell + 5 apps open | <= 250MB |
| Desktop shell + 10 apps open | <= 400MB |

Memory targets are measured as browser tab memory usage via `performance.measureUserAgentSpecificMemory()` or equivalent.

## 15. Accessibility Requirements

### 15.1 Accessibility Audit Gate

The accessibility gate (Gate 4) verifies:

1. **Keyboard Navigation**: Every interactive element in every first-party app is reachable via keyboard (Tab, Arrow keys, Enter, Escape).
2. **Screen Reader**: All first-party apps expose correct ARIA roles, labels, and states. Tested with at least one screen reader (VoiceOver on Safari or NVDA on Firefox).
3. **Color Contrast**: All text meets WCAG 2.1 AA contrast ratios. Verified by automated contrast checking tool.
4. **Focus Management**: Focus indicators are visible on all interactive elements. Focus moves correctly when dialogs open/close. Focus is trapped within modal dialogs.
5. **Semantic HTML**: All first-party apps use semantic HTML elements (button, nav, main, aside, dialog, etc.).
6. **Alt Text**: All meaningful images have descriptive alt text. Decorative images have empty alt attributes.

### 15.2 Accessibility Test Automation

- Automated contrast checking runs as part of Gate 4 in CI.
- ARIA validation runs via axe-core or equivalent integrated into E2E tests.
- Manual screen reader testing is performed once per release cycle by a human tester.

## 16. Observability and Logging

### 16.1 QA Observability

All gate evaluations, test runs, and benchmark results are published to the observability system (spec 14) with:

- Timestamp
- Gate/test/benchmark identifier
- Pass/fail result
- Duration
- Details (failure reason, actual vs expected values)

### 16.2 Release Metrics

The following metrics are tracked across releases:

- Total test count and pass rate
- Coverage percentage per crate and aggregate
- Benchmark values (trend over time)
- Number of open bugs by severity
- Number of waivers per release
- Time from feature freeze to release

## 17. Testing Requirements

### 17.1 Testing the QA Framework Itself

The gate evaluation engine must have its own tests:

- Unit tests for each gate's evaluation logic.
- Integration test: run all gates against a known-good build, verify all pass.
- Integration test: run all gates against a deliberately broken build, verify correct gate fails.
- Test that coverage measurement tooling works correctly.
- Test that changelog generation produces correct output.

### 17.2 Test Isolation Rules

- Unit tests must not depend on external services (network, filesystem outside temp dirs).
- Integration tests may use in-memory databases but must not depend on external services.
- E2E tests may use mock AI providers but must not depend on real external API keys.
- All tests must be idempotent and runnable in any order.
- All tests must clean up their state after execution.

## 18. Acceptance Criteria

### 18.1 Per-Release Acceptance Checklist

- [ ] All eight gates pass
- [ ] Release manifest generated and reviewed
- [ ] Changelog reviewed by at least one engineer
- [ ] Release notes drafted
- [ ] Known issues documented
- [ ] All P1 waivers approved and documented
- [ ] Performance benchmarks compared against previous release
- [ ] At least 3 of 4 target browsers pass full smoke tests
- [ ] Accessibility audit passed (automated + manual)
- [ ] Security audit passed
- [ ] Release tagged in git
- [ ] Release artifacts published with checksums

### 18.2 Per-Feature Acceptance Checklist

- [ ] Implementation matches spec behavior
- [ ] Unit tests written and passing
- [ ] Integration test written for cross-boundary behavior (if applicable)
- [ ] Error handling covers all defined failure modes
- [ ] Accessibility: keyboard navigable, screen reader compatible
- [ ] Performance: no regression beyond 10% for affected benchmarks
- [ ] Security: no new attack surface without explicit review
- [ ] Observability: appropriate logging added
- [ ] Code review approved

### 18.3 Per-Subsystem Acceptance Checklist

- [ ] All spec sections implemented
- [ ] All out-of-scope items explicitly not implemented
- [ ] All public interfaces match spec
- [ ] All failure modes handled
- [ ] All security requirements met
- [ ] Coverage target met
- [ ] Integration tests with dependent subsystems passing
- [ ] Documentation complete

## 19. Build Order and Dependencies
**Layer 16**. Depends on: all specs (final gate before release)

### 19.1 QA Framework Build Order

The QA framework depends on all other subsystems being buildable. Its own build order within the release process:

1. All crates compile (`cargo build --workspace`)
2. Code quality gate runs (clippy, fmt)
3. Unit tests run (`cargo test --workspace`)
4. Integration tests run
5. E2E tests run (requires running server)
6. Performance benchmarks run
7. Security audit runs
8. Accessibility audit runs
9. Gate evaluation engine produces final report
10. Release manifest generated

### 19.2 Dependencies on Other Specs

- Spec 01 (Repository Conventions): CI pipeline configuration, repository structure
- Spec 14 (Observability): QA result logging and metrics
- Spec 22 (Admin/Diagnostics): Diagnostic data for troubleshooting test failures
- All subsystem specs: Their individual testing requirements feed into this framework

## 20. Non-Goals and Anti-Patterns

### 20.1 Non-Goals

- Load testing beyond single-user browser sessions (multi-user is not in v1 scope)
- Fuzz testing (valuable but not required for v1)
- Chaos engineering (valuable but not required for v1)
- Automated visual regression testing (valuable but not required for v1)
- Cross-device testing (mobile, tablet) for v1
- Performance testing on low-end hardware (v1 targets modern browsers on modern hardware)

### 20.2 Anti-Patterns

- **Releasing with skipped tests**: Every skipped test must have an associated issue and must be fixed before the next release.
- **Manual gate overrides**: Gates must not be manually overridden without a documented waiver.
- **Subjective quality judgments**: All acceptance criteria must be binary pass/fail.
- **Testing only the happy path**: Every subsystem must test error paths and failure modes.
- **Ignoring coverage declines**: A PR that reduces coverage is blocked unless the reduction is justified and documented.
- **Testing only on one browser**: All four target browsers must be verified.
- **Snapshot-based test approvals without review**: Snapshot tests must be reviewed by a human when updated.

## 21. Implementation Instructions for Claude Code / Codex

### 21.1 Subsystem Ownership

- This spec owns: release gates, QA strategy, coverage targets, benchmark definitions, acceptance checklists, release process.
- This spec does NOT own: individual test implementations (those belong to each subsystem), CI pipeline implementation (spec 01), individual subsystem performance (each subsystem spec).

### 21.2 Recommended Implementation Order

1. Create the `xtask` binary crate for QA and release commands.
2. Implement Gate 1 (code quality): clippy and fmt wrappers.
3. Implement Gate 2 (tests): cargo test runner with coverage reporting.
4. Implement smoke test suite (10 tests).
5. Implement integration test infrastructure.
6. Implement E2E test infrastructure (Playwright setup).
7. Implement performance benchmark runner.
8. Implement gate evaluation engine.
9. Implement release manifest generator.
10. Implement changelog generator.

### 21.3 What Can Be Stubbed Initially

- Performance benchmarks can use placeholder thresholds initially, to be calibrated after the first working build.
- Browser compatibility testing can be limited to Chrome during early development.
- Accessibility manual testing can be deferred until feature freeze.

### 21.4 What Must Be Real in v1

- All 10 smoke tests must be real, passing tests.
- Coverage reporting must be real and enforced.
- Gate evaluation must be automated and enforced in CI.
- Changelog generation must produce real output.
- Performance benchmarks must have calibrated targets.

### 21.5 What Cannot Be Inferred

- The exact coverage percentages are deliberate choices; do not adjust them without updating this spec.
- The exact benchmark targets are deliberate choices; do not adjust them without updating this spec.
- The browser version minimums are deliberate choices; do not lower them without updating this spec.
- The P0/P1 blocking rules are non-negotiable.

### 21.6 Stop Conditions

This subsystem is done when:

- [ ] `cargo xtask qa --all-gates` runs successfully against a known-good build
- [ ] `cargo xtask qa --all-gates` correctly fails against a deliberately broken build
- [ ] Coverage reporting produces per-crate numbers
- [ ] All 10 smoke tests exist and pass
- [ ] At least 5 integration tests exist and pass
- [ ] At least 5 E2E tests exist and pass (on Chrome)
- [ ] Benchmark runner produces results for all defined benchmark categories
- [ ] Release manifest generator produces valid JSON output
- [ ] Changelog generator produces correctly formatted output
- [ ] All tests in this subsystem's own test suite pass
