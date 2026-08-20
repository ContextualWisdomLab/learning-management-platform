# Assessment result reference v1

This is a CWL-owned boundary contract for importing an immutable assessment-result reference from Psychometrics Commons. It is not a QTI conformance claim and it never copies assessment responses, item data, scores, or psychometric payloads into the LMS.

The HTTP adapter accepts the schema at `contracts/assessment_result_reference/v1.schema.json` through:

```text
POST /v1/tenants/{tenant_id}/learners/{learner_id}/registrations/{learning_registration_id}/assessment-results
```

The route stores `evidence_kind=assessment`, the authority/reference/digest/version, the external result status, and the source observation time. Rust completion evaluation accepts assessment evidence only when its external status is `passed`; `failed` and `inconclusive` remain immutable evidence but cannot produce a completed decision.

QTI 3.0 remains the interoperability target for assessment content/results exchange. Psychometrics Commons remains the authority for assessment administration and scoring; this contract is the LMS reference boundary only. Provider-consumer contract tests and a released external client remain required before claiming an implemented cross-repository integration.
