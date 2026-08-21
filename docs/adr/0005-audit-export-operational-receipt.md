# ADR-0005: Audit-export operational receipt

**Status:** Accepted for the bounded audit API
**Date:** 2026-08-21

## Context

The tenant audit export can now be walked with a stable keyset cursor, but an
operator still needs a compact record of what a response contained when a
page is copied into an incident or reconciliation record. The receipt must
not add provider payloads or make the LMS a second audit authority.

## Decision

Keep the existing JSON array response and add response headers for every audit
export page:

- `X-Audit-Export-Event-Count` is the number of events in the page;
- `X-Audit-Export-Receipt-Digest` is the lowercase SHA-256 digest of each
  ordered pair formed from the lowercase canonical UUID text of
  `audit_event_record_id` and `event_digest`, encoded as UTF-8, separated by a
  NUL byte and terminated by a NUL byte;
- `X-Audit-Export-Next-Occurred-At` and
  `X-Audit-Export-Next-Event-ID` identify the last event and can be supplied as
  the next keyset cursor. They are omitted for an empty page.

The receipt is response evidence for the caller to retain with its operational
record. It is not a persisted audit event and does not expose source payloads.

## Consequences

Existing clients that only parse the JSON array remain compatible. Export
consumers can verify page count and ordered event provenance independently,
record the exact resume cursor, and correlate a page with an incident or
reconciliation record without copying assessment, credential, or billing
payloads.

## Verification

The exact-head API smoke path verifies receipt count, digest recomputation, and
last-event cursor headers for both the first and subsequent keyset pages while
retaining tenant isolation, payload exclusion, and partial-cursor rejection.
