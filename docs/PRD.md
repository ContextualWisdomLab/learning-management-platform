# PRD — Learning Management Platform Foundation

## Goal

Enable a tenant to represent an external customer or partner learner without an
Orgmetra worker dependency, while retaining an explicit workforce ACL for
employees.

## Actors and first journey

- **Learner:** receives tenant-scoped learning access.
- **Sponsor:** selects or funds access without becoming the learner.
- **Payer:** owns the commercial transaction outside this bounded context.
- **Tenant administrator:** governs affiliations and later offerings.

The first executable journey creates a customer or partner affiliation with no
worker reference. Employee creation fails closed until an Orgmetra reference is
supplied.

## Acceptance criteria

1. Customer and Partner affiliations succeed without Orgmetra.
2. Employee affiliations fail without a nonblank workforce reference.
3. Non-employee affiliations reject workforce references.
4. Semantic identifiers are nonblank and validity windows move forward.
5. The behavior is dependency-free, publicly documented, and measured at 100%
   line and branch coverage on the exact PR head.

Enrollment, completion policy, credentials, persistence, API, and UI are
separate incremental outcomes in the Gap baseline.

