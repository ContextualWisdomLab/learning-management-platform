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
non-false `continue-on-error`, an `if` condition, or an `exit` command. Jobs
containing the gate must also be unconditional and may use only explicit
`continue-on-error: false`; dynamic expressions therefore fail closed. The
contract itself runs in a separate required step. Twenty-one rejection
mutations prove comments, unrelated fields, nested block scalars, trailing shell
operators, conditional jobs or steps, early successful exits, and literal or
expression-based failure tolerance cannot impersonate the required gate. Bare
`exit;` is treated as an early exit even without a numeric status.
Explicit-false job and step controls prove the safe form remains accepted.
Non-gating state is monotonic, so an explicit-false property cannot override a
condition or early exit detected earlier in the same step.
Central required workflows provide independent security evidence.

Future database, HTTP, and UI slices require realistic PostgreSQL migration and
rollback tests, async load evidence, authorization tests, and browser E2E tests;
they are not implied by the current in-memory kernel.
