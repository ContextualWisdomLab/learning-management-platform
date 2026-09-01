---
title: Learning Management Platform
---

# Learning Management Platform

Learning Management Platform is a standards-oriented learning system for employees and external learners. It owns offerings, learner affiliations, enrollment, registrations, completion policy, and versioned completion decisions without forcing learning identity into an HR record.

## Start here

The current repository is a product and architecture foundation rather than a claimed production LMS deployment. Start with the [README](https://github.com/ContextualWisdomLab/learning-management-platform#readme) for the buyer-facing product boundary, first end-to-end journey, ecosystem ownership model, and repository verification guidance.

## Product responsibility

The platform owns learning tenants and learner profiles, learner affiliations independent of employment, catalog/offering projections, enrollment and registration state, progress projections, immutable completion-policy revisions, versioned completion decisions, credential orchestration state, and integration/audit boundaries.

Identity, employment, authored content releases, observed learning activity, psychometric evidence, and commercial entitlement remain authoritative in their owning products and are consumed through explicit versioned contracts rather than cross-repository database reads.

## Documentation

- [README](https://github.com/ContextualWisdomLab/learning-management-platform#readme) — product value, boundaries, journey, and maturity.
- [Product requirements](https://github.com/ContextualWisdomLab/learning-management-platform/blob/develop/docs/PRD.md) — product scope, users, jobs, and non-goals.
- [Technical requirements](https://github.com/ContextualWisdomLab/learning-management-platform/blob/develop/docs/TRD.md) — technical target and implementation constraints.
- [Architecture](https://github.com/ContextualWisdomLab/learning-management-platform/blob/develop/ARCHITECTURE.md) — system context and authority boundaries.
- [Product and technical gap baseline](https://github.com/ContextualWisdomLab/learning-management-platform/blob/develop/docs/product-technical-gap-baseline.md) — evidence-backed current gaps.
- [Releases](https://github.com/ContextualWisdomLab/learning-management-platform/releases) — published release history when available.
- [Ask DeepWiki](https://deepwiki.com/ContextualWisdomLab/learning-management-platform) — repository-grounded questions and code navigation.

## Evidence boundary

Protected `develop` remains repository authority. Architecture and PRD scope do not establish a running API, database implementation, browser journey, standards conformance, production deployment, customer deployment, or released artifact unless current protected-branch and live operational evidence independently proves it.

This file is a public documentation landing source. GitHub Pages publication is a separate repository-facing state and must be verified live before it is claimed available.
