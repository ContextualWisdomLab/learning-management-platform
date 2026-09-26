# ADR 0001 — External learner affiliation boundary

## Status

Proposed

## Problem

Learning access must support partner and customer academy users who have no
employment record. Making Orgmetra mandatory would leak the workforce model
into Learning Management and block a commercial external-learner journey.

## Constraints

- Learning Management owns affiliations; Orgmetra owns workforce truth.
- External identities cross boundaries only through released contracts and ACLs.
- The first delta must be testable without a database or network dependency.
- Proposed code cannot claim a released API, database, or UI.

## Alternatives

1. **Require an Orgmetra worker for every learner.** Rejected because customer
   and partner learners are not workers.
2. **Copy an Orgmetra worker schema locally.** Rejected because it creates two
   writers and couples persistence across bounded contexts.
3. **Use one optional worker reference with no affiliation invariant.** Rejected
   because it silently admits workforce data for non-employees and employees
   without a resolvable workforce identity.
4. **Selected: discriminate affiliation kind and enforce the ACL reference at
   aggregate construction.** This keeps workforce truth external while making
   invalid states unrepresentable in the first kernel.

## Evidence

- [Issue #2](https://github.com/ContextualWisdomLab/learning-management-platform/issues/2)
  defines the repository boundary and external-learner acceptance criteria.
- [Issue #3](https://github.com/ContextualWisdomLab/learning-management-platform/issues/3)
  defines the first Partner & Customer Academy vertical.
- [PR #23](https://github.com/ContextualWisdomLab/learning-management-platform/pull/23)
  carries the test-first implementation and exact-head evidence.

## Effects and risks

The product can model external access without inventing workforce records. The
opaque worker reference still needs a released Orgmetra ACL adapter before an
employee flow is deployable. Unix epoch seconds are intentionally a kernel
representation; database and API timestamp translation remains unselected.

## Concrete scenes

- A partner learner is accepted with `Partner` and no worker reference.
- An employee request without an Orgmetra reference is rejected before storage.
- A customer payload containing a worker reference is rejected as boundary
  leakage.
- A replay with an end instant equal to its start is rejected as ambiguous.

## Follow-up

Define PostgreSQL migrations, tenancy authorization, API idempotency, and the
Partner & Customer Academy UI in independent Proposed ADRs and PRs.

