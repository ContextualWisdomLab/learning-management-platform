# Agent development rules

- Learners are first-class identities independent of employment status, payer, sponsor, or contracting organization.
- Do not duplicate Keyverse credentials, LRS statements, Studio authoring state, Psychometrics Commons responses, or Billing provider truth.
- Completion decisions must reference exact policy revisions and immutable evidence snapshots.
- The LMS may store only an external snapshot identifier, source authority, immutable digest, observed version, and decision-time metadata in a local `decision_evidence_reference`; it must not copy the authoritative source payload. The external system remains the source of truth.
- Use two-or-more-word `snake_case` database object names and third normal form.
- Direct cross-repository database reads are prohibited; use versioned API/event contracts.
- Production statement and branch coverage must be 100%, with complete public API documentation.
- Realistic tests must include employee-linked, partner, customer, self-sponsored, repeated-enrollment, and multi-tenant learner journeys.
