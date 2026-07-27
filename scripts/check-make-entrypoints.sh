#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
REQUIRED=(
  Makefile
  examples/catalog/gallery.json
  scripts/check-gallery-catalog.jq
  scripts/example-preview.jq
  scripts/tests/example-index-contracts.sh
  scripts/write-examples-index.sh
)

for relative in "${REQUIRED[@]}"; do
  [[ -f "$ROOT/$relative" ]] || {
    echo "missing build entrypoint: $relative" >&2
    exit 1
  }
done

SHELL_FILES=()
while IFS= read -r file; do SHELL_FILES+=("$file"); done < <(
  find "$ROOT/scripts" -type f -name '*.sh' -print | LC_ALL=C sort
)
[[ ${#SHELL_FILES[@]} -gt 0 ]] || {
  echo "no shell entrypoints found" >&2
  exit 1
}
bash -n "${SHELL_FILES[@]}"
make -s -C "$ROOT" help >/dev/null
rg -q '^test-example-index:' "$ROOT/Makefile"
git -C "$ROOT" check-ignore -q -- examples-preview/.guard
git -C "$ROOT" check-ignore -q -- examples-preview.staging.123/.guard
jq -e -f "$ROOT/scripts/check-gallery-catalog.jq" \
  "$ROOT/examples/catalog/gallery.json" >/dev/null
jq --argjson edge 240 --argjson fps 12 --argjson window null \
  -f "$ROOT/scripts/example-preview.jq" >/dev/null <<'JSON'
{"project":{"sequences":[],"render_configs":[],"materials":[],"relations":[],"annotations":[]}}
JSON

for target in render_e2e_tests delivery_e2e_tests probe_e2e_tests workflow_e2e_tests; do
  [[ -f "$ROOT/crates/veac-runtime/tests/$target.rs" ]] || {
    echo "missing runtime E2E target: $target" >&2
    exit 1
  }
  rg -q -- "--test $target" "$ROOT/Makefile" || {
    echo "Makefile does not run runtime E2E target: $target" >&2
    exit 1
  }
done
[[ -f "$ROOT/crates/veac-cli/tests/cli_tests.rs" ]] || {
  echo "missing CLI E2E target: cli_tests" >&2
  exit 1
}
rg -q -- '--test cli_tests' "$ROOT/Makefile" || {
  echo "Makefile does not run CLI E2E target: cli_tests" >&2
  exit 1
}

for target in check-language-docs check-example-capabilities \
  test-example-capabilities check-examples build-examples serve-examples \
  clean-examples e2e; do
  rg -q -- "^$target:" "$ROOT/Makefile" || {
    echo "Makefile does not expose target: $target" >&2
    exit 1
  }
done

for target in test-e2e examples examples-check examples-serve examples-clean; do
  if rg -q -- "^$target:" "$ROOT/Makefile"; then
    echo "retired Make target remains: $target" >&2
    exit 1
  fi
  if rg -q --glob '!check-make-entrypoints.sh' \
    "\\bmake +$target\\b" "$ROOT/.github" "$ROOT/docs" \
    "$ROOT/README.md" "$ROOT/CONTRIBUTING.md" "$ROOT/scripts"; then
    echo "retired Make target is still referenced: $target" >&2
    exit 1
  fi
done

echo "Make and example-preview entrypoint checks passed."
