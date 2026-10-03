# Learning Management Platform

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/ContextualWisdomLab/learning-management-platform)

**A standards-oriented learning platform for employees and external learners without forcing learning identity into an HR record.**

Learning Management Platform is designed for employee, customer, partner, certification-candidate, association-member, and self-sponsored learning. It owns the learning journey—offerings, learner affiliations, enrollment, registrations, completion policy, and versioned completion decisions—while preserving the authority of the systems that own identity, content, observed activity, psychometric evidence, employment, and billing. Employment linkage is optional: employee affiliations require an opaque Orgmetra worker reference, while every non-employee affiliation rejects one.

The first buyer-facing vertical is a **Partner & Customer Academy**: onboard an external learner, establish entitlement, enroll them, launch standards-based learning, consume activity and assessment evidence, publish a reproducible completion decision, and hand off a portable credential reference without inventing an employee record.

## Why it exists

Many LMS designs quietly equate learner, employee, login, payer, sponsor, and organization. That breaks down for customer education, partner certification, associations, candidate learning, and mixed B2B/B2C programs.

This product treats those roles as separate first-class concepts and keeps high-value external facts in their owning systems.

| Need | Product responsibility |
| --- | --- |
| External and employee learning | One learner/enrollment model that does not require employment |
| Reproducible completion | Immutable policy revisions + immutable evidence references + superseding corrections |
| Multi-tenant learning | Tenant-scoped identities, relationships, evidence, and future authorization boundaries |
| Standards-based integration | Versioned API/event/package contracts; planned standards adoption is evidence-bound |
| Privacy without unusable masking | Minimal local projections/references instead of copying provider truth |
| Ecosystem composition | Explicit ownership boundaries; no cross-repository application-table reads |

## Product boundary

See `docs/ARCHITECTURE.md`, `docs/DATA_MODEL.md`, `docs/product-technical-gap-baseline.md`, and `docs/doctoring/STANDARD_TRACEABILITY.md`.

## Executable kernel

The current implementation branch contains a Rust domain kernel, a PostgreSQL migration, and a small learner-registration HTTP adapter. Run the domain checks with `cargo test --workspace --all-targets --locked`; apply `migrations/0001_learning_kernel.sql` with a dedicated migration role; then run the API with a separate `NOSUPERUSER NOBYPASSRLS` application role that owns no tables and cannot create in the application schema.

The adapter requires `LMS_TENANT_API_KEY_SHA256` as a non-empty JSON object that maps each authorized tenant UUID to the lowercase SHA-256 digest of its bootstrap bearer key. `POST /v1/tenants/{tenant_id}/learners` accepts `Authorization: Bearer <key>` only when that key is bound to the requested tenant. Because this bounded adapter serves plaintext HTTP, `LMS_BIND_ADDRESS` is restricted to loopback; external traffic requires a separately governed TLS-capable edge. This fail-closed bootstrap seam is not a Keyverse/OIDC conformance claim; the released Keyverse identity contract remains follow-up work.

The API exposes `GET /healthz` and accepts opaque identity references at the learner-registration endpoint. CI proves unauthenticated rejection, token-to-tenant authorization, a cross-tenant RLS write rejection, a non-owner application role, eight forced-RLS policies, and disposable migration rollback/reapply under the separate migration role. External identity, content, evidence, assessment, and billing adapters remain separate follow-up contracts.

The platform owns:

- learning tenants and learner profiles;
- learner affiliations independent of employment;
- catalog/offering projections tied to immutable content releases;
- enrollment, registration, and learning-attempt state;
- local progress projections from external activity evidence;
- immutable completion-policy revisions and versioned completion decisions;
- credential orchestration state and integration/audit boundaries.

It does **not** become authoritative for adjacent systems merely because it consumes their evidence:

| Authority | Owning product/system | LMS boundary |
| --- | --- | --- |
| Identity, authentication credentials, OIDC/federation/SCIM | **Keyverse** | stable external subject/reference only |
| Employment and organization truth | **Orgmetra** | optional effective-dated worker/employment reference |
| Authored learning content releases | **Learning Content Studio** | immutable release reference |
| Observed learning activity | **Learning Record Store** | versioned evidence/progress projection |
| Assessment sessions, responses, result snapshots | **Psychometrics Commons** | immutable result/evidence reference |
| Commercial entitlement/payment truth | **Billing Control Plane** | versioned entitlement projection/reference |

Direct cross-repository database reads are prohibited. Integrations use explicit versioned contracts and do not copy authoritative provider payloads into the LMS as convenience state.

## First end-to-end journey

```text
external or employee learner identity
                │
                ▼
       learner + affiliation
                │
      entitlement projection
                │
                ▼
      offering → enrollment
                │
                ▼
          registration
                │
     activity / assessment evidence
                │
                ▼
     exact completion-policy revision
                │
                ▼
   immutable completion decision
                │
                ▼
       credential orchestration
```

A completion decision references exactly one registration, exactly one authoritative `completion_policy_revision`, and one or more immutable evidence references. If evidence metadata also carries `policy_revision_reference`, that value is only an audit mirror: it must identify the same authoritative revision or fail closed.

## Current state

Protected `develop` is still an **architecture and documentation foundation**. This candidate branch adds an executable Rust kernel, PostgreSQL schema, and bounded registration adapter, but it is not an integrated LMS release and does not claim a browser workflow, production deployment, standards conformance, customer deployment, or released artifact.

Protected `develop` remains shipped repository authority. Open PR behavior is candidate truth until it integrates through current review and exact-head checks.

The current prioritized implementation gaps are maintained in [`docs/product-technical-gap-baseline.md`](docs/product-technical-gap-baseline.md); that file deliberately does not persist mutable PR-head SHAs as current evidence.

## Repository validation

The repository-owned quality workflow validates both the documentation/bootstrap contract and the Rust/PostgreSQL kernel against the exact PR head. The kernel checks use the pinned Rust toolchain and dependency lock; the database path uses separate migration and `NOSUPERUSER NOBYPASSRLS` application roles.

The repository-owned contexts are `validate` and `rust-kernel` in [`.github/workflows/quality.yml`](.github/workflows/quality.yml). Promotion decisions must use the exact current head and live repository/organization governance; predecessor checks and stale approvals do not transfer after a push.

## Architecture and data model

The initial target is a modular monolith with explicit bounded modules for tenant management, learner registry, catalog/offering, enrollment, progress projection, completion policy, credential orchestration, integration, and audit/provenance. Repository splits should follow real ownership, trust, or release-cadence boundaries rather than internal class structure.

The data model requires third normal form, two-or-more-word `snake_case` database object names, tenant-scoped foreign keys, effective dating where relationships change over business time, immutable policy revisions, and immutable/superseding completion decisions.

Start with:

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — product and integration architecture;
- [`docs/DATA_MODEL.md`](docs/DATA_MODEL.md) — learner, enrollment, completion, evidence, and entitlement relationships;
- [`docs/adr/0001-learning-authority-boundary.md`](docs/adr/0001-learning-authority-boundary.md) — authority boundary decision.

## Standards posture

The repository tracks a standards-oriented target profile for learning interoperability and credentials, but a documented target is not implementation or conformance. Exact revisions, official sources, planned-vs-proven status, and evidence expectations are maintained in [`docs/doctoring/STANDARD_TRACEABILITY.md`](docs/doctoring/STANDARD_TRACEABILITY.md).

Do not infer standards certification or interoperability from the presence of a standards name in this README.

## Documentation map

| Goal | Start here |
| --- | --- |
| Architecture | [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) |
| Data model | [`docs/DATA_MODEL.md`](docs/DATA_MODEL.md) |
| Authority ADR | [`docs/adr/0001-learning-authority-boundary.md`](docs/adr/0001-learning-authority-boundary.md) |
| Standards traceability | [`docs/doctoring/STANDARD_TRACEABILITY.md`](docs/doctoring/STANDARD_TRACEABILITY.md) |
| Product / technical gaps | [`docs/product-technical-gap-baseline.md`](docs/product-technical-gap-baseline.md) |
| Change history | [`CHANGELOG.md`](CHANGELOG.md) |
| Contributor / agent rules | [`AGENTS.md`](AGENTS.md), [`CLAUDE.md`](CLAUDE.md) |

## Contributing

Preserve learner independence from employment, external authority boundaries, exact policy/evidence replayability, tenant isolation, and versioned integration contracts. Do not duplicate Keyverse credentials, LRS statements, authored content, psychometric response payloads, Billing provider truth, or another repository's application tables.

Changes to public/product contracts should update the corresponding architecture/data/ADR/gap documentation and remain subject to the repository's exact-head validation and central security/review governance.

## License

Learning Management Platform is licensed under the [Apache License 2.0](LICENSE). Repository-authored documentation and Rust source use that grant; dependencies, generated assets, and imported components retain their own obligations and must remain compatible with the organization's commercial-use policy.
