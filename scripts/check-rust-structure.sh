#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
source "$SCRIPT_DIR/coverage-policy.sh"

"$SCRIPT_DIR/check-file-size.sh"
"$SCRIPT_DIR/tests/file-size-contracts.sh"
"$SCRIPT_DIR/tests/architecture-dependency-contracts.sh"
python3 "$SCRIPT_DIR/check-architecture-boundaries.py"
"$SCRIPT_DIR/tests/architecture-boundary-contracts.sh"
"$SCRIPT_DIR/tests/stdlib-codegen-contracts.sh"
"$SCRIPT_DIR/tests/project-backend-identity-contracts.sh"
"$SCRIPT_DIR/check-capability-evidence.sh"
"$SCRIPT_DIR/check-make-entrypoints.sh"
"$SCRIPT_DIR/tests/ci-workflow-contracts.sh"
"$SCRIPT_DIR/tests/install-script-contracts.sh"
"$SCRIPT_DIR/tests/release-smoke-contracts.sh"

status=0

while IFS= read -r -d '' file; do
  if [[ $(basename "$file") != "mod.rs" && -f "${file%.rs}/mod.rs" ]]; then
    echo "error: $file conflicts with ${file%.rs}/mod.rs" >&2
    status=1
  fi

  if ! is_coverage_test_source "$file"; then
    violations=$(awk '
      function reset() { attributes = ""; first_attribute = 0 }
      /^[[:space:]]*#\[/ {
        if (first_attribute == 0) first_attribute = NR
        attributes = attributes $0 "\n"
        next
      }
      /^[[:space:]]*$/ { next }
      {
        test_cfg = attributes ~ /#\[cfg[[:space:]]*\([[:space:]]*test[[:space:]]*\)\]/
        allowed = $0 ~ /^[[:space:]]*(pub(\([^)]*\)|\(crate\)|\(super\))?[[:space:]]+)?(mod|use)[[:space:]]/ ||
                  $0 ~ /^[[:space:]]*extern[[:space:]]+crate[[:space:]]/
        test_module = $0 ~ /mod[[:space:]]+(r#)?(tests?|unit_tests|test_support|test_[[:alnum:]_]*|[[:alnum:]_]*_(test|tests|test_support))[[:space:]]*;/
        test_path = attributes ~ /#\[path[[:space:]]*=[[:space:]]*"[^"]*(tests?|unit_tests|test_support)\// ||
                    attributes ~ /#\[path[[:space:]]*=[[:space:]]*"([^"]*\/)?(tests?|unit_tests|test_support|test_[^\/"]*|[^\/"]*_(test|tests|test_support))[.]rs"/
        if (test_cfg && !allowed) print first_attribute ": cfg(test) may only gate test modules or imports"
        if ((test_module || test_path) && !test_cfg) print NR ": coverage-excluded module is not cfg(test)-gated"
        reset()
      }
      END {
        if (attributes ~ /#\[cfg[[:space:]]*\([[:space:]]*test[[:space:]]*\)\]/) print first_attribute ": dangling cfg(test) attribute"
      }
    ' "$file")
    if [[ -n "$violations" ]]; then
      echo "error: test-only code is mixed into production coverage: $file" >&2
      echo "$violations" >&2
      status=1
    fi
    if rg -q '^[[:space:]]*#\[[[:space:]]*test[[:space:]]*\]|cfg[[:space:]]*![[:space:]]*\([[:space:]]*test[[:space:]]*\)|#\[cfg_attr[[:space:]]*\([[:space:]]*test' "$file"; then
      echo "error: test execution logic is outside a coverage-excluded module: $file" >&2
      status=1
    fi
  fi
done < <(find crates -type f -name '*.rs' -print0)

if matches=$(rg -n -U --glob '*.rs' '\binclude[[:space:]]*![[:space:]]*\(' crates); then
  echo "error: include! is not allowed for Rust module splitting:" >&2
  echo "$matches" >&2
  status=1
fi

exit "$status"
