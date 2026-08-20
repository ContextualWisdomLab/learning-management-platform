# Changelog

## Unreleased

### Added

- Initial LMS authority and integration boundaries.
- First-class learner and enrollment data-model baseline covering both employee-linked and non-employee journeys.
- Standards adoption and operating-profile traceability.
- Product and technical gap baseline with the first buyer journey, modular target, delivery order, and APA 7 source map.
- Repository development rules.
- Rust learner domain kernel, tenant-scoped PostgreSQL migration, and learner-registration API smoke path.
- Bounded offering, external entitlement projection, enrollment, and learning registration API path on the stacked kernel branch.
- Effective-dated learner affiliation API coverage with PostgreSQL exclusion-conflict mapping on the next stack.
- Added launch attempts and LRS-owned progress projections with out-of-order observation protection.
- Added policy revision, external evidence reference, and Rust-evaluated completion decision persistence.
- Added tenant-safe credential reference issuance from a completed registration and exact completion decision.
- Added idempotent credential reference revocation with tenant and decision-boundary checks.
- Added versioned assessment-result reference handoff with external outcome status and passed-only completion evaluation.
- Added registration-bound evidence references and idempotent assessment import retries.
- Added tenant-isolated append-only audit events with correlation, source version, and event digest for assessment import, completion publication, and credential lifecycle transitions.
- Added optional `X-Correlation-ID` UUID propagation for audited assessment, completion, and credential operations, with generated IDs when callers omit the header.
- Added tenant-scoped, bounded audit-event export without source payload fields.
- Added PostgreSQL operational evidence baseline for forced RLS, audit export indexing, and explicit retention/rollback boundaries.
