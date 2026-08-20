# PostgreSQL operational readiness baseline

**As of:** 2026-08-20  
**Status:** executable evidence baseline; not a production-readiness or certification claim.

The current migration is a forward-only embedded migration. The application role receives a transaction-local `app.tenant_id`, and tenant-owned relations use PostgreSQL row-level security as defense in depth. `audit_event_record` is append-only, protected by a mutation-rejecting trigger, and indexed by tenant and occurrence time for bounded export.

## Current evidence

The quality workflow checks the live migration catalog for:

- 16 public tenant policies;
- 16 relations with row-level security enabled;
- 16 relations with forced row-level security;
- the `audit_event_record_tenant_occurred_idx` export index;
- five audit events, including a completion correction, and their caller correlation IDs in the API smoke path;
- audit mutation rejection and cross-tenant export isolation.

The local verification also applies `migrations/0001_learning_kernel.sql` to a fresh PostgreSQL 18.4 database and confirms the audit export index and 16 policies.

## Known operational boundary

- No production partitioning is claimed. Tenant/time indexing is the measured first step; partitioning requires append-rate, tenant-skew, vacuum, and query-plan evidence.
- No retention purge is implemented. Audit rows are append-only, so archive and retention ownership must be designed before deletion or partition detach is allowed.
- No automated backup/restore or migration rollback drill is claimed. The migration is forward-only; rollback evidence requires a disposable restore rehearsal and an explicit forward repair migration.
- No CSAP, SOC 2, or NIST certification claim is made by these checks. Control mapping and incident receipts remain open work.

The next operational slice is a disposable backup/restore and forward-migration rehearsal with measured audit volume and tenant skew.
