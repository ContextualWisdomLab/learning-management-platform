# Product and technical gap baseline

**Status:** Code-current planning baseline. This document is not a product-readiness, release, deployment, certification, customer, or standards-conformance claim.

## Evidence authority

Protected `develop` is the shipped repository authority. Open PRs are candidate truth only until protected integration. Mutable PR head SHAs, workflow run IDs, review states, and mergeability are deliberately **not persisted as current evidence in this file** because a commit that records its own mutable review head becomes stale as soon as the branch moves. For integration decisions, re-fetch PR #1 and its exact current head directly from GitHub and accept only evidence attached to that unchanged head.

Historical bootstrap facts may remain pinned when they are explicitly historical. Protected `develop@1b89a16bbbd6c4b7c6ee4e8b81e2c8c651d1ce2c` was the repository initialization baseline and contained only the bootstrap README. PR #1 introduces the documentation/product architecture candidate described below; it must not be represented as shipped behavior before merge.

## Executive verdict

The repository is still a documentation and architecture foundation, not yet a usable learning-management product. The buyer-facing opportunity is clear: let an employee, partner, customer, candidate, association member, or self-sponsored learner complete learning without manufacturing an HR record, while keeping identity, content, activity evidence, assessment responses/results, and billing truth in their owning systems.

The next customer-visible milestone is executable: a tenant-isolated learner can be entitled, enrolled, launched, progress-tracked, assessed when required, and issued a reproducible completion result without requiring an Orgmetra worker reference.

## Product requirements baseline

### First buyer journey

1. A tenant administrator creates or imports a course offering from an immutable content-release reference.
2. A learner enters through Keyverse or an approved external identity reference; no employee record is required.
3. The learner receives a sponsor- or self-funded entitlement projection and enrolls in the offering.
4. The LMS launches the registered learning activity and consumes progress projections from the LRS.
5. The learner completes required activity and, when applicable, an assessment delegated to Psychometrics Commons.
6. A versioned completion policy evaluates immutable evidence references and publishes an immutable decision.
7. Credential orchestration issues a portable credential reference without copying source content, assessment payloads, or billing-provider truth.

### Roles that must not collapse

`learner_profile`, login identity, employee/worker reference, sponsor, payer, contracting organization, and credential beneficiary are separate concepts. One login identity may link to several tenant-scoped learner affiliations. A learner may have no worker reference, several effective-dated affiliations, or concurrent affiliations in different tenants.

### Acceptance slice

The first vertical is complete only when an unchanged candidate head demonstrates all of the following:

- a non-employee learner completes the registration-to-completion journey;
- an employee-linked learner follows the same domain path without special-case duplication;
- sponsor, payer, learner, identity, organization, and credential recipient remain distinguishable in API and database records;
- content release, LRS evidence, assessment result, identity, and billing objects remain external references;
- the completion result names the exact authoritative policy revision and immutable evidence references and can be replayed;
- any optional `decision_time_metadata.policy_revision_reference` exactly mirrors the authoritative tenant-scoped `completion_policy_revision` FK or the write fails closed;
- cross-tenant reads and writes fail closed;
- repeated enrollment, concurrent affiliations, expiry, correction, replay, retry, and out-of-order integration events are tested;
- real PostgreSQL, browser E2E, security, migration/rollback, provenance, and observability evidence is attached to the candidate head.

## Technical target baseline

### Modular-monolith boundaries

The first implementation should remain one deployable service with explicit modules. Split repositories only when ownership, release cadence, or trust boundaries require independent deployment.

| Module | Owns | Must call |
|---|---|---|
| Tenant management | tenant lifecycle and isolation context | authorization context |
| Learner registry | learner profile and affiliations | Keyverse/Orgmetra references |
| Catalog and offering | catalog records, immutable release references, offerings | Learning Content Studio contract |
| Enrollment | entitlement-aware enrollment, registration, attempts | Billing and LRS contracts |
| Progress projection | local progress view and sync state | LRS contract |
| Completion policy | policy revisions, evidence references, immutable decisions | LRS and Psychometrics Commons contracts |
| Credential orchestration | credential issue/revoke projection | Open Badges/CLR-compatible contract |
| Integration hub | versioned API/event clients, idempotency, outbox, retries | all external authorities |
| Audit and provenance | actor, tenant, correlation, source, digest, decision history | authorization and observability |

### Data and time invariants

- Every tenant-owned table has a tenant key, and every cross-tenant foreign key includes the tenant boundary.
- Authoritative facts use third-normal-form relations with descriptive two-or-more-word `snake_case` names.
- Effective-dated relationships use `valid_from` and `valid_to`; correctable observed facts also retain transaction/observation metadata and a superseding relation.
- Completion policies are immutable revisions. `completion_decision.completion_policy_revision` is the authoritative revision identity for replay/audit.
- `decision_time_metadata.policy_revision_reference`, when present, is only a non-authoritative audit mirror and must exactly match that authoritative revision; mismatch fails closed.
- A correction creates a new completion decision pointing to the prior decision; published evidence is never rewritten in place.
- `decision_evidence_reference` stores source authority, opaque snapshot ID, digest, observed version, and explicitly allowlisted scalar decision-time metadata, never source payload copies.
- Entitlements are versioned local projections of external authority, never a second billing ledger.

### PostgreSQL and hot-partition baseline

Use PostgreSQL row-level security as defense in depth, with application authorization and tenant-scoped foreign keys still required. Keep low-volume identity/enrollment tables ordinary until measurement justifies partitioning. For append-heavy audit, outbox, and progress/evidence projection tables, use measured time-range partitions for retention and a tenant hash bucket only when load evidence shows a single-tenant hot partition. Every partitioned write path requires index, retention, recovery, and rollback evidence.

### Integration contract baseline

No module may read another repository's application tables. Each external boundary needs a versioned OpenAPI/package/event schema, consumer/provider contract tests, idempotency key, correlation ID, observed source version, digest, retry/dead-letter policy, and documented authority. Until those artifacts exist, the boundary remains `planned`, not `implemented`.

### Security, privacy, accessibility, and operability

PII must remain usable for legitimate work without being copied into every service. The target control set uses least privilege, purpose- and tenant-scoped authorization, field-level audit, consent/retention policy, encryption, controlled break-glass access, and opaque external references where full source data are unnecessary.

The future UI targets WCAG 2.2 AA, keyboard/screen-reader behavior, focus/error semantics, locale/time-zone consistency, design tokens, and a Storybook inventory. There is no current UI surface, so no Figma publication claim is made.

Production readiness requires evidence for CSAP/SOC 2-oriented controls, SBOM/provenance, key rotation, backup/restore, migration rollback, rate limits, alerting, SLOs, audit export, incident response, and realistic load. NIST CSF 2.0 may organize controls; it is not a certification.

## Gap register and delivery order

| ID | Priority | Buyer-visible gap | Current evidence | Exit evidence |
|---|---:|---|---|---|
| G-01 | P0 | No executable LMS kernel/API | documentation/architecture candidate only | running service with health and tenant context |
| G-02 | P0 | No executable learner/identity/employee/sponsor/payer separation | data-model contract only | PostgreSQL schema + integration tests for each role |
| G-03 | P0 | No external learner enrollment path | product journey only | non-employee browser/API E2E |
| G-04 | P0 | No implemented time-aware affiliation/correction model | requirements only | valid-time + replay/correction tests |
| G-05 | P0 | No released versioned external contracts | planned identifiers only | schemas, clients, contract tests, idempotent adapters |
| G-06 | P0 | No deterministic completion/evidence engine | decision contract prose only | replay from exact policy/evidence versions |
| G-07 | P1 | No provenance/audit runtime evidence | no runtime source | correlation-linked audit/decision history + export |
| G-08 | P1 | No PostgreSQL 3NF/RLS/migration/hot-partition implementation | model design only | migration, RLS, load, retention, rollback evidence |
| G-09 | P1 | No security/compliance implementation | standards/control baseline only | tested controls, SBOM, provenance |
| G-10 | P1 | No accessibility/UI/design-system surface | no frontend | Storybook, token, browser interaction, i18n evidence |
| G-11 | P1 | No runtime coverage/E2E evidence | documentation validation only | 100% owned statement/branch + edge/E2E evidence |
| G-12 | P1 | No explicit public source license on protected branch | licensing decision may exist only on active PR until merge | protected root LICENSE + README/package metadata as applicable |
| G-13 | P2 | No release/rollback ecosystem evidence | no released artifact | versioned release, changelog, provenance, rollback evidence |

## PR and issue integration loop

For every open PR: re-fetch the exact current head, inspect reviews/threads and exact-head checks, correct valid repository-owned failures, and merge only through live protected governance. Do not persist a mutable current head SHA in this file as if it were durable product evidence. A predecessor check is historical after a push; a green check is not semantic approval; a merge is not runtime proof.

After the documentation/bootstrap PR integrates, the executable modular-monolith foundation and the external-learner vertical are the next bounded product slices. New runtime evidence should update this gap register rather than accumulate self-staling PR status prose.

## Standards and research evidence

The standards profile remains planned adoption. Exact revisions, official sources, and evidence status are maintained in [`docs/doctoring/STANDARD_TRACEABILITY.md`](doctoring/STANDARD_TRACEABILITY.md). The profile includes the repository-selected LTI/LTI Advantage, QTI, CASE, Open Badges, CLR, xAPI/IEEE, and learning-management quality/security references, but documentation alone is not conformance.

Learning-analytics research reinforces the product boundary: activity data are proxy signals and can create surveillance, aggregation, secondary-use, exclusion, distortion, and decisional-interference risks. Progress projections therefore remain evidence inputs and cannot silently become high-impact completion or employment decisions without an explicit versioned policy and auditable governance.
