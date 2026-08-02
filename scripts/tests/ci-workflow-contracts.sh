#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
MAKEFILE="$ROOT/Makefile"
CI="$ROOT/.github/workflows/ci.yml"
FULL="$ROOT/.github/workflows/example-previews.yml"
CATALOG="$ROOT/examples/catalog/gallery.json"
EXPECTED_SMOKE="minimal,all-features,executable-mechanisms,text-overlay,delivery-codec-matrix,programming-language"

fail() {
  echo "CI workflow contract failed: $*" >&2
  exit 1
}

for file in "$MAKEFILE" "$CI" "$FULL" "$CATALOG"; do
  [[ -f $file ]] || fail "missing file: ${file#"$ROOT"/}"
done

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
rg -q '^[[:space:]]+name: FFmpeg integration$' <<<"$ffmpeg_job" ||
  fail "required FFmpeg job identity changed"
rg -q 'run: make build-examples-smoke$' <<<"$ffmpeg_job" ||
  fail "required CI must build only the smoke example set"
if rg -q 'build-examples([^a-zA-Z0-9_-]|$)' <<<"$ffmpeg_job"; then
  fail "required CI must not build every example"
fi
rg -q '^[[:space:]]+timeout-minutes: 35$' <<<"$ffmpeg_job" ||
  fail "required FFmpeg CI must have a bounded timeout"

rg -q '^  workflow_dispatch:$' "$FULL" || fail "full previews must be manually runnable"
rg -q '^  schedule:$' "$FULL" || fail "full previews must run on a weekly schedule"
if rg -q '^  (push|pull_request):' "$FULL"; then
  fail "full previews must not run for pushes or pull requests"
fi
rg -q 'run: make build-examples$' "$FULL" ||
  fail "full preview workflow must render every example"
rg -q '^[[:space:]]+path: examples-preview/$' "$FULL" ||
  fail "full preview workflow must publish the complete gallery"

echo "CI workflow contracts passed"
