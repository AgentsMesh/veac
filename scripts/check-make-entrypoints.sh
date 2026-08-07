#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)
source "$ROOT/scripts/example-preview-cli.sh"
# shellcheck source=coverage-policy.sh
source "$ROOT/scripts/coverage-policy.sh"
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
rg -q '^RUST_TOOLCHAIN \?= 1[.]85[.]0$' "$ROOT/Makefile"
install_dry_run=$(RUST_TOOLCHAIN=1.85.0 make -s -n -C "$ROOT" install)
[[ $install_dry_run == "cargo +1.85.0 install --path crates/veac-cli --locked --root \"$HOME/.local\"" ]]
all_examples_dry_run=$(make -s -n -C "$ROOT" build-examples)
rg -Fq 'VEAC_EXAMPLES=""' <<<"$all_examples_dry_run"
serve_dry_run=$(make -s -n -C "$ROOT" serve-examples)
rg -Fq 'python3 -m http.server "8000" --bind 127.0.0.1' <<<"$serve_dry_run"
rg -q '^test-example-index:' "$ROOT/Makefile"
rg -q '^test-example-render-contracts:' "$ROOT/Makefile"
rg -q '^build-examples: check-examples ' "$ROOT/Makefile"
rg -q '^[[:space:]]*@unset VEAC_BIN;' "$ROOT/Makefile"
rg -q '^PREVIEW_MAX_EDGE \?= 480$' "$ROOT/Makefile"
rg -q 'VEAC_PREVIEW_MAX_EDGE:-480' "$ROOT/scripts/build-examples.sh"
rg -q '^unset VEAC_BIN$' "$ROOT/scripts/build-examples.sh"
git -C "$ROOT" check-ignore -q -- examples-preview/.guard
git -C "$ROOT" check-ignore -q -- examples-preview.staging.123/.guard
jq -e -f "$ROOT/scripts/check-gallery-catalog.jq" \
  "$ROOT/examples/catalog/gallery.json" >/dev/null
awk -v minimum="$COVERAGE_MINIMUM" 'BEGIN { exit !(minimum > 95.02) }' || {
  echo "coverage threshold must be strictly greater than 95.02" >&2
  exit 1
}
for contract in example-preview example-preview-fixture example-preview-provenance \
  example-preview-build-flow example-preview-finalization \
  example-workflow-evidence render-evidence text-render-evidence \
  text-animation-render-evidence timing-render-evidence \
  media-smoke-render-evidence all-features-render-evidence \
  agentsmesh-intro-render-evidence advanced-color-render-evidence \
  visual-mechanism-render-evidence generated-graphics-render-evidence \
  masks-render-evidence mask-shape-render-evidence video-stabilization-render-evidence \
  video-effects-sharpen resolution-chain-render-evidence executable-family-render-evidence \
  workflow-showcase-render-evidence workflow-showcase-media \
  delivery-codec-render-evidence; do
  rg -q "bash scripts/tests/$contract-contracts.sh" "$ROOT/Makefile"
done

for target in language_docs_contract executable_docs_contract executable_temporal_docs \
  program_nominal_docs programming_components_docs_contract \
  nominal_source_edit_docs_contract; do
  rg -q -- "--test $target" "$ROOT/Makefile" || {
    echo "Makefile does not run language docs contract: $target" >&2
    exit 1
  }
done
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
VEAC_BIN="$tmp/stale-veac"
printf '#!/usr/bin/env bash\nexit 99\n' > "$VEAC_BIN"
chmod +x "$VEAC_BIN"
veac=$(prepare_example_preview_cli "$ROOT" 1.85.0)
[[ "$veac" == "$tmp/target/debug/veac" ]]
[[ $(wc -l < "$cargo_log" | tr -d ' ') -eq 2 ]]
[[ $(sed -n '1p' "$cargo_log") == \
  "+1.85.0 build --manifest-path $ROOT/Cargo.toml --package veac-cli --bin veac" ]]
[[ $(sed -n '2p' "$cargo_log") == \
  "+1.85.0 metadata --manifest-path $ROOT/Cargo.toml --format-version 1 --no-deps" ]]
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
[[ $(rg -c -- '-- --test-threads=1' "$ROOT/scripts/coverage.sh") -eq 2 ]] || {
  echo "coverage entrypoints must serialize every test target" >&2
  exit 1
}
[[ -f "$ROOT/crates/veac-cli/tests/cli_tests.rs" ]] || {
  echo "missing CLI E2E target: cli_tests" >&2
  exit 1
}
rg -q -- '--test cli_tests' "$ROOT/Makefile" || {
  echo "Makefile does not run CLI E2E target: cli_tests" >&2
  exit 1
}

for target in doctor install build structure fmt-check check clippy test coverage coverage-packages verify \
  check-language-docs check-stdlib-codegen test-stdlib-codegen check-example-capabilities \
  test-example-capabilities check-examples build-examples serve-examples \
  clean-examples e2e e2e-executable; do
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
