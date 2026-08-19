# Architecture

The Learning Management Platform is the authoritative system for learning offerings, enrollment, learner affiliations, progression projections, and versioned completion policy. A learner may be an employee, contractor, customer, partner, certification candidate, association member, or self-sponsored individual.

It does not own authentication credentials, authored source content, xAPI statements, psychometric responses, or payment-provider state.

## Integration boundaries

- Keyverse: identity, OIDC, federation, and optional SCIM linkage.
- Orgmetra: optional employment and organizational linkage; non-employees remain first-class learners.
- Learning Content Studio: immutable content releases.
- Learning Record Store: observed learning activity.
- Psychometrics Commons: assessment sessions and immutable result snapshots.
- Semantic Data Portal: competency and learning-outcome concepts.
- Billing Control Plane: entitlement and commercial authorization.

## First vertical

Partner & Customer Academy: external learner onboarding, sponsor or self entitlement, enrollment, standards-based learning activity, assessment handoff, completion-policy evaluation, and portable credential issuance.
