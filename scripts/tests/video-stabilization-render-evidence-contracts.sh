#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
TMP=$(mktemp -d "${TMPDIR:-/tmp}/veac-stabilization-contracts.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
for module in common pixels video-stabilization; do
  # shellcheck source=/dev/null
  source "$ROOT/scripts/example-render-evidence/$module.sh"
done
# shellcheck source=/dev/null
source "$ROOT/scripts/tests/video-stabilization-render-fixtures.sh"

clone_fixture() { mkdir -p "$1"; cp -R "$VALID/video-effects" "$1/video-effects"; }
expect_failure() {
  local label=$1 root=$2 expected=$3 log="$TMP/$1.log"
  if (PREVIEW_ROOT=$root check_video_stabilization_evidence) >"$log" 2>&1; then
    echo "$label unexpectedly passed" >&2; exit 1
  fi
  rg -qF "$expected" "$log" || { cat "$log" >&2; echo "$label failed incorrectly" >&2; exit 1; }
}
json_variant() {
  local label=$1 relative=$2 filter=$3 expected=$4 root="$TMP/$1"
  clone_fixture "$root"; jq "$filter" "$root/video-effects/$relative" >"$root/value.json"
  mv "$root/value.json" "$root/video-effects/$relative"; expect_failure "$label" "$root" "$expected"
}
video_variant() {
  local label=$1 mode=$2 expected=$3 root="$TMP/$1"
  clone_fixture "$root"; make_stabilization_video "$root/video-effects" "$mode"
  expect_failure "$label" "$root" "$expected"
}

VALID="$TMP/valid"
make_stabilization_fixture "$VALID"
PREVIEW_ROOT=$VALID check_video_stabilization_evidence

json_variant different_source project/project.veac.json \
  '(.. | objects | select(.id? == "itm_stabilize-after") | .source.material_id) = "med_other"' \
  'authoring stabilization contract failed'
json_variant different_source_start project/project.veac.json \
  '(.. | objects | select(.id? == "itm_stabilize-after") | .source_mapping.time_map.source_start.value) = 100' \
  'authoring stabilization contract failed'
json_variant missing_effect project/project.preview.veac.json \
  '(.. | objects | select(.id? == "itm_stabilize-after") | .effects) = []' \
  'preview stabilization contract failed'
json_variant disabled_effect project/project.preview.veac.json \
  '(.. | objects | select(.id? == "itm_stabilize-after") | .effects[0].enabled) = false' \
  'preview stabilization contract failed'
json_variant missing_plan_effect plans/preview/out_preview.json \
  '(.. | objects | select(.id? == "itm_stabilize-after") | .effects) = []' \
  'plan stabilization contract failed'

video_variant unchanged_jitter same-jitter 'does not reduce grid motion'
video_variant stable_input stable-before 'before segment is already stable'
video_variant frozen_output frozen-after 'after segment is frozen'
video_variant shifted_frozen_output frozen-shifted 'after segment content is frozen'
video_variant missing_landmark missing-grid 'grid landmark is missing'

echo 'video stabilization render evidence contract tests passed'
