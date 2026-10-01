# Product and technical gap baseline

**As of:** 2026-09-30
**Status:** Code-current planning baseline. This document is not a product-readiness, release, deployment, certification, customer, or standards-conformance claim.

## Evidence authority

Protected `develop` is the shipped repository authority. Open PRs are candidate truth only until protected integration. Mutable PR head SHAs, workflow run IDs, review states, and mergeability are deliberately **not persisted as current evidence in this file** because a commit that records its own mutable review head becomes stale as soon as the branch moves. For integration decisions, re-fetch PR #1 and its exact current head directly from GitHub and accept only evidence attached to that unchanged head.

Historical bootstrap facts may remain pinned when they are explicitly historical. Protected `develop@1b89a16bbbd6c4b7c6ee4e8b81e2c8c651d1ce2c` was the repository initialization baseline and contained only the bootstrap README. PR #1 introduces the documentation/product architecture candidate described below; it must not be represented as shipped behavior before merge.

## Executive verdict

Protected `develop` is still a documentation and architecture foundation. The current candidate stack adds an executable Rust/PostgreSQL registration-to-enrollment kernel and tenant-safe persistence, but it is not yet a usable learning-management product. The buyer-facing opportunity is clear: let an employee, partner, customer, candidate, association member, or self-sponsored learner complete learning without manufacturing an HR record, while keeping identity, content, activity evidence, assessment responses/results, and billing truth in their owning systems. No released integration client, versioned API contract, browser journey, progress projection, or production test suite exists yet, so the first commercial slice remains the external-learner registration-to-completion vertical proven with real PostgreSQL and end-to-end tests.

The next customer-visible milestone is executable: a tenant-isolated learner can be entitled, enrolled, launched, progress-tracked, assessed when required, and issued a reproducible completion result without requiring an Orgmetra worker reference.

## Current-state evidence

| Evidence | Observed fact | Consequence |
|---|---|---|
| `develop@1b89a16bbbd6c4b7c6ee4e8b81e2c8c651d1ce2c` | Contains only the bootstrap `README.md`. | No runtime behavior exists on the base branch. |
| PR [#1](https://github.com/ContextualWisdomLab/learning-management-platform/pull/1) | Adds governance, architecture, data-model, ADR, standards, exact-head quality validation, explicit evidence-metadata allowlisting, and registration-to-decision cardinality. | The bootstrap is candidate evidence and must not be described as integrated behavior before protected merge. |
| PR [#4](https://github.com/ContextualWisdomLab/learning-management-platform/pull/4), canonical owner branch `feat/external-learner-kernel` | Adds the tenant-safe Rust kernel, PostgreSQL migration, registration adapter, employee-only Orgmetra worker-reference invariant, all nine non-employee variants, dependency lock, guarded deserialization, loopback-only plaintext binding, replay-complete evidence hashing, and tenant-and-learner completion constraints. | It remains candidate evidence until current exact-head checks, review, and protected integration succeed. |
| PR [#5](https://github.com/ContextualWisdomLab/learning-management-platform/pull/5) | Adds the bearer-authorized offering→entitlement→enrollment→registration adapter, tenant-scoped RLS, closed-registration retry invariant, and real PostgreSQL/API smoke path on top of PR #4. | This is an open Draft implementation PR, not merged product evidence; current exact-head checks and independent review still gate it. |
| Issue [#2](https://github.com/ContextualWisdomLab/learning-management-platform/issues/2) | Defines the repository boundary, modular-monolith slices, PostgreSQL 3NF, adapters, accessibility, and evidence gates. | This is the foundation backlog, not delivered functionality. |
| Issue [#3](https://github.com/ContextualWisdomLab/learning-management-platform/issues/3) | Defines the external-learner vertical and acceptance criteria for identity separation, effective dating, replayable completion, tenancy, and coverage. | This is the first product slice to implement after the bootstrap merges. |

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
- Every learner-owned enrollment link includes both tenant and learner identity; entitlement, enrollment, registration, completion, and evidence rows cannot be joined across learners inside one tenant.
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

| ID | Priority | Buyer-visible gap | Current evidence | Exit evidence | Next change |
|---|---:|---|---|---|---|
| G-01 | P0 | No merged executable LMS kernel or API | PR #4 adds the Rust kernel and learner registration; PR #5 extends the bounded enrollment API | Running service with documented health and tenant context on a merged exact head | Merge foundation stack after independent review and current checks |
| G-02 | P0 | No merged learner/identity/employee/sponsor/payer separation | PR #4 kernel requires an employee worker reference and rejects it for all nine non-employee kinds; PR #5 adds offering and entitlement separation, while the affiliation PostgreSQL/API projection remains open | Real PostgreSQL schema and integration tests for all roles | Carry the kernel invariant through the dependent affiliation schema/API stack |
| G-03 | P0 | No completed external learner journey | PR #5 implements bearer-authorized offering, entitlement projection, enrollment, and registration | Non-employee journey passes browser/API E2E through completion | Add launch, progress, assessment, completion, and credential slices |
| G-04 | P0 | No time-aware affiliation or correction model | Requirements only | Effective-dated and replay/correction tests pass | Add valid-time and decision transaction metadata |
| G-05 | P0 | No versioned external contracts | PR #1 lists planned identifiers only | Schemas, clients, contract tests, idempotent adapters | Add integration package and outbox |
| G-06 | P0 | No integrated completion/evidence publication | PR #4 adds a bounded evaluator whose replay fingerprint covers ordered policy and evidence metadata, plus normalized policy, revision, decision, and evidence-reference storage; no publication API or transaction connects them | Persisted replay produces the same decision from exact policy/evidence versions | Add the completion publication adapter and replay fixture |
| G-07 | P1 | No provenance, audit, or operational evidence | No runtime source | Correlation-linked audit and decision history with export | Add audit/provenance module |
| G-08 | P1 | No production PostgreSQL operability evidence | PR #4 adds 3NF relations, forced RLS, a non-superuser CI application role, tenant-and-learner decision/evidence foreign keys, valid-time constraint fixtures, and disposable rollback/reapply evidence; PR #5 extends tenant-and-learner foreign keys across entitlement, enrollment, registration, and completion candidate rows | Load, retention, recovery, and governed production rollback evidence | Add measured load, retention, and governed production rollback evidence |
| G-09 | P1 | No security/compliance control implementation | Standards profile only | CSAP/SOC 2/NIST control map with test receipts, SBOM, provenance | Add security and operability gates |
| G-10 | P1 | No accessibility/UI/design-system surface | No frontend files | Storybook, token tests, browser interaction and i18n evidence | Start UI only after API journey is real |
| G-11 | P1 | No realistic tests or coverage | PR #4 covers employee, partner, customer, self-sponsored, multi-tenant, duplicate-request, and domain-boundary cases; full enrollment journey coverage remains open | 100% owned statement/branch coverage plus edge and E2E evidence | Extend coverage with enrollment and external-learner journey slices |
| G-12 | P1 | No explicit public source license on protected branch | Apache-2.0 grant is candidate-only until the foundation integrates | Protected root LICENSE plus accurate source/dependency notices | Integrate the foundation through ordinary protection |
| G-13 | P2 | No release, rollback, or cross-repository ecosystem loop | Open stacked PRs; no released artifact | Versioned release, changelog, provenance, rollback evidence | Merge the repaired stack, then release a bounded vertical |

## PR and issue integration loop

For every open PR: re-fetch the exact current head, inspect reviews/threads and exact-head checks, correct valid repository-owned failures, and merge only through live protected governance. Do not persist a mutable current head SHA in this file as if it were durable product evidence. A predecessor check is historical after a push; a green check is not semantic approval; a merge is not runtime proof.

The current loop is:

1. PR #1: keep the bootstrap Draft until its exact head is reviewed and its current checks pass.
2. PR #4: verify the repaired canonical learner-kernel owner at its current exact head and integrate only after required review and checks.
3. PR #5: non-force restack onto PR #4, then verify the bearer-authorized enrollment path, PostgreSQL evidence, and exact-head checks.
4. Dependent affiliation PRs: carry the invariant through PostgreSQL and API contracts without copying owner code or retiring valid deltas.
5. Issue #3: prove the remaining external-learner journey on the repaired stack.

New runtime evidence updates this gap register rather than accumulating self-staling PR status prose.

## Standards and research evidence

The standards profile remains planned adoption. Exact revisions, official sources, and evidence status are maintained in [`docs/doctoring/STANDARD_TRACEABILITY.md`](doctoring/STANDARD_TRACEABILITY.md). The profile includes the repository-selected LTI/LTI Advantage, QTI, CASE, Open Badges, CLR, xAPI/IEEE, and learning-management quality/security references, but documentation alone is not conformance.

Learning-analytics research reinforces the product boundary: activity data are proxy signals and can create surveillance, aggregation, secondary-use, exclusion, distortion, and decisional-interference risks. Progress projections therefore remain evidence inputs and cannot silently become high-impact completion or employment decisions without an explicit versioned policy and auditable governance.

## References

1EdTech Consortium. (n.d.). *Competencies and Academic Standards Exchange (CASE).* Retrieved August 20, 2026, from https://www.1edtech.org/standards/case

1EdTech Consortium. (n.d.). *Comprehensive Learner Record standard.* Retrieved August 20, 2026, from https://www.1edtech.org/standards/clr

1EdTech Consortium. (n.d.). *Learning Tools Interoperability (LTI).* Retrieved August 20, 2026, from https://www.1edtech.org/standards/lti

1EdTech Consortium. (n.d.). *Open Badges.* Retrieved August 20, 2026, from https://www.1edtech.org/standards/open-badges

1EdTech Consortium. (n.d.). *Question & Test Interoperability (QTI).* Retrieved August 20, 2026, from https://www.1edtech.org/standards/qti

Badiuzzaman, M. B., & Rahman, S. S. (2026). Beyond compliance: A Solove-informed analysis of tracking, profiling, and student privacy in learning management systems. *Frontiers in Education, 11*. https://doi.org/10.3389/feduc.2026.1871384

IEEE Standards Association. (2023). *IEEE Standard for learning technology—JavaScript Object Notation (JSON) data model format and Representational State Transfer (RESTful) web service for learner experience data tracking and access (IEEE Std 9274.1.1-2023).* https://standards.ieee.org/ieee/9274.1.1/7321/

National Institute of Standards and Technology. (2024). *The NIST Cybersecurity Framework (CSF) 2.0* (NIST CSWP 29). https://doi.org/10.6028/NIST.CSWP.29

PostgreSQL Global Development Group. (2026). *PostgreSQL 18 documentation: Row security policies; table partitioning.* https://www.postgresql.org/docs/current/ddl-rowsecurity.html; https://www.postgresql.org/docs/current/ddl-partitioning.html
