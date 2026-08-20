# Product Requirements Document

**As of:** 2026-08-20  
**Status:** bounded external-learner vertical; open PR evidence is not merged product evidence.

## Product promise

A tenant can serve a learner who is not an employee: entitlement, enrollment,
learning evidence, assessment outcome, completion, credential reference, and
correction remain traceable without manufacturing an HR record.

The buyer-visible correction behavior is explicit: when a published completion
decision is wrong, a client can submit `supersedes_decision_id`. The service
validates that the predecessor belongs to the same tenant, learner, and
registration, creates a new immutable decision, and records the correction in
the audit stream.

## Primary users and jobs

| User | Job to be done | Acceptance signal |
|---|---|---|
| Tenant learning administrator | Operate an external learner journey without an employee row | Real PostgreSQL/API path reaches completion and credential reference |
| Learner | Receive a reproducible completion result from owned evidence and an assessment authority | Passed assessment and required evidence produce `completed` |
| Auditor/operator | Explain what changed without exposing provider payloads | Tenant-scoped audit export contains provenance, digest, correlation, and correction action |
| Integrator | Retry external assessment delivery safely | Registration-bound idempotency returns the same evidence reference |

## Functional requirements

1. Keep identity, content, learning evidence, assessment, billing, and
   credential authority at their owning-system boundaries.
2. Enforce tenant and learner boundaries in the API transaction and PostgreSQL
   RLS; cross-tenant composite foreign keys fail closed.
3. Evaluate immutable completion policy revisions in the Rust kernel and accept
   assessment evidence only when the external result is `passed`.
4. Persist a correction as a new decision. Never rewrite the predecessor,
   evidence, or audit event.
5. Require correlation IDs to be valid UUIDs when supplied and preserve them in
   audited writes.
6. Export bounded provenance metadata only; source assessment, credential, and
   provider payloads must not be copied into the export.

## Non-functional requirements

- New security- and integrity-critical logic remains in Rust.
- Database objects use two-or-more-word `snake_case` names and normalized
  relations; append-heavy audit access has a tenant/time index.
- RLS is enabled and forced on tenant-owned tables; audit mutation is rejected.
- Verification must use pinned Rust checks, real PostgreSQL/API evidence, and
  exact current-head CI before protected merge.
- No production partitioning, retention purge, backup/restore, certification,
  browser UI, or released provider client is claimed by this slice.

## Traceability

| Requirement | Design/implementation | Evidence |
|---|---|---|
| External learner boundary | `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md` | PR #4–#12 stacked API and migration work |
| Immutable correction | ADR-0003, `create_completion_decision` | PR #17 exact head and real PostgreSQL/API run |
| Provenance export | ADR-0002, `audit_event_record` | PR #13–#16 and quality smoke path |
| Standards and research | `docs/doctoring/STANDARD_TRACEABILITY.md` | APA 7 source map; conformance remains unclaimed |

The current gap register and open-PR ledger remain authoritative in
`docs/product-technical-gap-baseline.md`.
