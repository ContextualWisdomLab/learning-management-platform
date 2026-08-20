# PostgreSQL operational readiness baseline

**As of:** 2026-08-20  
**Status:** executable evidence baseline; not a production-readiness or certification claim.

The current migration is a forward-only embedded migration. The application role receives a transaction-local `app.tenant_id`, and tenant-owned relations use PostgreSQL row-level security as defense in depth. `audit_event_record` is append-only, protected by a mutation-rejecting trigger, and indexed by tenant and occurrence time for bounded/keyset export.

## Current evidence

The quality workflow checks the live migration catalog for:

- 16 public tenant policies;
- 16 relations with row-level security enabled;
- 16 relations with forced row-level security;
- the `audit_event_record_tenant_occurred_idx` export index;
- the `audit_event_record_tenant_correlation_idx` incident-correlation index;
- five audit events, including a completion correction, and their caller correlation IDs in the API smoke path;
- audit mutation rejection, cross-tenant export isolation, and keyset pagination.

The local verification also applies `migrations/0001_learning_kernel.sql` to a fresh PostgreSQL 18.4 database and confirms the audit export index and 16 policies.

The disposable recovery rehearsal (`scripts/postgres_recovery_rehearsal.sh`) was
run locally against PostgreSQL 18.4 on 2026-08-20. It inserted 1,000
anonymized audit events across two non-empty tenants (900/100, skew ratio
9.00), completed the custom-format dump and restore in under one second,
restored both tenant distributions unchanged, preserved 16 forced-RLS
relations, and applied the forward `0002_audit_correlation_index.sql`
migration. The databases and rows were disposable rehearsal data.

A separate real PostgreSQL/API smoke run on the same date returned a first page
of two events, a cursor page of the remaining three in stable order, HTTP 400
for a partial cursor, and an empty result for a second tenant. Its database,
tenant references, and audit rows were disposable and anonymized.

## Known operational boundary

- No production partitioning is claimed. Tenant/time indexing is the measured first step; partitioning requires append-rate, tenant-skew, vacuum, and query-plan evidence.
- No retention purge is implemented. Audit rows are append-only, so archive and retention ownership must be designed before deletion or partition detach is allowed.
- The disposable backup/restore and forward-migration rehearsal is now
  executable evidence, not production recovery certification. The migration
  remains forward-only; rollback evidence still requires a governed restore
  cutover and an explicit rollback decision.
- The correlation index is a normal forward migration. A production rollout
  must choose an online/index-lock strategy appropriate to the observed table
  size; this rehearsal does not make that decision.
- No CSAP, SOC 2, or NIST certification claim is made by these checks. Control mapping and incident receipts remain open work.

The next operational slice is measured load, vacuum, query-plan, and tenant-skew
evidence at production-shaped volume, followed by an explicit retention and
rollback runbook.
