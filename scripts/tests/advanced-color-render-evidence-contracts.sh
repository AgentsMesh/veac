#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/veac-advanced-color-contracts.XXXXXX")
trap 'rm -rf "$TMP"' EXIT

for module in common pixels mechanism-pixels visual-mechanisms advanced-color-stages; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/advanced-color-render-fixtures.sh"
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/visual-mechanism-video-fixtures.sh"

clone_fixture() {
  local root=$1
  mkdir -p "$root"
  cp -R "$VALID/advanced-color" "$root/advanced-color"
}

expect_failure() {
  local label=$1 root=$2 expected=$3 log="$TMP/$1.log"
  if (PREVIEW_ROOT=$root check_advanced_color_stage_evidence) >"$log" 2>&1; then
    echo "$label unexpectedly passed" >&2; exit 1
  fi
  rg -qF "$expected" "$log" || { cat "$log" >&2; echo "$label failed incorrectly" >&2; exit 1; }
}

json_variant() {
  local label=$1 relative=$2 filter=$3 expected=$4 root="$TMP/$1"
  clone_fixture "$root"
  jq "$filter" "$root/advanced-color/$relative" >"$root/value.json"
  mv "$root/value.json" "$root/advanced-color/$relative"
  expect_failure "$label" "$root" "$expected"
}

video_variant() {
  local label=$1 mode=$2 expected=$3 root="$TMP/$1"
  clone_fixture "$root"
  if [[ $mode == label-only ]]; then
    make_advanced_color_label_only_video "$root/advanced-color"
    assert_unique_frames "$root/advanced-color/rendered/preview.mp4" 6 \
      0.5 1.5 2.5 3.5 5 7
  else
    make_advanced_color_video "$root/advanced-color" "$mode"
  fi
  expect_failure "$label" "$root" "$expected"
}

VALID="$TMP/valid"
make_advanced_color_fixture "$VALID"
PREVIEW_ROOT=$VALID check_advanced_color_stage_evidence

json_variant author_stage_order project/project.veac.json \
  '(.. | objects | select(.id? == "itm_full") | .visual.color_pipeline.stages) |= [.[1],.[0],.[2],.[3],.[4],.[5]]' \
  'authoring stage contract failed'
json_variant preview_stage_order project/project.preview.veac.json \
  '(.. | objects | select(.id? == "itm_full") | .visual.color_pipeline.stages) |= [.[0],.[2],.[1],.[3],.[4],.[5]]' \
  'preview stage contract failed'
json_variant plan_stage_order plans/preview/out_preview.json \
  '(.. | objects | select(.id? == "itm_full") | .visual.color_pipeline.stages) |= reverse' \
  'plan stage contract failed'
json_variant author_lut_kind project/project.veac.json \
  '(.project.materials[] | select(.id == "med_cinematic") | .kind) = "lut1d"' \
  'authoring stage contract failed'
json_variant plan_missing_input plans/preview/out_preview.json \
  'del(.inputs[] | select(.id == "pin_cinematic"))' \
  'plan stage contract failed'
json_variant plan_input_kind plans/preview/out_preview.json \
  '(.inputs[] | select(.id == "pin_tone-curve") | .kind.material_kind) = "lut3d"' \
  'plan stage contract failed'
json_variant wrong_window project/project.veac.json \
  '(.. | objects | select(.id? == "itm_tone") | .record_range.start.value) = 5000' \
  'authoring stage contract failed'
json_variant opaque_source project/project.veac.json \
  '(.. | objects | select(.id? == "itm_reference") | .source.generator.gradient.stops[].color.alpha) = 255' \
  'authoring stage contract failed'
json_variant author_placement project/project.veac.json \
  '(.. | objects | select(.authorship.logical_path?[-1] == "hsl") | .visual.placement) = {type:"absolute",position:{x:{unit:"pixels",value:0},y:{unit:"pixels",value:0}}}' \
  'authoring stage contract failed'

video_variant identical_rendered_stages same-stages 'expected 6 distinct label-free mechanism regions'
video_variant label_only_stages label-only 'expected 6 distinct label-free mechanism regions'
video_variant opaque_rendered_gradient opaque 'alpha over dark/light backgrounds is wrong'
video_variant wrong_rendered_composite bad-composite 'alpha over dark/light backgrounds is wrong'

quarter="$TMP/quarter-footprint"
clone_fixture "$quarter"
make_quarter_footprint_video "$quarter/advanced-color/rendered/preview.mp4" "$quarter/value.mp4"
mv "$quarter/value.mp4" "$quarter/advanced-color/rendered/preview.mp4"
expect_failure quarter-footprint "$quarter" '覆盖右侧无标签 ROI'

echo 'advanced-color render evidence contract tests passed'
