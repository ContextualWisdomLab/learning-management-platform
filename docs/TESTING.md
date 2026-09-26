# Test Strategy

Integration tests exercise the public crate API and cover successful external
and employee affiliations plus each fail-closed invariant. Synthetic identifiers
are used only in tests.

The Quality workflow verifies the exact submitted commit, formatting, Clippy
with warnings denied, all targets, a locked dependency graph, and 100% line and
branch coverage for owned production code. `quality_workflow_contract.sh`
requires formatting, Clippy, and tests to use stable Rust 1.90.0 explicitly and
requires unstable branch coverage to use the pinned nightly-2026-05-13
toolchain. Its bounded YAML structure scanner admits commands only from
`jobs.*.steps[]` step-level `run` blocks and admits toolchain pins only from
SHA-pinned `dtolnay/rust-toolchain` steps under `with.toolchain`. Required
quality commands must match exactly and run in steps without
`continue-on-error: true`; the contract itself runs in a separate required
step. Mutation tests prove comments, unrelated fields, nested block scalars,
trailing shell operators, and non-gating steps cannot impersonate the required
gate. Central required workflows provide independent security evidence.

Future database, HTTP, and UI slices require realistic PostgreSQL migration and
rollback tests, async load evidence, authorization tests, and browser E2E tests;
they are not implied by the current in-memory kernel.
