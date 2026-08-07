#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
# shellcheck disable=SC2034
SCRIPT_DIR="$ROOT/scripts"
TMP=$(mktemp -d "${TMPDIR:-/tmp}/veac-masks-contracts.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
for module in common pixels composition-masks composition; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/masks-render-fixtures.sh"

clone_fixture() { mkdir -p "$1"; cp -R "$VALID/masks-and-mattes" "$1/masks-and-mattes"; }
expect_failure() {
  local label=$1 root=$2 expected=$3 log="$TMP/$1.log"
  if (PREVIEW_ROOT=$root check_mask_evidence) >"$log" 2>&1; then
    echo "$label unexpectedly passed" >&2; exit 1
  fi
  rg -qF "$expected" "$log" || {
    cat "$log" >&2; echo "$label failed incorrectly" >&2; exit 1;
  }
}
json_variant() {
  local label=$1 relative=$2 filter=$3 expected=$4 root="$TMP/$1"
  clone_fixture "$root"
  jq "$filter" "$root/masks-and-mattes/$relative" >"$root/value.json"
  mv "$root/value.json" "$root/masks-and-mattes/$relative"
  expect_failure "$label" "$root" "$expected"
}
video_variant() {
  local label=$1 mode=$2 expected=$3 root="$TMP/$1"
  clone_fixture "$root"; make_masks_video "$root/masks-and-mattes" "$mode"
  expect_failure "$label" "$root" "$expected"
}

VALID="$TMP/valid"
make_masks_fixture "$VALID"
PREVIEW_ROOT=$VALID check_mask_evidence
json_variant absolute_author project/project.veac.json \
  '(..|objects|select(.id?=="itm_circle")|.visual.placement)={"type":"absolute","position":{
    "x":{"unit":"pixels","value":0},"y":{"unit":"pixels","value":0}}}' \
  'authoring centered mask contract failed'
json_variant off_center_preview project/project.preview.veac.json \
  '(..|objects|select(.id?=="itm_rectangle")|.visual.transform.anchor)={"x":0,"y":0}' \
  'preview centered mask contract failed'
json_variant framed_plan plans/preview/out_preview.json \
  '(..|objects|select(.id?=="itm_combined")|.visual.frame)={"fit":"contain"}' \
  'plan centered mask contract failed'
video_variant off_center_pixels off-center 'clipped, off-center, or label-only'
video_variant label_only_pixels label-only 'clipped, off-center, or label-only'

echo 'masks-and-mattes render evidence contract tests passed'
