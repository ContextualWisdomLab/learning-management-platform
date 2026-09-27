#!/usr/bin/env bash
set -euo pipefail

workflow="${WORKFLOW_PATH:-.github/workflows/quality.yml}"
rust_toolchain_action="dtolnay/rust-toolchain@6bed0761d98439e5a578e2877258200ad565ba87"

workflow_records() {
  awk '
    BEGIN {
      job_indent = -1
      with_indent = -1
    }
    function indentation(line) {
      match(line, /[^[:space:]]/)
      return RSTART ? RSTART - 1 : 0
    }
    function trim(value) {
      sub(/^[[:space:]]+/, "", value)
      sub(/[[:space:]]+$/, "", value)
      return value
    }
    function flush_step() {
      for (command_index = 1; command_index <= step_command_count; command_index++) {
        print "run\t" step_commands[command_index] "\t" \
          (step_non_gating ? "non-gating" : "gating") "\t" step_command_count
      }
      if (step_started && step_uses != "" && step_toolchain != "") {
        print "toolchain\t" step_uses "\t" step_toolchain
      }
      step_started = 0
      step_uses = ""
      step_toolchain = ""
      step_command_count = 0
      step_non_gating = 0
      with_indent = -1
    }
    {
      line_indent = indentation($0)

      if (in_run) {
        if ($0 ~ /^[[:space:]]*$/ || line_indent > run_indent) {
          command = trim($0)
          if (command != "" && command !~ /^#/) {
            step_command_count++
            step_commands[step_command_count] = command
          }
          next
        }
        in_run = 0
      }

      if (in_scalar) {
        if ($0 ~ /^[[:space:]]*$/ || line_indent > scalar_indent) {
          next
        }
        in_scalar = 0
      }

      if ($0 ~ /^[[:space:]]*$/ || $0 ~ /^[[:space:]]*#/) {
        next
      }

      if (in_steps && line_indent <= steps_indent) {
        flush_step()
        in_steps = 0
      }
      if (in_jobs && line_indent <= jobs_indent &&
          $0 !~ /^[[:space:]]*jobs:[[:space:]]*$/) {
        in_jobs = 0
        job_indent = -1
      }

      if (line_indent == 0 && $0 ~ /^jobs:[[:space:]]*$/) {
        in_jobs = 1
        jobs_indent = 0
        next
      }
      if (in_jobs && line_indent == jobs_indent + 2 &&
          $0 ~ /^[[:space:]]*[A-Za-z0-9_-]+:[[:space:]]*$/) {
        job_indent = line_indent
        next
      }
      if (job_indent >= 0 && line_indent == job_indent + 2 &&
          $0 ~ /^[[:space:]]*(continue-on-error|if):[[:space:]]*/) {
        value = $0
        sub(/^[[:space:]]*(continue-on-error|if):[[:space:]]*/, "", value)
        sub(/[[:space:]]*#.*/, "", value)
        if (tolower(trim(value)) != "false" || $0 ~ /^[[:space:]]*if:/) {
          print "quality jobs must be unconditional and gating" > "/dev/stderr"
          exit 1
        }
        next
      }
      if (job_indent >= 0 && line_indent == job_indent + 2 &&
          $0 ~ /^[[:space:]]*steps:[[:space:]]*$/) {
        in_steps = 1
        steps_indent = line_indent
        next
      }

      if (in_steps && line_indent == steps_indent + 2 &&
          $0 ~ /^[[:space:]]*-[[:space:]]+/) {
        flush_step()
        step_started = 1
        step_indent = line_indent
        step_non_gating = 0
        step_command_count = 0
        item = trim($0)
        if (item ~ /^-[[:space:]]+uses:[[:space:]]*/) {
          sub(/^-[[:space:]]+uses:[[:space:]]*/, "", item)
          step_uses = trim(item)
        }
        next
      }

      if (step_started && with_indent >= 0 && line_indent <= with_indent) {
        with_indent = -1
      }
      if (step_started && line_indent == step_indent + 2 &&
          $0 ~ /^[[:space:]]*uses:[[:space:]]*/) {
        value = $0
        sub(/^[[:space:]]*uses:[[:space:]]*/, "", value)
        step_uses = trim(value)
        next
      }
      if (step_started && line_indent == step_indent + 2 &&
          $0 ~ /^[[:space:]]*with:[[:space:]]*$/) {
        with_indent = line_indent
        next
      }
      if (step_started && line_indent == step_indent + 2 &&
          $0 ~ /^[[:space:]]*continue-on-error:[[:space:]]*/) {
        value = $0
        sub(/^[[:space:]]*continue-on-error:[[:space:]]*/, "", value)
        sub(/[[:space:]]*#.*/, "", value)
        if (tolower(trim(value)) != "false") {
          step_non_gating = 1
        }
        next
      }
      if (step_started && line_indent == step_indent + 2 &&
          $0 ~ /^[[:space:]]*if:[[:space:]]*/) {
        step_non_gating = 1
        next
      }
      if (step_started && with_indent >= 0 && line_indent == with_indent + 2 &&
          $0 ~ /^[[:space:]]*toolchain:[[:space:]]*/) {
        value = $0
        sub(/^[[:space:]]*toolchain:[[:space:]]*/, "", value)
        step_toolchain = trim(value)
        next
      }
      if (step_started && line_indent == step_indent + 2 &&
          $0 ~ /^[[:space:]]*run:[[:space:]]*\|[[:space:]]*$/) {
        in_run = 1
        run_indent = line_indent
        next
      }

      if ($0 ~ /:[[:space:]]*[|>][-+0-9]*[[:space:]]*(#.*)?$/) {
        in_scalar = 1
        scalar_indent = line_indent
      }
    }
    END { flush_step() }
  ' "$workflow"
}

require_run_command() {
  local expected="$1"
  local failure="$2"

  if ! workflow_records | awk -F '\t' -v expected="$expected" '
    $1 == "run" && $2 == expected && $3 == "gating" && $4 == 1 {
      found = 1
    }
    END { exit(found ? 0 : 1) }
  '; then
    printf '%s\n' "$failure" >&2
    exit 1
  fi
}

require_toolchain_step() {
  local expected="$1"
  local failure="$2"
  local expected_record

  expected_record="$(printf 'toolchain\t%s\t%s' "$rust_toolchain_action" "$expected")"

  if ! workflow_records | grep -Fqx -- "$expected_record"; then
    printf '%s\n' "$failure" >&2
    exit 1
  fi
}

require_toolchain_step "1.90.0" \
  "missing pinned stable Rust toolchain"
require_toolchain_step "nightly-2026-05-13" \
  "missing pinned nightly branch-coverage toolchain"
require_run_command "cargo +1.90.0 fmt --all --check" \
  "formatting must use the pinned stable toolchain"
require_run_command "cargo +1.90.0 clippy --all-targets --locked -- -D warnings" \
  "Clippy must use the pinned stable toolchain"
require_run_command "cargo +1.90.0 test --all-targets --locked" \
  "tests must use the pinned stable toolchain"
require_run_command "cargo +nightly-2026-05-13 llvm-cov --locked --branch --json --output-path target/llvm-cov.json" \
  "branch coverage must use the pinned nightly toolchain"
require_run_command "jq -e '.data[0].totals.lines.percent == 100 and .data[0].totals.branches.percent == 100' target/llvm-cov.json >/dev/null" \
  "line and branch coverage must both be complete"
require_run_command "git diff --exit-code" \
  "quality checks must not modify tracked files"
