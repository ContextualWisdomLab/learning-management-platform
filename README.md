# Learning Management Platform

A standards-oriented learning-management system for employees, customers, partners, certification candidates, association members, and self-sponsored learners.

## Scope

The platform owns learning offerings, enrollment, learner affiliations, progression projections, and versioned completion decisions. Employment linkage is optional rather than assumed.

Identity remains in Keyverse; authored releases remain in Learning Content Studio; observed learning activity remains in the Learning Record Store; assessment response/result authority remains in Psychometrics Commons; commercial entitlement remains in the Billing Control Plane.

The first buyer-facing vertical is a Partner & Customer Academy with external learner onboarding, entitlement, enrollment, standards-based activity, assessment handoff, completion, and portable credentials.

## Branching and promotion

Product work targets `develop`. Promotion from `develop` to `main` requires all of the following on the exact candidate head:

- an independent semantic review with no unresolved blocking thread;
- `Learning Management Quality` successful;
- organization-required `Security Scan` successful;
- organization-required `SAST Semgrep` successful;
- any additional checks required by the live repository ruleset successful.

A predecessor-head result, queued check, skipped required check, or stale approval is not promotion evidence. The live GitHub ruleset remains authoritative if it requires more than this repository baseline.

See `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`, `docs/product-technical-gap-baseline.md`, and `docs/doctoring/STANDARD_TRACEABILITY.md`.
