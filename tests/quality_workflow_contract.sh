#!/usr/bin/env bash
set -euo pipefail

workflow=".github/workflows/quality.yml"

require_text() {
  local expected="$1"
  local failure="$2"

  if ! grep -Fq -- "$expected" "$workflow"; then
    printf '%s\n' "$failure" >&2
    exit 1
  fi
}

require_text "toolchain: 1.90.0" \
  "missing pinned stable Rust toolchain"
require_text "toolchain: nightly-2026-05-13" \
  "missing pinned nightly branch-coverage toolchain"
require_text "cargo +1.90.0 fmt --all --check" \
  "formatting must use the pinned stable toolchain"
require_text "cargo +1.90.0 clippy --all-targets --locked -- -D warnings" \
  "Clippy must use the pinned stable toolchain"
require_text "cargo +1.90.0 test --all-targets --locked" \
  "tests must use the pinned stable toolchain"
require_text "cargo +nightly-2026-05-13 llvm-cov --locked --branch" \
  "branch coverage must use the pinned nightly toolchain"
