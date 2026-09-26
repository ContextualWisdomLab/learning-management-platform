#!/usr/bin/env bash
set -euo pipefail

workflow="${WORKFLOW_PATH:-.github/workflows/quality.yml}"

require_text() {
  local expected="$1"
  local failure="$2"

  if ! grep -Fq -- "$expected" "$workflow"; then
    printf '%s\n' "$failure" >&2
    exit 1
  fi
}

require_run_command() {
  local expected="$1"
  local failure="$2"

  if ! awk -v expected="$expected" '
    function indentation(line) {
      match(line, /[^[:space:]]/)
      return RSTART ? RSTART - 1 : 0
    }
    /^[[:space:]]*run:[[:space:]]*\|[[:space:]]*$/ {
      run_indent = indentation($0)
      in_run = 1
      next
    }
    in_run {
      if ($0 !~ /^[[:space:]]*$/ && indentation($0) <= run_indent) {
        in_run = 0
        next
      }
      command = $0
      sub(/^[[:space:]]+/, "", command)
      if (command !~ /^#/ && index(command, expected) == 1 &&
          (length(command) == length(expected) ||
           substr(command, length(expected) + 1, 1) == " ")) {
        found = 1
      }
    }
    END { exit(found ? 0 : 1) }
  ' "$workflow"; then
    printf '%s\n' "$failure" >&2
    exit 1
  fi
}

require_text "toolchain: 1.90.0" \
  "missing pinned stable Rust toolchain"
require_text "toolchain: nightly-2026-05-13" \
  "missing pinned nightly branch-coverage toolchain"
require_run_command "cargo +1.90.0 fmt --all --check" \
  "formatting must use the pinned stable toolchain"
require_run_command "cargo +1.90.0 clippy --all-targets --locked -- -D warnings" \
  "Clippy must use the pinned stable toolchain"
require_run_command "cargo +1.90.0 test --all-targets --locked" \
  "tests must use the pinned stable toolchain"
require_run_command "cargo +nightly-2026-05-13 llvm-cov --locked --branch" \
  "branch coverage must use the pinned nightly toolchain"
