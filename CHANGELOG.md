# Changelog

## Unreleased

- Clarify the bootstrap evidence metadata allowlist, registration-to-decision cardinality, and exact repository validation check for PR #1.
- Remove self-staling mutable PR-head evidence from the durable product/technical gap baseline; exact current integration evidence is resolved live from GitHub.
- Make the completion-policy revision authority explicit: the tenant-scoped `completion_policy_revision` foreign key is authoritative and any metadata mirror must match it or fail closed.

### Added

- Added a fail-closed learner-affiliation invariant: employees require a nonblank opaque Orgmetra worker reference and all nine non-employee affiliation kinds reject one.
- Initial LMS authority and integration boundaries.
- First-class learner and enrollment data-model baseline covering both employee-linked and non-employee journeys.
- Standards adoption and operating-profile traceability.
- Product and technical gap baseline with the first buyer journey, modular target, delivery order, and evidence-authority rules.
- Repository development rules.
- Rust learner domain kernel, tenant-scoped PostgreSQL migration, and learner-registration API smoke path.
- Bounded offering, external entitlement projection, enrollment, and learning registration API path on the stacked kernel branch.
- Effective-dated learner affiliation API coverage with PostgreSQL exclusion-conflict mapping on the next stack.
- Employee-only opaque Orgmetra worker-reference persistence and bearer-authorized affiliation API fixtures, with non-employee rejection at both API and database boundaries.
- Added non-superuser/NOBYPASSRLS CI application-role verification, customer/self-sponsored multi-tenant affiliation coverage, and disposable migration rollback/reapply rehearsal.
- Added fail-closed bearer-key-to-tenant authorization, separate migration/application database roles, non-owner application-role assertions, forced-RLS isolation tests, and a committed Rust 1.97.1 dependency lock.
- Product-first README for buyers, maintainers, and integrators.
- Apache License 2.0 source grant for the repository foundation.

### Fixed

- Validate affiliation and policy deserialization, bind plaintext HTTP to loopback only, hash replay-relevant evidence metadata, and enforce tenant-and-learner decision/evidence foreign keys with real PostgreSQL constraint fixtures.
- Commit the Rust 1.97.1 dependency lock and stop resolving mutable dependency versions inside CI.
- Restrict initial learner registration to `active` membership so a first registration cannot create an `ended` or `suspended` membership that has no transition path back to active.
- Permit a closed enrollment registration to be followed by a new registration while rejecting replay of the same tenant-scoped external reference.
- Enforce tenant-and-learner ownership across entitlement, enrollment, registration, and completion foreign keys.
- Pin the PostgreSQL 18 Alpine CI service by immutable multi-architecture digest and stop piping local HTTP responses into an interpreter; store and parse bounded response files instead.
