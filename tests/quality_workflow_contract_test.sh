#!/usr/bin/env bash
set -euo pipefail

workflow=".github/workflows/quality.yml"

for command in \
  "cargo +1.90.0 fmt --all --check" \
  "cargo +1.90.0 clippy --all-targets --locked -- -D warnings" \
  "cargo +1.90.0 test --all-targets --locked" \
  "cargo +nightly-2026-05-13 llvm-cov --locked --branch"; do
  mutated_workflow="$(mktemp)"
  sed "s|^          ${command}|          # ${command}|" "$workflow" > "$mutated_workflow"
  printf '\nx-command-decoy: "%s"\n' "$command" >> "$mutated_workflow"

  if WORKFLOW_PATH="$mutated_workflow" bash tests/quality_workflow_contract.sh \
    >/dev/null 2>&1; then
    printf 'contract accepted a command outside an executable run block: %s\n' "$command" >&2
    rm -f "$mutated_workflow"
    exit 1
  fi

  rm -f "$mutated_workflow"
done

mutated_workflow="$(mktemp)"
sed 's|^          cargo +1.90.0 fmt --all --check|          # cargo +1.90.0 fmt --all --check|' \
  "$workflow" > "$mutated_workflow"
cat >> "$mutated_workflow" <<'EOF'

      - name: Run block scalar decoy
        env:
          COMMAND_DECOY: |
            run: |
              cargo +1.90.0 fmt --all --check
        run: echo "$COMMAND_DECOY"
EOF

if WORKFLOW_PATH="$mutated_workflow" bash tests/quality_workflow_contract.sh \
  >/dev/null 2>&1; then
  printf '%s\n' 'contract accepted run syntax nested inside an env block scalar' >&2
  rm -f "$mutated_workflow"
  exit 1
fi
rm -f "$mutated_workflow"

for toolchain in "1.90.0" "nightly-2026-05-13"; do
  mutated_workflow="$(mktemp)"
  sed "0,/^          toolchain: ${toolchain}$/s//          toolchain: decoy-toolchain/" \
    "$workflow" > "$mutated_workflow"
  cat >> "$mutated_workflow" <<EOF

      - name: Toolchain block scalar decoy
        env:
          TOOLCHAIN_DECOY: |
            toolchain: ${toolchain}
        run: echo \"\$TOOLCHAIN_DECOY\"
EOF

  if WORKFLOW_PATH="$mutated_workflow" bash tests/quality_workflow_contract.sh \
    >/dev/null 2>&1; then
    printf 'contract accepted a toolchain value outside a rust-toolchain step: %s\n' \
      "$toolchain" >&2
    rm -f "$mutated_workflow"
    exit 1
  fi

  rm -f "$mutated_workflow"
done
