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
- `progress_projection`
- `completion_policy`
- `completion_policy_revision`
- `decision_evidence_reference`
- `completion_decision`
- `credential_record`
- `audit_event_record`

A learner is not assumed to be an employee, login account, payer, or contracting organization. Optional employment linkage is represented as an effective-dated `learning_affiliation` or external worker reference with `valid_from` and `valid_to`; no employee row is synthesized for a non-employee learner.

`login_identity_reference` is a global opaque reference to an identity authority and external subject. `learner_profile` links one such identity to a stable learner, while `tenant_membership` grants tenant-scoped participation. This permits one identity to have memberships in several tenants without copying credentials or treating a login identity as an employee.

## Completion policy and decision relationships

`completion_policy` owns the stable policy identity. `completion_policy_revision` owns immutable revision content and has a many-to-one relationship to `completion_policy`. The pair `(tenant_id, completion_policy_id, revision_number)` is unique, and an accepted revision is never updated in place.

`decision_evidence_reference` stores only the external source authority, opaque snapshot/reference ID, immutable digest, observed source version, assessment result status when the kind is `assessment`, registration binding, idempotency key for assessment imports, and decision-time metadata. It does **not** duplicate LRS statements, Psychometrics Commons result payloads, Studio content, or Billing provider truth. Rust completion evaluation accepts assessment evidence only when the external status is `passed`.

`completion_decision` has exactly one `learning_registration`, exactly one `completion_policy_revision`, and one or more `decision_evidence_reference` rows through tenant-scoped foreign keys. A registration may have multiple superseding decisions, but each decision is immutable after publication; a completion request may supply `supersedes_decision_id` only for an earlier decision in the same tenant, learner, and registration, creating a new decision with an explicit relation to the prior decision.

`credential_record` is a tenant-scoped reference projection issued only from a completed registration and its exact completion decision. It stores the credential authority, opaque external credential reference, lifecycle status, and issue/revocation timestamps; it does not store a badge payload or become the Open Badges/CLR authority. The four-column decision foreign key prevents a credential from combining a learner, registration, and decision from different rows.

`audit_event_record` is an append-only, tenant-scoped provenance record. It stores an opaque service actor, correlation UUID, action and entity identity, source authority/version, event digest, and occurrence time; it never copies learner, assessment, credential, or provider payloads. Row-level security and a mutation-rejecting trigger protect the record in addition to the API transaction boundary.

Cardinality baseline:

```text
completion_policy 1 ---- * completion_policy_revision
completion_policy_revision 1 ---- * completion_decision
learning_registration 1 ---- * completion_decision
learning_registration 1 ---- * decision_evidence_reference
completion_decision 1 ---- * decision_evidence_reference
completion_decision 1 ---- * credential_record
learning_registration 1 ---- * credential_record
learner_profile 1 ---- * enrollment_record
enrollment_record 1 ---- * learning_registration
course_offering 1 ---- * enrollment_record
```

## External entitlement projection

`access_entitlement` is a versioned local reference/projection of an entitlement owned by the Billing Control Plane or another authorized entitlement authority. It stores the external entitlement reference, source authority, effective interval, and observed version/digest. It does not store provider payment objects or become the authoritative commercial permission record.

The executable registration path creates a `course_offering`, projects an active `access_entitlement` only for an active tenant membership, creates an `enrollment_record` only when the offering and entitlement are active for the same learner, and creates one `learning_registration` for that enrollment. The migration uses tenant-scoped composite foreign keys and RLS for each relation; external billing and content payloads remain out of the database.

`learning_attempt` records a learner's launch against a registration and immutable content-release reference. `progress_projection` stores only the LRS authority, opaque activity reference, source version/digest, observed time, and a bounded progress value/state; it never stores the source activity payload. Repeated source versions are idempotent when newer or equal observations arrive, while older observations are rejected so progress cannot move backward silently.

Completion is persisted only after the Rust policy engine evaluates a tenant-scoped registration, exact policy revision, and immutable evidence references. `completion_decision` stores the replay fingerprint and links its evidence through `completion_decision_evidence`; it does not copy the LRS or assessment payload.

Credential issuance is a separate projection step: the API accepts an external authority/reference after completion, and PostgreSQL enforces the tenant, learner, registration, and decision relationship. Repeating an authority/reference is a conflict rather than a second local credential record. Revocation is an idempotent lifecycle transition that records `revoked_at`; the external credential authority remains responsible for the portable credential payload and final status.

All authoritative facts are normalized to 3NF; repeated names, provider payloads, and external-system facts are referenced through dedicated identifiers rather than embedded denormalized copies.

The executable schema is `migrations/0001_learning_kernel.sql`. It applies composite tenant foreign keys, effective-dated affiliation exclusion, and PostgreSQL row-level security policies. The migration intentionally does not store source payloads from Keyverse, the LRS, Psychometrics Commons, Learning Content Studio, or Billing Control Plane.
