# Changelog

## Unreleased

- Clarify the bootstrap evidence metadata allowlist, registration-to-decision cardinality, and exact repository validation check for PR #1.

### Added

- Added a fail-closed learner-affiliation invariant: employees require a nonblank opaque Orgmetra worker reference and all nine non-employee affiliation kinds reject one.
- Initial LMS authority and integration boundaries.
- First-class learner and enrollment data-model baseline covering both employee-linked and non-employee journeys.
- Standards adoption and operating-profile traceability.
- Product and technical gap baseline with the first buyer journey, modular target, delivery order, and APA 7 source map.
- Repository development rules.
- Rust learner domain kernel, tenant-scoped PostgreSQL migration, and learner-registration API smoke path.
- Bounded offering, external entitlement projection, enrollment, and learning registration API path on the stacked kernel branch.
- Added non-superuser/NOBYPASSRLS CI application-role verification, customer/self-sponsored multi-tenant affiliation coverage, and disposable migration rollback/reapply rehearsal.
- Added fail-closed bearer-key-to-tenant authorization, separate migration/application database roles, non-owner application-role assertions, forced-RLS isolation tests, and a committed Rust 1.97.1 dependency lock.

### Fixed

- Commit the Rust 1.97.1 dependency lock and stop resolving mutable dependency versions inside CI.
- Restrict initial learner registration to `active` membership so a first registration cannot create an `ended` or `suspended` membership that has no transition path back to active.
