# Data model baseline

The authoritative database uses third normal form and two-or-more-word `snake_case` object names. Tenant-scoped relations carry `tenant_id`, and cross-tenant foreign-key relationships fail closed.

Initial entities:

- `learning_tenant`
- `login_identity_reference`
- `learner_profile`
- `learning_affiliation`
- `tenant_membership`
- `sponsor_account`
- `contracting_organization`
- `billing_account_reference`
- `representative_assignment`
- `consent_record`
- `access_entitlement`
- `course_catalog_record`
- `course_offering`
- `enrollment_record`
- `learning_registration`
- `learning_attempt`
- `completion_policy`
- `completion_policy_revision`
- `decision_evidence_reference`
- `completion_decision`
- `credential_record`

A learner is not assumed to be an employee, login account, payer, or contracting organization. The Rust domain kernel requires an opaque Orgmetra worker reference only when `affiliation_kind` is `employee` and rejects that reference for all non-employee kinds. The current PostgreSQL migration represents effective-dated `learning_affiliation` rows with `valid_from` and `valid_to` but does not yet persist the worker reference; that schema/API projection remains a dependent change. No employee row is synthesized for a non-employee learner.

`login_identity_reference` is a global opaque reference to an identity authority and external subject. `learner_profile` links one such identity to a stable learner, while `tenant_membership` grants tenant-scoped participation. This permits one identity to have memberships in several tenants without copying credentials or treating a login identity as an employee.

## Completion policy and decision relationships

`completion_policy` owns the stable policy identity. `completion_policy_revision` owns immutable revision content and has a many-to-one relationship to `completion_policy`. The pair `(tenant_id, completion_policy_id, revision_number)` is unique, and an accepted revision is never updated in place.

`decision_evidence_reference` stores only the external source authority, opaque snapshot/reference ID, immutable digest, observed source version, and an allowlisted `decision_time_metadata` object. That object is scalar-only and may contain only `decision_reason_code`, `decision_method`, `evaluated_at`, `policy_revision_reference`, and, for a correction, `correction_reason_code`; it may not contain evidence claims, credentials, LRS statements, psychometric responses, billing payloads, raw PII, or authoritative source payloads. It does **not** duplicate LRS statements, Psychometrics Commons result payloads, Studio content, or Billing provider truth.

`completion_decision` references exactly one `learning_registration` through a tenant-and-learner-scoped foreign key, exactly one `completion_policy_revision`, and one or more `decision_evidence_reference` rows through tenant-and-learner-scoped foreign keys. Entitlement-to-enrollment and enrollment-to-registration links use the same tenant-and-learner ownership boundary, so records inside one tenant cannot be joined across learners. A superseding decision is likewise constrained to the same tenant and learner. The tenant-scoped `completion_policy_revision` foreign key is the authoritative policy revision for replay and audit. If `decision_time_metadata.policy_revision_reference` is present on an attached evidence reference, it is a non-authoritative audit mirror and MUST resolve to exactly that same policy revision; absence is allowed, but mismatch is rejected fail-closed and must never select or override policy authority. A learning registration may have multiple decisions. A decision is immutable after publication; correction creates a superseding decision with an explicit relation to the prior decision.

Cardinality baseline:

```text
completion_policy 1 ---- * completion_policy_revision
completion_policy_revision 1 ---- * completion_decision
learning_registration 1 ---- * completion_decision
completion_decision 1 ---- * decision_evidence_reference
learner_profile 1 ---- * enrollment_record
enrollment_record 1 ---- * learning_registration
course_offering 1 ---- * enrollment_record
```

## External entitlement projection

`access_entitlement` is a versioned local reference/projection of an entitlement owned by the Billing Control Plane or another authorized entitlement authority. It stores the external entitlement reference, source authority, effective interval, and observed version/digest. It does not store provider payment objects or become the authoritative commercial permission record.

The executable registration path creates a `course_offering`, projects an active `access_entitlement` only for an active tenant membership, creates an `enrollment_record` only when the offering and entitlement are active for the same learner, and creates a `learning_registration` for that enrollment. An enrollment may be registered again after a prior registration closes; each tenant-scoped external registration reference remains unique so replay of the same authority fact fails closed. The migration uses tenant-scoped composite foreign keys and RLS for each relation; external billing and content payloads remain out of the database.

All authoritative facts are normalized to 3NF; repeated names, provider payloads, and external-system facts are referenced through dedicated identifiers rather than embedded denormalized copies.

The executable schema is `migrations/0001_learning_kernel.sql`. It applies composite tenant foreign keys, effective-dated affiliation exclusion, and PostgreSQL row-level security policies. The migration intentionally does not store source payloads from Keyverse, the LRS, Psychometrics Commons, Learning Content Studio, or Billing Control Plane.
