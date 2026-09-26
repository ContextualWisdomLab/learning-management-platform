# Architecture

## Bounded context

The Learning Management bounded context owns learner-to-tenant participation:
affiliation, enrollment, progress, completion policy, and credential
orchestration. The first aggregate is `LearningAffiliation`, a time-varying link
between a `learner_id` and `tenant_id`.

`AffiliationKind` is Ubiquitous Language. `Employee` requires an opaque
Orgmetra worker reference. `Customer`, `Partner`, `Contractor`, `Candidate`,
`Student`, `Guardian`, `AssociationMember`, `PublicLearner`, and
`SelfSponsoredLearner` explicitly reject that reference, allowing the same
product flow to serve non-workforce learners without inventing employees.

## Context Map

| Context | Relationship | Data allowed into this context |
|---|---|---|
| Keyverse | Identity ACL, upstream | Subject and tenant claims through a released OIDC/SCIM contract |
| Orgmetra | Workforce ACL, upstream | Opaque worker reference for `Employee` only |
| learning-record-store | Learning-event ACL, downstream projection | Released event contract; no cross-service SQL |
| Billing owner | Entitlement ACL, upstream | Read-only entitlement decision; no billing arithmetic |
| Psychometrics Commons / fast-mlsirm | Placement/scoring ACL, upstream | Immutable released result contract only |

## Invariants

1. `tenant_id`, `affiliation_id`, and `learner_id` are nonblank.
2. A bounded validity interval ends strictly after it starts.
3. `Employee` carries one nonblank Orgmetra worker reference.
4. Every non-employee affiliation carries no Orgmetra worker reference.
5. External systems remain references behind ACLs; this repository never reads
   their databases.

Database, API, enrollment, completion, and UI designs remain Proposed and are
tracked in the Gap baseline rather than implied by this kernel.
