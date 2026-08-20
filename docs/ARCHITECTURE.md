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

The current implementation slice is `crates/lms_kernel`: Rust domain rules enforce non-employee affiliations, effective dates, tenant/learner evidence boundaries, passed-assessment requirements, and replay fingerprints. `crates/lms_kernel/src/bin/lms_api.rs` provides health plus affiliation, offering, learner, entitlement, enrollment, registration, attempt, progress-projection, policy, evidence, assessment-result-reference, completion-decision, credential-reference, and append-only audit-event writes backed by the migration in `migrations/0001_learning_kernel.sql`. This is an executable bounded affiliation/registration/enrollment/progress/assessment-reference/completion/credential/audit kernel, not yet the complete Partner & Customer Academy journey. Released external adapters, assessment execution/scoring, and browser E2E remain open gaps in `docs/product-technical-gap-baseline.md`.
