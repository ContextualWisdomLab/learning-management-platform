# Learning Management Platform

Learning Management Platform is the ContextualWisdomLab system of record for
tenant-scoped learner affiliations, enrollments, progress, completion policy,
and credential orchestration. This repository is at foundation stage: its first
domain slice models every declared non-employee affiliation without requiring
workforce identity.

The current code is a dependency-free Rust library. It does not implement an
HTTP API, database adapter, or user interface yet.

## Development

The pinned toolchain is Rust 1.90.0.

```sh
cargo fmt --all --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --all-targets --locked
```

Architecture, ownership boundaries, and delivery status are maintained in
[`ARCHITECTURE.md`](ARCHITECTURE.md) and
[`docs/product-technical-gap-baseline.md`](docs/product-technical-gap-baseline.md).
