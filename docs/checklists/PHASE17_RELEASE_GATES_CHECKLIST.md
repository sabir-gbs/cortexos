# Phase 17 Release Gates Checklist

Goal:

- convert the implemented system into a release-verifiable product with measurable gates

Use with:

- [23_release_readiness_qa_acceptance_framework.md](/home/sabir/projects/cortexos/docs/specs/23_release_readiness_qa_acceptance_framework.md)
- [appendix_c_definition_of_done.md](/home/sabir/projects/cortexos/docs/specs/appendix_c_definition_of_done.md)

## Scope Of This PR

In scope:

- E2E gates
- coverage enforcement
- CI hardening
- release validation workflow

Out of scope:

- undocumented waivers for failing gates

## PR Success Criteria

- release readiness is measurable
- all workspace gates are enforceable in CI
- critical flows have E2E coverage

## Work Order

- [ ] enforce workspace fmt, lint, build, test, and typecheck gates
- [ ] add/finish E2E harness for critical flows
- [ ] enforce coverage thresholds
- [ ] represent accessibility, security, and observability checks in release validation
- [ ] document release validation outputs and failure criteria

## Validation Checklist

- [ ] release claims are backed by passing gates
- [ ] no required gate exists only as prose

## Stop Conditions

- stop if a release gate is being waived without explicit documented approval
