# Agent Instructions

## Product boundary

This repository is the canonical writer for learner affiliation, enrollment,
progress, completion policy, and credential orchestration. Do not copy source,
schemas, databases, or mutable branch artifacts from Orgmetra, Keyverse,
learning-record-store, learning-interoperability-contracts, Psychometrics
Commons, or fast-mlsirm.

Until an upstream owner publishes an immutable release, use only a documented
port, feature flag, or test double. External identities are opaque references;
they are never reimplemented here.

## Change discipline

- Work test-first and keep every not-ready PR Draft.
- Never Force Push or destructively rebase.
- Public Rust items require rustdoc and owned production code requires 100%
  line and branch coverage.
- Keep ADR status Proposed until exact-head Checks and independent review pass.
- Update `CHANGELOG.md` and `docs/product-technical-gap-baseline.md` with every
  product delta.
- Use descriptive, two-or-more-word organization-owned identifiers.

## Verification

Run format, Clippy with warnings denied, tests, and LLVM coverage using the
commands encoded in `.github/workflows/quality.yml`. Organization-required
security workflows remain owned by `ContextualWisdomLab/.github`.
