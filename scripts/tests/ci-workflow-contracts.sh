#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
MAKEFILE="$ROOT/Makefile"
CI="$ROOT/.github/workflows/ci.yml"
FULL="$ROOT/.github/workflows/example-previews.yml"
RELEASE="$ROOT/.github/workflows/release.yml"
RELEASE_SMOKE="$ROOT/.github/scripts/smoke-release-archive.sh"
CATALOG="$ROOT/examples/catalog/gallery.json"
EXPECTED_SMOKE="minimal,all-features,executable-mechanisms,text-overlay,delivery-codec-matrix,programming-language"

fail() {
  echo "CI workflow contract failed: $*" >&2
  exit 1
}

for file in "$MAKEFILE" "$CI" "$FULL" "$RELEASE" "$RELEASE_SMOKE" "$CATALOG"; do
  [[ -f $file ]] || fail "missing file: ${file#"$ROOT"/}"
done

rg -q 'uses: dtolnay/rust-toolchain@1[.]85[.]0$' "$RELEASE" ||
  fail "release Rust toolchain must be pinned to 1.85.0"
if rg -q 'rust-toolchain@(stable|beta|nightly|master)' "$RELEASE"; then
  fail "release Rust toolchain must not track a moving channel"
fi
rg -q 'cargo install cross --version 0[.]2[.]5 --locked$' "$RELEASE" ||
  fail "cross must be installed from one locked release"
[[ $(rg -c 'build --locked --release --bin' "$RELEASE") -eq 2 ]] ||
  fail "both release build paths must consume the committed lockfile"
if rg -q 'cargo install cross --git|cp README[.]md LICENSE .*\|\| true' "$RELEASE"; then
  fail "release inputs must not be floating or optional"
fi

smoke=$(sed -n 's/^EXAMPLE_SMOKE_SET := //p' "$MAKEFILE")
[[ $smoke == "$EXPECTED_SMOKE" ]] || fail "unexpected example smoke set: $smoke"
IFS=, read -r -a examples <<<"$smoke"
for example in "${examples[@]}"; do
  jq -e --arg id "$example" 'any(.targets[]; .id == $id and .example != null)' \
    "$CATALOG" >/dev/null || fail "uncataloged smoke example: $example"
done

rg -Fq 'build-examples-smoke: EXAMPLES := $(EXAMPLE_SMOKE_SET)' "$MAKEFILE" ||
  fail "smoke target must bind the selected example set"
rg -q '^build-examples-smoke: check-examples-static _build-examples ' "$MAKEFILE" ||
  fail "smoke target must use static checks and the shared preview builder"
dry_run=$(make -s -n -C "$ROOT" build-examples-smoke)
rg -Fq "VEAC_EXAMPLES=\"$EXPECTED_SMOKE\"" <<<"$dry_run" ||
  fail "smoke target does not pass its selection to the preview builder"
if rg -q 'render-evidence-contracts|test-example-render-contracts' <<<"$dry_run"; then
  fail "smoke target must not repeat heavyweight render-evidence contracts"
fi

ffmpeg_job=$(awk '
  $0 == "  ffmpeg-integration:" { found = 1 }
  found && $0 ~ /^  [[:alnum:]_-]+:$/ && $0 != "  ffmpeg-integration:" { exit }
  found { print }
' "$CI")
lint_job=$(awk '
  $0 == "  lint:" { found = 1 }
  found && $0 ~ /^  [[:alnum:]_-]+:$/ && $0 != "  lint:" { exit }
  found { print }
' "$CI")
rg -q 'run: make structure$' <<<"$lint_job" ||
  fail "ordinary CI must run the complete structure and language-doc contract"
rg -q 'apt-get install --yes .*jq.*ripgrep' <<<"$lint_job" ||
  fail "lint must install the shell-contract dependencies"
if rg -q 'run: bash scripts/check-rust-structure[.]sh$' <<<"$lint_job"; then
  fail "ordinary CI must not bypass the Make structure contract"
fi
structure_dry_run=$(make -s -n -C "$ROOT" structure)
for contract in scripts/check-rust-structure.sh scripts/check-language-docs.sh; do
  rg -Fq "$contract" <<<"$structure_dry_run" ||
    fail "make structure omits $contract"
done
rg -q '^[[:space:]]+name: FFmpeg integration$' <<<"$ffmpeg_job" ||
  fail "required FFmpeg job identity changed"
rg -Uq 'uses: actions/checkout@v4\n[[:space:]]+with:\n[[:space:]]+lfs: true' \
  <<<"$ffmpeg_job" || fail "FFmpeg CI must materialize Git LFS media"
rg -q 'apt-get install --yes .*jq.*ripgrep' <<<"$ffmpeg_job" ||
  fail "example smoke CI must install the preview-tool dependencies"
rg -q 'run: make build-examples-smoke$' <<<"$ffmpeg_job" ||
  fail "required CI must build only the smoke example set"
if rg -q 'build-examples([^a-zA-Z0-9_-]|$)' <<<"$ffmpeg_job"; then
  fail "required CI must not build every example"
fi
rg -q '^[[:space:]]+timeout-minutes: 35$' <<<"$ffmpeg_job" ||
  fail "required FFmpeg CI must have a bounded timeout"

e2e_dry_run=$(make -s -n -C "$ROOT" e2e)
for target in temporal_curve_source_e2e component_temporal_source_e2e plugin_source_e2e \
  executable_provenance for_each_source_e2e executable_local_image_e2e \
  executable_centered_dissolve_e2e executable_audio_caption_e2e; do
  rg -Fq -- "--test $target" <<<"$e2e_dry_run" ||
    fail "make e2e omits source-to-pixel target: $target"
done
[[ $(rg -c -- '-- --ignored --test-threads=1' <<<"$e2e_dry_run") -eq 3 ]] ||
  fail "make e2e must run exactly three focused ignored executable guards"
rg -q 'run: make e2e$' <<<"$ffmpeg_job" ||
  fail "ordinary FFmpeg CI must run the complete make e2e contract"
if rg -q 'run: make build-examples$' <<<"$ffmpeg_job"; then
  fail "ordinary CI must not render all examples"
fi

rg -q '^  workflow_dispatch:$' "$FULL" || fail "full previews must be manually runnable"
rg -q '^  schedule:$' "$FULL" || fail "full previews must run on a weekly schedule"
if rg -q '^  (push|pull_request):' "$FULL"; then
  fail "full previews must not run for pushes or pull requests"
fi
rg -Uq 'uses: actions/checkout@v4\n[[:space:]]+with:\n[[:space:]]+lfs: true' "$FULL" ||
  fail "full previews must materialize Git LFS media"
rg -q 'run: make build-examples$' "$FULL" ||
  fail "full preview workflow must render every example"
rg -q 'apt-get install --yes .*jq.*ripgrep' "$FULL" ||
  fail "full previews must install the preview-tool dependencies"
rg -q '^[[:space:]]+path: examples-preview/$' "$FULL" ||
  fail "full preview workflow must publish the complete gallery"

release_smoke=$(awk '
  $0 == "  smoke:" { found = 1 }
  found && $0 ~ /^  [[:alnum:]_-]+:$/ && $0 != "  smoke:" { exit }
  found { print }
' "$RELEASE")
release_job=$(awk '
  $0 == "  release:" { found = 1 }
  found && $0 ~ /^  [[:alnum:]_-]+:$/ && $0 != "  release:" { exit }
  found { print }
' "$RELEASE")
rg -q '^[[:space:]]+needs: build$' <<<"$release_smoke" ||
  fail "packaged smoke must depend on every build artifact"
rg -q '^[[:space:]]+needs: smoke$' <<<"$release_job" ||
  fail "release publication must depend on packaged smoke"
rg -q 'smoke-release-archive[.]sh' <<<"$release_smoke" ||
  fail "packaged smoke helper is not executed"
if rg -q 'continue-on-error|make build-examples' <<<"$release_smoke"; then
  fail "release smoke must fail closed and remain independent from examples"
fi

for contract in \
  'x86_64-unknown-linux-gnu[[:space:]]*\n[[:space:]]*runner: ubuntu-latest' \
  'aarch64-unknown-linux-gnu[[:space:]]*\n[[:space:]]*runner: ubuntu-24.04-arm' \
  'x86_64-apple-darwin[[:space:]]*\n[[:space:]]*runner: macos-15-intel' \
  'aarch64-apple-darwin[[:space:]]*\n[[:space:]]*runner: macos-15'; do
  rg -Uq "$contract" <<<"$release_smoke" ||
    fail "release smoke is missing native runner contract: $contract"
done
[[ $(rg -c '^[[:space:]]+- target:' <<<"$release_smoke") -eq 4 ]] ||
  fail "release smoke must cover exactly four packaged targets"
# These probes intentionally match literal shell source.
# shellcheck disable=SC2016
for command in '"$binary" --help' '"$binary" language-spec' 'language-spec --schema'; do
  rg -Fq "$command" "$RELEASE_SMOKE" || fail "release smoke omits: $command"
done

echo "CI workflow contracts passed"
