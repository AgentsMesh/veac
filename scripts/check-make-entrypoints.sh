#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
source "$ROOT/scripts/example-preview-cli.sh"
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
rg -q '^build-examples: check-examples ' "$ROOT/Makefile"
rg -q '^PREVIEW_MAX_EDGE \?= 480$' "$ROOT/Makefile"
rg -q 'VEAC_PREVIEW_MAX_EDGE:-480' "$ROOT/scripts/build-examples.sh"
git -C "$ROOT" check-ignore -q -- examples-preview/.guard
git -C "$ROOT" check-ignore -q -- examples-preview.staging.123/.guard
jq -e -f "$ROOT/scripts/check-gallery-catalog.jq" \
  "$ROOT/examples/catalog/gallery.json" >/dev/null
jq --argjson edge 240 --argjson fps 12 --argjson window null \
  -f "$ROOT/scripts/example-preview.jq" >/dev/null <<'JSON'
{"project":{"sequences":[],"render_configs":[],"materials":[],"relations":[],"annotations":[]}}
JSON

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
cargo_log="$tmp/cargo.log"
cargo() {
  printf '%s\n' "$*" >> "$cargo_log"
  case " $* " in
    *' build '*) ;;
    *' metadata '*) printf '{"target_directory":"%s"}\n' "$tmp/target" ;;
    *) return 64 ;;
  esac
}
unset VEAC_BIN
veac=$(prepare_example_preview_cli "$ROOT" 1.85.0)
[[ "$veac" == "$tmp/target/debug/veac" ]]
[[ $(wc -l < "$cargo_log" | tr -d ' ') -eq 2 ]]
[[ $(sed -n '1p' "$cargo_log") == \
  "+1.85.0 build --manifest-path $ROOT/Cargo.toml --package veac-cli --bin veac" ]]
[[ $(sed -n '2p' "$cargo_log") == \
  "+1.85.0 metadata --manifest-path $ROOT/Cargo.toml --format-version 1 --no-deps" ]]

: > "$cargo_log"
VEAC_BIN="$tmp/custom-veac"
veac=$(prepare_example_preview_cli "$ROOT" 1.85.0)
[[ "$veac" == "$VEAC_BIN" ]]
[[ ! -s "$cargo_log" ]]
unset VEAC_BIN

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
