# Learning Management Platform

A standards-oriented learning-management system for employees, customers, partners, certification candidates, association members, and self-sponsored learners.

## Scope

The platform owns learning offerings, enrollment, learner affiliations, progression projections, and versioned completion decisions. Employment linkage is optional rather than assumed.

Identity remains in Keyverse; authored releases remain in Learning Content Studio; observed learning activity remains in the Learning Record Store; assessment response/result authority remains in Psychometrics Commons; commercial entitlement remains in the Billing Control Plane.

The first buyer-facing vertical is a Partner & Customer Academy with external learner onboarding, entitlement, enrollment, standards-based activity, assessment handoff, completion, and portable credentials.

## Branching and promotion

Product work targets `develop`. Promotion from `develop` to `main` requires all of the following on the exact candidate head:

- an independent semantic review with no unresolved blocking thread;
- the repository's `validate` check from `.github/workflows/quality.yml` successful;
- the central required `.github/workflows/security-scan.yml` workflow successful;
- the central required `.github/workflows/sast-semgrep.yml` workflow successful;
- any additional checks required by the live repository ruleset successful.

A predecessor-head result, queued check, skipped required check, or stale approval is not promotion evidence. The live GitHub ruleset remains authoritative if it requires more than this repository baseline.

See `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`, `docs/product-technical-gap-baseline.md`, and `docs/doctoring/STANDARD_TRACEABILITY.md`.

## Executable kernel

The current implementation branch contains a Rust domain kernel, a PostgreSQL migration, and a small learner-registration HTTP adapter. Run the domain checks with `cargo test --workspace --all-targets --locked`; apply `migrations/0001_learning_kernel.sql` with a dedicated migration role; then run the API with a separate `NOSUPERUSER NOBYPASSRLS` application role that owns no tables and cannot create in the application schema.

The adapter requires `LMS_TENANT_API_KEY_SHA256` as a non-empty JSON object that maps each authorized tenant UUID to the lowercase SHA-256 digest of its bootstrap bearer key. `POST /v1/tenants/{tenant_id}/learners` accepts `Authorization: Bearer <key>` only when that key is bound to the requested tenant. This fail-closed bootstrap seam is not a Keyverse/OIDC conformance claim; the released Keyverse identity contract remains follow-up work.

The API exposes `GET /healthz` and accepts opaque identity references at the learner-registration endpoint. CI proves unauthenticated rejection, token-to-tenant authorization, a cross-tenant RLS write rejection, a non-owner application role, eight forced-RLS policies, and disposable migration rollback/reapply under the separate migration role. External identity, content, evidence, assessment, and billing adapters remain separate follow-up contracts.
