# ADR 0001: Learning-management authority boundary

## Status

Accepted

Approved by: ContextualWisdomLab repository owner  
Approval date: 2026-08-19

## Decision

The platform owns learning offerings, enrollment, learner affiliations, progression projections, and versioned completion decisions. Keyverse is the external authority for identity, authentication credentials, OIDC, federation, and SCIM; the LMS stores only stable external subject/reference identifiers and never copies credentials. Employment linkage, authored content, observed learning statements, assessment responses, and commercial payment/provider state remain external authorities.

## Consequences

Non-employee learners are supported without synthetic HR records. Completion decisions are reproducible from exact policy and evidence versions. Integrations remain replaceable behind versioned contracts. Identity lifecycle changes in Keyverse update linkage state rather than rewriting LMS learner identity or credential data.
