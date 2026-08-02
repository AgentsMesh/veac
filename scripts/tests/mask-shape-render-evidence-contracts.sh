#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/veac-mask-shape-contracts.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
for module in common pixels mask-shape-gallery; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/mask-shape-render-fixtures.sh"

clone_fixture() { mkdir -p "$1"; cp -R "$VALID/mask-shape-gallery" "$1/mask-shape-gallery"; }
expect_failure() {
  local label=$1 root=$2 expected=$3 log="$TMP/$1.log"
  if (PREVIEW_ROOT=$root check_mask_shape_gallery_evidence) >"$log" 2>&1; then
    echo "$label unexpectedly passed" >&2; exit 1
  fi
  rg -qF "$expected" "$log" || { cat "$log" >&2; echo "$label failed incorrectly" >&2; exit 1; }
}
video_variant() {
  local label=$1 mode=$2 expected=$3 root="$TMP/$1"
  clone_fixture "$root"; make_mask_shape_video "$root/mask-shape-gallery" "$mode"
  expect_failure "$label" "$root" "$expected"
}
json_variant() {
  local label=$1 relative=$2 filter=$3 expected=$4 root="$TMP/$1"
  clone_fixture "$root"
  jq "$filter" "$root/mask-shape-gallery/$relative" >"$root/value.json"
  mv "$root/value.json" "$root/mask-shape-gallery/$relative"
  expect_failure "$label" "$root" "$expected"
}

VALID="$TMP/valid"
make_mask_shape_fixture "$VALID"
PREVIEW_ROOT=$VALID check_mask_shape_gallery_evidence

video_variant heart_is_rectangle wrong-heart 'mask heart rendered topology is wrong'
video_variant star_is_circle wrong-star 'mask star rendered topology is wrong'
video_variant linear_is_full_frame wrong-linear 'mask linear rendered half-plane is wrong'
video_variant mirror_is_one_sided wrong-mirror 'mask mirror rendered center band is wrong'
json_variant wrong_author_window project/project.veac.json \
  '(.. | objects | select(.id? == "itm_star") | .record_range.start.value) = 900' \
  'authoring mask contract failed'
json_variant wrong_preview_shape project/project.preview.veac.json \
  '(.. | objects | select(.id? == "itm_heart") | .visual.masks[0].shape.type) = "rectangle"' \
  'preview mask contract failed'
json_variant wrong_plan_rotation plans/preview/out_preview.json \
  '(.. | objects | select(.id? == "itm_mirror") | .visual.masks[0].rotation_degrees.value) = 12' \
  'plan mask contract failed'

echo 'mask-shape render evidence contract tests passed'
