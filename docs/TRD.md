# TRD — Learning Affiliation Kernel

## Selected design

A dependency-free Rust library owns the `LearningAffiliation` aggregate and
validates invariants at construction. Unix epoch seconds provide an explicit,
portable time representation for this pre-persistence slice. The future API and
database adapters must translate at the boundary rather than weakening the
aggregate.

## Interfaces

- Input: semantic identifiers, affiliation kind, optional opaque worker
  reference, inclusive start, optional exclusive end.
- Output: immutable affiliation or one typed, fail-closed error.
- External I/O: none.

## Quality and operations

Rust 1.90.0 is pinned. CI checks the exact submitted SHA, uses SHA-pinned GitHub
Actions, verifies the cargo-llvm-cov archive hash, denies Clippy warnings, and
requires 100% owned production line and branch coverage. Central organization
workflows own CodeQL, Semgrep, secret scanning, SBOM, and provenance gates.

## Deferred adapters

PostgreSQL, Keyverse, Orgmetra, LRS, billing, placement, and UI adapters are not
hidden inside this crate. Each requires a released upstream contract and its own
test-first slice.

