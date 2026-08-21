# Technical Requirements Document

**As of:** 2026-08-20  
**Status:** executable bounded kernel; production-readiness claims remain explicitly separated.

## Runtime boundary

The Rust crate `lms_kernel` owns domain validation and deterministic completion
evaluation. `crates/lms_kernel/src/bin/lms_api.rs` is a thin Axum adapter. The
embedded migration is the PostgreSQL schema authority. Identity, content,
activity, assessment scoring, billing, and credential payloads remain external
contracts; this service stores opaque references and source digests.

## Completion correction transaction

For `POST /v1/tenants/{tenant_id}/learners/{learner_id}/registrations/{learning_registration_id}/completion-decisions`:

1. Parse the optional `supersedes_decision_id` and the optional correlation
   header at the trust boundary.
2. Begin a tenant-scoped transaction and set `app.tenant_id`.
3. If a predecessor is supplied, require it to match the tenant, learner, and
   registration; otherwise return HTTP 400 without a new decision or audit row.
4. Load the exact policy revision and registration-bound evidence, then call the
   Rust completion evaluator.
5. Insert a new `completion_decision` row with the predecessor relation. The
   predecessor remains immutable.
6. Record `completion_decision.published` or
   `completion_decision.corrected` in the same transaction, then commit.

The response returns `completion_decision_id`, `supersedes_decision_id`, policy
revision, evaluation time, replay fingerprint, and completion status. A
correction receives a new decision UUID even when its evidence and fingerprint
match the predecessor.

## Data and security controls

- Tenant-scoped composite foreign keys prevent mixed-tenant learner,
  registration, evidence, decision, and credential references.
- PostgreSQL RLS is enabled and forced on all tenant-owned relations.
- `audit_event_record` is append-only, digest-bound, and protected by a
  mutation-rejecting trigger.
- API responses expose opaque UUID references and audit provenance, not source
  payloads or actor secrets.
- A missing correlation header generates a UUID; a malformed UUID is HTTP 400.
- Audit export accepts `limit=1..1000`; larger histories use the paired
  `after_occurred_at` and `after_audit_event_record_id` keyset cursor. A partial
  cursor is HTTP 400. Each response retains its JSON array contract and adds
  count, ordered page receipt-digest, and last-event cursor headers; the receipt
  digest does not contain source payloads.

## Verification contract

The exact-head workflow must verify the migration catalog, valid and invalid
correction requests, predecessor linkage, decision count, correlation IDs,
corrected audit action, bounded/keyset export, receipt headers and digest
recomputation, payload exclusion, RLS isolation, and append-only mutation
rejection. Local verification additionally uses a
PostgreSQL 18.4 database and a `NOSUPERUSER` application role.

Operational gaps are deliberate: production-shaped partitioning and recovery,
retention ownership, rollback rehearsal, provider consumer contracts, browser
E2E, and release certification are separate follow-up slices. A disposable
backup/restore and forward-migration rehearsal is executable evidence, not a
production recovery claim.
