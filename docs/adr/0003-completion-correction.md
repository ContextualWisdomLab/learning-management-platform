# ADR-0003: Immutable completion correction

**Status:** Accepted for the bounded kernel  
**Date:** 2026-08-20

## Context

External assessment and learning evidence can be corrected after a completion
decision has been published. Rewriting the old decision would destroy the
history needed by credential, audit, and replay consumers. A correction must
also be unable to attach to another learner or registration.

## Decision

Accept an optional `supersedes_decision_id` on the completion-decision request.
Within the tenant transaction, the API requires the referenced decision to
belong to the same tenant, learner, and learning registration. It then inserts
a new decision, returns both UUIDs, and records
`completion_decision.corrected`. The predecessor, evidence rows, and prior
audit event remain immutable. Invalid or cross-registration predecessor
references return HTTP 400 before a new decision is written.

The correction relation is deliberately local to the LMS decision history. It
does not make the LMS the authority for assessment scoring, credential payloads,
identity, content, or billing.

## Consequences

- Consumers can follow a complete decision history and choose the newest valid
  decision under their own policy.
- Audit export can distinguish an original publication from a correction without
  copying source payloads.
- Retention, archive, transaction-time metadata, and provider reconciliation
  remain separate operational decisions; append-only history cannot be silently
  purged.
- No Figma file ID is applicable because this ADR changes an API/data path and
  the repository has no frontend. A future UI ADR must include the Figma ID and
  Storybook/design-token evidence.

## Verification

- Pinned Rust formatting, tests, Clippy, and documentation checks pass locally.
- `actionlint` and `git diff --check` pass.
- A real PostgreSQL 18.4/API run with a `NOSUPERUSER` role verified two linked
  decisions, invalid predecessor HTTP 400, corrected audit export, direct
  cross-tenant RLS reads of zero, and source-payload exclusion.
- The exact-head quality smoke path asserts both invalid and valid correction
  behavior before protected merge.
