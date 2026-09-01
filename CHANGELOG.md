# Changelog

## Unreleased

- Clarify the bootstrap evidence metadata allowlist, registration-to-decision cardinality, and exact repository validation check for PR #1.
- Remove self-staling mutable PR-head evidence from the durable product/technical gap baseline; exact current integration evidence is resolved live from GitHub.
- Make the completion-policy revision authority explicit: the tenant-scoped `completion_policy_revision` foreign key is authoritative and any metadata mirror must match it or fail closed.

### Added

- Initial LMS authority and integration boundaries.
- First-class learner and enrollment data-model baseline covering both employee-linked and non-employee journeys.
- Standards adoption and operating-profile traceability.
- Product and technical gap baseline with the first buyer journey, modular target, delivery order, and evidence-authority rules.
- Repository development rules.
- Product-first README for buyers, maintainers, and integrators.
- Apache License 2.0 source grant for the repository foundation.
