# Appendix C: Definition of Done

## Purpose

This appendix defines the authoritative "definition of done" (DoD) for CortexOS at three levels: per-feature, per-subsystem, and per-release. Every piece of work in CortexOS must satisfy the applicable level of DoD before it is considered complete. These criteria are binary (pass/fail) with no subjective judgments.

## Scope

This DoD applies to all work on CortexOS: Rust backend crates, TypeScript frontend code, first-party apps, games, and infrastructure/tooling.

---

## C.1. Per-Feature Definition of Done

A feature is a discrete unit of user-visible or system-visible functionality tracked as a single issue or pull request.

A feature is **done** when ALL of the following criteria are met:

### C.1.1. Implementation Completeness

- [ ] The implementation matches the behavior specified in the relevant spec section(s).
- [ ] All specified error paths are handled with defined error messages.
- [ ] All specified edge cases are implemented.
- [ ] No TODO, FIXME, HACK, or XXX comments remain in the code (they must be resolved or converted to tracked issues).
- [ ] No stubbed or placeholder behavior remains for the feature's core logic.
- [ ] No commented-out code remains in the implementation.

### C.1.2. Code Quality

- [ ] `cargo clippy --all-targets --all-features -- -D warnings` produces zero warnings for the changed files.
- [ ] `cargo fmt --all -- --check` produces zero formatting violations.
- [ ] No compiler warnings for the changed files.
- [ ] All public functions, traits, structs, and enums have rustdoc comments.
- [ ] No unnecessary `unwrap()` or `expect()` calls in production code paths (use proper error handling).
- [ ] No hardcoded secrets, API keys, or credentials.
- [ ] No `println!` or `dbg!` macros in production code (use the observability system).

### C.1.3. Testing

- [ ] Unit tests cover the feature's public API contracts.
- [ ] Unit tests cover the feature's error paths.
- [ ] At least one integration test covers the feature's interaction with dependent subsystems (if the feature crosses crate boundaries).
- [ ] All new tests pass.
- [ ] No existing tests are broken by the change.
- [ ] Test coverage for the affected crate does not decrease below its minimum target (see spec 23, section 8.1).

### C.1.4. Security

- [ ] User input is validated at the server boundary (not only on the client).
- [ ] No new attack surface is introduced without an explicit security review note in the PR.
- [ ] Permission checks are enforced server-side (not only on the client).
- [ ] No new dependencies with known vulnerabilities (cargo audit passes).

### C.1.5. Accessibility

- [ ] All new interactive elements are keyboard-navigable.
- [ ] All new interactive elements have appropriate ARIA labels or roles.
- [ ] Color contrast ratios for new UI elements meet WCAG 2.1 AA.
- [ ] Focus management is correct (especially for dialogs and modals).

### C.1.6. Observability

- [ ] Appropriate log statements added for: initialization, errors, and significant state changes.
- [ ] Log levels are correct: ERROR for failures, WARN for degraded behavior, INFO for significant events, DEBUG for diagnostics.
- [ ] No sensitive data (PII, secrets, full request/response bodies) is logged at INFO or above.

### C.1.7. Documentation

- [ ] Public API documentation is updated.
- [ ] If the feature introduces a new setting, it is documented in the relevant spec and appendix A.
- [ ] If the feature changes existing behavior, the spec is updated to match.

### C.1.8. Review

- [ ] Code review approved by at least one other contributor.
- [ ] All review comments addressed (either resolved or deferred with a tracked issue).

---

## C.2. Per-Subsystem Definition of Done

A subsystem is a major component defined by its own spec file (specs 01-22). A subsystem is **done** when ALL of the following criteria are met, in addition to all per-feature criteria for every feature within the subsystem.

### C.2.1. Spec Compliance

- [ ] Every section of the subsystem spec has been implemented.
- [ ] Every "Out of Scope" item in the spec is explicitly NOT implemented (no scope creep).
- [ ] Every public interface defined in the spec exists and matches the spec's signature.
- [ ] Every data model defined in the spec is implemented with the specified fields and types.
- [ ] Every failure mode defined in the spec is handled.
- [ ] Every invariant defined in the spec is enforced (via assertions, type system, or runtime checks).

### C.2.2. Testing

- [ ] Unit test coverage meets or exceeds the minimum target for the crate (see spec 23, section 8.1).
- [ ] Integration tests exist for every cross-crate dependency listed in the spec.
- [ ] All integration tests pass.
- [ ] Every error path defined in the spec has at least one test.
- [ ] Every state transition defined in the spec has at least one test.
- [ ] Tests are deterministic and pass consistently (no flaky tests).

### C.2.3. Security and Permissions

- [ ] All permission checks defined in the spec are enforced server-side.
- [ ] No client-side authorization as source of truth.
- [ ] All inputs at subsystem boundaries are validated.
- [ ] Security-sensitive operations are logged in the audit trail (where applicable).
- [ ] The subsystem does not expose more data than the spec permits.

### C.2.4. Performance

- [ ] The subsystem meets all performance requirements defined in its spec.
- [ ] No unbounded memory growth under normal operation.
- [ ] No blocking operations on the main thread (async where appropriate).

### C.2.5. Error Handling

- [ ] Every error type defined in the spec is implemented.
- [ ] Error messages are user-appropriate (no internal jargon leaked to UI).
- [ ] Errors are propagated correctly to dependent subsystems.
- [ ] Recovery behavior matches the spec.

### C.2.6. Integration

- [ ] All dependent subsystems can successfully interact with this subsystem.
- [ ] All public APIs are stable and documented.
- [ ] No hidden coupling with other subsystems (all dependencies are explicit).
- [ ] The subsystem works correctly when dependent subsystems are unavailable (degraded mode as defined in spec).

### C.2.7. Documentation

- [ ] All public APIs have complete rustdoc.
- [ ] The crate-level documentation explains the subsystem's purpose, scope, and ownership.
- [ ] Any deviations from the spec are documented with the reason for the deviation.

---

## C.3. Per-Release Definition of Done

A release is a versioned distribution of CortexOS. A release is **done** when ALL of the following criteria are met, in addition to all per-subsystem criteria for every subsystem included in the release.

### C.3.1. Code Quality Gates

- [ ] `cargo clippy --all-targets --all-features -- -D warnings` passes with zero warnings across the entire workspace.
- [ ] `cargo fmt --all -- --check` passes with zero violations across the entire workspace.
- [ ] `cargo build --release` succeeds with zero warnings.
- [ ] No compiler warnings across the entire workspace.

### C.3.2. Test Gates

- [ ] `cargo test --workspace` passes with zero failures.
- [ ] All integration tests pass (zero failures).
- [ ] All E2E tests for critical user flows pass (zero failures).
- [ ] All smoke tests pass (zero failures).
- [ ] No quarantined (flaky) tests exceed the maximum of 5.
- [ ] Every crate meets its minimum test coverage target.

### C.3.3. Performance Gates

- [ ] Cold startup time <= 3000ms (measured per spec 23).
- [ ] Warm startup time <= 1500ms.
- [ ] App launch time <= 500ms.
- [ ] AI request overhead <= 500ms (excluding provider latency).
- [ ] Window drag/resize >= 60fps (16ms per frame).
- [ ] Desktop shell baseline memory <= 80MB.
- [ ] No performance regression > 15% from previous release on any benchmark.

### C.3.4. Security Gates

- [ ] `cargo audit` reports zero known vulnerabilities in dependencies.
- [ ] No secrets or credentials in source code (automated scan passes).
- [ ] All API endpoints requiring authentication enforce it.
- [ ] All user inputs validated at server boundaries.
- [ ] AI action audit trail is functional and queryable.
- [ ] No client-side authorization as source of truth anywhere in the codebase.

### C.3.5. Accessibility Gates

- [ ] All first-party apps are fully keyboard-navigable.
- [ ] All first-party apps have correct ARIA roles and labels.
- [ ] Color contrast ratios meet WCAG 2.1 AA across all first-party apps.
- [ ] Focus indicators visible on all interactive elements.
- [ ] Screen reader testing completed for at least one browser/screen reader combination.
- [ ] Modal dialogs trap focus correctly.

### C.3.6. Documentation Gates

- [ ] All public Rust APIs have rustdoc comments.
- [ ] Changelog generated from conventional commits and reviewed.
- [ ] Release notes drafted covering all user-visible changes.
- [ ] Known issues documented.
- [ ] Migration notes written for any breaking changes.

### C.3.7. Compatibility Gates

- [ ] Smoke tests pass on Google Chrome 110+.
- [ ] Smoke tests pass on Mozilla Firefox 115+.
- [ ] Smoke tests pass on Apple Safari 16.4+.
- [ ] Smoke tests pass on Microsoft Edge 110+.
- [ ] E2E tests pass on at least Chrome and Firefox.

### C.3.8. Bug Triage

- [ ] Zero P0 bugs open (no exceptions).
- [ ] Zero P1 bugs open (or each P1 has a documented, approved waiver).
- [ ] All P2 bugs triaged and assigned to a release or backlog.

### C.3.9. Release Artifacts

- [ ] Git tag created with version number.
- [ ] Release artifacts built and published.
- [ ] SHA-256 checksums generated and published alongside artifacts.
- [ ] Release manifest generated with all gate results.

---

## C.4. Code Quality Rules (Apply at All Levels)

These rules are invariant and apply to every pull request, feature, subsystem, and release.

### C.4.1. Formatting

- `cargo fmt` is the authoritative formatter. No custom formatting rules.
- All code must pass `cargo fmt --all -- --check` with zero violations.

### C.4.2. Linting

- `cargo clippy --all-targets --all-features -- -D warnings` is the authoritative linter.
- All code must pass with zero warnings.
- Clippy configuration must not be relaxed to suppress warnings. Fix the code, not the linter.

### C.4.3. Warnings

- Zero compiler warnings in `cargo build`.
- Zero compiler warnings in `cargo test`.
- Unused import warnings, dead code warnings, and unreachable pattern warnings must be fixed, not suppressed with `#[allow(...)]`.

### C.4.4. Dependencies

- Every dependency must be declared with a minimum version in `Cargo.toml`.
- Every dependency must pass `cargo audit`.
- No duplicate dependencies that can be avoided.
- Workspace-level dependency management (shared versions where possible).

---

## C.5. Test Gates (Apply at All Levels)

### C.5.1. Unit Tests

- Every Rust crate has a `tests` module (either inline or in a `tests/` directory).
- Unit tests cover: public API contracts, error paths, boundary conditions.
- Unit tests must be deterministic: same input always produces same result.
- Unit tests must be isolated: no dependency on external services, network, or filesystem outside temp directories.
- Unit tests must be idempotent: runnable in any order, any number of times.

### C.5.2. Integration Tests

- Cross-crate boundaries must be covered by integration tests.
- Integration tests live in `tests/integration/`.
- Integration tests may use in-memory databases.
- Integration tests must not depend on external services.
- Integration tests must clean up after themselves.

### C.5.3. E2E Tests

- Critical user flows must be covered by E2E tests.
- E2E tests run against a fully running CortexOS instance.
- E2E tests use headless browser automation.
- E2E tests must not depend on external AI API keys (use mock providers).

---

## C.6. Checklist Template

### Feature Checklist (Copy for each feature PR)

```
Feature: [feature name]
Spec: [spec number and section]

Implementation:
[ ] Matches spec behavior
[ ] All error paths handled
[ ] All edge cases implemented
[ ] No TODO/FIXME/HACK comments

Code Quality:
[ ] clippy passes
[ ] fmt passes
[ ] no compiler warnings
[ ] rustdoc on public APIs
[ ] no unwrap/expect in production paths
[ ] no hardcoded secrets

Testing:
[ ] unit tests for public API
[ ] unit tests for error paths
[ ] integration test (if crosses crate boundary)
[ ] all tests pass
[ ] coverage not decreased

Security:
[ ] input validated at server boundary
[ ] permissions enforced server-side
[ ] no new vulnerabilities

Accessibility:
[ ] keyboard navigable
[ ] ARIA labels/roles
[ ] color contrast AA

Observability:
[ ] appropriate logging
[ ] correct log levels
[ ] no sensitive data in logs

Review:
[ ] code review approved
[ ] review comments resolved
```

### Subsystem Checklist (Copy for each subsystem completion review)

```
Subsystem: [subsystem name]
Spec: [spec number]

Spec Compliance:
[ ] all sections implemented
[ ] out-of-scope items not implemented
[ ] public interfaces match spec
[ ] data models match spec
[ ] failure modes handled
[ ] invariants enforced

Testing:
[ ] coverage target met
[ ] integration tests for all cross-crate dependencies
[ ] error paths tested
[ ] state transitions tested
[ ] no flaky tests

Security:
[ ] permissions server-side
[ ] inputs validated
[ ] audit trail functional

Performance:
[ ] meets performance requirements
[ ] no unbounded memory growth
[ ] no blocking on main thread

Integration:
[ ] dependent subsystems can interact
[ ] APIs stable and documented
[ ] no hidden coupling
[ ] degraded mode works

Documentation:
[ ] all public APIs documented
[ ] crate-level docs complete
```

### Release Checklist (Copy for each release)

```
Release: vX.Y.Z
Date: YYYY-MM-DD

Code Quality:
[ ] clippy zero warnings
[ ] fmt zero violations
[ ] build zero warnings

Tests:
[ ] unit tests pass
[ ] integration tests pass
[ ] e2e tests pass
[ ] smoke tests pass
[ ] coverage targets met
[ ] no more than 5 quarantined tests

Performance:
[ ] cold start <= 3000ms
[ ] warm start <= 1500ms
[ ] app launch <= 500ms
[ ] AI overhead <= 500ms
[ ] window operations >= 60fps
[ ] memory baseline <= 80MB
[ ] no regression > 15%

Security:
[ ] cargo audit clean
[ ] no secrets in code
[ ] auth enforced
[ ] inputs validated
[ ] audit trail functional

Accessibility:
[ ] keyboard navigation
[ ] ARIA roles/labels
[ ] color contrast AA
[ ] focus indicators
[ ] screen reader tested

Documentation:
[ ] rustdoc complete
[ ] changelog generated
[ ] release notes drafted
[ ] known issues documented

Compatibility:
[ ] Chrome passes
[ ] Firefox passes
[ ] Safari passes
[ ] Edge passes

Bug Triage:
[ ] zero P0
[ ] zero P1 (or all waived)
[ ] all P2 triaged

Artifacts:
[ ] tag created
[ ] artifacts published
[ ] checksums generated
[ ] manifest generated
```

---

## C.7. Waiver and Exception Process

### When a Criterion Cannot Be Met

1. The contributor files an exception request as a comment on the PR or release issue.
2. The exception request must include: which criterion is not met, why it cannot be met, the risk of proceeding without it, and the plan to resolve it.
3. For feature-level exceptions: the PR reviewer approves or rejects.
4. For release-level exceptions: the release lead approves or rejects.
5. Approved exceptions are documented in the release manifest.

### What Cannot Be Waived

The following criteria cannot be waived under any circumstances:

- Zero P0 bugs
- `cargo clippy` passes
- `cargo fmt` passes
- No secrets in source code
- Server-side permission enforcement
- No client-side authorization as source of truth

### P1 Waiver Process

P1 waivers follow the process defined in spec 23, section 6.3:
- Bug ID, justification, risk assessment, approving engineer, expiry (maximum one subsequent release).
- Recorded in the release manifest.
- Listed in release notes under "Known Issues."
