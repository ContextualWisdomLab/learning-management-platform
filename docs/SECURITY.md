# Security Boundary

- Fail closed on blank identifiers, invalid validity windows, and workforce ACL
  misuse.
- Treat tenant and learner identifiers as untrusted boundary input in future
  adapters.
- Never store identity credentials or billing secrets in this repository.
- Resolve workforce identity only through a released Orgmetra ACL; never query
  its database.
- Resolve authentication and federation through released Keyverse contracts.
- Keep synthetic identities in tests and anonymize operational evidence.

Threat modeling, tenant authorization, audit events, dependency review, SBOM,
provenance, and incident operations remain required before deployment.
