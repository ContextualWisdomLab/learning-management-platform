# ADR-0004: Stable tenant audit-export pagination

**Status:** Accepted for the bounded audit API
**Date:** 2026-08-20

## Context

The audit export is tenant-scoped and bounded to 1,000 rows per response. A
limit-only API cannot retrieve a large tenant's complete history without
restarting from the beginning, and an offset would become unstable as new
append-only events arrive.

## Decision

Keep the existing JSON array response and add an optional keyset cursor:

- `after_occurred_at` is the last returned event's `occurred_at` timestamp;
- `after_audit_event_record_id` is the last returned event's UUID;
- both parameters must be supplied together;
- the next page uses `(occurred_at, audit_event_record_id)` greater-than
  ordering and remains tenant-scoped by the transaction-local RLS context.

The existing `limit` remains bounded from 1 through 1,000. The cursor exposes
provenance identifiers already present in the response; it does not expose
source payloads or create a second audit authority.

## Consequences

Clients can walk a large audit history without offset drift while preserving
backward compatibility for existing limit-only callers. A client should retain
the last response item as the next cursor and stop when a page is empty. The
contract remains a forward-only read surface; retention, archival, and
partitioning are separate operational decisions.

## Verification

The `rust-kernel` PostgreSQL/API smoke path verifies a five-event first page,
the cursor page containing the remaining events in stable order, and HTTP 400
for a partial cursor. The query uses
`audit_event_record_tenant_occurred_idx` and the exact tenant transaction
context.
