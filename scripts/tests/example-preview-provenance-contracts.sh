#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)
source "$ROOT/scripts/example-preview-provenance.sh"
source "$ROOT/scripts/tests/example-preview-layout-fixtures.sh"
FILTER="$ROOT/scripts/example-preview.jq"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
ENTRY="$TMP/demo"

fail() {
  echo "preview provenance contract failed: $*" >&2
  exit 1
}

expect_failure() {
  local label=$1
  shift
  if ("$@") >/dev/null 2>&1; then
    fail "$label unexpectedly passed"
  fi
}

prepare_preview_fixture_dirs "$ENTRY"
cat >"$ENTRY/project/project.veac.json" <<'JSON'
{"project":{"id":"prj_demo","sequences":[{"id":"seq_main","settings":{"frame_rate":{"numerator":30,"denominator":1}},"tracks":[],"applies":[]}],"relations":[],"annotations":[],"render_configs":[{"id":"out_preview","sequence_id":"seq_main","raster":{"width":1280,"height":720,"frame_rate":{"numerator":30,"denominator":1}},"deliverables":[{"id":"dlv_preview","target":{"type":"file","name":"preview.mp4"},"kind":{"type":"video","settings":{"hardware":{"type":"auto"}}}}]}]}}
JSON
cp "$ENTRY/project/project.veac.json" "$TMP/preview-input.json"
jq --argjson edge 480 --argjson fps 12 --argjson window null \
  -f "$FILTER" "$TMP/preview-input.json" >"$ENTRY/project/project.preview.veac.json"
cat >"$ENTRY/plans/preview/out_preview.json" <<'JSON'
{"output":{"render_config_id":"out_preview"}}
JSON

verify_preview_derivation "$TMP/preview-input.json" \
  "$ENTRY/project/project.preview.veac.json" "$FILTER" 480 12 null
verify_example_preview_layout "$ENTRY"

cp "$ENTRY/project/project.preview.veac.json" "$TMP/preview.good.json"
jq '.project.render_configs[0].raster.width = 478' "$TMP/preview.good.json" \
  >"$ENTRY/project/project.preview.veac.json"
expect_failure changed_derivative verify_preview_derivation "$TMP/preview-input.json" \
  "$ENTRY/project/project.preview.veac.json" "$FILTER" 480 12 null
cp "$TMP/preview.good.json" "$ENTRY/project/project.preview.veac.json"

jq '.project.render_configs[0].deliverables[0].target.name = "other.mp4"' \
  "$TMP/preview.good.json" >"$ENTRY/project/project.preview.veac.json"
expect_failure changed_output_identity verify_canonical_roles \
  "$ENTRY/project/project.veac.json" "$ENTRY/project/project.preview.veac.json"
cp "$TMP/preview.good.json" "$ENTRY/project/project.preview.veac.json"

mv "$ENTRY/plans/preview/out_preview.json" "$TMP/plan.json"
expect_failure missing_plan verify_example_preview_layout "$ENTRY"
mv "$TMP/plan.json" "$ENTRY/plans/preview/out_preview.json"
printf '{}\n' >"$ENTRY/plans/preview/extra.json"
expect_failure extra_plan verify_example_preview_layout "$ENTRY"
rm "$ENTRY/plans/preview/extra.json"

jq '.output.render_config_id = "out_wrong"' "$ENTRY/plans/preview/out_preview.json" \
  >"$TMP/plan.bad.json"
mv "$TMP/plan.bad.json" "$ENTRY/plans/preview/out_preview.json"
expect_failure wrong_plan_config verify_example_preview_layout "$ENTRY"
printf '{"output":{"render_config_id":"out_preview"}}\n' \
  >"$ENTRY/plans/preview/out_preview.json"

printf '{}\n' >"$ENTRY/plan.json"
expect_failure legacy_plan verify_example_preview_layout "$ENTRY"
rm "$ENTRY/plan.json"
printf '{}\n' >"$ENTRY/project/project.raw.json"
expect_failure legacy_raw verify_example_preview_layout "$ENTRY"
rm "$ENTRY/project/project.raw.json"
printf '{}\n' >"$ENTRY/project/.project.preview-input.veac.json"
expect_failure preview_input_scratch verify_example_preview_layout "$ENTRY"
rm "$ENTRY/project/.project.preview-input.veac.json"

mv "$ENTRY/project/project.preview.veac.json" "$TMP/preview.json"
ln -s "$TMP/preview.json" "$ENTRY/project/project.preview.veac.json"
expect_failure linked_preview verify_example_preview_layout "$ENTRY"
rm "$ENTRY/project/project.preview.veac.json"
mv "$TMP/preview.json" "$ENTRY/project/project.preview.veac.json"

if example_preview_plan "$ENTRY" '../escape' >/dev/null 2>&1; then
  fail "unsafe plan config ID was accepted"
fi

echo "example preview provenance contracts passed"
