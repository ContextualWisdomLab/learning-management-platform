# Agent development rules

- Learners are first-class identities independent of employment status, payer, sponsor, or contracting organization.
- Do not duplicate Keyverse credentials, LRS statements, Studio authoring state, Psychometrics Commons responses, or Billing provider truth.
- Completion decisions must reference exact policy revisions and immutable evidence snapshots.
- The LMS may store only an external snapshot identifier, source authority, immutable digest, observed version, and an allowlisted `decision_time_metadata` object in a local `decision_evidence_reference`; its scalar keys are limited to `decision_reason_code`, `decision_method`, `evaluated_at`, `policy_revision_reference`, and, only for a correction, `correction_reason_code`. Evidence claims, credentials, LRS statements, psychometric responses, billing payloads, raw PII, and authoritative source payloads are forbidden; the external system remains the source of truth.
- Use two-or-more-word `snake_case` database object names and third normal form.
- Direct cross-repository database reads are prohibited; use versioned API/event contracts.
- Production statement and branch coverage must be 100%, with complete public API documentation.
- Realistic tests must include employee-linked, partner, customer, self-sponsored, repeated-enrollment, and multi-tenant learner journeys.
- The executable Rust kernel must pass `cargo fmt --all -- --check`, locked workspace tests, Clippy with warnings denied, and `cargo doc` with warnings denied.
- PostgreSQL changes must be applied to a real PostgreSQL instance in CI; tenant isolation, effective dates, migration rollback, and API retry/duplicate behavior require executable evidence.
