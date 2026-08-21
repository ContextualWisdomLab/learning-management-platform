# Architecture

The Learning Management Platform is the authoritative system for learning offerings, enrollment, learner affiliations, progression projections, and versioned completion policy. A learner may be an employee, contractor, customer, partner, certification candidate, association member, or self-sponsored individual.

It does not own authentication credentials, authored source content, xAPI statements, psychometric responses, or payment-provider state.

## Integration boundaries

Direct cross-repository database reads are prohibited. Every integration uses a released, versioned API or event contract; until that contract exists, the integration is considered planned rather than implemented.

| Authority | Ownership boundary | Planned versioned contract identifier |
|---|---|---|
| Keyverse | Identity, OIDC, federation, optional SCIM linkage | `keyverse_identity_reference/v1` via versioned identity API |
| Orgmetra | Optional employment and organizational linkage; non-employees remain first-class learners | `orgmetra_worker_reference/v1` via versioned HR reference API/event |
| Learning Content Studio | Immutable content releases | `learning_content_release/v1` event/API contract |
| Learning Record Store | Observed learning activity and immutable evidence references | `learning_evidence_reference/v1` API plus xAPI-version-aware integration |
| Psychometrics Commons | Assessment sessions and immutable result snapshots | `assessment_result_reference/v1` event/API contract |
| Semantic Data Portal | Competency and learning-outcome concepts | `competency_reference/v1` API contract |
| Billing Control Plane | Commercial entitlement and authorization | `learning_entitlement_reference/v1` API/event contract |

These identifiers name CWL-owned boundary contracts; they are not conformance claims. Each becomes implemented only when a released schema/client and provider-consumer contract test exist.

## First vertical

Partner & Customer Academy: external learner onboarding, sponsor or self entitlement, enrollment, standards-based learning activity, assessment handoff, completion-policy evaluation, and portable credential issuance.

## Executable baseline

The current implementation slice is `crates/lms_kernel`: Rust domain rules enforce non-employee affiliations, effective dates, tenant/learner evidence boundaries, and replay fingerprints. `crates/lms_kernel/src/bin/lms_api.rs` provides health plus offering, learner, entitlement, enrollment, and registration endpoints backed by the migration in `migrations/0001_learning_kernel.sql`. The CI API connection uses a `NOSUPERUSER NOBYPASSRLS` role, and `scripts/postgres_rollback_rehearsal.sql` verifies the disposable migration can be removed and reapplied. This is an executable bounded registration/enrollment kernel, not yet the complete Partner & Customer Academy journey. Progress, external adapters, completion persistence, credentials, and browser E2E remain open gaps in `docs/product-technical-gap-baseline.md`.
