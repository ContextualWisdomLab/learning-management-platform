# ADR-0002: PostgreSQL operational evidence boundary

**Status:** Accepted for the current bounded kernel  
**Date:** 2026-08-20

## Context

The learner-registration kernel depends on tenant isolation, append-heavy evidence/audit projections, and a forward-only embedded migration. A table or policy definition alone is not evidence that the running database has the required security and export shape. Premature partitioning or an untested destructive rollback would add operational risk.

## Decision

1. Keep PostgreSQL RLS and tenant-scoped foreign keys as defense-in-depth; the application transaction must still set `app.tenant_id` and enforce resource ownership.
2. Keep `audit_event_record` append-only and use the `(tenant_id, occurred_at, audit_event_record_id)` index as the initial export/ingestion access path.
3. Require quality checks to inspect enabled/forced RLS, policy count, the audit export index, audit event count, correlation propagation, cross-tenant isolation, and mutation rejection on the exact migration.
4. Use a disposable backup/restore rehearsal with measured tenant skew as the
   minimum recovery evidence, while deferring time/hash partitioning, retention
   deletion, production recovery certification, and rollback claims until
   production-shaped load evidence and governed cutover procedures exist.

## Consequences

The current service has executable tenant/RLS/audit and disposable recovery
evidence without pretending that retention, production recovery, or
certification is complete. The append-only rule means future retention must use
an explicitly governed archive boundary or a forward migration; it must not
silently delete decision history.

## Verification

See `docs/operational/POSTGRES_READINESS.md`,
`scripts/postgres_recovery_rehearsal.sh`,
`migrations/0001_learning_kernel.sql`,
`migrations/0002_audit_correlation_index.sql`, and the `rust-kernel` PostgreSQL
smoke test in `.github/workflows/quality.yml`.
