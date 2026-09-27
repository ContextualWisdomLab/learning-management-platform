# Product-Technical Gap Baseline

## Current evidence

- Default branch baseline: `develop` at `1b89a16bbbd6c4b7c6ee4e8b81e2c8c651d1ce2c`.
- Test-first RED predecessor: `d675b5da872417694454ea8e7fab5e132e7edcc8`.
- Active delivery: [PR #23](https://github.com/ContextualWisdomLab/learning-management-platform/pull/23).
- Architecture source: [Issue #2](https://github.com/ContextualWisdomLab/learning-management-platform/issues/2).
- Commercial journey source: [Issue #3](https://github.com/ContextualWisdomLab/learning-management-platform/issues/3).
- ADR 0001 remains Proposed until exact-head Checks and independent review pass.

## Context Map and boundaries

Learning Management is the canonical writer for affiliation, enrollment,
progress, completion policy, and credential orchestration. Keyverse owns
identity/federation, Orgmetra owns workforce truth, learning-record-store owns
xAPI storage, billing owns monetary truth, and Psychometrics Commons/fast-mlsirm
own assessment/scoring. This repository consumes only immutable released
contracts through ACLs; it never copies source or uses cross-service SQL.

## Gap register

| Gap | Buyer-visible risk | Action and evidence | Status |
|---|---|---|---|
| External learner foundation | Commercial learner relationships fail if every learner must be an employee | PR #23 covers all ten declared affiliation kinds under one employee-only Orgmetra invariant | In review |
| PostgreSQL persistence | No durable tenant, learner, affiliation, enrollment, or policy record | Proposed 3NF ERD; add migration, invariant, concurrency, and rollback tests | Proposed |
| Identity and tenancy | No authentication, SCIM provisioning, or tenant authorization | Consume an immutable Keyverse release through an ACL; fail closed before release | Blocked on contract |
| Learning-event projection | No xAPI/cmi5 projection or replay | Consume a released learning-interoperability-contracts contract and LRS API; no schema copy | Blocked on release |
| Enrollment/completion | No versioned deterministic completion policy | Add a minimal aggregate, replay fixture, and audit event after persistence | Proposed |
| CEFR placement | No placement workflow | Issues #21/#22 remain dependent on immutable interoperability and psychometrics releases | Blocked on releases |
| Partner & Customer Academy UI | No sellable learner/admin journey | Product-owned Figma, tokens, shadcn/ui, Storybook state/locale matrix, E2E, and k6 | Proposed |
| Security and release | No deployable artifact, SBOM, provenance, or rollback proof | Central required Checks plus exact-revision release evidence after a service exists | Proposed |
| Reproducible branch coverage | Stable Rust cannot execute the unstable `cargo-llvm-cov --branch` mode, and permissive guards can accept decoys, skipped jobs/steps, early exits, trailing shell operators, or expression-based failure tolerance | Dependency-free bounded scanner requires exact commands in unconditional gating `jobs.*.steps[]`, treats bare `exit;` as terminating, keeps non-gating state monotonic, accepts only explicit `continue-on-error: false`, pins nightly-2026-05-13, and mutation-tests twenty-one structural, control-flow, and shell boundaries | In review |
| Operability/performance | No service SLO or measured page p95 | Add async service, realistic workload, connection-close proof, and k6 p95 ≤20 ms | Proposed |

## Next safe action

After PR #23 reaches exact-head GREEN and review, implement PostgreSQL
affiliation persistence as a separate Draft PR. Keep UI, LRS, Keyverse,
Orgmetra, billing, and psychometrics integrations behind ports or feature flags
until their immutable owner releases are verified.
