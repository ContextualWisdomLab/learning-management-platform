# Data model baseline

The authoritative database uses third normal form and two-or-more-word `snake_case` object names.

Initial entities:

- `learning_tenant`
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
- `completion_decision`
- `credential_record`

A learner is not assumed to be an employee, login account, payer, or contracting organization. Employment linkage is optional and effective-dated. Completion decisions reference the exact policy revision and evidence snapshots used at decision time.
