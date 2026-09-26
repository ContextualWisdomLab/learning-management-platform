# Test Strategy

Integration tests exercise the public crate API and cover successful external
and employee affiliations plus each fail-closed invariant. Synthetic identifiers
are used only in tests.

The Quality workflow verifies the exact submitted commit, formatting, Clippy
with warnings denied, all targets, a locked dependency graph, and 100% line and
branch coverage for owned production code. Central required workflows provide
independent security evidence.

Future database, HTTP, and UI slices require realistic PostgreSQL migration and
rollback tests, async load evidence, authorization tests, and browser E2E tests;
they are not implied by the current in-memory kernel.

