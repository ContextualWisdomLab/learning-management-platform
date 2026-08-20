# Learning Management Platform

A standards-oriented learning-management system for employees, customers, partners, certification candidates, association members, and self-sponsored learners.

## Scope

The platform owns learning offerings, enrollment, learner affiliations, progression projections, versioned completion decisions, and credential reference orchestration. Employment linkage is optional rather than assumed.

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

See `docs/PRD.md`, `docs/TRD.md`, `docs/UML.md`, `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`, `docs/product-technical-gap-baseline.md`, and `docs/doctoring/STANDARD_TRACEABILITY.md`.

## Executable kernel

The current implementation branch contains a Rust domain kernel, a PostgreSQL migration, and a small learner-affiliation/registration/enrollment/progress/assessment/completion/credential HTTP adapter. Run the domain checks with `cargo test --workspace --all-targets --locked`; run the API with `DATABASE_URL=postgres://... cargo run --bin lms_api`. The API applies the embedded migrations on startup, exposes `GET /healthz`, supports the bounded path `learners → affiliations → offerings → entitlements → enrollments → registrations → attempts → progress projections → assessment result reference → completion decision → credential reference`, and provides tenant-scoped audit export at `GET /v1/tenants/{tenant_id}/audit-events`. Audited writes accept `X-Correlation-ID`; completion corrections accept `supersedes_decision_id` and create a new immutable decision; export returns provenance metadata without source payloads. External identity, content, and billing adapters remain separate follow-up contracts; Psychometrics Commons remains the assessment authority.

Run `scripts/postgres_recovery_rehearsal.sh` with native libpq connection
environment variables to exercise disposable backup/restore, tenant-skew
assertions, RLS preservation, and a forward migration.
